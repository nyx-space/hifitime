/*
* Hifitime
* Copyright (C) 2017-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*
* Documentation: https://nyxspace.com/
*/

use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyTuple},
};

use crate::leap_seconds::{LatestLeapSeconds, LeapSecondsFile};
use crate::prelude::*;
use crate::ut1::Ut1Provider;
use crate::{MonthName, Polynomial, Weekday};

// Keep the module at the top
#[pymodule]
fn hifitime(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Epoch>()?;
    m.add_class::<TimeScale>()?;
    m.add_class::<TimeSeries>()?;
    m.add_class::<Duration>()?;
    m.add_class::<Unit>()?;
    m.add_class::<LatestLeapSeconds>()?;
    m.add_class::<LeapSecondsFile>()?;
    m.add_class::<Ut1Provider>()?;
    m.add_class::<MonthName>()?;
    m.add_class::<PyHifitimeError>()?;
    m.add_class::<PyDurationError>()?;
    m.add_class::<PyParsingError>()?;
    m.add_class::<Polynomial>()?;
    m.add_class::<Weekday>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", env!("CARGO_PKG_DESCRIPTION"))?;
    m.add("__author__", env!("CARGO_PKG_AUTHORS"))?;

    // Export constants
    m.add("JD_J1900", crate::JD_J1900)?;
    m.add("JD_J2000", crate::JD_J2000)?;
    m.add("MJD_J1900", crate::MJD_J1900)?;
    m.add("MJD_J2000", crate::MJD_J2000)?;
    m.add("ET_EPOCH_S", crate::ET_EPOCH_S)?;
    m.add("MJD_OFFSET", crate::MJD_OFFSET)?;
    m.add("DAYS_PER_YEAR", crate::DAYS_PER_YEAR)?;
    m.add("DAYS_PER_YEAR_NLD", crate::DAYS_PER_YEAR_NLD)?;
    m.add("DAYS_PER_CENTURY", crate::DAYS_PER_CENTURY)?;
    m.add("DAYS_IN_CENTURY", crate::DAYS_PER_CENTURY)?;
    m.add("DAYS_PER_WEEK", crate::DAYS_PER_WEEK)?;
    m.add("SECONDS_PER_MINUTE", crate::SECONDS_PER_MINUTE)?;
    m.add("SECONDS_PER_HOUR", crate::SECONDS_PER_HOUR)?;
    m.add("SECONDS_PER_DAY", crate::SECONDS_PER_DAY)?;
    m.add("SECONDS_PER_CENTURY", crate::SECONDS_PER_CENTURY)?;
    m.add("SECONDS_PER_YEAR", crate::SECONDS_PER_YEAR)?;
    m.add("SECONDS_PER_TROPICAL_YEAR", crate::SECONDS_PER_TROPICAL_YEAR)?;
    m.add("SECONDS_PER_SIDEREAL_YEAR", crate::SECONDS_PER_SIDEREAL_YEAR)?;

    m.add("NANOSECONDS_PER_MICROSECOND", crate::NANOSECONDS_PER_MICROSECOND)?;
    m.add("NANOSECONDS_PER_MILLISECOND", crate::NANOSECONDS_PER_MILLISECOND)?;
    m.add("NANOSECONDS_PER_SECOND", crate::NANOSECONDS_PER_SECOND)?;
    m.add("NANOSECONDS_PER_MINUTE", crate::NANOSECONDS_PER_MINUTE)?;
    m.add("NANOSECONDS_PER_HOUR", crate::NANOSECONDS_PER_HOUR)?;
    m.add("NANOSECONDS_PER_DAY", crate::NANOSECONDS_PER_DAY)?;
    m.add("NANOSECONDS_PER_CENTURY", crate::NANOSECONDS_PER_CENTURY)?;

    Ok(())
}

#[pyclass]
#[pyo3(name = "HifitimeError", extends = PyException)]
pub struct PyHifitimeError {}

#[pymethods]
impl PyHifitimeError {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: Bound<'_, PyTuple>, _kwargs: Option<Bound<'_, PyDict>>) -> Self {
        Self {}
    }
}

#[pyclass]
#[pyo3(name = "ParsingError", extends = PyException)]
pub struct PyParsingError {}

#[pymethods]
impl PyParsingError {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: Bound<'_, PyTuple>, _kwargs: Option<Bound<'_, PyDict>>) -> Self {
        Self {}
    }
}

#[pyclass]
#[pyo3(name = "DurationError", extends = PyException)]
pub struct PyDurationError {}

#[pymethods]
impl PyDurationError {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: Bound<'_, PyTuple>, _kwargs: Option<Bound<'_, PyDict>>) -> Self {
        Self {}
    }
}

// convert you library error into a PyErr using the custom exception type
impl From<HifitimeError> for PyErr {
    fn from(err: HifitimeError) -> Self {
        PyErr::new::<PyHifitimeError, _>(err.to_string())
    }
}

impl From<ParsingError> for PyErr {
    fn from(err: ParsingError) -> PyErr {
        PyErr::new::<PyParsingError, _>(err.to_string())
    }
}

impl From<DurationError> for PyErr {
    fn from(err: DurationError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}
