mod to_string;
mod kismet_tracing;

use crate::utils::{Mod, TrampolineWrapper};
use anyhow::{anyhow, Context};
use libmem::Address;
use manasdk::{EPropertyFlags, FFrame, FName, FNativeFuncPtr, FRotator, FScriptName, FString, FVector, FVector2D, UBoolProperty, UByteProperty, UClass, UDoubleProperty, UEnum, UEnumProperty, UFloatProperty, UFunction, UInt64Property, UInt8Property, UIntProperty, UNameProperty, UObject, UObjectProperty, UProperty, UScriptStruct, UStrProperty, UStructProperty, UUInt32Property, UUInt64Property};
use std::any::Any;
use std::ffi::c_void;
use std::sync::OnceLock;
use tracing::{info, info_span, instrument, trace_span};
use tracing_subscriber::fmt::format;
use crate::tracer::kismet_tracing::get_params;

static VIRTUAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();
static FINAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();
static GNATIVES: OnceLock<&'static [FNativeFuncPtr; 0x100]> = OnceLock::new();

#[derive(Default)]
pub struct Tracer {}
const EX_END_FUNCTION_PARAMS: u8 = 0x16;

fn log_function_call(
    context: &UObject,
    stack: &FFrame,
    result: *mut c_void,
    fun: FNativeFuncPtr,
    is_final: bool
) {
    unsafe {
        let (function, code_offset) = if is_final {
            ((stack.code as *const *const UFunction).read_unaligned().as_ref(), size_of::<usize>())
        } else {
            (if let Some(function_name) = (stack.code as *const FScriptName).as_ref()
                .map(|it| it.clone().into()) {
                context.class.as_ref().unwrap().find_function_by_name(&function_name)
            } else {
                None
            }, size_of::<FScriptName>())
        };

        let function_name = function.map(|it| it.name()).unwrap_or("Unknown".to_string());
        let object_name = context.name();
        let class_name = context.class.as_ref().map(|it| it.name()).unwrap_or("Unknown".to_string());
        let params = get_params(function, code_offset, context, stack);

        if is_final {
            trace_span!(target: "tracer", "fn_final", name = function_name, object = object_name, class = class_name, params).in_scope(|| {
                fun(context, stack, result);
            });
        } else {
            trace_span!(target: "tracer", "fn_virtual", name = function_name, object = object_name, class = class_name, params).in_scope(|| {
                fun(context, stack, result);
            });
        }
    }
}


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
        GNATIVES.set(gnatives).unwrap();

        #[instrument(name = "virtual", target = "tracer", level = "trace", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_virtual_function(context: &UObject, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = VIRTUAL_FUNCTION_TRAMPOLINE.get() {
                log_function_call(context, stack, result, trampoline.get(), false);
                //trampoline.get()(context, stack, result);
            }
        }

        #[instrument(name = "final", target = "tracer", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_final_function(context: &UObject, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = FINAL_FUNCTION_TRAMPOLINE.get() {
                log_function_call(context, stack, result, trampoline.get(), true);
//                trampoline.get()(context, stack, result);
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
