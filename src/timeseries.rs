use super::{Duration, Epoch};
/*

NOTE: This is taken from itertools: https://docs.rs/itertools-num/0.1.3/src/itertools_num/linspace.rs.html#78-93 .

*/

/// An iterator of a sequence of evenly spaced Epochs.
///
/// Iterator element type is `F`.
#[derive(Clone, Debug)]
pub struct TimeSeries {
    start: Epoch,
    end: Epoch,
    step: Duration,
    cur: Epoch,
    incl: bool,
}

impl Iterator for TimeSeries {
    type Item = Epoch;

    #[inline]
    fn next(&mut self) -> Option<Epoch> {
        let next_item = self.cur + self.step;
        if (!self.incl && next_item >= self.end) || (self.incl && next_item > self.end) {
            None
        } else {
            self.cur = next_item;
            Some(next_item)
        }
    }
}

impl DoubleEndedIterator for TimeSeries {
    #[inline]
    fn next_back(&mut self) -> Option<Epoch> {
        let next_item = self.cur - self.step;
        if next_item < self.start {
            None
        } else {
            Some(next_item)
        }
    }
}

impl ExactSizeIterator for TimeSeries where TimeSeries: Iterator {}

/// Return an iterator of evenly spaced Epochs, inclusive on start and _exclusive_ on end.
/// ```
/// use hifitime::{Epoch, TimeUnit};
/// let time_series = epoch_iter(start, end, step);
/// for epoch in time_series {
///     println!("{}", epoch);
/// }
/// ```
#[inline]
pub fn epoch_iter(start: Epoch, end: Epoch, step: Duration) -> TimeSeries {
    TimeSeries {
        start,
        end,
        step,
        cur: start,
        incl: false,
    }
}

/// Return an iterator of evenly spaced Epochs, inclusive on start and inclusive on end.
#[inline]
pub fn epoch_iter_incl(start: Epoch, end: Epoch, step: Duration) -> TimeSeries {
    TimeSeries {
        start,
        end,
        step,
        cur: start,
        incl: true,
    }
}

#[test]
fn test_timeseries() {
    use super::TimeUnit;
    let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
    let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
    let step = TimeUnit::Hour * 2;
    let mut count = 0;

    let time_series = epoch_iter(start, end, step);
    for epoch in time_series {
        println!("{}", epoch);
        count += 1;
    }

    assert_eq!(count, 5, "Should have five items in this iterator");

    count = 0;
    let time_series = epoch_iter_incl(start, end, step);
    for epoch in time_series {
        println!("{}", epoch);
        count += 1;
    }

    assert_eq!(count, 6, "Should have six items in this iterator");
}
