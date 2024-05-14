/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use pyo3::{
    exceptions::{PyBaseException, PyException},
    prelude::*,
    types::{PyDict, PyTuple},
};

use crate::leap_seconds::{LatestLeapSeconds, LeapSecondsFile};
use crate::prelude::*;
use crate::ut1::Ut1Provider;

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
    m.add_class::<PyEpochError>()?;
    m.add_class::<PyDurationError>()?;
    m.add_class::<PyParsingError>()?;
    Ok(())
}

#[pyclass]
#[pyo3(name = "EpochError", extends = PyBaseException)]
pub struct PyEpochError {}

#[pymethods]
impl PyEpochError {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: Bound<'_, PyTuple>, _kwargs: Option<Bound<'_, PyDict>>) -> Self {
        Self {}
    }
}

#[pyclass]
#[pyo3(name = "ParsingError", extends = PyBaseException)]
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
#[pyo3(name = "DurationError", extends = PyBaseException)]
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
impl From<EpochError> for PyErr {
    fn from(err: EpochError) -> Self {
        PyErr::new::<PyEpochError, _>(err.to_string())
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
