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

use aluvm::{CoreConfig, LibSite};
use amplify::num::u256;
use hypersonic::{Api, ApiInner, AppendApi, Codex, DestructibleApi, Identity, Schema, FIELD_ORDER_SECP};
use hypersonic::embedded::{EmbeddedArithm, EmbeddedImmutable, EmbeddedProc};
use ifaces::CommonTypes;
use strict_types::SemId;
use schemata::scripts;

const PANDORA: &str = "dns:pandoraprime.ch";

fn codex() -> Codex {
    let lib = scripts::success();
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
            0 => LibSite::new(lib_id, 0),
            1 => LibSite::new(lib_id, 0),
            2 => LibSite::new(lib_id, 0),
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
                adaptor: EmbeddedImmutable(u256::ZERO),
            }
        },
        readers: empty!(),
        verifiers: tiny_bmap! {
            vname!("issue") => 0,
            vname!("transfer") => 1,
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
        .save("../compiled/NonInflatableAsset.issuer")
        .expect("unable to save issuer to a file");

}
