use pyo3::{
    types::{PyModule, PyTuple},
    PyCell, PyResult, Python,
};
use tracing::{debug, info};

pub struct PythonDaemon;

impl PythonDaemon {
    pub async fn run(&self) {
        let mut python_script = std::env::current_dir().expect("Cannot get current directory");
        python_script.push("trade-ml");
        python_script.push("src");
        python_script.push("trademl");
        python_script.push("entrypoint.py");

        let python_script = python_script.to_str().unwrap();

        Python::with_gil(|py| {
            info!("Created python daemon: Python v{}", py.version());

            let sys = PyModule::import(py, "sys")?;
            let sys_path = sys.getattr("path")?;
            sys_path.call_method("append", PyTuple::new(py, vec![python_script]), None)?;
            debug!("Imported sys");

            let import_lib_util = PyModule::import(py, "importlib.util")?;
            debug!("Imported importlib.util");

            let spec = import_lib_util.call_method(
                "spec_from_file_location",
                PyTuple::new(py, vec!["module.name", python_script]),
                None,
            )?;

            let entrypoint: &PyModule = import_lib_util
                .call_method("module_from_spec", PyTuple::new(py, vec![spec]), None)?
                .cast_as()?;
            debug!("Imported entrypoint");

            spec.getattr("loader")?.call_method(
                "exec_module",
                PyTuple::new(py, vec![entrypoint]),
                None,
            )?;
            debug!("Executed entrypoint module");

            let interface = PyCell::new(py, crate::interface::NodeInterface {})?;
            entrypoint.call_method("run", PyTuple::new(py, vec![interface]), None)?;
            PyResult::Ok(())
        })
        .expect("Python global interpreter lock failed")
    }
}
