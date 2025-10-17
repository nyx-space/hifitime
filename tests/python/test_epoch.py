from hifitime import (
    Duration,
    Epoch,
    HifitimeError,
    LatestLeapSeconds,
    MonthName,
    ParsingError,
    Polynomial,
    TimeScale,
    TimeSeries,
    Unit,
    Ut1Provider,
    Weekday,
)
from datetime import datetime, timezone
import pickle


def test_strtime():
    """
    Tests both strp and strftime
    """
    epoch = Epoch("2023-04-13 23:31:17 UTC")
    dt = datetime(2023, 4, 13, 23, 31, 17)

    epoch_fmt = epoch.strftime("%A, %d %B %Y %H:%M:%S")
    dt_fmt = dt.strftime("%A, %d %B %Y %H:%M:%S")

    assert epoch_fmt == dt_fmt

    assert Epoch.strptime(dt_fmt, "%A, %d %B %Y %H:%M:%S") == epoch

    assert pickle.loads(pickle.dumps(epoch)) == epoch

    try:
        epoch.strftime("%o")
    except ParsingError as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch parsing error")

    epoch_tdb = epoch.to_time_scale(TimeScale.TDB)
    assert str(epoch_tdb) == "2023-04-13T23:32:26.185636390 TDB"
    assert epoch_tdb.time_scale == TimeScale.TDB

    assert epoch.next(Weekday.Monday) == Epoch("2023-04-17T23:31:17 UTC")
    assert epoch.month_name() == MonthName.April
    assert MonthName(3) == MonthName.March
    assert int(MonthName.August) == 8


def test_utcnow():
    epoch = Epoch.system_now()
    try:
        dt = datetime.now(timezone.utc)
    except Exception:
        dt = datetime.utcnow()

    # Hifitime uses a different clock to Python and print down to the nanosecond
    assert dt.isoformat()[:20] == f"{epoch}"[:20]


def test_time_series():
    """
    Time series are really cool way to iterate through a time without skipping a beat.
    """

    nye = Epoch("2022-12-31 23:59:00 UTC")

    time_series = TimeSeries(
        nye,
        nye + Unit.Second * 10,
        Unit.Second * 1,
        inclusive=True,
    )
    print(time_series)

    assert pickle.loads(pickle.dumps(time_series)) == time_series

    for num, epoch in enumerate(time_series):
        print(f"#{num}:\t{epoch}")

    assert num == 10
    # Once consumed, the iterator in the time series will be different,
    # so the pickling will return something different
    assert pickle.loads(pickle.dumps(time_series)) != time_series


def test_duration_eq():
    """
    Checks that Duration comparisons work
    """

    assert Unit.Second * 0.0 == Duration("0 ns")
    assert Unit.Second * 1.0 >= Duration("0 ns")
    assert Unit.Second * 1.0 > Duration("0 ns")
    assert Duration("0 ns") <= Unit.Second * 1.0
    assert Duration("0 ns") < Unit.Second * 1.0

    dur = Duration("37 min 26 s")
    assert pickle.loads(pickle.dumps(dur)) == dur


def test_epoch_exceptions():
    try:
        Epoch("invalid")
    except HifitimeError as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch epoch error")

    # Check that we can catch it with the builtin exception types
    try:
        Epoch("invalid")
    except Exception as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch as exception")

    try:
        Epoch("invalid")
    except BaseException as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch as base exception")


def test_regression_gh249():
    e = Epoch.from_gregorian(
        year=2022,
        month=3,
        day=1,
        hour=1,
        minute=1,
        second=59,
        nanos=1,
        time_scale=TimeScale.GPST,
    )
    assert e.strftime("%Y %m %d %H %M %S %f %T") == "2022 03 01 01 01 59 000000001 GPST"
    e = Epoch.from_gregorian(
        year=2022,
        month=3,
        day=1,
        hour=1,
        minute=1,
        second=59,
        nanos=1,
        time_scale=TimeScale.UTC,
    )
    assert e.strftime("%Y %m %d %H %M %S %f %T") == "2022 03 01 01 01 59 000000001 UTC"


def test_interop():
    hifinow = Epoch.system_now()
    lofinow = hifinow.todatetime()
    hifirtn = Epoch.fromdatetime(lofinow)
    assert hifirtn.timedelta(hifinow).abs() < Unit.Microsecond * 1
    # Now test with timezone, expect an error
    tz_datetime = datetime(2023, 10, 8, 15, 30, tzinfo=timezone.utc)
    try:
        Epoch.fromdatetime(tz_datetime)
    except Exception as e:
        print(e)
    else:
        assert False, "tz aware dt did not fail"
    # Repeat after the strip
    assert Epoch.fromdatetime(tz_datetime.replace(tzinfo=None)) == Epoch(
        "2023-10-08 15:30:00"
    )


