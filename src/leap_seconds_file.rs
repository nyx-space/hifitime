/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

#[cfg(feature = "python")]
use pyo3::prelude::*;

use std::{fs::File, io::Read};

use core::ops::Index;

use crate::{
    leap_seconds::{LeapSecond, LeapSecondProvider},
    Errors, ParsingErrors,
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
    pub fn from_path(path: &str) -> Result<Self, Errors> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(Errors::ParseError(ParsingErrors::IOError(e.kind()))),
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            return Err(Errors::ParseError(ParsingErrors::IOError(e.kind())));
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
                        return Err(Errors::ParseError(ParsingErrors::UnknownFormat));
                    }

                    let timestamp_tai_s: u64 = match lexical_core::parse(data[0].as_bytes()) {
                        Ok(val) => val,
                        Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
                    };

                    let delta_at: u8 = match lexical_core::parse(data[1].as_bytes()) {
                        Ok(val) => val,
                        Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
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
    pub fn __new__(path: String) -> Result<Self, Errors> {
        Self::from_path(&path)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
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
