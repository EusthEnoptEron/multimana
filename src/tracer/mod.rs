mod kismet_tracing;
mod to_string;

use crate::tracer::kismet_tracing::get_params;
use crate::utils::{Message, EventHandler, Mod, TrampolineWrapper};
use anyhow::{anyhow, Context};
use libmem::Address;
use std::any::Any;
use std::ffi::c_void;
use std::sync::OnceLock;
use tracing::{info, instrument, trace_span};
use tracing::field;
use manasdk::{EPropertyFlags, FFrame, FNativeFuncPtr, FScriptName, UObject, UObjectPointer};
use manasdk::core_u_object::{UFunction};
use manasdk::engine::{UGameplayStatics, UWorld};
use manasdk::py_char_base::APyCharBase;
use manasdk::x21::{AActPlayerState, UActCharacterMovementComponent};
use crate::tracer::to_string::to_string_fproperty;

static VIRTUAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();
static FINAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();
static LOCAL_FINAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> =
    OnceLock::new();
static LOCAL_VIRTUAL_FUNCTION_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> =
    OnceLock::new();
static MATH_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();
static CONTEXT_TRAMPOLINE: OnceLock<TrampolineWrapper<FNativeFuncPtr>> = OnceLock::new();

static GNATIVES: OnceLock<&'static [FNativeFuncPtr; 0x100]> = OnceLock::new();

#[derive(Default)]
pub struct Tracer {
    pawn_ref: OnceLock<UObjectPointer<APyCharBase>>
}

const EX_END_FUNCTION_PARAMS: u8 = 0x16;

