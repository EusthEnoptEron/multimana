from x21.x21_hud import PyX21Hud
from x21.ui.widget_component_pool import WidgetComponentPool
from x21.x21_game_instance import PyX21GameInstance, GAME_INSTANCE
import unreal_engine as ue
import inspect
from unreal_engine.classes import Actor, WidgetComponentPoolBase, WidgetComponent, HUD

class ProxyWidgetComponentPool(WidgetComponentPool):
    """
    This is a proxy around thw WidgetComponentPool that provides the damage widgets.
    It makes sure that the owner player is correctly set on new widgets.

    I originally wanted to simply extend `WidgetComponentPool` and call it a day, but the version of UEP seems to have
    issues with inheritance... so I'm proxying.
    """
    def __init__(self):
        super().__init__()
        self.proxy = None

    def set_proxy(self, proxy: WidgetComponentPool):
        self.proxy = proxy

    def get_damage_class(self):
        ue.log(f"get_damage_class")

        return self.proxy.get_damage_class()

    def init(self, game_instance):
        ue.log(f"init")

        self.proxy.init(game_instance)

    def ClearPool(self):
        ue.log(f"ClearPool")

        self.proxy.ClearPool()

    def CreatePoolCache(self):
        ue.log(f"CreatePoolCache")

        self.proxy.CreatePoolCache()

    def OnLeavingMatchState(self):
        ue.log(f"OnLeavingMatchState")

        self.proxy.OnLeavingMatchState()

    def Reclaim(self, WidgetComp: WidgetComponent):
        ue.log(f"Reclaim")

        self.proxy.Reclaim(WidgetComp)

    def Acquire(self, OwnedActor: Actor) -> WidgetComponent:
        ue.log_warning(f"acquiring widget {self} / {OwnedActor}")
        widget = None
        try:
            widget = self.proxy.Acquire(OwnedActor)

            prev_frame = inspect.currentframe().f_back
            caller = prev_frame.f_locals['self']

            if caller.is_a(HUD):
                ctrl = caller.PlayerOwner
                widget.SetOwnerPlayer(ctrl.Player)
        except Exception as e:
            ue.log_warning(f"Error occurred: {e}")

        return widget


widget_pool = ProxyWidgetComponentPool()
widget_pool.set_proxy(GAME_INSTANCE.widget_pool)
widget_pool.init(GAME_INSTANCE)
GAME_INSTANCE.widget_pool = widget_pool
