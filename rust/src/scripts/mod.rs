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

mod collection;
mod dividable;
mod fungible;
mod shared;
mod unique;

pub use collection::{collection, FN_FAC_TRANSFER};
pub use dividable::{fractionable, FN_UAC_TRANSFER};
pub use fungible::{fungible, FN_FUNGIBLE_ISSUE, FN_FUNGIBLE_TRANSFER};
pub use shared::shared_lib;
pub use unique::{unique, FN_UNIQUE_TRANSFER};

pub(self) use shared::{FN_ASSET_SPEC, FN_SUM_INPUTS, FN_SUM_OUTPUTS};
pub(self) use unique::{FN_GLOBAL_VERIFY_TOKEN, FN_OWNED_TOKEN};

pub const FN_RGB21_ISSUE: u16 = 0; // In all libs it must be the first method
