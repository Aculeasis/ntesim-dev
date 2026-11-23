use super::*;

#[test]
fn zankou_unbound_skill_variant_fails_visibly() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.stochastic_crit = None;

    req.actors[0].character_id = "character:1036".into();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].gear_procs.clear();
    for id in [
        crate::HypothesisId::DamageFixedCritRateAsProbabilityV1,
        crate::characters::zankou::forms::MODEL,
        crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID,
    ] {
        req.model_assumptions
            .insert(id.into(), id.description().into());
    }

    req.action_metadata.clear();
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Zankou_Melee".into(),
            damage_effect_id: "GE_Player_Zankou_Melee1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_000_000,
            actor_index: 0,
            ability_id: "GA_Zankou_Skill".into(),
            damage_effect_id: "GE_Player_Zankou_Skill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 2_000_000,
            actor_index: 0,
            ability_id: "GA_Zankou_UltraSkill".into(),
            damage_effect_id: "GE_Player_Zankou_ForceUltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 20_000_000,
            hit_delay_us: 100_000,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 3_000_000,
            actor_index: 0,
            ability_id: "GA_Zankou_Skill".into(),
            damage_effect_id: "GE_Player_Zankou_Skill4_2_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 500_000,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
    ];
    req.relative_duration_us = 6_000_000;
    let error = run_game_scenario(&req, &limits).unwrap_err();
    assert!(
        error.contains("unsupported Zankou Skill variant"),
        "{error}"
    );
}

#[test]
fn zankou_resonance_6_refreshes_only_after_connected_hits_and_expires_at_twenty_seconds() {
    let run = |nodes: &str| {
        let source = format!(
            "options duration=45 crit=fixed;zankou char lvl=80/80 awakening={nodes} talent=1,1,1 qte=1;target hp=100000000 def=0 resist=0;active zankou;zankou attack;zankou attack;wait(20);zankou attack;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &Default::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.execution_completed,
            "{:?}",
            result.report.halt_diagnostic
        );
        super::super::assert_native_replay(&result);
        result
    };
    let baseline = run("[1,2,3,4,5]");
    let enhanced = run("[1,2,3,4,5,6]");
    assert_eq!(baseline.report.hit_receipts.len(), 5);
    assert_eq!(enhanced.report.hit_receipts.len(), 5);
    for (i, (a, b)) in baseline
        .report
        .hit_receipts
        .iter()
        .zip(&enhanced.report.hit_receipts)
        .enumerate()
    {
        let factor = ExactFraction::new(if i == 0 || i == 3 { 5 } else { 7 }, 5).unwrap();
        assert_eq!(b.non_crit_damage, a.non_crit_damage.mul(factor).unwrap());
    }
    let value = serde_json::to_value(&enhanced.report).unwrap();
    assert!(
        crate::simulation::tests::support::final_buffs(&value)
            .iter()
            .all(|buff| buff["id"] != "zankou_resonance_6_atk")
    );
}
