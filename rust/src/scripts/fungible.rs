// RGB issuers
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2019-2025 by Dr Maxim Orlovsky <orlovsky@pandoraprime.ch>
// Written in 2024-2025 by Dr Maxim Orlovsky <orlovsky@pandoraprime.ch>
//
// Copyright (C) 2019-2022 Pandora Core SA, Neuchatel, Switzerland.
// Copyright (C) 2022-2025 Pandora Prime Inc, Neuchatel, Switzerland.
// Copyright (C) 2019-2025 Dr Maxim Orlovsky.
// All rights under the above copyrights are reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use amplify::num::u256;
use hypersonic::uasm;
use zkaluvm::alu::CompiledLib;

use super::{shared_lib, FN_ASSET_SPEC, FN_GLOBAL_ABSENT};
use crate::{G_SUPPLY, O_AMOUNT};

pub const FN_FUNGIBLE_ISSUE: u16 = 0;
pub const FN_FUNGIBLE_TRANSFER: u16 = 1;

/// Sum input owned state
///
/// # Input
///
/// None
///
/// # Output
///
/// `E2` contains the sum of inputs.
///
/// # Reset registers
///
/// `EA`-`ED`.
///
/// # Side effects
///
/// Extinguishes the input destructible state iterator
pub const FN_FUNGIBLE_SUM_INPUTS: u16 = 2;

/// Sum output owned state
///
/// # Input
///
/// None
///
/// # Output
///
/// `E3` contains the sum of outputs.
///
/// # Reset registers
///
/// `EA`-`ED`, `E8`.
///
/// # Side effects
///
/// Extinguishes the output destructible state iterator
pub const FN_FUNGIBLE_SUM_OUTPUTS: u16 = 4;

pub const ERRNO_PRECISION_OVERFLOW: u256 = u256::from_inner([1, 1, 0, 0]);
pub const ERRNO_NO_ISSUED: u256 = u256::from_inner([2, 1, 0, 0]);
pub const ERRNO_SUM_ISSUE_MISMATCH: u256 = u256::from_inner([3, 1, 0, 0]);
pub const ERRNO_UNEXPECTED_GLOBAL: u256 = u256::from_inner([4, 1, 0, 0]);
pub const ERRNO_SUM_MISMATCH: u256 = u256::from_inner([5, 1, 0, 0]);
pub const ERRNO_UNEXPECTED_OWNED_TYPE_IN: u256 = u256::from_inner([6, 1, 0, 0]);
pub const ERRNO_INVALID_BALANCE_IN: u256 = u256::from_inner([7, 1, 0, 0]);
pub const ERRNO_UNEXPECTED_OWNED_TYPE_OUT: u256 = u256::from_inner([8, 1, 0, 0]);
pub const ERRNO_INVALID_BALANCE_OUT: u256 = u256::from_inner([9, 1, 0, 0]);

