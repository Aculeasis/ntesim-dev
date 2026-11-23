use super::*;
use serde_json::Value;

fn source() -> &'static str {
    include_str!("../../../../../examples/sakiri-suppression-cycles.nte")
}

#[test]
fn sakiri_suppression_cycles_retain_actual_recipients_origin_and_base_def_reduction() {
    for (awakening, duration_us) in [(0, 3_000_000), (1, 6_000_000)] {
        let src = source().replacen("awakening=0", &format!("awakening={awakening}"), 1);
        let req = crate::dsl::compiler::compile_scenario(&src, Default::default())
            .unwrap()
            .request;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
        let controls: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("control_applications").is_some())
            .collect();
        assert_eq!(controls.len(), 2);
        for hit in controls {
            let applications = hit["control_applications"].as_array().unwrap();
            assert_eq!(applications.len(), 1);
            let applied = &applications[0];
            assert_eq!(applied["kind"], "suppress");
            assert_eq!(applied["source_actor_index"], hit["actor_index"]);
            assert_eq!(applied["target_index"], 0);
            assert_eq!(applied["command_index"], hit["command_index"]);
            assert_eq!(applied["duration_us"], duration_us);
            assert_eq!(
                applied["expires_at_us"],
                hit["time_us"].as_u64().unwrap() + duration_us
            );
        }
        let mut immune = req.clone();
        immune.targets[0].control_immune = true;
        let baseline = run_game_scenario(&immune, &ExecutionLimits::default()).unwrap();
        assert_eq!(baseline["execution_completed"], true);
        let read = |hit: &Value| {
            ExactFraction::from_ratio(
                serde_json::from_value(hit["non_crit_damage"].clone()).unwrap(),
            )
            .unwrap()
        };
        let mut changed = 0;
        let mut changed_break = 0;
        for (hit, bare) in report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .zip(baseline["hit_receipts"].as_array().unwrap())
        {
            if read(hit) != read(bare) {
                assert_eq!(hit["target_index"], 0);
                // K80=1080. Total1500 - .10 * base1000 =1400, not1350.
                assert_eq!(
                    read(hit),
                    read(bare)
                        .mul(ExactFraction::new(129, 124).unwrap())
                        .unwrap()
                );
                changed += 1;
                if hit.get("break_contribution").is_some() {
                    changed_break += 1;
                }
            }
        }
        assert!(changed > 0 && changed_break > 0);
        let statuses = report["final_target_states"][0]["statuses"]
            .as_array()
            .unwrap();
        assert!(
            statuses
                .iter()
                .any(|status| status["id"] == "sakiri.impish_trick_def")
        );
        assert!(
            !statuses
                .iter()
                .any(|status| status["id"] == "ntesim.control.suppress")
        );
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}

#[test]
fn target_def_reduction_uses_base_clamps_and_expires_at_the_real_deadline() {
    use ntesim_core::buffs::dispatch::{
        BuffDefinition, BuffManager, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
    };
    let req = super::sakiri_actions::request();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    let target = &mut state.targets[0];
    target.defense = 1500;
    let mut buffs = BuffManager::default();
    buffs
        .apply_buff(
            BuffDefinition {
                id: "def-test".into(),
                source: "test".into(),
                recipient: BuffRecipient::SpecificTarget(0.into()),
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                semantic_tags: Default::default(),
                duration_us: Some(20_000_000),
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: std::collections::BTreeMap::from([(
                    "def_percent_reduction".into(),
                    Ratio {
                        numerator: 1,
                        denominator: 10,
                    },
                )]),
            },
            0,
            0,
        )
        .unwrap();
    for (base, numerator, denominator) in [
        (None, 1350, 1),
        (Some(1000), 1400, 1),
        (Some(1001), 13999, 10),
        (Some(20000), 0, 1),
    ] {
        target.base_defense = base;
        assert_eq!(
            crate::runtime_api::target_defense(&buffs, target, 0, 19_999_999).unwrap(),
            ExactFraction::new(numerator, denominator).unwrap()
        );
        assert_eq!(
            crate::runtime_api::target_defense(&buffs, target, 0, 20_000_000).unwrap(),
            ExactFraction::new(1500, 1).unwrap()
        );
    }
}

#[test]
fn sakiri_after_hit_control_reaches_zero_rounded_loss_but_not_a_defeated_target() {
    let src = "options duration=2 crit=fixed; sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120; target hp=1000000 def=0 resist=0 control_immune=false; active sakiri; sakiri burst;";
    let mut req = crate::dsl::compiler::compile_scenario(src, Default::default())
        .unwrap()
        .request;
    req.actors[0].static_stats.base_atk = 1;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1,
        denominator: 1000,
    };
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert!(hits.iter().all(|hit| hit["applied_hp_loss"] == 0));
    assert_eq!(
        hits.iter()
            .filter(|hit| hit.get("control_applications").is_some())
            .count(),
        1
    );
    req.targets[0].hp_current = 1;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 10000,
        denominator: 1,
    };
    let fatal = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(
        fatal["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit.get("control_applications").is_none())
    );
}

#[test]
fn control_target_inputs_reject_numeric_booleans_and_fractional_base_def() {
    for source in [
        source().replacen("control_immune=false", "control_immune=1", 1),
        source().replacen("base_def=1000", "base_def=1/2", 1),
    ] {
        assert!(crate::dsl::compiler::compile_scenario(&source, Default::default()).is_err());
    }
}

#[test]
fn sakiri_control_models_require_strict_acceptance_before_committing_the_cast() {
    let mut req = super::sakiri_actions::request();
    req.targets[0].control_immune = false;
    req.actors[0].talent_levels = None;
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut req.actions[0]
    {
        *ability_id = "GA_Sagiri_UltraSkill".into();
        *damage_effect_id = "GE_Player_Sagiri_UltraSkill1_Damage".into();
    }
    for id in [
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
        crate::HypothesisId::SakiriSkillBurstPacketsV1,
        crate::characters::sakiri::BURST_ATK_MODEL,
    ] {
        req.model_assumptions
            .insert(id.into(), id.description().into());
    }
    let id = crate::HypothesisId::SakiriSuppressV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let id = crate::HypothesisId::CombatControlEffectsV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
}
