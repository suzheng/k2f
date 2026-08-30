use k2f_sdk::SYSTEM_PROMPT;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::Path;

mod cli;
mod editor;

pub(crate) fn py_err(e: k2f_sdk::AgentError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

#[pyfunction]
fn system_prompt() -> &'static str {
    SYSTEM_PROMPT
}

#[pyfunction]
fn official_templates() -> Vec<&'static str> {
    k2f_sdk::OFFICIAL_IDS.to_vec()
}

#[pyfunction]
fn resolve_template(template: &str) -> PyResult<String> {
    Ok(k2f_sdk::resolve(template)
        .map_err(py_err)?
        .display()
        .to_string())
}

#[pyfunction]
fn copy_template(template: &str, dest: &str) -> PyResult<()> {
    k2f_sdk::copy_to(template, Path::new(dest)).map_err(py_err)
}

#[pyfunction]
fn generate_signing_key() -> PyResult<(String, String, String)> {
    let key = k2f_sdk::generate_key().map_err(py_err)?;
    Ok((key.secret_hex, key.public_hex, key.fingerprint))
}

#[pyfunction]
#[pyo3(signature = (package_bytes, secret_hex, signed_by=None, signed_at=None))]
fn sign<'py>(
    py: Python<'py>,
    package_bytes: &[u8],
    secret_hex: &str,
    signed_by: Option<&str>,
    signed_at: Option<i64>,
) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = k2f_sdk::sign(package_bytes, secret_hex, signed_by, signed_at).map_err(py_err)?;
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
#[pyo3(signature = (md, title="Document", template="report"))]
fn markdown_to_k2f<'py>(
    py: Python<'py>,
    md: &str,
    title: &str,
    template: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let opts = k2f_sdk::MarkdownOptions::new(title, template).map_err(py_err)?;
    let result = k2f_sdk::markdown_to_k2f(md, opts).map_err(py_err)?;
    Ok(PyBytes::new(py, &result.bytes))
}

#[pyfunction]
fn k2f_to_markdown(package_bytes: &[u8]) -> PyResult<String> {
    k2f_sdk::k2f_to_markdown(package_bytes).map_err(py_err)
}

#[pyfunction]
fn format_schemas() -> std::collections::BTreeMap<String, String> {
    k2f_sdk::format_schemas()
}

#[pymodule]
fn k2f(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<editor::Editor>()?;
    m.add_function(wrap_pyfunction!(system_prompt, m)?)?;
    m.add_function(wrap_pyfunction!(format_schemas, m)?)?;
    m.add_function(wrap_pyfunction!(official_templates, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_template, m)?)?;
    m.add_function(wrap_pyfunction!(copy_template, m)?)?;
    m.add_function(wrap_pyfunction!(generate_signing_key, m)?)?;
    m.add_function(wrap_pyfunction!(sign, m)?)?;
    m.add_function(wrap_pyfunction!(markdown_to_k2f, m)?)?;
    m.add_function(wrap_pyfunction!(k2f_to_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(cli::_cli_main, m)?)?;
    Ok(())
}
