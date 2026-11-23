use super::*;

#[test]
fn zankou_two_ordinary_chains_retain_exact_native_cycle_overflow_and_carry() {
    let mut request = super::zankou_actions::request();
    request.relative_duration_us = 15_000_000;
    let input = request.actions[0].clone();
    request.actions = [
        0, 1_084_427, 2_241_797, 3_359_148, 5_337_383, 6_811_396, 7_895_823, 9_053_193, 10_170_544,
        12_148_779,
    ]
    .into_iter()
    .map(|time| {
        let mut input = input.clone();
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut input {
            *timestamp_us = time;
        }
        input
    })
    .collect();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{:?}",
        report["halt_diagnostic"]
    );
    // Independently sum the native per-packet Cycle, counting the two cross
    // cuts and two pierces in stage five: 2.405+1.958+6.193+9.905+14.339=34.8.
    let ledger = &report["final_actor_states"][0]["cycle"];
    assert_eq!(ledger["initial"], 100);
    assert_eq!(ledger["current"], 100);
    assert_eq!(ledger["granted"], 69);
    assert_eq!(ledger["overflow"], 69);
    assert_eq!(ledger["carry"], serde_json::json!("3/5"));
    assert_eq!(ledger["conserved"], true);
}

#[test]
fn zankou_ordinary_skill_cycle_is_packet_scoped_without_invented_periodic_cycle() {
    let mut request = super::zankou_actions::request();
    request.relative_duration_us = 40_000_000;
    for id in [
        crate::HypothesisId::DamageFixedCritRateAsProbabilityV1,
        crate::characters::packet_semantics::ZANKOU_DOT_PACKET_HYPOTHESIS_ID,
    ] {
        request
            .model_assumptions
            .insert(id.into(), id.description().into());
    }
    let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        cooldown_us,
        ..
    } = &mut request.actions[0]
    else {
        unreachable!()
    };
    *ability_id = "GA_Zankou_Skill".into();
    *damage_effect_id = "GE_Player_Zankou_Skill2_Damage".into();
    *cooldown_us = 5_000_000;
    request.targets.push(request.targets[0].clone());
    request.targets[1].target_index = 1;
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{:?}",
        report["halt_diagnostic"]
    );
    // Four*.945+2.52=6.3, independently of AoE recipients.
    let ledger = &report["final_actor_states"][0]["cycle"];
    assert_eq!(ledger["granted"], 6);
    assert_eq!(ledger["carry"], serde_json::json!("3/10"));
    assert_eq!(ledger["conserved"], true);
    assert!(
        !report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hit| hit["damage_effect_id"] == "GE_Player_Zankou_DotDamage")
    );
}
