use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};
use anyhow::{anyhow, bail, Context};
use lazy_static::lazy_static;
use libmem::Trampoline;
use tracing::{error, info};
use crate::multiplayer::MultiplayerMod;
use crate::tracer::Tracer;

#[derive(Debug)]
pub struct TrampolineWrapper<T>(Trampoline, PhantomData<T>);

impl<T> TrampolineWrapper<T> {
    pub(crate) fn get(&self) -> T {
        unsafe {
            self.0.callable()
        }
    }
}

impl<T> From<Trampoline> for TrampolineWrapper<T> {
    fn from(value: Trampoline) -> Self {
        TrampolineWrapper(
            value,
            PhantomData::default(),
        )
    }
}

lazy_static! {
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

pub trait Mod : Any + Send + Sync + 'static {
    fn id() -> u32 where Self: Sized;
    fn name(&self) -> &'static str;
    
    fn as_any(&self) -> &dyn Any;

    fn init(&self) -> anyhow::Result<()>;
    fn tick(&self) -> anyhow::Result<()>;

    fn call_in_place(fun: impl Fn(&Self) -> anyhow::Result<()>) -> anyhow::Result<()> where Self: Sized {
        let id = Self::id();
        // Get the write lock
        let mod_ = &MODS[&id];

        // Downcast the trait object to the specific type
        let mod_ = mod_.as_any().downcast_ref::<Self>().context("Unable to downcast mod")?;

        // Call the function with the mutable reference to the specific mod
        if let Err(e) = fun(mod_) {
            error!("Error occurred while calling call_in_place! {}", e);
        }

        Ok(())
    }
}
