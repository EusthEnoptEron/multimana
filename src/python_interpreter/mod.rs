mod bindings;

use crate::statics::MESSAGE_BUS;
use crate::utils::Message::PythonOutput;
use crate::utils::{EventHandler, Message, MessageBus, Mod};
use anyhow::anyhow;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::Python;
use std::any::Any;
use std::sync::{Mutex};
use tracing::{error, info, trace, warn};
use crate::multiplayer::MultiplayerMod;

#[derive(Default)]
pub struct PythonInterpreterMod {
    locals: Mutex<Option<Py<PyDict>>>,
}

#[pyfunction]
fn log(text: &str, severity: i32) {
    match severity {
        0 => error!("{text}"),
        1 => warn!("{text}"),
        _ => trace!("{text}"),
    }
}

#[pyfunction]
fn on_hero_changing(hero_id: &str) {
   info!("Hero is changing to: {}", hero_id);
    let _ = MultiplayerMod::call_in_place(|it| {
        it.on_player_one_is_changing_heroes(hero_id)
    });
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
            m.add_function(wrap_pyfunction!(on_hero_changing, &m)?)?;

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

impl EventHandler for PythonInterpreterMod {
    fn handle_evt(&self, e: &Message) -> anyhow::Result<()> {
        if let Message::ExecutePython { code, eval } = e {
            info!("Executing code: {code}");
            MESSAGE_BUS.dispatch(PythonOutput { output: code.clone() });

            Python::with_gil(|py| {
                let mut lock = self.locals.lock().unwrap();
                let locals = lock.take().map(|it| it.into_bound(py))
                    .unwrap_or_else(|| PyDict::new_bound(py));
                
                let result = if *eval {
                    py.eval_bound(code.as_str(), None, None).map(|it| it.to_string())
                } else {
                    py.run_bound(code.as_str(), None, None).map(|_| "OK".to_string())
                };
                
                match result {
                    Ok(bound) => {
                        MESSAGE_BUS.dispatch(PythonOutput { output: bound });
                    }
                    Err(err) => {
                        MESSAGE_BUS.dispatch(PythonOutput { output: err.to_string() });
                    }
                }
                
                lock.replace(locals.unbind());
            });
        }

        Ok(())
    }
}