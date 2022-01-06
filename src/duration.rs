extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate divrem;

use self::regex::Regex;
use self::serde::{de, Deserialize, Deserializer};
use self::divrem::{DivEuclid, DivRemEuclid};
use crate::{Errors, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use std::str::FromStr;



const SECONDS_PER_MINUTE_U: u64 = 60;
const MINUTES_PER_HOUR_U: u64 = 60;
const HOURS_PER_DAY_U: u64 = 24;
const SECONDS_PER_HOUR_U: u64 = SECONDS_PER_MINUTE_U * MINUTES_PER_HOUR_U;
const SECONDS_PER_DAY_U: u64 = SECONDS_PER_HOUR_U * HOURS_PER_DAY_U;
const DAYS_PER_CENTURY_U: u64 = 36_525;
const NS_PER_DAY_U: u64 = 1e9 as u64 * SECONDS_PER_DAY_U;
const NS_PER_CENTURY_U: u64 = DAYS_PER_CENTURY_U * NS_PER_DAY_U;
const ONE: u64 = 1_u64;

/// Defines generally usable durations for high precision math with Epoch
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Duration {
    ns : u64, // 1 century is about 3.1e18 ns, and max value of u64 is about 1e19.26
    centuries : i16 // +- 9.22e18 centuries is the possible range for a Duration
                    // Reducing the range could be a good tradeoff for a lowerm memory footprint
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}



macro_rules! impl_ops_f {
    ($type:ident) => {
        impl Mul<$type> for TimeUnit {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Century => {
                        Duration::from_days_f(q as f64 * DAYS_PER_CENTURY_U as f64)
                    }
                    TimeUnit::Day => Duration::from_days_f(q as f64),
                    TimeUnit::Hour => Duration::from_hours_f(q as f64),
                    TimeUnit::Minute => Duration::from_minutes_f(q as f64),
                    TimeUnit::Second => Duration::from_seconds_f(q as f64),
                    TimeUnit::Millisecond => Duration::from_milliseconds_f(q as f64),
                    TimeUnit::Microsecond => Duration::from_microseconds_f(q as f64),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds_f(q as f64),
                }
            }
        }

        impl Mul<TimeUnit> for $type {
            type Output = Duration;
            fn mul(self, q: TimeUnit) -> Duration {
                match q {
                    TimeUnit::Century => Duration::from_days_f(
                        self as f64 * DAYS_PER_CENTURY_U as f64,
                    ),
                    TimeUnit::Day => Duration::from_days_f(self as f64),
                    TimeUnit::Hour => Duration::from_hours_f(self as f64),
                    TimeUnit::Minute => Duration::from_minutes_f(self as f64),
                    TimeUnit::Second => Duration::from_seconds_f(self as f64),
                    TimeUnit::Millisecond => Duration::from_milliseconds_f(self as f64),
                    TimeUnit::Microsecond => Duration::from_microseconds_f(self as f64),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds_f(self as f64),
                }
            }
        }

        impl Mul<$type> for Duration {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                Duration::from_seconds_f(self.in_seconds() * q as f64)
            
            }
        }

        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Duration {
                Duration::from_seconds_f(self.in_seconds() / q as f64)

            }
        }

        impl Mul<Duration> for $type {
            type Output = Duration;
            fn mul(self, q: Duration) -> Duration {
                Duration::from_seconds_f(self as f64 * q.in_seconds())
            
            }
        }

        impl TimeUnitHelper for $type {}
    };
}

