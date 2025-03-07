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

use hypersonic::uasm;
use zkaluvm::alu::CompiledLib;

use crate::{G_DETAILS, G_NAME, G_PRECISION, G_TICKER, O_AMOUNT};

/// Checks globals defining assent specification to be present and contain correct state type.
///
/// NB: Doesn't check that the values of that globals fulfill ASCII criteria (like length, use of
/// specific chars etc.). This is not enforced by consensus here and instead, the contract will just
/// fail to read it's state under RGB20, 21, 25 or other interface.
///
/// # Input
///
/// Procedure takes no input
///
/// # Output
///
/// `EB` contains the value of [`G_PRECISION`].
///
/// # Reset registers
///
/// `E1`, `EA`, `EC`-`ED`
pub const FN_ASSET_SPEC: u16 = 0;

/// Sums input owned state
///
/// # Input
///
/// - `EE`: value expected to be present in the third field element (`EC` register returned from
///   `ldi`). If the value of register is `None` and `EC` is not `None`, the procedure fails.
///   Otherwise, if the value in `EC` and `EE` is not equal, the procedure skips that input.
///
/// # Output
///
/// `E1` contains the sum of inputs.
///
/// # Reset registers
///
/// `EA`-`ED`, `E8`.
pub const FN_SUM_INPUTS: u16 = 1;

/// Sums output owned state
///
/// # Input
///
/// - `EE`: value expected to be present in the third field element (`EC` register returned from
///   `ldi`). If the value of register is `None` and `EC` is not `None`, the procedure fails.
///   Otherwise, if the value in `EC` and `EE` is not equal, the procedure skips that input.
///
/// # Output
///
/// `E2` contains the sum of inputs.
///
/// # Reset registers
///
/// `EA`-`ED`, `E8`.
pub const FN_SUM_OUTPUTS: u16 = 3;

pub(self) const LOOP_INPUTS: u16 = 2;
pub(self) const LOOP_OUTPUTS: u16 = 4;

