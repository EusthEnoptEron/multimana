from actor.battle_area.act_battle_area import PyActBattleArea
from unreal_engine.classes import PlayerController
from unreal_engine.classes import CommonUtils
from char.py_char_base import PyCharBase

from modding import utils

get_local_controlled_pawn_original = CommonUtils.GetLocalControlledPawn

def is_local_controlled(self, actor):
    return actor.Controller.is_a(PlayerController)


def get_local_controlled_pawn(world):
    match = utils.search_stack_for_parameter(lambda x: x and hasattr(x, 'is_a') and (x.is_a(PlayerController) or x.is_a(PyCharBase)))
    if match is not None:
        return match

    return get_local_controlled_pawn_original(world)


# CommonUtils.GetLocalControlledPawn = get_local_controlled_pawn
PyActBattleArea.is_local_controlled = is_local_controlled
