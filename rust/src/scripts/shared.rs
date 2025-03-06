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

use crate::{G_DETAILS, G_NAME, G_PRECISION, G_SUPPLY, G_TICKER, O_AMOUNT};

/// Checks globals defining assent specification to be present and contain correct state type.
///
/// NB: Doesn't check that the values of that globals fulfill ASCII criteria (like length, use of
/// specific chars etc.). This is not enforced by consensus here and instead, the contract will just
/// fail to read it's state under RGB20, 21, 25 or other interface.
///
/// # Input
///
/// Procedure takes no input
///
/// # Output
///
/// `EB` contains the value of [`G_PRECISION`].
///
/// # Reset registers
///
/// `E1`, `EA`, `EC`-`ED`
pub const FN_ASSET_SPEC: u16 = 0;

/// Sums input owned state
///
/// # Input
///
/// - `EE`: value expected to be present in the third field element (`EC` register returned from
///   `ldi`). If the value of register is `None` and `EC` is not `None`, the procedure fails.
///   Otherwise, if the value in `EC` and `EE` is not equal, the procedure skips that input.
///
/// # Output
///
/// `E1` contains the sum of inputs.
///
/// # Reset registers
///
/// `EA`-`ED`, `E8`.
pub const FN_SUM_INPUTS: u16 = 1;

/// Sums output owned state
///
/// # Input
///
/// - `EE`: value expected to be present in the third field element (`EC` register returned from
///   `ldi`). If the value of register is `None` and `EC` is not `None`, the procedure fails.
///   Otherwise, if the value in `EC` and `EE` is not equal, the procedure skips that input.
///
/// # Output
///
/// `E2` contains the sum of inputs.
///
/// # Reset registers
///
/// `EA`-`ED`, `E8`.
pub const FN_SUM_OUTPUTS: u16 = 3;

pub(self) const LOOP_INPUTS: u16 = 2;
pub(self) const LOOP_OUTPUTS: u16 = 4;

pub fn shared_lib() -> CompiledLib {
    assert_eq!(O_AMOUNT, G_NAME);
    assert_eq!(G_TICKER, G_DETAILS);

    let mut code = uasm! {
    // .proc FN_ASSET_SPEC
        nop                     ;// Marks start of routine / entry point / goto target

        ldo     :immutable      ;// Read first global state - name
        chk     CO              ;// - it must exist
        mov     E1, :O_AMOUNT   ;// - set E1 to the field element representing owned value (also global asset name)
        eq      EA, E1          ;// - it must have correct state type
        chk     CO              ;// - - or fail otherwise

        ldo     :immutable      ;// Read second global state (ticker for RGB20, details for RGB25)
        chk     CO              ;// - it must exist
        mov     E1, :G_DETAILS  ;// - set E1 to field element representing global asset ticker (or details)
        eq      EA, E1          ;// - it must have correct state type
        chk     CO              ;// - - or fail otherwise

        ldo     :immutable      ;// Third global state - precision
        chk     CO              ;// - it must exist
        mov     E1, :G_PRECISION;// - set E1 to field element representing global fractions
        eq      EA, E1          ;// - it must have correct state type
        chk     CO              ;// - - or fail otherwise
        test    EC              ;// - there must be no other field elements than in EC in the precision
        not     CO;
        chk     CO              ;// - or fail otherwise
        test    ED              ;// - there must be no other field elements than in ED in the precision
        not     CO;
        chk     CO              ;// - or fail otherwise

        mov     E1, :G_SUPPLY   ;// Set EF to field element representing global supply

        // Clear up
        clr     E1;
        clr     EA;
        clr     EC;
        clr     ED;

        ret;

    // .proc FN_SUM_INPUTS
        nop                     ;// Marks start of routine / entry point / goto target
        mov     E1, 0           ;// Set initial sum to zero
        mov     E8, :O_AMOUNT   ;// Set EE to the field element representing owned value
        rsti    :destructible   ;// Start iteration over inputs

    // .loop LOOP_INPUTS
        nop;
        ldi     :destructible   ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, E8          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EE              ;// Is `EE` set to none?
        jif     CO, +7          ;// - branch to enforce `EC` to be none as well

        eq      EC, EE          ;// ensure EC value equals to EE
        jif     CO, :LOOP_INPUTS;// - read next input otherwise
        jmp     +4              ;// process to normal flow

        eq      EC, EE          ;// ensure EC is not set
        not     CO;
        chk     CO              ;// fail if not

        test    ED              ;// ensure ED is not set
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 64:bits     ;// ensure the value fits in u64
        add     E1, EB          ;// add input to input accumulator
        fits    E1, 64:bits     ;// ensure we do not overflow

        jmp     :LOOP_INPUTS    ;// loop

    // .proc FN_SUM_OUTPUTS
        nop                     ;// Marks start of routine / entry point / goto target
        mov     E2, 0           ;// Set initial sum to zero
        mov     E8, :O_AMOUNT   ;// Set EE to the field element representing owned value
        rsto    :destructible   ;// Start iteration over outputs

    // .loop LOOP_OUTPUTS
        nop;
        ldo     :destructible   ;// load next state value

        // Finish if no more elements are present
        not     CO;
        jif     CO, +3;
        ret;

        eq      EA, E8          ;// do we have a correct state type?
        chk     CO              ;// fail if not

        test    EE              ;// Is `EE` set to none?
        jif     CO, +7          ;// - branch to enforce `EC` to be none as well

        eq      EC, EE          ;// ensure EC value equals to EE
        jif     CO, :LOOP_OUTPUTS;// - read next input otherwise
        jmp     +4              ;// process to normal flow

        eq      EC, EE          ;// ensure EC is not set
        not     CO;
        chk     CO              ;// fail if not

        test    ED              ;// ensure ED is not set
        not     CO;
        chk     CO              ;// fail if not

        fits    EB, 64:bits     ;// ensure the value fits in u64
        add     E2, EB          ;// add input to input accumulator
        fits    E2, 64:bits     ;// ensure we do not overflow

        jmp     :LOOP_OUTPUTS   ;// loop
    };

    CompiledLib::compile(&mut code, &[]).unwrap_or_else(|err| panic!("Invalid script: {err}"))
}
