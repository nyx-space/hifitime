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
day computations for [2015-06-30 23:59:59](https://goo.gl/o3KXSR),
[2015-06-30 23:59:60](https://goo.gl/QyUyrC) and [2015-07-01 00:00:00](https://goo.gl/Y25hpn).

## Does not include

* [ ] Dates only, or times only (i.e. handles only the combination of both)
* [ ] Custom formatting of date time objects (cf. [issue \#4](https://github.com/ChristopherRabotin/hifitime/issues/4))
* [ ] An initializer from machine time (cf. [issue \#8](https://github.com/ChristopherRabotin/hifitime/issues/8))
* [ ] A simple to use TimeZone offset (cf. [issue \#9](https://github.com/ChristopherRabotin/hifitime/issues/9))

### Note on short links
The validation tools used generate very long URLs, which aren't supported by `rustfmt`.
As such, whenever a validation link is provided, it has been shortened using Google's
http://goo.gl service. If this is an issue, please add `info/` between `goo.gl/` and the
unique identifier: this will allow you to see the redirection link prior to being redirected
(as well as the link analytics). For example, `https://goo.gl/o3KXSR` becomes
`https://goo.gl/info/o3KXSR`.
