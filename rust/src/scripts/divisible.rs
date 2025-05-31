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

use super::{shared_lib, unique, FN_ASSET_SPEC, FN_GLOBAL_VERIFY_TOKEN};
use crate::{G_NFT, O_AMOUNT};

/// Sum input owned state for a specific token id.
///
/// # Input
///
// TODO: Use En register to match AluVM ABI
/// `EE` contains the token id which must match the second field element of the owned state.
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
pub const FN_NFT_SUM_INPUTS: u16 = 8;

/// Sum output owned state for a specific token id.
///
/// # Input
///
// TODO: Use En register to match AluVM ABI
/// `EE` contains the token id which must match the second field element of the owned state.
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
pub const FN_NFT_SUM_OUTPUTS: u16 = 10;

pub const FN_DIVISIBLE_TRANSFER: u16 = 6;

pub fn divisible() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();
    let uda = unique().into_lib().lib_id();

    const NEXT_TOKEN: u16 = 1;
    const END_TOKENS: u16 = 2;
    const NEXT_OWNED: u16 = 3;
    const NEXT_GLOBAL: u16 = 4;
    const END_TOKEN: u16 = 5;
    const LOOP_TOKEN: u16 = 7;
    const LOOP_INPUTS: u16 = 9;
    const LOOP_OUTPUTS: u16 = 11;

    // TODO: Check the correctness and completeness of the implementation
    let mut code = uasm! {
     proc FN_RGB21_ISSUE:
        call    shared, FN_ASSET_SPEC   ;// Call asset check
        fits    E4, 64.bits     ;// The precision must fit into u64
        chk     CO              ;// - or fail otherwise
        mov     E2, E4          ;// Save the precision value to match it against issued amounts

        // Validate global tokens and issued amounts
        put     E4, 0           ;// Start counter for tokens

     label NEXT_TOKEN:
        ldo     immutable      ;// Read fourth global state - token information
        jif     CO, END_TOKENS ;// Complete token validation if no more tokens left

        // Verify token spec
        call    uda, FN_GLOBAL_VERIFY_TOKEN   ;// Verify token spec
        // TODO: Ensure all token ids are unique

        // Check issued supply
        call    FN_NFT_SUM_OUTPUTS    ;// Sum outputs
        eq      E2, E3          ;// check that 'fractions' supply equals to the sum of outputs
        chk     CO              ;// fail if not
        put     E8, 1           ;// E8 will hold 1 as a constant for counter-increment operation
        add     E4, E8          ;// Increment token counter
        jmp     NEXT_TOKEN     ;// Process to the next token

        // Validate that owned tokens match the list of issued tokens
      label END_TOKENS:
        rsto    destructible   ;// Reset state iterator
      label NEXT_OWNED:
        rsto    immutable      ;// Reset state iterator
        ldo     destructible   ;// Iterate over tokens
        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;
        mov     E6, EC          ;// Save token id
        put     E5, 0           ;// Start counter
        put     E7, G_NFT   ;// Set E7 to field element representing token data
      label NEXT_GLOBAL:
        ldo     immutable      ;// Load global state
        jif     CO, END_TOKEN  ;// We've done
        eq      EA, E7          ;// It must has correct state type
        jif     CO, NEXT_GLOBAL;// If not, goto next global state
        eq      EB, E6          ;// Check if the token id match
        jif     CO, NEXT_GLOBAL;// Skip otherwise
        put     E8, 1           ;// E8 will hold 1 as a constant for counter increment operation
        add     E5, E8          ;// Increase counter
      label END_TOKEN:
        put     E8, 0           ;// E8 will hold 0 as a constant for `eq` operation
        eq      E5, E8          ;// Check that the token has allocations
        not     CO              ;// We need to invert CO so if no allocations we fail
        chk     CO              ;// Fail otherwise
        jmp     NEXT_OWNED     ;// Go to the next owned

      proc FN_DIVISIBLE_TRANSFER:
        // Verify that no global state is defined
        cknxo   immutable      ;// Try to iterate over global state
        not     CO              ;// Invert result (we need NO state as a Success)
        chk     CO              ;// Fail if there is a global state

        put     EE, O_AMOUNT;// Set EE to the field element representing owned value

        // For each token verify the sum of inputs equal sum of outputs
      label LOOP_TOKEN:
        ldi     immutable      ;// Iterate over tokens
        not     CO;
        jif     CO, +3;
        ret                     ;// Finish if no more tokens
        mov     EE, EB          ;// Save token id for FN_SUM_OUTPUTS
        call    FN_NFT_SUM_INPUTS     ;// Compute sum of inputs
        call    FN_NFT_SUM_OUTPUTS    ;// Compute sum of outputs
        eq      E2, E3          ;// check that the sum of inputs equals sum of outputs
        chk     CO              ;// fail if not
        jmp     LOOP_TOKEN     ;// Process to the next token

        // TODO: Check that no tokens not listed in global state are defined

     proc FN_SUM_INPUTS:
        put     E2, 0           ;// Set initial sum to zero
        put     EH, O_AMOUNT    ;// Set EH to the field element representing the owned value
        rsti    destructible    ;// Start iteration over inputs

     label LOOP_INPUTS:
        ldi     destructible    ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, EH          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        eq      EC, EE          ;// ensure EC value equals to EE
        jif     CO, LOOP_INPUTS ;// - read next input otherwise
        jmp     +4              ;// process to normal flow

        test    ED              ;// ensure ED is not set
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 64.bits     ;// ensure the value fits in u64
        chk     CO              ;// fail if not
        add     E2, EB          ;// add input to input accumulator
        fits    E2, 64.bits     ;// ensure we do not overflow
        chk     CO              ;// fail if not

        jmp     LOOP_INPUTS     ;// loop

     proc FN_SUM_OUTPUTS:
        put     E3, 0           ;// Set initial sum to zero
        put     EH, O_AMOUNT    ;// Set EH to the field element representing the owned value
        rsto    destructible    ;// Start iteration over outputs

     label LOOP_OUTPUTS:
        ldo     destructible    ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, EH          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        eq      EC, EE          ;// ensure EC value equals to EE
        jif     CO, LOOP_OUTPUTS;// - read next input otherwise
        jmp     +6              ;// process to normal flow

        test    ED              ;// ensure ED is not set
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 64.bits     ;// ensure the value fits in u64
        chk     CO              ;// fail if not
        add     E3, EB          ;// add input to input accumulator
        fits    E3, 64.bits     ;// ensure we do not overflow
        chk     CO              ;// fail if not

        jmp     LOOP_OUTPUTS    ;// loop
    };

    CompiledLib::compile(&mut code, &[&shared_lib(), &unique()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FN_RGB21_ISSUE, G_DETAILS, G_NAME, G_PRECISION, G_SUPPLY};
    use hypersonic::{AuthToken, Instr, StateCell, StateData, StateValue, VmContext};
    use strict_types::StrictDumb;
    use zkaluvm::alu::{CoreConfig, Lib, LibId, Vm};
    use zkaluvm::{GfaConfig, FIELD_ORDER_SECP};

    const CONFIG: CoreConfig = CoreConfig {
        halt: true,
        complexity_lim: Some(580_000_000),
    };

    fn harness() -> (CompiledLib, Vm<Instr<LibId>>, impl Fn(LibId) -> Option<Lib>) {
        let vm = Vm::<Instr<LibId>>::with(
            CONFIG,
            GfaConfig {
                field_order: FIELD_ORDER_SECP,
            },
        );
        fn resolver(id: LibId) -> Option<Lib> {
            let lib = divisible();
            let unique = unique();
            let shared = shared_lib();
            if lib.as_lib().lib_id() == id {
                return Some(lib.into_lib());
            }
            if unique.as_lib().lib_id() == id {
                return Some(unique.into_lib());
            }
            if shared.as_lib().lib_id() == id {
                return Some(shared.into_lib());
            }
            panic!("Unknown library: {id}");
        }
        (divisible(), vm, resolver)
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
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        const SUPPLY: u64 = 1000_u64;
        let mut context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, SUPPLY),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, SUPPLY),
            ][..],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, SUPPLY),
            ],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_SUPPLY, SUPPLY),
            ],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_SUPPLY, 0u8)],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
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
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
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
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    #[ignore]
    fn genesis_correct() {
        const TOKEN_ID: u64 = 0;
        const SUPPLY: u64 = 1000_u64;
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::Triple {
                    first: O_AMOUNT.into(),
                    third: TOKEN_ID.into(),
                    second: SUPPLY.into(),
                },
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, SUPPLY),
                StateData::new(G_SUPPLY, TOKEN_ID),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(res);
    }
}
