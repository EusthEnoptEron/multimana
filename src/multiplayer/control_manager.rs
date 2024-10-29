use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};
use manasdk::ai_module::AAIController;
use manasdk::engine::{AController, APawn, APlayerController, APlayerState, UGameplayStatics, UWorld};
use manasdk::py_char_base::APyCharBase;
use manasdk::{AsObjectPointer, HasClassObject, UObjectPointer};
use manasdk::engine_settings::{ETwoPlayerSplitScreenType, UGameMapsSettings};
use manasdk::x21::{AACTPlayerController, AActAIController, AActGameState, ACharacterBase, UActUIFunctionLibrary, USakuraBlueprintFunctionLibrary};
use manasdk::x21_game_mode::APyX21GameMode;
use crate::utils::{EventHandler, LoggableOption, Message};

#[derive(Default)]
struct State {
    enabled: bool,
    active_claims: HashMap<String, ClaimRef>,
}

#[derive(Clone, Default)]
pub struct ControlManager {
    state: Arc<RwLock<State>>,
}


#[derive(Debug)]
pub struct Claim {
    player_id: u8,
    hero_id: String,
    character: Option<UObjectPointer<APyCharBase>>,
    player_controller: UObjectPointer<AACTPlayerController>,
    ai_controller: UObjectPointer<AActAIController>,
}

#[derive(Debug)]
pub struct ClaimRef {
    player_id: u8,
    hero_id: String,
}

impl Claim {
    fn get_ref(&self) -> ClaimRef {
        ClaimRef { player_id: self.player_id, hero_id: self.hero_id.clone() }
    }

    pub fn character(&self) -> &Option<UObjectPointer<APyCharBase>> {
        &self.character
    }

    pub fn hero_id(&self) -> &str {
        &self.hero_id
    }
}

