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
use crate::O_AMOUNT;

pub const FN_RGB21_ISSUE: u16 = 0;
pub const FN_RGB21_TRANSFER: u16 = 6;

pub(self) const NEXT_TOKEN: u16 = 1;
pub(self) const END_TOKENS: u16 = 2;
pub(self) const NEXT_OWNED: u16 = 3;
pub(self) const NEXT_GLOBAL: u16 = 4;
pub(self) const END_TOKEN: u16 = 5;
pub(self) const LOOP_TOKEN: u16 = 7;

pub fn non_fungible() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();

    let mut code = uasm! {
    // .proc FN_RGB21_ISSUE
        nop                     ;// Marks start of routine / entry point / goto target

        call    shared, :FN_ASSET_SPEC   ;// Call asset check
        fits    EB, 64:bits     ;// The precision must fit into u64
        chk     CO              ;// - or fail otherwise
        mov     E1, EB          ;// Save fractions value to match it against issued amounts

        // Validate global tokens and issued amounts
        mov     E3, 0           ;// Start counter for tokens
    // .label NEXT_TOKEN
        nop;
        ldo     :immutable      ;// Read fourth global state - token information
        jif     CO, :END_TOKENS ;// Complete token validation if no more tokens left

        // Verify token spec
        eq      EA, EH          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        test    EB              ;// Token id must be set
        chk     CO              ;// Or we should fail
        mov     EE, EB          ;// Save token id for FN_SUM_OUTPUTS
        test    EC              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need test to fail)
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        not     CO              ;// invert CO value (we need test to fail)
        chk     CO              ;// fail if not

        // Check issued supply
        call    shared, :FN_SUM_OUTPUTS    ;// Sum outputs
        eq      E1, E2          ;// check that circulating supply equals to the sum of outputs
        chk     CO              ;// fail if not
        mov     E8, 1           ;// E8 will hold 1 as a constant for counter increment operation
        add     E3, E8          ;// Increment token counter
        jmp     :NEXT_TOKEN     ;// Process to the next token

        // Validate that owned tokens match the list of issued tokens
    // .label END_TOKENS
        nop;
        cknxo   :immutable      ;// Check there is no more global state
        chk     CO              ;// Fail otherwise

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
        mov     E8, 1           ;// E8 will hold 1 as a constant for counter increment operation
        add     E5, E8          ;// Increase counter
    // .label END_TOKEN
        nop;
        mov     E8, 0           ;// E8 will hold 0 as a constant for `eq` operation
        eq      E5, E8          ;// Check that token has allocations
        not     CO              ;// We need to invert CO so if no allocations we fail
        chk     CO              ;// Fail otherwise
        jmp     :NEXT_OWNED     ;// Go to the next owned

    // .proc SUB_TRANSFER_RGB21
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
        mov     EE, EB          ;// Save token id for FN_SUM_OUTPUTS
        call    shared, :FN_SUM_INPUTS     ;// Compute sum of inputs
        call    shared, :FN_SUM_OUTPUTS    ;// Compute sum of outputs
        eq      E1, E2          ;// check that the sum of inputs equals sum of outputs
        chk     CO              ;// fail if not
        jmp     :LOOP_TOKEN     ;// Process to the next token
    };

    CompiledLib::compile(&mut code, &[&shared_lib()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
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
