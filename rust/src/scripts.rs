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

use crate::{
    GLOBAL_ASSET_DETAILS, GLOBAL_ASSET_NAME, GLOBAL_PRECISION, GLOBAL_SUPPLY, GLOBAL_TICKER,
    OWNED_VALUE,
};

pub fn success() -> CompiledLib {
    let mut code = uasm! {
        nop;
        stop;
    };
    CompiledLib::compile(&mut code).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

pub const FUNGIBLE_LIB_ID: &str = "";

pub const SUB_FUNGIBLE_ISSUE_RGB20: u16 = 0;
pub const SUB_FUNGIBLE_ISSUE_RGB25: u16 = 1;
pub const SUB_FUNGIBLE_TRANSFER: u16 = 3;

const SUB_FUNGIBLE_GENESIS: u16 = 2;
const SUB_FUNGIBLE_SUM_INPUTS: u16 = 4;
const SUB_FUNGIBLE_SUM_OUTPUTS: u16 = 5;

pub fn fungible() -> CompiledLib {
    assert_eq!(OWNED_VALUE, GLOBAL_ASSET_NAME);

    let mut code = uasm! {
    // .routine SUB_FUNGIBLE_ISSUE_RGB20
        nop                               ;// Marks start of routine / entry point / goto target
        mov     EF, :GLOBAL_TICKER        ;// Set EF to field element representing global ticker
        jmp     :SUB_FUNGIBLE_GENESIS     ;// Pass to the generic genesis validation routine

    // .routine SUB_FUNGIBLE_ISSUE_RGB25
        nop                               ;// Marks start of routine / entry point / goto target
        mov     EF, :GLOBAL_ASSET_DETAILS ;// Set EF to field element representing global details
        jmp     :SUB_FUNGIBLE_GENESIS     ;// Pass to the generic genesis validation routine

    // .routine SUB_FUNGIBLE_GENESIS
        nop                     ;// Marks start of routine / entry point / goto target
        // Set initial values
        mov     EE, :OWNED_VALUE;// Set EE to the field element representing owned value (also global asset name)
        mov     EG, :GLOBAL_PRECISION      ;// Set EF to field element representing global precision
        mov     EH, :GLOBAL_SUPPLY         ;// Set EF to field element representing global circulation
        mov     E2, 0           ;// E3 will contain sum of outputs
        // Validate verbose globals
        ldo     :immutable      ;// Read first global state - name
        chk     CO              ;// It must exist
        eq      EA, EE          ;// It must have correct state type
        chk     CO              ;// Or fail otherwise
        ldo     :immutable      ;// Read second global state (ticker for RGB20, details for RGB25)
        chk     CO              ;// It must exist
        eq      EA, EF          ;// It must have correct state type
        chk     CO              ;// Or fail otherwise
        ldo     :immutable      ;// Read second global state - precision
        chk     CO              ;// It must exist
        eq      EA, EG          ;// It must have correct state type
        chk     CO              ;// Or fail otherwise
        // Validate circulating supply
        ldo     :immutable      ;// Read second global state - circulating supply
        chk     CO              ;// It must exist
        eq      EA, EH          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        test    EB              ;// It must be set
        chk     CO              ;// Or we should fail
        mov     E1, EB          ;// Save supply
        test    EC              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need test to fail)
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need test to fail)
        chk     CO              ;// fail if not
        call    :SUB_FUNGIBLE_SUM_OUTPUTS   ;// Compute sum of outputs
        eq      E1, E2          ;// check that circulating supply equals to the sum of outputs
        chk     CO              ;// fail if not
        ret;

    // .routine SUB_FUNGIBLE_TRANSFER
        // Set initial values
        nop                     ;// Marks start of routine / entry point / goto target
        mov     EE, :OWNED_VALUE;// Set EE to the field element representing owned value
        mov     E1, 0           ;// E1 will contain sum of inputs
        mov     E2, 0           ;// E2 will contain sum of outputs
        // Verify owned state
        call    :SUB_FUNGIBLE_SUM_INPUTS    ;// Compute sum of inputs
        call    :SUB_FUNGIBLE_SUM_OUTPUTS   ;// Compute sum of outputs
        eq      E1, E2          ;// check that the sum of inputs equals sum of outputs
        chk     CO              ;// fail if not
        // Verify that no global state is assigned
        ldo     :immutable      ;// Try to iterate over global state
        not     CO              ;// Invert result (we need NO state as a Success)
        chk     CO              ;// Fail if there is a global state
        ret;

    // .routine SUB_FUNGIBLE_SUM_INPUTS
        // Start iterations:
        nop                     ;// Marks start of routine / entry point / goto target

        ldi     :destructible   ;// load next state value
        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, EE          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EC              ;// ensure other field elements are empty
        not     CO;
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 8:bits      ;// ensure the value fits in 8 bits
        add     E1, EB          ;// add input to input accumulator
        fits    E1, 8:bits      ;// ensure we do not overflow
        jmp     :SUB_FUNGIBLE_SUM_INPUTS    ;// loop

    // .routine SUB_FUNGIBLE_SUM_OUTPUTS
        // Start iterations:
        nop                     ;// Mark the start of the routine
        ldo     :destructible   ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, EE          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EC              ;// ensure other field elements are empty
        not     CO;
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 8:bits      ;// ensure the value fits in 8 bits
        add     E2, EB          ;// add input to input accumulator
        fits    E2, 8:bits      ;// ensure we do not overflow
        jmp     :SUB_FUNGIBLE_SUM_OUTPUTS   ;// loop
    };

    CompiledLib::compile(&mut code).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypersonic::{AuthToken, Instr, StateCell, StateData, StateValue, VmContext};
    use strict_types::StrictDumb;
    use zkaluvm::alu::{CoreConfig, Lib, LibId, Vm};
    use zkaluvm::{GfaConfig, FIELD_ORDER_SECP};

    const CONFIG: CoreConfig = CoreConfig {
        halt: true,
        complexity_lim: Some(180_000_000),
    };

    fn harness() -> (CompiledLib, Vm<Instr<LibId>>, impl Fn(LibId) -> Option<Lib>) {
        let vm = Vm::<Instr<LibId>>::with(
            CONFIG,
            GfaConfig {
                field_order: FIELD_ORDER_SECP,
            },
        );
        fn resolver(id: LibId) -> Option<Lib> {
            let lib = fungible();
            assert_eq!(id, lib.as_lib().lib_id());
            Some(lib.into_lib())
        }
        (fungible(), vm, resolver)
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
            .exec(lib.routine(SUB_FUNGIBLE_ISSUE_RGB20), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        let mut context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
                data: StateValue::new(OWNED_VALUE, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(GLOBAL_TICKER, 0u8),
                StateData::new(GLOBAL_PRECISION, 18_u8),
                StateData::new(GLOBAL_SUPPLY, 1000_u64),
            ][..],
            &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_PRECISION, 18_u8),
                StateData::new(GLOBAL_SUPPLY, 1000_u64),
            ],
            &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_TICKER, 0u8),
                StateData::new(GLOBAL_SUPPLY, 1000_u64),
            ],
            &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_TICKER, 0u8),
                StateData::new(GLOBAL_PRECISION, 18_u8),
            ],
            &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_TICKER, 0u8),
            ],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(SUB_FUNGIBLE_ISSUE_RGB20), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_missing_owned() {
        let context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[],
            immutable_output: &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_TICKER, 0u8),
                StateData::new(GLOBAL_PRECISION, 18_u8),
                StateData::new(GLOBAL_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(SUB_FUNGIBLE_ISSUE_RGB20), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_supply_mismatch() {
        let context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
                data: StateValue::new(OWNED_VALUE, 1001_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_TICKER, 0u8),
                StateData::new(GLOBAL_PRECISION, 18_u8),
                StateData::new(GLOBAL_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(SUB_FUNGIBLE_ISSUE_RGB20), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_correct() {
        let context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
                data: StateValue::new(OWNED_VALUE, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(GLOBAL_ASSET_NAME, 0u8),
                StateData::new(GLOBAL_TICKER, 0u8),
                StateData::new(GLOBAL_PRECISION, 18_u8),
                StateData::new(GLOBAL_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(SUB_FUNGIBLE_ISSUE_RGB20), &context, resolver)
            .is_ok();
        assert!(res);
    }

    fn transfer_harness(inp: &[&[u64]], out: &[&[u64]], should_success: bool) {
        let inputs = inp.into_iter().map(|vals| {
            vals.into_iter()
                .map(|val| StateValue::new(OWNED_VALUE, *val))
                .collect::<Vec<_>>()
        });
        let lock = None;
        let auth = AuthToken::strict_dumb();
        let outputs = out.into_iter().map(|vals| {
            vals.into_iter()
                .map(|val| StateCell {
                    data: StateValue::new(OWNED_VALUE, *val),
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
                read_once_input: input.as_slice(),
                immutable_input: &[],
                read_once_output: output.as_slice(),
                immutable_output: &[],
            };
            let res = vm
                .exec(lib.routine(SUB_FUNGIBLE_TRANSFER), &context, resolver)
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
