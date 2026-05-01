# Introduction to Hifitime

Hifitime is a powerful Rust and Python library designed for time management. It provides extensive functionalities with precise operations for time calculation in different time scales, making it suitable for engineering and scientific applications where general relativity and time dilation matter. Hifitime guarantees nanosecond precision for 65,536 centuries around the reference epoch of the initialization time scale, e.g. 01 January 1900 for TAI. Hifitime is also formally verified using the [`Kani` model checker](https://model-checking.github.io/kani/), read more about it [this verification here](https://model-checking.github.io/kani-verifier-blog/2023/03/31/how-kani-helped-find-bugs-in-hifitime.html).

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

Hifitime offers means to convert between time scales coarsely and precisely.
The `Polynomial` structure allows description of the state of a Timescale with respect
to a reference, as typically needed by precise applications or Timescale monitoring & maintenance.

## Non-features
* Time-agnostic / date-only epochs. Hifitime only supports the combination of date and time, but the `Epoch::{at_midnight, at_noon}` is provided as helper functions.

# Design
No software is perfect, so please report any issue or bug on [Github](https://github.com/nyx-space/hifitime/issues/new).

## Duration
Under the hood, a Duration is represented as a 16 bit signed integer of centuries (`i16`) and a 64 bit unsigned integer (`u64`) of the nanoseconds past that century. The overflowing and underflowing of nanoseconds is handled by changing the number of centuries such that the nanoseconds number never represents more than one century (just over four centuries can be stored in 64 bits).

Advantages:
1. Exact precision of a duration: using a floating point value would cause large durations (e.g. Julian Dates) to have less precision than smaller durations. Durations in hifitime have exactly one nanosecond of precision for 65,536 centuries.
2. Skipping floating point operations allows this library to be used in embedded devices without a floating point unit.
3. Duration arithmetics are exact, e.g. one third of an hour is exactly twenty minutes and not "0.33333 hours."

Disadvantages:
1. Most astrodynamics applications require the computation of a duration in floating point values such as when querying an ephemeris. This design leads to an overhead of about 5.2 nanoseconds according to the benchmarks (`Duration to f64 seconds` benchmark). You may run the benchmarks with `cargo bench`.

## Epoch
The Epoch stores a duration with respect to the reference of a time scale, and that time scale itself. For monotonic time on th Earth, [Standard of Fundamental Astronomy (SOFA)](https://www.iausofa.org/) recommends of opting for a glitch-free time scale like TAI (i.e. without discontinuities like leap seconds or non-uniform seconds like TDB).

## Leap second support

Leap seconds allow TAI (the absolute time reference) and UTC (the civil time reference) to not drift too much. In short, UTC allows humans to see the sun at zenith at noon, whereas TAI does not worry about that. Leap seconds are introduced to allow for UTC to catch up with the absolute time reference of TAI. Specifically, UTC clocks are "stopped" for one second to make up for the accumulated difference between TAI and UTC. These leap seconds are announced several months in advance by IERS, cf. in the [IETF leap second reference](https://data.iana.org/time-zones/data/leap-seconds.list).

The "placement" of these leap seconds in the formatting of a UTC date is left up to the software: there is no common way to handle this. Some software prevents a second tick, i.e. at 23:59:59 the UTC clock will tick for _two seconds_ (instead of one) before hoping to 00:00:00. Some software, like hifitime, allow UTC dates to be formatted as 23:59:60 on strictly the days when a leap second is inserted. For example, the date `2016-12-31 23:59:60 UTC` is a valid date in hifitime because a leap second was inserted on 01 Jan 2017.

As of version **4.1.2**, you may call `LatestLeapSeconds::default().is_up_to_date()` in Rust and `LatestLeapSeconds().is_up_to_date()` in Python to check that Hifitime is up to date with the latest leap seconds. In Rust, you'll need to enable the `lts` (long term support) crate feature.

### Important
Prior to the first leap second, NAIF SPICE claims that there were nine seconds of difference between TAI and UTC: this is different from the [Standard of Fundamental Astronomy (SOFA)](https://www.iausofa.org/). SOFA's `iauDat` function will return non-integer leap seconds from 1960 to 1972. It will return an error for dates prior to 1960. **Hifitime only accounts for leap seconds announced by [IERS](https://data.iana.org/time-zones/data/leap-seconds.list)** in its computations: there is a ten (10) second jump between TAI and UTC on 01 January 1972. This allows the computation of UNIX time to be a specific offset of TAI in hifitime. However, the prehistoric (pre-1972) leap seconds as returned by SOFA are available in the `leap_seconds()` method of an epoch if the `iers_only` parameter is set to false.

## Ephemeris Time vs Dynamic Barycentric Time (TDB)
In theory, as of January 2000, ET and TDB should now be identical. _However_, the NASA NAIF leap seconds files (e.g. [naif00012.tls](./naif00012.tls)) use a simplified algorithm to compute the TDB:
> Equation \[4\], which ignores small-period fluctuations, is accurate to about 0.000030 seconds.

In order to provide full interoperability with NAIF, hifitime uses the NAIF algorithm for "ephemeris time" and the [ESA algorithm](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB) for "dynamical barycentric time." Hence, if exact NAIF behavior is needed, use all of the functions marked as `et` instead of the `tdb` functions, such as `epoch.to_et_seconds()` instead of `epoch.to_tdb_seconds()`.

## Formal Verification with Kani

Hifitime uses the [Kani model checker](https://model-checking.github.io/kani/) for formal verification. The verification harnesses serve two purposes:

**Panic-freedom proofs (`#[kani::proof]`):** Most harnesses verify that functions do not panic for any possible input. These use `kani::any()` to generate fully symbolic inputs and call the function under test. They prove the absence of arithmetic overflow, division by zero, out-of-bounds access, and other runtime failures across the entire input space.

**Functional correctness contracts (`#[kani::ensures]` + `#[kani::proof_for_contract]`):** Selected functions have [formal specifications](https://model-checking.github.io/kani/reference/experimental/contracts.html) attached directly to the function signature. These contracts express postconditions that the function must satisfy, for example, that `total_nanoseconds()` returns `centuries * NPC + nanoseconds`, or that `Duration::min` returns a value no greater than either input. The `proof_for_contract` harnesses verify these contracts for all inputs, enabling compositional verification: callers can rely on the contract without re-verifying the implementation.

**Loop contracts (`#[kani::loop_invariant]`):** The `Duration::Mul<f64>` precision-finding loop is annotated with a loop invariant that bounds the iteration variable, enabling Kani to verify termination inductively rather than by unrolling.

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
1. Install the egg: `pip install dist/hifitime-4.0.0.alpha1-cp311-cp311-linux_x86_64.whl`
1. Run the tests using the environmental pytest: `.venv/bin/pytest`

### Type hinting

Hifitime uses the approach from [`dora`](https://github.com/dora-rs/dora/pull/493) to enable type hinting in IDEs. This approach requires running `maturin` twice: once to generate to the bindings and a second time for it to incorporate the `pyi` file.

```bash
maturin develop -F python;
python generate_stubs.py hifitime hifitime.pyi;
maturin develop
```