macro_rules! impl_ops_u {
    ($type:ident) => {
        impl Mul<$type> for TimeUnit {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Century => {
                        Duration::from_days_u(q as u128 * DAYS_PER_CENTURY_U as u128)
                    }
                    TimeUnit::Day => Duration::from_days_u(q as u128),
                    TimeUnit::Hour => Duration::from_hours_u(q as u128),
                    TimeUnit::Minute => Duration::from_minutes_u(q as u128),
                    TimeUnit::Second => Duration::from_seconds_u(q as u128),
                    TimeUnit::Millisecond => Duration::from_milliseconds_u(q as u128),
                    TimeUnit::Microsecond => Duration::from_microseconds_u(q as u128),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds_u(q as u128),
                }
            }
        }

        impl Mul<TimeUnit> for $type {
            type Output = Duration;
            fn mul(self, q: TimeUnit) -> Duration {
                match q {
                    TimeUnit::Century => Duration::from_days_u(
                        self as u128 * DAYS_PER_CENTURY_U as u128,
                    ),
                    TimeUnit::Day => Duration::from_days_u(self as u128),
                    TimeUnit::Hour => Duration::from_hours_u(self as u128),
                    TimeUnit::Minute => Duration::from_minutes_u(self as u128),
                    TimeUnit::Second => Duration::from_seconds_u(self as u128),
                    TimeUnit::Millisecond => Duration::from_milliseconds_u(self as u128),
                    TimeUnit::Microsecond => Duration::from_microseconds_u(self as u128),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds_u(self as u128),
                }
            }
        }

        impl Mul<$type> for Duration {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                Self::from_nanoseconds_i(self.total_ns() * q as i128)
                
            }
        }

        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Duration {
                Self::from_nanoseconds_i(self.total_ns() / q as i128)
                
            }
        }

        impl Mul<Duration> for $type {
            type Output = Duration;
            fn mul(self, q: Duration) -> Duration {
                Duration::from_nanoseconds_i(q.total_ns() * self as i128)
                
            }
        }

        impl TimeUnitHelper for $type {}
    };
}

macro_rules! impl_ops_i {
    ($type:ident) => {
        impl Mul<$type> for TimeUnit {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Century => {
                        Duration::from_days_i(q as i128 * DAYS_PER_CENTURY_U as i128)
                    }
                    TimeUnit::Day => Duration::from_days_i(q as i128),
                    TimeUnit::Hour => Duration::from_hours_i(q as i128),
                    TimeUnit::Minute => Duration::from_minutes_i(q as i128),
                    TimeUnit::Second => Duration::from_seconds_i(q as i128),
                    TimeUnit::Millisecond => Duration::from_milliseconds_i(q as i128),
                    TimeUnit::Microsecond => Duration::from_microseconds_i(q as i128),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds_i(q as i128),
                }
            }
        }

        impl Mul<TimeUnit> for $type {
            type Output = Duration;
            fn mul(self, q: TimeUnit) -> Duration {
                match q {
                    TimeUnit::Century => Duration::from_days_i(
                        self as i128 * DAYS_PER_CENTURY_U as i128,
                    ),
                    TimeUnit::Day => Duration::from_days_i(self as i128),
                    TimeUnit::Hour => Duration::from_hours_i(self as i128),
                    TimeUnit::Minute => Duration::from_minutes_i(self as i128),
                    TimeUnit::Second => Duration::from_seconds_i(self as i128),
                    TimeUnit::Millisecond => Duration::from_milliseconds_i(self as i128),
                    TimeUnit::Microsecond => Duration::from_microseconds_i(self as i128),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds_i(self as i128),
                }
            }
        }

        impl Mul<$type> for Duration {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                Self::from_nanoseconds_i(self.total_ns() * q as i128)
                
            }
        }

        impl Div<$type> for Duration {
            type Output = Duration;
            fn div(self, q: $type) -> Duration {
                Self::from_nanoseconds_i(self.total_ns() / q as i128)
                
            }
        }

        impl Mul<Duration> for $type {
            type Output = Duration;
            fn mul(self, q: Duration) -> Duration {
                Duration::from_nanoseconds_i(q.total_ns() * self as i128)
            }
        }

        impl TimeUnitHelper for $type {}
    };
}

impl Duration {
    pub fn new(ns : u64, centuries : i16) -> Self {
        let mut out = Duration { ns, centuries };
        out.normalize();
        out
    }

    pub fn total_ns(self) -> i128 {
        self.centuries as i128 * NS_PER_CENTURY_U as i128 + self.ns as i128
    }

    fn normalize(&mut self) {
        if self.ns > NS_PER_CENTURY_U as u64 {
            let carry = self.ns / NS_PER_CENTURY_U as u64;
            self.centuries = self.centuries.saturating_add(carry as i16);
            self.ns %= NS_PER_CENTURY_U as u64;
        }
    }

