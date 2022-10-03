/*
 * Hifitime, part of the Nyx Space tools
 * Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Apache
 * v. 2.0. If a copy of the Apache License was not distributed with this
 * file, You can obtain one at https://www.apache.org/licenses/LICENSE-2.0.
 *
 * Documentation: https://nyxspace.com/
 */

use pyo3::{exceptions::PyException, prelude::*};

use crate::prelude::*;

impl std::convert::From<Errors> for PyErr {
    fn from(err: Errors) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

#[pymodule]
fn hifitime(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Epoch>()?;
    m.add_class::<TimeSystem>()?;
    m.add_class::<TimeSeries>()?;
    m.add_class::<Duration>()?;
    m.add_class::<Unit>()?;
    Ok(())
}
