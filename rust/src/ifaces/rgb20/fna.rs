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

use hypersonic::{
    Aggregator, Api, CallState, Codex, CodexId, GlobalApi, Identity, Issuer, OwnedApi, RawBuilder,
    RawConvertor, Semantics, StateArithm, StateBuilder, StateConvertor, SubAggregator,
};
use ifaces::CommonTypes;
use strict_types::SemId;
use zkaluvm::alu::CoreConfig;
use zkaluvm::FIELD_ORDER_SECP;

use crate::scripts::{FN_FUNGIBLE_ISSUE, FN_FUNGIBLE_TRANSFER};
use crate::{
    scripts, ERRNO_INVALID_BALANCE_IN, ERRNO_INVALID_BALANCE_OUT, ERRNO_INVALID_PRECISION,
    ERRNO_NO_ISSUED, ERRNO_NO_NAME, ERRNO_NO_PRECISION, ERRNO_NO_TICKER, ERRNO_PRECISION_OVERFLOW,
    ERRNO_SUM_ISSUE_MISMATCH, ERRNO_SUM_MISMATCH, ERRNO_UNEXPECTED_GLOBAL,
    ERRNO_UNEXPECTED_GLOBAL_IN, ERRNO_UNEXPECTED_GLOBAL_OUT, ERRNO_UNEXPECTED_OWNED_IN,
    ERRNO_UNEXPECTED_OWNED_TYPE_IN, ERRNO_UNEXPECTED_OWNED_TYPE_OUT, G_NAME, G_PRECISION, G_SUPPLY,
    G_TICKER, O_AMOUNT, PANDORA,
};

pub const VERIFIER_GENESIS: u16 = 0;
pub const VERIFIER_TRANSFER: u16 = 1;

pub fn issuer() -> Issuer {
    let types = CommonTypes::new();
    let codex = codex();
    let api = api(codex.codex_id());

    let semantics = Semantics {
        version: 0,
        default: api,
        custom: none!(),
        codex_libs: small_bset![
            scripts::shared_lib().into_lib(),
            scripts::fungible().into_lib(),
        ],
        api_libs: none!(),
        types: types.type_system(),
    };
    Issuer::new(codex, semantics).expect("invalid issuer")
}

pub fn codex() -> Codex {
    let lib = scripts::fungible();
    Codex {
        name: tiny_s!("Fungible Non-inflatable Asset"),
        developer: Identity::from(PANDORA),
        version: default!(),
        features: none!(),
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
        codex_id,
        conforms: tiny_bset!(20),
        default_call: Some(CallState::with("transfer", "balance")),
        global: tiny_bmap! {
            vname!("ticker") => GlobalApi {
                published: true,
                sem_id: types.get("RGBContract.Ticker"),
                convertor: StateConvertor::TypedEncoder(G_TICKER),
                builder: StateBuilder::TypedEncoder(G_TICKER),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("name") => GlobalApi {
                published: true,
                sem_id: types.get("RGBContract.AssetName"),
                convertor: StateConvertor::TypedEncoder(G_NAME),
                builder: StateBuilder::TypedEncoder(G_NAME),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("precision") => GlobalApi {
                published: true,
                sem_id: types.get("RGBContract.Precision"),
                convertor: StateConvertor::TypedEncoder(G_PRECISION),
                builder: StateBuilder::TypedEncoder(G_PRECISION),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("issued") => GlobalApi {
                published: true,
                sem_id: types.get("RGBContract.Amount"),
                convertor: StateConvertor::TypedEncoder(G_SUPPLY),
                builder: StateBuilder::TypedEncoder(G_SUPPLY),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
        },
        owned: tiny_bmap! {
            vname!("balance") => OwnedApi {
                sem_id: types.get("RGBContract.Amount"),
                arithmetics: StateArithm::Fungible,
                convertor: StateConvertor::TypedEncoder(O_AMOUNT),
                builder: StateBuilder::TypedEncoder(O_AMOUNT),
                witness_sem_id: SemId::unit(),
                witness_builder: StateBuilder::Unit
            }
        },
        aggregators: tiny_bmap! {
            vname!("name") => Aggregator::Take(SubAggregator::TheOnly(vname!("name"))),
            vname!("ticker") => Aggregator::Take(SubAggregator::TheOnly(vname!("ticker"))),
            vname!("precision") => Aggregator::Take(SubAggregator::TheOnly(vname!("precision"))),
            vname!("supply") => Aggregator::Take(SubAggregator::SumOrDefault(vname!("issued"))),
            vname!("maxSupply") => Aggregator::Take(
                SubAggregator::Copy(vname!("issuedSupply"))
            ),
        },
        verifiers: tiny_bmap! {
            vname!("issue") => VERIFIER_GENESIS,
            vname!("transfer") => VERIFIER_TRANSFER,
            vname!("_") => VERIFIER_TRANSFER,
        },
        errors: tiny_bmap! {
            ERRNO_NO_TICKER => tiny_s!("no RGB20 ticker is set, or it is misplaced in the global state declaration (the ticker should be declared first)"),
            ERRNO_NO_NAME => tiny_s!("no RGB20 asset name is set, or it is misplaced in the global state declaration (the name should be declared second)"),
            ERRNO_NO_PRECISION => tiny_s!("no RGB20 precision is set, or it is misplaced in the global state declaration (the precision should be declared third)"),
            ERRNO_INVALID_PRECISION => tiny_s!("invalid RGB20 ticket precision value"),
            ERRNO_UNEXPECTED_OWNED_IN => tiny_s!("operation must have no inputs"),
            ERRNO_UNEXPECTED_GLOBAL_IN => tiny_s!("operation must not use any global state"),
            ERRNO_UNEXPECTED_GLOBAL_OUT => tiny_s!("operation must not declare any global state"),
            ERRNO_INVALID_BALANCE_IN => tiny_s!("invalid value for an input balance"),
            ERRNO_INVALID_BALANCE_OUT => tiny_s!("invalid value for an output balance"),
            ERRNO_NO_ISSUED => tiny_s!("no information about the issued supply found"),
            ERRNO_PRECISION_OVERFLOW => tiny_s!("the precision overflows the maximum value"),
            ERRNO_SUM_ISSUE_MISMATCH => tiny_s!("the declared issued supply does not match the output balance"),
            ERRNO_SUM_MISMATCH => tiny_s!("the sum of inputs is not equal to the sum of outputs"),
            ERRNO_UNEXPECTED_GLOBAL => tiny_s!("unexpected global state"),
            ERRNO_UNEXPECTED_OWNED_TYPE_IN => tiny_s!("unexpected operation input"),
            ERRNO_UNEXPECTED_OWNED_TYPE_OUT => tiny_s!("unexpected operation output"),
        },
    }
}
