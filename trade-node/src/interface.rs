use pyo3::prelude::*;

// Keep in sync with trade-ml/src/trademl/interface.py
#[pyclass]
pub struct NodeInterface {}

#[pymethods]
impl NodeInterface {
    fn get_data(&self) -> PyObject {
        todo!()
    }
}
