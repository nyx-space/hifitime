import hifitime

def test_constants():
    assert hifitime.DAYS_PER_CENTURY == 36525.0
    assert hifitime.DAYS_IN_CENTURY == 36525.0
    assert hifitime.JD_J2000 == 2451545.0
    assert hifitime.SECONDS_PER_DAY == 86400.0
    assert hifitime.NANOSECONDS_PER_SECOND == 1000000000
    assert hifitime.NANOSECONDS_PER_DAY == 86400 * 1000000000
    assert hifitime.ET_EPOCH_S == 3155716800

    # Just check existence of others
    assert hasattr(hifitime, "JD_J1900")
    assert hasattr(hifitime, "MJD_J1900")
    assert hasattr(hifitime, "MJD_J2000")
    assert hasattr(hifitime, "MJD_OFFSET")
    assert hasattr(hifitime, "DAYS_PER_YEAR")
    assert hasattr(hifitime, "DAYS_PER_YEAR_NLD")
    assert hasattr(hifitime, "DAYS_PER_WEEK")
    assert hasattr(hifitime, "SECONDS_PER_MINUTE")
    assert hasattr(hifitime, "SECONDS_PER_HOUR")
    assert hasattr(hifitime, "SECONDS_PER_CENTURY")
    assert hasattr(hifitime, "SECONDS_PER_YEAR")
    assert hasattr(hifitime, "SECONDS_PER_TROPICAL_YEAR")
    assert hasattr(hifitime, "SECONDS_PER_SIDEREAL_YEAR")
    assert hasattr(hifitime, "NANOSECONDS_PER_MICROSECOND")
    assert hasattr(hifitime, "NANOSECONDS_PER_MILLISECOND")
    assert hasattr(hifitime, "NANOSECONDS_PER_MINUTE")
    assert hasattr(hifitime, "NANOSECONDS_PER_HOUR")
    assert hasattr(hifitime, "NANOSECONDS_PER_CENTURY")
