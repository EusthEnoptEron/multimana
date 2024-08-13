use tracing::{info, warn};
use manasdk::{AActor, APawn, ETwoPlayerSplitScreenType, HasClassObject, TArray, TFixedSizeArray, UClass, UField, UGameMapsSettings, UGameplayStatics, UObjectPointer, UWorld};
use anyhow::{bail, Context, Result};

#[derive(Default)]
pub struct MultiplayerMod {
    initialized: bool
}

impl MultiplayerMod {
    pub fn init(&mut self) {
        info!("Loading multiplayer mod");
    }

    pub fn tick(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        let world = UWorld::get_world().context("World not found")?;
        
        let player_controller = UGameplayStatics::get_player_controller(world, 0);
        let pawn = UGameplayStatics::get_player_pawn(world, 0);

        if let (Some(controller), Some(pawn)) = (player_controller.as_ref(), pawn.as_ref()) {
            if pawn.class.as_ref().unwrap().name() == "BP_P001c_C" {
                info!("Initializing");
                self.initialized = true;
                if let Err(e) = self.try_enable_split_screen(pawn, world) {
                    warn!("Failed to enable split screen: {:?}", e);
                } else {
                    info!("Split screen activated.");
                }
            }
        }
        

        Ok(())
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