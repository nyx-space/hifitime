'''
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
'''

from hifitime import Duration, Epoch, TimeSeries, TimeScale, Unit

if __name__ == "__main__":
    # All of the functions available in Rust are also available in Python.
    # However, functions that initialize a new Epoch or Duration had to be renamed: `Epoch::from_blah` becomes `Epoch.init_from_blah` in Python.

    # Let's start by getting the system time.
    e = Epoch.system_now()
    # By default, it'll be printed in UTC
    print(f"UTC epoch: {e}")
    # But we can also print it in TAI
    print(f"TAI epoch: {e.to_gregorian_tai()}")

    # Or we can print it in a specific time system
    print(f"ET epoch: {e.to_string_gregorian(TimeScale.ET)}")
    print(f"TDB epoch: {e.to_string_gregorian(TimeScale.TDB)}")

    # Hifitime mainly allows for nanosecond precision of durations for 64 centuries (centered on J1900).
    print(f"min negative = {Duration.min_negative()}")
    print(f"min positive = {Duration.min_positive()}")

    # And more importantly, it does not suffer from rounding issues, even when the duration are very large.
    print(f"Max duration: {Duration.max()}")    # 1196851200 days
    print(f"Nanosecond precision: {Duration.max() - Unit.Nanosecond * 1.0}")
    assert f"{Unit.Day * 1.2}" == "1 days 4 h 48 min"
    assert f"{Unit.Day * 1.200001598974}" == "1 days 4 h 48 min 138 ms 151 μs 353 ns"

    # It also saturates the duration
    print(f"Saturated add: {Duration.max() + Unit.Day * 1.0}")

    # You can also get all of the epochs between two different epochs at a specific step size.
    # This is like numpy's `linspace` with high fidelity durations
    time_series = TimeSeries(Epoch.system_now(),
                             Epoch.system_now() + Unit.Day * 0.3,
                             Unit.Hour * 0.5,
                             inclusive=True)
    print(time_series)
    for (num, epoch) in enumerate(time_series):
        print(f"#{num}:\t{epoch}")

    e1 = Epoch.system_now()
    e3 = e1 + Unit.Day * 1.5998
    epoch_delta = e3.timedelta(e1)
    assert epoch_delta == Unit.Day * 1 + Unit.Hour * 14 + Unit.Minute * 23 + Unit.Second * 42.720
    print(epoch_delta)