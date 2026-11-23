use super::nanally_actions::{make_action, request};
use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};

const NORMAL_CHARGE_MILLI: [u64; 14] = [
    1_138, 1_492, 470, 806, 4_884, 267, 267, 267, 267, 267, 267, 267, 2_966, 2_974,
];
const SKILL_CHARGE_MILLI: [u64; 5] = [1_260; 5];
const NORMAL_DAMAGE_MILLI: [u64; 14] = [
    507, 665, 209, 360, 2_177, 118, 118, 118, 118, 118, 118, 118, 1_321, 1_325,
];

fn normal(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    )
}

fn skill(at: u64) -> GameActionCommand {
    make_action(
        at,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    )
}

fn push_chain(actions: &mut Vec<GameActionCommand>, start: u64) {
    for offset in super::nanally_clock::STARTS.into_iter().take(5) {
        actions.push(normal(start + offset));
    }
}

fn zero_start_underboss_request() -> GameScenarioRequest {
    let mut req = request();
    req.relative_duration_us = 51_000_000;
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].hp_current = 10_000_000;
    req.targets[0].defense = 1_080;
    req.targets[0].resistance = Ratio::zero();
    for actor in &mut req.actors {
        actor.initial_energy = 0;
        actor.energy_capacity = 100;
        actor.gear_procs.clear();
    }
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([2]).unwrap();
    req.actors[0].talent_levels = Some([8, 8, 8]);
    for (slot, character_id) in [(1, "character:1019"), (2, "character:1003")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character_id.into();
        ally.awakening = ntesim_data::AwakeningSelection::default();
        ally.talent_levels = Some([1; 3]);
        req.actors.push(ally);
    }

    let mut actions = vec![skill(0)];
    for start in [
        1_200_000, 7_200_000, 13_200_000, 19_200_000, 25_200_000, 31_200_000,
    ] {
        push_chain(&mut actions, start);
    }
    actions.push(make_action(
        37_200_000,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
        0,
    ));
    push_chain(&mut actions, 41_000_000);
    actions.push(skill(47_000_000));
    req.actions = actions;
    req
}

fn charge_packets() -> Vec<u64> {
    let mut packets = Vec::new();
    packets.extend(SKILL_CHARGE_MILLI);
    for _ in 0..6 {
        packets.extend(NORMAL_CHARGE_MILLI);
    }
    packets.extend(NORMAL_CHARGE_MILLI);
    packets.extend(SKILL_CHARGE_MILLI);
    packets
}

fn reduced_fraction(numerator: u64, denominator: u64) -> Value {
    if numerator == 0 {
        return Value::Null;
    }
    let (mut a, mut b) = (numerator, denominator);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    json!(format!("{}/{}", numerator / a, denominator / a))
}

#[derive(Clone, Default)]
struct EnergyOracle {
    cumulative_numerator: u64,
    gross_granted: u64,
    energy: u64,
    overflow: u64,
    spent: u64,
}

