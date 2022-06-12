mod model;
mod read;
mod error;
use pyo3::prelude::*;

#[pyfunction]
fn read_xyz(filepath: &str) -> PyResult<String> {
    
    let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1).enable_all().build().unwrap();

    match runtime.block_on(read::read_xyz(filepath)) {
        Ok(_) => Ok("OK".to_string()),
        Err(e) => Err(pyo3::PyErr::from(e)),
    }

}

/// A Python module implemented in Rust.
#[pymodule]
fn sqlmd(_py: Python, m: &PyModule) -> PyResult<()> {
    let r = m.add_function(wrap_pyfunction!(read_xyz, m)?)?;
    Ok(())
}