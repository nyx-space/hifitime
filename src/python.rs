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
