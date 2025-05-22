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
use crate::{fractionable, O_AMOUNT};

pub const FN_FAC_TRANSFER: u16 = 6;
pub const FN_UNIQUE: u16 = 3;

pub fn collection() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();
    let uniq = unique().into_lib().lib_id();

    const CHECK_TOKENS: u16 = 1;
    const VERIFY_AMOUNT: u16 = 2;
    const NEXT_OUTPUT: u16 = 4;
    const NEXT_GLOBAL: u16 = 5;

    let mut code = uasm! {
      proc FN_RGB21_ISSUE:
        call    shared, FN_ASSET_SPEC; // Check asset specification

        // Check there is no fractionality
        put     E2, 1;
        eq      EB, E2;         // EB still contains fractions from asset spec
        chk     CO;
        clr     E2;

        call    CHECK_TOKENS;
        call    FN_UNIQUE;
        ret;

      routine CHECK_TOKENS:
        ldo     immutable;      // Read token information
        chk     CO;
        jif     CO, +3;         // Return if no more state is count
        ret;

        call    uniq, FN_GLOBAL_VERIFY_TOKEN; // Verify token spec
        rsto    destructible;   // Start iteration over owned tokens
        put     E2, 0;          // Initialize token counter
        call    VERIFY_AMOUNT;  // Verify token amount
        put     E7, 1;          // Check token fraction is exactly 1
        eq      EB, E7;
        chk     CO;
        jmp     CHECK_TOKENS;   // Loop next token

        ret;

      proc VERIFY_AMOUNT:
        ldo     destructible;
        chk     CO;
        jif     CO, +3;
        ret;

        put     E7, O_AMOUNT;   // Check that the state type is correct
        eq      EA, E7;
        chk     CO;

        eq      EC, EE;         // Filter by token Id
        chk     CO;
        jif     CO, +3;
        ret;

        put     E7, 1;          // Check the amount is correct
        eq      EB, E7;
        chk     CO;

        add     E2, E7;         // Increase token counter

        test    ED;             // The last field element must be empty
        chk     CO;

        jmp     VERIFY_AMOUNT; // Process to the next token

      // Check we do not use tokens not listed in the global state
      // TODO: Ensure all token ids are unique
      proc FN_UNIQUE:
        rsto    destructible;  // Reset output owned state iterator
        put     E2, 1;          // We need this for the first cycle to succeed

      label NEXT_OUTPUT:
        put     E7, 1;          // Check there is a token
        eq      E2, E7;
        chk     CO;

        ldo     destructible;  // Load next token data
        jif     CO, +3;         // Return if no more tokens left
        ret;

        put     E2, 0;          // Initialize token counter for the global state
        mov     EE, EB;         // Save the token id
        rsto    immutable;     // Start iteration over global state

      label NEXT_GLOBAL:
        ldo     immutable;
        jif     CO, NEXT_OUTPUT;// No more tokens in global state, processing to the next output

        eq      EC, EE;         // Filter by token id
        jif     CO, NEXT_GLOBAL;

        add     E2, E7;         // Increment token counter
        jmp     NEXT_GLOBAL;

      proc FN_UAC_TRANSFER:
        cknxo   immutable;     // No new global state must be defined
        not     CO;
        chk     CO;

        // TODO: Complete implementation
        ret;
    };

    CompiledLib::compile(&mut code, &[&shared_lib(), &unique(), &fractionable()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripts::fractionable;
    use hypersonic::Instr;
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
            let lib = collection();
            let unique = unique();
            let fractionable = fractionable();
            let shared = shared_lib();
            if lib.as_lib().lib_id() == id {
                return Some(lib.into_lib());
            }
            if fractionable.as_lib().lib_id() == id {
                return Some(fractionable.into_lib());
            }
            if unique.as_lib().lib_id() == id {
                return Some(unique.into_lib());
            }
            if shared.as_lib().lib_id() == id {
                return Some(shared.into_lib());
            }
            panic!("Unknown library: {id}");
        }
        (collection(), vm, resolver)
    }
}