    pub fn from_value_f(mut value : f64, century_divider : u64, ns_multiplier : u64) -> Self {
        let centuries = (value.div_euclid(century_divider as f64)) as i16;
        value = value.rem_euclid(century_divider as f64);

        // Risks : Overflow, loss of precision, unexpected roundings
        let ns = (value * ns_multiplier as f64).round() as u64; 
        Self {
            ns, centuries
        }
    }
    pub fn from_days_f(days: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U;
        let ns_multiplier = SECONDS_PER_DAY_U * 1e9 as u64;
        Self::from_value_f(days, century_divider, ns_multiplier)
    }
    pub fn from_hours_f(hours: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U;
        let ns_multiplier = SECONDS_PER_HOUR_U * 1e9 as u64;
        Self::from_value_f(hours, century_divider, ns_multiplier)
    }
    pub fn from_minutes_f(minutes: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U;
        let ns_multiplier = SECONDS_PER_MINUTE_U * 1e9 as u64;
        Self::from_value_f(minutes, century_divider, ns_multiplier)
    }
    pub fn from_seconds_f(seconds: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U;
        let ns_multiplier = 1e9 as u64;
        Self::from_value_f(seconds, century_divider, ns_multiplier)
    }
    pub fn from_milliseconds_f(ms: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e3 as u64;
        let ns_multiplier = 1e6 as u64;
        Self::from_value_f(ms, century_divider, ns_multiplier)
    }
    pub fn from_microseconds_f(us: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e6 as u64;
        let ns_multiplier = 1e3 as u64;
        Self::from_value_f(us, century_divider, ns_multiplier)
    }
    pub fn from_nanoseconds_f(ns: f64) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e9 as u64;
        let ns_multiplier = 1;
        Self::from_value_f(ns, century_divider, ns_multiplier)
    }



    pub fn from_value_u(mut value : u128, century_divider : u64, ns_multiplier : u64) -> Self {
        let centuries = (value.div_euclid(century_divider as u128)) as i16;
        value = value.rem_euclid(century_divider as u128);

        // Risks : Overflow, loss of precision, unexpected roundings
        let ns = (value * ns_multiplier as u128) as u64; 
        Self {
            ns, centuries
        }
    }
    pub fn from_days_u(days: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U;
        let ns_multiplier = SECONDS_PER_DAY_U * 1e9 as u64;
        Self::from_value_u(days, century_divider, ns_multiplier)
    }
    pub fn from_hours_u(hours: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U;
        let ns_multiplier = SECONDS_PER_HOUR_U * 1e9 as u64;
        Self::from_value_u(hours, century_divider, ns_multiplier)
    }
    pub fn from_minutes_u(minutes: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U;
        let ns_multiplier = SECONDS_PER_MINUTE_U * 1e9 as u64;
        Self::from_value_u(minutes, century_divider, ns_multiplier)
    }
    pub fn from_seconds_u(seconds: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U;
        let ns_multiplier = 1e9 as u64;
        Self::from_value_u(seconds, century_divider, ns_multiplier)
    }
    pub fn from_milliseconds_u(ms: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e3 as u64;
        let ns_multiplier = 1e6 as u64;
        Self::from_value_u(ms, century_divider, ns_multiplier)
    }
    pub fn from_microseconds_u(us: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e6 as u64;
        let ns_multiplier = 1e3 as u64;
        Self::from_value_u(us, century_divider, ns_multiplier)
    }
    pub fn from_nanoseconds_u(ns: u128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e9 as u64;
        let ns_multiplier = 1 as u64;
        Self::from_value_u(ns, century_divider, ns_multiplier)
    }



    
    pub fn from_value_i(mut value : i128, century_divider : u64, ns_multiplier : u64) -> Self {
        let centuries = (value.div_euclid(century_divider as i128)) as i16;
        value = value.rem_euclid(century_divider as i128);

        // Risks : Overflow, loss of precision, unexpected roundings
        let ns = (value * ns_multiplier as i128) as u64; 
        Self {
            ns, centuries
        }
    }
    pub fn from_days_i(days: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U;
        let ns_multiplier = SECONDS_PER_DAY_U * 1e9 as u64;
        Self::from_value_i(days, century_divider, ns_multiplier)
    }
    pub fn from_hours_i(hours: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U;
        let ns_multiplier = SECONDS_PER_HOUR_U * 1e9 as u64;
        Self::from_value_i(hours, century_divider, ns_multiplier)
    }
    pub fn from_minutes_i(minutes: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U;
        let ns_multiplier = SECONDS_PER_MINUTE_U * 1e9 as u64;
        Self::from_value_i(minutes, century_divider, ns_multiplier)
    }
    pub fn from_seconds_i(seconds: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U;
        let ns_multiplier = 1e9 as u64;
        Self::from_value_i(seconds, century_divider, ns_multiplier)
    }
    pub fn from_milliseconds_i(ms: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e3 as u64;
        let ns_multiplier = 1e6 as u64;
        Self::from_value_i(ms, century_divider, ns_multiplier)
    }
    pub fn from_microseconds_i(us: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e6 as u64;
        let ns_multiplier = 1e3 as u64;
        Self::from_value_i(us, century_divider, ns_multiplier)
    }
    pub fn from_nanoseconds_i(ns: i128) -> Self {
        let century_divider = DAYS_PER_CENTURY_U * HOURS_PER_DAY_U * MINUTES_PER_HOUR_U * SECONDS_PER_MINUTE_U * 1e9 as u64;
        let ns_multiplier = 1 as u64;
        Self::from_value_i(ns, century_divider, ns_multiplier)
    }





    /// Creates a new duration from the provided unit
    pub fn from_f64(value: f64, unit: TimeUnit) -> Self {
        unit * value
    }

    /// Returns this duration in f64 in the provided unit.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    pub fn in_unit_f64(&self, unit: TimeUnit) -> f64 {
        self.in_unit(unit)
    }

    /// Returns this duration in seconds f64.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    pub fn in_seconds(&self) -> f64 {
        (self.ns as f64 / 1e9) + (self.centuries as i64 * DAYS_PER_CENTURY_U as i64 * SECONDS_PER_DAY_U as i64) as f64
    }

    /// Returns the value of this duration in the requested unit.
    pub fn in_unit(&self, unit: TimeUnit) -> f64 {
        self.in_seconds() * unit.from_seconds()
    }

    /// Returns the absolute value of this duration
    #[must_use]
    pub fn abs(&self) -> Self {
        if self.centuries < 0 { -*self } else { *self }
    }

    

    pub fn decompose(&self) -> (i8, u64, u64, u64, u64, u64, u64, u64, u64, u64) {

        let total_ns : i128 = self.centuries as i128 * NS_PER_CENTURY_U as i128 + self.ns as i128;

        let sign = total_ns.signum() as i8;
        let mut ns_left = total_ns.abs() as u64;


        let centuries = ns_left / NS_PER_CENTURY_U;

        
        let years = ns_left / (1e9 as u64 * SECONDS_PER_DAY_U as u64 * 365);
        ns_left %= 1e9 as u64 * SECONDS_PER_DAY_U as u64 * 365;


        let days = ns_left / (1e9 as u64 * SECONDS_PER_DAY_U as u64);
        ns_left %= 1e9 as u64 * SECONDS_PER_DAY_U as u64;

        let hours = ns_left / (1e9 as u64 * SECONDS_PER_HOUR_U as u64);
        ns_left %= 1e9 as u64 * SECONDS_PER_HOUR_U as u64;

        let minutes = ns_left / (1e9 as u64 * SECONDS_PER_MINUTE_U as u64);
        ns_left %= 1e9 as u64 * SECONDS_PER_MINUTE_U as u64;

        let seconds = ns_left / (1e9 as u64);
        ns_left %= 1e9 as u64;

        let ms = ns_left / (1e6 as u64);
        ns_left %= 1e6 as u64;

        let us = ns_left / (1e3 as u64);
        ns_left %= 1e3 as u64;

        let ns = dbg!(ns_left);

        (sign, centuries, years, days, hours, minutes, seconds, ms, us, ns)
    }
}

