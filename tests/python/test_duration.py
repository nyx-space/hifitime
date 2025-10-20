from hifitime import Duration, HifitimeError


def test_consts_init():
    print(Duration.MIN())
    print(Duration.MIN_POSITIVE())
    print(Duration.MIN_NEGATIVE())
    print(Duration.MAX())
    print(Duration.EPSILON())
    print(Duration.ZERO())


def test_exceptions():
    try:
        Duration("invalid")
    except HifitimeError as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch duration error")

    # Check that we can catch it with the builtin exception types
    try:
        Duration("invalid")
    except Exception as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch as exception")


def test_constructors():
    from hifitime import Unit

    assert Duration.from_seconds(1234.5) == Unit.Second * 1234.5
    assert Duration.from_minutes(1234.5) == Unit.Minute * 1234.5
    assert Duration.from_hours(1234.5) == Unit.Hour * 1234.5
    assert Duration.from_days(1234.5) == Unit.Day * 1234.5
    assert Duration.from_milliseconds(1234.5) == Unit.Millisecond * 1234.5
    assert Duration.from_microseconds(1234.5) == Unit.Microsecond * 1234.5
    assert Duration.from_nanoseconds(1234.5) == Unit.Nanosecond * 1234.5

    try:
        Duration("invalid")
    except BaseException as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch as exception")