pub fn fungible() -> CompiledLib {
    const LOOP_INPUTS: u16 = 3;
    const LOOP_OUTPUTS: u16 = 5;

    let shared = shared_lib().into_lib().lib_id();

    let mut code = uasm! {
     routine FN_FUNGIBLE_ISSUE:
        call    shared, FN_ASSET_SPEC;// Call asset check

        put     E1, ERRNO_PRECISION_OVERFLOW; // Set error code for the case of failure
        fits    E4, 8.bits;     // The precision must fit into a byte
        chk     CO;             // - or fail otherwise

        // Validate circulating supply
        put     E1, ERRNO_NO_ISSUED; // Set error code for the case of failure
        ldo     immutable;      // Read last global state - circulating supply
        chk     CO;             // It must exist
        put     E8, G_SUPPLY;   // Load supply type
        eq      EA, E8;         // It must have a correct state type
        chk     CO;             // Or fail otherwise
        test    EB;             // It must be set
        chk     CO;             // Or we should fail
        mov     E2, EB;         // Save supply
        test    EC;             // ensure other field elements are empty
        not     CO;             // invert CO value (we need the test to fail)
        chk     CO;             // fail if not
        test    ED;             // ensure other field elements are empty
        not     CO;             // invert CO value (we need the test to fail)
        chk     CO;             // fail if not

        // Validate that the issued amount is equal to the sum of the outputs
        put     E3, 0;          // E3 will contain the sum of outputs
        call    FN_FUNGIBLE_SUM_OUTPUTS;// Compute a sum of outputs
        put     E1, ERRNO_SUM_ISSUE_MISMATCH; // Set error code for the case of failure
        eq      E2, E3;         // check that circulating supply equals to the sum of outputs
        chk     CO;             // fail if not

        // Check there is no more global state
        put     E1, ERRNO_UNEXPECTED_GLOBAL; // Set error code for the case of failure
        ldo     immutable;
        not     CO;
        chk     CO;

        clr     E1;             // Clear the error code
        ret;

     routine FN_FUNGIBLE_TRANSFER:
        // Verify that no global state is defined
        call    shared, FN_GLOBAL_ABSENT;

        // Verify owned state
        call    FN_FUNGIBLE_SUM_INPUTS; // Compute a sum of inputs into E2
        call    FN_FUNGIBLE_SUM_OUTPUTS; // Compute a sum of outputs into E3
        put     E1, ERRNO_SUM_MISMATCH; // Set error code for the case of failure
        // TODO: Check the sum is not zero
        eq      E2, E3;         // check that the sum of inputs equals the sum of outputs
        chk     CO;             // fail if not

        clr     E1;             // Clear the error code
        ret;

     proc FN_FUNGIBLE_SUM_INPUTS:
        put     E2, 0;          // Set initial sum to zero
        put     EH, O_AMOUNT;   // Set EH to the field element representing the owned value
        rsti    destructible;   // Start iteration over inputs

     label LOOP_INPUTS:
        ldi     destructible;   // load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        put     E1, ERRNO_UNEXPECTED_OWNED_TYPE_IN; // Set error code for the case of failure
        eq      EA, EH;         // do we have a correct state type?
        chk     CO;             // fail if not

        put     E1, ERRNO_INVALID_BALANCE_IN; // Set error code for the case of failure
        eq      EC, EE;         // ensure EC is not set
        not     CO;
        chk     CO;             // fail if not

        test    ED;             // ensure ED is not set
        not     CO;
        chk     CO;             // fail if not

        fits    EB, 64.bits;    // ensure the value fits in u64
        chk     CO;             // fail if not
        add     E2, EB;         // add input to input accumulator
        fits    E2, 64.bits;    // ensure we do not overflow
        chk     CO;             // fail if not

        jmp     LOOP_INPUTS;    // loop

     proc FN_FUNGIBLE_SUM_OUTPUTS:
        put     E3, 0;          // Set initial sum to zero
        put     EH, O_AMOUNT;   // Set EH to the field element representing the owned value
        rsto    destructible;   // Start iteration over outputs

     label LOOP_OUTPUTS:
        ldo     destructible;   // load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        put     E1, ERRNO_UNEXPECTED_OWNED_TYPE_OUT; // Set error code for the case of failure
        eq      EA, EH;         // do we have a correct state type?
        chk     CO;             // fail if not

        put     E1, ERRNO_INVALID_BALANCE_OUT; // Set error code for the case of failure
        test    EC;             // ensure EC is not set
        not     CO;
        chk     CO;             // fail if not

        test    ED;             // ensure ED is not set
        not     CO;
        chk     CO;             // fail if not

        fits    EB, 64.bits;    // ensure the value fits in u64
        chk     CO;             // fail if not
        add     E3, EB;         // add input to input accumulator
        fits    E3, 64.bits;    // ensure we do not overflow
        chk     CO;             // fail if not

        jmp     LOOP_OUTPUTS;   // loop
    };

    CompiledLib::compile(&mut code, &[&shared_lib()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use amplify::num::u256;
    use hypersonic::{AuthToken, Input, Instr, StateCell, StateData, StateValue, VmContext};
    use strict_types::StrictDumb;
    use zkaluvm::alu::{CoreConfig, CoreExt, Lib, LibId, Supercore, Vm};
    use zkaluvm::{GfaConfig, GfaCore, RegE, FIELD_ORDER_SECP};

    use super::*;
    use crate::{G_NAME, G_PRECISION, G_SUPPLY, G_TICKER, O_AMOUNT};

    const CONFIG: CoreConfig = CoreConfig {
        halt: true,
        complexity_lim: Some(500_000_000),
    };

    fn harness() -> (CompiledLib, Vm<Instr<LibId>>, impl Fn(LibId) -> Option<Lib>) {
        let vm = Vm::<Instr<LibId>>::with(
            CONFIG,
            GfaConfig {
                field_order: FIELD_ORDER_SECP,
            },
        );
        fn resolver(id: LibId) -> Option<Lib> {
            let fungible = fungible();
            let shared = shared_lib();
            if fungible.as_lib().lib_id() == id {
                return Some(fungible.into_lib());
            }
            if shared.as_lib().lib_id() == id {
                return Some(shared.into_lib());
            }
            panic!("Unknown library: {id}");
        }
        (fungible(), vm, resolver)
    }

    const AMOUNTS_OVERFLOW: &[&[u64]] = &[
        &[u64::MAX, 1, 1],
        &[u64::MAX - 1, 2],
        &[u64::MAX, u64::MAX - 1],
        &[u64::MAX, u64::MAX - 1, 1],
        &[u64::MAX, u64::MAX],
        &[u64::MAX / 2 + 1, u64::MAX / 2 + 1],
        &[u64::MAX / 2, u64::MAX / 2, 2],
    ];
    const AMOUNTS_OK: &[&[u64]] = &[
        &[],
        &[0u64],
        &[1u64; 4],
        &[10u64; 100],
        &[u64::MAX - 1, 1],
        &[u64::MAX],
        &[u64::MAX / 2 - 1, u64::MAX / 2],
    ];

    #[test]
    fn sum_inputs_overflow() {
        for input in AMOUNTS_OVERFLOW {
            let (lib, mut vm, resolver) = harness();
            let input = input
                .into_iter()
                .map(|val| {
                    (
                        Input::strict_dumb(),
                        StateCell {
                            data: StateValue::new(O_AMOUNT, *val),
                            auth: strict_dumb!(),
                            lock: None,
                        },
                    )
                })
                .collect::<Vec<_>>();
            let context = VmContext {
                witness: none!(),
                destructible_input: input.as_slice(),
                immutable_input: &[],
                destructible_output: &[],
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_FUNGIBLE_SUM_INPUTS), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn sum_outputs_overflow() {
        for output in AMOUNTS_OVERFLOW {
            let (lib, mut vm, resolver) = harness();
            let output = output
                .into_iter()
                .map(|val| StateCell {
                    data: StateValue::new(O_AMOUNT, *val),
                    auth: AuthToken::strict_dumb(),
                    lock: None,
                })
                .collect::<Vec<_>>();
            let context = VmContext {
                witness: none!(),
                destructible_input: &[],
                immutable_input: &[],
                destructible_output: output.as_slice(),
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_FUNGIBLE_SUM_OUTPUTS), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn sum_inputs() {
        for input in AMOUNTS_OK {
            let (lib, mut vm, resolver) = harness();
            let sum = input.iter().sum::<u64>();
            let input = input
                .into_iter()
                .map(|val| {
                    (
                        Input::strict_dumb(),
                        StateCell {
                            data: StateValue::new(O_AMOUNT, *val),
                            auth: strict_dumb!(),
                            lock: None,
                        },
                    )
                })
                .collect::<Vec<_>>();
            let context = VmContext {
                witness: none!(),
                destructible_input: input.as_slice(),
                immutable_input: &[],
                destructible_output: &[],
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_FUNGIBLE_SUM_INPUTS), &context, resolver)
                .is_ok();
            let gfa: GfaCore = vm.core.cx.subcore();
            assert_eq!(gfa.get(RegE::E2).unwrap().to_u256(), u256::from(sum));
            assert!(res);
        }
    }

    #[test]
    fn sum_outputs() {
        let lock = None;
        let auth = AuthToken::strict_dumb();
        for output in AMOUNTS_OK {
            let (lib, mut vm, resolver) = harness();
            let sum = output.iter().sum::<u64>();
            let output = output
                .into_iter()
                .map(|val| StateCell {
                    data: StateValue::new(O_AMOUNT, *val),
                    auth,
                    lock,
                })
                .collect::<Vec<_>>();
            let context = VmContext {
                witness: none!(),
                destructible_input: &[],
                immutable_input: &[],
                destructible_output: output.as_slice(),
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_FUNGIBLE_SUM_OUTPUTS), &context, resolver)
                .is_ok();
            let gfa: GfaCore = vm.core.cx.subcore();
            assert_eq!(gfa.get(RegE::E3).unwrap().to_u256(), u256::from(sum));
            assert!(res);
        }
    }

    #[test]
    fn genesis_empty() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_FUNGIBLE_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        let mut context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ][..],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_TICKER, 0u8)],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_FUNGIBLE_ISSUE), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_missing_owned() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_FUNGIBLE_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_supply_mismatch() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, 1001_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_FUNGIBLE_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_correct() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_FUNGIBLE_ISSUE), &context, resolver)
            .is_ok();
        assert!(res);
    }

    fn transfer_harness(inp: &[&[u64]], out: &[&[u64]], should_success: bool) {
        let inputs = inp.into_iter().map(|vals| {
            vals.into_iter()
                .map(|val| {
                    (
                        Input::strict_dumb(),
                        StateCell {
                            data: StateValue::new(O_AMOUNT, *val),
                            auth: strict_dumb!(),
                            lock: None,
                        },
                    )
                })
                .collect::<Vec<_>>()
        });
        let lock = None;
        let auth = AuthToken::strict_dumb();
        let outputs = out.into_iter().map(|vals| {
            vals.into_iter()
                .map(|val| StateCell {
                    data: StateValue::new(O_AMOUNT, *val),
                    auth,
                    lock,
                })
                .collect::<Vec<_>>()
        });
        for (input, output) in inputs.flat_map(|inp| {
            outputs
                .clone()
                .into_iter()
                .map(move |out| (inp.clone(), out))
        }) {
            let (lib, mut vm, resolver) = harness();
            let context = VmContext {
                witness: none!(),
                destructible_input: input.as_slice(),
                immutable_input: &[],
                destructible_output: output.as_slice(),
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_FUNGIBLE_TRANSFER), &context, resolver)
                .is_ok();
            if should_success {
                assert!(res);
            } else {
                assert!(!res);
            }
        }
    }

    #[test]
    fn transfer_deflation() {
        transfer_harness(&[&[1001], &[99, 900]], &[&[1000], &[100, 900]], false);
    }

    #[test]
    fn transfer_inflation() {
        transfer_harness(&[&[999], &[101, 900]], &[&[1000], &[100, 900]], false);
    }

    #[test]
    fn transfer_overflow() {
        transfer_harness(&[&[1]], &[&[u64::MAX - 1, 2]], false);
    }

    #[test]
    fn transfer_correct() {
        transfer_harness(&[&[1000], &[100, 900]], &[&[1000], &[100, 900]], true);
    }
}
