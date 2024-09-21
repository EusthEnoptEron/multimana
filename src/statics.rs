use crate::multiplayer::MultiplayerMod;
use crate::tracer::Tracer;
use crate::utils::{MessageBus, Mod};
use concurrent_queue::ConcurrentQueue;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tracing::{error, info};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{reload, Registry};
use crate::python_interpreter::PythonInterpreterMod;

pub static TRACER_RELOAD_HANDLE: OnceLock<reload::Handle<Targets, Registry>> = OnceLock::new();

lazy_static! {
    pub static ref WORKER_QUEUE: ConcurrentQueue<Box<dyn FnOnce() + Send + Sync>> = ConcurrentQueue::bounded(5usize);
    pub static ref MESSAGE_BUS: MessageBus = MessageBus::new();

    // Mods are registered via their id
    pub static ref MODS: HashMap<u32, Arc<dyn Mod + 'static>> = {
        let mut m: HashMap<u32, Arc<dyn Mod + 'static>> = HashMap::new();
        m.insert(MultiplayerMod::id(), { 
            let mod_ = Arc::new(MultiplayerMod::default());
            MESSAGE_BUS.add_handler(mod_.clone()).unwrap();
            mod_
        });
        m.insert(Tracer::id(), { 
            let mod_ = Arc::new(Tracer::default());
            MESSAGE_BUS.add_handler(mod_.clone()).unwrap();
            mod_
        });
        m.insert(PythonInterpreterMod::id(), { 
            let mod_ = Arc::new(PythonInterpreterMod::default());
            MESSAGE_BUS.add_handler(mod_.clone()).unwrap();
            mod_
        });

        info!("Initializing mods...");
        for item in m.values() {
            if let Err(error) = item.init() {
                error!("Failed to initialize mod: {}", error);
            }
        }
        info!("We're all set");

        m
    };
}
