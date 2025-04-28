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

mod fungible;
mod non_fungible;
mod shared;

pub use fungible::{fungible, FN_FUNGIBLE_ISSUE, FN_FUNGIBLE_TRANSFER};
pub use non_fungible::{
    collection, fractionable, unique, FN_FAC_TRANSFER, FN_RGB21_ISSUE, FN_UAC_TRANSFER,
    FN_UDA_TRANSFER,
};
pub use shared::{shared_lib, FN_ASSET_SPEC, FN_SUM_INPUTS, FN_SUM_OUTPUTS};
