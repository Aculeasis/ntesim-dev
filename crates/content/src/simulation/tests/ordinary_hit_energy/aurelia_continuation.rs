use super::*;

const AURELIA_CHAIN_CHARGE_MILLI: [u64; 9] = [2168, 3936, 1872, 1872, 1872, 620, 620, 620, 620];

#[derive(Default)]
struct AureliaEnergyOracle {
    packet_by_recipient: [usize; 3],
    cumulative: [u64; 3],
    generated: [u64; 3],
    energy: [u64; 3],
    overflow: [u64; 3],
    spent: [u64; 3],
}

fn aurelia_continuing_request() -> GameScenarioRequest {
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1070")
        .unwrap();
    let mut req = request(case, false);
    for actor in &mut req.actors {
        actor.energy_capacity = 120;
    }
    let mut actions = Vec::new();
    for input in 0..36 {
        let mut normal = action("GA_Mitsuki_Melee", "GE_Player_Mitsuki_Melee1_Damage", 0);
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut normal {
            *timestamp_us = input * 500_000;
        }
        actions.push(normal);
    }
    let mut skill = action("GA_Mitsuki_Skill", "GE_Player_Mitsuki_Skill_Damage", 0);
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut skill {
        *timestamp_us = 18_000_000;
    }
    actions.push(skill);
    let mut burst = action(
        "GA_Mitsuki_UltraSkill",
        "GE_Player_Mitsuki_UltraSkill_Damage",
        0,
    );
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        energy_cost,
        ..
    } = &mut burst
    {
        *timestamp_us = 19_000_000;
        *energy_cost = 100;
    }
    actions.push(burst);
    for input in 0..4 {
        let mut normal = action("GA_Mitsuki_Melee", "GE_Player_Mitsuki_Melee1_Damage", 0);
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut normal {
            *timestamp_us = 20_000_000 + input * 500_000;
        }
        actions.push(normal);
    }
    req.actions = actions;
    req.relative_duration_us = 24_000_000;
    req
}

fn aurelia_expected_charge_packets() -> Vec<u64> {
    AURELIA_CHAIN_CHARGE_MILLI
        .into_iter()
        .cycle()
        .take(9 * AURELIA_CHAIN_CHARGE_MILLI.len())
        .chain([6700])
        .chain(AURELIA_CHAIN_CHARGE_MILLI)
        .collect()
}

fn apply_aurelia_cost(change: &Value, oracle: &mut AureliaEnergyOracle) {
    assert_eq!(change["recipient_actor_index"], 0);
    let cost = change["spent"].as_u64().unwrap();
    assert_eq!(change["energy_before"], oracle.energy[0]);
    if cost == 0 {
        assert_eq!(change["energy_after"], oracle.energy[0]);
        return;
    }
    assert_eq!(cost, 100);
    assert_eq!(oracle.energy[0], 120);
    assert_eq!(change["energy_after"], 20);
    oracle.energy[0] = 20;
    oracle.spent[0] += cost;
}

fn apply_aurelia_hit(
    fanout: &Value,
    packets: &[u64],
    packet_index: usize,
    oracle: &mut AureliaEnergyOracle,
) {
    assert_eq!(fanout["source_actor_index"], 0);
    let base = packets[packet_index];
    assert_eq!(fanout["base"], fraction(base, 1000));
    let recipients = fanout["recipients"].as_array().unwrap();
    assert_eq!(recipients.len(), 3);
    for (expected_recipient, entry) in recipients.iter().enumerate() {
        let recipient = entry["recipient_actor_index"].as_u64().unwrap() as usize;
        assert_eq!(recipient, expected_recipient);
        assert_eq!(oracle.packet_by_recipient[recipient], packet_index);
        oracle.packet_by_recipient[recipient] += 1;
        let (distribution_n, distribution_d) = if recipient == 0 { (1, 1) } else { (3, 5) };
        let denominator = 1000 * distribution_d;
        oracle.cumulative[recipient] += base * distribution_n;
        let next_generated = oracle.cumulative[recipient] / denominator;
        let gross = next_generated - oracle.generated[recipient];
        let retained = gross.min(120 - oracle.energy[recipient]);
        assert_eq!(entry["energy_before"], oracle.energy[recipient]);
        assert_eq!(entry["gross"], gross);
        assert_eq!(entry["retained"], retained);
        assert_eq!(entry["overflow"], gross - retained);
        assert_eq!(entry["energy_after"], oracle.energy[recipient] + retained);
        assert_eq!(
            entry["distribution"],
            fraction(distribution_n, distribution_d)
        );
        oracle.generated[recipient] = next_generated;
        oracle.energy[recipient] += retained;
        oracle.overflow[recipient] += gross - retained;
    }
}

fn assert_aurelia_continuing_energy_ledger(report: &Value, packets: &[u64]) {
    let mut oracle = AureliaEnergyOracle::default();
    let mut packet_index = 0;
    for event in report["trace"].as_array().unwrap() {
        if let Some(change) = event.get("energy_change") {
            if change["source"] == "ability_cost" {
                apply_aurelia_cost(change, &mut oracle);
            }
        } else if let Some(fanout) = event.get("energy_fanout")
            && fanout["source"] == "aurelia_hit"
        {
            apply_aurelia_hit(fanout, packets, packet_index, &mut oracle);
            packet_index += 1;
        }
    }
    assert_eq!(oracle.packet_by_recipient, [91; 3]);
    assert_aurelia_final_states(report, &oracle);
}

fn assert_aurelia_final_states(report: &Value, oracle: &AureliaEnergyOracle) {
    for recipient in 0..3 {
        let state = &report["final_actor_states"][recipient];
        assert_eq!(state["energy"], oracle.energy[recipient]);
        assert_eq!(state["granted_energy"], oracle.generated[recipient]);
        assert_eq!(state["overflow_energy"], oracle.overflow[recipient]);
        assert_eq!(state["spent_energy"], oracle.spent[recipient]);
        let distribution_denominator = if recipient == 0 { 1000 } else { 5000 };
        assert_eq!(
            state["charge_remainder"],
            fraction(
                oracle.cumulative[recipient] % distribution_denominator,
                distribution_denominator
            )
        );
    }
}

#[test]
fn aurelia_continuing_normal_skill_burst_reconciles_energy_and_post_spend_chain() {
    let req = aurelia_continuing_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 97);
    assert_eq!(hits[81]["ability_id"], "GA_Mitsuki_Skill");
    assert_eq!(hits[82]["ability_id"], "GA_Mitsuki_UltraSkill");
    assert_eq!(hits[88]["ability_id"], "GA_Mitsuki_Melee");

    let packets = aurelia_expected_charge_packets();
    assert_eq!(packets.len(), 91);
    assert_aurelia_continuing_energy_ledger(&report, &packets);
    assert_eq!(report["final_actor_states"][0]["energy"], 34);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 148);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 14);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][1]["energy"], 89);
    assert_eq!(report["final_actor_states"][2]["energy"], 89);
}
