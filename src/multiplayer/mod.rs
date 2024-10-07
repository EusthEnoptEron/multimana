mod player_handler;
mod control_manager;
mod input_manager;

use crate::utils::{EventHandler, Message, Mod, TrampolineWrapper};
use anyhow::{anyhow, Context, Result};
use libmem::Address;
use manasdk::core_u_object::UFunction;
use manasdk::engine::{AActor, APawn, UEngine, UGameEngine, UGameViewportClient, UGameplayStatics, UWorld};
use manasdk::engine_settings::{ETwoPlayerSplitScreenType, UGameMapsSettings};
use manasdk::py_char_base::APyCharBase;
use manasdk::x21::{AActGameState, USakuraBlueprintFunctionLibrary, USakuraEventFunctionLibrary, USakuraEventStateFunctionLibrary};
use manasdk::x21_player_state::APyX21PlayerState;
use manasdk::{EClassCastFlags, FFrame, FNativeFuncPtr, HasClassObject, TFixedSizeArray, UObject, UObjectPointer};
use std::any::Any;
use std::ffi::c_void;
use std::sync::RwLock;
use tracing::{error, info, instrument, warn};
use manasdk::x21_game_mode::APyX21GameMode;
use crate::multiplayer::control_manager::ControlManager;
use crate::multiplayer::input_manager::InputManager;
use crate::multiplayer::player_handler::PlayerHandler;

#[derive(Default)]
struct MultiplayerData {
    initialized: bool,
    exec_function: Option<TrampolineWrapper<FNativeFuncPtr>>,
    control_manager: ControlManager,
    player_handlers: Vec<PlayerHandler>
}

#[derive(Default)]
pub struct MultiplayerMod {
    inner: RwLock<MultiplayerData>,
}

impl EventHandler for MultiplayerMod {
    fn handle_evt(&self, e: &Message) -> Result<()> {
        Ok(())
    }
}

impl Mod for MultiplayerMod {
    fn id() -> u32
    where
        Self: Sized,
    {
        0
    }

    fn name(&self) -> &'static str {
        "Multiplayer Mod"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn init(&self) -> Result<()> {
        info!("Loading multiplayer mod");

        let function: &UFunction = UObject::find_function(|it| it.name() == "ExecutePythonScript")
            .context("Unable to find entry function")?;
        
        // ####################
        // # Hooking OnExec
        // ####################
        fn on_exec_function(context: &UObject, stack: &FFrame, result: *mut c_void) {
            let _ = MultiplayerMod::call_in_place(|this| {
                this.on_process_event(context, stack, result);
                Ok(())
            });
        }

        self.inner
            .write()
            .map_err(|_| anyhow!("Unable to set exec func"))?
            .exec_function
            .replace(unsafe {
                libmem::hook_code(
                    function.exec_function as Address,
                    on_exec_function as Address,
                )
                    .context("Unable to hook into exec function")?
                    .into()
            });

        // ####################
        // # Hooking Inputs
        // ####################
        let _ = InputManager::instance();
        
        info!("Hooked into exec function");
        {
            let mut inner = self.inner
                .write()
                .map_err(|_| anyhow!("Unable to create handlers"))?;

            let control_manager = inner.control_manager.clone();
            inner.player_handlers.push(PlayerHandler::new(1, control_manager.clone()));
            inner.player_handlers.push(PlayerHandler::new(2, control_manager.clone()));
            inner.player_handlers.push(PlayerHandler::new(3, control_manager.clone()));
        }
        
        Ok(())
    }

    fn tick(&self) -> Result<()> {
        let mut inner = self.inner.write().ok().context("Could not read data")?;
        let world = UWorld::get_world().context("World not found")?;
        
        let enabled = UGameplayStatics::get_game_mode(world).try_get()?.cast::<APyX21GameMode>().map(|mode| {
            !mode.is_main_menu_open && !USakuraEventFunctionLibrary::is_playing_non_playable_event(world)
        }).unwrap_or_default();
        
        inner.control_manager.set_enabled(enabled);
        
        for handler in inner.player_handlers.iter_mut() {
            handler.tick(world);
        }
        
        Ok(())
    }
}

impl MultiplayerMod {
    #[instrument(name = "process", target="tracer", fields(name = stack.node.name(), owner = context.name()), skip_all)]
    fn on_process_event(&self, context: &UObject, stack: &FFrame, result: *mut c_void) {
        self.inner.read().ok().map(|inner| {
            if let Some(exec_function) = &inner.exec_function {
                exec_function.get()(context.into(), stack, result);
            }
        });
    }
    
    pub fn on_player_one_is_changing_heroes(&self, hero_id: &str) -> Result<()> {
        let mut inner = self.inner.write().ok().context("Could not read data")?;
        for handler in inner.player_handlers.iter_mut() {
            handler.on_player_changing(hero_id);
        }
        
        Ok(())
    }

    fn try_enable_split_screen(&self, pawn: &APawn, world: &UWorld) -> Result<()> {
        info!("Creating player");
        let second_player = UGameplayStatics::create_player(world, 1, true)
            .try_get()
            .context("Unable to create second player")?;
        info!("Created player: {}", second_player.class_hierarchy());
        
        Ok(())
    }
}
