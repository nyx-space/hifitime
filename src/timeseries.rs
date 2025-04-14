/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use super::{Duration, Epoch};

use core::fmt;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // Import is indeed used.
use num_traits::Float;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/*

NOTE: This is taken from itertools: https://docs.rs/itertools-num/0.1.3/src/itertools_num/linspace.rs.html#78-93 .

*/

/// An iterator of a sequence of evenly spaced Epochs.
///
/// (Python documentation hints)
/// :type start: Epoch
/// :type end: Epoch
/// :type step: Duration
/// :type inclusive: bool
/// :rtype: TimeSeries
#[cfg_attr(kani, derive(kani::Arbitrary))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "hifitime"))]
pub struct TimeSeries {
    start: Epoch,
    duration: Duration,
    step: Duration,
    cur: i128,
    incl: bool,
}

impl TimeSeries {
    /// Return an iterator of evenly spaced Epochs, **inclusive** on start and **exclusive** on end.
    /// ```
    /// use hifitime::{Epoch, Unit, TimeSeries};
    /// let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
    /// let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
    /// let step = Unit::Hour * 2;
    /// let time_series = TimeSeries::exclusive(start, end, step);
    /// let mut cnt = 0;
    /// for epoch in time_series {
    ///     println!("{}", epoch);
    ///     cnt += 1
    /// }
    /// assert_eq!(cnt, 6)
    /// ```
    #[inline]
    pub fn exclusive(start: Epoch, end: Epoch, step: Duration) -> TimeSeries {
        // Start one step prior to start because next() just moves forward
        Self {
            start,
            duration: end - start,
            step,
            cur: 0,
            incl: false,
        }
    }

    /// Return an iterator of evenly spaced Epochs, inclusive on start **and** on end.
    /// ```
    /// use hifitime::{Epoch, Unit, TimeSeries};
    /// let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
    /// let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
    /// let step = Unit::Hour * 2;
    /// let time_series = TimeSeries::inclusive(start, end, step);
    /// let mut cnt = 0;
    /// for epoch in time_series {
    ///     println!("{}", epoch);
    ///     cnt += 1
    /// }
    /// assert_eq!(cnt, 7)
    /// ```
    #[inline]
    pub fn inclusive(start: Epoch, end: Epoch, step: Duration) -> TimeSeries {
        // Start one step prior to start because next() just moves forward
        Self {
            start,
            duration: end - start,
            step,
            cur: 0,
            incl: true,
        }
    }
}

impl fmt::Display for TimeSeries {
    // Prints this duration with automatic selection of the units, i.e. everything that isn't zero is ignored
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{} : {} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

impl fmt::LowerHex for TimeSeries {
    /// Prints the Epoch in TAI
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{:x} : {:x} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

impl fmt::UpperHex for TimeSeries {
    /// Prints the Epoch in TT
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{:X} : {:X} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

impl fmt::LowerExp for TimeSeries {
    /// Prints the Epoch in TDB
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{:e} : {:e} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

impl fmt::UpperExp for TimeSeries {
    /// Prints the Epoch in ET
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{:E} : {:E} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

impl fmt::Pointer for TimeSeries {
    /// Prints the Epoch in UNIX
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{:p} : {:p} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

impl fmt::Octal for TimeSeries {
    /// Prints the Epoch in GPS
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeSeries [{:o} : {:o} : {}]",
            self.start,
            if self.incl {
                self.start + self.duration
            } else {
                self.start + self.duration - self.step
            },
            self.step
        )
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl TimeSeries {
    #[new]
    /// Return an iterator of evenly spaced Epochs
    /// If inclusive is set to true, this iterator is inclusive on start **and** on end.
    /// If inclusive is set to false, only the start epoch is included in the iteration.
    fn new_py(start: Epoch, end: Epoch, step: Duration, inclusive: bool) -> Self {
        if inclusive {
            Self::inclusive(start, end, step)
        } else {
            Self::exclusive(start, end, step)
        }
    }

    fn __getnewargs__(&self) -> Result<(Epoch, Epoch, Duration, bool), PyErr> {
        Ok((self.start, self.start + self.duration, self.step, self.incl))
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Epoch> {
        slf.next()
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
    }

    #[cfg(feature = "python")]
    fn __eq__(&self, other: Self) -> bool {
        *self == other
    }
}

impl Iterator for TimeSeries {
    type Item = Epoch;

    #[inline]
    fn next(&mut self) -> Option<Epoch> {
        let next_offset = self.cur * self.step;
        if (!self.incl && next_offset >= self.duration)
            || (self.incl && next_offset > self.duration)
        {
            None
        } else {
            self.cur += 1;
            Some(self.start + next_offset)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len() + 1))
    }
}

impl DoubleEndedIterator for TimeSeries {
    #[inline]
    fn next_back(&mut self) -> Option<Epoch> {
        // Offset from the end of the iterator
        self.cur += 1;
        let offset = self.cur * self.step;
        // if offset < -self.duration - self.step {
        if (!self.incl && offset > self.duration)
            || (self.incl && offset > self.duration + self.step)
        {
            None
        } else {
            Some(self.start + self.duration - offset)
        }
    }
}

impl ExactSizeIterator for TimeSeries
where
    TimeSeries: Iterator,
{
    fn len(&self) -> usize {
        let approx = (self.duration.to_seconds() / self.step.to_seconds()).abs();
        if self.incl {
            if approx.ceil() >= usize::MAX as f64 {
                usize::MAX
            } else {
                approx.ceil() as usize
            }
        } else if approx.floor() >= usize::MAX as f64 {
            usize::MAX
        } else {
            approx.floor() as usize
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Epoch, TimeSeries, Unit};

    #[test]
    fn test_exclusive_timeseries() {
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
            println!("tests::exclusive_timeseries::{}", epoch);
            count += 1;
        }

        assert_eq!(count, 6, "Should have five items in this iterator");
    }

    #[test]
    fn test_inclusive_timeseries() {
        let start = Epoch::from_gregorian_utc_at_midnight(2017, 1, 14);
        let end = Epoch::from_gregorian_utc_at_noon(2017, 1, 14);
        let step = Unit::Hour * 2;

        let mut count = 0;
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
            println!("tests::inclusive_timeseries::{}", epoch);
            count += 1;
        }

        assert_eq!(count, 7, "Should have six items in this iterator");
    }

    #[test]
    fn ts_over_leap_second() {
        let start = Epoch::from_gregorian_utc(2016, 12, 31, 23, 59, 59, 0);
        let times = TimeSeries::exclusive(start, start + Unit::Second * 5, Unit::Second * 1);
        let expect_end = start + Unit::Second * 4;
        let mut cnt = 0;
        let mut cur_epoch = start;

        for epoch in times {
            cnt += 1;
            cur_epoch = epoch;
        }

        assert_eq!(cnt, 5); // Five because the first item is always inclusive
        assert_eq!(cur_epoch, expect_end, "incorrect last item in iterator");
    }

    #[test]
    fn ts_backward() {
        let start = Epoch::from_gregorian_utc(2015, 1, 1, 12, 0, 0, 0);
        let times = TimeSeries::exclusive(start, start + Unit::Second * 5, Unit::Second * 1);
        let mut cnt = 0;
        let mut cur_epoch = start;

        for epoch in times.rev() {
            cnt += 1;
            cur_epoch = epoch;
            let expect = start + Unit::Second * (5 - cnt);
            assert_eq!(expect, epoch, "incorrect item in iterator");
        }

        assert_eq!(cnt, 5); // Five because the first item is always inclusive
        assert_eq!(cur_epoch, start, "incorrect last item in iterator");
    }
}