def test_ephemeris_time_todatetime():
    """
    Test for the issue highlighted by [issue 421][issue-421].

    [issue-421]: https://github.com/nyx-space/hifitime/issues/421
    """
    test_epoch = Epoch("2025-03-07T12:01:09.185475585 ET")

    actual_datetime = test_epoch.todatetime()
    expected_datetime = datetime(year=2025, month=3, day=7, hour=12, minute=0)

    # Before fixing, `actual_datetime = (1925, 3, 8, 0, 1, 9, 185475)`.
    assert actual_datetime == expected_datetime


def test_polynomial():
    t_gpst = Epoch.from_gregorian(2020, 1, 1, 0, 0, 0, 0, TimeScale.GPST)

    gpst_utc_polynomials = Polynomial.from_constant_offset_nanoseconds(1.0)
    gpst_reference = t_gpst - Unit.Hour * 1.0
    t_utc = t_gpst.precise_timescale_conversion(
        True, gpst_reference, gpst_utc_polynomials, TimeScale.UTC
    )

    assert t_utc.time_scale == TimeScale.UTC

    reversed = t_utc.to_time_scale(TimeScale.GPST) + Unit.Nanosecond * 1.0
    assert reversed == t_gpst


def test_with_functions():
    epoch = Epoch("2023-04-13 23:31:17 UTC")
    epoch_other = Epoch("2024-05-14 10:20:30 UTC")

    # Test with_hms
    modified_epoch = epoch.with_hms(1, 2, 3)
    assert modified_epoch.hours() == 1
    assert modified_epoch.minutes() == 2
    assert modified_epoch.seconds() == 3
    assert modified_epoch.nanoseconds() == epoch.nanoseconds()

    # Test with_hms_from
    modified_epoch = epoch.with_hms_from(epoch_other)
    assert modified_epoch.hours() == epoch_other.hours()
    assert modified_epoch.minutes() == epoch_other.minutes()
    assert modified_epoch.seconds() == epoch_other.seconds()
    # Check that subseconds are preserved
    assert modified_epoch.nanoseconds() == epoch.nanoseconds()

    # Test with_time_from
    modified_epoch = epoch.with_time_from(epoch_other)
    assert modified_epoch.hours() == epoch_other.hours()
    assert modified_epoch.minutes() == epoch_other.minutes()
    assert modified_epoch.seconds() == epoch_other.seconds()
    assert modified_epoch.nanoseconds() == epoch_other.nanoseconds()

    # Test with_hms_strict
    modified_epoch = epoch.with_hms_strict(4, 5, 6)
    assert modified_epoch.hours() == 4
    assert modified_epoch.minutes() == 5
    assert modified_epoch.seconds() == 6
    assert modified_epoch.nanoseconds() == 0

    # Test with_hms_strict_from
    modified_epoch = epoch.with_hms_strict_from(epoch_other)
    assert modified_epoch.hours() == epoch_other.hours()
    assert modified_epoch.minutes() == epoch_other.minutes()
    assert modified_epoch.seconds() == epoch_other.seconds()
    assert modified_epoch.nanoseconds() == 0


def test_latest_leap():
    assert LatestLeapSeconds().is_up_to_date(), "hifitime needs updating!"


def test_ut1_provider():
    provider = Ut1Provider.from_eop_file("data/example_eop2.short")

    # 1. Test with an epoch before any data is available in the provider.
    # The provider data starts on 2024-10-17.
    epoch_before = Epoch("2023-04-13 23:31:17 UTC")
    ut1_offset_before = epoch_before.ut1_offset(provider)
    assert ut1_offset_before is None, (
        "Expected no offset for an epoch before the provider's data range"
    )

    # 2. Test with an epoch that falls within the provider's data range.
    # The first data point is for MJD 60600.0 (2024-10-17), with TAI-UT1 = 36943.3633 ms.
    epoch_in_range = Epoch("2024-10-17 12:00:00 UTC")
    ut1_offset_in_range = epoch_in_range.ut1_offset(provider)

    assert ut1_offset_in_range is not None
    expected_offset_seconds = 36.9433633
    assert abs(ut1_offset_in_range.to_seconds() - expected_offset_seconds) < 1e-9, (
        "UT1 offset does not match expected value"
    )

    # 3. Test `to_ut1` and `to_ut1_duration`
    ut1_epoch = epoch_in_range.to_ut1(provider)
    # TAI = UT1 + offset  <=> UT1 = TAI - offset
    # So ut1_epoch should be epoch_in_range - ut1_offset
    assert epoch_in_range.timedelta(ut1_epoch).abs() == ut1_offset_in_range.abs()

    ut1_duration = epoch_in_range.to_ut1_duration(provider)
    assert ut1_duration == epoch_in_range.to_tai_duration() - ut1_offset_in_range

    # 4. Test round-trip with `from_ut1_duration`
    epoch_from_ut1 = Epoch.from_ut1_duration(ut1_duration, provider)
    # This should round trip to epoch_in_range, allowing for small precision errors.
    assert epoch_from_ut1.timedelta(epoch_in_range).abs() == Duration.ZERO()
