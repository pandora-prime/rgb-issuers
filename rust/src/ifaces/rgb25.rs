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
use hypersonic::{Api, CallState, CodexId, DestructibleApi, Identity, ImmutableApi, RawBuilder, RawConvertor, StateArithm, StateBuilder, StateConvertor};
use ifaces::Rgb21Types;
use strict_types::SemId;

use crate::{G_DETAILS, G_NAME, G_PRECISION, G_SUPPLY, O_AMOUNT, PANDORA};

pub fn api(codex_id: CodexId) -> Api {
    let types = Rgb21Types::new();

    Api {
        version: default!(),
        codex_id,
        developer: Identity::from(PANDORA),
        conforms: Some(tn!("RGB25")),
        default_call: Some(CallState::with("transfer", "amount")),
        reserved: default!(),
        immutable: tiny_bmap! {
            vname!("name") => ImmutableApi {
                published: true,
                sem_id: types.get("RGBContract.AssetName"),
                convertor: StateConvertor::TypedEncoder(G_NAME),
                builder: StateBuilder::TypedEncoder(G_NAME),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("details") => ImmutableApi {
                published: true,
                sem_id: SemId::unit(),
                convertor: StateConvertor::TypedEncoder(G_DETAILS),
                builder: StateBuilder::TypedEncoder(G_DETAILS),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(types.get("RGBContract.Details"))
            },
            vname!("precision") => ImmutableApi {
                published: true,
                sem_id: types.get("RGBContract.Precision"),
                convertor: StateConvertor::TypedEncoder(G_PRECISION),
                builder: StateBuilder::TypedEncoder(G_PRECISION),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("circulating") => ImmutableApi {
                published: true,
                sem_id: types.get("RGBContract.Amount"),
                convertor: StateConvertor::TypedEncoder(G_SUPPLY),
                builder: StateBuilder::TypedEncoder(G_SUPPLY),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
        },
        destructible: tiny_bmap! {
            vname!("amount") => DestructibleApi {
                sem_id: types.get("RGBContract.Amount"),
                arithmetics: StateArithm::Fungible,
                convertor: StateConvertor::TypedEncoder(O_AMOUNT),
                builder: StateBuilder::TypedEncoder(O_AMOUNT),
                witness_sem_id: SemId::unit(),
                witness_builder: StateBuilder::TypedEncoder(O_AMOUNT)
            }
        },
        aggregators: empty!(),
        verifiers: tiny_bmap! {
            vname!("issue") => 0,
            vname!("transfer") => 1,
            vname!("_") => 0xFF,
        },
        errors: tiny_bmap! {
            u256::ZERO => tiny_s!("the sum of inputs is not equal to the sum of outputs")
        },
    }
}
