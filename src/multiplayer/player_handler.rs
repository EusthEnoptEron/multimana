use tracing::info;
use manasdk::engine::{UGameplayStatics, UWorld};
use manasdk::UObjectPointer;
use manasdk::x21::{AACTPlayerController};
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
