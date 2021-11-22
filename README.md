# hifitime 2

Precise date and time handling in Rust built on top of a tuple of two floats.
The Epoch used is TAI Epoch of 01 Jan 1900 at midnight, but that should not matter in
day-to-day use of this library.


[![Build Status](https://app.travis-ci.com/nyx-space/hifitime.svg?branch=master)](https://app.travis-ci.com/nyx-space/hifitime)
[![hifitime on crates.io][cratesio-image]][cratesio]
[![hifitime on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/hifitime.svg
[cratesio]: https://crates.io/crates/hifitime
[docsrs-image]: https://docs.rs/hifitime/badge.svg
[docsrs]: https://docs.rs/hifitime/


# Features

 * [x] Leap seconds (as announced by the IETF on a yearly basis)
 * [x] Julian dates and Modified Julian dates
 * [x] Clock drift via oscillator stability for simulation of time measuring hardware (via the `simulation` feature)
 * [x] UTC representation with ISO8601 formatting (and parsing in that format #45)
 * [x] High fidelity Ephemeris Time / Dynamic Barycentric Time (TDB) computations from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB) (caveat: up to 10ms difference with SPICE near 01 Jan 2000)
 * [x] Trivial support of time arithmetic (e.g. `TimeUnit::Hour * 2 + TimeUnit::Second * 3`)
 * [x] Supports ranges of Epochs and TimeSeries (linspace of `Epoch`s and `Duration`s)
 * [ ] Support for custom representations of time (e.g. NASA GMAT Modified Julian Date)
 * [ ] Trivial support of other time representations, such as TDT (cf #44)

Almost all examples are validated with external references, as detailed on a test-by-test
basis.

# Validation example
Validation is done using NASA's SPICE toolkit, and specifically the [spiceypy](https://spiceypy.readthedocs.io/) Python wrapper.

The most challenging validation is the definition of Ephemeris Time, which is very nearly the same as the Dynamic Barycentric Time (TDB).
These calculations in hifitime are from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB).

Hifitime uses a fixed offset for the computation of Ephemeris Time, as is recommended in Navipedia. For TDB however, the offset is based on the centuries since J2000 TT and therefore time varying.
I believe that SPICE uses TDB for all dates after J2000 TT. Hence, in the following validation, we will be comparing the SPICE ET with the Hifitime TDB.

The following examples are executed as part of the standard test suite (cf. the function called `spice_et_tdb`).

## Case 1
In SPICE, we chose to convert the UTC date `2012-02-07 11:22:33 UTC` into Ephemeris Time. SPICE responds with `381885819.18493587`.
Initializing the same UTC date in hifitime and requesting the TDB leads to `381885819.18493646`, which is an error of **596.05 nanoseconds**.

## Case 2
In SPICE, we chose to convert the UTC date `2002-02-07 00:00:00.000 UTC` into Ephemeris Time. SPICE responds with `66312064.18493876`.
Initializing the same UTC date in hifitime and requesting the TDB leads to a difference **618.39 nanoseconds**.

## Case 3
This tests that we can correctly compute TDB time which will have a negative number of days because the UTC input is prior to J2000 TT.
In SPICE, we chose to convert the UTC date `1996-02-07 11:22:33 UTC` into Ephemeris Time. SPICE responds with `-123035784.81506048`.
Initializing the same UTC date in hifitime and requesting the TDB leads to a difference **610.94 nanoseconds**.

## Case 4
In SPICE, we chose to convert the UTC date `2015-02-07 00:00:00.000 UTC` into Ephemeris Time. SPICE responds with `476580220.1849411`.
Initializing the same UTC date in hifitime and requesting the TDB leads to a difference **596.05 nanoseconds**.

## Case 5
In SPICE, we chose to convert the TDB Julian Date in days `2452312.500372511` into Ephemeris Time, and initialize a Hifitime Epoch with that result (`66312032.18493909`).
We then convert that epoch back into **days** of Julian Date TDB and Julian Date ET, both of which lead a difference **below machine precision** on a f64 (the equivalent of a double in C/C++).

# Notes

Please report and bugs by [clicking here](https://github.com/ChristopherRabotin/hifitime/issues/new).

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

## 2.2.3
+ More deterministic `as_jde_tdb_days()` in `Epoch`. In version 2.2.1, the ephemeris time and TDB _days_ were identical down to machine precision. After a number of validation cases in the rotation equations of the IAU Earth to Earth Mean Equator J2000 frame, the new formulation was shown to lead to less rounding errors when requesting the days. These rounding errors prevented otherwise trivial test cases. However, it adds an error of **40.2 nanoseconds** when initializing an Epoch with the days in ET and requesting the TDB days.

_Note:_ this was originally published as 2.2.2 but I'd forgotten to update one of the tests with the 40.2 ns error.