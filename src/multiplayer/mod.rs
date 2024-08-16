use crate::utils::{Mod, TrampolineWrapper};
use anyhow::{anyhow, bail, Context, Result};
use libmem::Address;
use manasdk::{AActor, APawn, EFunctionFlags, ETwoPlayerSplitScreenType, FFrame, FNativeFuncPtr, HasClassObject, TArray, TFixedSizeArray, UClass, UField, UFunction, UGameMapsSettings, UGameplayStatics, UObject, UObjectPointer, UWorld};
use std::any::Any;
use std::ffi::c_void;
use std::sync::RwLock;
use tracing::{info, instrument, span, warn, Level};


#[derive(Default)]
struct MultiplayerData {
    initialized: bool,
    exec_function: Option<TrampolineWrapper<FNativeFuncPtr>>,
}

#[derive(Default)]
pub struct MultiplayerMod {
    inner: RwLock<MultiplayerData>,
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
        
        let function: &UFunction = UObject::find_function(|it| it.function_flags.contains(EFunctionFlags::BlueprintEvent))
            .context("Unable to find entry function")?; 

        fn on_exec_function(context: &UObject, stack: &FFrame, result: *mut c_void) {
            let _ = MultiplayerMod::call_in_place(|this| {
                this.on_process_event(context, stack, result);
                Ok(())
            });
        }

        self.inner.write().map_err(|_| anyhow!("Unable to set exec func"))?.exec_function.replace(
            unsafe {
                libmem::hook_code(function.exec_function as Address, on_exec_function as Address)
                    .context("Unable to hook into exec function")?
                    .into()
            }
        );
        info!("Hooked into exec function");
        Ok(())
    }

    fn tick(&self) -> Result<()> {
        if self.inner.read().ok().context("Could not read data")?.initialized {
            return Ok(());
        }

        let world = UWorld::get_world().context("World not found")?;

        let player_controller = UGameplayStatics::get_player_controller(world, 0);
        let pawn = UGameplayStatics::get_player_pawn(world, 0);

        if let (Some(controller), Some(pawn)) = (player_controller.as_ref(), pawn.as_ref()) {
            if pawn.class.as_ref().unwrap().name() == "BP_P001c_C" {
                info!("Initializing");
                self.inner.write().ok().context("Could not lock data")?.initialized = true;
                if let Err(e) = self.try_enable_split_screen(pawn, world) {
                    warn!("Failed to enable split screen: {:?}", e);
                } else {
                    info!("Split screen activated.");
                }
            }
        }

        Ok(())
    }
}

impl MultiplayerMod {
    #[instrument(name = "fn", target="tracer", fields(name = stack.node.name(), owner = context.name()), skip_all)]
    fn on_process_event(&self, context: &UObject, stack: &FFrame, result: *mut c_void) {
        self.inner.read().ok().map(|inner| {
            if let Some(exec_function) = &inner.exec_function {
                exec_function.get()(context, stack, result);
            }
        });
    }

    fn try_enable_split_screen(&self, pawn: &APawn, world: &UWorld) -> Result<()> {
        // Enable split screen
        info!("Getting settings");
        let settings = UGameMapsSettings::get_game_maps_settings().try_get()?;
        info!("Got settings");
        settings.b_use_splitscreen = true;
        settings.b_offset_player_gamepad_ids = false;
        settings.two_player_splitscreen_layout = ETwoPlayerSplitScreenType::Vertical;


        info!("Creating player");
        let second_player = UGameplayStatics::create_player(world, 1, true)
            .try_get().context("Unable to create second player")?;
        info!("Created player");


        info!("Creating array");
        let mut actors = TFixedSizeArray::<&AActor, 5000usize>::new();
        //let mut actors = TArray::<&AActor>::default();
        info!("Gettings actors");
        UGameplayStatics::get_all_actors_of_class(world, APawn::static_class().into(), actors.as_mut());

        info!("Actor count: {} (capacity={})", actors.len(), actors.max_elements);
        if let Some(actor) = actors.iter().find(|it| it.class.as_ref().unwrap().name() == "BP_P002_C") {
            second_player.possess(actor.cast().context("Unable to cast actor to pawn")?);
        } else {
            bail!("No other pawns found!");
        }

        Ok(())
    }
}