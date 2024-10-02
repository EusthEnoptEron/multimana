use manasdk::UObjectPointer;
use manasdk::x21::{AACTPlayerController};
use crate::multiplayer::control_manager::{Claim, ControlManager};

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

    pub fn tick(&mut self) {
        // Handle the condition when we don't multiplayer
        if !self.connected {
            if let Some(claim) = self.current_claim.take() {
                self.control_manager.return_claim(claim);
            }
            return;
        }

        // Renew claim to character
        if let Some(claim) = self.current_claim.take() {
            self.current_claim = self.control_manager.renew_claim(claim);
        } else {
            self.current_claim = self.control_manager.claim(self.player_id, self.controller.clone());
        }
    }
}
