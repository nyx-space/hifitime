/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

#[cfg(feature = "python")]
use pyo3::prelude::*;

use ureq::get;
use ureq::Error;

use tabled::settings::Style;
use tabled::{Table, Tabled};

use std::{fs::File, io::Read};

use core::fmt;
use core::ops::Index;

use crate::{Duration, Epoch, HifitimeError, ParsingError, TimeScale, Unit};

impl Epoch {
    #[must_use]
    /// Initialize an Epoch from the provided UT1 duration since 1900 January 01 at midnight
    ///
    /// # Warning
    /// The time scale of this Epoch will be set to TAI! This is to ensure that no additional computations will change the duration since it's stored in TAI.
    /// However, this also means that calling `to_duration()` on this Epoch will return the TAI duration and not the UT1 duration!
    pub fn from_ut1_duration(duration: Duration, provider: &Ut1Provider) -> Self {
        let mut e = Self::from_tai_duration(duration);
        // Compute the TAI to UT1 offset at this time.
        // We have the time in TAI. But we were given UT1.
        // The offset is provided as offset = TAI - UT1 <=> TAI = UT1 + offset
        e.duration += e.ut1_offset(provider).unwrap_or(Duration::ZERO);
        e.time_scale = TimeScale::TAI;
        e
    }

    /// Get the accumulated offset between this epoch and UT1.
    /// Assumes the provider's records are sorted by ascending epoch (enforced in `from_eop_data`).
    ///
    /// Arguments
    /// -----------------
    /// * `provider`: Borrowed UT1 data source.
    ///
    /// Return
    /// ----------
    /// * `Some(Duration)` for the last record with `record.epoch <= self`, otherwise `None`.
    pub fn ut1_offset(&self, provider: &Ut1Provider) -> Option<Duration> {
        let s = provider.as_slice();

        // Fast-path: very common case — query is after the latest record.
        if let Some(last) = s.last() {
            if *self >= last.epoch {
                return Some(last.delta_tai_minus_ut1);
            }
        }

        // Find the index of the first element with epoch > self (monotonic predicate)
        let idx = s.partition_point(|r| r.epoch <= *self);

        // Candidate is the previous element if any exists
        let rec = s.get(idx.checked_sub(1)?)?;
        Some(rec.delta_tai_minus_ut1)
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UT1
    pub fn to_ut1_duration(&self, provider: &Ut1Provider) -> Duration {
        // TAI = UT1 + offset <=> UTC = TAI - offset
        self.to_tai_duration() - self.ut1_offset(provider).unwrap_or(Duration::ZERO)
    }

    #[must_use]
    /// Returns this time in a Duration past J1900 counted in UT1
    pub fn to_ut1(&self, provider: &Ut1Provider) -> Self {
        Self::from_tai_duration(self.to_ut1_duration(provider))
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Epoch {

    #[staticmethod]
    #[pyo3(name = "from_ut1_duration")]
    pub fn py_from_ut1_duration(duration: Duration, provider: PyRef<Ut1Provider>) -> PyResult<Self> {
        Ok(Epoch::from_ut1_duration(duration, &*provider))
    }

    #[pyo3(name = "ut1_offset")]
    pub fn py_ut1_offset(&self, provider: PyRef<Ut1Provider>) -> Option<Duration> {
        self.ut1_offset(&*provider)
    }

    #[pyo3(name = "to_ut1_duration")]
    pub fn py_to_ut1_duration(&self, provider: PyRef<Ut1Provider>) -> Duration {
        self.to_ut1_duration(&*provider)
    }

    #[pyo3(name = "to_ut1")]
    pub fn py_to_ut1(&self, provider: PyRef<Ut1Provider>) -> Self {
        self.to_ut1(&*provider)
    }
}

#[cfg_attr(kani, derive(kani::Arbitrary))]
#[cfg_attr(feature = "python",
    pyo3::pyclass(module = "hifitime", name = "DeltaTaiUt1", get_all)
)]
#[cfg_attr(not(feature = "python"), derive(Copy))]  // Copy only when NOT exposing to Python
#[derive(Clone, Debug, Default, Tabled)]
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
    /// Read-only view of the underlying UT1 records.
    ///
    /// Arguments
    /// -----------------
    /// * None.
    ///
    /// Return
    /// ----------
    /// * A slice `&[DeltaTaiUt1]` over all records.
    pub fn as_slice(&self) -> &[DeltaTaiUt1] {
        &self.data
    }

    /// Builds a UT1 provided by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.short> (short time scale UT1 data) and parsing it.
    pub fn download_short_from_jpl() -> Result<Self, HifitimeError> {
        Self::download_from_jpl("latest_eop2.short")
    }

    /// Build a UT1 provider by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.long> (long time scale UT1 data) and parsing it.
    pub fn download_from_jpl(version: &str) -> Result<Self, HifitimeError> {
        let url = format!("https://eop2-external.jpl.nasa.gov/eop2/{}", version);

        // Download the file
        match get(url).call() {
            Ok(resp) => {
                let Ok(jpl_response) = resp.into_body().read_to_string() else {
                    return Err(HifitimeError::Parse {
                        source: ParsingError::UnknownFormat,
                        details: "when reading EOP2 file from JPL",
                    });
                };
                Self::from_eop_data(jpl_response)
            }
            Err(Error::StatusCode(code)) => Err(HifitimeError::Parse {
                source: ParsingError::DownloadError { code: code },
                details: "when downloading EOP2 file from JPL",
            }),
            Err(_) => Err(HifitimeError::Parse {
                source: ParsingError::UnknownFormat,
                details: "when downloading EOP2 file from JPL",
            }),
        }
    }

