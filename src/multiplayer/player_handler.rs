use tracing::info;
use manasdk::engine::{UGameplayStatics, UWorld};
use manasdk::UObjectPointer;
use manasdk::wbp_hud::UWBP_HUD_C;
use manasdk::x21::{AACTPlayerController};
use manasdk::x21_function_library::UPyX21FunctionLibrary;
use manasdk::x21_hud::APyX21Hud;
use crate::multiplayer::control_manager::{Claim, ControlManager};
use crate::multiplayer::input_manager::InputManager;

pub struct PlayerHandler {
    /// The id of the player (1-3)
    player_id: u8,
    /// Whether the handler currently has a gamepad connected.
    connected: bool,
    /// The player controller, if any.
    controller: UObjectPointer<AACTPlayerController>,
    control_manager: ControlManager,
    current_claim: Option<Claim>,
}

impl PlayerHandler {
    pub fn new(player_id: u8, control_manager: ControlManager) -> Self {
        Self { player_id,
            connected: false,
            controller: UObjectPointer::default(),
            control_manager,
            current_claim: None
        }
    }

    pub fn tick(&mut self, world: &UWorld) {
        self.connected = InputManager::instance().get_controller_state(self.player_id).is_some();
        
        // Handle the condition when we don't multiplayer
        if !self.connected {
            if let Some(claim) = self.current_claim.take() {
                self.control_manager.return_claim(claim);
            }
            
            if self.controller.as_ref().is_some() {
                UGameplayStatics::remove_player(self.controller.as_ref().unwrap(), false);
                self.controller = UObjectPointer::default();
            }
            
            return;
        } else {
            if self.controller.as_ref().take_if(|it| it.is_valid()).is_none() {
                if world.name() == "MAP_AsyncPersistent" {
                    let controller = UGameplayStatics::create_player(world, self.player_id as i32, true).try_get().ok()
                        .or_else(|| UGameplayStatics::get_player_controller(world, self.player_id as i32).try_get().ok())
                        .and_then(|it| {
                            info!("Created {}", it.class_hierarchy());
                            if let Some(hud) = it.my_hud.cast::<APyX21Hud>().and_then(|it| it.try_get().ok()) {
                                hud.init_main_hud();
                                if let Some(main_hud) = UPyX21FunctionLibrary::py_get_main_hud(hud).cast::<UWBP_HUD_C>().and_then(|hud| hud.try_get().ok()) {
                                    info!("Got main HUD!");
                                    // if let Some(minimap) = main_hud.wbp_minimap.as_ref() {
                                    //     minimap.set_visibility(ESlateVisibility::Collapsed);
                                    // }

                                    // Update owner
                                    main_hud.set_owning_player(it);

                                    // Add to their screen
                                    main_hud.remove_from_viewport();
                                    main_hud.add_to_player_screen(0);
                                }
                            }
                            Some(it)
                        })
                        .and_then(|it| it.cast::<AACTPlayerController>())
                        .map(|it| UObjectPointer::from(it))
                        .unwrap_or_default();


                    info!("Created controller: {controller:?}");
                    self.controller = controller;
                }
            }
        }

        // Renew claim to character
        if let Some(claim) = self.current_claim.take() {
            self.current_claim = self.control_manager.renew_claim(claim);
        } else {
            self.current_claim = self.control_manager.claim(self.player_id, self.controller.clone());
        }
    }
    
    pub fn on_player_changing(&mut self, new_hero: &str) {
        if let Some(claim) = self.current_claim.take_if(|it| it.hero_id() == new_hero) {
            info!("Player is changing into our character... returning it immediately!");
            self.control_manager.return_claim(claim);
        }
    } 
}
