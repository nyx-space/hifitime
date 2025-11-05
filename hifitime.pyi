import datetime
import typing

@typing.final
class Duration:
    """Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.

**Important conventions:**
1. The negative durations can be mentally modeled "BC" years. One hours before 01 Jan 0000, it was "-1" years but  365 days and 23h into the current day.
It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
a duration with centuries = -1 and nanoseconds = 0 is _a greater duration_ (further from zero) than centuries = -1 and nanoseconds = 1.
Duration zero minus one nanosecond returns a century of -1 and a nanosecond set to the number of nanoseconds in one century minus one.
That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.

(Python documentation hints)"""

    def __init__(self, string_repr: str) -> None:
        """Defines generally usable durations for nanosecond precision valid for 32,768 centuries in either direction, and only on 80 bits / 10 octets.

**Important conventions:**
1. The negative durations can be mentally modeled "BC" years. One hours before 01 Jan 0000, it was "-1" years but  365 days and 23h into the current day.
It was decided that the nanoseconds corresponds to the nanoseconds _into_ the current century. In other words,
a duration with centuries = -1 and nanoseconds = 0 is _a greater duration_ (further from zero) than centuries = -1 and nanoseconds = 1.
Duration zero minus one nanosecond returns a century of -1 and a nanosecond set to the number of nanoseconds in one century minus one.
That difference is exactly 1 nanoseconds, where the former duration is "closer to zero" than the latter.
As such, the largest negative duration that can be represented sets the centuries to i16::MAX and its nanoseconds to NANOSECONDS_PER_CENTURY.
2. It was also decided that opposite durations are equal, e.g. -15 minutes == 15 minutes. If the direction of time matters, use the signum function.

(Python documentation hints)"""

    @staticmethod
    def EPSILON():...

    @staticmethod
    def MAX():...

    @staticmethod
    def MIN():...

    @staticmethod
    def MIN_NEGATIVE():...

    @staticmethod
    def MIN_POSITIVE():...

    @staticmethod
    def ZERO():...

    def abs(self) -> Duration:
        """Returns the absolute value of this duration"""

    def approx(self) -> Duration:
        """Rounds this duration to the largest units represented in this duration.

This is useful to provide an approximate human duration. Under the hood, this function uses `round`,
so the "tipping point" of the rounding is half way to the next increment of the greatest unit.
As shown below, one example is that 35 hours and 59 minutes rounds to 1 day, but 36 hours and 1 minute rounds
to 2 days because 2 days is closer to 36h 1 min than 36h 1 min is to 1 day.

# Example

```
use hifitime::{Duration, TimeUnits};

assert_eq!((2.hours() + 3.minutes()).approx(), 2.hours());
assert_eq!((24.hours() + 3.minutes()).approx(), 1.days());
assert_eq!((35.hours() + 59.minutes()).approx(), 1.days());
assert_eq!((36.hours() + 1.minutes()).approx(), 2.days());
assert_eq!((47.hours() + 3.minutes()).approx(), 2.days());
assert_eq!((49.hours() + 3.minutes()).approx(), 2.days());
```"""

    def ceil(self, duration: Duration) -> Duration:
        """Ceils this duration to the closest provided duration

This simply floors then adds the requested duration

# Example
```
use hifitime::{Duration, TimeUnits};

let two_hours_three_min = 2.hours() + 3.minutes();
assert_eq!(two_hours_three_min.ceil(1.hours()), 3.hours());
assert_eq!(two_hours_three_min.ceil(30.minutes()), 2.hours() + 30.minutes());
assert_eq!(two_hours_three_min.ceil(4.hours()), 4.hours());
assert_eq!(two_hours_three_min.ceil(1.seconds()), two_hours_three_min + 1.seconds());
assert_eq!(two_hours_three_min.ceil(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
```"""

    def decompose(self) -> tuple[int,int,int,int,int,int,int,int]:
        """Decomposes a Duration in its sign, days, hours, minutes, seconds, ms, us, ns"""

    def floor(self, duration: Duration) -> Duration:
        """Floors this duration to the closest duration from the bottom

# Example
```
use hifitime::{Duration, TimeUnits};

let two_hours_three_min = 2.hours() + 3.minutes();
assert_eq!(two_hours_three_min.floor(1.hours()), 2.hours());
assert_eq!(two_hours_three_min.floor(30.minutes()), 2.hours());
// This is zero because we floor by a duration longer than the current duration, rounding it down
assert_eq!(two_hours_three_min.floor(4.hours()), 0.hours());
assert_eq!(two_hours_three_min.floor(1.seconds()), two_hours_three_min);
assert_eq!(two_hours_three_min.floor(1.hours() + 1.minutes()), 2.hours() + 2.minutes());
assert_eq!(two_hours_three_min.floor(1.hours() + 5.minutes()), 1.hours() + 5.minutes());
```"""

    @staticmethod
    def from_all_parts(sign: int, days: int, hours: int, minutes: int, seconds: int, milliseconds: int, microseconds: int, nanoseconds: int) -> Duration:
        """Creates a new duration from its parts"""

    @staticmethod
    def from_days(value: float) -> Duration:
        """Creates a new duration from the provided number of days"""

    @staticmethod
    def from_hours(value: float) -> Duration:
        """Creates a new duration from the provided number of hours"""

    @staticmethod
    def from_microseconds(value: float) -> Duration:
        """Creates a new duration from the provided number of microseconds"""

    @staticmethod
    def from_milliseconds(value: float) -> Duration:
        """Creates a new duration from the provided number of milliseconds"""

    @staticmethod
    def from_minutes(value: float) -> Duration:
        """Creates a new duration from the provided number of minutes"""

    @staticmethod
    def from_nanoseconds(value: float) -> Duration:
        """Creates a new duration from the provided number of nanoseconds"""

    @staticmethod
    def from_parts(centuries: int, nanoseconds: int) -> Duration:
        """Create a normalized duration from its parts"""

    @staticmethod
    def from_seconds(value: float) -> Duration:
        """Creates a new duration from the provided number of seconds"""

    @staticmethod
    def from_total_nanoseconds(nanos: int) -> Duration:
        """Creates a new Duration from its full nanoseconds"""

    def is_negative(self) -> bool:
        """Returns whether this is a negative or positive duration."""

    def max(self, other: Duration) -> Duration:
        """Returns the maximum of the two durations.

```
use hifitime::TimeUnits;

let d0 = 20.seconds();
let d1 = 21.seconds();

assert_eq!(d1, d1.max(d0));
assert_eq!(d1, d0.max(d1));
```"""

    def min(self, other: Duration) -> Duration:
        """Returns the minimum of the two durations.

```
use hifitime::TimeUnits;

let d0 = 20.seconds();
let d1 = 21.seconds();

assert_eq!(d0, d1.min(d0));
assert_eq!(d0, d0.min(d1));
```"""

    def round(self, duration: Duration) -> Duration:
        """Rounds this duration to the closest provided duration

This performs both a `ceil` and `floor` and returns the value which is the closest to current one.
# Example
```
use hifitime::{Duration, TimeUnits};

let two_hours_three_min = 2.hours() + 3.minutes();
assert_eq!(two_hours_three_min.round(1.hours()), 2.hours());
assert_eq!(two_hours_three_min.round(30.minutes()), 2.hours());
assert_eq!(two_hours_three_min.round(4.hours()), 4.hours());
assert_eq!(two_hours_three_min.round(1.seconds()), two_hours_three_min);
assert_eq!(two_hours_three_min.round(1.hours() + 5.minutes()), 2.hours() + 10.minutes());
```"""

    def signum(self) -> int:
        """Returns the sign of this duration
+ 0 if the number is zero
+ 1 if the number is positive
+ -1 if the number is negative"""

    def to_parts(self) -> tuple[int,int]:
        """Returns the centuries and nanoseconds of this duration
NOTE: These items are not public to prevent incorrect durations from being created by modifying the values of the structure directly."""

    def to_seconds(self) -> float:
        """Returns this duration in seconds f64.
For high fidelity comparisons, it is recommended to keep using the Duration structure."""

    def to_unit(self, unit: Unit) -> float:...

    def total_nanoseconds(self) -> int:
        """Returns the total nanoseconds in a signed 128 bit integer"""

    def __add__():
        """Return self+value."""

    def __div__():...

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __getnewargs__(self):...

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __mul__():
        """Return self*value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __radd__():
        """Return value+self."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __rmul__():
        """Return value*self."""

    def __rsub__():
        """Return value-self."""

    def __str__(self) -> str:
        """Return str(self)."""

    def __sub__():
        """Return self-value."""

