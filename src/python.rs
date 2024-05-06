/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. https://github.com/nyx-space/hifitime/graphs/contributors)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use pyo3::{exceptions::PyException, prelude::*};

use crate::leap_seconds::{LatestLeapSeconds, LeapSecondsFile};
use crate::prelude::*;
use crate::ut1::Ut1Provider;

#[derive(Debug)]
#[repr(C)]
#[cfg_attr(feature = "python", pyclass)]
pub enum Exceptions {
    EpochError { err: String },
    ParsingError { err: String },
    DurationError { err: String },
}

impl fmt::Display for Exceptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EpochError { err } => write!(f, "{err}"),
            Self::DurationError { err } => write!(f, "{err}"),
            Self::ParsingError { err } => write!(f, "{err}"),
        }
    }
}

impl From<Exceptions> for PyErr {
    fn from(err: Exceptions) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<EpochError> for PyErr {
    fn from(err: EpochError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<ParsingError> for PyErr {
    fn from(err: ParsingError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<DurationError> for PyErr {
    fn from(err: DurationError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl From<EpochError> for Exceptions {
    fn from(err: EpochError) -> Exceptions {
        Exceptions::EpochError {
            err: err.to_string(),
        }
    }
}

impl From<ParsingError> for Exceptions {
    fn from(err: ParsingError) -> Exceptions {
        Exceptions::ParsingError {
            err: err.to_string(),
        }
    }
}

impl From<DurationError> for Exceptions {
    fn from(err: DurationError) -> Exceptions {
        Exceptions::DurationError {
            err: err.to_string(),
        }
    }
}

#[pymodule]
fn hifitime(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Epoch>()?;
    m.add_class::<Exceptions>()?;
    m.add_class::<TimeScale>()?;
    m.add_class::<TimeSeries>()?;
    m.add_class::<Duration>()?;
    m.add_class::<Unit>()?;
    m.add_class::<LatestLeapSeconds>()?;
    m.add_class::<LeapSecondsFile>()?;
    m.add_class::<Ut1Provider>()?;
    Ok(())
}
