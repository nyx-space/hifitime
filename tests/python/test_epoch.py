from hifitime import Duration, Epoch, HifitimeError, ParsingError, Polynomial, TimeScale, TimeSeries, Unit, Weekday
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
    e = Epoch.init_from_gregorian(year=2022, month=3, day=1, hour=1, minute=1, second=59, nanos=1, time_scale=TimeScale.GPST)
    assert e.strftime("%Y %m %d %H %M %S %f %T") == "2022 03 01 01 01 59 000000001 GPST"
    e = Epoch.init_from_gregorian(year=2022, month=3, day=1, hour=1, minute=1, second=59, nanos=1, time_scale=TimeScale.UTC)
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
    assert Epoch.fromdatetime(tz_datetime.replace(tzinfo=None)) == Epoch("2023-10-08 15:30:00")


def test_polynomial():
    t_gpst = Epoch.init_from_gregorian(2020, 1, 1, 0, 0, 0, 0, TimeScale.GPST)

    gpst_utc_polynomials = Polynomial.from_constant_offset_nanoseconds(1.0)
    gpst_reference = t_gpst - Unit.Hour * 1.0
    t_utc = t_gpst.precise_timescale_conversion(True, gpst_reference, gpst_utc_polynomials, TimeScale.UTC)

    assert t_utc.time_scale == TimeScale.UTC

    reversed = t_utc.to_time_scale(TimeScale.GPST) + Unit.Nanosecond * 1.0
    assert reversed == t_gpst

    backwards = t_utc.precise_timescale_conversion(False, gpst_reference, gpst_utc_polynomials, TimeScale.GPST)
    assert backwards == t_gpst

    gpst_reference = t_gpst - Unit.Minute * 30.0
    t_utc = t_gpst.precise_timescale_conversion(True, gpst_reference, gpst_utc_polynomials, TimeScale.UTC)
    assert t_utc.time_scale == TimeScale.UTC

    reversed = t_utc.to_time_scale(TimeScale.GPST) + Unit.Nanosecond * 1.0
    assert reversed == t_gpst
