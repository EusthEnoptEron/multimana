mod bindings;

use crate::utils::{EventHandler, Message, Mod};
use std::any::Any;
use tracing::info;
use crate::python_interpreter::bindings::{py_compile_string, py_exec_code_module};

// language=python
const BOOTSTRAP_SCRIPT: &str = "
import sys

# Add a directory to the module search path
sys.path.append('C:/Users/eusth/.rye/py/cpython@3.7.3/Lib')
sys.path.append('Scripts')

import bootstrap
";

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
        
        let script = py_compile_string(BOOTSTRAP_SCRIPT, "modding.py")?;
        py_exec_code_module("modding", script)?;
        
        info!("Python interpreter initialized!");
        Ok(())
    }

    fn tick(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
