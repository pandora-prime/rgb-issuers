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

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_types;

#[cfg(not(feature = "std"))]
compile_error!("feature std must be used");

mod ifaces;
mod scripts;

pub use ifaces::*;
pub use scripts::*;

pub const PANDORA: &str = "dns:pandoraprime.ch";

use amplify::num::u256;

pub const G_NAME: u256 = u256::ZERO;
pub const G_TICKER: u256 = u256::ONE;
pub const G_PRECISION: u256 = u256::from_inner([2, 0, 0, 0]);
pub const G_SUPPLY: u256 = u256::from_inner([3, 0, 0, 0]);
pub const G_NFT: u256 = G_SUPPLY;
pub const G_DETAILS: u256 = G_TICKER;
pub const O_AMOUNT: u256 = u256::ZERO;

// TODO: Export codex constructors.
