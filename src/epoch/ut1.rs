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

use reqwest::{blocking::get, StatusCode};

use tabled::settings::Style;
use tabled::{Table, Tabled};

use std::{fs::File, io::Read};

use core::fmt;
use core::ops::Index;

use crate::{Duration, Epoch, EpochError, ParsingError, TimeScale, Unit};

impl Epoch {
    #[must_use]
    /// Initialize an Epoch from the provided UT1 duration since 1900 January 01 at midnight
    ///
    /// # Warning
    /// The time scale of this Epoch will be set to TAI! This is to ensure that no additional computations will change the duration since it's stored in TAI.
    /// However, this also means that calling `to_duration()` on this Epoch will return the TAI duration and not the UT1 duration!
    pub fn from_ut1_duration(duration: Duration, provider: Ut1Provider) -> Self {
        let mut e = Self::from_tai_duration(duration);
        // Compute the TAI to UT1 offset at this time.
        // We have the time in TAI. But we were given UT1.
        // The offset is provided as offset = TAI - UT1 <=> TAI = UT1 + offset
        e.duration += e.ut1_offset(provider).unwrap_or(Duration::ZERO);
        e.time_scale = TimeScale::TAI;
        e
    }

    /// Get the accumulated offset between this epoch and UT1, assuming that the provider includes all data.
    pub fn ut1_offset(&self, provider: Ut1Provider) -> Option<Duration> {
        for delta_tai_ut1 in provider.rev() {
            if self > &delta_tai_ut1.epoch {
                return Some(delta_tai_ut1.delta_tai_minus_ut1);
            }
        }
        None
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UT1
    pub fn to_ut1_duration(&self, provider: Ut1Provider) -> Duration {
        // TAI = UT1 + offset <=> UTC = TAI - offset
        self.to_tai_duration() - self.ut1_offset(provider).unwrap_or(Duration::ZERO)
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UT1
    pub fn to_ut1(&self, provider: Ut1Provider) -> Self {
        Self::from_tai_duration(self.to_ut1_duration(provider))
    }
}

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
    /// Builds a UT1 provided by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.short> (short time scale UT1 data) and parsing it.
    pub fn download_short_from_jpl() -> Result<Self, EpochError> {
        Self::download_from_jpl("latest_eop2.short")
    }

    /// Build a UT1 provider by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.long> (long time scale UT1 data) and parsing it.
    pub fn download_from_jpl(version: &str) -> Result<Self, EpochError> {
        match get(format!(
            "https://eop2-external.jpl.nasa.gov/eop2/{}",
            version
        )) {
            Ok(resp) => {
                let eop_data = String::from_utf8(resp.bytes().unwrap().to_vec()).unwrap();
                Self::from_eop_data(eop_data)
            }
            Err(e) => Err(EpochError::Parse {
                source: ParsingError::DownloadError {
                    code: e.status().unwrap_or(StatusCode::SEE_OTHER),
                },
                details: "when downloading EOP2 file from JPL",
            }),
        }
    }

    /// Builds a UT1 provider from the provided path to an EOP file.
    pub fn from_eop_file(path: &str) -> Result<Self, EpochError> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(EpochError::Parse {
                    source: ParsingError::InOut { err: e.kind() },
                    details: "when opening EOP file",
                })
            }
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            return Err(EpochError::Parse {
                source: ParsingError::InOut { err: e.kind() },
                details: "when reading EOP file",
            });
        }

        Self::from_eop_data(contents)
    }

    /// Builds a UT1 provider from the provided EOP data
    pub fn from_eop_data(contents: String) -> Result<Self, EpochError> {
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
                return Err(EpochError::Parse {
                    source: ParsingError::UnknownFormat,
                    details: "expected EOP line to contain 4 comma-separated columns",
                });
            }

            let mjd_tai_days: f64 = match lexical_core::parse(data[0].trim().as_bytes()) {
                Ok(val) => val,
                Err(err) => {
                    return Err(EpochError::Parse {
                        source: ParsingError::Lexical { err },
                        details: "when parsing MJD TAI days (zeroth column)",
                    })
                }
            };

            let delta_ut1_ms: f64;
            match lexical_core::parse(data[3].trim().as_bytes()) {
                Ok(val) => delta_ut1_ms = val,
                Err(err) => {
                    return Err(EpochError::Parse {
                        source: ParsingError::Lexical { err },
                        details: "when parsing ΔUT1 in ms (last column)",
                    })
                }
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
    pub fn __new__() -> Result<Self, EpochError> {
        Self::download_short_from_jpl()
    }

    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
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
