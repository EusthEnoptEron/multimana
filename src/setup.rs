use crate::multiplayer::MultiplayerMod;
use crate::utils::{Mod, TrampolineWrapper, MODS};
use anyhow::Context;
use libmem::{Address};
use manasdk::{UEngine, UGameplayStatics, UObject, UObjectPointer, UWorld};
use std::cell::{OnceCell, RefCell};
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock, RwLock};
use std::thread::sleep;
use std::time::Duration;
use tracing::{error, info, warn};

type TickFn = fn(this: *const c_void);
static ORIGINAL_TICK: OnceLock<TrampolineWrapper<TickFn>> = OnceLock::new();
fn tick(this: *const c_void) {
    for mod_ in MODS.values() {
        if let Err(error) = mod_.tick() {
            error!("Error in tick: {} mod={}", error, mod_.name());
        }
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
    
    info!("Looking into module {}", module.name);

    let tick_ptr = unsafe {
        libmem::sig_scan(
            "48 89 4C 24 08 55 53 56 57 41 54 41 55 41 56 41 57 48 8D AC 24 ?? ?? ?? ?? 48 81 EC ?? ?? ?? ?? 83 3D",
            module.base,
            module.size,
        ).context("Tick pointer not found")?
    };

    ORIGINAL_TICK.set(unsafe { libmem::hook_code(tick_ptr, tick as Address).context("Unable to create tick hook") }?.into()).unwrap();
    
    Ok(())
}
