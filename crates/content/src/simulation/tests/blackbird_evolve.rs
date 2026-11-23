use super::*;
use serde_json::{Value, json};

fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    for id in [
        crate::HypothesisId::BlackbirdOrdinaryClockV1,
        crate::HypothesisId::BlackbirdOrdinaryMaliceV1,
        crate::HypothesisId::BlackbirdCoordinatedBirdV1,
        crate::HypothesisId::BlackbirdOrdinarySecondaryV1,
        crate::HypothesisId::BlackbirdQualifiedSupportV1,
        crate::HypothesisId::BlackbirdPrimaryFormV1,
        crate::HypothesisId::BlackbirdSkillBurstPacketsV1,
        crate::HypothesisId::BlackbirdNormalChainV1,
        crate::HypothesisId::BlackbirdEvolveActionsV1,
    ] {
        req.model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actors[0].purchased_skill_level = 8;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.hp = 10000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.relative_duration_us = 35_000_000;
    for target_index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = target_index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
        target.level = 80;
        target.resistance = Ratio::zero();
    }
    req
}

fn burst(time: u64) -> GameActionCommand {
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

fn evolve_melee(time: u64) -> GameActionCommand {
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

fn evolve_skill(time: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "GA_BlackBird_Evolve_Skill".into(),
        damage_effect_id: "GE_Player_BlackBird_Evolve_Skill_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 894_130,
        cooldown_us: 1_000_000,
        hit_delay_us: 100,
    }
}

fn evolve_ultraskill(time: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: "GA_BlackBird_Evolve_UltraSkill".into(),
        damage_effect_id: "GE_Player_BlackBird_Evolve_UltraSkill_Damage".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 4_083_334,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn run(req: &GameScenarioRequest) -> Value {
    run_game_scenario(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn blackbird_evolve_melee_chain_multiplicities_and_extra_packets() {
    let mut req = request(1);
    // Cast UltraSkill at 0 (takes 100 energy, activates transformation form).
    // Burst primary packets land around 3.2-5.0s, recovery ends around 5.1s.
    // Evolve melee stages:
    // Stage 1: t = 6_000_000
    // Stage 2: t = 7_500_000
    // Stage 3: t = 9_000_000
    // Stage 4: t = 11_500_000
    // Stage 5: t = 13_500_000
    req.actions = vec![
        burst(0),
        evolve_melee(6_000_000),
        evolve_melee(7_500_000),
        evolve_melee(9_000_000),
        evolve_melee(11_500_000),
        evolve_melee(13_500_000),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    let hits = report["hit_receipts"].as_array().unwrap();

    // Verify multiplicities of primary packets: [2, 5, 6, 1, 6]
    let m1_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee1_Damage")
        .collect();
    let m2_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee2_Damage")
        .collect();
    let m3_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee3_Damage")
        .collect();
    let m4_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee4_Damage")
        .collect();
    let m5_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee5_Damage")
        .collect();

    assert_eq!(m1_hits.len(), 2, "Stage 1 must have 2 primary hits");
    assert_eq!(m2_hits.len(), 5, "Stage 2 must have 5 primary hits");
    assert_eq!(m3_hits.len(), 6, "Stage 3 must have 6 primary hits");
    assert_eq!(m4_hits.len(), 1, "Stage 4 must have 1 primary hit");
    assert_eq!(m5_hits.len(), 6, "Stage 5 must have 6 primary hits");

    // Total primary hits = 20
    assert_eq!(
        m1_hits.len() + m2_hits.len() + m3_hits.len() + m4_hits.len() + m5_hits.len(),
        20
    );

    // Each primary hit spawns an extra packet consuming Malice
    let m1_extra: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee1_Extra_Damage")
        .collect();
    let m2_extra: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee2_Extra_Damage")
        .collect();
    let m3_extra: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee3_Extra_Damage")
        .collect();
    let m4_extra: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee4_Extra_Damage")
        .collect();
    let m5_extra: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee5_Extra_Damage")
        .collect();

    assert_eq!(m1_extra.len(), 2);
    assert_eq!(m2_extra.len(), 5);
    assert_eq!(m3_extra.len(), 6);
    assert_eq!(m4_extra.len(), 1);
    assert_eq!(m5_extra.len(), 6);

    // Malice spent: 2*0.541591 + 5*0.369949 + 6*0.349951 + 1*2.258020 + 6*0.784891
    // = 1.083182 + 1.849745 + 2.099706 + 2.258020 + 4.709346 = 11.999999
    // Starting Malice: 20 -> remaining = 20 - 11.999999 = 8.000001
    let malice = &report["final_actor_states"][0]["fractional_resources"]["blackbird.malice"];
    assert_eq!(malice["current"], json!("8000001/1000000"));
    assert_eq!(malice["spent"], json!("11999999/1000000"));
}

#[test]
fn blackbird_evolve_skill_permission_requirement_consumption_and_a2_heal() {
    let mut req = request(1);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2); // A2 enabled
    req.actors[0].initial_hp = 5000;
    req.actors[0].static_stats.hp = 10000;
    req.actors[0].static_stats.base_atk = 1000;

    // Melee stages 1..5, then Evolve Skill at 15_000_000 (after Melee5 recovery finishes at ~14.68s)
    req.actions = vec![
        burst(0),
        evolve_melee(6_000_000),
        evolve_melee(7_500_000),
        evolve_melee(9_000_000),
        evolve_melee(11_500_000),
        evolve_melee(13_500_000),
        evolve_skill(15_000_000),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // Evolve Skill has 4 primary hits: GE_Player_BlackBird_Evolve_Skill_Damage
    let hits = report["hit_receipts"].as_array().unwrap();
    let skill_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Skill_Damage")
        .collect();
    assert_eq!(skill_hits.len(), 4, "Evolve Skill delivers 4 hits");

    // Evolve Skill extra packets: GE_Player_BlackBird_Evolve_Skill_Extra_Damage
    // Initial 20 Malice - 11.999999 (Melee 1..5) = 8.000001 remaining.
    // Skill hits spend 3.25 each: hit 1 spends 3.25, hit 2 spends 3.25, hit 3 spends 1.500001,
    // exhausting Malice. Hit 4 spends 0, so exactly 3 extra packets are scheduled.
    let skill_extras: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Skill_Extra_Damage")
        .collect();
    assert_eq!(
        skill_extras.len(),
        3,
        "Evolve Skill spawns 3 extras before Malice is exhausted"
    );

    // A2 heal: 4 * baseATK = 4000. Initial HP = 5000 -> final HP = 9000
    let final_hp = report["final_actor_states"][0]["hp"].as_u64().unwrap();
    assert_eq!(final_hp, 9000, "A2 heals 4 * base_atk = 4000");

    // Evolved Skill shares the Skill talent level: level8 coefficient 1.800.
    // With team ATK buff active, live ATK = 1200. DEF = 0.5.
    // A1 modify_ratio = 1.5 -> multiplier (1 + 1.5) = 2.5.
    // 1200 * 1.8 * 2.5 * 0.5 = 2700.
    let first_skill_hit = skill_hits[0];
    assert_eq!(first_skill_hit["non_crit_damage"], json!("2700/1"));

    // After timing is legal, consumed permission rejects without another hit/heal.
    let mut req_no_perm = req.clone();
    req_no_perm.actions.push(evolve_skill(20_000_000));
    let rejected = run(&req_no_perm);
    assert_eq!(rejected["success"], false);
    assert_eq!(rejected["rejected_action"]["reason"]["kind"], "other");
    assert!(
        rejected["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("requires active skill permission")
    );
    assert!(
        rejected["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["command_index"] != req_no_perm.actions.len() - 1)
    );
    assert_eq!(
        rejected["final_actor_states"][0]["hp"],
        report["final_actor_states"][0]["hp"]
    );
}

#[test]
fn blackbird_evolve_ultraskill_finisher_cost_form_removal_and_a1_spend() {
    // Scenario with initial energy 100:
    // Burst at 0 costs 100, leaving 0 energy. Finisher at 6_000_000 costs 0 energy.
    let mut req = request(1);
    req.actors[0].energy_capacity = 100;
    req.actors[0].initial_energy = 100;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(3); // A3 (A1 + A2 + A3)

    req.actions = vec![burst(0), evolve_ultraskill(6_000_000)];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );

    // 100 energy cost deducted once for burst, 0 for finisher, actor finishes with 0 energy
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["energy"], 0);
    assert_eq!(actor["spent_energy"], 100);

    // 7 primary finisher hits delivered
    let hits = report["hit_receipts"].as_array().unwrap();
    let finisher_hits: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_UltraSkill_Damage")
        .collect();
    assert_eq!(finisher_hits.len(), 7, "Finisher delivers 7 primary hits");

    // The finisher shares the Burst talent level; R3 raises it to level9 (1.666).
    // With R3, resonance_evolved_burst_ratio_modify = 0.5 (1.5x).
    // ATK with team buff = 1200. DEF = 0.5.
    // 1200 * 1.666 * 1.5 * 0.5 = 1499.4
    assert_eq!(finisher_hits[0]["non_crit_damage"], json!("7497/5"));

    // With A1: Hit 1 spends all remaining Malice (starts at 20, spends 20) and spawns exactly 1 extra packet!
    // Hits 2..7 conserve Malice and spawn NO extra packets.
    let finisher_extras: Vec<_> = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_UltraSkill_Extra_Damage")
        .collect();
    assert_eq!(
        finisher_extras.len(),
        1,
        "A1 finisher hit 1 spawns exactly 1 extra packet"
    );

    // Malice fully consumed: initial 20 + 20 (A3 form entry grant) = 40 Malice spent
    let malice = &actor["fractional_resources"]["blackbird.malice"];
    assert_eq!(malice["current"], json!("0/1"));
    assert_eq!(malice["spent"], json!("40/1"));

    // Form was removed at commit: subsequent evolved melee is rejected
    let mut req_post = req.clone();
    req_post.actions.push(evolve_melee(11_000_000));
    let err = run_game_scenario(&req_post, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("require active transformation form"),
        "Evolved melee after finisher must be rejected: {err}"
    );

    // Without A1 (A0 scenario): finisher spends 0 Malice and spawns 0 extras
    let mut req_a0 = req.clone();
    req_a0.actors[0].awakening = ntesim_data::AwakeningSelection::default();
    let report_a0 = run(&req_a0);
    assert_eq!(report_a0["execution_completed"], true);
    let hits_a0 = report_a0["hit_receipts"].as_array().unwrap();
    let extras_a0: Vec<_> = hits_a0
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_BlackBird_Evolve_UltraSkill_Extra_Damage")
        .collect();
    assert_eq!(extras_a0.len(), 0, "A0 finisher spawns 0 extras");
    let malice_a0 = &report_a0["final_actor_states"][0]["fractional_resources"]["blackbird.malice"];
    assert_eq!(malice_a0["current"], json!("20/1"));
}

#[test]
fn blackbird_malice_curves_use_effective_normal_talent_and_individual_a3() {
    for (effects, numerator, denominator) in [
        (vec![3], 194_920_225_673u64, 500_000_000_000u64),
        (vec![1, 2, 3], 42_102_742_749, 100_000_000_000),
        (vec![1, 2, 4], 14_034_247_583, 50_000_000_000),
    ] {
        let mut req = request(1);
        req.actors[0].talent_levels = Some([8, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actions = vec![burst(0), evolve_melee(6_000_000)];
        let report = run(&req);
        let extra = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["damage_effect_id"] == "GE_Player_BlackBird_Evolve_Melee1_Extra_Damage")
            .unwrap();
        // Native spend .541591 times k8+k3_8=.719806, k9+k3_9=.777390,
        // or k9=.518260 when three selected effects omit individual A3.
        assert_eq!(
            extra["scaling_rate"],
            json!(format!("{}/{}", numerator, denominator))
        );
    }
}

#[test]
fn blackbird_malice_extra_shares_captured_partial_spend_across_all_connected_recipients() {
    let mut req = request(3);
    req.actions = vec![
        burst(0),
        evolve_melee(6_000_000),
        evolve_melee(7_500_000),
        evolve_melee(9_000_000),
        evolve_melee(11_500_000),
        evolve_melee(13_500_000),
        evolve_skill(15_000_000),
    ];
    let report = run(&req);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let extras: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .unwrap()
                .ends_with("_Extra_Damage")
        })
        .collect();
    assert_eq!(extras.len(), 69);
    let partial: Vec<_> = extras
        .iter()
        .filter(|hit| hit["time_us"] == 15_516_842)
        .collect();
    assert_eq!(partial.len(), 3);
    for (index, hit) in partial.iter().enumerate() {
        assert_eq!(hit["target_index"], index);
        // Last1.500001 Malice *k8 .479871, captured before the pool reaches0.
        assert_eq!(hit["scaling_rate"], json!("719806979871/1000000000000"));
        assert_eq!(hit["energy_group_id"], partial[0]["energy_group_id"]);
    }
    let actor = &report["final_actor_states"][0];
    assert_eq!(
        actor["fractional_resources"]["blackbird.malice"]["spent"],
        json!("20/1")
    );
    assert_eq!(actor["energy"], 26);
    assert_eq!(actor["charge_remainder"], json!("643/1000"));
}

#[test]
fn blackbird_evolve_rejects_unadmitted_or_unaccepted_actions() {
    // 1. Evolve melee outside of form is rejected
    let mut req_no_form = request(1);
    req_no_form.actions = vec![evolve_melee(0)];
    let err = run_game_scenario(&req_no_form, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("require active transformation form"),
        "Evolve melee without form rejected: {err}"
    );

    // 2. Evolve skill without permission is rejected
    let mut req_no_perm = request(1);
    req_no_perm.actions = vec![burst(0), evolve_skill(10_000_000)];
    let rejected = run(&req_no_perm);
    assert_eq!(rejected["success"], false);
    assert_eq!(rejected["rejected_action"]["reason"]["kind"], "other");
    assert!(
        rejected["halt_diagnostic"]
            .as_str()
            .unwrap()
            .contains("requires active skill permission")
    );

    // 3. Evolve ultraskill outside of form is rejected
    let mut req_no_finisher_form = request(1);
    req_no_finisher_form.actions = vec![evolve_ultraskill(0)];
    let err = run_game_scenario(&req_no_finisher_form, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("require active transformation form"),
        "Evolve ultraskill without form rejected: {err}"
    );

    // 4. Hypothesis not in model assumptions rejects evolved actions
    let mut req_unaccepted = request(1);
    req_unaccepted
        .model_assumptions
        .remove(crate::HypothesisId::BlackbirdEvolveActionsV1.as_str());
    req_unaccepted.actions = vec![evolve_melee(0)];
    let err = run_game_scenario(&req_unaccepted, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("UNSUPPORTED_BLACKBIRD_EVOLVE_INPUT"),
        "Unaccepted evolve hypothesis rejected: {err}"
    );

    // 5. Ordinary melee while form is active is rejected
    let mut req_ordinary_in_form = request(1);
    let mut ordinary_action = super::blackbird_actions::request().actions.remove(0);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut ordinary_action {
        *timestamp_us = 6_000_000;
    }
    req_ordinary_in_form.actions = vec![burst(0), ordinary_action];
    let err = run_game_scenario(&req_ordinary_in_form, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains(
            "ordinary abilities cannot be cast while Blackbird transformation form is active"
        ),
        "Ordinary ability in form rejected: {err}"
    );
}
