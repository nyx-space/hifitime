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
#[cfg(feature = "python")]
use pyo3::types::PyType;

use ureq::get;
use ureq::Error;

use log::{debug, info, warn};

use tabled::settings::Style;
use tabled::{Table, Tabled};

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use core::fmt;
use core::ops::Index;

use crate::{Duration, Epoch, HifitimeError, ParsingError, TimeScale, Unit};

/// Default maximum age of the cached EOP2 file: one day.
///
/// JPL publishes a new EOP2 file once per day, so a cached file younger than one day is reused
/// instead of being downloaded again. See [`Ut1Provider::download_from_jpl_cached`].
pub const DEFAULT_UT1_CACHE_MAX_AGE: Duration =
    Duration::from_total_nanoseconds(86_400 * 1_000_000_000);

/// Where the data of a [`Ut1Provider`] comes from.
///
/// This is returned by [`Ut1Provider::source`] so that callers may log or inspect the origin of the
/// UT1 data in use, for instance to detect that a stale cache is being used because the JPL archive
/// could not be reached.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Ut1DataSource {
    /// The data was parsed from a payload or a file provided by the caller.
    #[default]
    Provided,
    /// The data was read from the local cache, which was last written `age` ago.
    Cache {
        /// Time elapsed since the cache file was written.
        age: Duration,
    },
    /// The data was downloaded from the JPL EOP2 archive.
    Download,
}

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

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Epoch {
    #[classmethod]
    #[pyo3(name = "from_ut1_duration")]
    /// Initialize a new Epoch from a duration in UT1
    ///
    /// :type duration: Duration
    /// :type provider: Ut1Provider
    /// :rtype: Epoch
    pub fn py_from_ut1_duration(
        _cls: &Bound<'_, PyType>,
        duration: Duration,
        provider: PyRef<Ut1Provider>,
    ) -> PyResult<Self> {
        Ok(Epoch::from_ut1_duration(duration, &provider))
    }

    /// Get the accumulated offset between this epoch and UT1.
    ///
    /// :type provider: Ut1Provider
    /// :rtype: Duration
    #[pyo3(name = "ut1_offset")]
    pub fn py_ut1_offset(&self, provider: PyRef<Ut1Provider>) -> Option<Duration> {
        self.ut1_offset(&provider)
    }

    /// Returns this time in a Duration past J1900 counted in UT1
    ///
    /// :type provider: Ut1Provider
    /// :rtype: Duration
    #[pyo3(name = "to_ut1_duration")]
    pub fn py_to_ut1_duration(&self, provider: PyRef<Ut1Provider>) -> Duration {
        self.to_ut1_duration(&provider)
    }

    /// Convert this epoch to Ut1
    ///
    /// :type provider: Ut1Provider
    /// :rtype: Epoch
    #[pyo3(name = "to_ut1")]
    pub fn py_to_ut1(&self, provider: PyRef<Ut1Provider>) -> Self {
        self.to_ut1(&provider)
    }
}

#[cfg_attr(kani, derive(kani::Arbitrary))]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "hifitime", name = "DeltaTaiUt1", get_all, set_all)
)]
#[derive(Copy, Clone, Debug, Default, Tabled)]
pub struct DeltaTaiUt1 {
    pub epoch: Epoch,
    pub delta_tai_minus_ut1: Duration,
}

#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(get_all))]
#[derive(Clone, Debug, Default)]
/// A structure storing all of the TAI-UT1 data
pub struct Ut1Provider {
    /// vector of Delta TAI-UT1 values
    /// :rtype: list
    data: Vec<DeltaTaiUt1>,
    /// current position of the iterator
    /// :rtype: int
    iter_pos: usize,
    /// where the data of this provider comes from
    /// :rtype: str
    source: Ut1DataSource,
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

    /// Where the data of this provider comes from.
    ///
    /// This is [`Ut1DataSource::Provided`] unless the provider was built by one of the caching
    /// constructors, in which case it tells whether the data was downloaded or read from the cache,
    /// and how old the cached data was.
    pub fn source(&self) -> Ut1DataSource {
        self.source
    }

    /// Default path of the cached EOP2 file for this version, inside the user's cache directory.
    ///
    /// The directory is `<cache dir>/hifitime/ut1`, where `<cache dir>` is `%LOCALAPPDATA%` on
    /// Windows, `~/Library/Caches` on macOS, and `$XDG_CACHE_HOME` (or `~/.cache`) elsewhere. Set
    /// `HIFITIME_UT1_CACHE_DIR` to override it. Returns `None` if no cache directory can be
    /// determined, in which case the caching constructors simply download the file.
    ///
    /// The version string is sanitized so that versions containing slashes, such as
    /// `2022/221222_190002-marge_eop2.short`, map to a single file name.
    pub fn cache_path(version: &str) -> Option<PathBuf> {
        cache_dir().map(|dir| dir.join(sanitize_version(version)))
    }

