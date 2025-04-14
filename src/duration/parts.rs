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
        let days = value.to_integer_unit(Unit::Day);
        value -= days.days();
        let hours = value.to_integer_unit(Unit::Hour);
        value -= hours.hours();
        let minutes = value.to_integer_unit(Unit::Minute);
        value -= minutes.minutes();
        let seconds = value.to_integer_unit(Unit::Second);
        value -= seconds.seconds();
        let milliseconds = value.to_integer_unit(Unit::Millisecond);
        value -= milliseconds.milliseconds();
        let microseconds = value.to_integer_unit(Unit::Microsecond);
        value -= microseconds.microseconds();
        let nanoseconds = value.to_integer_unit(Unit::Nanosecond);
        value -= nanoseconds.nanoseconds();
        let picoseconds = value.to_integer_unit(Unit::Picosecond);
        value -= picoseconds.picoseconds();
        let femtoseconds = value.to_integer_unit(Unit::Femtosecond);
        value -= femtoseconds.femtoseconds();
        let attoseconds = value.to_integer_unit(Unit::Attosecond);
        value -= attoseconds.attoseconds();
        let zeptoseconds = value.to_integer_unit(Unit::Zeptosecond);

        // Everything should fit in the expected types now
        Self {
            sign,
            centuries: 0,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
            picoseconds,
            femtoseconds,
            attoseconds,
            zeptoseconds,
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
