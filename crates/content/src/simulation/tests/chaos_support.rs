use super::chaos_actions::make_action;
use super::*;
use serde_json::{Value, json};

pub(super) fn request() -> GameScenarioRequest {
    let mut req = super::chaos_clock::request(3);
    let model = crate::HypothesisId::ChaosQualifiedSupportV1;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].purchased_skill_level = 8;
    req.actors[0].talent_levels = Some([1, 8, 1]);
    let mut outgoing = req.actors[0].clone();
    outgoing.slot = 1;
    outgoing.character_id = "character:1042".into();
    outgoing.initial_cycle = 100;
    outgoing.initial_energy = 0;
    req.actors.push(outgoing);
    req.active_actor_index = 1;
    req.actions = vec![super::reactions::switch(0, 0)];
    req.relative_duration_us = 5_000_000;
    req
}
fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    )
}
fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}
fn hits<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn chaos_qualified_support_uses_one_native_packet_no_incoming_energy_and_no_hidden_normal() {
    let report = run(&request());
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 3);
    let qte = hits(&report, "GE_Player_Chaos_QTE1_Damage");
    assert_eq!(qte.len(), 3);
    for hit in &qte {
        assert_eq!(hit["command_index"], 0);
        assert_eq!(hit["time_us"], 704_643);
        assert_eq!(hit["scaling_rate"], json!("857/250"));
        assert_eq!(
            super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_H"
        );
    }
    let owner = &report["final_actor_states"][0];
    assert_eq!(owner["energy"], 6);
    assert_eq!(owner["charge_remainder"], json!("2/5"));
    assert!(owner["cycle"].is_null()); // No Cycle grant/spend enables this optional ledger.
    assert_eq!(
        owner["fractional_resources"]["chaos.crime"]["current"],
        json!("0/1")
    );
    let ally = &report["final_actor_states"][1];
    assert_eq!(ally["cycle"]["current"], 0);
    assert_eq!(ally["energy"], 3);
    assert_eq!(ally["charge_remainder"], json!("21/25"));
}

#[test]
fn chaos_support_completion_routes_one_authored_appear_normal_with_separate_normal_talent() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3);
    req.actions.push(normal(1_376_352));
    req.actions.push(normal(1_895_214));
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        hits(&report, "GE_Player_Chaos_QTE1_Damage")[0]["scaling_rate"],
        json!("857/250")
    );
    let first = hits(&report, "GE_Player_Chaos_Melee1_Damage");
    assert_eq!(first.len(), 3);
    assert_eq!(first[0]["time_us"], 1_594_172);
    assert_eq!(first[0]["scaling_rate"], json!("197/1000"));
    assert_eq!(
        hits(&report, "GE_Player_Chaos_Melee2_Damage")[0]["time_us"],
        2_132_534
    );
    req.actions[1] = normal(1_376_351);
    assert_eq!(
        run(&req)["rejected_action"]["reason"]["until_us"],
        1_376_352
    );
}

#[test]
fn chaos_support_requires_qualified_adjacent_switch_and_never_authored_qte() {
    for (character, cycle) in [("character:1042", 99), ("character:1010", 100)] {
        let mut req = request();
        req.actors[1].character_id = character.into();
        req.actors[1].initial_cycle = cycle;
        req.actions.push(normal(0));
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert!(hits(&report, "GE_Player_Chaos_QTE1_Damage").is_empty());
        assert_eq!(
            hits(&report, "GE_Player_Chaos_Melee1_Damage")[0]["time_us"],
            214_573
        );
        assert!(report["final_actor_states"][1]["cycle"].is_null());
    }
    let mut req = request();
    req.actions = vec![make_action(
        0,
        0,
        "GA_Chaos071_QTE",
        "GE_Player_Chaos_QTE1_Damage",
        0,
        0,
    )];
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("engine-owned")
    );
}