impl ControlManager {
    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut state) = self.state.write() {
            state.enabled = enabled;

            // Enable split screen
            if let Some(settings) = UGameMapsSettings::get_game_maps_settings().try_get().ok() {
                settings.b_use_splitscreen = enabled;
                settings.b_offset_player_gamepad_ids = false;
                settings.two_player_splitscreen_layout = ETwoPlayerSplitScreenType::Vertical;
            }

            for member in self.get_available_members() {
                if let Some(camera_cmp) = member.camera_cmp.as_mut() {
                    camera_cmp.bit_set_b_constrain_aspect_ratio(false);
                }
            }
        }
    }

    fn get_available_members(&self) -> Vec<&mut APyCharBase> {
        UWorld::get_world()
            .and_then(|world| UGameplayStatics::get_game_mode(world).try_get().ok())
            .and_then(|mode| mode.cast::<APyX21GameMode>())
            .map(|mode| {
                mode.cached_team_players.iter()
                    .cloned()
                    .filter_map(|it| it.try_get().ok())
                    .filter_map(|it| it.cast_mut::<APyCharBase>())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// Tries to claim a character for the player.
    ///
    /// Returns a claim when all the following is true:
    ///
    ///- the [player_id] does not already hold another claim.
    ///- a player is available .
    pub fn claim(&self, player_id: u8, player_controller: UObjectPointer<AACTPlayerController>) -> Option<Claim> {
        let mut state = self.state.write().ok()?;

        if player_controller.as_ref().take_if(|it| !it.b_forbid_setting_rotation_from_pawn).is_none() {
            return None;
        }

        // Check if claim already exists
        if state.active_claims.values().any(|it| it.player_id == player_id) {
            error!("Tried to acquire new claim even though the controller already had one!");
            return None;
        }

        for member in self.get_available_members() {
            let hero_id = member.get_hero_id().to_string().unwrap_or_default();

            // If no one else has claimed this hero...
            if state.active_claims.get(&hero_id).is_none() {
                // ...and the hero is  ai-controlled...
                if let Some(ai_controller) = member.controller.as_ref()?.cast::<AActAIController>() {
                    // ...give it to the claimer!
                    let claim = Claim {
                        player_id,
                        hero_id: hero_id.clone(),
                        character: if state.enabled { Some(member.deref().into()) } else { None },
                        player_controller,
                        ai_controller: ai_controller.into(),
                    };

                    info!("Player {} claimed character {}", claim.player_id, claim.hero_id);
                    state.active_claims.insert(hero_id, claim.get_ref());

                    if state.enabled {
                        self.ensure_claim(&claim, member);
                    }

                    return Some(claim);
                }
            }
        }

        None
    }

    /// Renews a claim to a character.
    pub fn renew_claim(&self, claim: Claim) -> Option<Claim> {
        if let Some(mut claim) = self.sanity_check(claim) {
            let matching_member = self.get_available_members().into_iter().find(|it| it.get_hero_id().to_string().unwrap_or_default() == claim.hero_id);
            if let Some(member) = matching_member {
                // Only keep a reference to the character when we're enabled
                if self.state.read().map(|it| it.enabled).unwrap_or_default() {
                    claim.character = Some(member.into());

                    self.ensure_claim(&claim, member);
                } else {
                    if let Some(character) = claim.character.take().and_then(|it| it.try_get().ok()) {
                        self.ensure_evicted(&claim, character);
                    }
                }

                Some(claim)
            } else {
                // Team member does no longer exist, so evict
                self.return_claim(claim);
                None
            }
        } else {
            None
        }
    }

    /// Yields the claim to a character.
    pub fn return_claim(&self, claim: Claim) {
        info!("Player {} yielded control of {}", claim.player_id, claim.hero_id);

        let mut state = self.state.write().expect("Unable to write to state");
        state.active_claims.remove(&claim.hero_id);

        if let Some(character) = claim.character.clone().and_then(|it| it.try_get().ok()) {
            self.ensure_evicted(&claim, character);
        } else {
            info!("...but they are not in control of any?");
        }
    }

    fn sanity_check(&self, claim: Claim) -> Option<Claim> {
        if let (Some(expected_character), Some(player_controller)) = (&claim.character, claim.player_controller.as_ref()) {
            let actual_character = &player_controller.pawn;
            if !expected_character.is_same(actual_character) && actual_character.as_ref().is_some() {
                // Not controlling who we are supposed to be controlling!

                if let Some(actual_character_ref) = actual_character.as_ref().and_then(|it| it.cast::<APyCharBase>()) {
                    info!("Player is controlling wrong character... returning it to AI!");
                    self.ensure_evicted(&claim, actual_character_ref);
                }

                info!("Forcefully returned claim");
                let mut state = self.state.write().expect("Unable to write to state");
                state.active_claims.remove(&claim.hero_id);
                return None;
            }
        }

        Some(claim)
    }

    fn ensure_claim(&self, claim: &Claim, character: &APyCharBase) -> Option<()> {
        if claim.character.is_none() {
            return self.ensure_evicted(claim, character);
        }

        let mut player_controller_ref = claim.player_controller.clone();
        let player_controller = player_controller_ref.as_mut()?;
        self.transfer_control(character, player_controller, |it| it.as_ref().map(|it| !it.is_player_controller()).unwrap_or_default())
    }

    fn ensure_evicted(&self, claim: &Claim, character: &APyCharBase) -> Option<()> {
        let mut ai_controller_ref = claim.ai_controller.clone();
        let ai_controller = ai_controller_ref.as_mut().take_if(|it| it.is_valid()).with_log_if_none("No AI controller found in claim!")?;

        self.transfer_control(character, ai_controller, |it| it.is_same(&claim.player_controller))
    }

    fn transfer_control(&self, next_target: &APyCharBase, target_controller: &mut AController, source_controller_check: impl FnOnce(UObjectPointer<AController>) -> bool) -> Option<()> {
        let is_bot = target_controller.is_a(AAIController::static_class());

        if next_target.controller != target_controller.deref().as_pointer() && source_controller_check(next_target.controller.clone()) {
            let game_state = UWorld::get_world()
                .and_then(|world| UGameplayStatics::get_game_state(world).try_get().ok())
                .and_then(|it| it.cast::<AActGameState>())?;

            let mut curr_target_state_ref = target_controller.player_state.clone();
            let mut next_target_state_ref = next_target.player_state.clone();

            if let (Some(curr_target_state), Some(next_target_state)) = (curr_target_state_ref.clone().try_get().ok(), next_target_state_ref.clone().try_get().ok()) {
                info!("Possessing character {} (is_bot={is_bot}) ({} -> {})", next_target.name.to_string().unwrap_or_default(),
                    curr_target_state.name().to_string(),
                    next_target_state.name().to_string()
                );

                USakuraBlueprintFunctionLibrary::exchange_player_state_unique_id(
                    curr_target_state,
                    next_target_state,
                );

                let other_controller = next_target.controller.clone();
                target_controller.player_state = next_target_state_ref.clone();
                target_controller.possess(next_target);

                if let Some(ctrl) = other_controller.clone().try_get().ok() {
                    ctrl.player_state = curr_target_state_ref.clone();
                    game_state.un_register_player(curr_target_state)
                }

                if let Some(mut_state) = next_target_state_ref.as_mut() {
                    mut_state.bit_set_b_is_a_bot(is_bot)
                }

                if is_bot {
                    game_state.un_register_player(next_target_state);
                } else {
                    game_state.register_player(next_target_state);
                }

                if let Some(ai_controller) = target_controller.cast::<AActAIController>() {
                    ai_controller.register_crowd_agent();
                }
            } else {
                warn!("Unable to get states!");
            }
        }

        Some(())
    }
}