fn log_function_call(
    context: &UObject,
    stack: &FFrame,
    result: *mut c_void,
    fun: FNativeFuncPtr,
    is_final: bool,
) {
    unsafe {
        let (function, code_offset) = if is_final {
            (
                (stack.code as *const *const UFunction)
                    .read_unaligned()
                    .as_ref(),
                size_of::<usize>(),
            )
        } else {
            (
                if let Some(function_name) = (stack.code as *const FScriptName)
                    .as_ref()
                    .map(|it| it.clone().into())
                {
                    context
                        .class
                        .as_ref()
                        .unwrap()
                        .find_function_by_name(&function_name)
                } else {
                    None
                },
                size_of::<FScriptName>(),
            )
        };

        let function_name = function
            .map(|it| it.name())
            .unwrap_or("Unknown".to_string());
        let object_name = context.name();
        let class_name = context
            .class
            .as_ref()
            .map(|it| it.name())
            .unwrap_or("Unknown".to_string());
        let params = get_params(function, code_offset, context, stack);

        let span = if is_final {
            trace_span!(target: "tracer", "fn_final", name = function_name, object = object_name, class = class_name, params, result = field::Empty)
        } else {
            trace_span!(target: "tracer", "fn_virtual", name = function_name, object = object_name, class = class_name, params, result = field::Empty)
        };

        span.in_scope(|| {
            fun(context.into(), stack, result);
            if let Some(function) = function.filter(|it| it.return_value_offset != u16::MAX) {
                if let Some(return_prop) = function.child_properties().find(|it| it.property_flags.contains(EPropertyFlags::ReturnParm)) {
                    span.record("result", to_string_fproperty(return_prop, result));
                }
            }
        });
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
        fn ex_virtual_function(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = VIRTUAL_FUNCTION_TRAMPOLINE.get() {
                log_function_call(context.as_ref().unwrap(), stack, result, trampoline.get(), false);
            }
        }

        #[instrument(name = "final", target = "tracer", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_final_function(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = FINAL_FUNCTION_TRAMPOLINE.get() {
                log_function_call(context.as_ref().unwrap(), stack, result, trampoline.get(), true);
            }
        }

        #[instrument(name = "virtual_local", target = "tracer", level = "trace", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_local_virtual_function(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = LOCAL_VIRTUAL_FUNCTION_TRAMPOLINE.get() {
                log_function_call(context.as_ref().unwrap(), stack, result, trampoline.get(), false);
            }
        }

        #[instrument(name = "final_local", target = "tracer", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_local_final_function(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = LOCAL_FINAL_FUNCTION_TRAMPOLINE.get() {
                log_function_call(context.as_ref().unwrap(), stack, result, trampoline.get(), true);
            }
        }

        #[instrument(name = "math", target = "tracer", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_math(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = MATH_TRAMPOLINE.get() {
                log_function_call(context.as_ref().unwrap(), stack, result, trampoline.get(), true);
            }
        }

        #[instrument(name = "context", target = "tracer", fields(name = stack.node.name(), owner = context.name()
        ), skip_all)]
        fn ex_context(context: UObjectPointer<UObject>, stack: &FFrame, result: *mut c_void) {
            if let Some(trampoline) = CONTEXT_TRAMPOLINE.get() {
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

            info!("Hooking local final function");
            LOCAL_FINAL_FUNCTION_TRAMPOLINE
                .set(
                    libmem::hook::hook_code(
                        gnatives[0x46] as Address,
                        ex_local_final_function as Address,
                    )
                    .context("")?
                    .into(),
                )
                .map_err(|_| anyhow!("Failed to set trampoline"))?;

            info!("Hooking local virtual function");
            LOCAL_VIRTUAL_FUNCTION_TRAMPOLINE
                .set(
                    libmem::hook::hook_code(
                        gnatives[0x45] as Address,
                        ex_local_virtual_function as Address,
                    )
                    .context("")?
                    .into(),
                )
                .map_err(|_| anyhow!("Failed to set trampoline"))?;

            info!("Hooking math function");
            MATH_TRAMPOLINE
                .set(
                    libmem::hook::hook_code(gnatives[0x68] as Address, ex_math as Address)
                        .context("")?
                        .into(),
                )
                .map_err(|_| anyhow!("Failed to set trampoline"))?;
            //
            // info!("Hooking context function");
            // CONTEXT_TRAMPOLINE
            //     .set(
            //         libmem::hook::hook_code(gnatives[0x19] as Address, ex_context as Address)
            //             .context("")?
            //             .into(),
            //     )
            //     .map_err(|_| anyhow!("Failed to set trampoline"))?;
        }
        info!("Done");

        Ok(())
    }

    fn tick(&self) -> anyhow::Result<()> {
        Ok(())
    }
}


impl EventHandler for Tracer {
    fn handle_evt(&self, e: &Message) -> anyhow::Result<()> {
        match e {
            Message::LogPlayerPawn => {
                let pawn = self.pawn_ref.get_or_init(|| {
                    let world = UWorld::get_world().context("Could not get world").unwrap();
                    UGameplayStatics::get_player_pawn(world, 0).try_get()
                        .context("Could not get player pawn").unwrap()
                        .cast::<APyCharBase>()
                        .context("Cannot cast to pychar").unwrap()
                        .into()
                }).as_ref().context("Unable to get pawn")?;
                
                info!("Pawn class: {}", pawn.class_hierarchy());
                info!("Pawn: {pawn:#?}");

                let movement = pawn.get_movement_component().try_get()
                    .context("Could not get movement component")?
                    .cast::<UActCharacterMovementComponent>()
                    .context("Could not cast to UActCharacterMovementComponent")?;

                info!("Movement: {movement:#?}");

                let player_state = pawn.player_state.as_ref()
                    .context("Could not get player state")?
                    .cast::<AActPlayerState>()
                    .context("Could not cast to AActPlayerState")?;
                
                info!("State: {player_state:#?}");

                let input_comp = pawn.input_component.as_ref()
                    .context("Could not get input component")?;

                info!("Input: {input_comp:#?}");
                info!("Input Class: {:#?}", input_comp.class_hierarchy());
            }
        }

        Ok(())
    }
}