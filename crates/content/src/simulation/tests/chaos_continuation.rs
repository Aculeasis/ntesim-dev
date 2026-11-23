use super::chaos_actions::{make_action, request};
use super::*;
use serde_json::json;

fn assert_chaos_continuation_damage(report: &serde_json::Value, relative_duration_us: u64) {
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 72);
    let receipts = report["hit_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 72);
    assert_eq!(receipts[0]["time_us"], 0);
    assert_eq!(receipts[64]["time_us"], 21_600_000);
    assert_eq!(
        receipts[65]["damage_effect_id"],
        "GE_Player_Chaos_QTE1_Damage"
    );
    assert_eq!(receipts[65]["time_us"], 22_400_000);
    assert_eq!(receipts[65]["non_crit_damage"], json!("500/1"));
    assert_eq!(
        receipts[66]["damage_effect_id"],
        "GE_Player_Chaos_UltraSkill1_Damage"
    );
    assert_eq!(
        receipts[70]["damage_effect_id"],
        "GE_Player_Chaos_UltraSkill2_Damage"
    );
    assert_eq!(receipts[70]["time_us"], 23_000_000);
    assert_eq!(
        receipts[71]["damage_effect_id"],
        "GE_Player_Chaos_Melee1_Damage"
    );
    assert_eq!(receipts[71]["time_us"], 25_000_000);
    assert_eq!(receipts[71]["non_crit_damage"], json!("1183/10"));
    let total_damage: ExactFraction = receipts
        .iter()
        .try_fold(ExactFraction::new(0, 1).unwrap(), |acc, receipt| {
            acc.add(
                serde_json::from_value::<ExactFraction>(receipt["non_crit_damage"].clone())
                    .map_err(|err| err.to_string())?,
            )
        })
        .unwrap();
    // Independent from the receipt sum: 5 full Lv1 Normal chains (5*1819.5),
    // the first three stages of the sixth (505.5), a Warrant-boosted
    // continuation hit (118.3), the QTE direct packet (500), and five Burst
    // packets (5200.65), at 1000 ATK over the full requested horizon.
    assert_eq!(total_damage, ExactFraction::new(308_439, 20).unwrap());
    assert_eq!(relative_duration_us, 25_200_100);
    let modeled_dps = total_damage
        .div(ExactFraction::new(relative_duration_us as u128, 1_000_000).unwrap())
        .unwrap();
    assert_eq!(
        modeled_dps,
        ExactFraction::new(154_219_500, 252_001).unwrap()
    );
}

fn assert_chaos_continuation_energy(report: &serde_json::Value) {
    let ordinary_grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["energy_fanout"]["source"] == "chaos_hit")
        .collect();
    let qte_grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "chaos_qte_direct_packet_energy_grant")
        .collect();
    assert_eq!(ordinary_grants.len(), 67);
    assert!(ordinary_grants.iter().all(|event| {
        event["energy_fanout"]["recipients"]
            .as_array()
            .unwrap()
            .len()
            == 3
    }));
    assert_eq!(qte_grants.len(), 1);
    let qte_fanout = &qte_grants[0]["energy_fanout"];
    assert_eq!(qte_fanout["source"], "chaos_hit");
    assert_eq!(qte_fanout["base"], json!("32/5"));
    let qte_recipients = qte_fanout["recipients"].as_array().unwrap();
    assert_eq!(qte_recipients.len(), 3);
    assert_eq!(qte_recipients[0]["energy_before"], 96);
    assert_eq!(qte_recipients[0]["energy_after"], 100);
    assert_eq!(qte_recipients[0]["retained"], 4);
    assert_eq!(qte_recipients[0]["overflow"], 2);
    for (recipient, gross) in [(0, 6), (1, 4), (2, 4)] {
        assert_eq!(
            qte_recipients[recipient]["recipient_actor_index"],
            recipient
        );
        assert_eq!(qte_recipients[recipient]["gross"], gross);
    }
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 1);
    assert_eq!(report["final_actor_states"][1]["energy"], 62);
    assert_eq!(report["final_actor_states"][2]["energy"], 62);
}

#[test]
fn chaos_zero_start_qte_burst_and_normal_continuation_reconciles_damage_and_energy() {
    // Twenty-eight bounded Normal inputs yield 65 ordinary packets. Their
    // caller timing follows the authored 800ms input interval; QTE recovery,
    // Burst recovery and the continuation start are scenario timings, not
    // native animation evidence. The exact caller-supplied QTE packet crosses
    // the modeled 100-cost threshold under its separate direct-input hypothesis.
    let mut req = request();
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 100;
    for (slot, id) in [(1, "character:1019"), (2, "character:1003")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = id.into();
        req.actors.push(ally);
    }

    let mut actions: Vec<_> = (0..28)
        .map(|index| {
            make_action(
                index * 800_000,
                0,
                "GA_Chaos071_Melee",
                "GE_Player_Chaos_Melee1_Damage",
                0,
                0,
            )
        })
        .collect();
    let mut qte = make_action(
        22_400_000,
        0,
        "GA_Chaos071_QTE",
        "GE_Player_Chaos_QTE1_Damage",
        0,
        15_000_000,
    );
    let burst = make_action(
        23_000_000,
        0,
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100,
        20_000_000,
    );
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut qte {
        *recovery_us = 500_000;
    }
    let mut burst = burst;
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut burst {
        *recovery_us = 2_000_000;
    }
    actions.extend([qte, burst]);
    actions.push(make_action(
        25_000_000,
        0,
        "GA_Chaos071_Melee",
        "GE_Player_Chaos_Melee1_Damage",
        0,
        0,
    ));
    req.actions = actions;
    for command in &mut req.actions {
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = command {
            *hit_delay_us = 0;
        }
    }
    req.relative_duration_us = 25_200_100;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_chaos_continuation_damage(&report, req.relative_duration_us);
    assert_chaos_continuation_energy(&report);
    assert!(
        report["resolved_model_assumptions"]
            .get("chaos.qte_direct_packet_energy_v1")
            .is_some()
    );
    assert!(
        report["resolved_model_assumptions"]
            .get("chaos.ordinary_hit_energy_v1")
            .is_some()
    );
}
