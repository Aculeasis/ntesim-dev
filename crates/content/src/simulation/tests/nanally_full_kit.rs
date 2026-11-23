use super::*;

#[test]
fn nanally_full_kit_equipped_a0_two_continuing_periods_reconcile_all_native_paths() {
    let source = include_str!("../../../../../presets/native/nanally-full-kit.nte");
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(
        result.request.actors[0].weapon_id,
        "weapon:fork_wushoutieyu"
    );
    assert_eq!(result.request.actors[0].awakening.count(), 0);
    assert_eq!(result.report.hits_settled, 275);
    let actor = &result.report.final_actor_states[0];
    assert_eq!(
        (
            actor.initial_energy,
            actor.energy,
            actor.granted_energy,
            actor.spent_energy,
            actor.overflow_energy
        ),
        (100, 100, 252, 200, 52)
    );
    assert_eq!(
        actor.charge_remainder,
        Some(ExactFraction::new(101, 125).unwrap())
    );
    let cycle = actor.cycle.as_ref().unwrap();
    assert_eq!(
        (
            cycle.initial,
            cycle.current,
            cycle.granted,
            cycle.spent,
            cycle.overflow
        ),
        (0, 11, 382, 200, 171)
    );
    assert_eq!(cycle.carry, ExactFraction::new(53, 250).unwrap());
    assert!(cycle.conserved);
    let count = |packet: &str| {
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == packet)
            .count()
    };
    assert_eq!(count("ntesim.reaction.blossom"), 40);
    let off_field_pistil = result
        .report
        .hit_receipts
        .iter()
        .find(|hit| hit.damage_effect_id == "ntesim.reaction.blossom" && hit.time_us == 41_777_110)
        .unwrap();
    assert_eq!(off_field_pistil.actor_index, 0);
    assert_eq!(
        off_field_pistil.damage,
        ExactFraction::new(4500, 1).unwrap()
    ); // Level80 base9000, CI0, DEF1/2.
    assert!(
        result
            .report
            .trace
            .iter()
            .any(|event| event.time_us == 41_777_110
                && event.active_actor == 1
                && event.hit_index.is_some())
    );
    // Authority retains its World-time follow-up ICD through the next
    // cinematic and the second paid Blossom handoff.
    assert_eq!(count("GE_Nanally010_Lv1_Damage"), 10);
    assert_eq!(count("GE_Nanally010_Lv3_Damage"), 0);
    assert_eq!(count("GE_Player_Nanally_Skill1_Damage"), 15);
    assert_eq!(count("GE_Player_Nanally_UltraSkill1_Damage"), 2);
    assert_eq!(count("GE_Player_Nanally_UltraSkill2_Damage"), 10);
    assert_eq!(count("GE_Player_Nanally_UltraSkill3_Damage"), 2);
    assert_eq!(count("GE_Player_Nanally_AirAttack1_Damage"), 2);
    assert_eq!(count("GE_Player_Nanally_BranchPraojectile_Damage"), 4);
    assert_eq!(count("GE_Player_Nanally_PerfectEvadeAttack_Damage"), 2);
    assert_eq!(count("GE_Player_Nanally_QTE1_Damage"), 2);
    assert_eq!(count("GE_Player_Nanally_Cat_AirAttack_Damage"), 2);
    assert_eq!(count("GE_Player_Nanally_Cat_Skill_Damage"), 10);
    assert_eq!(count("GE_Player_Nanally_Cat_Branch5_Damage"), 6);
    assert_eq!(
        result
            .report
            .trace
            .iter()
            .filter(|event| event.event_kind == "raging_flames_window")
            .count(),
        2
    );
    assert!(
        result
            .report
            .trace
            .iter()
            .any(|event| event.event_kind == "raging_flames_skill_stack")
    );
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        report["combat_stamina"]["total_spent"],
        serde_json::json!("60/1")
    );
    assert_eq!(
        report["combat_stamina"]["current"],
        serde_json::json!("0/1")
    );
    super::assert_native_replay(&result);
}
