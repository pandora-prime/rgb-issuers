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

use super::{shared_lib, FN_ASSET_SPEC, FN_SUM_INPUTS, FN_SUM_OUTPUTS};
use crate::{G_SUPPLY, O_AMOUNT};

pub const FN_RGB21_ISSUE: u16 = 0;
pub const FN_UDA_TRANSFER: u16 = 3;
pub const FN_UAC_TRANSFER: u16 = 3;
pub const FN_FAC_TRANSFER: u16 = 6;
pub const FN_UNIQUE: u16 = 3;

pub(self) const FN_VERIFY_TOKEN: u16 = 1;

pub fn unique() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();

    const VERIFY_AMOUNT: u16 = 2;

    let mut code = uasm! {
      .proc: FN_RGB21_ISSUE;
        call    shared, :FN_ASSET_SPEC   ;// Call asset check.
        // Check that there is no fractionality
        mov     E2, 1;
        eq      EB, E2;
        chk     CO;
        clr     E2;

        ldo     :immutable      ;// Read fourth global state - token information
        call    :FN_VERIFY_TOKEN;// Verify token spec
        cknxo   :immutable      ;// Verify there are no more tokens
        not     CO;
        chk     CO;

        call    :VERIFY_AMOUNT  ;// Verify token amount
        ret;

      // Verify token spec
      .proc: FN_VERIFY_TOKEN;
        mov     E7, :G_SUPPLY   ;// Set E7 to field element representing token data
        eq      EA, E7          ;// It must have the correct state type
        chk     CO              ;// Or fail otherwise
        test    EB              ;// Token id must be set
        chk     CO              ;// Or we should fail
        mov     EE, EB          ;// Save token id for VERIFY_AMOUNT
        test    EC              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need the test to fail)
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need the test to fail)
        chk     CO              ;// fail if not
        ret;

      .routine: VERIFY_AMOUNT;
        ldo     :destructible;
        mov     E7, :O_AMOUNT;   // Set E7 to field element representing token data
        eq      EA, E7;
        chk     CO;
        mov     E7, 1;
        eq      EB, E7;          // The Amount must be exactly 1
        chk     CO;
        eq      EC, EE;         // Check that the token id is correct
        chk     CO;
        test    ED;             // The rest of the field elements must be empty
        chk     CO;

        cknxo   :destructible;   // Verify there are no more tokens
        not     CO;
        chk     CO;
        ret;

      .proc: FN_UDA_TRANSFER;
        ret; // TODO: Implement
    };

    CompiledLib::compile(&mut code, &[&shared_lib()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

pub fn collection() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();
    let uda = unique().into_lib().lib_id();

    const CHECK_TOKENS: u16 = 1;
    const VERIFY_AMOUNT: u16 = 2;
    const NEXT_OUTPUT: u16 = 4;
    const NEXT_GLOBAL: u16 = 5;

    let mut code = uasm! {
      .proc: FN_RGB21_ISSUE;
        call    shared, :FN_ASSET_SPEC; // Check asset specification

        // Check there is no fractionality
        mov     E2, 1;
        eq      EB, E2;         // EB still contains fractions from asset spec
        chk     CO;
        clr     E2;

        call    :CHECK_TOKENS;
        call    :FN_UNIQUE;
        ret;

      .routine: CHECK_TOKENS;
        ldo     :immutable;     // Read token information
        chk     CO;
        jif     CO, +3;         // Return if no more state is count
        ret;

        call    uda, :FN_VERIFY_TOKEN; // Verify token spec
        rsto    :destructible;  // Start iteration over owned tokens
        mov     E2, 0;          // Initialize token counter
        call    :VERIFY_AMOUNT; // Verify token amount
        mov     E7, 1;          // Check token fraction is exactly 1
        eq      EB, E7;
        chk     CO;
        jmp     :CHECK_TOKENS;  // Loop next token

        ret;

      .proc: VERIFY_AMOUNT;
        ldo     :destructible;
        chk     CO;
        jif     CO, +3;
        ret;

        mov     E7, :O_AMOUNT;  // Check that the state type is correct
        eq      EA, E7;
        chk     CO;

        eq      EC, EE;         // Filter by token Id
        chk     CO;
        jif     CO, +3;
        ret;

        mov     E7, 1;          // Check the amount is correct
        eq      EB, E7;
        chk     CO;

        add     E2, E7;         // Increase token counter

        test    ED;             // The last field element must be empty
        chk     CO;

        jmp     :VERIFY_AMOUNT; // Process to the next token

      // Check we do not use tokens not listed in the global state
      .proc: FN_UNIQUE;
        rsto    :destructible;  // Reset output owned state iterator
        mov     E2, 1;          // We need this for the first cycle to succeed

      .label: NEXT_OUTPUT;
        mov     E7, 1;          // Check there is a token
        eq      E2, E7;
        chk     CO;

        ldo     :destructible;  // Load next token data
        jif     CO, +3;         // Return if no more tokens left
        ret;

        mov     E2, 0;          // Initialize token counter for the global state
        mov     EE, EB;         // Save the token id
        rsto    :immutable;     // Start iteration over global state

      .label: NEXT_GLOBAL;
        ldo     :immutable;
        jif     CO, :NEXT_OUTPUT;// No more tokens in global state, processing to the next output

        eq      EC, EE;         // Filter by token id
        jif     CO, :NEXT_GLOBAL;

        add     E2, E7;         // Increment token counter
        jmp     :NEXT_GLOBAL;

      .proc: FN_UAC_TRANSFER;
        nop;

        cknxo   :immutable;     // No new global state must be defined
        not     CO;
        chk     CO;

        // TODO: Complete implementation
        ret;
    };

    CompiledLib::compile(&mut code, &[&shared_lib(), &unique()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

pub fn fractionable() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();
    let uda = unique().into_lib().lib_id();

    const NEXT_TOKEN: u16 = 1;
    const END_TOKENS: u16 = 2;
    const NEXT_OWNED: u16 = 3;
    const NEXT_GLOBAL: u16 = 4;
    const END_TOKEN: u16 = 5;
    const LOOP_TOKEN: u16 = 7;

    // TODO: Check the correctness of the implementation
    let mut code = uasm! {
     .proc: FN_RGB21_ISSUE;
        call    shared, :FN_ASSET_SPEC   ;// Call asset check
        fits    EB, 64:bits     ;// The precision must fit into u64
        chk     CO              ;// - or fail otherwise
        mov     E2, EB          ;// Save 'fractions' value to match it against issued amounts

        // Validate global tokens and issued amounts
        mov     E4, 0           ;// Start counter for tokens

     .label: NEXT_TOKEN;
        ldo     :immutable      ;// Read fourth global state - token information
        jif     CO, :END_TOKENS ;// Complete token validation if no more tokens left

        // Verify token spec
        call    uda, :FN_VERIFY_TOKEN   ;// Verify token spec
        // TODO: Ensure all token ids are unique

        // Check issued supply
        call    shared, :FN_SUM_OUTPUTS    ;// Sum outputs
        eq      E2, E3          ;// check that 'fractions' supply equals to the sum of outputs
        chk     CO              ;// fail if not
        mov     E8, 1           ;// E8 will hold 1 as a constant for counter-increment operation
        add     E4, E8          ;// Increment token counter
        jmp     :NEXT_TOKEN     ;// Process to the next token

        // Validate that owned tokens match the list of issued tokens
      .label: END_TOKENS;
        rsto    :destructible   ;// Reset state iterator
      .label: NEXT_OWNED;
        rsto    :immutable      ;// Reset state iterator
        ldo     :destructible   ;// Iterate over tokens
        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;
        mov     E6, EC          ;// Save token id
        mov     E5, 0           ;// Start counter
        mov     E7, :G_SUPPLY   ;// Set E7 to field element representing token data
      .label: NEXT_GLOBAL;
        ldo     :immutable      ;// Load global state
        jif     CO, :END_TOKEN  ;// We've done
        eq      EA, E7          ;// It must has correct state type
        jif     CO, :NEXT_GLOBAL;// If not, goto next global state
        eq      EB, E6          ;// Check if the token id match
        jif     CO, :NEXT_GLOBAL;// Skip otherwise
        mov     E8, 1           ;// E8 will hold 1 as a constant for counter increment operation
        add     E5, E8          ;// Increase counter
      .label: END_TOKEN;
        mov     E8, 0           ;// E8 will hold 0 as a constant for `eq` operation
        eq      E5, E8          ;// Check that the token has allocations
        not     CO              ;// We need to invert CO so if no allocations we fail
        chk     CO              ;// Fail otherwise
        jmp     :NEXT_OWNED     ;// Go to the next owned

      .proc: SUB_TRANSFER_RGB21;
        // Verify that no global state is defined
        cknxo   :immutable      ;// Try to iterate over global state
        not     CO              ;// Invert result (we need NO state as a Success)
        chk     CO              ;// Fail if there is a global state

        mov     EE, :O_AMOUNT;// Set EE to the field element representing owned value

        // For each token verify the sum of inputs equal sum of outputs
      .label: LOOP_TOKEN;
        ldi     :immutable      ;// Iterate over tokens
        not     CO;
        jif     CO, +3;
        ret                     ;// Finish if no more tokens
        mov     EE, EB          ;// Save token id for FN_SUM_OUTPUTS
        call    shared, :FN_SUM_INPUTS     ;// Compute sum of inputs
        call    shared, :FN_SUM_OUTPUTS    ;// Compute sum of outputs
        eq      E2, E3          ;// check that the sum of inputs equals sum of outputs
        chk     CO              ;// fail if not
        jmp     :LOOP_TOKEN     ;// Process to the next token

        // TODO: Check that no tokens not listed in global state are defined
    };

    CompiledLib::compile(&mut code, &[&shared_lib(), &unique()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{G_DETAILS, G_NAME, G_PRECISION, G_SUPPLY};
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
            let lib = fractionable();
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
        (fractionable(), vm, resolver)
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
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        const SUPPLY: u64 = 1000_u64;
        let mut context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
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
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[],
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
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
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
    fn genesis_correct() {
        const TOKEN_ID: u64 = 0;
        const SUPPLY: u64 = 1000_u64;
        let context = VmContext {
            read_once_input: &[],
            immutable_input: &[],
            read_once_output: &[StateCell {
                data: StateValue::Triple {
                    first: O_AMOUNT.into(),
                    second: SUPPLY.into(),
                    third: TOKEN_ID.into(),
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
