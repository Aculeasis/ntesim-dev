use super::*;
use crate::runtime_api::{DamageContext, HitModifiers};
use crate::simulation::state::GameSimulatorState;

#[test]
fn blackbird_relative_upgrades_select_primary_packets_and_exclude_extra_siblings() {
    let mut request = super::blackbird_actions::request();
    request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    let state = GameSimulatorState::new(&request);
    let action = super::support::declared_test_action(
        "GA_BlackBird_Evolve_Skill",
        crate::simulation::metadata::ActionCategory::Skill,
    );
    for (ability, primary, extra, relative_numerator, relative_denominator) in [
        (
            "GA_BlackBird_Evolve_Skill",
            "GE_Player_BlackBird_Evolve_Skill_Damage",
            "GE_Player_BlackBird_Evolve_Skill_Extra_Damage",
            3,
            2,
        ),
        (
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
            "GE_Player_BlackBird_SkillAssist_Damage",
            1,
            1,
        ),
        (
            "GA_BlackBird_UltraSkill",
            "GE_Player_BlackBird_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Extra_Damage",
            1,
            1,
        ),
        (
            "GA_BlackBird_Evolve_UltraSkill",
            "GE_Player_BlackBird_Evolve_UltraSkill_Damage",
            "GE_Player_BlackBird_Evolve_UltraSkill_Extra_Damage",
            1,
            2,
        ),
    ] {
        for (packet, expected) in [
            (
                primary,
                ExactFraction::new(relative_numerator, relative_denominator).unwrap(),
            ),
            (extra, ExactFraction::new(0, 1).unwrap()),
        ] {
            let ctx = DamageContext {
                foreign_dot_stacks: 0,
                packet_group_id: None,
                command_index: 0,
                config: &request.actors[0],
                start_fons: None,
                ability_id: ability,
                action_metadata: &action,
                target: &state.targets[0],
                actor: &state.actors[0],
                world_time_us: 0,
                time_us: 0,
                debuff_count: 0,
                damage_effect_id: packet,
                actor_index: 0,
                active_actor_index: 0,
                target_count: 1,
                non_boss_target_count: 1,
                target_index: 0,
                buffs: &state.buffs,
                buff_stats: Default::default(),
                owned_status_counts: &Default::default(),
                team_has_three_elements: false,
            };
            let modifiers = crate::characters::blackbird::awakenings::damage_bonus(
                &ctx,
                HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
            )
            .unwrap();
            assert_eq!(modifiers.ratio_modify, expected, "{packet}");
            assert!(modifiers.ratio_add.is_zero());
            assert!(modifiers.damage_bonus.is_zero());
            assert!(modifiers.final_bonus.is_zero());
        }
    }
}