    /// Builds a UT1 provided by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.short> (short time scale UT1 data) and parsing it.
    ///
    /// This always hits the network: see [`Self::download_short_from_jpl_cached`] to reuse a local
    /// copy of the file.
    pub fn download_short_from_jpl() -> Result<Self, HifitimeError> {
        Self::download_from_jpl("latest_eop2.short")
    }

    /// Same as [`Self::download_short_from_jpl`], but reusing the local copy of the file while it is
    /// younger than [`DEFAULT_UT1_CACHE_MAX_AGE`], and falling back to it if the download fails.
    pub fn download_short_from_jpl_cached() -> Result<Self, HifitimeError> {
        Self::download_from_jpl_cached("latest_eop2.short", DEFAULT_UT1_CACHE_MAX_AGE)
    }

    /// Build a UT1 provider by downloading the data from <https://eop2-external.jpl.nasa.gov/eop2/latest_eop2.long> (long time scale UT1 data) and parsing it.
    ///
    /// This always hits the network: see [`Self::download_from_jpl_cached`] to reuse a local copy of
    /// the file.
    pub fn download_from_jpl(version: &str) -> Result<Self, HifitimeError> {
        let mut provider = Self::from_eop_data(Self::download_payload(version)?)?;
        provider.source = Ut1DataSource::Download;
        Ok(provider)
    }

    /// Build a UT1 provider from the JPL EOP2 archive, caching the downloaded file on disk.
    ///
    /// The cache policy is the one agreed in <https://github.com/nyx-space/hifitime/issues/425>:
    ///
    /// 1. If the cached file exists and is younger than `max_cache_age`, it is parsed and no request
    ///    is made to JPL.
    /// 2. Otherwise the file is downloaded from JPL and written to the cache.
    /// 3. If that download fails but a cached file exists, whatever its age, the cached file is used
    ///    and a warning reports the download failure and the age of the cached data.
    /// 4. If the download fails and there is no cached file, the download error is returned.
    ///
    /// The cache file is written to the path returned by [`Self::cache_path`]; if no cache directory
    /// can be determined, this behaves like [`Self::download_from_jpl`].
    ///
    /// Use [`Self::source`] to know which of the cached data or the downloaded data was used.
    pub fn download_from_jpl_cached(
        version: &str,
        max_cache_age: Duration,
    ) -> Result<Self, HifitimeError> {
        match Self::cache_path(version) {
            Some(cache_path) => {
                Self::download_from_jpl_cached_at(version, &cache_path, max_cache_age)
            }
            None => {
                debug!(
                    "no cache directory found for UT1 data, downloading {version} from JPL on every call"
                );
                Self::download_from_jpl(version)
            }
        }
    }

    /// Same as [`Self::download_from_jpl_cached`] but with an explicit path for the cache file.
    ///
    /// This is useful for applications that want to choose where the cache lives, and for tests.
    pub fn download_from_jpl_cached_at(
        version: &str,
        cache_path: &Path,
        max_cache_age: Duration,
    ) -> Result<Self, HifitimeError> {
        let cached_age = cached_age(cache_path);

        if let Some(age) = cached_age {
            if age <= max_cache_age {
                match Self::from_eop_file_path(cache_path) {
                    Ok(mut provider) => {
                        info!(
                            "using cached UT1 data from {} (age {}, maximum cache age {})",
                            cache_path.display(),
                            fmt_age(age),
                            fmt_age(max_cache_age)
                        );
                        provider.source = Ut1DataSource::Cache { age };
                        return Ok(provider);
                    }
                    Err(err) => warn!(
                        "cached UT1 file {} could not be parsed ({}), downloading it again",
                        cache_path.display(),
                        err
                    ),
                }
            } else {
                debug!(
                    "cached UT1 file {} is older than the maximum cache age (age {}, maximum {}), downloading it again",
                    cache_path.display(),
                    fmt_age(age),
                    fmt_age(max_cache_age)
                );
            }
        }

        match Self::download_payload(version) {
            Ok(payload) => {
                if let Err(err) = write_cache(cache_path, &payload) {
                    warn!(
                        "downloaded UT1 data from JPL but could not cache it at {} ({}), the next call will download it again",
                        cache_path.display(),
                        err
                    );
                }
                let mut provider = Self::from_eop_data(payload)?;
                provider.source = Ut1DataSource::Download;
                Ok(provider)
            }
            Err(err) => match cached_age {
                // The download failed but a cached file exists: keep going with it, as agreed in
                // https://github.com/nyx-space/hifitime/issues/425.
                Some(age) => {
                    warn!(
                        "could not download UT1 data from JPL ({}), falling back to the cached data of {} (age {})",
                        err,
                        cache_path.display(),
                        fmt_age(age)
                    );
                    let mut provider = Self::from_eop_file_path(cache_path)?;
                    provider.source = Ut1DataSource::Cache { age };
                    Ok(provider)
                }
                None => Err(err),
            },
        }
    }