impl fmt::Display for Duration {
    // Prints this duration with automatic selection of the highest and sub-second unit
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let (sign, centuries, years, days, hours, minutes, seconds, milli, us, nano) = self.decompose();
        
        let values = [centuries, years, days, hours, minutes, seconds, milli, us, nano];
        let names = ["centuries", "years", "days", "h", "min", "s", "ms", "us", "ns"];
        
        let print_all = false;

        let mut interval_start = None;
        let mut interval_end = None;

        if print_all {
            interval_start = Some(0);
            interval_end = Some(values.len()-1);
        } else {
            for index in 0..values.len() {
                if interval_start.is_none() {
                    if values[index] > 0 { 
                        interval_start = Some(index);
                        interval_end = Some(index);
                    }
                } else {
                    if values[index] > 0 {
                        interval_end = Some(index);
                    }
                }
            }
        }
        assert!(interval_start.is_some());
        assert!(interval_end.is_some());

        if sign == -1 {
            write!(f, "-")?;
        }
        
        for i in interval_start.unwrap()..=interval_end.unwrap() {
            write!(f, "{} {} ", values[i], names[i])?;
        }


        Ok(())

    }
}

impl fmt::LowerExp for Duration {
    // Prints the duration with appropriate units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds_f64 = self.in_seconds();
        let seconds_f64_abs = seconds_f64.abs();
        if seconds_f64_abs < 1e-5 {
            fmt::Display::fmt(&(seconds_f64 * 1e9), f)?;
            write!(f, " ns")
        } else if seconds_f64_abs < 1e-2 {
            fmt::Display::fmt(&(seconds_f64 * 1e3), f)?;
            write!(f, " ms")
        } else if seconds_f64_abs < 3.0 * SECONDS_PER_MINUTE {
            fmt::Display::fmt(&(seconds_f64), f)?;
            write!(f, " s")
        } else if seconds_f64_abs < SECONDS_PER_HOUR {
            fmt::Display::fmt(&(seconds_f64 / SECONDS_PER_MINUTE), f)?;
            write!(f, " min")
        } else if seconds_f64_abs < SECONDS_PER_DAY {
            fmt::Display::fmt(&(seconds_f64 / SECONDS_PER_HOUR), f)?;
            write!(f, " h")
        } else {
            fmt::Display::fmt(&(seconds_f64 / SECONDS_PER_DAY), f)?;
            write!(f, " days")
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Duration {

        // Usage of u128 is to avoid possibility of overflow and type-system carry from u64
        // may switch approach when carrying_add is stabilized
        let mut total_ns: u128 = self.ns as u128 + rhs.ns as u128;
        let mut century_carry = 0;
        if total_ns > NS_PER_CENTURY_U as u128 {
            century_carry = total_ns / NS_PER_CENTURY_U as u128;
            total_ns %=  NS_PER_CENTURY_U as u128;
            // total_ns is now guaranteed to be less than u64_max
        }
        
        let total_centuries = 
            self.centuries
                .saturating_add(rhs.centuries)
                .saturating_add(century_carry as i16); 
                
        Self { ns : total_ns as u64, centuries : total_centuries }
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        let mut total_ns: i128 = self.ns as i128 - rhs.ns as i128;
        let mut century_borrow = 0;
        if total_ns < 0 {
            century_borrow = (-total_ns / NS_PER_CENTURY_U as i128)+1;
            total_ns += century_borrow * NS_PER_CENTURY_U as i128;
        }


        let total_centuries = self.centuries.saturating_sub(rhs.centuries).saturating_sub(century_borrow as i16); 
        Self { ns : total_ns as u64, centuries : total_centuries }
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

// Allow adding with a TimeUnit directly
impl Add<TimeUnit> for Duration {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: TimeUnit) -> Duration {
        self + rhs * 1
    }
}

impl AddAssign<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn add_assign(&mut self, rhs: TimeUnit) {
        *self = *self + rhs * 1;
    }
}

impl Sub<TimeUnit> for Duration {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn sub(self, rhs: TimeUnit) -> Duration {
        self - rhs * 1
    }
}

impl SubAssign<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn sub_assign(&mut self, rhs: TimeUnit) {
        *self = *self - rhs * 1;
    }
}

