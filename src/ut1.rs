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

use reqwest::{blocking::get, StatusCode};

use tabled::{Style, Table, Tabled};

use std::{fs::File, io::Read};

use core::fmt;
use core::ops::Index;

use crate::{Duration, Epoch, Errors, ParsingErrors, Unit};

#[derive(Copy, Clone, Debug, Default, Tabled)]
pub struct DeltaTaiUt1 {
    pub epoch: Epoch,
    pub delta_tai_minus_ut1: Duration,
}

#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone, Debug, Default)]
/// A structure storing all of the TAI-UT1 data
pub struct Ut1Provider {
    data: Vec<DeltaTaiUt1>,
    iter_pos: usize,
}

impl Ut1Provider {
    /// Build a UT1 provider by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.short> (short time scale UT1 data) and parsing it.
    pub fn download_short_from_jpl() -> Result<Self, Errors> {
        match get("https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.short") {
            Ok(resp) => {
                let eop_data = String::from_utf8(resp.bytes().unwrap().to_vec()).unwrap();
                Self::from_eop_data(eop_data)
            }
            Err(e) => Err(Errors::ParseError(ParsingErrors::DownloadError(
                e.status().unwrap_or(StatusCode::SEE_OTHER),
            ))),
        }
    }

    /// Builds a UT1 provider from the provided path to an EOP file.
    pub fn from_eop_file(path: &str) -> Result<Self, Errors> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(Errors::ParseError(ParsingErrors::IOError(e.kind()))),
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            return Err(Errors::ParseError(ParsingErrors::IOError(e.kind())));
        }

        Self::from_eop_data(contents)
    }

    /// Builds a UT1 provider from the provided EOP data
    pub fn from_eop_data(contents: String) -> Result<Self, Errors> {
        let mut me = Self::default();

        let mut ignore = true;
        for line in contents.lines() {
            if line == " EOP2=" {
                // Data will start after this line
                ignore = false;
                continue;
            } else if line == " $END" {
                // We've reached the end of the EOP data file.
                break;
            }
            if ignore {
                continue;
            }

            // We have data of interest!
            let data: Vec<&str> = line.split(',').collect();
            if data.len() < 4 {
                return Err(Errors::ParseError(ParsingErrors::UnknownFormat));
            }

            let mjd_tai_days: f64;
            match lexical_core::parse(data[0].trim().as_bytes()) {
                Ok(val) => mjd_tai_days = val,
                Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
            }

            let delta_ut1_ms: f64;
            match lexical_core::parse(data[3].trim().as_bytes()) {
                Ok(val) => delta_ut1_ms = val,
                Err(_) => return Err(Errors::ParseError(ParsingErrors::ValueError)),
            }

            me.data.push(DeltaTaiUt1 {
                epoch: Epoch::from_mjd_tai(mjd_tai_days),
                delta_tai_minus_ut1: delta_ut1_ms * Unit::Millisecond,
            });
        }

        Ok(me)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Ut1Provider {
    #[new]
    pub fn __new__() -> Result<Self, Errors> {
        Self::download_short_from_jpl()
    }

    fn __repr__(&self) -> String {
        format!("{self}")
    }
}

impl fmt::Display for Ut1Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new(&self.data);
        table.with(Style::rounded());
        write!(f, "{}", table)
    }
}

impl Iterator for Ut1Provider {
    type Item = DeltaTaiUt1;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_pos += 1;
        self.data.get(self.iter_pos - 1).copied()
    }
}

impl DoubleEndedIterator for Ut1Provider {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.iter_pos == self.data.len() {
            None
        } else {
            self.iter_pos += 1;
            self.data.get(self.data.len() - self.iter_pos).copied()
        }
    }
}

impl Index<usize> for Ut1Provider {
    type Output = DeltaTaiUt1;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}
