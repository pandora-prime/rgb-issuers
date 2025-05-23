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
mod divisible;
mod fungible;
mod shared;
mod unique;

pub use collection::{collection, FN_FAC_TRANSFER};
pub use divisible::{divisible, FN_DIVISIBLE_TRANSFER, FN_NFT_SUM_INPUTS, FN_NFT_SUM_OUTPUTS};
pub use fungible::{
    fungible, ERRNO_INVALID_BALANCE_IN, ERRNO_INVALID_BALANCE_OUT, ERRNO_NO_ISSUED,
    ERRNO_PRECISION_OVERFLOW, ERRNO_SUM_ISSUE_MISMATCH, ERRNO_SUM_MISMATCH,
    ERRNO_UNEXPECTED_GLOBAL, ERRNO_UNEXPECTED_OWNED_TYPE_IN, ERRNO_UNEXPECTED_OWNED_TYPE_OUT,
    FN_FUNGIBLE_ISSUE, FN_FUNGIBLE_SUM_INPUTS, FN_FUNGIBLE_SUM_OUTPUTS, FN_FUNGIBLE_TRANSFER,
};
pub use shared::{
    shared_lib, ERRNO_INVALID_PRECISION, ERRNO_NO_NAME, ERRNO_NO_PRECISION, ERRNO_NO_TICKER,
    ERRNO_UNEXPECTED_GLOBAL_IN, ERRNO_UNEXPECTED_GLOBAL_OUT, ERRNO_UNEXPECTED_OWNED_IN,
    FN_ASSET_SPEC, FN_GLOBAL_ABSENT,
};
pub use unique::{
    unique, ERRNO_FRACTIONALITY, ERRNO_INVALID_TOKEN_ID, ERRNO_NO_INPUT, ERRNO_NO_OUTPUT,
    ERRNO_NO_TOKEN_ID, ERRNO_TOKEN_EXCESS, ERRNO_TOKEN_EXCESS_IN, ERRNO_TOKEN_EXCESS_OUT,
    FN_GLOBAL_VERIFY_TOKEN, FN_OWNED_TOKEN, FN_UNIQUE_TRANSFER,
};

pub const FN_RGB21_ISSUE: u16 = 0; // In all libs it must be the first method