impl PartialEq<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn eq(&self, unit: &TimeUnit) -> bool {
        *self == *unit * 1
    }
}

impl PartialOrd<TimeUnit> for Duration {
    #[allow(clippy::identity_op)]
    fn partial_cmp(&self, unit: &TimeUnit) -> Option<Ordering> {
        let unit_deref = *unit;
        let unit_as_duration: Duration = unit_deref * 1;
        if self < &unit_as_duration {
            Some(Ordering::Less)
        } else if self > &unit_as_duration {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Neg for Duration {
    type Output = Duration;

    fn neg(self) -> Self::Output {
        Self { ns: NS_PER_CENTURY_U - self.ns, centuries : -self.centuries-1 }
    }
}

impl FromStr for Duration {
    type Err = Errors;

    /// Attempts to convert a simple string to a Duration. Does not yet support complicated durations.
    ///
    /// Identifiers:
    ///  + d, days, day
    ///  + h, hours, hour
    ///  + min, mins, minute
    ///  + s, second, seconds
    ///  + ms, millisecond, milliseconds
    ///  + us, microsecond, microseconds
    ///  + ns, nanosecond, nanoseconds
    ///
    /// # Example
    /// ```
    /// use hifitime::{Duration, TimeUnit};
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Duration::from_str("1 d").unwrap(), TimeUnit::Day * 1);
    /// assert_eq!(Duration::from_str("10.598 days").unwrap(), TimeUnit::Day * 10.598);
    /// assert_eq!(Duration::from_str("10.598 min").unwrap(), TimeUnit::Minute * 10.598);
    /// assert_eq!(Duration::from_str("10.598 us").unwrap(), TimeUnit::Microsecond * 10.598);
    /// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), TimeUnit::Second * 10.598);
    /// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), TimeUnit::Nanosecond * 10.598);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reg = Regex::new(r"^(\d+\.?\d*)\W*(\w+)$").unwrap();
        match reg.captures(s) {
            Some(cap) => {
                let value = cap[1].to_owned().parse::<f64>().unwrap();
                match cap[2].to_owned().to_lowercase().as_str() {
                    "d" | "days" | "day" => Ok(TimeUnit::Day * value),
                    "h" | "hours" | "hour" => Ok(TimeUnit::Hour * value),
                    "min" | "mins" | "minute" | "minutes" => Ok(TimeUnit::Minute * value),
                    "s" | "second" | "seconds" => Ok(TimeUnit::Second * value),
                    "ms" | "millisecond" | "milliseconds" => Ok(TimeUnit::Millisecond * value),
                    "us" | "microsecond" | "microseconds" => Ok(TimeUnit::Microsecond * value),
                    "ns" | "nanosecond" | "nanoseconds" => Ok(TimeUnit::Nanosecond * value),
                    _ => Err(Errors::ParseError(format!(
                        "unknown duration unit in `{}`",
                        s
                    ))),
                }
            }
            None => Err(Errors::ParseError(format!(
                "Could not parse duration: `{}`",
                s
            ))),
        }
    }
}

/// A trait to automatically convert some primitives to a duration
///
/// ```
/// use hifitime::prelude::*;
/// use std::str::FromStr;
///
/// assert_eq!(Duration::from_str("1 d").unwrap(), 1.days());
/// assert_eq!(Duration::from_str("10.598 days").unwrap(), 10.598_f64.days());
/// assert_eq!(Duration::from_str("10.598 min").unwrap(), 10.598_f64.minutes());
/// assert_eq!(Duration::from_str("10.598 us").unwrap(), 10.598_f64.microseconds());
/// assert_eq!(Duration::from_str("10.598 seconds").unwrap(), 10.598_f64.seconds());
/// assert_eq!(Duration::from_str("10.598 nanosecond").unwrap(), 10.598_f64.nanoseconds());
/// ```
pub trait TimeUnitHelper: Copy + Mul<TimeUnit, Output = Duration> {
    fn centuries(self) -> Duration {
        self * TimeUnit::Century
    }
    fn days(self) -> Duration {
        self * TimeUnit::Day
    }
    fn hours(self) -> Duration {
        self * TimeUnit::Hour
    }
    fn minutes(self) -> Duration {
        self * TimeUnit::Minute
    }
    fn seconds(self) -> Duration {
        self * TimeUnit::Second
    }
    fn milliseconds(self) -> Duration {
        self * TimeUnit::Millisecond
    }
    fn microseconds(self) -> Duration {
        self * TimeUnit::Microsecond
    }
    fn nanoseconds(self) -> Duration {
        self * TimeUnit::Nanosecond
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeUnit {
    /// 36525 days, it the number of days per century in the Julian calendar
    Century,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl Add for TimeUnit {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn add(self, rhs: Self) -> Duration {
        self * 1 + rhs * 1
    }
}

impl Sub for TimeUnit {
    type Output = Duration;

    #[allow(clippy::identity_op)]
    fn sub(self, rhs: Self) -> Duration {
        self * 1 - rhs * 1
    }
}

impl TimeUnit {
    pub fn in_seconds(self) -> f64 {
        match self {
            TimeUnit::Century => DAYS_PER_CENTURY_U as f64 * SECONDS_PER_DAY_U as f64,
            TimeUnit::Day => SECONDS_PER_DAY_U as f64,
            TimeUnit::Hour => SECONDS_PER_HOUR_U as f64,
            TimeUnit::Minute => SECONDS_PER_MINUTE_U as f64,
            TimeUnit::Second => ONE as f64,
            TimeUnit::Millisecond => 1e-3,
            TimeUnit::Microsecond => 1e-6,
            TimeUnit::Nanosecond => 1e-9,
        }
    }

    pub fn in_seconds_f64(self) -> f64 {
        self.in_seconds()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_seconds(self) -> f64 {
        1.0 / self.in_seconds()
    }
}



impl_ops_u!(u8);
impl_ops_i!(i8);

impl_ops_u!(u16);
impl_ops_i!(i16);

impl_ops_u!(u32);
impl_ops_i!(i32);

impl_ops_u!(u64);
impl_ops_i!(i64);

impl_ops_u!(u128);
impl_ops_i!(i128);

impl_ops_f!(f32);
impl_ops_f!(f64);



#[test]
fn time_unit() {
    use std::f64::EPSILON;
    // Check that the same number is created for different types
    assert_eq!(TimeUnit::Day * 10.0, TimeUnit::Day * 10);
    assert_eq!(TimeUnit::Hour * -7.0, TimeUnit::Hour * -7);
    assert_eq!(TimeUnit::Minute * -2.0, TimeUnit::Minute * -2);
    assert_eq!(TimeUnit::Second * 3.0, TimeUnit::Second * 3);
    assert_eq!(TimeUnit::Millisecond * 4.0, TimeUnit::Millisecond * 4);
    assert_eq!(TimeUnit::Nanosecond * 5.0, TimeUnit::Nanosecond * 5);

    // Check the LHS multiplications match the RHS ones
    assert_eq!(10.0 * TimeUnit::Day, TimeUnit::Day * 10);
    assert_eq!(-7 * TimeUnit::Hour, TimeUnit::Hour * -7.0);
    assert_eq!(-2.0 * TimeUnit::Minute, TimeUnit::Minute * -2);
    assert_eq!(3.0 * TimeUnit::Second, TimeUnit::Second * 3);
    assert_eq!(4.0 * TimeUnit::Millisecond, TimeUnit::Millisecond * 4);
    assert_eq!(5.0 * TimeUnit::Nanosecond, TimeUnit::Nanosecond * 5);

    let d: Duration = 1.0 * TimeUnit::Hour / 3 - 20 * TimeUnit::Minute;
    assert!(d.abs() < TimeUnit::Nanosecond);
    assert_eq!(3 * (20 * TimeUnit::Minute), TimeUnit::Hour);

    // Test operations
    let seven_hours = TimeUnit::Hour * 7;
    let six_minutes = TimeUnit::Minute * 6;
    // let five_seconds = TimeUnit::Second * 5.0;
    let five_seconds = 5.0.seconds();
    let sum: Duration = seven_hours + six_minutes + five_seconds;
    assert!((sum.in_seconds() - 25565.0).abs() < EPSILON);

    let neg_sum = dbg!(-dbg!(sum));
    assert!(dbg!((dbg!(neg_sum.in_seconds()) + dbg!(25565.0)).abs()) < EPSILON);

    assert_eq!(neg_sum.abs(), sum, "abs failed");

    let sub: Duration = seven_hours - six_minutes - five_seconds;
    assert!((sub.in_seconds() - 24835.0).abs() < EPSILON);

    // Test fractional
    let quarter_hour = 0.25 * TimeUnit::Hour;
    let third_hour = (1.0 / 3.0) * TimeUnit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    assert!((sum.in_unit_f64(TimeUnit::Minute) - 35.0).abs() < EPSILON);
    println!(
        "Duration: {}\nFloating: {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );

    let quarter_hour = -0.25 * TimeUnit::Hour;
    let third_hour: Duration = -1 * TimeUnit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.in_unit(TimeUnit::Millisecond).floor()
        - sum.in_unit(TimeUnit::Second).floor() * 1000.0;
    println!("{:?}", delta * -1.0 == 0.0);
    assert!((sum.in_unit_f64(TimeUnit::Minute) + 35.0).abs() < EPSILON);
}

#[test]
fn duration_print() {
    // Check printing adds precision
    assert_eq!(
        format!("{}", TimeUnit::Day * 10.0 + TimeUnit::Hour * 5).trim(),
        "10 days 5 h"
    );

    assert_eq!(
        format!("{}", TimeUnit::Hour * 5 + TimeUnit::Millisecond * 256).trim(),
        "5 h 0 min 0 s 256 ms"
    );

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * 5 + TimeUnit::Millisecond * 256 + TimeUnit::Nanosecond
        ).trim(),
        "5 h 0 min 0 s 256 ms 0 us 1 ns"
    );

    assert_eq!(
        format!("{}", TimeUnit::Hour + TimeUnit::Second).trim(),
        "1 h 0 min 1 s"
    );

    // Check printing negative durations only shows one negative sign
    assert_eq!(
        format!("{}", TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256).trim(),
        "-5 h 0 min 0 s 256 ms"
    );

    let d : Duration = TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * -3;
    dbg!(d.in_seconds());

    assert_eq!(
        format!(
            "{}",
            TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + dbg!(TimeUnit::Nanosecond * -3)
        ).trim(),
        "-5 h 0 min 0 s 256 ms 0 us 3 ns"
    );

    assert_eq!(
        format!(
            "{}",
            (TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256)
                - (TimeUnit::Hour * -5 + TimeUnit::Millisecond * -256 + TimeUnit::Nanosecond * 2)
        ).trim(),
        "-2 ns"
    );

    assert_eq!(
        format!("{}", Duration::from_nanoseconds_f(2.0)).trim(),
        "2 ns"
    );

    // Check that we support nanoseconds pas GPS time
    let now = TimeUnit::Nanosecond * 1286495254000000203_u128;
    assert_eq!(
        format!("{}", now).trim(),
        "40 years 289 days 23 h 47 min 34 s 0 ms 0 us 203 ns"
    );

    let arbitrary = 14889.days()
        + 23.hours()
        + 47.minutes()
        + 34.seconds()
        + 0.milliseconds()
        + 123.nanoseconds();
    assert_eq!(
        format!("{}", arbitrary).trim(),
        "40 years 289 days 23 h 47 min 34 s 0 ms 0 us 123 ns"
    );

    // Test fractional
    let quarter_hour = 0.25 * TimeUnit::Hour;
    let third_hour = (1.0 / 3.0) * TimeUnit::Hour;
    let sum: Duration = quarter_hour + third_hour;
    println!(
        "Duration: {}\nFloating: {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum).trim(), "35 min"); // Note the automatic unit selection

    let quarter_hour = -0.25 * TimeUnit::Hour;
    let third_hour: Duration = -1 * TimeUnit::Hour / 3;
    let sum: Duration = quarter_hour + third_hour;
    let delta = sum.in_unit(TimeUnit::Millisecond).floor()
        - sum.in_unit(TimeUnit::Second).floor() * 1000.0;
    println!("{:?}", delta * -1.0 == 0.0); // This floating-point comparison looks wrong
    assert_eq!(format!("{}", sum).trim(), "-35 min"); // Note the automatic unit selection
}

#[test]
fn deser_test() {
    use self::serde_derive::Deserialize;
    #[derive(Deserialize)]
    struct _D {
        pub _d: Duration,
    }
}
