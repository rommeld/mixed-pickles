use pyo3::prelude::*;

#[pyclass]
struct Commit(String);

#[pymethods]
impl Commit {
    #[new]
    fn new(hash: String) -> Self {
        Commit(hash)
    }

    #[getter]
    fn hash(&self) -> &str {
        &self.0
    }
}

#[pymodule]
fn mixed_pickles(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Commit>()?;
    Ok(())
}
