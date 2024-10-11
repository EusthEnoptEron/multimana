from unreal_engine.classes import CommonUtils, HUD, UserWidget
from x21.ui.ui_hud import UIHUD
import unreal_engine as ue

CommonUtils__GetLocalPCHud = CommonUtils.GetLocalPCHud
def CommonUtils__GetLocalPCHud_override(world):
    try:
        if world.is_a(UserWidget):
            ctrl = world.GetOwningPlayer()
            return ctrl.MyHud

        if world.is_a(HUD):
            return world
    except Exception as e:
        ue.log_error(f"Error while getting local player controller: {e}")
    return CommonUtils__GetLocalPCHud(world)
CommonUtils.GetLocalPCHud = CommonUtils__GetLocalPCHud_override


CommonUtils__GetLocalPLayerController = CommonUtils.GetLocalPlayerController
def GetLocalPlayerControllerOverride(world):
    try:
        if world.is_a(UserWidget):
            ctrl = world.GetOwningPlayer()
            return ctrl

        if world.is_a(HUD):
            ctrl = world.PlayerOwner
            return ctrl
    except Exception as e:
        ue.log_error(f"Error while getting local player controller: {e}")

    return CommonUtils__GetLocalPLayerController(world)

CommonUtils.GetLocalPlayerController = GetLocalPlayerControllerOverride