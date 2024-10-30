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


CommonUtils__GetLocalPlayerController = CommonUtils.GetLocalPlayerController
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

    return CommonUtils__GetLocalPlayerController(world)

CommonUtils.GetLocalPlayerController = GetLocalPlayerControllerOverride


CommonUtils__GetLocalControlledPawn = CommonUtils.GetLocalControlledPawn
def GetLocalControlledPawnOverride(world):
    try:
        ctrl = CommonUtils.GetLocalPlayerController(world)
        if ctrl is not None:
            pawn = ctrl.Pawn
            if pawn is not None:
                return pawn
    except Exception as e:
        ue.log_error(f"Error while getting local player controller: {e}")

    return CommonUtils__GetLocalControlledPawn(world)

CommonUtils.GetLocalControlledPawn = GetLocalControlledPawnOverride