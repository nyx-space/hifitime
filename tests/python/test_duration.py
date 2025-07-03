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

    try:
        Duration("invalid")
    except BaseException as e:
        print(f"caught {e}")
    else:
        raise AssertionError("failed to catch as exception")
