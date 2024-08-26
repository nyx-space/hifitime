/*
* Hifitime, part of the Nyx Space tools
* Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Apache
* v. 2.0. If a copy of the Apache License was not distributed with this
* file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
*
* Documentation: https://nyxspace.com/
*/

use typed_builder::TypedBuilder;

use super::{Duration, TimeUnits, Unit};

#[derive(Copy, Clone, Debug, TypedBuilder)]
pub struct DurationParts {
    #[builder(default = 1)]
    pub sign: i8,
    #[builder(default = 0)]
    pub centuries: i128,
    #[builder(default = 0)]
    pub days: i128,
    #[builder(default = 0)]
    pub hours: i128,
    #[builder(default = 0)]
    pub minutes: i128,
    #[builder(default = 0)]
    pub seconds: i128,
    #[builder(default = 0)]
    pub milliseconds: i128,
    #[builder(default = 0)]
    pub microseconds: i128,
    #[builder(default = 0)]
    pub nanoseconds: i128,
    #[builder(default = 0)]
    pub picoseconds: i128,
    #[builder(default = 0)]
    pub femtoseconds: i128,
    #[builder(default = 0)]
    pub attoseconds: i128,
    #[builder(default = 0)]
    pub zeptoseconds: i128,
}

impl From<Duration> for DurationParts {
    fn from(mut value: Duration) -> Self {
        let sign = value.signum();
        value = value.abs();
        let days = value.to_unit(Unit::Day).floor();
        value -= days.days();
        let hours = value.to_unit(Unit::Hour).floor();
        value -= hours.hours();
        let minutes = value.to_unit(Unit::Minute).floor();
        value -= minutes.minutes();
        let seconds = value.to_unit(Unit::Second).floor();
        value -= seconds.seconds();
        let milliseconds = value.to_unit(Unit::Millisecond).floor();
        value -= milliseconds.milliseconds();
        let microseconds = value.to_unit(Unit::Microsecond).floor();
        value -= microseconds.microseconds();
        let nanoseconds = value.to_unit(Unit::Nanosecond).floor();
        value -= nanoseconds.nanoseconds();
        let picoseconds = dbg!(dbg!(value.to_unit(Unit::Picosecond)).floor());
        value -= picoseconds.picoseconds();
        let femtoseconds = value.to_unit(Unit::Femtosecond).floor();
        value -= femtoseconds.femtoseconds();
        let attoseconds = value.to_unit(Unit::Attosecond).floor();
        value -= attoseconds.attoseconds();
        let zeptoseconds = value.to_unit(Unit::Zeptosecond).round();

        // Everything should fit in the expected types now
        Self {
            sign,
            centuries: 0,
            days: days as i128,
            hours: hours as i128,
            minutes: minutes as i128,
            seconds: seconds as i128,
            milliseconds: milliseconds as i128,
            microseconds: microseconds as i128,
            nanoseconds: nanoseconds as i128,
            picoseconds: picoseconds as i128,
            femtoseconds: femtoseconds as i128,
            attoseconds: attoseconds as i128,
            zeptoseconds: zeptoseconds as i128,
        }
    }
}

impl From<DurationParts> for Duration {
    fn from(value: DurationParts) -> Self {
        let me: Duration = value.days.days()
            + value.hours.hours()
            + value.minutes.minutes()
            + value.seconds.seconds()
            + value.milliseconds.milliseconds()
            + value.microseconds.microseconds()
            + value.nanoseconds.nanoseconds()
            + value.picoseconds.picoseconds()
            + value.femtoseconds.femtoseconds()
            + value.attoseconds.attoseconds()
            + value.zeptoseconds.zeptoseconds();
        if value.sign < 0 {
            -me
        } else {
            me
        }
    }
}
