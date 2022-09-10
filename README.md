# hifitime 3

Scientifically accurate date and time handling with guaranteed nanosecond precision for 32,768 years _before_ 01 January 1900 and 32,767 years _after_ that reference epoch.
Formally verified to not crash on operations on epochs and durations using the [`Kani`](https://model-checking.github.io/kani/) model checking.

[![hifitime on crates.io][cratesio-image]][cratesio]
[![Build Status](https://github.com/nyx-space/hifitime/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/nyx-space/hifitime/actions)
[![hifitime on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/hifitime.svg
[cratesio]: https://crates.io/crates/hifitime
[docsrs-image]: https://docs.rs/hifitime/badge.svg
[docsrs]: https://docs.rs/hifitime/


# Features

 * [x] Initialize a high precision Epoch from the system time in UTC
 * [x] Leap seconds (as announced by the IETF on a yearly basis)
 * [x] UTC representation with ISO8601 formatting
 * [x] Trivial support of time arithmetic: addition (e.g. `2.hours() + 3.seconds()`), subtraction (e.g. `2.hours() - 3.seconds()`), round/floor/ceil operations (e.g. `2.hours().round(3.seconds())`)
 * [x] Supports ranges of Epochs and TimeSeries (linspace of `Epoch`s and `Duration`s)
 * [x] Trivial conversion between many time systems
 * [x] High fidelity Ephemeris Time / Dynamic Barycentric Time (TDB) computations from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB)
 * [x] Julian dates and Modified Julian dates
 * [x] Embedded device friendly: `no-std` and `const fn` where possible

This library is validated against NASA/NAIF SPICE for the Ephemeris Time to Universal Coordinated Time computations: there are exactly zero nanoseconds of difference between SPICE and hifitime for the computation of ET and UTC after 01 January 1972. Refer to the [leap second](#leap-second-support) section for details. Other examples are validated with external references, as detailed on a test-by-test basis.

## Supported time systems

+ Temps Atomique International (TAI)
+ Universal Coordinated Time (UTC)
+ Terrestrial Time (TT)
+ Ephemeris Time (ET) without the small perturbations as per NASA/NAIF SPICE leap seconds kernel
+ Dynamic Barycentric Time (TDB), a higher fidelity ephemeris time
+ Global Positioning System (GPS), and UNIX
## Non-features
* Time-agnostic / date-only epochs. Hifitime only supports the combination of date and time, but the `Epoch::{at_midnight, at_noon}` is provided as a helper function.

# Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
hifitime = "3.4"
```

And add the following to your crate root:

```rust
extern crate hifitime;
```

## Examples:
### Time creation
```rust
use hifitime::{Epoch, Unit, TimeUnits};

#[cfg(feature = "std")]
{
// Initialization from system time is only availble when std feature is enabled
let now = Epoch::now().unwrap();
println!("{}", now);
}

let mut santa = Epoch::from_gregorian_utc_hms(2017, 12, 25, 01, 02, 14);
assert_eq!(santa.as_mjd_utc_days(), 58112.043217592590);
assert_eq!(santa.as_jde_utc_days(), 2458112.5432175924);

assert_eq!(
    santa + 3600 * Unit::Second,
    Epoch::from_gregorian_utc_hms(2017, 12, 25, 02, 02, 14),
    "Could not add one hour to Christmas"
);

assert_eq!(
    santa + 60.0.minutes(),
    Epoch::from_gregorian_utc_hms(2017, 12, 25, 02, 02, 14),
    "Could not add one hour to Christmas"
);

assert_eq!(
    santa + 1.hours(),
    Epoch::from_gregorian_utc_hms(2017, 12, 25, 02, 02, 14),
    "Could not add one hour to Christmas"
);

#[cfg(feature = "std")]
{
use std::str::FromStr;
let dt = Epoch::from_gregorian_utc_hms(2017, 1, 14, 0, 31, 55);
assert_eq!(dt, Epoch::from_str("2017-01-14T00:31:55 UTC").unwrap());
// And you can print it too, although by default it will print in UTC
assert_eq!(dt.as_gregorian_utc_str(), "2017-01-14T00:31:55 UTC".to_string());
assert_eq!(format!("{}", dt), "2017-01-14T00:31:55 UTC".to_string());
}
```
### Time differences, time unit, and duration handling

Comparing times will lead to a Duration type. Printing that will automatically select the unit.

```rust
use hifitime::{Epoch, Unit, Duration, TimeUnits};

let at_midnight = Epoch::from_gregorian_utc_at_midnight(2020, 11, 2);
let at_noon = Epoch::from_gregorian_utc_at_noon(2020, 11, 2);
assert_eq!(at_noon - at_midnight, 12 * Unit::Hour);
assert_eq!(at_noon - at_midnight, 1 * Unit::Day / 2);
assert_eq!(at_midnight - at_noon, -1.days() / 2);

let delta_time = at_noon - at_midnight;
assert_eq!(format!("{}", delta_time), "12 h".to_string());
// And we can multiply durations by a scalar...
let delta2 = 2 * delta_time;
assert_eq!(format!("{}", delta2), "1 days".to_string());
// Or divide them by a scalar.
assert_eq!(format!("{}", delta2 / 2.0), "12 h".to_string());

// And of course, these comparisons account for differences in time systems
let at_midnight_utc = Epoch::from_gregorian_utc_at_midnight(2020, 11, 2);
let at_noon_tai = Epoch::from_gregorian_tai_at_noon(2020, 11, 2);
assert_eq!(format!("{}", at_noon_tai - at_midnight_utc), "11 h 59 min 23 s".to_string());
```

Timeunits and frequency units are trivially supported. Hifitime only supports up to nanosecond precision (but guarantees it for 64 millenia), so any duration less than one nanosecond is truncated.

```rust
use hifitime::{Epoch, Unit, Freq, Duration, TimeUnits};

// One can compare durations
assert!(10.seconds() > 5.seconds());
assert!(10.days() + 1.nanoseconds() > 10.days());

// Those durations are more precise than floating point since this is integer math in nanoseconds
let d: Duration = 1.0.hours() / 3 - 20.minutes();
assert!(d.abs() < Unit::Nanosecond);
assert_eq!(3 * 20.minutes(), Unit::Hour);

// And also frequencies but note that frequencies are converted to Durations!
// So the duration of that frequency is compared, hence the following:
assert!(10 * Freq::Hertz < 5 * Freq::Hertz);
assert!(4 * Freq::MegaHertz > 5 * Freq::MegaHertz);

// And asserts on the units themselves
assert!(Freq::GigaHertz < Freq::MegaHertz);
assert!(Unit::Second > Unit::Millisecond);
```

### Iterating over times ("linspace" of epochs)
Finally, something which may come in very handy, line spaces between times with a given step.

```rust
use hifitime::{Epoch, Unit, TimeSeries};
let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
let step = 2 * Unit::Hour;
let time_series = TimeSeries::inclusive(start, end, step);
let mut cnt = 0;
for epoch in time_series {
    println!("{}", epoch);
    cnt += 1
}
// Check that there are indeed six two-hour periods in a half a day,
// including start and end times.
assert_eq!(cnt, 7)
```

# Design
No software is perfect, so please report any issue or bugs on [Github](https://github.com/nyx-space/hifitime/issues/new).

## Duration
Under the hood, a Duration is represented as a 16 bit signed integer of centuries (`i16`) and a 64 bit unsigned integer (`u64`) of the nanoseconds past that century. The overflowing and underflowing of nanoseconds is handled by changing the number of centuries such that the nanoseconds number never represents more than one century (just over four centuries can be stored in 64 bits).

Advantages:
1. Exact precision of a duration: using a floating point value would cause large durations (e.g. Julian Dates) to have less precision than smaller durations. Durations in hifitime have exactly one nanosecond of precision for 65,536 years.
2. Skipping floating point operations allows this library to be used in embedded devices without a floating point unit.
3. Duration arithmetics are exact, e.g. one third of an hour is exactly twenty minutes and not "0.33333 hours."

Disadvantages:
1. Most astrodynamics applications require the computation of a duration in floating point values such as when querying an ephemeris. This design leads to an overhead of about 500 nanoseconds according to the benchmarks. You may run the benchmarks with `cargo bench`.

### Printing

When Durations are printed, only the units whose value is non-zero is printed. For example, `5.hours() + 256.0.milliseconds() + 1.0.nanoseconds()` will be printed as "5 h 256 ms 1 ns".

```rust
use hifitime::{Duration, TimeUnits};

assert_eq!(
    format!(
        "{}",
        5.hours() + 256.0.milliseconds() + 1.0.nanoseconds()
    ),
    "5 h 256 ms 1 ns"
);

assert_eq!(
    format!(
        "{}",
        5.days() + 1.0.nanoseconds()
    ),
    "5 days 1 ns"
);
```

## Epoch
The Epoch is simply a wrapper around a Duration. All epochs are stored in TAI duration with respect to 01 January 1900 at noon (the official TAI epoch). The choice of TAI meets the [Standard of Fundamental Astronomy (SOFA)](https://www.iausofa.org/) recommendation of opting for a glitch-free time scale (i.e. without discontinuities like leap seconds or non-uniform seconds like TDB).

### Printing and formatting

Epochs can be formatted in the following time systems:

+ UTC: `{epoch}`
+ TAI: `{epoch:x}`
+ TT: `{epoch:X}`
+ TDB: `{epoch:e}`
+ ET: `{epoch:E}`
+ UNIX: `{epoch:p}`
+ GPS: `{epoch:o}`

```rust
use hifitime::Epoch;

let epoch = Epoch::from_gregorian_utc_hms(2022, 9, 6, 23, 24, 29);

assert_eq!(format!("{epoch}"), "2022-09-06T23:24:29 UTC");
assert_eq!(format!("{epoch:x}"), "2022-09-06T23:25:06 TAI");
assert_eq!(format!("{epoch:X}"), "2022-09-06T23:25:38.184000015 TT");
assert_eq!(format!("{epoch:E}"), "2022-09-06T23:25:38.182538986 ET");
assert_eq!(format!("{epoch:e}"), "2022-09-06T23:25:38.182541370 TDB");
assert_eq!(format!("{epoch:p}"), "1662506669"); // UNIX seconds
assert_eq!(format!("{epoch:o}"), "1346541887000000000"); // GPS nanoseconds
```

## Leap second support

Leap seconds allow TAI (the absolute time reference) and UTC (the civil time reference) to not drift too much. In short, UTC allows humans to see the sun at zenith at noon, whereas TAI does not worry about that. Leap seconds are introduced to allow for UTC to catch up with the absolute time reference of TAI. Specifically, UTC clocks are "stopped" for one second to make up for the accumulated difference between TAI and UTC. These leap seconds are announced several months in advance by IERS, cf. in the [IETF leap second reference](https://www.ietf.org/timezones/data/leap-seconds.list).

The "placement" of these leap seconds in the formatting of a UTC date is left up to the software: there is no common way to handle this. Some software prevents a second tick, i.e. at 23:59:59 the UTC clock will tick for _two seconds_ (instead of one) before hoping to 00:00:00. Some software, like hifitime, allow UTC dates to be formatted as 23:59:60 on strictly the days when a leap second is inserted. For example, the date `2016-12-31 23:59:60 UTC` is a valid date in hifitime because a leap second was inserted on 01 Jan 2017.

### Important
Prior to the first leap second, NAIF SPICE claims that there were nine seconds of difference between TAI and UTC: this is different from the [Standard of Fundamental Astronomy (SOFA)](https://www.iausofa.org/). SOFA's `iauDat` function will return non-integer leap seconds from 1960 to 1972. It will return an error for dates prior to 1960. **Hifitime only accounts for leap seconds announced by [IERS](https://www.ietf.org/timezones/data/leap-seconds.list)** in its computations: there is a ten (10) second jump between TAI and UTC on 01 January 1972. This allows the computation of UNIX time to be a specific offset of TAI in hifitime. However, the prehistoric (pre-1972) leap seconds as returned by SOFA are available in the `leap_seconds()` method of an epoch if the `iers_only` parameter is set to false.

## Ephemeris Time vs Dynamic Barycentric Time (TDB)
In theory, as of January 2000, ET and TDB should now be identical. _However_, the NASA NAIF leap seconds files (e.g. [naif00012.tls](./naif00012.tls)) use a simplified algorithm to compute the TDB:
> Equation \[4\], which ignores small-period fluctuations, is accurate to about 0.000030 seconds.

In order to provide full interoperability with NAIF, hifitime uses the NAIF algorithm for "ephemeris time" and the [ESA algorithm](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB) for "dynamical barycentric time." Hence, if exact NAIF behavior is needed, use all of the functions marked as `et` instead of the `tdb` functions, such as `epoch.as_et_seconds()` instead of `epoch.as_tdb_seconds()`.


# Changelog

## 3.4.0
+ Ephemeris Time and Dynamical Barycentric Time fixed to use the J2000 reference epoch instead of the J1900 reference epoch. This is a **potentially breaking change** if you relied on the previous one century error when converting from/to ET/TDB into/from UTC _and storing the data as a string_. There is **no difference** if the original representation was used.
+ Ephemeris Time now **strictly** matches NAIF SPICE: **the error between SPICE and hifitime is now zero nanoseconds.** after the introduction of the first leap second. Prior to the first leap second, NAIF SPICE claims that there were nine seconds of difference between TAI and UTC: this is different from SOFA. Hifitime instead does not account for leap seconds in prehistoric (pre-1972) computations at all.
+ The [_Standard of Fundamentals of Astronomy_ (SOFA)](https://www.iausofa.org/2021_0512_C.html) leap seconds from 1960 to 1972 are now available with the `leap_seconds() -> Option<f64>` function on an instance of Epoch. **Importantly**, no difference in the behavior of hifitime should be noticed here: the prehistoric leap seconds are ignored in all calculations in hifitime and only provided to meet the SOFA calculations.
+ `Epoch` and `Duration` now have the C memory representation to allow for hifitime to be embedded in C more easily.
+ `Epoch` and `Duration` can now be encoded or decoded as ASN1 DER with the `asn1der` crate feature (disabled by default).

## 3.3.0
+ Formal verification of the normalization operation on `Duration`, which in turn guarantees that `Epoch` operations cannot panic, cf. [#127](https://github.com/nyx-space/hifitime/issues/127)
+ Fix `len` and `size_hint` for `TimeSeries`, cf. [#131](https://github.com/nyx-space/hifitime/issues/131), reported by [@d3v-null](https://github.com/d3v-null), thanks for the find!
+ `Epoch` now implements `Eq` and `Ord`, cf. [#133](https://github.com/nyx-space/hifitime/pull/133), thanks [@mkolopanis](https://github.com/mkolopanis) for the PR!
+ `Epoch` can now be printed in different time systems with format modifiers, cf. [#130](https://github.com/nyx-space/hifitime/issues/130)
+ (minor) `as_utc_duration` in `Epoch` is now public, cf. [#129](https://github.com/nyx-space/hifitime/issues/129)
+ (minor) The whole crate now uses `num-traits` thereby skipping the explicit use of `libm`. Basically, operations on `f64` look like normal Rust again, cf. [#128](https://github.com/nyx-space/hifitime/issues/128)
+ (minor) Move the tests to their own folder to make it obvious that this is thoroughly tested

## 3.2.0
+ Fix no-std implementation by using `libm` for non-core f64 operations
+ Add UNIX timestamp, thanks [@mkolopanis](https://github.com/mkolopanis)
+ Enums now derive `Eq` and some derive `Ord` (where relevant) [#118](https://github.com/nyx-space/hifitime/issues/118)
+ Use const fn where possible and switch to references where possible [#119](https://github.com/nyx-space/hifitime/issues/119)
+ Allow extracting the centuries and nanoseconds of a `Duration` and `Epoch`, respectively with to_parts and to_tai_parts [#122](https://github.com/nyx-space/hifitime/issues/122)
+ Add `ceil`, `floor`, `round` operations to `Epoch` and `Duration`
## 3.1.0
+ Add `#![no_std]` support
+ Add `to_parts` to `Duration` to extract the centuries and nanoseconds of a duration
+ Allow building an `Epoch` from its duration and parts in TAI system
+ Add pure nanosecond (`u64`) constructor and getter for GPST since GPS based clocks will count in nanoseconds
### Possibly breaking change
+ `Errors::ParseError` no longer contains a `String` but an enum `ParsingErrors` instead. This is considered possibly breaking because it would only break code in the cases where a datetime parsing or unit parsing was caught and handled (uncommon). Moreover, the output is still `Display`-able.
## 3.0.0
+ Backend rewritten from TwoFloat to a struct of the centuries in `i16` and nanoseconds in `u64`. Thanks to [@pwnorbitals](https://github.com/pwnorbitals) for proposing the idea in #[107](https://github.com/nyx-space/hifitime/issues/107) and writing the proof of concept. This leads to at least a 2x speed up in most calculations, cf. [this comment](https://github.com/nyx-space/hifitime/pull/107#issuecomment-1040702004).
+ Fix GPS epoch, and addition of a helper functions in `Epoch` by [@cjordan](https://github.com/cjordan)

## 2.2.3
+ More deterministic `as_jde_tdb_days()` in `Epoch`. In version 2.2.1, the ephemeris time and TDB _days_ were identical down to machine precision. After a number of validation cases in the rotation equations of the IAU Earth to Earth Mean Equator J2000 frame, the new formulation was shown to lead to less rounding errors when requesting the days. These rounding errors prevented otherwise trivial test cases. However, it adds an error of **40.2 nanoseconds** when initializing an Epoch with the days in ET and requesting the TDB days.

_Note:_ this was originally published as 2.2.2 but I'd forgotten to update one of the tests with the 40.2 ns error.