use std::cell::{LazyCell, OnceCell, RefCell};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::{LazyLock, Mutex, OnceLock, RwLock};
use std::thread::sleep;
use std::time::Duration;
use anyhow::Context;
use libmem::{Address, Trampoline};
use tracing::{info, warn};
use manasdk::{UClass, UEngine, UGameplayStatics, UObject, UObjectPointer, UWorld};
use multiplayer::MultiplayerMod;

#[derive(Debug)]
struct TrampolineWrapper<T>(Trampoline, PhantomData<T>);

impl<T> TrampolineWrapper<T> {
    fn get(&self) -> T {
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


static MOD: LazyLock<Mutex<RefCell<MultiplayerMod>>> = LazyLock::new(|| {
    let mut mod_ = MultiplayerMod::default();
    mod_.init();

    Mutex::new(RefCell::new(mod_))
});

type TickFn = fn(this: *const c_void);
static ORIGINAL_TICK: OnceLock<TrampolineWrapper<TickFn>> = OnceLock::new();
fn tick(this: *const c_void) {
    if let Err(e) = MOD.lock().unwrap().borrow_mut().tick() {
        warn!("Error in tick: {}", e);
    }
    
    if let Some(original_fn) = ORIGINAL_TICK.get() {
        original_fn.get()(this);
    }
}

pub fn setup() -> anyhow::Result<()> {
    info!("Attached!");

    sleep(Duration::from_secs(5));

    let module = libmem::enum_modules().context("Unable to get modules")?
        .first()
        .cloned()
        .context("Unable to find any modules")?;

    let tick_ptr = unsafe {
        libmem::sig_scan(
            "48 89 4C 24 08 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 ?? ?? ?? ?? 48 81 EC ?? ?? ?? ?? 83 3D E9 F9 8E 06 FF",
            module.base,
            module.size,
        ).context("Tick pointer not found")?
    };

    ORIGINAL_TICK.set(unsafe { libmem::hook_code(tick_ptr, tick as Address).context("Unable to create tick hook") }?.into()).unwrap();

    Ok(())
}
