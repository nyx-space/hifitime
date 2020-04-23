# hifitime 1.0

Precise date and time handling in Rust built on top of `std::f64`.
The Epoch used is TAI Epoch of 01 Jan 1900 at midnight, but that should not matter in
day-to-day use of this library.


[![Build Status](https://travis-ci.org/ChristopherRabotin/hifitime.svg?branch=master)](https://travis-ci.org/ChristopherRabotin/hifitime)
[![hifitime on crates.io][cratesio-image]][cratesio]
[![hifitime on docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/hifitime.svg
[cratesio]: https://crates.io/crates/hifitime
[docsrs-image]: https://docs.rs/hifitime/badge.svg?version=1.0
[docsrs]: https://docs.rs/hifitime/1.0/


# Features

 * [x] Leap seconds (as announced by the IETF on a yearly basis)
 * [x] Julian dates and Modified Julian dates
 * [x] Clock drift via oscillator stability for simulation of time measuring hardware (via the `simulation` feature)
 * [x] UTC representation with ISO8601 formatting (and parsing in that format #45)
 * [x] High fidelity Ephemeris Time / Dynamic Barycentric Time (TDB) computations from [ESA's Navipedia](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB) (caveat: up to 10ms difference with SPICE near 01 Jan 2000)
 * [ ] Support for custom representations of time (e.g. NASA GMAT Modified Julian Date)
 * [ ] Trivial support of other time representations, such as TDT (cf #44)

Almost all examples are validated with external references, as detailed on a test-by-test
basis.

## Notes

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