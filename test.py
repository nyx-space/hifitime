from hifitime import Epoch, Duration

# Test subtracting a Duration from an Epoch
epoch1 = Epoch("2024-01-01 00:00:00 UTC")
duration = Duration("1 day")
epoch2 = epoch1 - duration
print(f"{epoch1} - {duration} = {epoch2}")
assert epoch2 == Epoch("2023-12-31 00:00:00 UTC")

# Test subtracting an Epoch from an Epoch
epoch3 = Epoch("2024-01-02 00:00:00 UTC")
duration2 = epoch3 - epoch1
print(f"{epoch3} - {epoch1} = {duration2}")
assert duration2 == Duration("1 day")

# Test subtracting a non-Epoch/Duration object
try:
    epoch1 - 1
except TypeError as e:
    print(f"Caught expected error: {e}")
    assert "unsupported operand type(s) for -" in str(e)
