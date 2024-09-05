use pyo3_stub_gen::Result;

#[cfg(feature = "python")]
fn main() -> Result<()> {
    let stub = hifitime::python::stub_info()?;
    stub.generate()?;
    Ok(())
}

#[cfg(not(feature = "python"))]
fn main() -> Result<()> {
    Ok(())
}