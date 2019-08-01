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


## Features

 * [x] Leap seconds (as announced by the IETF on a yearly basis)
 * [x] Julian dates and Modified Julian dates
 * [x] Clock drift via oscillator stability for simulation of time measuring hardware (via the `simulation` feature)
 * [ ] **TODO**: UTC representation with ISO8601 formatting (and parsing in that format #45)
 * [ ] Support for custom representations of time (e.g. NASA GMAT Modified Julian Date)
 * [ ] Trivial support of other time representations, such as TDT (cf #44)

Almost all examples are validated with external references, as detailed on a test-by-test
basis.

### Leap second support
Each time computing library may decide when the extra leap second exists as explained
in the [IETF leap second reference](https://www.ietf.org/timezones/data/leap-seconds.list).
To ease computation, `hifitime` decides that second is the 60th of a UTC date, if such exists.
Note that this second exists at a different time than defined on NASA HEASARC. That tool is
used for validation of Julian dates. As an example of how this is handled, check the Julian
day computations for [2015-06-30 23:59:59](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A59&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes),
[2015-06-30 23:59:60](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-06-30+23%3A59%3A60&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes) and [2015-07-01 00:00:00](https://heasarc.gsfc.nasa.gov/cgi-bin/Tools/xTime/xTime.pl?time_in_i=2015-07-01+00%3A00%3A00&time_in_c=&time_in_d=&time_in_j=&time_in_m=&time_in_sf=&time_in_wf=&time_in_sl=&time_in_snu=&time_in_s=&time_in_h=&time_in_n=&time_in_f=&time_in_sz=&time_in_ss=&time_in_sn=&timesys_in=u&timesys_out=u&apply_clock_offset=yes).

# Changelog
## version 1.0 - COMPLETE rewrite
The previous API is **totally incompatible** with version 1.0.

### In this new version:
+ Correct and explicit support for TAI versus UTC (previously, UTC was _implied_ in the `Datetime` struct and TAI was _implied_ in the `Instant` struct)
+ Removed annoyingly high precision which prevented simple day-to-day operations on seconds (as needed in most time simulation software)
 
### Motivation
After using this extensively for a few months, I've come to realize that it was highly ambiguous, and frankly a pain in the neck to use because of the complexity of the `Instant` struct. It made common operations complicated for no good reason. And at least one major bug existed (the difference between two `Instant` of MJD was different from the actual difference of MJDs converted to seconds).