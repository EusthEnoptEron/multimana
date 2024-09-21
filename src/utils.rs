use crate::statics::MODS;
use anyhow::{anyhow, Context};
use concurrent_queue::ConcurrentQueue;
use libmem::Trampoline;
use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tracing::error;

#[derive(Debug)]
pub struct TrampolineWrapper<T>(Trampoline, PhantomData<T>);

impl<T> TrampolineWrapper<T> {
    pub(crate) fn get(&self) -> T {
        unsafe { self.0.callable() }
    }
}

impl<T> From<Trampoline> for TrampolineWrapper<T> {
    fn from(value: Trampoline) -> Self {
        TrampolineWrapper(value, PhantomData::default())
    }
}

pub trait Mod: Any + EventHandler + Send + Sync + 'static {
    fn id() -> u32
    where
        Self: Sized;
    fn name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn init(&self) -> anyhow::Result<()>;
    fn tick(&self) -> anyhow::Result<()>;

    fn call_in_place(fun: impl Fn(&Self) -> anyhow::Result<()>) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let id = Self::id();
        // Get the write lock
        let mod_ = &MODS[&id];

        // Downcast the trait object to the specific type
        let mod_ = mod_
            .as_any()
            .downcast_ref::<Self>()
            .context("Unable to downcast mod")?;

        // Call the function with the mutable reference to the specific mod
        if let Err(e) = fun(mod_) {
            error!("Error occurred while calling call_in_place! {}", e);
        }

        Ok(())
    }
}

pub struct MessageBus {
    handlers: RwLock<Vec<Arc<dyn EventHandler>>>,
    message_queue: ConcurrentQueue<Message>,
}

impl MessageBus {
    pub fn add_handler(&self, handler: Arc<dyn EventHandler>) -> anyhow::Result<()> {
        let mut handlers = self
            .handlers
            .write()
            .map_err(|_| anyhow!("Unable to lock handlers"))?;
        
        handlers.push(handler);

        Ok(())
    }

    pub fn dispatch(&self, evt: Message) {
        if let Err(e) = self.message_queue.push(evt) {
            error!("Failed to push event to queue: {e}");
        }
    }
    
    pub fn tick(&self) {
        if let Some(handlers) = self.handlers.read().ok() {
            while let Ok(evt) = self.message_queue.pop() {
                for handler in handlers.iter() {
                    if let Err(e) = handler.handle_evt(&evt) {
                        error!("Error happened in handler for evt ({evt:?}): {e}");
                    }
                }
            }    
        }
    }

    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
            message_queue: ConcurrentQueue::bounded(50),
        }
    }
}

pub trait EventHandler: Send + Sync + 'static {
    fn handle_evt(&self, e: &Message) -> anyhow::Result<()>;
}

#[derive(Clone, Debug)]
pub enum Message {
    LogPlayerPawn,
}
