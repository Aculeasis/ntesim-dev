use super::blackbird_secondary::{dash, stamina};
use super::*;
use crate::characters::blackbird::secondary::{EVOLVE_PLUNGE, EVOLVE_RIPOSTE};
use crate::characters::blackbird::support::{EVOLVED_EXTRA_PACKET, EVOLVED_PACKET};
use serde_json::{Value, json};

fn models(req: &mut GameScenarioRequest) {
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdCoordinatedBirdV1,
        crate::HypothesisId::BlackbirdOrdinarySecondaryV1,
        crate::HypothesisId::BlackbirdEvolvedSecondaryV1,
        crate::HypothesisId::BlackbirdQualifiedSupportV1,
        crate::HypothesisId::BlackbirdEvolvedSupportV1,
        crate::HypothesisId::BlackbirdPrimaryFormV1,
        crate::HypothesisId::BlackbirdEvolveActionsV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
}

pub(super) fn request() -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    req.actions.clear();
    models(&mut req);
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.relative_duration_us = 20_000_000;
    req
}

pub(super) fn burst(time: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "GA_BlackBird_UltraSkill".into(),
        damage_effect_id: "GE_Player_BlackBird_UltraSkill_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 100,
        recovery_us: 5_085_206,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

pub(super) fn evolve_melee(time: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "GA_BlackBird_Evolve_Melee".into(),
        damage_effect_id: "GE_Player_BlackBird_Evolve_Melee1_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn plunge(req: &mut GameScenarioRequest, time: u64) {
    let index = req.actions.len();
    let mut action = evolve_melee(time);
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut action
    {
        *damage_effect_id = EVOLVE_PLUNGE.into();
    }
    req.actions.push(action);
    req.plunge_inputs.insert(
        index,
        crate::simulation::GamePlungeInput {
            landing_delay_us: 600_000,
            height_bonus: Ratio {
                numerator: 1,
                denominator: 2,
            },
        },
    );
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

pub(super) fn hits<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn blackbird_evolved_plunge_uses_form_family_height_and_owner_clock() {
    for targets in [1, 3] {
        let mut req = request();
        req.actions = vec![burst(0)];
        for target_index in 1..targets {
            let mut target = req.targets[0].clone();
            target.target_index = target_index;
            req.targets.push(target);
        }
        for target in &mut req.targets {
            target.hp_current = 100_000_000;
            target.hp_max = 100_000_000;
        }
        plunge(&mut req, 6_000_000);
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let evolved = hits(&report, EVOLVE_PLUNGE);
        assert_eq!(evolved.len(), targets);
        for hit in &evolved {
            assert_eq!(hit["time_us"], 6_700_643);
            // Team ATK 1200 from the transformed Burst: .560*1.5*1200*.5 =504.
            assert_eq!(hit["non_crit_damage"], json!("504/1"));
            let meta = support::hit_metadata(&report, hit);
            assert_eq!(
                meta["binding"]["packet_ga_name"],
                "GA_BlackBird_Evolve_Melee"
            );
            assert_eq!(
                meta["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X"
            );
        }
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["granted_energy"], 1);
        assert_eq!(actor["charge_remainder"], json!("17/25"));
        assert_eq!(actor["cycle"]["carry"], json!("3/10"));
        // Form suppresses ordinary Malice gains; the evolved plunge spends .933204.
        assert_eq!(
            actor["fractional_resources"]["blackbird.malice"]["current"],
            json!("4766699/250000")
        );

        req.actions.push(evolve_melee(7_009_948));
        assert_eq!(run(&req)["execution_completed"], false);
        req.actions[2] = evolve_melee(7_009_949);
        assert_eq!(run(&req)["execution_completed"], true);
    }
}

#[test]
fn blackbird_evolved_plunge_requires_form_and_the_evolved_binding() {
    let mut req = request();
    plunge(&mut req, 0);
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("require active transformation form"),
        "evolved plunge outside form: {err}"
    );

    req = request();
    req.actions = vec![burst(0)];
    plunge(&mut req, 6_000_000);
    req.model_assumptions
        .remove(crate::HypothesisId::BlackbirdEvolvedSecondaryV1.as_str());
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("evolved_secondary_v1"),
        "unaccepted evolved secondary: {err}"
    );
}

