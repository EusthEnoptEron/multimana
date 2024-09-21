mod bindings;

use crate::utils::{EventHandler, Message, Mod};
use pyo3::prelude::*;
use pyo3::Python;
use std::any::Any;
use anyhow::anyhow;
use pyo3::types::PyDict;
use tracing::{error, info, trace, warn};

#[derive(Default)]
pub struct PythonInterpreterMod {}

impl EventHandler for PythonInterpreterMod {
    fn handle_evt(&self, e: &Message) -> anyhow::Result<()> {
        Ok(())
    }
}


#[pyfunction]
fn log(text: &str, severity: i32) {
    match severity {
        0 => error!("{text}"),
        1 => warn!("{text}"),
        _ => trace!("{text}"),
    }
}

impl Mod for PythonInterpreterMod {
    fn id() -> u32
    where
        Self: Sized,
    {
        3
    }

    fn name(&self) -> &'static str {
        "python_interpreter"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn init(&self) -> anyhow::Result<()> {
        info!("Installing python interpreter");

        let script = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/bootstrap.py"));

        Python::with_gil(|py| {
            // Create new module
            let m = PyModule::new_bound(py, "mod_extensions")?;
            m.add_function(wrap_pyfunction!(log, &m)?)?;

            // Import and get sys.modules
            let sys = PyModule::import_bound(py, "sys")?;
            let py_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into().map_err(|e| anyhow!("{e}"))?;

            // Insert foo into sys.modules
            py_modules.set_item("mod_extensions", m)?;
            
            PyModule::from_code_bound(py, script, "bootstrap.py", "bootstrap")?;
            anyhow::Ok(())
        })?;

        info!("Python interpreter initialized!");
        Ok(())
    }

    fn tick(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