@typing.final
class DurationError:
    __cause__: typing.Any
    __context__: typing.Any
    __suppress_context__: typing.Any
    __traceback__: typing.Any
    args: typing.Any

    def add_note():
        """Exception.add_note(note) --
add a note to the exception"""

    def with_traceback():
        """Exception.with_traceback(tb) --
set self.__traceback__ to tb and return self."""

    def __getattribute__():
        """Return getattr(self, name)."""

    def __init__():
        """Initialize self.  See help(type(self)) for accurate signature."""

    def __repr__():
        """Return repr(self)."""

    def __setstate__():...

    def __str__():
        """Return str(self)."""

@typing.final
class Epoch:
    """Defines a nanosecond-precision Epoch.

Refer to the appropriate functions for initializing this Epoch from different time scales or representations.

(Python documentation hints)"""
    duration: Duration
    time_scale: TimeScale

    def __init__(self, string_repr: str) -> None:
        """Defines a nanosecond-precision Epoch.

Refer to the appropriate functions for initializing this Epoch from different time scales or representations.

(Python documentation hints)"""

    def ceil(self, duration: Duration) -> Epoch:
        """Ceils this epoch to the closest provided duration in the TAI time scale

# Example
```
use hifitime::{Epoch, TimeUnits};

let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
assert_eq!(
e.ceil(1.hours()),
Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
);

// 45 minutes is a multiple of 3 minutes, hence this result
let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
assert_eq!(
e.ceil(3.minutes()),
Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 45, 0)
);
```"""

    def day_of_month(self) -> int:
        """Returns the number of days since the start of the Gregorian month in the current time scale.

# Example
```
use hifitime::Epoch;
let dt = Epoch::from_gregorian_tai_at_midnight(2025, 7, 3);
assert_eq!(dt.day_of_month(), 3);
```"""

    def day_of_year(self) -> float:
        """Returns the number of days since the start of the year."""

    def duration_in_year(self) -> Duration:
        """Returns the duration since the start of the year"""

    def floor(self, duration: Duration) -> Epoch:
        """Floors this epoch to the closest provided duration

# Example
```
use hifitime::{Epoch, TimeUnits};

let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
assert_eq!(
e.floor(1.hours()),
Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 0, 0)
);

let e = Epoch::from_gregorian_tai(2022, 10, 3, 17, 44, 29, 898032665);
assert_eq!(
e.floor(3.minutes()),
Epoch::from_gregorian_tai_hms(2022, 10, 3, 17, 42, 0)
);
```"""

    @staticmethod
    def from_bdt_days(days: float) -> Epoch:
        """Initialize an Epoch from the number of days since the BeiDou Time Epoch,
defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def from_bdt_nanoseconds(nanoseconds: float) -> Epoch:
        """Initialize an Epoch from the number of days since the BeiDou Time Epoch,
defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
This may be useful for time keeping devices that use BDT as a time source."""

    @staticmethod
    def from_bdt_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def from_datetime(dt: datetime.datetime) -> Epoch:
        """Builds an Epoch in UTC from the provided datetime after timezone correction if any is present."""

    @staticmethod
    def from_et_duration(duration_since_j2000: Duration) -> Epoch:
        """Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)"""

    @staticmethod
    def from_et_seconds(seconds_since_j2000: float) -> Epoch:
        """Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)"""

    @staticmethod
    def from_gpst_days(days: float) -> Epoch:
        """Initialize an Epoch from the number of days since the GPS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def from_gpst_nanoseconds(nanoseconds: float) -> Epoch:
        """Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
This may be useful for time keeping devices that use GPS as a time source."""

    @staticmethod
    def from_gpst_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the number of seconds since the GPS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def from_gregorian(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int, time_scale: TimeScale) -> Epoch:
        """Initialize from the Gregorian parts"""

    @staticmethod
    def from_gregorian_at_midnight(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
        """Initialize from the Gregorian parts, time set to midnight"""

    @staticmethod
    def from_gregorian_at_noon(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
        """Initialize from the Gregorian parts, time set to noon"""

    @staticmethod
    def from_gregorian_utc(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int) -> Epoch:
        """Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
Use maybe_from_gregorian_tai if unsure."""

    @staticmethod
    def from_gst_days(days: float) -> Epoch:
        """Initialize an Epoch from the number of days since the Galileo Time Epoch,
starting on August 21st 1999 Midnight UT,
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def from_gst_nanoseconds(nanoseconds: float) -> Epoch:
        """Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
starting on August 21st 1999 Midnight UT,
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
This may be useful for time keeping devices that use GST as a time source."""

    @staticmethod
    def from_gst_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
starting on August 21st 1999 Midnight UT,
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def from_jde_et(days: float) -> Epoch:
        """Initialize from the JDE days"""

    @staticmethod
    def from_jde_tai(days: float) -> Epoch:
        """Initialize an Epoch from given JDE in TAI time scale"""

    @staticmethod
    def from_jde_tdb(days: float) -> Epoch:
        """Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days"""

    @staticmethod
    def from_jde_utc(days: float) -> Epoch:
        """Initialize an Epoch from given JDE in UTC time scale"""

    @staticmethod
    def from_mjd_tai(days: float) -> Epoch:
        """Initialize an Epoch from given MJD in TAI time scale"""

    @staticmethod
    def from_mjd_utc(days: float) -> Epoch:
        """Initialize an Epoch from given MJD in UTC time scale"""

    @staticmethod
    def from_ptp_duration(duration: Duration) -> Epoch:
        """Initialize an Epoch from the provided IEEE 1588-2008 (PTPv2) duration since TAI midnight 1970 January 01.
PTP uses the TAI timescale but with the Unix Epoch for compatibility with unix systems."""

    @staticmethod
    def from_ptp_nanoseconds(nanoseconds: int) -> Epoch:
        """Initialize an Epoch from the provided IEEE 1588-2008 (PTPv2) nanoseconds timestamp since TAI midnight 1970 January 01.
PTP uses the TAI timescale but with the Unix Epoch for compatibility with unix systems."""

    @staticmethod
    def from_ptp_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the provided IEEE 1588-2008 (PTPv2) second timestamp since TAI midnight 1970 January 01.
PTP uses the TAI timescale but with the Unix Epoch for compatibility with unix systems."""

    @staticmethod
    def from_qzsst_days(days: float) -> Epoch:
        """Initialize an Epoch from the number of days since the QZSS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def from_qzsst_nanoseconds(nanoseconds: int) -> Epoch:
        """Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
This may be useful for time keeping devices that use QZSS as a time source."""

    @staticmethod
    def from_qzsst_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def from_tai_days(days: float) -> Epoch:
        """Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight"""

    @staticmethod
    def from_tai_duration(duration: Duration) -> Epoch:
        """Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch."""

    @staticmethod
    def from_tai_parts(centuries: int, nanoseconds: int) -> Epoch:
        """Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch."""

    @staticmethod
    def from_tai_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight"""

    @staticmethod
    def from_tdb_duration(duration_since_j2000: Duration) -> Epoch:
        """Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI."""

    @staticmethod
    def from_tdb_seconds(seconds_j2000: float) -> Epoch:
        """Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct."""

    @staticmethod
    def from_tt_duration(duration: Duration) -> Epoch:
        """Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

    @staticmethod
    def from_tt_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

    @staticmethod
    def from_unix_milliseconds(milliseconds: float) -> Epoch:
        """Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01."""

    @staticmethod
    def from_unix_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01."""

    @staticmethod
    def from_ut1_duration(duration: Duration, provider: Ut1Provider) -> Epoch:
        """Initialize a new Epoch from a duration in UT1"""

    @staticmethod
    def from_utc_days(days: float) -> Epoch:
        """Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight"""

    @staticmethod
    def from_utc_seconds(seconds: float) -> Epoch:
        """Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight"""

    @staticmethod
    def fromdatetime(dt: datetime.datetime) -> Epoch:
        """Builds an Epoch in UTC from the provided datetime. Datetime must either NOT have any timezone, or timezone MUST be UTC."""

    def hours(self) -> int:
        """Returns the hours of the Gregorian representation  of this epoch in the time scale it was initialized in."""

    @staticmethod
    def init_from_bdt_days(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_bdt_days` instead
Initialize an Epoch from the number of days since the BeiDou Time Epoch,
defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def init_from_bdt_nanoseconds(nanoseconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_bdt_nanoseconds` instead
Initialize an Epoch from the number of days since the BeiDou Time Epoch,
defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
This may be useful for time keeping devices that use BDT as a time source."""

    @staticmethod
    def init_from_bdt_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_bdt_seconds` instead
Initialize an Epoch from the number of seconds since the BeiDou Time Epoch,
defined as January 1st 2006 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def init_from_et_duration(duration_since_j2000: Duration) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_et_duration` instead
Initialize an Epoch from the Ephemeris Time duration past 2000 JAN 01 (J2000 reference)"""

    @staticmethod
    def init_from_et_seconds(seconds_since_j2000: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_et_seconds` instead
Initialize an Epoch from the Ephemeris Time seconds past 2000 JAN 01 (J2000 reference)"""

    @staticmethod
    def init_from_gpst_days(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gpst_days` instead
Initialize an Epoch from the number of days since the GPS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def init_from_gpst_nanoseconds(nanoseconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gpst_nanoseconds` instead
Initialize an Epoch from the number of nanoseconds since the GPS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
This may be useful for time keeping devices that use GPS as a time source."""

    @staticmethod
    def init_from_gpst_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gpst_seconds` instead
Initialize an Epoch from the number of seconds since the GPS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def init_from_gregorian(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int, time_scale: TimeScale) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gregorian` instead
Initialize from the Gregorian parts"""

    @staticmethod
    def init_from_gregorian_at_midnight(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gregorian_at_midnight` instead
Initialize from the Gregorian parts, time set to midnight"""

    @staticmethod
    def init_from_gregorian_at_noon(year: int, month: int, day: int, time_scale: TimeScale) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gregorian_at_noon` instead
Initialize from the Gregorian parts, time set to noon"""

    @staticmethod
    def init_from_gregorian_utc(year: int, month: int, day: int, hour: int, minute: int, second: int, nanos: int) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gregorian_utc` instead
Builds an Epoch from the provided Gregorian date and time in TAI. If invalid date is provided, this function will panic.
Use maybe_from_gregorian_tai if unsure."""

    @staticmethod
    def init_from_gst_days(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gst_days` instead
Initialize an Epoch from the number of days since the Galileo Time Epoch,
starting on August 21st 1999 Midnight UT,
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def init_from_gst_nanoseconds(nanoseconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gst_nanoseconds` instead
Initialize an Epoch from the number of nanoseconds since the Galileo Time Epoch,
starting on August 21st 1999 Midnight UT,
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
This may be useful for time keeping devices that use GST as a time source."""

    @staticmethod
    def init_from_gst_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_gst_seconds` instead
Initialize an Epoch from the number of seconds since the Galileo Time Epoch,
starting on August 21st 1999 Midnight UT,
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    @staticmethod
    def init_from_jde_et(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_jde_et` instead
Initialize from the JDE days"""

    @staticmethod
    def init_from_jde_tai(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_jde_tai` instead
Initialize an Epoch from given JDE in TAI time scale"""

    @staticmethod
    def init_from_jde_tdb(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_jde_tdb` instead
Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) in JD days"""

    @staticmethod
    def init_from_jde_utc(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_jde_utc` instead
Initialize an Epoch from given JDE in UTC time scale"""

    @staticmethod
    def init_from_mjd_tai(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_mjd_tai` instead
Initialize an Epoch from given MJD in TAI time scale"""

    @staticmethod
    def init_from_mjd_utc(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_mjd_utc` instead
Initialize an Epoch from given MJD in UTC time scale"""

    @staticmethod
    def init_from_qzsst_days(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_qzsst_days` instead
Initialize an Epoch from the number of days since the QZSS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def init_from_qzsst_nanoseconds(nanoseconds: int) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_qzsst_nanoseconds` instead
Initialize an Epoch from the number of nanoseconds since the QZSS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
This may be useful for time keeping devices that use QZSS as a time source."""

    @staticmethod
    def init_from_qzsst_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_qzsst_seconds` instead
Initialize an Epoch from the number of seconds since the QZSS Time Epoch,
defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    @staticmethod
    def init_from_tai_days(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tai_days` instead
Initialize an Epoch from the provided TAI days since 1900 January 01 at midnight"""

    @staticmethod
    def init_from_tai_duration(duration: Duration) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tai_duration` instead
Creates a new Epoch from a Duration as the time difference between this epoch and TAI reference epoch."""

    @staticmethod
    def init_from_tai_parts(centuries: int, nanoseconds: int) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tai_parts` instead
Creates a new Epoch from its centuries and nanosecond since the TAI reference epoch."""

    @staticmethod
    def init_from_tai_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tai_seconds` instead
Initialize an Epoch from the provided TAI seconds since 1900 January 01 at midnight"""

    @staticmethod
    def init_from_tdb_duration(duration_since_j2000: Duration) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tdb_duration` instead
Initialize from Dynamic Barycentric Time (TDB) (same as SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI."""

    @staticmethod
    def init_from_tdb_seconds(seconds_j2000: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tdb_seconds` instead
Initialize an Epoch from Dynamic Barycentric Time (TDB) seconds past 2000 JAN 01 midnight (difference than SPICE)
NOTE: This uses the ESA algorithm, which is a notch more complicated than the SPICE algorithm, but more precise.
In fact, SPICE algorithm is precise +/- 30 microseconds for a century whereas ESA algorithm should be exactly correct."""

    @staticmethod
    def init_from_tt_duration(duration: Duration) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tt_duration` instead
Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

    @staticmethod
    def init_from_tt_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_tt_seconds` instead
Initialize an Epoch from the provided TT seconds (approximated to 32.184s delta from TAI)"""

    @staticmethod
    def init_from_unix_milliseconds(milliseconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_unix_milliseconds` instead
Initialize an Epoch from the provided UNIX millisecond timestamp since UTC midnight 1970 January 01."""

    @staticmethod
    def init_from_unix_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_unix_seconds` instead
Initialize an Epoch from the provided UNIX second timestamp since UTC midnight 1970 January 01."""

    @staticmethod
    def init_from_utc_days(days: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_utc_days` instead
Initialize an Epoch from the provided UTC days since 1900 January 01 at midnight"""

    @staticmethod
    def init_from_utc_seconds(seconds: float) -> Epoch:
        """WARNING: Deprecated since 4.1.1; Use `from_utc_seconds` instead
Initialize an Epoch from the provided UTC seconds since 1900 January 01 at midnight"""

    def isoformat(self) -> str:
        """Equivalent to `datetime.isoformat`, and truncated to 23 chars, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options"""

    def leap_seconds(self, iers_only: bool) -> float:
        """Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds and the SOFA scaling from 1960 to 1972, depending on flag.
Returns None if the epoch is before 1960, year at which UTC was defined.

# Why does this function return an `Option` when the other returns a value
This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960."""

    def leap_seconds_iers(self) -> int:
        """Get the accumulated number of leap seconds up to this Epoch accounting only for the IERS leap seconds."""

    def leap_seconds_with_file(self, iers_only: bool, provider: LeapSecondsFile) -> float:
        """Get the accumulated number of leap seconds up to this Epoch from the provided LeapSecondProvider.
Returns None if the epoch is before 1960, year at which UTC was defined.

# Why does this function return an `Option` when the other returns a value
This is to match the `iauDat` function of SOFA (src/dat.c). That function will return a warning and give up if the start date is before 1960."""

    def max(self, other: Epoch) -> Epoch:
        """Returns the maximum of the two epochs.

```
use hifitime::Epoch;

let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);

assert_eq!(e1, e1.max(e0));
assert_eq!(e1, e0.max(e1));
```

_Note:_ this uses a pointer to `self` which will be copied immediately because Python requires a pointer."""

    def microseconds(self) -> int:
        """Returns the microseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

    def milliseconds(self) -> int:
        """Returns the milliseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

    def min(self, other: Epoch) -> Epoch:
        """Returns the minimum of the two epochs.

```
use hifitime::Epoch;

let e0 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 20);
let e1 = Epoch::from_gregorian_utc_at_midnight(2022, 10, 21);

assert_eq!(e0, e1.min(e0));
assert_eq!(e0, e0.min(e1));
```

_Note:_ this uses a pointer to `self` which will be copied immediately because Python requires a pointer."""

    def minutes(self) -> int:
        """Returns the minutes of the Gregorian representation  of this epoch in the time scale it was initialized in."""

    def month_name(self) -> MonthName:...

    def nanoseconds(self) -> int:
        """Returns the nanoseconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

    def next(self, weekday: Weekday) -> Epoch:
        """Returns the next weekday.

```
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
assert_eq!(epoch.weekday_utc(), Weekday::Saturday);
assert_eq!(epoch.next(Weekday::Sunday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 3));
assert_eq!(epoch.next(Weekday::Monday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 4));
assert_eq!(epoch.next(Weekday::Tuesday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 5));
assert_eq!(epoch.next(Weekday::Wednesday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 6));
assert_eq!(epoch.next(Weekday::Thursday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 7));
assert_eq!(epoch.next(Weekday::Friday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 8));
assert_eq!(epoch.next(Weekday::Saturday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 9));
```"""

    def next_weekday_at_midnight(self, weekday: Weekday) -> Epoch:...

    def next_weekday_at_noon(self, weekday: Weekday) -> Epoch:...

    def precise_timescale_conversion(self, forward: bool, reference_epoch: Epoch, polynomial: Polynomial, target: TimeScale) -> Epoch:
        """Converts this [Epoch] into targeted [TimeScale] using provided [Polynomial].

## Input
- forward: whether this is forward or backward conversion.
For example, using GPST-UTC [Polynomial]
- GPST->UTC is the forward conversion
- UTC->GPST is the backward conversion
- reference_epoch: any reference [Epoch] for the provided [Polynomial].

While we support any time difference, it should remain short in pratice (a day at most, for precise applications).
- polynomial: that must be valid for this reference [Epoch], used in the equation `a0 + a1*dt + a2*dt² = GPST-UTC` for example.
- target: targetted [TimeScale] we will transition to.

Example:
```
use hifitime::{Epoch, TimeScale, Polynomial, Unit};

// random GPST Epoch for forward conversion to UTC
let t_gpst = Epoch::from_gregorian(2020, 01, 01, 0, 0, 0, 0, TimeScale::GPST);

// Let's say we know the GPST-UTC polynomials for that day,
// They allow precise forward transition from GPST to UTC,
// and precise backward transition from UTC to GPST.
let gpst_utc_polynomials = Polynomial::from_constant_offset_nanoseconds(1.0);

// This is the reference [Epoch] attached to the publication of these polynomials.
// You should use polynomials that remain valid and were provided recently (usually one day at most).
// Example: polynomials were published 1 hour ago.
let gpst_reference = t_gpst - 1.0 * Unit::Hour;

// Forward conversion (to UTC) GPST - a0 + a1 *dt + a2*dt² = UTC
let t_utc = t_gpst.precise_timescale_conversion(true, gpst_reference, gpst_utc_polynomials, TimeScale::UTC)
.unwrap();

// Verify we did transition to UTC
assert_eq!(t_utc.time_scale, TimeScale::UTC);

// Verify the resulting [Epoch] is the coarse GPST->UTC transition + fine correction
let reversed = t_utc.to_time_scale(TimeScale::GPST) + 1.0 * Unit::Nanosecond;
assert_eq!(reversed, t_gpst);

// Apply the backward transition, from t_utc back to t_gpst.
// The timescale conversion works both ways: (from UTC) GPST = UTC + a0 + a1 *dt + a2*dt²
let backwards = t_utc.precise_timescale_conversion(false, gpst_reference, gpst_utc_polynomials, TimeScale::GPST)
.unwrap();

assert_eq!(backwards, t_gpst);

// It is important to understand that your reference point does not have to be in the past.
// The only logic that should prevail is to always minimize interpolation gap.
// In other words, if you can access future interpolation information that would minimize the data gap, they should prevail.
// Example: +30' in the future.
let gpst_reference = t_gpst + 30.0 * Unit::Minute;

// Forward conversion (to UTC) but using polynomials that were released 1 hour after t_gpst
let t_utc = t_gpst.precise_timescale_conversion(true, gpst_reference, gpst_utc_polynomials, TimeScale::UTC)
.unwrap();

// Verifications
assert_eq!(t_utc.time_scale, TimeScale::UTC);

let reversed = t_utc.to_time_scale(TimeScale::GPST) + 1.0 * Unit::Nanosecond;
assert_eq!(reversed, t_gpst);
```"""

    def previous(self, weekday: Weekday) -> Epoch:
        """Returns the next weekday.

```
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc_at_midnight(1988, 1, 2);
assert_eq!(epoch.previous(Weekday::Friday), Epoch::from_gregorian_utc_at_midnight(1988, 1, 1));
assert_eq!(epoch.previous(Weekday::Thursday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 31));
assert_eq!(epoch.previous(Weekday::Wednesday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 30));
assert_eq!(epoch.previous(Weekday::Tuesday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 29));
assert_eq!(epoch.previous(Weekday::Monday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 28));
assert_eq!(epoch.previous(Weekday::Sunday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 27));
assert_eq!(epoch.previous(Weekday::Saturday), Epoch::from_gregorian_utc_at_midnight(1987, 12, 26));
```"""

    def previous_weekday_at_midnight(self, weekday: Weekday) -> Epoch:...

    def previous_weekday_at_noon(self, weekday: Weekday) -> Epoch:...

    def round(self, duration: Duration) -> Epoch:
        """Rounds this epoch to the closest provided duration in TAI

# Example
```
use hifitime::{Epoch, TimeUnits};

let e = Epoch::from_gregorian_tai_hms(2022, 5, 20, 17, 57, 43);
assert_eq!(
e.round(1.hours()),
Epoch::from_gregorian_tai_hms(2022, 5, 20, 18, 0, 0)
);
```"""

    def seconds(self) -> int:
        """Returns the seconds of the Gregorian representation  of this epoch in the time scale it was initialized in."""

    def strftime(self, format_str: str) -> str:
        """Formats the epoch according to the given format string. Supports a subset of C89 and hifitime-specific format codes. Refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for available format options."""

    @staticmethod
    def strptime(epoch_str: str, format_str: str) -> Epoch:
        """Equivalent to `datetime.strptime`, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options"""

    @staticmethod
    def system_now() -> Epoch:
        """Returns the computer clock in UTC"""

    def timedelta(self, other: Duration) -> Duration:
        """Differences between two epochs"""

    def to_bdt_days(self) -> float:
        """Returns days past BDT (BeiDou) Time Epoch, defined as Jan 01 2006 UTC
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    def to_bdt_duration(self) -> Duration:
        """Returns `Duration` past BDT (BeiDou) time Epoch."""

    def to_bdt_nanoseconds(self) -> int:
        """Returns nanoseconds past BDT (BeiDou) Time Epoch, defined as Jan 01 2006 UTC
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
NOTE: This function will return an error if the centuries past GST time are not zero."""

    def to_bdt_seconds(self) -> float:
        """Returns seconds past BDT (BeiDou) Time Epoch"""

    def to_datetime(self, set_tz: bool=None) -> datetime.datetime:
        """Returns a Python datetime object from this Epoch (truncating the nanoseconds away)
If set_tz is True, then this will return a time zone aware datetime object"""

    def to_duration_in_time_scale(self, ts: TimeScale) -> Duration:
        """Returns this epoch with respect to the provided time scale.
This is needed to correctly perform duration conversions in dynamical time scales (e.g. TDB)."""

    def to_et_centuries_since_j2000(self) -> float:
        """Returns the number of centuries since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)"""

    def to_et_days_since_j2000(self) -> float:
        """Returns the number of days since Ephemeris Time (ET) J2000 (used for Archinal et al. rotations)"""

    def to_et_duration(self) -> Duration:
        """Returns the duration between J2000 and the current epoch as per NAIF SPICE.

# Warning
The et2utc function of NAIF SPICE will assume that there are 9 leap seconds before 01 JAN 1972,
as this date introduces 10 leap seconds. At the time of writing, this does _not_ seem to be in
line with IERS and the documentation in the leap seconds list.

In order to match SPICE, the as_et_duration() function will manually get rid of that difference."""

    def to_et_seconds(self) -> float:
        """Returns the Ephemeris Time seconds past 2000 JAN 01 midnight, matches NASA/NAIF SPICE."""

    def to_gpst_days(self) -> float:
        """Returns days past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    def to_gpst_duration(self) -> Duration:
        """Returns `Duration` past GPS time Epoch."""

    def to_gpst_nanoseconds(self) -> int:
        """Returns nanoseconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
NOTE: This function will return an error if the centuries past GPST time are not zero."""

    def to_gpst_seconds(self) -> float:
        """Returns seconds past GPS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    def to_gregorian(self, time_scale: TimeScale=None) -> tuple[int,int,int,int,int,int,int]:
        """Converts the Epoch to the Gregorian parts in the (optionally) provided time scale as (year, month, day, hour, minute, second)."""

    def to_gst_days(self) -> float:
        """Returns days past GST (Galileo) Time Epoch,
starting on August 21st 1999 Midnight UT
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>)."""

    def to_gst_duration(self) -> Duration:
        """Returns `Duration` past GST (Galileo) time Epoch."""

    def to_gst_nanoseconds(self) -> int:
        """Returns nanoseconds past GST (Galileo) Time Epoch, starting on August 21st 1999 Midnight UT
(cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS>).
NOTE: This function will return an error if the centuries past GST time are not zero."""

    def to_gst_seconds(self) -> float:
        """Returns seconds past GST (Galileo) Time Epoch"""

    def to_isoformat(self) -> str:
        """The standard ISO format of this epoch (six digits of subseconds) in the _current_ time scale, refer to <https://docs.rs/hifitime/latest/hifitime/efmt/format/struct.Format.html> for format options."""

    def to_jde_et(self, unit: Unit) -> float:...

    def to_jde_et_days(self) -> float:
        """Returns the Ephemeris Time JDE past epoch"""

    def to_jde_et_duration(self) -> Duration:...

    def to_jde_tai(self, unit: Unit) -> float:
        """Returns the Julian Days from epoch 01 Jan -4713 12:00 (noon) in desired Duration::Unit"""

    def to_jde_tai_days(self) -> float:
        """Returns the Julian days from epoch 01 Jan -4713, 12:00 (noon)
as explained in "Fundamentals of astrodynamics and applications", Vallado et al.
4th edition, page 182, and on [Wikipedia](https://en.wikipedia.org/wiki/Julian_day)."""

    def to_jde_tai_duration(self) -> Duration:
        """Returns the Julian Days from epoch 01 Jan -4713 12:00 (noon) as a Duration"""

    def to_jde_tai_seconds(self) -> float:
        """Returns the Julian seconds in TAI."""

    def to_jde_tdb_days(self) -> float:
        """Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)"""

    def to_jde_tdb_duration(self) -> Duration:...

    def to_jde_tt_days(self) -> float:
        """Returns days past Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

    def to_jde_tt_duration(self) -> Duration:...

    def to_jde_utc_days(self) -> float:
        """Returns the Julian days in UTC."""

    def to_jde_utc_duration(self) -> Duration:
        """Returns the Julian days in UTC as a `Duration`"""

    def to_jde_utc_seconds(self) -> float:
        """Returns the Julian Days in UTC seconds."""

    def to_mjd_tai(self, unit: Unit) -> float:
        """Returns this epoch as a duration in the requested units in MJD TAI"""

    def to_mjd_tai_days(self) -> float:
        """`as_mjd_days` creates an Epoch from the provided Modified Julian Date in days as explained
[here](http://tycho.usno.navy.mil/mjd.html). MJD epoch is Modified Julian Day at 17 November 1858 at midnight."""

    def to_mjd_tai_seconds(self) -> float:
        """Returns the Modified Julian Date in seconds TAI."""

    def to_mjd_tt_days(self) -> float:
        """Returns days past Modified Julian epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

    def to_mjd_tt_duration(self) -> Duration:...

    def to_mjd_utc(self, unit: Unit) -> float:
        """Returns the Modified Julian Date in the provided unit in UTC."""

    def to_mjd_utc_days(self) -> float:
        """Returns the Modified Julian Date in days UTC."""

    def to_mjd_utc_seconds(self) -> float:
        """Returns the Modified Julian Date in seconds UTC."""

    def to_nanoseconds_in_time_scale(self, time_scale: TimeScale) -> int:
        """Attempts to return the number of nanoseconds since the reference epoch of the provided time scale.
This will return an overflow error if more than one century has past since the reference epoch in the provided time scale.
If this is _not_ an issue, you should use `epoch.to_duration_in_time_scale().to_parts()` to retrieve both the centuries and the nanoseconds
in that century."""

    def to_qzsst_days(self) -> float:
        """Returns days past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    def to_qzsst_duration(self) -> Duration:
        """Returns `Duration` past QZSS time Epoch."""

    def to_qzsst_nanoseconds(self) -> int:
        """Returns nanoseconds past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>).
NOTE: This function will return an error if the centuries past QZSST time are not zero."""

    def to_qzsst_seconds(self) -> float:
        """Returns seconds past QZSS Time Epoch, defined as UTC midnight of January 5th to 6th 1980 (cf. <https://gssc.esa.int/navipedia/index.php/Time_References_in_GNSS#GPS_Time_.28GPST.29>)."""

    def to_rfc3339(self) -> str:
        """Returns this epoch in UTC in the RFC3339 format"""

    def to_tai(self, unit: Unit) -> float:
        """Returns the epoch as a floating point value in the provided unit"""

    def to_tai_days(self) -> float:
        """Returns the number of days since J1900 in TAI"""

    def to_tai_duration(self) -> Duration:
        """Returns this time in a Duration past J1900 counted in TAI"""

    def to_tai_parts(self) -> tuple:
        """Returns the TAI parts of this duration"""

    def to_tai_seconds(self) -> float:
        """Returns the number of TAI seconds since J1900"""

    def to_tdb_centuries_since_j2000(self) -> float:
        """Returns the number of centuries since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)"""

    def to_tdb_days_since_j2000(self) -> float:
        """Returns the number of days since Dynamic Barycentric Time (TDB) J2000 (used for Archinal et al. rotations)"""

    def to_tdb_duration(self) -> Duration:
        """Returns the Dynamics Barycentric Time (TDB) as a high precision Duration since J2000

## Algorithm
Given the embedded sine functions in the equation to compute the difference between TDB and TAI from the number of TDB seconds
past J2000, one cannot solve the revert the operation analytically. Instead, we iterate until the value no longer changes.

1. Assume that the TAI duration is in fact the TDB seconds from J2000.
2. Offset to J2000 because `Epoch` stores everything in the J1900 but the TDB duration is in J2000.
3. Compute the offset `g` due to the TDB computation with the current value of the TDB seconds (defined in step 1).
4. Subtract that offset to the latest TDB seconds and store this as a new candidate for the true TDB seconds value.
5. Compute the difference between this candidate and the previous one. If the difference is less than one nanosecond, stop iteration.
6. Set the new candidate as the TDB seconds since J2000 and loop until step 5 breaks the loop, or we've done five iterations.
7. At this stage, we have a good approximation of the TDB seconds since J2000.
8. Reverse the algorithm given that approximation: compute the `g` offset, compute the difference between TDB and TAI, add the TT offset (32.184 s), and offset by the difference between J1900 and J2000."""

    def to_tdb_seconds(self) -> float:
        """Returns the Dynamic Barycentric Time (TDB) (higher fidelity SPICE ephemeris time) whose epoch is 2000 JAN 01 noon TAI (cf. <https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems#TDT_-_TDB.2C_TCB>)"""

    def to_time_of_week(self) -> tuple:
        """Converts this epoch into the time of week, represented as a rolling week counter into that time scale
and the number of nanoseconds elapsed in current week (since closest Sunday midnight).
This is usually how GNSS receivers describe a timestamp."""

    def to_time_scale(self, ts: TimeScale) -> Epoch:
        """Converts self to another time scale

As per the [Rust naming convention](https://rust-lang.github.io/api-guidelines/naming.html#ad-hoc-conversions-follow-as_-to_-into_-conventions-c-conv),
this borrows an Epoch and returns an owned Epoch."""

    def to_tt_centuries_j2k(self) -> float:
        """Returns the centuries passed J2000 TT"""

    def to_tt_days(self) -> float:
        """Returns days past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

    def to_tt_duration(self) -> Duration:
        """Returns `Duration` past TAI epoch in Terrestrial Time (TT)."""

    def to_tt_seconds(self) -> float:
        """Returns seconds past TAI epoch in Terrestrial Time (TT) (previously called Terrestrial Dynamical Time (TDT))"""

    def to_tt_since_j2k(self) -> Duration:
        """Returns the duration past J2000 TT"""

    def to_unix(self, unit: Unit) -> float:
        """Returns the duration since the UNIX epoch in the provided unit."""

    def to_unix_days(self) -> float:
        """Returns the number days since the UNIX epoch defined 01 Jan 1970 midnight UTC."""

    def to_unix_duration(self) -> Duration:
        """Returns the Duration since the UNIX epoch UTC midnight 01 Jan 1970."""

    def to_unix_milliseconds(self) -> float:
        """Returns the number milliseconds since the UNIX epoch defined 01 Jan 1970 midnight UTC."""

    def to_unix_seconds(self) -> float:
        """Returns the number seconds since the UNIX epoch defined 01 Jan 1970 midnight UTC."""

    def to_ut1(self, provider: Ut1Provider) -> Epoch:
        """Convert this epoch to Ut1"""

    def to_ut1_duration(self, provider: Ut1Provider) -> Duration:
        """Returns this time in a Duration past J1900 counted in UT1"""

    def to_utc(self, unit: Unit) -> float:
        """Returns the number of UTC seconds since the TAI epoch"""

    def to_utc_days(self) -> float:
        """Returns the number of UTC days since the TAI epoch"""

    def to_utc_duration(self) -> Duration:
        """Returns this time in a Duration past J1900 counted in UTC"""

    def to_utc_seconds(self) -> float:
        """Returns the number of UTC seconds since the TAI epoch"""

    def todatetime(self, set_tz: bool=None) -> datetime.datetime:
        """Returns a Python datetime object from this Epoch (truncating the nanoseconds away).
If set_tz is True, then this will return a time zone aware datetime object"""

    def ut1_offset(self, provider: Ut1Provider) -> Duration:
        """Get the accumulated offset between this epoch and UT1."""

    def weekday(self) -> Weekday:
        """Returns weekday (uses the TAI representation for this calculation)."""

    def weekday_in_time_scale(self, time_scale: TimeScale) -> Weekday:
        """Returns the weekday in provided time scale **ASSUMING** that the reference epoch of that time scale is a Monday.
You _probably_ do not want to use this. You probably either want `weekday()` or `weekday_utc()`.
Several time scales do _not_ have a reference day that's on a Monday, e.g. BDT."""

    def weekday_utc(self) -> Weekday:
        """Returns weekday in UTC timescale"""

    def with_hms(self, hours: int, minutes: int, seconds: int) -> Epoch:
        """Returns a copy of self where the time is set to the provided hours, minutes, seconds
Invalid number of hours, minutes, and seconds will overflow into their higher unit.
Warning: this does _not_ set the subdivisions of second to zero."""

    def with_hms_from(self, other: Epoch) -> Epoch:
        """Returns a copy of self where the hours, minutes, seconds is set to the time of the provided epoch but the
sub-second parts are kept from the current epoch.

```
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
let other = other_utc.to_time_scale(TimeScale::TDB);

assert_eq!(
epoch.with_hms_from(other),
Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 13)
);
```"""

    def with_hms_strict(self, hours: int, minutes: int, seconds: int) -> Epoch:
        """Returns a copy of self where the time is set to the provided hours, minutes, seconds
Invalid number of hours, minutes, and seconds will overflow into their higher unit.
Warning: this will set the subdivisions of seconds to zero."""

    def with_hms_strict_from(self, other: Epoch) -> Epoch:
        """Returns a copy of self where the time is set to the time of the other epoch but the subseconds are set to zero.

```
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
let other = other_utc.to_time_scale(TimeScale::TDB);

assert_eq!(
epoch.with_hms_strict_from(other),
Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 0)
);
```"""

    def with_time_from(self, other: Epoch) -> Epoch:
        """Returns a copy of self where all of the time components (hours, minutes, seconds, and sub-seconds) are set to the time of the provided epoch.

```
use hifitime::prelude::*;

let epoch = Epoch::from_gregorian_utc(2022, 12, 01, 10, 11, 12, 13);
let other_utc = Epoch::from_gregorian_utc(2024, 12, 01, 20, 21, 22, 23);
// If the other Epoch is in another time scale, it does not matter, it will be converted to the correct time scale.
let other = other_utc.to_time_scale(TimeScale::TDB);

assert_eq!(
epoch.with_time_from(other),
Epoch::from_gregorian_utc(2022, 12, 01, 20, 21, 22, 23)
);
```"""

    def year(self) -> int:
        """Returns the number of Gregorian years of this epoch in the current time scale."""

    def year_days_of_year(self) -> tuple:
        """Returns the year and the days in the year so far (days of year)."""

    def __add__():
        """Return self+value."""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __getnewargs__(self):...

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __radd__():
        """Return value+self."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __rsub__():
        """Return value-self."""

    def __str__(self) -> str:
        """Return str(self)."""

    def __sub__():
        """Return self-value."""

@typing.final
class HifitimeError:
    __cause__: typing.Any
    __context__: typing.Any
    __suppress_context__: typing.Any
    __traceback__: typing.Any
    args: typing.Any

    def add_note():
        """Exception.add_note(note) --
add a note to the exception"""

    def with_traceback():
        """Exception.with_traceback(tb) --
set self.__traceback__ to tb and return self."""

    def __getattribute__():
        """Return getattr(self, name)."""

    def __init__():
        """Initialize self.  See help(type(self)) for accurate signature."""

    def __repr__():
        """Return repr(self)."""

    def __setstate__():...

    def __str__():
        """Return str(self)."""

@typing.final
class LatestLeapSeconds:
    """List of leap seconds from <https://data.iana.org/time-zones/data/leap-seconds.list>.
This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
The unannoucned leap seconds come from dat.c in the SOFA library."""

    def __init__(self) -> None:
        """List of leap seconds from <https://data.iana.org/time-zones/data/leap-seconds.list>.
This list corresponds the number of seconds in TAI to the UTC offset and to whether it was an announced leap second or not.
The unannoucned leap seconds come from dat.c in the SOFA library."""

    def is_up_to_date(self) -> bool:
        """Downloads the latest leap second list from IANA, and returns whether the embedded leap seconds are still up to date

```
use hifitime::leap_seconds::LatestLeapSeconds;

assert!(LatestLeapSeconds::default().is_up_to_date().unwrap(), "Hifitime needs to update its leap seconds list!");
```"""

    def __repr__(self) -> str:
        """Return repr(self)."""

@typing.final
class LeapSecondsFile:
    """A leap second provider that uses an IERS formatted leap seconds file."""

    def __init__(self, path: str) -> None:
        """A leap second provider that uses an IERS formatted leap seconds file."""

    def __repr__(self) -> str:
        """Return repr(self)."""

@typing.final
class MonthName:
    """Defines Month names, can be initialized either from its variant or its integer (1 for January)."""

    def __init__(self, month: int) -> None:
        """Defines Month names, can be initialized either from its variant or its integer (1 for January)."""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __int__(self) -> None:
        """int(self)"""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""
    April: MonthName = ...
    August: MonthName = ...
    December: MonthName = ...
    February: MonthName = ...
    January: MonthName = ...
    July: MonthName = ...
    June: MonthName = ...
    March: MonthName = ...
    May: MonthName = ...
    November: MonthName = ...
    October: MonthName = ...
    September: MonthName = ...

@typing.final
class ParsingError:
    __cause__: typing.Any
    __context__: typing.Any
    __suppress_context__: typing.Any
    __traceback__: typing.Any
    args: typing.Any

    def add_note():
        """Exception.add_note(note) --
add a note to the exception"""

    def with_traceback():
        """Exception.with_traceback(tb) --
set self.__traceback__ to tb and return self."""

    def __getattribute__():
        """Return getattr(self, name)."""

    def __init__():
        """Initialize self.  See help(type(self)) for accurate signature."""

    def __repr__():
        """Return repr(self)."""

    def __setstate__():...

    def __str__():
        """Return str(self)."""

@typing.final
class Polynomial:
    """Interpolation [Polynomial] used for example in [TimeScale]
maintenance, precise monitoring or conversions."""

    def __init__(self) -> None:
        """Interpolation [Polynomial] used for example in [TimeScale]
maintenance, precise monitoring or conversions."""

    def correction_duration(self, time_interval: Duration) -> Duration:
        """Calculate the correction (as [Duration] once again) from [Self] and given
the interpolation time interval"""

    @staticmethod
    def from_constant_offset(constant: Duration) -> Polynomial:
        """Create a [Polynomial] structure that is only made of a static offset"""

    @staticmethod
    def from_constant_offset_nanoseconds(nanos: float) -> Polynomial:
        """Create a [Polynomial] structure from a static offset expressed in nanoseconds"""

    @staticmethod
    def from_offset_and_rate(constant: Duration, rate: Duration) -> Polynomial:
        """Create a [Polynomial] structure from both static offset and rate of change:"""

    @staticmethod
    def from_offset_rate_nanoseconds(offset_ns: float, drift_ns_s: float) -> Polynomial:
        """Create a [Polynomial] structure from a static offset and drift, in nanoseconds and nanoseconds.s⁻¹"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class TimeScale:
    """Enum of the different time systems available"""

    def __init__(self) -> None:
        """Enum of the different time systems available"""

    def uses_leap_seconds(self) -> bool:
        """Returns true if self takes leap seconds into account"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __int__(self) -> None:
        """int(self)"""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""
    BDT: TimeScale = ...
    ET: TimeScale = ...
    GPST: TimeScale = ...
    GST: TimeScale = ...
    QZSST: TimeScale = ...
    TAI: TimeScale = ...
    TDB: TimeScale = ...
    TT: TimeScale = ...
    UTC: TimeScale = ...

@typing.final
class TimeSeries:
    """An iterator of a sequence of evenly spaced Epochs.

(Python documentation hints)"""

    def __init__(self, start: Epoch, end: Epoch, step: Duration, inclusive: bool) -> None:
        """An iterator of a sequence of evenly spaced Epochs.

(Python documentation hints)"""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __getnewargs__(self):...

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __iter__(self) -> typing.Any:
        """Implement iter(self)."""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __next__(self) -> typing.Any:
        """Implement next(self)."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __str__(self) -> str:
        """Return str(self)."""

@typing.final
class Unit:
    """An Enum to perform time unit conversions."""

    def __init__(self) -> None:
        """An Enum to perform time unit conversions."""

    def from_seconds(self):...

    def in_seconds(self):...

    def __add__():
        """Return self+value."""

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __int__(self) -> None:
        """int(self)"""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __mul__():
        """Return self*value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __radd__():
        """Return value+self."""

    def __repr__(self) -> str:
        """Return repr(self)."""

    def __rmul__():
        """Return value*self."""

    def __rsub__():
        """Return value-self."""

    def __sub__():
        """Return self-value."""
    Century: Unit = ...
    Day: Unit = ...
    Hour: Unit = ...
    Microsecond: Unit = ...
    Millisecond: Unit = ...
    Minute: Unit = ...
    Nanosecond: Unit = ...
    Second: Unit = ...
    Week: Unit = ...

@typing.final
class Ut1Provider:
    """A structure storing all of the TAI-UT1 data"""
    data: list
    iter_pos: int

    def __init__(self) -> None:
        """A structure storing all of the TAI-UT1 data"""

    def as_list(self) -> list:
        """Returns the list of Delta TAI-UT1 values"""

    @staticmethod
    def from_eop_file(path: str) -> Ut1Provider:
        """Builds a UT1 provider from the provided path to an EOP file."""

    def __repr__(self) -> str:
        """Return repr(self)."""

@typing.final
class Weekday:

    def __init__(self):...

    def __eq__(self, value: typing.Any) -> bool:
        """Return self==value."""

    def __ge__(self, value: typing.Any) -> bool:
        """Return self>=value."""

    def __gt__(self, value: typing.Any) -> bool:
        """Return self>value."""

    def __int__(self) -> None:
        """int(self)"""

    def __le__(self, value: typing.Any) -> bool:
        """Return self<=value."""

    def __lt__(self, value: typing.Any) -> bool:
        """Return self<value."""

    def __ne__(self, value: typing.Any) -> bool:
        """Return self!=value."""

    def __repr__(self) -> str:
        """Return repr(self)."""
    Friday: Weekday = ...
    Monday: Weekday = ...
    Saturday: Weekday = ...
    Sunday: Weekday = ...
    Thursday: Weekday = ...
    Tuesday: Weekday = ...
    Wednesday: Weekday = ...