fn assert_independent_energy_ledger(req: &GameScenarioRequest, report: &Value) {
    let mut pools = vec![EnergyOracle::default(); req.actors.len()];
    let packets = charge_packets();
    let mut packet_index = 0;
    let mut costs = 0;
    for event in report["trace"].as_array().unwrap() {
        if let Some(change) = event.get("energy_change") {
            assert_eq!(change["source"], "ability_cost");
            costs += 1;
            assert_eq!(change["source_actor_index"], 0);
            assert_eq!(change["recipient_actor_index"], 0);
            let spent = change["spent"].as_u64().unwrap();
            if spent == 100 {
                assert_eq!(change["energy_before"], 100);
                assert_eq!(change["energy_after"], 0);
                pools[0].energy = 0;
                pools[0].spent += 100;
            } else {
                assert_eq!(spent, 0);
                assert_eq!(change["energy_after"], change["energy_before"]);
            }
            continue;
        }
        let Some(fanout) = event.get("energy_fanout") else {
            continue;
        };

        assert_eq!(fanout["source"], "nanally_hit");
        assert_eq!(fanout["source_actor_index"], 0);
        let milli = packets[packet_index];
        assert_eq!(fanout["base"], reduced_fraction(milli, 1_000));
        let recipients = fanout["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), req.actors.len());
        for (recipient_index, entry) in recipients.iter().enumerate() {
            assert_eq!(entry["recipient_actor_index"], recipient_index);
            let (distribution_n, distribution_d) =
                if recipient_index == 0 { (1, 1) } else { (3, 5) };
            let denominator = 1_000 * distribution_d;
            let pool = &mut pools[recipient_index];
            pool.cumulative_numerator += milli * distribution_n;
            let next_gross = pool.cumulative_numerator / denominator;
            let gross = next_gross - pool.gross_granted;
            let retained = gross.min(req.actors[recipient_index].energy_capacity - pool.energy);
            assert_eq!(
                entry["distribution"],
                reduced_fraction(distribution_n, distribution_d)
            );
            assert_eq!(entry["energy_before"], pool.energy);
            assert_eq!(entry["gross"], gross);
            assert_eq!(entry["retained"], retained);
            assert_eq!(entry["overflow"], gross - retained);
            assert_eq!(entry["energy_after"], pool.energy + retained);
            pool.energy += retained;
            pool.overflow += gross - retained;
            pool.gross_granted = next_gross;
        }
        packet_index += 1;
    }

    assert_eq!(costs, req.actions.len());
    assert_eq!(packet_index, packets.len());
    for (index, (pool, state)) in pools
        .iter()
        .zip(report["final_actor_states"].as_array().unwrap())
        .enumerate()
    {
        let denominator = if index == 0 { 1_000 } else { 5_000 };
        assert_eq!(state["energy"], pool.energy);
        assert_eq!(state["granted_energy"], pool.gross_granted);
        assert_eq!(state["overflow_energy"], pool.overflow);
        assert_eq!(state["spent_energy"], pool.spent);
        assert_eq!(
            state["charge_remainder"],
            reduced_fraction(pool.cumulative_numerator % denominator, denominator)
        );
    }
}

fn assert_damage_packets(report: &Value) {
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 129);
    assert_eq!(report["hits_settled"], 129);

    let normal_hits = hits
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("GE_Player_Nanally_Melee"))
        })
        .collect::<Vec<_>>();
    assert_eq!(normal_hits.len(), 98);
    let normal_effects = [
        "GE_Player_Nanally_Melee1_Damage",
        "GE_Player_Nanally_Melee2_Damage",
        "GE_Player_Nanally_Melee3_Damage",
        "GE_Player_Nanally_Melee4_Damage",
        "GE_Player_Nanally_Melee5_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee6_Damage",
        "GE_Player_Nanally_Melee9_Damage",
        "GE_Player_Nanally_Melee10_Damage",
    ];
    for (index, hit) in normal_hits.iter().enumerate() {
        assert_eq!(
            hit["damage_effect_id"],
            normal_effects[index % normal_effects.len()]
        );
        assert_eq!(
            damage(hit),
            ExactFraction::new(
                NORMAL_DAMAGE_MILLI[index % NORMAL_DAMAGE_MILLI.len()] as u128,
                2,
            )
            .unwrap()
        );
    }

    let skill_hits = hits
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Skill1_Damage")
        .collect::<Vec<_>>();
    assert_eq!(skill_hits.len(), 10);
    assert!(
        skill_hits
            .iter()
            .all(|hit| damage(hit) == ExactFraction::new(857, 1).unwrap())
    );

    let burst_hits = hits
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("GE_Player_Nanally_UltraSkill"))
        })
        .collect::<Vec<_>>();
    assert_eq!(burst_hits.len(), 7);
    let burst_coefficients = [5_023, 725, 725, 725, 725, 725, 8_490];
    let burst_effects = [
        "GE_Player_Nanally_UltraSkill1_Damage",
        "GE_Player_Nanally_UltraSkill2_Damage",
        "GE_Player_Nanally_UltraSkill2_Damage",
        "GE_Player_Nanally_UltraSkill2_Damage",
        "GE_Player_Nanally_UltraSkill2_Damage",
        "GE_Player_Nanally_UltraSkill2_Damage",
        "GE_Player_Nanally_UltraSkill3_Damage",
    ];
    for ((hit, coefficient), effect) in burst_hits.iter().zip(burst_coefficients).zip(burst_effects)
    {
        assert_eq!(hit["damage_effect_id"], effect);
        assert_eq!(damage(hit), ExactFraction::new(coefficient, 2).unwrap());
    }

    let air_attacks = hits
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Cat_AirAttack_Damage")
        .collect::<Vec<_>>();
    assert_eq!(air_attacks.len(), 0);
    assert!(
        air_attacks
            .iter()
            .all(|hit| damage(hit) == ExactFraction::new(857, 2).unwrap())
    );

    let coordinated_normal = hits
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("GE_Player_Nanally_Cat_Melee"))
        })
        .collect::<Vec<_>>();
    assert_eq!(coordinated_normal.len(), 9);
    let coordinated_effects = [
        "GE_Player_Nanally_Cat_Melee1_Damage",
        "GE_Player_Nanally_Cat_Melee2_Damage",
        "GE_Player_Nanally_Cat_Melee3_Damage",
        "GE_Player_Nanally_Cat_Melee4_Damage",
        "GE_Player_Nanally_Cat_Melee5_Damage",
        "GE_Player_Nanally_Cat_Melee5_Damage",
        "GE_Player_Nanally_Cat_Melee5_Damage",
        "GE_Player_Nanally_Cat_Melee6_Damage",
        "GE_Player_Nanally_Cat_Melee7_Damage",
    ];
    for ((hit, coefficient), effect) in coordinated_normal
        .iter()
        .zip([428, 386, 728, 686, 543, 543, 543, 771, 1_028])
        .zip(coordinated_effects)
    {
        assert_eq!(hit["damage_effect_id"], effect);
        assert_eq!(damage(hit), ExactFraction::new(coefficient, 2).unwrap());
    }

    let coordinated_skill = hits
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Cat_Skill_Damage")
        .collect::<Vec<_>>();
    assert_eq!(coordinated_skill.len(), 5);
    assert!(
        coordinated_skill
            .iter()
            .all(|hit| damage(hit) == ExactFraction::new(411, 2).unwrap())
    );

    let total = hits
        .iter()
        .fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(damage(hit)).unwrap()
        });
    assert_eq!(total, ExactFraction::new(93_719, 2).unwrap());
}

fn damage(hit: &Value) -> ExactFraction {
    serde_json::from_value(hit["non_crit_damage"].clone()).unwrap()
}

#[test]
fn nanally_zero_start_funds_burst_and_continues_under_underboss() {
    let req = zero_start_underboss_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_damage_packets(&report);
    assert_independent_energy_ledger(&req, &report);
    assert_eq!(report["final_actor_states"][0]["energy"], 23);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 128);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 5);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("793/1000")
    );
    for recipient in [1, 2] {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 77);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            77
        );
        assert_eq!(
            report["final_actor_states"][recipient]["overflow_energy"],
            0
        );
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            json!("1379/5000")
        );
    }
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("93719/2")
    );
    for hypothesis in [
        "nanally.ordinary_hit_energy_v1",
        "nanally.normal_chain_v1",
        "nanally.skill_five_hits_v1",
        "nanally.burst_seven_hits_v1",
        "nanally.burst_summon_state_v1",
        "nanally.underboss_coordination_v2",
    ] {
        assert!(
            report["resolved_model_assumptions"][hypothesis].is_string(),
            "{hypothesis}"
        );
    }
}
