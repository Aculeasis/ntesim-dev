use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/nestbird-gaze-cycles.nte");
const DOT: &str = "GE_Player_Zankou_DotUltraDamage";
const BURST: &str = "GE_Player_Zankou_MagicUltraSkill2_Damage";
// Frozen buff_NestBird_down ratios converted to positive reductions per tier.
const REDUCTIONS: [(u64, u64); 5] = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)];
// Zankou's five-input melee chain repeats a fixed packet schedule; attack:30
// replays it six times, settling every chain packet on target zero. Each
// packet pays its native ChargeAdd once per delivery.
const CHAIN_PACKETS: [(&str, u32); 15] = [
    ("GE_Player_Zankou_Melee1_Damage", 6),
    ("GE_Player_Zankou_Melee1_1_Damage", 6),
    ("GE_Player_Zankou_Melee2_Damage", 6),
    ("GE_Player_Zankou_Melee3_Damage", 6),
    ("GE_Player_Zankou_Melee3_1_Damage", 6),
    ("GE_Player_Zankou_Melee3_2_Damage", 6),
    ("GE_Player_Zankou_Melee4_Damage", 6),
    ("GE_Player_Zankou_Melee4_1_Damage", 6),
    ("GE_Player_Zankou_Melee4_2_Damage", 6),
    ("GE_Player_Zankou_Melee4_3_Damage", 6),
    ("GE_Player_Zankou_Melee4_4_Damage", 6),
    ("GE_Player_Zankou_Melee5_Damage", 12),
    ("GE_Player_Zankou_Melee5_1_Damage", 12),
    ("GE_Player_Zankou_Melee5_2_Damage", 6),
    ("GE_Player_Zankou_Melee6_Damage", 6),
];

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    crate::simulation::PreparedSourceScenario::new(
        &source,
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
    .request
}

fn gaze_marks(report: &Value) -> Vec<Value> {
    crate::simulation::tests::support::final_buffs(report)
        .into_iter()
        .filter(|buff| buff["id"] == "nest_bird.warning_gaze")
        .collect()
}

fn dot_tick_times(report: &Value, target_index: u64) -> Vec<u64> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == DOT && hit["target_index"] == target_index)
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect()
}

fn packet_charge(effect_id: &str) -> ExactFraction {
    let row = ntesim_data::native_action_metadata::resolve_damage_effect(
        "ntesim-current",
        ntesim_data::stat_projection::frozen_tables(),
        effect_id,
    )
    .unwrap();
    let raw = row
        .raw_row
        .as_ref()
        .and_then(|row| row.get("ChargeAdd"))
        .unwrap_or_else(|| panic!("{effect_id} has no ChargeAdd row"));
    let scalar = ntesim_data::native::parse_native_scalar(raw).unwrap();
    ExactFraction::new(scalar.numerator(), scalar.denominator()).unwrap()
}

#[test]
fn nest_bird_continuing_cycles_mark_both_dummies_through_two_off_field_bursts() {
    for tier in 1..=5u32 {
        let mut req = cycles_request(tier);
        req.relative_duration_us = 109_868_375;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        // Two continuing Burst cycles: the recast lands after the 20s cooldown
        // with energy refilled only by the interleaved 30-input melee chain.
        let bursts: Vec<u64> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == BURST && hit["target_index"] == 0)
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(bursts, vec![4_216_667, 70_782_239], "tier {tier}");
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
        // Independent ChargeAdd oracle for the continuing refill.
        let mut granted = ExactFraction::new(0, 1).unwrap();
        for (effect_id, count) in CHAIN_PACKETS {
            let charge = packet_charge(effect_id);
            for _ in 0..count {
                granted = granted.add(charge).unwrap();
            }
        }
        // The real final handoff earns Mint Support. Its 8.4 energy packet
        // fans out 3/5 to off-field Zankou, retaining the fractional carry.
        granted = granted.add(ExactFraction::new(126, 25).unwrap()).unwrap();
        let whole: u64 = granted.round_u64().unwrap();
        assert_eq!(
            report["final_actor_states"][0]["granted_energy"], whole,
            "tier {tier}"
        );
        let remainder = granted
            .sub(ExactFraction::new(u128::from(whole), 1).unwrap())
            .unwrap();
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            serde_json::to_value(remainder).unwrap(),
            "tier {tier}"
        );
        // Each declared dummy receives 29 half-open Vile Ash ticks per cast.
        // Bloodfeast applies at montage4216667; the second follows six
        // full Reality chains (6811396 each) and the authored twenty-second wait.
        for target in 0..2u64 {
            let times = dot_tick_times(&report, target);
            let expected: Vec<u64> = (1..30)
                .map(|second| 5_697_196 + second * 1_000_000)
                .chain((1..30).map(|second| 72_262_768 + second * 1_000_000))
                .collect();
            assert_eq!(times, expected, "tier {tier} target {target}");
        }
        // Both dummies keep exactly one refreshed Warning Gaze instance: the
        // second cast's last tick refreshes the same mark for twenty seconds.
        let marks = gaze_marks(&report);
        assert_eq!(marks.len(), 2, "tier {tier}");
        let (numerator, denominator) = REDUCTIONS[tier as usize - 1];
        for (index, mark) in marks.iter().enumerate() {
            assert_eq!(mark["owner_actor"], 0);
            assert_eq!(mark["stacks"], 1);
            assert_eq!(mark["recipient"]["target"], index.to_string());
            assert_eq!(mark["duration"]["expires_at_us"], 109_868_376);
            assert_eq!(
                mark["stat_modifiers"]["outgoing_damage_reduction"],
                serde_json::json!(format!("{}/{}", numerator, denominator))
            );
        }
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
        // The exact expiry boundary removes the marks without losing damage.
        req.relative_duration_us = 109_868_376;
        let expired = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(expired["execution_completed"], true);
        assert!(gaze_marks(&expired).is_empty());
        assert_eq!(expired["total_damage_dealt"], report["total_damage_dealt"]);
        assert_eq!(expired["hits_settled"], report["hits_settled"]);
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            expired,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );
        assert_eq!(
            expired,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}
