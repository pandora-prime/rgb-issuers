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
use hypersonic::uasm;
use zkaluvm::alu::CompiledLib;

use super::{shared_lib, FN_ASSET_SPEC, FN_GLOBAL_ABSENT};
use crate::{ERRNO_UNEXPECTED_GLOBAL_IN, G_NFT, O_AMOUNT};

pub const FN_UNIQUE_TRANSFER: u16 = 3;

pub const FN_GLOBAL_VERIFY_TOKEN: u16 = 1;
pub const FN_OWNED_TOKEN: u16 = 2;

pub const ERRNO_FRACTIONALITY: u256 = u256::from_inner([1, 2, 0, 0]);
pub const ERRNO_NO_TOKEN_ID: u256 = u256::from_inner([2, 2, 0, 0]);
pub const ERRNO_INVALID_TOKEN_ID: u256 = u256::from_inner([3, 2, 0, 0]);
pub const ERRNO_TOKEN_EXCESS: u256 = u256::from_inner([4, 2, 0, 0]);
pub const ERRNO_NO_INPUT: u256 = u256::from_inner([5, 2, 0, 0]);
pub const ERRNO_TOKEN_EXCESS_IN: u256 = u256::from_inner([6, 2, 0, 0]);
pub const ERRNO_NO_OUTPUT: u256 = u256::from_inner([7, 2, 0, 0]);
pub const ERRNO_TOKEN_EXCESS_OUT: u256 = u256::from_inner([8, 2, 0, 0]);

