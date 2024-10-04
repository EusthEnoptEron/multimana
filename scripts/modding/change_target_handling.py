from ability.ability_change_target import PyAbilityChangeTarget
import mod_extensions
import unreal_engine as ue


PyAbilityChangeTarget_do_change_target = PyAbilityChangeTarget.do_change_target
def PyAbilityChangeTarget_do_change_target_override(self, next_target):
    hero_id = next_target.GetHeroID()
    mod_extensions.on_hero_changing(hero_id)
    PyAbilityChangeTarget_do_change_target(self, next_target)

PyAbilityChangeTarget.do_change_target = PyAbilityChangeTarget_do_change_target_override