#[test]
fn blackbird_evolved_riposte_is_paid_once_and_spends_exact_malice() {
    let mut req = request();
    stamina(&mut req, 100);
    req.actions = vec![burst(0), dash(6_000_000, true), evolve_melee(7_004_760)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ripostes = hits(&report, EVOLVE_RIPOSTE);
    assert_eq!(ripostes.len(), 1);
    assert_eq!(ripostes[0]["time_us"], 7_230_719);
    // 1.260*1200*.5 =756 with the transformed Burst's team ATK.
    assert_eq!(ripostes[0]["non_crit_damage"], json!("756/1"));
    let riposte_meta = support::hit_metadata(&report, ripostes[0]);
    assert_eq!(riposte_meta["action"]["input_role"], "critical_riposte");
    assert_eq!(
        riposte_meta["binding"]["packet_ga_name"],
        "GA_BlackBird_PerfectEvade"
    );
    let extras = hits(
        &report,
        crate::characters::blackbird::secondary::EVOLVE_RIPOSTE_EXTRA,
    );
    assert_eq!(extras.len(), 1);
    assert_eq!(extras[0]["time_us"], 7_297_719);
    // k(level1)=.28 on the exact2.099708 spend at1200 ATK.
    assert_eq!(extras[0]["non_crit_damage"], json!("11023467/31250"));
    let actor = &report["final_actor_states"][0];
    assert_eq!(
        actor["fractional_resources"]["blackbird.malice"]["current"],
        json!("4475073/250000")
    );
    assert_eq!(actor["granted_energy"], 2);
    assert_eq!(actor["charge_remainder"], json!("13/25"));
    assert_eq!(actor["cycle"]["carry"], json!("1/2"));

    // contact+.130s pause extends the .839275s recovery by .1287s.
    req.actions.push(evolve_melee(7_972_734));
    let rejected = run(&req);
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 7_972_735);
    req.actions[3] = evolve_melee(7_972_735);
    assert_eq!(run(&req)["execution_completed"], true);
}

#[test]
fn blackbird_evolved_riposte_needs_a_critical_dodge_and_its_model() {
    let mut req = request();
    stamina(&mut req, 100);
    req.actions = vec![burst(0), dash(6_000_000, false), evolve_melee(7_004_760)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(hits(&report, EVOLVE_RIPOSTE).is_empty());
    // Stage1 of the evolved Normal chain carries its native multiplicity of2.
    assert_eq!(
        hits(&report, "GE_Player_BlackBird_Evolve_Melee1_Damage").len(),
        2
    );

    let mut req = request();
    stamina(&mut req, 100);
    req.actions = vec![burst(0), dash(6_000_000, true), evolve_melee(7_004_760)];
    req.model_assumptions
        .remove(crate::HypothesisId::BlackbirdEvolvedSecondaryV1.as_str());
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("evolved_secondary_v1"),
        "unaccepted evolved riposte: {err}"
    );
}

fn qte_request() -> GameScenarioRequest {
    let mut req =
        super::reactions::request(&[(0, "character:1004", 100, 0), (1, "character:1042", 0, 0)]);
    req.active_actor_index = 1;
    req.relative_duration_us = 30_000_000;
    models(&mut req);
    for actor in &mut req.actors {
        actor.character_level = 80;
        actor.character_max_level = Some(80);
        actor.purchased_skill_level = 8;
        actor.talent_levels = Some([1, 8, 1]);
        actor.weapon_id.clear();
        actor.gear_procs.clear();
        actor.set_id = None;
        actor.initial_energy = 0;
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.crit_rate = Ratio::zero();
        actor.static_stats.elemental_bonus = Ratio::zero();
    }
    req.actors[1].initial_energy = 100;
    req.actors[1].energy_capacity = 100;
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
        target.level = 80;
        target.defense = 1080;
        target.resistance = Ratio::zero();
    }
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 1,
            ability_id: "GA_BlackBird_UltraSkill".into(),
            damage_effect_id: "GE_Player_BlackBird_UltraSkill_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 5_085_206,
            cooldown_us: 0,
            hit_delay_us: 100,
        },
        super::reactions::switch(6_000_000, 0),
        super::reactions::switch(7_000_000, 1),
    ];
    req
}

