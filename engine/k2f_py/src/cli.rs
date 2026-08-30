use pyo3::prelude::*;

#[pyfunction]
pub fn _cli_main() -> PyResult<()> {
    let args: Vec<String> = Python::with_gil(|py| py.import("sys")?.getattr("argv")?.extract())?;
    match k2f_cli::run_from(args) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
    }
}
