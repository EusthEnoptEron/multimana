mod bindings;

use crate::python_interpreter::bindings::{py_compile_string, py_exec_code_module};
use crate::utils::{EventHandler, Message, Mod};
use anyhow::{anyhow, Context};
use pyo3::prelude::*;
use pyo3::Python;
use std::any::Any;
use std::thread::Thread;
use tracing::info;

#[derive(Default)]
pub struct PythonInterpreterMod {}

impl EventHandler for PythonInterpreterMod {
    fn handle_evt(&self, e: &Message) -> anyhow::Result<()> {
        Ok(())
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
