# Introduction to Hifitime

Hifitime is a powerful Rust and Python library designed for time management. It provides extensive functionalities with precise operations for time calculation in different time scales, making it suitable for engineering and scientific applications where general relativity and time dilation matter. Hifitime guarantees nanosecond precision for 65,536 years around 01 January 1900 TAI. Hifitime is also formally verified using the [`Kani` model checker](https://model-checking.github.io/kani/), read more about it [this verification here](https://model-checking.github.io/kani-verifier-blog/2023/03/31/how-kani-helped-find-bugs-in-hifitime.html).

Most users of Hifitime will only need to rely on the `Epoch` and `Duration` structures, and optionally the `Weekday` enum for week based computations. Scientific applications may make use of the `TimeScale` enum as well.

## Usage

First, install `hifitime` either with `cargo add hifitime` in your Rust project or `pip install hifitime` in Python.

If building from source, note that the Python package is only built if the `python` feature is enabled.

### Epoch ("datetime" equivalent)

**Create an epoch in different time scales.**

```rust
use hifitime::prelude::*;
use core::str::FromStr;
// Create an epoch in UTC
let epoch = Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 37);
// Or from a string
let epoch_from_str = Epoch::from_str("2000-02-29T14:57:29.000000037 UTC").unwrap();
assert_eq!(epoch, epoch_from_str);
// Creating it from TAI will effectively show the number of leap seconds in between UTC an TAI at that epoch
let epoch_tai = Epoch::from_gregorian_tai(2000, 2, 29, 14, 57, 29, 37);
// The difference between two epochs is a Duration
let num_leap_s = epoch - epoch_tai;
assert_eq!(format!("{num_leap_s}"), "32 s");

// Trivially convert to another time scale
// Either by grabbing a subdivision of time in that time scale
assert_eq!(epoch.to_gpst_days(), 7359.623402777777); // Compare to the GPS time scale

// Or by fetching the exact duration
let mjd_offset = Duration::from_str("51603 days 14 h 58 min 33 s 184 ms 37 ns").unwrap();
assert_eq!(epoch.to_mjd_tt_duration(), mjd_offset); // Compare to the modified Julian days in the Terrestrial Time time scale.
```

In Python:
```python
>>> from hifitime import *
>>> epoch = Epoch("2000-02-29T14:57:29.000000037 UTC")
>>> epoch
2000-02-29T14:57:29.000000037 UTC
>>> epoch_tai = Epoch.init_from_gregorian_tai(2000, 2, 29, 14, 57, 29, 37)
>>> epoch_tai
2000-02-29T14:57:29.000000037 TAI
>>> epoch.timedelta(epoch_tai)
32 s
>>> epoch.to_gpst_days()
7359.623402777777
>>> epoch.to_mjd_tt_duration()
51603 days 14 h 58 min 33 s 184 ms 37 ns
>>> 
```

**Hifitime provides several date time formats like RFC2822, ISO8601, or RFC3339.**

```rust
use hifitime::efmt::consts::{ISO8601, RFC2822, RFC3339};
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc(2000, 2, 29, 14, 57, 29, 37);
// The default Display shows the UTC time scale
assert_eq!(format!("{epoch}"), "2000-02-29T14:57:29.000000037 UTC");
// Format it in RFC 2822
let fmt = Formatter::new(epoch, RFC2822);
assert_eq!(format!("{fmt}"), format!("Tue, 29 Feb 2000 14:57:29"));

// Or in ISO8601
let fmt = Formatter::new(epoch, ISO8601);
assert_eq!(
    format!("{fmt}"),
    format!("2000-02-29T14:57:29.000000037 UTC")
);

// Which is somewhat similar to RFC3339
let fmt = Formatter::new(epoch, RFC3339);
assert_eq!(
    format!("{fmt}"),
    format!("2000-02-29T14:57:29.000000037+00:00")
);
```

**Need some custom format? Hifitime also supports the C89 token, cf. [the documentation](https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html).**

```rust
use core::str::FromStr;
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);

// Parsing with a custom format
assert_eq!(
    Epoch::from_format_str("Sat, 07 Feb 2015 11:22:33", "%a, %d %b %Y %H:%M:%S").unwrap(),
    epoch
);

// And printing with a custom format
let fmt = Format::from_str("%a, %d %b %Y %H:%M:%S").unwrap();
assert_eq!(
    format!("{}", Formatter::new(epoch, fmt)),
    "Sat, 07 Feb 2015 11:22:33"
);
```

**You can also grab the current system time in UTC, if the `std` feature is enabled (default), and find the next or previous day of the week.**
```rust
use hifitime::prelude::*;

#[cfg(feature = "std")]
{
    let now = Epoch::now().unwrap();
    println!("{}", now.next(Weekday::Tuesday));
    println!("{}", now.previous(Weekday::Sunday));
}
```

**Oftentimes, we'll want to query something at a fixed step between two epochs. Hifitime makes this trivial with `TimeSeries`.**

```rust
use hifitime::prelude::*;

let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
let end = start + 12.hours();
let step = 2.hours();

let time_series = TimeSeries::inclusive(start, end, step);
let mut cnt = 0;
for epoch in time_series {
    #[cfg(feature = "std")]
    println!("{}", epoch);
    cnt += 1
}
// Check that there are indeed seven two-hour periods in a half a day,
// including start and end times.
assert_eq!(cnt, 7)
```

In Python:
```python
>>> from hifitime import *
>>> start = Epoch.init_from_gregorian_utc_at_midnight(2017, 1, 14)
>>> end = start + Unit.Hour*12
>>> iterator = TimeSeries(start, end, step=Unit.Hour*2, inclusive=True)
>>> for epoch in iterator:
...     print(epoch)
... 
2017-01-14T00:00:00 UTC
2017-01-14T02:00:00 UTC
2017-01-14T04:00:00 UTC
2017-01-14T06:00:00 UTC
2017-01-14T08:00:00 UTC
2017-01-14T10:00:00 UTC
2017-01-14T12:00:00 UTC
>>> 

```

### Duration

```rust
use hifitime::prelude::*;
use core::str::FromStr;

// Create a duration using the `TimeUnits` helping trait.
let d = 5.minutes() + 7.minutes() + 35.nanoseconds();
assert_eq!(format!("{d}"), "12 min 35 ns");

// Or using the built-in enums
let d_enum = 12 * Unit::Minute + 35.0 * Unit::Nanosecond;

// But it can also be created from a string
let d_from_str = Duration::from_str("12 min 35 ns").unwrap();
assert_eq!(d, d_from_str);
```

**Hifitime guarantees nanosecond precision, but most human applications don't care too much about that. Durations can be rounded to provide a useful approximation for humans.**

```rust
use hifitime::prelude::*;

// Create a duration using the `TimeUnits` helping trait.
let d = 5.minutes() + 7.minutes() + 35.nanoseconds();
// Round to the nearest minute
let rounded = d.round(1.minutes());
assert_eq!(format!("{rounded}"), "12 min");

// And this works on Epochs as well.
let previous_post = Epoch::from_gregorian_utc_hms(2015, 2, 7, 11, 22, 33);
let example_now = Epoch::from_gregorian_utc_hms(2015, 8, 17, 22, 55, 01);

// We'll round to the nearest fifteen days
let this_much_ago = example_now - previous_post;
assert_eq!(format!("{this_much_ago}"), "191 days 11 h 32 min 28 s");
let about_this_much_ago_floor = this_much_ago.floor(15.days());
assert_eq!(format!("{about_this_much_ago_floor}"), "180 days");
let about_this_much_ago_ceil = this_much_ago.ceil(15.days());
assert_eq!(format!("{about_this_much_ago_ceil}"), "195 days");
```

In Python:

```python
>>> from hifitime import *
>>> d = Duration("12 min 32 ns")
>>> d.round(Unit.Minute*1)
12 min
>>> d
12 min 32 ns
>>> 
```

[![hifitime on crates.io][cratesio-image]][cratesio]
[![hifitime on docs.rs][docsrs-image]][docsrs]
[![minimum rustc: 1.70](https://img.shields.io/badge/minimum%20rustc-1.70-yellowgreen?logo=rust)](https://www.whatrustisit.com)
[![Build Status](https://github.com/nyx-space/hifitime/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/nyx-space/hifitime/actions)
[![Build Status](https://github.com/nyx-space/hifitime/actions/workflows/formal_verification.yml/badge.svg?branch=master)](https://github.com/nyx-space/hifitime/actions)
[![codecov](https://codecov.io/gh/nyx-space/hifitime/branch/master/graph/badge.svg?token=l7zU57rUGs)](https://codecov.io/gh/nyx-space/hifitime)

[cratesio-image]: https://img.shields.io/crates/v/hifitime.svg
[cratesio]: https://crates.io/crates/hifitime
[docsrs-image]: https://docs.rs/hifitime/badge.svg
[docsrs]: https://docs.rs/hifitime/

# Comparison with `time` and `chrono`

First off, both `time` and `chrono` are fantastic libraries in their own right. There's a reason why they have millions and millions of downloads. Secondly, hifitime was started in October 2017, so quite a while before the revival of `time` (~ 2019).

One of the key differences is that both `chrono` and `time` separate the concepts of "time" and "date." Hifitime asserts that this is physically invalid: both a time and a date are an offset from a reference in a given time scale. That's why, Hifitime does not separate the components that make up a date, but instead, only stores a fixed duration with respect to TAI. Moreover, Hifitime is formally verified with a model checker, which is much more thorough than property testing.

More importantly, neither `time` nor `chrono` are suitable for astronomy, astrodynamics, or any physics that must account for time dilation due to relativistic speeds or lack of the Earth as a gravity source (which sets the "tick" of a second).

Hifitime also natively supports the UT1 time scale (the only "true" time) if built with the `ut1` feature.

# Features

 * [x] Initialize a high precision Epoch from the system time in UTC
 * [x] Leap seconds (as announced by the IETF on a yearly basis)
 * [x] UTC representation with ISO8601 and RFC3339 formatting and blazing fast parsing (45 nanoseconds)
 * [x] Trivial support of time arithmetic: addition (e.g. `2.hours() + 3.seconds()`), subtraction (e.g. `2.hours() - 3.seconds()`), round/floor/ceil operations (e.g. `2.hours().round(3.seconds())`)
 * [x] Supports ranges of Epochs and TimeSeries (linspace of `Epoch`s and `Duration`s)
 * [x] Trivial conversion between many time scales
 * [x] High fidelity Ephemeris Time / Dynamic Barycentric Time (TDB) computations from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB)
 * [x] Julian dates and Modified Julian dates
 * [x] Embedded device friendly: `no-std` and `const fn` where possible

This library is validated against NASA/NAIF SPICE for the Ephemeris Time to Universal Coordinated Time computations: there are exactly zero nanoseconds of difference between SPICE and hifitime for the computation of ET and UTC after 01 January 1972. Refer to the [leap second](#leap-second-support) section for details. Other examples are validated with external references, as detailed on a test-by-test basis.

## Supported time scales

+ Temps Atomique International (TAI)
+ Universal Coordinated Time (UTC)
+ Terrestrial Time (TT)
+ Ephemeris Time (ET) without the small perturbations as per NASA/NAIF SPICE leap seconds kernel
+ Dynamic Barycentric Time (TDB), a higher fidelity ephemeris time
+ Global Positioning System (GPST)
+ Galileo System Time (GST)
+ BeiDou Time (BDT)
+ UNIX
## Non-features
* Time-agnostic / date-only epochs. Hifitime only supports the combination of date and time, but the `Epoch::{at_midnight, at_noon}` is provided as helper functions.

# Design
No software is perfect, so please report any issue or bug on [Github](https://github.com/nyx-space/hifitime/issues/new).

## Duration
Under the hood, a Duration is represented as a 16 bit signed integer of centuries (`i16`) and a 64 bit unsigned integer (`u64`) of the nanoseconds past that century. The overflowing and underflowing of nanoseconds is handled by changing the number of centuries such that the nanoseconds number never represents more than one century (just over four centuries can be stored in 64 bits).

Advantages:
1. Exact precision of a duration: using a floating point value would cause large durations (e.g. Julian Dates) to have less precision than smaller durations. Durations in hifitime have exactly one nanosecond of precision for 65,536 years.
2. Skipping floating point operations allows this library to be used in embedded devices without a floating point unit.
3. Duration arithmetics are exact, e.g. one third of an hour is exactly twenty minutes and not "0.33333 hours."

Disadvantages:
1. Most astrodynamics applications require the computation of a duration in floating point values such as when querying an ephemeris. This design leads to an overhead of about 5.2 nanoseconds according to the benchmarks (`Duration to f64 seconds` benchmark). You may run the benchmarks with `cargo bench`.

## Epoch
The Epoch is simply a wrapper around a Duration. All epochs are stored in TAI duration with respect to 01 January 1900 at noon (the official TAI epoch). The choice of TAI meets the [Standard of Fundamental Astronomy (SOFA)](https://www.iausofa.org/) recommendation of opting for a glitch-free time scale (i.e. without discontinuities like leap seconds or non-uniform seconds like TDB).

### Printing and parsing

Epochs can be formatted and parsed in the following time scales:

+ UTC: `{epoch}`
+ TAI: `{epoch:x}`
+ TT: `{epoch:X}`
+ TDB: `{epoch:e}`
+ ET: `{epoch:E}`
+ UNIX: `{epoch:p}`
+ GPS: `{epoch:o}`

## Leap second support

Leap seconds allow TAI (the absolute time reference) and UTC (the civil time reference) to not drift too much. In short, UTC allows humans to see the sun at zenith at noon, whereas TAI does not worry about that. Leap seconds are introduced to allow for UTC to catch up with the absolute time reference of TAI. Specifically, UTC clocks are "stopped" for one second to make up for the accumulated difference between TAI and UTC. These leap seconds are announced several months in advance by IERS, cf. in the [IETF leap second reference](https://www.ietf.org/timezones/data/leap-seconds.list).

The "placement" of these leap seconds in the formatting of a UTC date is left up to the software: there is no common way to handle this. Some software prevents a second tick, i.e. at 23:59:59 the UTC clock will tick for _two seconds_ (instead of one) before hoping to 00:00:00. Some software, like hifitime, allow UTC dates to be formatted as 23:59:60 on strictly the days when a leap second is inserted. For example, the date `2016-12-31 23:59:60 UTC` is a valid date in hifitime because a leap second was inserted on 01 Jan 2017.

### Important
Prior to the first leap second, NAIF SPICE claims that there were nine seconds of difference between TAI and UTC: this is different from the [Standard of Fundamental Astronomy (SOFA)](https://www.iausofa.org/). SOFA's `iauDat` function will return non-integer leap seconds from 1960 to 1972. It will return an error for dates prior to 1960. **Hifitime only accounts for leap seconds announced by [IERS](https://www.ietf.org/timezones/data/leap-seconds.list)** in its computations: there is a ten (10) second jump between TAI and UTC on 01 January 1972. This allows the computation of UNIX time to be a specific offset of TAI in hifitime. However, the prehistoric (pre-1972) leap seconds as returned by SOFA are available in the `leap_seconds()` method of an epoch if the `iers_only` parameter is set to false.

## Ephemeris Time vs Dynamic Barycentric Time (TDB)
In theory, as of January 2000, ET and TDB should now be identical. _However_, the NASA NAIF leap seconds files (e.g. [naif00012.tls](./naif00012.tls)) use a simplified algorithm to compute the TDB:
> Equation \[4\], which ignores small-period fluctuations, is accurate to about 0.000030 seconds.

In order to provide full interoperability with NAIF, hifitime uses the NAIF algorithm for "ephemeris time" and the [ESA algorithm](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB) for "dynamical barycentric time." Hence, if exact NAIF behavior is needed, use all of the functions marked as `et` instead of the `tdb` functions, such as `epoch.to_et_seconds()` instead of `epoch.to_tdb_seconds()`.


# Changelog

## 4.0.0 (WIP)

+ Minimum Support Rust Version (MSRV) bumped to 1.77.0
+ Major refactoring of the code for ease of maintenance and removal of deprecrated functions from 3.x
+ Centralization of all time scale conversions into the `to_time_scale` function -- huge effort by [@gwbres](https://github.com/gwbres)
+ Removed `der` encoding/decoding for Epoch and Duration.

## 3.9.0

+ Update to der version 0.7.x.
+ Introduce %y formatter by @gwbres in https://github.com/nyx-space/hifitime/pull/268
+ **Possible breaking change**: Fix day of year computation by @ChristopherRabotin in https://github.com/nyx-space/hifitime/pull/273

## 3.8.5

Changes from 3.8.2 are only dependency upgrades until this release.

Minimum Supported Rust version bumped from 1.64 to **1.70**.

## 3.8.2
+ Clarify README and add a section comparing Hifitime to `time` and `chrono`, cf. [#221](https://github.com/nyx-space/hifitime/issues/221)
+ Fix incorrect printing of Gregorian dates prior to to 1900, cf. [#204](https://github.com/nyx-space/hifitime/issues/204)

## 3.8.1 (unreleased)
+ Fix documentation for the formatter, cf. [#202](https://github.com/nyx-space/hifitime/pull/202)
+ Update MSRV to 1.59 for rayon v 1.10

## 3.8.0
Thanks again to [@gwbres](https://github.com/gwbres) for his work in this release!

+ Fix CI of the formal verification and upload artifacts, cf. [#179](https://github.com/nyx-space/hifitime/pull/179)
+ Introduce time of week construction and conversion by [@gwbres](https://github.com/gwbres), cf.[#180](https://github.com/nyx-space/hifitime/pull/180) and [#188](https://github.com/nyx-space/hifitime/pull/188)
+ Fix minor typo in `src/timeunits.rs` by [@gwbres](https://github.com/gwbres), cf. [#189](https://github.com/nyx-space/hifitime/pull/189)
+ Significantly extend formal verification of `Duration` and `Epoch`, and introduce `kani::Arbitrary` to `Duration` and `Epoch` for users to formally verify their use of time, cf. [#192](https://github.com/nyx-space/hifitime/pull/192)
+ It is now possible to specify a Leap Seconds file (in IERS format) using the `LeapSecondsFile::from_path` (requires the `std` feature to read the file), cf. [#43](https://github.com/nyx-space/hifitime/issues/43).
+ UT1 time scale is now supported! You must build a `Ut1Provider` structure with data from the JPL Earth Orientation Parameters, or just use `Ut1Provider::download_short_from_jpl()` to automatically download the data from NASA JPL.
+ `strptime` and `strftime` equivalents from C89 are now supported, cf. [#181](https://github.com/nyx-space/hifitime/issues/181). Please refer to the [documentation](https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html) for important limitations and how to build a custom formatter.
+ ISO Day of Year and Day In Year are now supported for initialization of an Epoch (provided a time scale and a year), and formatting, cf. [#182](https://github.com/nyx-space/hifitime/issues/182).
+ **Python:** the representation of an epoch is now in the time scale it was initialized in

## 3.7.0
Huge thanks to [@gwbres](https://github.com/gwbres) who put in all of the work for this release. These usability changes allow [Rinex](https://crates.io/crates/rinex) to use hifitime, check out this work.
+ timescale.rs: derive serdes traits when feasible by @gwbres in https://github.com/nyx-space/hifitime/pull/167
+ timecale.rs: introduce format/display by @gwbres in https://github.com/nyx-space/hifitime/pull/168
+ readme: fix BeiDou typo by @gwbres in https://github.com/nyx-space/hifitime/pull/169
+ epoch: derive Hash by @gwbres in https://github.com/nyx-space/hifitime/pull/170
+ timescale: identify GNSS timescales from standard 3 letter codes by @gwbres in https://github.com/nyx-space/hifitime/pull/171
+ timescale: standard formatting is now available by @gwbres in https://github.com/nyx-space/hifitime/pull/174
+ epoch, duration: improve and fix serdes feature by @gwbres in https://github.com/nyx-space/hifitime/pull/175
+ epoch, timescale: implement default trait by @gwbres in https://github.com/nyx-space/hifitime/pull/176

## 3.6.0
+ Galileo System Time and BeiDou Time are now supported, huge thanks to [@gwbres](https://github.com/gwbres) for all that work!
+ Significant speed improvement in the initialization of Epochs from their Gregorian representation, thanks [@conradludgate](https://github.com/conradludgate) for [#160](https://github.com/nyx-space/hifitime/pull/160).
+ Epoch and Duration now have a `min` and `max` function which respectively returns a copy of the epoch/duration that is the smallest or the largest between `self` and `other`, cf. [#164](https://github.com/nyx-space/hifitime/issues/164).
+ [Python] Duration and Epochs now support the operators `>`, `>=`, `<`, `<=`, `==`, and `!=`. Epoch now supports `init_from_gregorian` with a time scape, like in Rust. Epochs can also be subtracted from one another using the `timedelta` function, cf. [#162](https://github.com/nyx-space/hifitime/issues/162).
+ TimeSeries can now be formatted in different time scales, cf. [#163](https://github.com/nyx-space/hifitime/issues/163)

## 3.5.0
+ Epoch now store the time scale that they were defined in: this allows durations to be added in their respective time scales. For example, adding 36 hours to 1971-12-31 at noon when the Epoch is initialized in UTC will lead to a different epoch than adding that same duration to an epoch initialized at the same time in TAI (because the first leap second announced by IERS was on 1972-01-01), cf. the `test_add_durations_over_leap_seconds` test.
+ RFC3339 and ISO8601 fully supported for initialization of an Epoch, including the offset, e.g. `Epoch::from_str("1994-11-05T08:15:30-05:00")`, cf. [#73](https://github.com/nyx-space/hifitime/issues/73).
+ Python package available on PyPI! To build the Python package, you must first install `maturin` and then build with the `python` feature flag. For example, `maturin develop -F python && python` will build the Python package in debug mode and start a new shell where the package can be imported.
+ Fix bug when printing Duration::MIN (or any duration whose centuries are minimizing the number of centuries).
+ TimeSeries can now be formatted
+ Epoch can now be `ceil`-ed, `floor`-ed, and `round`-ed according to the time scale they were initialized in, cf. [#145](https://github.com/nyx-space/hifitime/issues/145).
+ Epoch can now be initialized from Gregorian when specifying the time system: `from_gregorian`, `from_gregorian_hms`, `from_gregorian_at_noon`, `from_gregorian_at_midnight`.
+ Fix bug in Duration when performing operations on durations very close to `Duration::MIN` (i.e. minus thirty-two centuries).
+ Duration parsing now supports multiple units in a string and does not use regular expressions. THis allows it to work with `no-std`.
+ Epoch parsing no longer requires `regex`.
+ Functions are not more idiomatic: all of the `as_*` functions become `to_*` and `in_*` also becomes `to_*`, cf.  [#155](https://github.com/nyx-space/hifitime/issues/155).

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

# Important Update on Versioning Strategy

We want to inform our users of an important change in Hifitime's versioning approach. Starting with version 3.9.0, minor version updates may include changes that could potentially break backward compatibility. While we strive to maintain stability and minimize disruptions, this change allows us to incorporate significant improvements and adapt more swiftly to evolving user needs. We recommend users to carefully review the release notes for each update, even minor ones, to understand any potential impacts on their existing implementations. Our commitment to providing a robust and dynamic time management library remains steadfast, and we believe this change in versioning will better serve the evolving demands of our community.

# Development

Thanks for considering to help out on Hifitime!

For Rust development, `cargo` is all you need, along with build tools for the minimum supported Rust version.

## Python development

First, please install [maturin](https://www.maturin.rs/) and set up a Python virtual environment from which to develop. Also make sure that the package version in Cargo.toml is _greater_ than any published version. For example, if the latest version published on [PyPi](https://pypi.org/project/hifitime/) is 4.0.0-a.0 (for alpha-0), make sure that you change the Cargo.toml file such that you're at least in version `alpha-1`, or the `pip install` will download from PyPi instead of installing from the local folder. To run the Python tests, you must install `pytest` in your virtual environment.

The exact steps should be:

1. Jump into the virtual environment: `source .venv/bin/activate` (e.g.)
1. Make sure pytest is installed: `pip install pytest`
1. Build hifitime specifying the output folder of the Python egg: `maturin build -F python --out dist`
1. Install the egg: `pip install dist/hifitime-4.0.0.dev1-cp311-cp311-linux_x86_64.whl`
1. Run the tests using the environmental pytest: `.venv/bin/pytest`