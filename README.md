# hifitime 3

Scientifically accurate date and time handling with guaranteed nanosecond precision for 32,768 years _before_ 01 January 1900 and 32,767 years _after_ that reference epoch.

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
 * [x] Trivial conversion between the time systems TAI, TT, ET, TDB, GPS, and UNIX.
 * [x] High fidelity Ephemeris Time / Dynamic Barycentric Time (TDB) computations from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB)
 * [x] Julian dates and Modified Julian dates
 * [x] Embedded device friendly: `no-std` and `const fn` where possible
 * [ ] Support for custom representations of time (e.g. NASA GMAT Modified Julian Date)
 * [ ] Trivial support of other time representations, such as TDT (cf #44)

Almost all examples are validated with external references, as detailed on a test-by-test basis.

## Non-features
* Time-agnostic / date-only epochs. Hifitime only supports the combination of date and time, but the `Epoch::{at_midnight, at_noon}` is provided as a helper function.

# Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
hifitime = "3.2"
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
// assert_eq!(format!("{}", delta_time), "12 h 0 min 0 s".to_string());
// And we can multiply durations by a scalar...
let delta2 = 2 * delta_time;
// assert_eq!(format!("{}", delta2), "1 days 0 h 0 min 0 s".to_string());
// Or divide them by a scalar.
// assert_eq!(format!("{}", delta2 / 2.0), "12 h 0 min 0 s".to_string());

// And of course, these comparisons account for differences in time systems
let at_midnight_utc = Epoch::from_gregorian_utc_at_midnight(2020, 11, 2);
let at_noon_tai = Epoch::from_gregorian_tai_at_noon(2020, 11, 2);
// assert_eq!(format!("{}", at_noon_tai - at_midnight_utc), "11 h 59 min 23 s".to_string());
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

# Validation examples
Validation is done using NASA's SPICE toolkit, and specifically the [spiceypy](https://spiceypy.readthedocs.io/) Python wrapper.

The most challenging validation is the definition of Ephemeris Time, which is very nearly the same as the Dynamic Barycentric Time (TDB).
These calculations in hifitime are from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB).

Hifitime uses a fixed offset for the computation of Ephemeris Time, as is recommended in Navipedia. For TDB however, the offset is based on the centuries since J2000 TT and therefore time varying.
I believe that SPICE uses TDB for all dates after J2000 TT. Hence, in the following validation, we will be comparing the SPICE ET with the Hifitime TDB.

The following examples are executed as part of the standard test suite (cf. the function called `spice_et_tdb`).

_Note:_ the differences shown here are likely due to a combination of SPICE using a different formulation for the calculation (using the constants in the SPICE kernels) and computing everything on a 64-bit floating point value. [By design](https://en.wikipedia.org/wiki/IEEE_754), a 64-bit floating point value has approximation errors. Hifitime performs all calculations on integers, which do not suffer from rounding errors.

## Case 1
In SPICE, we chose to convert the UTC date `2012-02-07 11:22:33 UTC` into Ephemeris Time. SPICE responds with `381885819.18493587`.
Initializing the same UTC date in hifitime and requesting the TDB leads to `381885819.18493646`, which is an error of **596.05 nanoseconds**.

## Case 2
In SPICE, we chose to convert the UTC date `2002-02-07 00:00:00.000 UTC` into Ephemeris Time. SPICE responds with `66312064.18493876`.
Initializing the same UTC date in hifitime and requesting the TDB leads to a difference **633.29 nanoseconds**.

## Case 3
This tests that we can correctly compute TDB time which will have a negative number of days because the UTC input is prior to J2000 TT.
In SPICE, we chose to convert the UTC date `1996-02-07 11:22:33 UTC` into Ephemeris Time. SPICE responds with `-123035784.81506048`.
Initializing the same UTC date in hifitime and requesting the TDB leads to a difference **640.74 nanoseconds**.

## Case 4
In SPICE, we chose to convert the UTC date `2015-02-07 00:00:00.000 UTC` into Ephemeris Time. SPICE responds with `476580220.1849411`.
Initializing the same UTC date in hifitime and requesting the TDB leads to a difference **655.65 nanoseconds**.

## Case 5
In SPICE, we chose to convert the TDB Julian Date in days `2452312.500372511` into Ephemeris Time, and initialize a Hifitime Epoch with that result (`66312032.18493909`).
We then convert that epoch back into **days** of Julian Date TDB and Julian Date ET, which lead a difference **below machine precision** for the TDB computation and **0.46 nanoseconds** for the ET computation on a 64-bit floating point (f64/double).

# Notes

Please report and bugs by [clicking here](https://github.com/nyx-space/hifitime/issues/new).

### Leap second support
Each time computing library may decide when the extra leap second exists as explained
in the [IETF leap second reference](https://www.ietf.org/timezones/data/leap-seconds.list).
To ease computation, `hifitime` decides that second is the 60th of a UTC date, if such exists.
Note that this second exists at a different time than defined on NASA HEASARC. That tool is
used for validation of Julian dates. As an example of how this is handled, check the Julian
day computations for [2015-06-30 23:59:59](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes),
[2015-06-30 23:59:60](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes) and [2015-07-01 00:00:00](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes).

### Ephemeris Time vs Dynamic Barycentric Time (TDB)
ET and TDB should now be identical. However, hifitime uses the European Space Agency's definition of TDB, detailed [here](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB). It seems that SPICE uses the older definition which has a fixed offset from TDT of 0.000935 seconds. This difference is more prominent around the TDB epoch of 01 January 2000.

# Changelog

## 3.3.0
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