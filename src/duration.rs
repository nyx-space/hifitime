use crate::fraction::ToPrimitive;
use crate::{Decimal, Fraction, SECONDS_PER_DAY, SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

/// Defines generally usable durations for high precision math with Epoch (all data is stored in seconds)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Duration(Decimal);

macro_rules! impl_ops_for_type {
    ($type:ident) => {
        impl Mul<$type> for TimeUnit {
            type Output = Duration;
            fn mul(self, q: $type) -> Duration {
                match self {
                    TimeUnit::Day => Duration::from_days(Decimal::from(q)),
                    TimeUnit::Hour => Duration::from_hours(Decimal::from(q)),
                    TimeUnit::Minute => Duration::from_minutes(Decimal::from(q)),
                    TimeUnit::Second => Duration::from_seconds(Decimal::from(q)),
                    TimeUnit::Millisecond => Duration::from_millseconds(Decimal::from(q)),
                    TimeUnit::Nanosecond => Duration::from_nanoseconds(Decimal::from(q)),
                }
            }
        }
    };
}

impl Duration {
    pub fn from_days(days: Decimal) -> Self {
        Self {
            0: days * Decimal::from(SECONDS_PER_DAY),
        }
    }
    pub fn from_hours(hours: Decimal) -> Self {
        Self {
            0: hours * Decimal::from(SECONDS_PER_HOUR),
        }
    }
    pub fn from_minutes(minutes: Decimal) -> Self {
        Self {
            0: minutes * Decimal::from(SECONDS_PER_MINUTE),
        }
    }
    pub fn from_seconds(seconds: Decimal) -> Self {
        Self { 0: seconds }
    }
    pub fn from_millseconds(ms: Decimal) -> Self {
        Self {
            0: ms * Decimal::from(1e-3),
        }
    }
    pub fn from_nanoseconds(ns: Decimal) -> Self {
        Self {
            0: ns * Decimal::from(1e-9),
        }
    }

    /// Creates a new duration from the provided unit
    pub fn from_f64(value: f64, unit: TimeUnit) -> Self {
        unit * value
    }

    /// Creates a new duration from the provided fraction and unit
    pub fn from_fraction(num: i64, denom: i64, unit: TimeUnit) -> Self {
        let num_u = num.abs() as u128;
        let denom_u = denom.abs() as u128;
        if (num < 0 && denom < 0) || (num > 0 && denom > 0) {
            Self(Decimal::from_fraction(Fraction::new(num_u, denom_u)) * unit.in_seconds())
        } else {
            Self(Decimal::from_fraction(Fraction::new_neg(num_u, denom_u)) * unit.in_seconds())
        }
    }

    /// Returns this duration in f64 in the provided unit.
    /// For high fidelity comparisons, it is recommended to keep using the Duration structure.
    pub fn in_unit_f64(&self, unit: TimeUnit) -> f64 {
        self.in_unit(unit).to_f64().unwrap()
    }

    /// Returns the value of this duration in the requested unit.
    pub fn in_unit(&self, unit: TimeUnit) -> Decimal {
        self.0 * unit.from_seconds()
    }
}

impl fmt::Display for Duration {
    // Prints the duration with appropriate units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds_f64 = self.0.to_f64().unwrap();
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

impl fmt::LowerExp for Duration {
    // Prints the duration with appropriate units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let seconds_f64 = self.0.to_f64().unwrap();
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeUnit {
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Nanosecond,
}

impl TimeUnit {
    pub fn in_seconds(self) -> Decimal {
        match self {
            TimeUnit::Day => Decimal::from(SECONDS_PER_DAY),
            TimeUnit::Hour => Decimal::from(SECONDS_PER_HOUR),
            TimeUnit::Minute => Decimal::from(SECONDS_PER_MINUTE),
            TimeUnit::Second => Decimal::from(1.0),
            TimeUnit::Millisecond => Decimal::from(1e-3),
            TimeUnit::Nanosecond => Decimal::from(1e-9),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_seconds(self) -> Decimal {
        Decimal::from(1) / self.in_seconds()
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Duration {
        Self { 0: self.0 + rhs.0 }
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
        Self { 0: self.0 - rhs.0 }
    }
}

impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl_ops_for_type!(f32);
impl_ops_for_type!(f64);
impl_ops_for_type!(u8);
impl_ops_for_type!(i8);
impl_ops_for_type!(u16);
impl_ops_for_type!(i16);
impl_ops_for_type!(u32);
impl_ops_for_type!(i32);
impl_ops_for_type!(u64);
impl_ops_for_type!(i64);
impl_ops_for_type!(u128);

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
    // Check that we support nanoseconds pas GPS time
    let now = TimeUnit::Nanosecond * 1286495254000000000_u128;
    assert_eq!(format!("{}", now), "14889.991365740741 days");

    // Test operations
    let seven_hours = TimeUnit::Hour * 7;
    let six_minutes = TimeUnit::Minute * 6;
    let five_seconds = TimeUnit::Second * 5.0;
    let sum: Duration = seven_hours + six_minutes + five_seconds;
    assert!((sum.in_unit_f64(TimeUnit::Second) - 25565.0).abs() < EPSILON);

    let sub: Duration = seven_hours - six_minutes - five_seconds;
    assert!((sub.in_unit_f64(TimeUnit::Second) - 24835.0).abs() < EPSILON);

    // Test fractional
    let quarter_hour = Duration::from_fraction(1, 4, TimeUnit::Hour);
    let third_hour = Duration::from_fraction(1, 3, TimeUnit::Hour);
    let sum = quarter_hour + third_hour;
    assert!((sum.in_unit_f64(TimeUnit::Minute) - 35.0).abs() < EPSILON);
    println!(
        "Duration: {}\nFloating: {}",
        sum.in_unit_f64(TimeUnit::Minute),
        (1.0 / 4.0 + 1.0 / 3.0) * 60.0
    );
    assert_eq!(format!("{}", sum), "35 min"); // Note the automatic unit selection

    let quarter_hour = Duration::from_fraction(-1, 4, TimeUnit::Hour);
    let third_hour = Duration::from_fraction(1, -3, TimeUnit::Hour);
    let sum = quarter_hour + third_hour;
    assert!((sum.in_unit_f64(TimeUnit::Minute) + 35.0).abs() < EPSILON);
    assert_eq!(format!("{}", sum), "-35 min"); // Note the automatic unit selection
    assert_eq!(format!("{:.2}", sum), "-35.00 min"); // Note the automatic unit selection
}

// TODO:
// 0. Display should print 15 days 5 h 9 min 59 s 159789 ns , but LowerExp and the floating point should remain as now
// 1. Epoch should only be add-able with Durations
// 2. Epoch sub should also return Durations
// 3. Initialize an Epoch from a "duration past J1900" etc.
// 4. MAYBE: Support the same kind of unit stuff here for time systems. Might be possible with an enum which calls itself recursively, although this might be complicated for TDB
