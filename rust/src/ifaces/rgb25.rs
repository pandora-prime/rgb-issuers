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
use hypersonic::embedded::{EmbeddedArithm, EmbeddedImmutable, EmbeddedProc};
use hypersonic::{Api, ApiInner, AppendApi, CallState, CodexId, DestructibleApi, Identity};
use ifaces::Rgb21Types;
use strict_types::SemId;

use crate::{G_DETAILS, G_NAME, G_PRECISION, G_SUPPLY, O_AMOUNT, PANDORA};

pub fn api(codex_id: CodexId) -> Api {
    let types = Rgb21Types::new();

    Api::Embedded(ApiInner::<EmbeddedProc> {
        version: default!(),
        codex_id,
        timestamp: 1732529307,
        name: None,
        developer: Identity::from(PANDORA),
        conforms: Some(tn!("RGB25")),
        default_call: Some(CallState::with("transfer", "amount")),
        reserved: default!(),
        append_only: tiny_bmap! {
            vname!("name") => AppendApi {
                sem_id: types.get("RGBContract.AssetName"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(G_NAME),
            },
            vname!("details") => AppendApi {
                sem_id: SemId::unit(),
                raw_sem_id: types.get("RGBContract.Details"),
                published: true,
                adaptor: EmbeddedImmutable(G_DETAILS),
            },
            vname!("precision") => AppendApi {
                sem_id: types.get("RGBContract.Precision"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(G_PRECISION),
            },
            vname!("circulating") => AppendApi {
                sem_id: types.get("RGBContract.Amount"),
                raw_sem_id: SemId::unit(),
                published: true,
                adaptor: EmbeddedImmutable(G_SUPPLY),
            },
        },
        destructible: tiny_bmap! {
            vname!("amount") => DestructibleApi {
                sem_id: types.get("RGBContract.Amount"),
                arithmetics: EmbeddedArithm::Fungible,
                adaptor: EmbeddedImmutable(O_AMOUNT),
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
