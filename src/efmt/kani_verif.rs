/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use super::{
    format::Format,
    formatter::{Formatter, Item},
};
use crate::parser::Token;
use crate::Epoch;
use crate::{duration::Duration, TimeScale};

#[cfg(kani)]
mod kani_harnesses {
    use super::*;
    use crate::*;
    #[kani::proof]
    fn kani_harness_need_gregorian() {
        let callee: Format = kani::any();
        callee.need_gregorian();
    }

    #[kani::proof]
    fn kani_harness_Item_new() {
        let token: Token = kani::any();
        let sep_char: Option<char> = kani::any();
        let second_sep_char: Option<char> = kani::any();
        Item::new(token, sep_char, second_sep_char);
    }

    #[kani::proof]
    fn kani_harness_sep_char_is() {
        let c_in: char = kani::any();
        let callee: Item = kani::any();
        callee.sep_char_is(c_in);
    }

    #[kani::proof]
    fn kani_harness_sep_char_is_not() {
        let c_in: char = kani::any();
        let callee: Item = kani::any();
        callee.sep_char_is_not(c_in);
    }

    #[kani::proof]
    fn kani_harness_second_sep_char_is() {
        let c_in: char = kani::any();
        let callee: Item = kani::any();
        callee.second_sep_char_is(c_in);
    }

    #[kani::proof]
    fn kani_harness_second_sep_char_is_not() {
        let c_in: char = kani::any();
        let callee: Item = kani::any();
        callee.second_sep_char_is_not(c_in);
    }

    #[kani::proof]
    fn kani_harness_Formatter_new() {
        let epoch: Epoch = kani::any();
        let format: Format = kani::any();
        Formatter::new(epoch, format);
    }

    #[kani::proof]
    fn kani_harness_Formatter_with_timezone() {
        let epoch: Epoch = kani::any();
        let offset: Duration = kani::any();
        let format: Format = kani::any();
        Formatter::with_timezone(epoch, offset, format);
    }

    #[kani::proof]
    fn kani_harness_Formatter_to_time_scale() {
        let epoch: Epoch = kani::any();
        let format: Format = kani::any();
        let time_scale: TimeScale = kani::any();
        Formatter::to_time_scale(epoch, format, time_scale);
    }

    #[kani::proof]
    fn kani_harness_set_timezone() {
        let offset: Duration = kani::any();
        let mut callee: Formatter = kani::any();
        callee.set_timezone(offset);
    }
}
