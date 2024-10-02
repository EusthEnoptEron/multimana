use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use tracing::info;
use manasdk::ai_module::AAIController;
use manasdk::engine::{AController, APawn, APlayerController, UGameplayStatics, UWorld};
use manasdk::py_char_base::APyCharBase;
use manasdk::{AsObjectPointer, HasClassObject, UObjectPointer};
use manasdk::engine_settings::{ETwoPlayerSplitScreenType, UGameMapsSettings};
use manasdk::x21::{AACTPlayerController, AActAIController, AActGameState, ACharacterBase, USakuraBlueprintFunctionLibrary};
use manasdk::x21_game_mode::APyX21GameMode;

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

        if player_controller.as_ref().is_none() {
            return None;
        }

        // Check if claim already exists
        if state.active_claims.values().any(|it| it.player_id == player_id) {
            return None;
        }

        for member in self.get_available_members() {
            let hero_id = member.get_hero_id().to_string().unwrap_or_default();

            if state.active_claims.get(&hero_id).is_none() {
                if let Some(ai_controller) = member.controller.as_ref()?.cast::<AActAIController>() {
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
    pub fn renew_claim(&self, mut claim: Claim) -> Option<Claim> {
        let matching_member = self.get_available_members().into_iter().find(|it| it.get_hero_id().to_string().unwrap_or_default() == claim.hero_id);
        if let Some(member) = matching_member {
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
        }
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
        let ai_controller = ai_controller_ref.as_mut()?;

        self.transfer_control(character, ai_controller, |it| it.is_same(&claim.player_controller))
    }

    fn transfer_control(&self, character: &APyCharBase, target_controller: &mut AController, source_controller_check: impl FnOnce(UObjectPointer<AController>) -> bool) -> Option<()> {
        let is_bot = target_controller.is_a(AAIController::static_class());

        if character.controller != target_controller.deref().as_pointer() && source_controller_check(character.controller.clone()) {
            let game_state = UWorld::get_world()
                .and_then(|world| UGameplayStatics::get_game_state(world).try_get().ok())
                .and_then(|it| it.cast::<AActGameState>())?;

            let controller_state_opt = target_controller.player_state.as_ref();
            let character_state_opt = character.player_state.as_ref();

            if let (Some(controller_state), Some(character_state)) = (controller_state_opt, character_state_opt) {
                info!("Possessing character {}", character.name.to_string().unwrap_or_default());
                USakuraBlueprintFunctionLibrary::exchange_player_state_unique_id(
                    controller_state,
                    character_state,
                );

                target_controller.player_state = character_state.into();
                target_controller.possess(character);

                if let Ok(mut_state) = UObjectPointer::from(character_state).try_get() {
                    mut_state.bit_set_b_is_a_bot(is_bot)
                }

                if is_bot {
                    game_state.un_register_player(character_state);
                } else {
                    game_state.register_player(character_state);
                }
            }
        }

        Some(())
    }
}