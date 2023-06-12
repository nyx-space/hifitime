from hifitime import Duration

def test_consts_init():
    print(Duration.MIN())
    print(Duration.MIN_POSITIVE())
    print(Duration.MIN_NEGATIVE())
    print(Duration.MAX())
    print(Duration.EPSILON())