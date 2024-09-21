import sys

# Set up logging
stdout_file = open('py_output.log', 'w', buffering = 1)
stderr_file = open('py_error.log', 'w', buffering = 1)
sys.stdout = stdout_file
sys.stderr = stderr_file

print("Set up logging")

import unreal_engine as ue
import mod_extensions

# We're redirecting the ue log functions to our internal logging handler
def log(text, severity):
    mod_extensions.log(text, severity)

def log_info(text):
    log(text, 2)

def log_warn(text):
    log(text, 1)

def log_error(text):
    log(text, 0)

ue.log = log_info
ue.log_warning = log_warn
ue.log_error = log_error

from actor.battle_area.act_battle_area import PyActBattleArea
from unreal_engine.classes import PlayerController
from unreal_engine.classes import CommonUtils
from char.py_char_base import PyCharBase

import modding.utils

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