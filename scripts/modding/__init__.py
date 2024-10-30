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


# import our overrides
import modding.change_player_handling
import modding.battle_area_handling
import modding.enemy_handling
import modding.contextual_player_controller
import modding.damage_handling
import modding.ring_menu_handling


# act_battle_area_receive_begin_play = PyActBattleArea.ReceiveBeginPlay
# 
# def ActBattleArea_ReceiveBeginPlay(self):
#     ue.log_error('ReceiveBeginPlay')
#     act_battle_area_receive_begin_play(self)
# 
# PyActBattleArea.ReceiveBeginPlay = ActBattleArea_ReceiveBeginPlay
# 
# 
# 
# 
# 
# from x21.ui.ui_change_target import UIChangeTarget
# from unreal_engine.structs import GameplayEventData, GameplayTag
# 
# on_action_input = UIChangeTarget.on_action_input
# 
# def on_action_input_override(self, GameplayTag: GameplayTag, IsPressed: bool):
#     ue.log_error(f'on_action_input: {GameplayTag}, {IsPressed}')
#     on_action_input(self, GameplayTag, IsPressed)
# 
# UIChangeTarget.on_action_input = on_action_input_override
# 
