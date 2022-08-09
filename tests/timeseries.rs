extern crate hifitime;

use hifitime::{Epoch, TimeSeries, Unit};

#[test]
fn test_timeseries() {
    let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
    let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
    let step = Unit::Hour * 2;

    let mut count = 0;
    let time_series = TimeSeries::exclusive(start, end, step);
    for epoch in time_series {
        if count == 0 {
            assert_eq!(
                epoch, start,
                "Starting epoch of exclusive time series is wrong"
            );
        } else if count == 5 {
            assert_ne!(epoch, end, "Ending epoch of exclusive time series is wrong");
        }
        #[cfg(feature = "std")]
        println!("{}", epoch);
        count += 1;
    }

    assert_eq!(count, 6, "Should have five items in this iterator");

    count = 0;
    let time_series = TimeSeries::inclusive(start, end, step);
    for epoch in time_series {
        if count == 0 {
            assert_eq!(
                epoch, start,
                "Starting epoch of inclusive time series is wrong"
            );
        } else if count == 6 {
            assert_eq!(epoch, end, "Ending epoch of inclusive time series is wrong");
        }
        #[cfg(feature = "std")]
        println!("{}", epoch);
        count += 1;
    }

    assert_eq!(count, 7, "Should have six items in this iterator");
}

#[test]
fn gh131_regression() {
    let start = Epoch::from_gregorian_utc(2022, 7, 14, 2, 56, 11, 228271007);
    let step = 0.5 * Unit::Microsecond;
    let steps = 1_000_000_000;
    let end = start + steps * step; // This is 500 ms later
    let times = TimeSeries::exclusive(start, end, step);
    // For an _exclusive_ time series, we skip the last item, so it's steps minus one
    assert_eq!(times.len(), steps as usize - 1);
    assert_eq!(times.len(), times.size_hint().0);

    // For an _inclusive_ time series, we skip the last item, so it's the steps count
    let times = TimeSeries::inclusive(start, end, step);
    assert_eq!(times.len(), steps as usize);
    assert_eq!(times.len(), times.size_hint().0);
}