fn marker_times(report: &Value) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "blackbird_qte_malice")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn blackbird_evolved_qte_requires_form_and_binds_its_native_row_without_b4_marker() {
    let req = qte_request();
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let qte = hits(&report, EVOLVED_PACKET);
    assert_eq!(qte.len(), 1);
    assert_eq!(qte[0]["time_us"], 7_878_181);
    assert_eq!(qte[0]["actor_index"], 1);
    // Support talent8: 3.428*1200*.5 =2056.8.
    assert_eq!(qte[0]["non_crit_damage"], json!("10284/5"));
    let qte_meta = support::hit_metadata(&report, qte[0]);
    assert_eq!(qte_meta["binding"]["packet_ga_name"], "GA_BlackBird_QTE");
    assert_eq!(
        qte_meta["packet"]["native_damage_source_category"],
        "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_H"
    );
    let extras = hits(&report, EVOLVED_EXTRA_PACKET);
    assert_eq!(extras.len(), 1);
    assert_eq!(extras[0]["time_us"], 7_945_181);
    // k(level1)=.28 on the exact1.666435 spend at1200 ATK.
    assert_eq!(extras[0]["non_crit_damage"], json!("6999027/25000"));
    assert!(
        marker_times(&report).is_empty(),
        "the evolved QTE has no ordinary QTE.2 B4 Malice marker"
    );
    let actor = &report["final_actor_states"][1];
    assert_eq!(
        actor["fractional_resources"]["blackbird.malice"]["current"],
        json!("3666713/200000")
    );
    assert_eq!(actor["granted_energy"], 6);
    // No equipped weapon means no per-actor charge ledger on this teammate.
    assert_eq!(actor["charge_remainder"], serde_json::Value::Null);

    // The separate1.266940s switch lock plus the .1287s contact pause.
    let mut locked = req.clone();
    locked.actions.push(super::reactions::switch(8_395_639, 0));
    let report = run(&locked);
    assert_eq!(report["execution_completed"], false);
    assert_eq!(report["rejected_action"]["reason"]["until_us"], 8_395_640);
    locked.actions[3] = super::reactions::switch(8_395_640, 0);
    assert_eq!(run(&locked)["execution_completed"], true);
}

#[test]
fn blackbird_evolved_qte_diagnoses_an_unaccepted_transformed_binding() {
    let mut req = qte_request();
    req.model_assumptions
        .remove(crate::HypothesisId::BlackbirdEvolvedSupportV1.as_str());
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("UNSUPPORTED_BLACKBIRD_EVOLVED_SUPPORT"),
        "unaccepted evolved QTE: {err}"
    );
}

#[test]
fn blackbird_extra_cancels_only_dead_or_unconnected_recipients_without_refunding_malice() {
    let mut req = request();
    req.targets[0].hp_current = 2700;
    let mut living = req.targets[0].clone();
    living.target_index = 1;
    living.hp_current = 100_000_000;
    living.hp_max = 100_000_000;
    let mut invulnerable = living.clone();
    invulnerable.target_index = 2;
    invulnerable.resistance = Ratio::one();
    req.targets.extend([living, invulnerable]);
    req.actions = vec![burst(0), evolve_melee(6_000_000)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extras = hits(&report, "GE_Player_BlackBird_Evolve_Melee1_Extra_Damage");
    assert_eq!(extras.len(), 2);
    for hit in extras {
        assert_eq!(hit["target_index"], 1);
    }
    assert_eq!(
        report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"]["spent"],
        json!("541591/500000")
    );
}