    /// Builds a UT1 provider from the provided path to an EOP file.
    pub fn from_eop_file(path: &str) -> Result<Self, HifitimeError> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(HifitimeError::Parse {
                    source: ParsingError::InOut { err: e.kind() },
                    details: "when opening EOP file",
                })
            }
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            return Err(HifitimeError::Parse {
                source: ParsingError::InOut { err: e.kind() },
                details: "when reading EOP file",
            });
        }

        Self::from_eop_data(contents)
    }

    /// Builds a UT1 provider from the provided EOP data.
    /// Single-pass, no per-line allocation:
    /// - Use `split(',')` and take exactly columns 0 and 3 (no `collect()`).
    /// - Track sortedness and only sort at the end if needed.
    /// - Trim CR/LF and ignore empty lines.
    ///
    /// Arguments
    /// -----------------
    /// * `contents`: The full EOP2 text payload from JPL.
    ///
    /// Return
    /// ----------
    /// * `Ok(Self)` with records sorted by ascending epoch.
    /// * `Err(HifitimeError)` on malformed lines.
    ///
    /// See also
    /// ------------
    /// * [`Ut1Provider::from_eop_file`] – File-based variant calling this parser.
    pub fn from_eop_data(contents: String) -> Result<Self, HifitimeError> {
        let mut me = Self::default();
        // Heuristic to reduce Vec reallocations
        me.data.reserve(contents.len() / 48);

        let mut in_data = false;
        let mut prev_epoch: Option<Epoch> = None;
        let mut already_sorted = true;

        for raw in contents.lines() {
            // Header section control
            if !in_data {
                if raw == " EOP2=" || raw == "EOP2=" {
                    in_data = true;
                }
                continue;
            }
            if raw == " $END" || raw == "$END" {
                break;
            }
            if raw.is_empty() {
                continue;
            }

            // Extract exactly columns 0 and 3 (others ignored)
            let mut cols = raw.split(',');
            let mjd_col = cols.next().ok_or_else(|| HifitimeError::Parse {
                source: ParsingError::UnknownFormat,
                details: "missing MJD column (0)",
            })?;
            let delta_col = cols.nth(2).ok_or_else(|| HifitimeError::Parse {
                source: ParsingError::UnknownFormat,
                details: "missing ΔUT1 column (3)",
            })?;

            // Parse numeric fields
            let mjd_tai_days: f64 =
                lexical_core::parse(mjd_col.trim().as_bytes()).map_err(|err| {
                    HifitimeError::Parse {
                        source: ParsingError::Lexical { err },
                        details: "when parsing MJD TAI days (column 0)",
                    }
                })?;

            let delta_ut1_ms: f64 =
                lexical_core::parse(delta_col.trim().as_bytes()).map_err(|err| {
                    HifitimeError::Parse {
                        source: ParsingError::Lexical { err },
                        details: "when parsing ΔUT1 in ms (column 3)",
                    }
                })?;

            let epoch = Epoch::from_mjd_tai(mjd_tai_days);
            if let Some(prev) = prev_epoch {
                if epoch < prev {
                    already_sorted = false;
                }
            }
            prev_epoch = Some(epoch);

            me.data.push(DeltaTaiUt1 {
                epoch,
                delta_tai_minus_ut1: delta_ut1_ms * Unit::Millisecond,
            });
        }

        if !already_sorted {
            me.data.sort_unstable_by(|a, b| {
                a.epoch
                    .partial_cmp(&b.epoch)
                    .expect("Epoch must be orderable (no NaN)")
            });
        }

        Ok(me)
    }
}


#[cfg_attr(feature = "python", pymethods)]
impl Ut1Provider {
    // For Python, return a list of owned objects.
    // Option A: return Python class instances
    pub fn as_list(&self, py: Python<'_>) -> PyResult<Vec<Py<DeltaTaiUt1>>> {
        self.data
            .iter()
            .cloned()                      // Clone each record (since not Copy)
            .map(|rec| Py::new(py, rec))   // Allocate a Py<DeltaTaiUt1>
            .collect()
    }

    #[staticmethod]
    #[pyo3(name="from_eop_file")]
    /// Builds a UT1 provider from the provided path to an EOP file.
    pub fn py_from_eop_file(path: &str) -> Result<Self, HifitimeError> {
        Ut1Provider::from_eop_file(path)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Ut1Provider {
    #[new]
    pub fn __new__() -> Result<Self, HifitimeError> {
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
        self.data.get(self.iter_pos - 1).cloned()
    }
}

impl DoubleEndedIterator for Ut1Provider {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.iter_pos == self.data.len() {
            None
        } else {
            self.iter_pos += 1;
            self.data.get(self.data.len() - self.iter_pos).cloned()
        }
    }
}

impl Index<usize> for Ut1Provider {
    type Output = DeltaTaiUt1;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl<'a> IntoIterator for &'a Ut1Provider {
    type Item = &'a DeltaTaiUt1;
    type IntoIter = std::slice::Iter<'a, DeltaTaiUt1>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

#[cfg(kani)]
mod kani_harnesses {
    use super::*;
    #[kani::proof]
    fn kani_harness_Ut1Provider_download_short_from_jpl() {
        Ut1Provider::download_short_from_jpl();
    }
}
