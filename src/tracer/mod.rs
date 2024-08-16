use crate::utils::{Mod, TrampolineWrapper};
use anyhow::{anyhow, Context};
use libmem::Address;
use manasdk::{FFrame, FNativeFuncPtr, UObject};
use std::any::Any;
use std::ffi::c_void;
use std::sync::OnceLock;
use tracing::{info, instrument};

static VIRTUAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();
static FINAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();

#[derive(Default)]
pub struct Tracer {}

impl Mod for Tracer {
    fn id() -> u32
    where
        Self: Sized,
    {
        1
    }

    fn name(&self) -> &'static str {
        "Tracer"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn init(&self) -> anyhow::Result<()> {
        info!("Loading tracer mod");

        let module = libmem::enum_modules()
            .context("Unable to get modules")?
            .first()
            .cloned()
            .context("Unable to find any modules")?;

        // 0x147c30bc0
        let step_fn = unsafe {
            libmem::sig_scan(
                "75 49 C6 05 ?? ?? ?? ?? ?? 48 8D 05 ?? ?? ?? ??",
                module.base,
                module.size,
            )
            .context("Unable to find gnatives")?
        };

        let offset: &u32 = unsafe { std::mem::transmute(step_fn + 12) };
        let gnatives_address = step_fn + 12 + 4 + (*offset) as usize;

        info!("GNatives Address: {:x}", gnatives_address);
        let gnatives: &[FNativeFuncPtr; 0x100] = unsafe { std::mem::transmute(gnatives_address) };

        #[instrument(name = "virtual", target = "tracer", level = "trace", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_virtual_function(context: &UObject, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = VIRTUAL_FUNCTION_TRAMPOLINE.get() {
                trampoline.get()(context, stack, result);
            }
        }

        #[instrument(name = "final", target = "tracer", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_final_function(context: &UObject, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = FINAL_FUNCTION_TRAMPOLINE.get() {
                trampoline.get()(context, stack, result);
            }
        }

        unsafe {
            info!("Hooking virtual function");
            VIRTUAL_FUNCTION_TRAMPOLINE
                .set(
                    libmem::hook::hook_code(
                        gnatives[0x1B] as Address,
                        ex_virtual_function as Address,
                    )
                    .context("")?
                    .into(),
                )
                .map_err(|_| anyhow!("Failed to set trampoline"))?;

            info!("Hooking final function");
            FINAL_FUNCTION_TRAMPOLINE
                .set(
                    libmem::hook::hook_code(
                        gnatives[0x1C] as Address,
                        ex_final_function as Address,
                    )
                    .context("")?
                    .into(),
                )
                .map_err(|_| anyhow!("Failed to set trampoline"))?;
        }
        info!("Done");

        Ok(())
    }

    fn tick(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
