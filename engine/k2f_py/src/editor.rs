use crate::py_err;
use k2f_sdk::Editor as Inner;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::Path;

#[pyclass]
pub struct Editor {
    inner: Inner,
}

#[pymethods]
impl Editor {
    #[staticmethod]
    fn open_bytes(bytes: &[u8]) -> PyResult<Self> {
        Ok(Self {
            inner: Inner::open(bytes).map_err(py_err)?,
        })
    }

    #[staticmethod]
    fn open_dir(path: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Inner::open_dir(Path::new(path)).map_err(py_err)?,
        })
    }

    fn outline(&self) -> PyResult<String> {
        self.inner.outline_json().map_err(py_err)
    }

    fn diff(&self) -> PyResult<String> {
        self.inner.diff_json().map_err(py_err)
    }

    fn get_node(&self, id: &str) -> PyResult<String> {
        self.inner.get_node_json(id).map_err(py_err)
    }

    fn selection(&self, id: &str) -> PyResult<String> {
        self.inner.selection_json(id).map_err(py_err)
    }

    fn clipboard(&self, id: &str) -> PyResult<String> {
        self.inner.clipboard_json(id).map_err(py_err)
    }

    fn replace_text(&mut self, id: &str, text: &str) -> PyResult<()> {
        self.inner.replace_text(id, text).map_err(py_err)
    }

    #[pyo3(signature = (id, role, variant=None))]
    fn set_role(&mut self, id: &str, role: &str, variant: Option<&str>) -> PyResult<()> {
        self.inner.set_role(id, role, variant).map_err(py_err)
    }

    fn insert_node(&mut self, parent_id: &str, index: usize, node_json: &str) -> PyResult<()> {
        self.inner
            .insert_node(parent_id, index, node_json)
            .map_err(py_err)
    }

    fn delete_node(&mut self, id: &str) -> PyResult<()> {
        self.inner.delete_node(id).map_err(py_err)
    }

    fn set_running_header(&mut self, text: &str) -> PyResult<()> {
        self.inner.set_running_header(text).map_err(py_err)
    }

    fn set_running_footer(&mut self, text: &str) -> PyResult<()> {
        self.inner.set_running_footer(text).map_err(py_err)
    }

    fn set_generated_by(&mut self, id: &str) {
        self.inner.set_generated_by(id);
    }

    fn search(&self, query: &str) -> Vec<String> {
        self.inner.search(query)
    }

    fn suggest(&mut self, id: &str, text: &str) -> PyResult<()> {
        self.inner.suggest(id, text).map_err(py_err)
    }

    fn accept_suggestion(&mut self, id: &str) -> PyResult<()> {
        self.inner.accept_suggestion(id).map_err(py_err)
    }

    fn reject_suggestion(&mut self, id: &str) -> PyResult<()> {
        self.inner.reject_suggestion(id).map_err(py_err)
    }

    fn save_bytes<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self.inner.save_bytes().map_err(py_err)?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn save_dir(&self, path: &str) -> PyResult<()> {
        self.inner.save_dir(Path::new(path)).map_err(py_err)
    }

    fn validate_package(&self) -> PyResult<()> {
        self.inner.validate_package().map_err(py_err)
    }

    #[pyo3(signature = (expected_content_hash=None))]
    fn save_with<'py>(
        &mut self,
        py: Python<'py>,
        expected_content_hash: Option<&str>,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self
            .inner
            .save_with(expected_content_hash)
            .map_err(py_err)?;
        Ok(PyBytes::new(py, &bytes))
    }

    #[pyo3(signature = (scale=2.0))]
    fn export_pdf_bytes_at<'py>(
        &self,
        py: Python<'py>,
        scale: f32,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let scale = k2f_sdk::parse_pdf_scale(scale).map_err(py_err)?;
        let bytes = self.inner.export_pdf_bytes_at(scale).map_err(py_err)?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn export_pdf_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        self.export_pdf_bytes_at(py, 2.0)
    }

    fn export_pptx_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self.inner.export_pptx_bytes().map_err(py_err)?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn export_docx_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = self.inner.export_docx_bytes().map_err(py_err)?;
        Ok(PyBytes::new(py, &bytes))
    }
}
