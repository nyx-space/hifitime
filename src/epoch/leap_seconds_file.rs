/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

#[cfg(feature = "python")]
use pyo3::prelude::*;

use std::{fs::File, io::Read, path::Path};

use core::ops::Index;

use crate::{
    leap_seconds::{LeapSecond, LeapSecondProvider},
    EpochError, ParsingError,
};

#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, Default)]
/// A leap second provider that uses an IERS formatted leap seconds file.
pub struct LeapSecondsFile {
    data: Vec<LeapSecond>,
    iter_pos: usize,
}

impl LeapSecondsFile {
    /// Builds a leap second provider from the provided Leap Seconds file in IERS format as found on <https://www.ietf.org/timezones/data/leap-seconds.list> .
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, EpochError> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(EpochError::Parse {
                    source: ParsingError::InOut { err: e.kind() },
                    details: "opening leap seconds file",
                })
            }
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            return Err(EpochError::Parse {
                source: ParsingError::InOut { err: e.kind() },
                details: "reading leap seconds file",
            });
        }

        let mut me = Self::default();

        for line in contents.lines() {
            if let Some(first_char) = line.chars().next() {
                if first_char == '#' {
                    continue;
                } else {
                    // We have data of interest!
                    let data: Vec<&str> = line.split_whitespace().collect();
                    if data.len() < 2 {
                        return Err(EpochError::Parse {
                            source: ParsingError::UnknownFormat,
                            details: "leap seconds file should have two columns exactly",
                        });
                    }

                    let timestamp_tai_s: u64 = match lexical_core::parse(data[0].as_bytes()) {
                        Ok(val) => val,
                        Err(_) => {
                            return Err(EpochError::Parse {
                                source: ParsingError::ValueError,
                                details: "first column value is not numeric",
                            })
                        }
                    };

                    let delta_at: u8 = match lexical_core::parse(data[1].as_bytes()) {
                        Ok(val) => val,
                        Err(_) => {
                            return Err(EpochError::Parse {
                                source: ParsingError::ValueError,
                                details: "second column value is not numeric",
                            })
                        }
                    };

                    me.data.push(LeapSecond {
                        timestamp_tai_s: (timestamp_tai_s as f64),
                        delta_at: (delta_at as f64),
                        announced_by_iers: true,
                    });
                }
            }
        }

        Ok(me)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl LeapSecondsFile {
    #[new]
    pub fn __new__(path: String) -> Result<Self, EpochError> {
        Self::from_path(&path)
    }

    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
    }
}

impl Iterator for LeapSecondsFile {
    type Item = LeapSecond;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_pos += 1;
        self.data.get(self.iter_pos - 1).copied()
    }
}

impl DoubleEndedIterator for LeapSecondsFile {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.iter_pos == self.data.len() {
            None
        } else {
            self.iter_pos += 1;
            self.data.get(self.data.len() - self.iter_pos).copied()
        }
    }
}

impl Index<usize> for LeapSecondsFile {
    type Output = LeapSecond;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl LeapSecondProvider for LeapSecondsFile {}

#[test]
fn leap_second_fetch() {
    use crate::leap_seconds::LatestLeapSeconds;
    use std::env;
    use std::path::PathBuf;
    let latest_leap_seconds = LatestLeapSeconds::default();

    // Load the IERS data
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("data")
        .join("leap-seconds.list");
    let leap_seconds = LeapSecondsFile::from_path(path.to_str().unwrap()).unwrap();

    assert_eq!(
        leap_seconds[0],
        LeapSecond::new(2_272_060_800.0, 10.0, true),
    );
    assert_eq!(
        leap_seconds[27],
        LeapSecond::new(3_692_217_600.0, 37.0, true)
    );

    for (lsi, leap_second) in leap_seconds.enumerate() {
        // The index offset is because the latest leap seconds include those not announced by the IERS, but the IERS file does not.
        assert_eq!(leap_second, latest_leap_seconds[lsi + 14]);
    }
}
