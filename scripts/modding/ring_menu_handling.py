from x21.ui.ability_ring_sub_menu import AbilityRingSubMenu
from unreal_engine.classes import CommonUtils
import unreal_engine as ue

get_local_controlled_player_index = AbilityRingSubMenu.get_local_controlled_player_index
def get_local_controlled_player_index_override(self):
    local_pawn = CommonUtils.GetLocalControlledPawn(self)

    for index, id in enumerate(self._AbilityRingSubMenu__team_hero_id_list):
        if id == local_pawn.GetHeroID():
            return index

    return 0

AbilityRingSubMenu.get_local_controlled_player_index = get_local_controlled_player_index_override