pub fn unique() -> CompiledLib {
    let shared = shared_lib().into_lib().lib_id();

    const VERIFY_GLOBAL_TOKEN: u16 = 4;
    const VERIFY_IN_TOKEN: u16 = 5;
    const VERIFY_OUT_TOKEN: u16 = 6;
    const VERIFY_TOKEN: u16 = 7;

    let mut code = uasm! {
    // Verification of unique token issue
    // Args: no
    // Returns: nothing
    proc FN_RGB21_ISSUE:
        call    shared, FN_ASSET_SPEC; // Call asset check.

        // Check that there is no fractionality
        put     E1, ERRNO_FRACTIONALITY; // Set error code for the case of failure
        put     EH, 1;
        eq      E4, EH;             // `E4` is returned from `FN_ASSET_SPEC` and contains fractions
        chk     CO;
        clr     EH;

        call    VERIFY_GLOBAL_TOKEN;// Verify token spec
        call    VERIFY_OUT_TOKEN;   // Verify the output token
        ret;

    // TODO: Put FN_GLOBAL_VERIFY_TOKEN and FN_OWNED_TOKEN into a separate library
    // Verify token spec
    // We export this procedure to be used in other libraries
    // Args: no
    // Returns: token id in `E3`
    proc FN_GLOBAL_VERIFY_TOKEN:
        put     E1, ERRNO_UNEXPECTED_GLOBAL_IN; // Set error code for the case of failure
        put     EH, G_NFT;      // Set E7 to field element representing token data
        eq      EA, EH;         // It must have the correct state type
        chk     CO;             // Or fail otherwise

        put     E1, ERRNO_NO_TOKEN_ID; // Set error code for the case of failure
        test    EB;             // Token id must be set
        chk     CO;             // Or we should fail
        mov     E3, EB;         // Save token id for returning it (used in VERIFY_AMOUNT)

        put     E1, ERRNO_INVALID_TOKEN_ID; // Set error code for the case of failure
        test    EC;             // ensure other field elements are empty
        not     CO;             // invert CO value (we need the test to fail)
        chk     CO;             // fail if not
        test    ED;             // ensure other field elements are empty
        not     CO;             // invert CO value (we need the test to fail)
        chk     CO;             // fail if not
        ret;

    // Get token allocation
    // We export this procedure to be used in other libraries
    // Args: none
    // Returns: token id in `E3`, fractions in `E4`
    proc FN_OWNED_TOKEN:
        put     EH, O_AMOUNT;   // Set E7 to field element representing token data
        eq      EA, EH;         // It must have the correct state type
        chk     CO;             // Or fail otherwise
        test    EB;             // Token id must be set
        chk     CO;             // Or we should fail
        mov     E3, EB;         // Save token id for returning it
        test    EC;             // Token fraction must be set
        chk     CO;             // Or we should fail
        mov     E4, EC;         // Save token fractions for returning it
        test    ED;             // ensure other field elements are empty
        not     CO;             // invert CO value (we need the test to fail)
        chk     CO;             // fail if not
        ret;

    // Verification of unique token transfer
    // Args: no
    // Returns: nothing
    proc FN_UNIQUE_TRANSFER:
        call    shared, FN_GLOBAL_ABSENT;
        call    VERIFY_IN_TOKEN;
        mov     E5, E3;         // Save the token id
        call    VERIFY_OUT_TOKEN;
        eq      E3, E5;         // Check that the tokens have the same id
        chk     CO;
        ret;

    routine VERIFY_GLOBAL_TOKEN:
        ldo     immutable;      // Read the fourth global state: token information
        call    FN_GLOBAL_VERIFY_TOKEN;// Verify token spec
        put     E1, ERRNO_TOKEN_EXCESS; // Set error code for the case of failure
        cknxo   immutable;      // Verify there are no more tokens
        not     CO;
        chk     CO;
        ret;

    routine VERIFY_IN_TOKEN:
        rsti    destructible;   // Restart the state iterator
        put     E1, ERRNO_NO_INPUT; // Set error code for the case of failure
        ldi     destructible;   // Read input token information
        chk     CO;
        call    VERIFY_TOKEN;   // Verify token fractions
        put     E1, ERRNO_TOKEN_EXCESS_IN; // Set error code for the case of failure
        cknxi   destructible;   // Verify there are no more tokens
        not     CO;
        chk     CO;
        ret;

    routine VERIFY_OUT_TOKEN:
        rsto    destructible;   // Restart the state iterator
        put     E1, ERRNO_NO_OUTPUT; // Set error code for the case of failure
        ldo     destructible;   // Read input token information
        chk     CO;
        call    VERIFY_TOKEN;   // Verify token fractions
        put     E1, ERRNO_TOKEN_EXCESS_OUT; // Set error code for the case of failure
        cknxo   destructible;   // Verify there are no more tokens
        not     CO;
        chk     CO;
        ret;

    routine VERIFY_TOKEN:
        call    FN_OWNED_TOKEN; // Get token fractions
        put     E1, ERRNO_FRACTIONALITY; // Set error code for the case of failure
        put     EH, 1;
        eq      E4, EH;         // Check there is no fractionality
        chk     CO;
        ret;
    };

    CompiledLib::compile(&mut code, &[&shared_lib()])
        .unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FN_RGB21_ISSUE, G_DETAILS, G_NAME, G_PRECISION, G_SUPPLY};
    use hypersonic::{AuthToken, Input, Instr, StateCell, StateData, StateValue, VmContext};
    use strict_types::StrictDumb;
    use zkaluvm::alu::{CoreConfig, Lib, LibId, Vm};
    use zkaluvm::{GfaConfig, FIELD_ORDER_SECP};

    const CONFIG: CoreConfig = CoreConfig {
        halt: true,
        complexity_lim: Some(580_000_000),
    };

    const TOKEN_ID: u64 = 0;
    const TOKEN_FRACTIONS: u64 = 1_u64;

    macro_rules! unique_token_in {
        () => {
            (
                Input {
                    addr: strict_dumb!(),
                    witness: StateValue::None,
                },
                unique_token_out!(),
            )
        };
    }
    macro_rules! unique_token_out {
        () => {
            StateCell {
                data: StateValue::Triple {
                    first: O_AMOUNT.into(),
                    second: TOKEN_ID.into(),
                    third: TOKEN_FRACTIONS.into(),
                },
                auth: AuthToken::strict_dumb(),
                lock: None,
            }
        };
    }

    fn harness() -> (CompiledLib, Vm<Instr<LibId>>, impl Fn(LibId) -> Option<Lib>) {
        let vm = Vm::<Instr<LibId>>::with(
            CONFIG,
            GfaConfig {
                field_order: FIELD_ORDER_SECP,
            },
        );
        fn resolver(id: LibId) -> Option<Lib> {
            let lib = unique();
            let unique = unique();
            let shared = shared_lib();
            if lib.as_lib().lib_id() == id {
                return Some(lib.into_lib());
            }
            if unique.as_lib().lib_id() == id {
                return Some(unique.into_lib());
            }
            if shared.as_lib().lib_id() == id {
                return Some(shared.into_lib());
            }
            panic!("Unknown library: {id}");
        }
        (unique(), vm, resolver)
    }

    #[test]
    fn genesis_empty() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        let mut context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, TOKEN_FRACTIONS),
            ][..],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_SUPPLY, TOKEN_FRACTIONS),
            ],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_SUPPLY, TOKEN_FRACTIONS),
            ],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_PRECISION, 1_u8),
            ],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_SUPPLY, 0u8)],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_missing_owned() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_PRECISION, 1_u8),
                StateData::new(G_SUPPLY, 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_supply_mismatch() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::Triple {
                    first: O_AMOUNT.into(),
                    second: TOKEN_ID.into(),
                    third: TOKEN_FRACTIONS.into(),
                },
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_PRECISION, 1_u8),
                StateData::new(G_SUPPLY, TOKEN_FRACTIONS + 1000_u64),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_nonunique() {
        const SUPPLY: u64 = 100_u64;
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::Triple {
                    first: O_AMOUNT.into(),
                    second: TOKEN_ID.into(),
                    third: SUPPLY.into(),
                },
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, SUPPLY),
                StateData::new(G_SUPPLY, TOKEN_ID),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_correct() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[
                StateData::new(G_DETAILS, 0u8),
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, TOKEN_FRACTIONS),
                StateData::new(G_SUPPLY, TOKEN_ID),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_RGB21_ISSUE), &context, resolver)
            .is_ok();
        assert!(res);
    }

    #[test]
    fn transfer_contains_globals() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[unique_token_in!()],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[StateData::new(G_DETAILS, 0u8)],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);

        let context = VmContext {
            witness: none!(),
            destructible_input: &[unique_token_in!()],
            immutable_input: &[StateValue::None],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_no_input() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_no_output() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[unique_token_in!()],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_wrong_in_id() {
        let mut inp = unique_token_in!();
        inp.1.data = StateValue::Triple {
            first: O_AMOUNT.into(),
            second: (TOKEN_ID + 1).into(),
            third: TOKEN_FRACTIONS.into(),
        };
        let context = VmContext {
            witness: none!(),
            destructible_input: &[inp],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_wrong_in_fractons() {
        let mut inp = unique_token_in!();
        inp.1.data = StateValue::Triple {
            first: O_AMOUNT.into(),
            second: TOKEN_ID.into(),
            third: (TOKEN_FRACTIONS + 1).into(),
        };
        let context = VmContext {
            witness: none!(),
            destructible_input: &[inp],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_wrong_out_id() {
        let mut token = unique_token_out!();
        token.data = StateValue::Triple {
            first: O_AMOUNT.into(),
            second: (TOKEN_ID + 1).into(),
            third: TOKEN_FRACTIONS.into(),
        };
        let context = VmContext {
            witness: none!(),
            destructible_input: &[unique_token_in!()],
            immutable_input: &[],
            destructible_output: &[token],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_wrong_out_fractons() {
        let mut token = unique_token_out!();
        token.data = StateValue::Triple {
            first: O_AMOUNT.into(),
            second: TOKEN_ID.into(),
            third: (TOKEN_FRACTIONS + 1).into(),
        };
        let context = VmContext {
            witness: none!(),
            destructible_input: &[unique_token_in!()],
            immutable_input: &[],
            destructible_output: &[token],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_wrong_inout_fractons() {
        let mut token = unique_token_out!();
        token.data = StateValue::Triple {
            first: O_AMOUNT.into(),
            second: TOKEN_ID.into(),
            third: (TOKEN_FRACTIONS + 1).into(),
        };
        let context = VmContext {
            witness: none!(),
            destructible_input: &[(Input::strict_dumb(), token)],
            immutable_input: &[],
            destructible_output: &[token],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn transfer_correct() {
        let context = VmContext {
            witness: none!(),
            destructible_input: &[unique_token_in!()],
            immutable_input: &[],
            destructible_output: &[unique_token_out!()],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_UNIQUE_TRANSFER), &context, resolver)
            .is_ok();
        assert!(res);
    }
}
