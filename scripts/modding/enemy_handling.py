from char.py_enemy_base import PyEnemyBase
import unreal_engine as ue, utils

from unreal_engine import FVector, FVector2D, FTransform
from unreal_engine.classes import ActorComponentBase, ActBattleArea, Actor, ActorComponent, BehaviorTree, CharacterBase, CollisionImpactComponent, DataTable, GameplayStatics, KismetMathLibrary, KismetSystemLibrary, Pawn, SakuraAbilityTargetInterface, SpringArmComponent, WidgetComponent, CommonUtils, ActAIAutoTracerComponent
from unreal_engine.enums import EAttachmentRule, EDebuffWithAnimType, ECollisionChannel, ECollisionResponse, EDrawDebugTrace, EEndPlayReason, EEnemyStrength, EMovementMode, ESlateVisibility, ETraceTypeQuery, EWidgetSpace, ECollisionEnabled, EAutoPossessAI, EMonsterPerceptionIconType, EVisibilityBasedAnimTickOption, EEnemyStatusType, ECharacterStateType

def create_health_widget_for_player(enemy: PyEnemyBase, player, player_index):
    ue.log_error(f"Creating widget for player {player_index}")
    try:
        health_widget_comp = enemy.AddComponentByClass(WidgetComponent, False, FTransform(), False)
        health_widget_comp.setup_attachment(enemy.HealthWidgetSpringArmComp)
        health_widget_comp.SetDrawSize(FVector2D(400, 100))
        health_widget_comp.SetRelativeLocation(FVector(0, 0, 50))
        health_widget_comp.SetWidgetSpace(EWidgetSpace.Screen)
        health_widget_comp.SetCollisionEnabled(ECollisionEnabled.NoCollision)
        health_widget_comp.SetOwnerPlayer(player)

        return health_widget_comp
    except Exception as e:
        ue.log_error(f"Error while creating hp widget: {e}")


class PyEnemyMultiplayerAdapter(ActorComponentBase):
    enemy: PyEnemyBase

    def __init__(self):
        self.PrimaryComponentTick.bCanEverTick = True
        self.AdditionalHealthWidgetComp = {}

    def PyTickComponent(self, DeltaTime: float):
        base_widget_comp = self.enemy.HealthWidgetComp.GetUserWidgetObject()
        for player_index in range(1, 4):
            player_controller = GameplayStatics.GetPlayerController(self, player_index)
            if player_controller is None:
                if player_index in self.AdditionalHealthWidgetComp:
                    widget_comp = self.AdditionalHealthWidgetComp[player_index]
                    del self.AdditionalHealthWidgetComp[player_index]
                    ue.log_error(f"Removing hp widget {player_index}")
                    widget_comp.DestroyComponent()
                continue

            player = player_controller.Player

            if player_index not in self.AdditionalHealthWidgetComp:
                widget_comp = create_health_widget_for_player(self.enemy, player, player_index)
                self.AdditionalHealthWidgetComp[player_index] = widget_comp


            widget_comp = self.AdditionalHealthWidgetComp[player_index]
            if ue.is_valid(widget_comp):
                healthy_bar_instance = widget_comp.GetUserWidgetObject()
                if not ue.is_valid(healthy_bar_instance) and self.enemy.after_receive_begin:
                    ue.log(f"Init enemy health bar for player {player_index}")
                    game_instance = utils.get_gameinstance(self)
                    widget_class = game_instance.get_user_widget_by_id(self.enemy.get_enemy_hp_type())
                    widget_comp.K2_SetWidgetClass(widget_class)
                    widget_comp.GetUserWidgetObject().InitInfo(self.enemy)
                    healthy_bar_instance = self.enemy.HealthWidgetComp.GetUserWidgetObject()
                if ue.is_valid(healthy_bar_instance) and ue.is_valid(base_widget_comp):
                    healthy_bar_instance.SetVisibility(base_widget_comp.GetVisibility())


PyEnemyBase__bind_events = PyEnemyBase.bind_events
def PyEnemyBase__bind_events__override(self):
    PyEnemyBase__bind_events(self)

    adapter = self.AddComponentByClass(PyEnemyMultiplayerAdapter, False, FTransform(), False)
    adapter.enemy = self

PyEnemyBase.bind_events = PyEnemyBase__bind_events__override