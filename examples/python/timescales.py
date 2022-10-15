'''
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
'''

try:
    import plotly.express as px
except ImportError:
    print('\nThis script requires `plotly` (pip install plotly)\n')

try:
    import pandas as pd
except ImportError:
    print('\nThis script requires `pandas` (pip install pandas)\n')

from hifitime import Epoch, TimeSeries, Unit

if __name__ == "__main__":
    '''
    The purpose of this script is to plot the differences between time systems.

    It will plot the difference between TAI, UTC, and all of the other timescales supported by hifitime.
    Then, as a separate plot, it will remove the UTC line to make the difference between other timescales more evident.
    '''
    # Start by building a time series from 1970 until 2023 with a step of 30 days.
    ts = TimeSeries(Epoch('1970-01-01 00:00:00 UTC'), Epoch('2023-01-01 00:00:00 UTC'),
                    Unit.Day * 30.0, True)

    # Define the storage array
    data = []
    # Define the columns
    columns = ["UTC Epoch", "Δ TT (s)", "Δ ET (s)", "Δ TDB (s)", "Δ UTC (s)", "ET-TDB (s)"]

    for epoch in ts:
        delta_utc = epoch.to_utc_duration() - epoch.to_tai_duration()
        delta_tt = epoch.to_tt_duration() - epoch.to_tai_duration()
        delta_tdb = epoch.to_tdb_duration_since_j1900() - epoch.to_tai_duration()
        delta_et = epoch.to_et_duration_since_j1900() - epoch.to_tai_duration()
        delta_et_tdb = delta_et - delta_tdb
        # Convert the epoch into a pandas datetime
        pd_epoch = pd.to_datetime(str(epoch))
        # Build the pandas series
        data.append([
            pd_epoch,
            delta_tt.to_seconds(),
            delta_et.to_seconds(),
            delta_tdb.to_seconds(),
            delta_utc.to_seconds(),
            delta_et_tdb.to_seconds(),
        ])

    df = pd.DataFrame(data, columns=columns)

    fig = px.line(df,
                  x='UTC Epoch',
                  y=columns[1:-1],
                  title="Time scale deviation with respect to TAI")
    fig.write_html("./target/time-scale-deviation.html")
    fig.show()

    fig = px.line(df,
                  x='UTC Epoch',
                  y=columns[1:-2],
                  title="Time scale deviation with respect to TAI (excl. UTC)")
    fig.write_html("./target/time-scale-deviation-no-utc.html")
    fig.show()

    fig = px.line(df,
                  x='UTC Epoch',
                  y=columns[-1],
                  title="Time scale deviation of TDB and ET with respect to TAI")
    fig.write_html("./target/time-scale-deviation-tdb-et.html")
    fig.show()