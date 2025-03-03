// RGB schemata
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

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;

use zkaluvm::alu::{CoreConfig, Lib, LibSite};
use amplify::num::u256;
use hypersonic::{uasm, Api, ApiInner, AppendApi, CallState, Codex, DestructibleApi, Identity, Schema, FIELD_ORDER_SECP};
use hypersonic::embedded::{EmbeddedArithm, EmbeddedImmutable, EmbeddedProc};
use ifaces::CommonTypes;
use strict_types::SemId;
use schemata::scripts;

const PANDORA: &str = "dns:pandoraprime.ch";

pub const OWNED_VALUE: u256 = u256::ZERO;

pub fn scripts() -> Lib {
    let mut code = uasm! {
        // PROC: ISSUE VALIDATION
        nop                     ;// Marks start of routine / entry point / goto target
        // Set initial values
        mov     EE, :OWNED_VALUE;// Set EE to the field element representing owned value (also global name)
        mov     EF, 1           ;// Set EF to field element representing global ticker
        mov     EG, 2           ;// Set EF to field element representing global precision
        mov     EH, 3           ;// Set EF to field element representing global circulation
        mov     E2, 0           ;// E3 will contain sum of outputs
        // Validate verbose globals
        ldo     :immutable      ;// Read first global state - name
        chk     CO              ;// It must exist
        eq      EA, EE          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        ldo     :immutable      ;// Read second global state - ticker
        chk     CO              ;// It must exist
        eq      EA, EF          ;// It must has correct state type
        chk     CO              ;// Or fail otherwise
        ldo     :immutable      ;// Read second global state - precision
        chk     CO              ;// It must exist
        eq      EA, EG          ;// It must has correct state type
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
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        chk     CO              ;// fail if not
        call    0x0003          ;// Compute sum of outputs
        eq      E1, E2          ;// check that circulating supply equals to the sum of outputs
        chk     CO              ;// fail if not
        ret;

        // PROC: TRANSFER VALIDATION
        // Set initial values
        nop                     ;// Marks start of routine / entry point / goto target
        mov     EE, :OWNED_VALUE;// Set EE to the field element representing owned value
        mov     E1, 0           ;// E1 will contain sum of inputs
        mov     E2, 0           ;// E2 will contain sum of outputs
        // Verify owned state
        call    0x0002          ;// Compute sum of inputs
        call    0x0003          ;// Compute sum of outputs
        eq      E1, E2          ;// check that the sum of inputs equals sum of outputs
        chk     CO              ;// fail if not
        // Verify that no global state is assigned
        ldo     :immutable      ;// Try to iterate over global state
        not     CO              ;// Invert result (we need NO state as a Success)
        chk     CO              ;// Fail if there is a global state
        ret;

        // PROC: SUM INPUTS
        // Start iterations:
        nop                     ;// Marks start of routine / entry point / goto target

        ldi     :destructible   ;// load next state value
        // Finish if no more elements are present
        jif     CO, +1;
        ret;

        eq      EA, EE          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EC              ;// ensure other field elements are empty
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        chk     CO              ;// fail if not

        fits    EB, 8:bits      ;// ensure the value fits in 8 bits
        add     E1, EB          ;// add input to input accumulator
        fits    E1, 8:bits      ;// ensure we do not overflow
        jmp     0x0002          ;// loop

        // PROC: SUM OUTPUTS
        // Start iterations:
        nop                     ;// Mark the start of the routine
        ldo     :destructible   ;// load next state value

        // Finish if no more elements are present
        jif     CO, +1;
        ret;

        eq      EA, EE          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EC              ;// ensure other field elements are empty
        chk     CO              ;// fail if not
        test    ED              ;// ensure other field elements are empty
        chk     CO              ;// fail if not

        fits    EB, 8:bits      ;// ensure the value fits in 8 bits
        add     E2, EB          ;// add input to input accumulator
        fits    E2, 8:bits      ;// ensure we do not overflow
        jmp     0x0003          ;// loop
    };

    Lib::compile(&mut code).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

fn codex() -> Codex {
    let lib = scripts();
    let lib_id = lib.lib_id();
    Codex {
        name: tiny_s!("NonInflatableAsset"),
        developer: Identity::from(PANDORA),
        version: default!(),
        timestamp: 1732529307,
        field_order: FIELD_ORDER_SECP,
        input_config: CoreConfig::default(),
        verification_config: CoreConfig::default(),
        verifiers: tiny_bmap! {
            0 => LibSite::new(lib_id, 0), // TODO: Write a custom script
            1 => LibSite::new(lib_id, 0),
            0xFF => LibSite::new(lib_id, 0), // Blank transition is just an ordinary self-transfer
        },
        reserved: default!(),
    }
}

fn api() -> Api {
    let types = CommonTypes::new();

    let codex = codex();

    Api::Embedded(ApiInner::<EmbeddedProc> {
        version: default!(),
        codex_id: codex.codex_id(),
        timestamp: 1732529307,
        name: None,
        developer: Identity::from(PANDORA),
        conforms: Some(tn!("RGB20")),
        default_call: Some(CallState::with("transfer", "owned")),
        reserved: default!(),
        append_only: tiny_bmap! {
            vname!("name") => AppendApi {
                sem_id: types.get("RGBContract.AssetName"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(u256::ZERO),
            },
            vname!("ticker") => AppendApi {
                sem_id: types.get("RGBContract.Ticker"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(u256::ONE),
            },
            vname!("precision") => AppendApi {
                sem_id: types.get("RGBContract.Precision"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(u256::from(2u8)),
            },
            vname!("circulating") => AppendApi {
                sem_id: types.get("RGBContract.Amount"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(u256::from(3u8)),
            },
        },
        destructible: tiny_bmap! {
            vname!("owned") => DestructibleApi {
                sem_id: types.get("RGBContract.Amount"),
                arithmetics: EmbeddedArithm::Fungible,
                adaptor: EmbeddedImmutable(OWNED_VALUE),
            }
        },
        readers: empty!(),
        verifiers: tiny_bmap! {
            vname!("issue") => 0,
            vname!("transfer") => 1,
            vname!("_") => 0xFF,
        },
        errors: tiny_bmap! {
            u256::ZERO => tiny_s!("sum of inputs is not equal to sum of outputs")
        },
    })
}

fn main() {
    let types = CommonTypes::new();
    let codex = codex();
    let api = api();

    // Creating DAO with three participants
    let issuer = Schema::new(codex, api, [scripts::success()], types.type_system());
    issuer
        .save("compiled/NonInflatableAsset.issuer")
        .expect("unable to save issuer to a file");

}