    /// Download the raw EOP2 payload for this version from JPL.
    fn download_payload(version: &str) -> Result<String, HifitimeError> {
        let url = format!("https://eop2-external.jpl.nasa.gov/eop2/{}", version);

        match get(&url).call() {
            Ok(resp) => match resp.into_body().read_to_string() {
                Ok(jpl_response) => Ok(jpl_response),
                Err(_) => Err(HifitimeError::Parse {
                    source: ParsingError::UnknownFormat,
                    details: "when reading EOP2 file from JPL",
                }),
            },
            Err(Error::StatusCode(code)) => Err(HifitimeError::Parse {
                source: ParsingError::DownloadError { code },
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
        Self::from_eop_file_path(Path::new(path))
    }

    /// Builds a UT1 provider from the provided path to an EOP file.
    fn from_eop_file_path(path: &Path) -> Result<Self, HifitimeError> {
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
            let mjd_col = cols.next().ok_or(HifitimeError::Parse {
                source: ParsingError::UnknownFormat,
                details: "missing MJD column (0)",
            })?;
            let delta_col = cols.nth(2).ok_or(HifitimeError::Parse {
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

    /// Returns the list of Delta TAI-UT1 values
    /// :rtype: list
    pub fn as_list(&self, py: Python<'_>) -> PyResult<Vec<Py<DeltaTaiUt1>>> {
        self.data.iter().map(|rec| Py::new(py, *rec)).collect()
    }

    #[classmethod]
    #[pyo3(name = "from_eop_file")]
    /// Builds a UT1 provider from the provided path to an EOP file.
    /// :type path: str
    /// :rtype: Ut1Provider
    pub fn py_from_eop_file(_cls: &Bound<'_, PyType>, path: &str) -> Result<Self, HifitimeError> {
        Ut1Provider::from_eop_file(path)
    }

    #[classmethod]
    #[pyo3(name = "from_jpl_cached")]
    #[pyo3(signature = (max_cache_age_seconds = 86_400.0))]
    /// Builds a UT1 provider from the JPL EOP2 archive, reusing a cached copy of the file while it is
    /// younger than `max_cache_age_seconds` and falling back to it if the download fails.
    /// :type max_cache_age_seconds: float
    /// :rtype: Ut1Provider
    pub fn py_from_jpl_cached(
        _cls: &Bound<'_, PyType>,
        max_cache_age_seconds: f64,
    ) -> Result<Self, HifitimeError> {
        Ut1Provider::download_from_jpl_cached(
            "latest_eop2.short",
            max_cache_age_seconds * Unit::Second,
        )
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

impl<'a> IntoIterator for &'a Ut1Provider {
    type Item = &'a DeltaTaiUt1;
    type IntoIter = std::slice::Iter<'a, DeltaTaiUt1>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl fmt::Display for Ut1DataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provided => write!(f, "data provided by the caller"),
            Self::Cache { age } => write!(f, "cached data of age {}", fmt_age(*age)),
            Self::Download => write!(f, "data downloaded from JPL"),
        }
    }
}

#[cfg(feature = "python")]
impl<'py> pyo3::IntoPyObject<'py> for Ut1DataSource {
    type Target = pyo3::types::PyString;
    type Output = pyo3::Bound<'py, pyo3::types::PyString>;
    type Error = core::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(pyo3::types::PyString::new(py, &self.to_string()))
    }
}

/// Path of the directory holding the cached UT1 files, if one can be determined.
///
/// `HIFITIME_UT1_CACHE_DIR` takes precedence over the platform default.
fn cache_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("HIFITIME_UT1_CACHE_DIR") {
        return Some(PathBuf::from(dir));
    }

    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);

    #[cfg(target_os = "macos")]
    let base =
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library").join("Caches"));

    #[cfg(all(not(windows), not(target_os = "macos")))]
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")));

    base.map(|base| base.join("hifitime").join("ut1"))
}

/// Turn a JPL version string into a file name: `2022/221222_190002-marge_eop2.short` becomes
/// `2022_221222_190002-marge_eop2.short`.
fn sanitize_version(version: &str) -> String {
    version
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' => '_',
            _ => c,
        })
        .collect()
}

/// Age of the cache file, i.e. how long ago it was written.
///
/// Returns `None` if the file does not exist or if its modification time cannot be read, or if the
/// system clock is set before that modification time.
fn cached_age(cache_path: &Path) -> Option<Duration> {
    let modified = std::fs::metadata(cache_path).ok()?.modified().ok()?;
    let elapsed = SystemTime::now().duration_since(modified).ok()?;
    Some(Duration::from_total_nanoseconds(
        i128::try_from(elapsed.as_nanos()).ok()?,
    ))
}

/// Write the downloaded payload to the cache, creating the parent directories as needed.
fn write_cache(cache_path: &Path, payload: &str) -> std::io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Write to a temporary file then rename it, so that a crash or a concurrent reader never sees a
    // half-written cache file.
    let tmp_path = cache_path.with_extension("tmp");
    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(payload.as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp_path, cache_path)
}

/// Format an age for the logs, in hours when it makes sense.
fn fmt_age(age: Duration) -> String {
    let hours = age.to_seconds() / 3_600.0;
    if hours >= 1.0 {
        format!("{hours:.1} h")
    } else {
        format!("{:.0} min", hours * 60.0)
    }
}
