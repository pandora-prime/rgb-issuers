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
    Api, CallState, CodexId, GlobalApi, OwnedApi, RawBuilder, RawConvertor, StateArithm,
    StateBuilder, StateConvertor,
};
use ifaces::Rgb21Types;
use strict_types::SemId;

use crate::{
    ERRNO_FRACTIONALITY, ERRNO_INVALID_PRECISION, ERRNO_INVALID_TOKEN_ID, ERRNO_NO_INPUT,
    ERRNO_NO_NAME, ERRNO_NO_OUTPUT, ERRNO_NO_PRECISION, ERRNO_NO_TICKER, ERRNO_NO_TOKEN_ID,
    ERRNO_TOKEN_EXCESS, ERRNO_TOKEN_EXCESS_IN, ERRNO_TOKEN_EXCESS_OUT, ERRNO_UNEXPECTED_GLOBAL_IN,
    ERRNO_UNEXPECTED_GLOBAL_OUT, ERRNO_UNEXPECTED_OWNED_IN, G_DETAILS, G_NAME, G_PRECISION,
    G_SUPPLY, O_AMOUNT,
};

pub const VERIFIER_GENESIS: u16 = 0;
pub const VERIFIER_TRANSFER: u16 = 1;

pub fn api(codex_id: CodexId) -> Api {
    let types = Rgb21Types::new();

    Api {
        codex_id,
        conforms: tiny_bset!(21),
        default_call: Some(CallState::with("transfer", "balance")),
        global: tiny_bmap! {
            // NFT collection name
            vname!("name") => GlobalApi {
                published: true,
                sem_id: types.get("RGBContract.AssetName"),
                convertor: StateConvertor::TypedEncoder(G_NAME),
                builder: StateBuilder::TypedEncoder(G_NAME),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("details") => GlobalApi {
                published: true,
                sem_id: SemId::unit(),
                convertor: StateConvertor::TypedEncoder(G_DETAILS),
                builder: StateBuilder::TypedEncoder(G_DETAILS),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(types.get("RGBContract.Details"))
            },
            vname!("maxFractions") => GlobalApi {
                published: true,
                sem_id: types.get("RGB21.TokenFractions"),
                convertor: StateConvertor::TypedEncoder(G_PRECISION),
                builder: StateBuilder::TypedEncoder(G_PRECISION),
                raw_convertor: RawConvertor::StrictDecode(SemId::unit()),
                raw_builder: RawBuilder::StrictEncode(SemId::unit())
            },
            vname!("token") => GlobalApi {
                published: true,
                sem_id: types.get("RGB21.TokenNo"),
                convertor: StateConvertor::TypedFieldEncoder(G_SUPPLY),
                builder: StateBuilder::TypedFieldEncoder(G_SUPPLY),
                raw_convertor: RawConvertor::StrictDecode(types.get("RGB21.NftSpec")),
                raw_builder: RawBuilder::StrictEncode(types.get("RGB21.NftSpec"))
            },
        },
        owned: tiny_bmap! {
            vname!("balance") => OwnedApi {
                sem_id: types.get("RGB21.Nft"),
                arithmetics: StateArithm::Fungible,
                convertor: StateConvertor::TypedFieldEncoder(O_AMOUNT),
                builder: StateBuilder::TypedFieldEncoder(O_AMOUNT),
                witness_sem_id: SemId::unit(),
                witness_builder: StateBuilder::Unit
            }
        },
        aggregators: empty!(),
        verifiers: tiny_bmap! {
            vname!("issue") => VERIFIER_GENESIS,
            vname!("transfer") => VERIFIER_TRANSFER,
            vname!("_") => VERIFIER_TRANSFER,
        },
        errors: tiny_bmap! {
            ERRNO_NO_TICKER => tiny_s!("no NFT ticker is set, or it is misplaced in the global state declaration (the ticker should be declared first)"),
            ERRNO_NO_NAME => tiny_s!("no NFT name is set, or it is misplaced in the global state declaration (the name should be declared second)"),
            ERRNO_NO_PRECISION => tiny_s!("no NFT fractionality is set, or it is misplaced in the global state declaration (the fractionality should be declared third)"),
            ERRNO_INVALID_PRECISION => tiny_s!("invalid NFT ticker fractionality value"),
            ERRNO_UNEXPECTED_OWNED_IN => tiny_s!("operation must have no inputs"),
            ERRNO_UNEXPECTED_GLOBAL_IN => tiny_s!("operation must not use any global state"),
            ERRNO_UNEXPECTED_GLOBAL_OUT => tiny_s!("operation must not declare any global state"),
            ERRNO_FRACTIONALITY => tiny_s!("the NFT token issued under this codex must be non-fractional"),
            ERRNO_INVALID_TOKEN_ID => tiny_s!("invalid token ID data"),
            ERRNO_NO_INPUT => tiny_s!("the transfer operation must have one input"),
            ERRNO_NO_OUTPUT => tiny_s!("the transfer operation must have one input"),
            ERRNO_NO_TOKEN_ID => tiny_s!("no token ID is set for the transfer"),
            ERRNO_TOKEN_EXCESS => tiny_s!("the number of issued NFT tokens must be one"),
            ERRNO_TOKEN_EXCESS_IN => tiny_s!("the number of transferred NFT token inputs must be one"),
            ERRNO_TOKEN_EXCESS_OUT => tiny_s!("the number of transferred NFT token outputs must be one"),
        },
    }
}
