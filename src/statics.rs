use crate::multiplayer::MultiplayerMod;
use crate::tracer::Tracer;
use crate::utils::Mod;
use concurrent_queue::ConcurrentQueue;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{error, info};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{reload, Registry};

pub static TRACER_RELOAD_HANDLE: OnceLock<reload::Handle<Targets, Registry>> = OnceLock::new();

lazy_static! {
    pub static ref WORKER_QUEUE: ConcurrentQueue<Box<dyn FnOnce() + Send>> = ConcurrentQueue::bounded(5usize);

    // Mods are registered via their id
    pub static ref MODS: HashMap<u32, Box<dyn Mod + 'static>> = {
        let mut m: HashMap<u32, Box<dyn Mod + 'static>> = HashMap::new();
        m.insert(MultiplayerMod::id(), Box::new(MultiplayerMod::default()));
        m.insert(Tracer::id(), Box::new(Tracer::default()));

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
