# hifitime 0.0.1

Precise date and time handling in Rust built on top of
[` std::time::Duration`](https://doc.rust-lang.org/std/time/struct.Duration.html).
The Epoch used is TAI Epoch of 01 Jan 1900 at midnight, but that should not matter in
day-to-day use of this library.


[![Build Status](https://travis-ci.org/ChristopherRabotin/hifitime.svg?branch=master)](https://travis-ci.org/ChristopherRabotin/hifitime)
[![codecov](https://codecov.io/gh/ChristopherRabotin/hifitime/branch/master/graph/badge.svg)](https://codecov.io/gh/ChristopherRabotin/hifitime)


## Features

 * [x] Leap seconds (as announced by the IETF on a yearly basis)
 * [x] Julian dates and Modified Julian dates
 * [x] UTC representation with ISO8601 formatting
 * [x] Allows building custom TimeSystem (e.g. Julian days)
 * [x] Time varying `TimeZone`s to represent static or very high speed reference frames (cf. the `tz` test in the `tests` module)

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

## Does not include

* [ ] Dates only, or times only (i.e. handles only the combination of both)
* [ ] Custom formatting of date time objects (cf. [issue \#4](https://github.com/ChristopherRabotin/hifitime/issues/4))
* [ ] An initializer from machine time (cf. [issue \#8](https://github.com/ChristopherRabotin/hifitime/issues/8))
* [ ] A simple to use TimeZone offset (cf. [issue \#12](https://github.com/ChristopherRabotin/hifitime/issues/12))
