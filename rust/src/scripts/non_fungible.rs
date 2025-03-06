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

use crate::{GLOBAL_ASSET_NAME, G_DETAILS, G_PRECISION, G_SUPPLY, O_AMOUNT};

pub const NON_FUNGIBLE_LIB_ID: &str = "";

pub const FN_RGB21_ISSUE: u16 = 0;
pub const FN_RGB21_TRANSFER: u16 = 6;

pub(self) const NEXT_TOKEN: u16 = 1;
pub(self) const END_TOKENS: u16 = 2;
pub(self) const NEXT_OWNED: u16 = 3;
pub(self) const NEXT_GLOBAL: u16 = 4;
pub(self) const END_TOKEN: u16 = 5;
pub(self) const LOOP_TOKEN: u16 = 7;
pub(self) const VERIFY_TOKEN: u16 = 8;
pub(self) const SUM_INPUTS: u16 = 9;
pub(self) const SUM_OUTPUTS: u16 = 10;

pub fn non_fungible() -> CompiledLib {
    assert_eq!(O_AMOUNT, GLOBAL_ASSET_NAME);

    let mut code = uasm! {
    // .routine SUB_ISSUE_RGB21
        nop                     ;// Marks start of routine / entry point / goto target
        // Set initial values
        mov     EE, :O_AMOUNT   ;// Set EE to the field element representing owned value (also global asset name)
        mov     EF, :G_DETAILS  ;// Set EF to field element representing global asset details
        mov     EG, :G_PRECISION;// Set EF to field element representing global fractions
        mov     EH, :G_SUPPLY   ;// Set EF to field element representing global tokens
        mov     E2, 0           ;// E3 will contain sum of outputs
        mov     E7, 0           ;// E7 will hold 0 as a constant for `eq` operation
        mov     E8, 1           ;// E8 will hold 1 as a constant for counter increment operation
        // Validate verbose globals
        ldo     :immutable      ;// Read first global state - name
        chk     CO              ;// It must exist
        eq      EA, EE          ;// It must have correct state type
        chk     CO              ;// Or fail otherwise
        ldo     :immutable      ;// Read second global state (ticker for RGB20, details for RGB25)
        chk     CO              ;// It must exist
        eq      EA, EF          ;// It must have correct state type
        chk     CO              ;// Or fail otherwise
        ldo     :immutable      ;// Read third global state - precision
        chk     CO              ;// It must exist
        eq      EA, EG          ;// It must have correct state type
        chk     CO              ;// Or fail otherwise
        mov     E1, EB          ;// Save fractions value to match it against issued amounts

        // Validate global tokens and issued amounts
        mov     E3, 0           ;// Start counter for tokens
    // .loop NEXT_TOKEN
        nop;
        ldo     :immutable      ;// Read fourth global state - token information
        jif     CO, :END_TOKENS ;// Complete token validation if no more tokens left
        call    :VERIFY_TOKEN   ;// Do token verification
        mov     E2, 0           ;// Initialize sum of outputs
        call    :SUM_OUTPUTS    ;// Sum outputs
        eq      E1, E2          ;// check that circulating supply equals to the sum of outputs
        chk     CO              ;// fail if not
        add     E3, E8          ;// Increment token counter
        rsto    :destructible   ;// Reset state iterator
        jmp     :NEXT_TOKEN     ;// Process to the next token

        // Validate that owned tokens match the list of issued tokens
    // .label END_TOKENS
        nop;
        rsto    :destructible   ;// Reset state iterator
    // .label NEXT_OWNED
        nop;
        rsto    :immutable      ;// Reset state iterator
        ldo     :destructible   ;// Iterate over tokens
        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;
        mov     E4, EB          ;// Save token id
        mov     E5, 0           ;// Start counter
    // .label NEXT_GLOBAL
        nop;
        ldo     :immutable      ;// Load global state
        jif     CO, :END_TOKEN  ;// We've done
        eq      EA, EH          ;// It must has correct state type
        jif     CO, :NEXT_GLOBAL;// If not, goto next global state
        eq      EB, E4          ;// Check if the token id match
        jif     CO, :NEXT_GLOBAL;// Skip otherwise
        add     E5, E8          ;// Increase counter
    // .label END_TOKEN
        nop;
        eq      E5, E7          ;// Check that token has allocations
        not     CO              ;// We need to invert CO so if no allocations we fail
        chk     CO              ;// Fail otherwise
        jmp     :NEXT_OWNED     ;// Go to the next owned

    // .routine SUB_TRANSFER_RGB21
        nop;
        // Verify that no global state is defined
        ldo     :immutable      ;// Try to iterate over global state
        not     CO              ;// Invert result (we need NO state as a Success)
        chk     CO              ;// Fail if there is a global state

        mov     EE, :O_AMOUNT;// Set EE to the field element representing owned value

        // For each token verify sum of inputs equal sum of outputs
    // .label LOOP_TOKEN
        nop;
        ldi     :immutable      ;// Iterate over tokens
        not     CO;
        jif     CO, +3;
        ret                     ;// Finish if no more tokens

        rsti    :destructible   ;// Start iteration over inputs
        rsto    :destructible   ;// Start iteration over outputs
        mov     E1, 0           ;// Initialize sum of outputs
        mov     E2, 0           ;// Initialize sum of outputs

        call    :SUM_INPUTS     ;// Compute sum of inputs
        call    :SUM_OUTPUTS    ;// Compute sum of outputs
        eq      E1, E2          ;// check that the sum of inputs equals sum of outputs
        chk     CO              ;// fail if not
        jmp     :LOOP_TOKEN     ;// Process to the next token

    // .routine VERIFY_TOKEN
        nop;
        eq      EA, EH          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        test    EB              ;// Token id must be set
        chk     CO              ;// Or we should fail
        mov     E5, EB          ;// Save token id
        test    EC              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need test to fail)
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need test to fail)
        chk     CO              ;// fail if not
        ret;

    // .routine SUM_INPUTS
        nop;
        // Iterate over allocations
        ldi     :destructible   ;// Load input allocation
        not     CO;
        jif     CO, +3;
        ret                     ;// Finish if no more tokens
        eq      EA, EE          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        eq      EB, E5          ;// Do we have the correct token id?
        jif     CO, :SUM_INPUTS;// Read next allocation
        fits    ED, 8:bits      ;// ensure the value fits in 8 bits
        add     E1, ED          ;// add supply to input accumulator
        fits    E1, 8:bits      ;// ensure we do not overflow
        jmp     :SUM_INPUTS     ;// Process to the next allocation

    // .routine SUM_OUTPUTS
        nop;
        // Iterate over allocations
        ldo     :destructible   ;// Load output allocation
        not     CO;
        jif     CO, +3;
        ret                     ;// Finish if no more tokens
        eq      EA, EE          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        eq      EB, E5          ;// Do we have the correct token id?
        jif     CO, :SUM_OUTPUTS;// Read next allocation
        fits    ED, 8:bits      ;// ensure the value fits in 8 bits
        add     E2, ED          ;// add supply to output accumulator
        fits    E2, 8:bits      ;// ensure we do not overflow
        jmp     :SUM_OUTPUTS    ;// Process to the next allocation
    };

    CompiledLib::compile(&mut code).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypersonic::Instr;
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
            let lib = non_fungible();
            assert_eq!(id, lib.as_lib().lib_id());
            Some(lib.into_lib())
        }
        (non_fungible(), vm, resolver)
    }
}