#[test]
fn chaos_interrupted_support_retains_reached_charge_but_cannot_authorize_appear() {
    for (time, qte_hits) in [(300_000, 0), (704_643, 3)] {
        let mut req = request();
        let model = crate::HypothesisId::ChaosForcedInterruptV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.interrupt_events = vec![crate::simulation::GameActorInterruptEvent {
            timestamp_us: time,
            actor_index: 0,
            armor_damage: None,
        }];
        req.actions.push(normal(time + 1));
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(hits(&report, "GE_Player_Chaos_QTE1_Damage").len(), qte_hits);
        assert_eq!(
            hits(&report, "GE_Player_Chaos_Melee1_Damage")[0]["time_us"],
            time + 214_574
        );
        assert!(report["interrupt_receipts"][0]["skill_cooldown_after_us"].is_null());
        assert_eq!(
            report["final_actor_states"][0]["energy"],
            if qte_hits == 0 { 0 } else { 7 }
        );
        assert_eq!(report["final_actor_states"][1]["cycle"]["current"], 0);
    }
}

#[test]
fn chaos_appear_combo_requires_its_quarter_second_hold_and_strict_progress() {
    for (time, duration, complete) in [
        (1_696_153, 200_000, false),
        (1_696_152, 250_000, false),
        (1_696_153, 250_000, true),
    ] {
        let mut req = request();
        for model in [
            crate::HypothesisId::ChaosNeutralVerdictV1,
            crate::HypothesisId::ChaosNormalComboVerdictV1,
        ] {
            req.model_assumptions
                .insert(model.into(), model.description().into());
        }
        req.actions.push(normal(1_376_352));
        req.hold_inputs.insert(
            2,
            crate::simulation::GameHoldInput {
                reserved_stocks: None,
                duration_us: duration,
                moving: false,
            },
        );
        req.actions.push(normal(time));
        let report = run(&req);
        assert_eq!(report["execution_completed"], complete);
        if complete {
            assert_eq!(
                hits(&report, "GE_Player_Chaos_Branch1_Damage")[0]["time_us"],
                1_862_844
            );
        } else if duration == 250_000 {
            assert_eq!(report["rejected_action"]["reason"]["until_us"], 1_696_153);
        }
    }
}

#[test]
fn chaos_appear_permission_ends_on_another_action_or_field_exit_without_resetting_resources() {
    let mut req = request();
    req.actions.extend([
        super::reactions::switch(1_376_352, 1),
        super::reactions::switch(2_000_000, 0),
        normal(2_000_000),
    ]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(hits(&report, "GE_Player_Chaos_QTE1_Damage").len(), 3);
    assert_eq!(
        hits(&report, "GE_Player_Chaos_Melee1_Damage")[0]["time_us"],
        2_214_573
    );
    assert_eq!(report["final_actor_states"][0]["energy"], 7);
    req.actions.truncate(1);
    req.actions.extend([
        make_action(
            1_376_352,
            0,
            "GA_Chaos071_Skill",
            "GE_Player_Chaos_Skill1_Damage",
            0,
            0,
        ),
        normal(2_431_346),
    ]);
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        hits(&report, "GE_Player_Chaos_Melee1_Damage")[0]["time_us"],
        2_645_919
    );
}

#[test]
fn chaos_support_inside_form_keeps_real_expiry_and_next_appear_uses_live_form() {
    for (entry, transformed) in [(6_000_000, true), (16_083_605, false)] {
        let mut req = request();
        let model = crate::HypothesisId::ChaosPrimaryFormV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.active_actor_index = 0;
        req.actors[0].initial_energy = 100;
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Chaos071_UltraSkill",
                "GE_Player_Chaos_UltraSkill1_Damage",
                100,
                0,
            ),
            super::reactions::switch(entry - 1_000_000, 1),
            super::reactions::switch(entry, 0),
            normal(entry + 1_376_352),
        ];
        req.relative_duration_us = 20_000_000;
        let report = run(&req);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            hits(&report, "GE_Player_Chaos_QTE1_Damage")[0]["time_us"],
            entry + 704_643
        );
        let row = if transformed {
            "GE_Player_Chaos_Melee1_Damage_Ultra"
        } else {
            "GE_Player_Chaos_Melee1_Damage"
        };
        assert_eq!(hits(&report, row)[0]["time_us"], entry + 1_594_172);
        let end = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["event_kind"] == "chaos_form_end")
            .unwrap();
        assert_eq!(end["time_us"], 16_802_493);
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chaos.crime"]["granted"],
            json!("1672/1")
        );
    }
}
