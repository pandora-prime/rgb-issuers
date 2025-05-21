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

use crate::scripts::{shared_lib, FN_FUNGIBLE_ISSUE, FN_FUNGIBLE_TRANSFER};
use crate::{scripts, G_NAME, G_PRECISION, G_SUPPLY, G_TICKER, O_AMOUNT, PANDORA};
use amplify::num::u256;
use hypersonic::{
    Api, CallState, Codex, CodexId, DestructibleApi, Identity, ImmutableApi, Issuer, RawBuilder,
    RawConvertor, StateAggregator, StateArithm, StateBuilder, StateConvertor,
};
use ifaces::CommonTypes;
use strict_types::SemId;
use zkaluvm::alu::CoreConfig;
use zkaluvm::FIELD_ORDER_SECP;

pub const VERIFIER_GENESIS: u16 = 0;
pub const VERIFIER_TRANSFER: u16 = 1;

pub fn issuer() -> Issuer {
    let lib = scripts::fungible();
    let types = CommonTypes::new();
    let codex = codex();
    let api = api(codex.codex_id());

    Issuer::new(
        codex,
        api,
        [shared_lib().into_lib(), lib.into_lib()],
        types.type_system(),
    )
}

pub fn codex() -> Codex {
    let lib = scripts::fungible();
    Codex {
        name: tiny_s!("Fungible Non-inflatable Asset"),
        developer: Identity::from(PANDORA),
        version: default!(),
        timestamp: 1732529307,
        field_order: FIELD_ORDER_SECP,
        input_config: CoreConfig::default(),
        verification_config: CoreConfig::default(),
        verifiers: tiny_bmap! {
            VERIFIER_GENESIS => lib.routine(FN_FUNGIBLE_ISSUE),
            VERIFIER_TRANSFER => lib.routine(FN_FUNGIBLE_TRANSFER),
        },
    }
}

pub fn api(codex_id: CodexId) -> Api {
    let types = CommonTypes::new();

    Api {
        version: default!(),
        codex_id,
        developer: Identity::from(PANDORA),
        conforms: Some(tn!("RGB20")),
        default_call: Some(CallState::with("transfer", "balance")),
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
            vname!("ticker") => ImmutableApi {
                published: true,
                sem_id: types.get("RGBContract.Ticker"),
                convertor: StateConvertor::TypedEncoder(G_TICKER),
                builder: StateBuilder::TypedEncoder(G_TICKER),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("precision") => ImmutableApi {
                published: true,
                sem_id: types.get("RGBContract.Precision"),
                convertor: StateConvertor::TypedEncoder(G_PRECISION),
                builder: StateBuilder::TypedEncoder(G_PRECISION),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("issued") => ImmutableApi {
                published: true,
                sem_id: types.get("RGBContract.Amount"),
                convertor: StateConvertor::TypedEncoder(G_SUPPLY),
                builder: StateBuilder::TypedEncoder(G_SUPPLY),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
        },
        destructible: tiny_bmap! {
            vname!("balance") => DestructibleApi {
                sem_id: types.get("RGBContract.Amount"),
                arithmetics: StateArithm::Fungible,
                convertor: StateConvertor::TypedEncoder(O_AMOUNT),
                builder: StateBuilder::TypedEncoder(O_AMOUNT),
                witness_sem_id: SemId::unit(),
                witness_builder: StateBuilder::TypedEncoder(O_AMOUNT)
            }
        },
        aggregators: tiny_bmap! {
            vname!("knownIssued") => StateAggregator::SumV(vname!("issued")),
            vname!("knownBurned") => StateAggregator::SumV(vname!("burned")),
            vname!("knownCirculating") => StateAggregator::SumV(vname!("issued")),
            vname!("maxSupply") => StateAggregator::SumV(vname!("issued")),
        },
        verifiers: tiny_bmap! {
            vname!("issue") => VERIFIER_GENESIS,
            vname!("transfer") => VERIFIER_TRANSFER,
            vname!("_") => VERIFIER_TRANSFER,
        },
        errors: tiny_bmap! {
            u256::ZERO => tiny_s!("the sum of inputs is not equal to the sum of outputs")
        },
    }
}
