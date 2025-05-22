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

use hypersonic::uasm;
use zkaluvm::alu::CompiledLib;

use crate::{G_DETAILS, G_NAME, G_PRECISION, G_TICKER, O_AMOUNT};

/// Checks globals defining assent specification to be present and contain the correct state type.
///
/// NB: Doesn't check that the values of that globals fulfill ASCII criteria (like length, use of
/// specific chars, etc.). This is not enforced by consensus here, and instead, the contract will
/// just fail to read its state under RGB20, 21, 25 or another interface.
///
/// # Input
///
/// Procedure takes no registry input.
///
/// It accepts the immutable outputs iterator at the current progress, without a reset.
///
/// # Output
///
/// `E4` contains the value of [`G_PRECISION`].
///
/// # Reset registers
///
/// `EA`-`ED`
///
/// # Side effects
///
/// Progresses immutable outputs iterator for three positions.
pub const FN_ASSET_SPEC: u16 = 0;

/// Ensure the global state is absent (both input and output).

/// # Input
///
/// None
///
/// # Output
///
/// None
///
/// # Side effects
///
/// Resets input and output global state iterators
pub const FN_GLOBAL_ABSENT: u16 = 1;

pub fn shared_lib() -> CompiledLib {
    assert_eq!(O_AMOUNT, G_NAME);
    assert_eq!(G_TICKER, G_DETAILS);

    let mut code = uasm! {
     proc FN_ASSET_SPEC:
        // There must be no inputs
        rsti    destructible;
        cknxi   immutable;
        not     CO;
        chk     CO;
        rsti    destructible;
        cknxi   destructible;
        not     CO;
        chk     CO;

        ldo     immutable;      // Read the first global state - ticker in RGB20, details in RGB21/25
        chk     CO;             // - it must exist
        put     EH, G_TICKER;   // - set E1 to the field element representing owned value (also global asset name)
        eq      EA, EH;         // - it must have the correct state type
        chk     CO;             // - - or fail otherwise

        ldo     immutable;      // Read the second global state - asset name
        chk     CO;             // - it must exist
        put     EH, G_NAME;     // - set E1 to a field element representing global asset ticker (or details)
        eq      EA, EH;         // - it must have the correct state type
        chk     CO;             // - - or fail otherwise

        ldo     immutable;      // The third global state - precision
        chk     CO;             // - it must exist
        put     EH, G_PRECISION;// - set E1 to a field element representing global fractions
        eq      EA, EH;         // - it must have the correct state type
        chk     CO;             // - - or fail otherwise
        test    EB;             // - there must be a value for the precision
        chk     CO;             // - or fail otherwise
        mov     E4, EB;         // Return G_PRECISION in `E4`
        test    EC;             // - there must be no other field elements than in EC in the precision
        not     CO;
        chk     CO;             // - or fail otherwise
        test    ED;             // - there must be no other field elements than in ED in the precision
        not     CO;
        chk     CO;             // - or fail otherwise

        // Clear up
        clr     EA;
        clr     EB;
        clr     EC;
        clr     ED;

        ret;

    proc FN_GLOBAL_ABSENT:
        rsti    immutable;
        cknxi   immutable;
        not     CO;
        chk     CO;
        rsto    immutable;
        cknxo   immutable;
        not     CO;
        chk     CO;
        ret;
    };

    CompiledLib::compile(&mut code, &[]).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypersonic::{AuthToken, Instr, StateCell, StateData, StateValue, VmContext};
    use strict_types::StrictDumb;
    use zkaluvm::alu::{CoreConfig, Lib, LibId, Vm};
    use zkaluvm::{GfaConfig, FIELD_ORDER_SECP};

    const CONFIG: CoreConfig = CoreConfig {
        halt: true,
        complexity_lim: Some(10_000_000_000),
    };

    fn harness() -> (CompiledLib, Vm<Instr<LibId>>, impl Fn(LibId) -> Option<Lib>) {
        let vm = Vm::<Instr<LibId>>::with(
            CONFIG,
            GfaConfig {
                field_order: FIELD_ORDER_SECP,
            },
        );
        fn resolver(id: LibId) -> Option<Lib> {
            let shared = shared_lib();
            if shared.as_lib().lib_id() == id {
                return Some(shared.into_lib());
            }
            panic!("Unknown library: {id}");
        }
        (shared_lib(), vm, resolver)
    }

    #[test]
    fn genesis_empty() {
        let context = VmContext {
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
            .is_ok();
        assert!(!res);
    }

    #[test]
    fn genesis_missing_globals() {
        let mut context = VmContext {
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ][..],
            &[
                StateData::new(G_NAME, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_TICKER, 0u8)],
            &[StateData::new(G_NAME, 0u8), StateData::new(G_DETAILS, 0u8)],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_wrong_order() {
        let mut context = VmContext {
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[StateCell {
                data: StateValue::new(O_AMOUNT, 1000_u64),
                auth: AuthToken::strict_dumb(),
                lock: None,
            }],
            immutable_output: &[],
        };
        let globals = [
            &[
                StateData::new(G_NAME, 1u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
            ][..],
            &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_NAME, 1u8),
            ],
            &[
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_NAME, 1u8),
            ],
            &[
                StateData::new(G_PRECISION, 18_u8),
                StateData::new(G_NAME, 1u8),
                StateData::new(G_TICKER, 0u8),
            ],
        ];
        for global in globals {
            context.immutable_output = global;
            let (lib, mut vm, resolver) = harness();
            let res = vm
                .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
                .is_ok();
            assert!(!res);
        }
    }

    #[test]
    fn genesis_correct() {
        let context = VmContext {
            destructible_input: &[],
            immutable_input: &[],
            destructible_output: &[],
            immutable_output: &[
                StateData::new(G_TICKER, 0u8),
                StateData::new(G_NAME, 1u8),
                StateData::new(G_PRECISION, 18_u8),
            ],
        };
        let (lib, mut vm, resolver) = harness();
        let res = vm
            .exec(lib.routine(FN_ASSET_SPEC), &context, resolver)
            .is_ok();
        assert!(res);
    }
}