pub fn shared_lib() -> CompiledLib {
    assert_eq!(O_AMOUNT, G_NAME);
    assert_eq!(G_TICKER, G_DETAILS);

    let mut code = uasm! {
    // .proc FN_ASSET_SPEC
        nop                     ;// Marks start of routine / entry point / goto target

        // There must be no inputs
        cknxi   :immutable;
        not     CO;
        chk     CO;
        cknxi   :destructible;
        not     CO;
        chk     CO;

        ldo     :immutable      ;// Read first global state - name
        chk     CO              ;// - it must exist
        mov     E1, :G_TICKER   ;// - set E1 to the field element representing owned value (also global asset name)
        eq      EA, E1          ;// - it must have correct state type
        chk     CO              ;// - - or fail otherwise

        ldo     :immutable      ;// Read second global state (ticker for RGB20, details for RGB25)
        chk     CO              ;// - it must exist
        mov     E1, :G_NAME     ;// - set E1 to field element representing global asset ticker (or details)
        eq      EA, E1          ;// - it must have correct state type
        chk     CO              ;// - - or fail otherwise

        ldo     :immutable      ;// Third global state - precision
        chk     CO              ;// - it must exist
        mov     E1, :G_PRECISION;// - set E1 to field element representing global fractions
        eq      EA, E1          ;// - it must have correct state type
        chk     CO              ;// - - or fail otherwise
        test    EC              ;// - there must be no other field elements than in EC in the precision
        not     CO;
        chk     CO              ;// - or fail otherwise
        test    ED              ;// - there must be no other field elements than in ED in the precision
        not     CO;
        chk     CO              ;// - or fail otherwise

        // Clear up
        clr     E1;
        clr     EA;
        clr     EC;
        clr     ED;

        ret;

    // .proc FN_SUM_INPUTS
        nop                     ;// Marks start of routine / entry point / goto target
        mov     E1, 0           ;// Set initial sum to zero
        mov     E8, :O_AMOUNT   ;// Set EE to the field element representing owned value
        rsti    :destructible   ;// Start iteration over inputs

    // .loop LOOP_INPUTS
        nop;
        ldi     :destructible   ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, E8          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EE              ;// Is `EE` set to none?
        jif     CO, +7          ;// - branch to enforce `EC` to be none as well

        eq      EC, EE          ;// ensure EC value equals to EE
        jif     CO, :LOOP_INPUTS;// - read next input otherwise
        jmp     +4              ;// process to normal flow

        eq      EC, EE          ;// ensure EC is not set
        not     CO;
        chk     CO              ;// fail if not

        test    ED              ;// ensure ED is not set
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 64:bits     ;// ensure the value fits in u64
        chk     CO              ;// fail if not
        add     E1, EB          ;// add input to input accumulator
        fits    E1, 64:bits     ;// ensure we do not overflow
        chk     CO              ;// fail if not

        jmp     :LOOP_INPUTS    ;// loop

    // .proc FN_SUM_OUTPUTS
        nop                     ;// Marks start of routine / entry point / goto target
        mov     E2, 0           ;// Set initial sum to zero
        mov     E8, :O_AMOUNT   ;// Set EE to the field element representing owned value
        rsto    :destructible   ;// Start iteration over outputs

    // .loop LOOP_OUTPUTS
        nop;
        ldo     :destructible   ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, E8          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EE              ;// Is `EE` set to none?
        jif     CO, +9          ;// - branch to enforce `EC` to be none as well

        eq      EC, EE          ;// ensure EC value equals to EE
        jif     CO, :LOOP_OUTPUTS;// - read next input otherwise
        jmp     +6              ;// process to normal flow

        test    EC              ;// ensure EC is not set
        not     CO;
        chk     CO              ;// fail if not

        test    ED              ;// ensure ED is not set
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 64:bits     ;// ensure the value fits in u64
        chk     CO              ;// fail if not
        add     E2, EB          ;// add input to input accumulator
        fits    E2, 64:bits     ;// ensure we do not overflow
        chk     CO              ;// fail if not

        jmp     :LOOP_OUTPUTS   ;// loop
    };

    CompiledLib::compile(&mut code, &[]).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use amplify::num::u256;
    use hypersonic::{AuthToken, Instr, StateCell, StateData, StateValue, VmContext};
    use strict_types::StrictDumb;
    use zkaluvm::alu::{CoreConfig, CoreExt, Lib, LibId, Supercore, Vm};
    use zkaluvm::{GfaConfig, GfaCore, RegE, FIELD_ORDER_SECP};

    const CONFIG: CoreConfig = CoreConfig {
        halt: true,
        complexity_lim: Some(10_000_000_000),
    };

    fn harness() -> (CompiledLib, Vm<Instr<LibId>>, impl Fn(LibId) -> Option<Lib>) {
        let vm = Vm::<Instr<LibId>>::with(
            CONFIG,
            GfaConfig {
                field_order: FIELD_ORDER_SECP,
            },
        );
        fn resolver(id: LibId) -> Option<Lib> {
            let shared = shared_lib();
            if shared.as_lib().lib_id() == id {
                return Some(shared.into_lib());
            }
            panic!("Unknown library: {id}");
        }
        (shared_lib(), vm, resolver)
    }

    #[test]
    fn genesis_empty() {
        let context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        let mut context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
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
            ][..],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_TICKER, 0u8)],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_DETAILS, 0u8)],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_wrong_order() {
        let mut context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(G_NAME, 1u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ][..],
            &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_NAME, 1u8),
            ],
            &[
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_NAME, 1u8),
            ],
            &[
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_NAME, 1u8),
                StateData::new(G_TICKER, 0u8),
            ],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_correct() {
        let context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[],
            immutable_output: &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_NAME, 1u8),
                StateData::new(G_PRECISION, 18_u8),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
            .is_ok();
        assert!(res);
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
                .map(|val| StateValue::new(O_AMOUNT, *val))
                .collect::<Vec<_>>();
            let context = VmContext {
                read_once_input: input.as_slice(),
                immutable_input: &[],
                read_once_output: &[],
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_SUM_INPUTS), &context, resolver)
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
                .map(|val| StateValue::new(O_AMOUNT, *val))
                .collect::<Vec<_>>();
            let context = VmContext {
                read_once_input: input.as_slice(),
                immutable_input: &[],
                read_once_output: &[],
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_SUM_INPUTS), &context, resolver)
                .is_ok();
            let gfa: GfaCore = vm.core.cx.subcore();
            assert_eq!(gfa.get(RegE::E1).unwrap().to_u256(), u256::from(sum));
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
                read_once_input: &[],
                immutable_input: &[],
                read_once_output: output.as_slice(),
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(FN_SUM_OUTPUTS), &context, resolver)
                .is_ok();
            let gfa: GfaCore = vm.core.cx.subcore();
            assert_eq!(gfa.get(RegE::E2).unwrap().to_u256(), u256::from(sum));
            assert!(res);
        }
    }
}
