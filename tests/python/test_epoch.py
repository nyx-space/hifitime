from hifitime import Duration, Epoch, EpochError, ParsingError, TimeScale, TimeSeries, Unit
from datetime import datetime
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


def test_utcnow():
    epoch = Epoch.system_now()
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
    # Once consummed, the iterator in the time series will be different,
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

def test_exceptions():
    try:
        Epoch("invalid")
    except EpochError as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch epoch error")

def test_regression_gh249():
    e = Epoch.init_from_gregorian(year=2022, month=3, day=1, hour=1, minute=1, second=59, nanos=1, time_scale=TimeScale.GPST)
    assert e.strftime("%Y %m %d %H %M %S %f %T") == "2022 03 01 01 01 59 000000001 GPST"
    e = Epoch.init_from_gregorian(year=2022, month=3, day=1, hour=1, minute=1, second=59, nanos=1, time_scale=TimeScale.UTC)
    assert e.strftime("%Y %m %d %H %M %S %f %T") == "2022 03 01 01 01 59 000000001 UTC"