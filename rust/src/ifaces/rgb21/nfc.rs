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

use hypersonic::{Codex, Identity, Issuer, Semantics};
use ifaces::Rgb21Types;
use zkaluvm::alu::CoreConfig;
use zkaluvm::FIELD_ORDER_SECP;

use super::{api, VERIFIER_GENESIS, VERIFIER_TRANSFER};
use crate::{scripts, FN_DIVISIBLE_TRANSFER, FN_RGB21_ISSUE, PANDORA};

pub fn issuer() -> Issuer {
    let types = Rgb21Types::new();
    let codex = codex();
    let api = api(codex.codex_id());

    let semantics = Semantics {
        version: 0,
        default: api,
        custom: none!(),
        codex_libs: small_bset![
            scripts::shared_lib().into_lib(),
            scripts::unique().into_lib(),
            scripts::collection().into_lib(),
        ],
        api_libs: none!(),
        types: types.type_system(),
    };
    Issuer::new(codex, semantics).expect("invalid issuer")
}

fn codex() -> Codex {
    let lib = scripts::unique();
    let codex = Codex {
        name: tiny_s!("Non-Fungible Asset Collection"),
        developer: Identity::from(PANDORA),
        version: default!(),
        timestamp: 1732529307,
        field_order: FIELD_ORDER_SECP,
        input_config: CoreConfig::default(),
        verification_config: CoreConfig::default(),
        verifiers: tiny_bmap! {
            VERIFIER_GENESIS => lib.routine(FN_RGB21_ISSUE),
            VERIFIER_TRANSFER => lib.routine(FN_DIVISIBLE_TRANSFER),
        },
    };
    codex
}
