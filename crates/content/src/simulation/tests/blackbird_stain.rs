use super::*;
use serde_json::json;

#[test]
fn blackbird_b2_stain_has_additive_base_live_damage_actor_ci_and_twenty_second_expiry() {
    for element in ["PSYCHE", "LAKSHANA", "NATURE", "PSYCHICALLY"] {
        let mut request = super::stain::fixture(&[2, 12_000_000, 19_999_999, 20_000_000], element);
        request.actors[0].character_level = 80;
        request.actors[0].character_max_level = Some(80);
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        // E(108)=1.075, (1.2+.1)*1.075=1.3975, independently of applier CI999.
        let boosted = if matches!(element, "PSYCHE" | "LAKSHANA") {
            json!("2795/2")
        } else {
            json!("1000/1")
        };
        let hits = report["hit_receipts"].as_array().unwrap();
        assert_eq!(hits.len(), 4);
        for hit in &hits[..3] {
            assert_eq!(hit["non_crit_damage"], boosted);
        }
        assert_eq!(hits[3]["non_crit_damage"], json!("1000/1"));
        assert_eq!(
            report["resolved_model_assumptions"]["blackbird.b2_stain_v1"],
            crate::HypothesisId::BlackbirdB2StainV1.description()
        );
    }
}

#[test]
fn blackbird_b2_stain_refresh_replaces_expiry_and_preserves_captured_bonus_after_source_defeat() {
    let mut request =
        super::stain::fixture(&[11_000_000, 20_000_000, 29_999_999, 30_000_000], "PSYCHE");
    request.relative_duration_us = 31_000_000;
    request.actors[0].character_level = 80;
    request.actors[0].character_max_level = Some(80);
    request.actors[1].initial_cycle = 100;
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut request.actions[1] {
        *timestamp_us = 10_000_000;
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    for hit in &hits[..3] {
        assert_eq!(hit["non_crit_damage"], json!("2795/2"));
    }
    assert_eq!(hits[3]["non_crit_damage"], json!("1000/1"));

    // Keep Blackbird off field and defeat him after Stain creation; the captured
    // target status remains. The damaging Hathor has CI0, so the factor is1.3.
    request = super::stain::fixture(&[2_000_000], "PSYCHE");
    request.actors[0].character_level = 80;
    request.actors[0].character_max_level = Some(80);
    request.actors[1].static_stats.cycle_intensity = Ratio::zero();
    request.actions.remove(1);
    let metadata = request.action_metadata.remove(&2).unwrap();
    request.action_metadata.insert(1, metadata);
    if let GameActionCommand::InvokeAbility { actor_index, .. } = &mut request.actions[1] {
        *actor_index = 1;
    }
    request.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 1_000_000,
        actor_index: 0,
        delivered_damage: request.actors[0].initial_hp,
    });
    let incoming = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    request
        .model_assumptions
        .insert(incoming.as_str().into(), incoming.description().into());
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1300/1")
    );
}

#[test]
fn blackbird_b2_stain_new_generation_after_passive_source_defeat_uses_ordinary_policy() {
    let mut request = super::stain::fixture(&[11_000_000, 21_999_999, 22_000_000], "PSYCHE");
    request.actors[0].character_id = "character:1020".into();
    request.actors[1].initial_cycle = 100;
    let mut blackbird = request.actors[0].clone();
    blackbird.slot = 2;
    blackbird.character_id = "character:1042".into();
    blackbird.character_level = 80;
    blackbird.character_max_level = Some(80);
    request.actors.push(blackbird);
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut request.actions[1] {
        *timestamp_us = 10_000_000;
    }
    let incoming = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    request
        .model_assumptions
        .insert(incoming.as_str().into(), incoming.description().into());
    request.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 1_000_000,
        actor_index: 2,
        delivered_damage: request.actors[2].initial_hp,
    });
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_actions"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 3);
    // The second application at10s replaces both1.3 and the old20s deadline.
    for hit in &hits[..2] {
        assert_eq!(hit["non_crit_damage"], json!("1290/1"));
    }
    assert_eq!(hits[2]["non_crit_damage"], json!("1000/1"));
}
