use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/thief-candy-dodge-cycles.nte");

/// Flat wearer melee schedule (time, packet): the two chains ride the
/// windows, the 16.4s stage lands past the 16s expiry, and the 28.2s attack
/// lands exactly on the second expiry boundary.
const MELEE_SCHEDULE: [(u64, &str); 36] = [
    (1_500_000, "GE_Player_Sagiri_Melee1_Damage"),
    (2_300_000, "GE_Player_Sagiri_Melee2_Damage"),
    (2_300_000, "GE_Player_Sagiri_Melee3_Damage"),
    (3_100_000, "GE_Player_Sagiri_Melee8_Damage"),
    (3_100_000, "GE_Player_Sagiri_Melee4_Damage"),
    (3_900_000, "GE_Player_Sagiri_Melee5_Damage"),
    (3_900_000, "GE_Player_Sagiri_Melee5_Damage"),
    (3_900_000, "GE_Player_Sagiri_Melee5_Damage"),
    (4_700_000, "GE_Player_Sagiri_Melee6_Damage"),
    (4_700_000, "GE_Player_Sagiri_Melee7_Damage"),
    (6_500_000, "GE_Player_Sagiri_Melee1_Damage"),
    (7_300_000, "GE_Player_Sagiri_Melee2_Damage"),
    (7_300_000, "GE_Player_Sagiri_Melee3_Damage"),
    (8_100_000, "GE_Player_Sagiri_Melee8_Damage"),
    (8_100_000, "GE_Player_Sagiri_Melee4_Damage"),
    (8_900_000, "GE_Player_Sagiri_Melee5_Damage"),
    (8_900_000, "GE_Player_Sagiri_Melee5_Damage"),
    (8_900_000, "GE_Player_Sagiri_Melee5_Damage"),
    (9_700_000, "GE_Player_Sagiri_Melee6_Damage"),
    (9_700_000, "GE_Player_Sagiri_Melee7_Damage"),
    (14_800_000, "GE_Player_Sagiri_Melee1_Damage"),
    (15_600_000, "GE_Player_Sagiri_Melee2_Damage"),
    (15_600_000, "GE_Player_Sagiri_Melee3_Damage"),
    (16_400_000, "GE_Player_Sagiri_Melee8_Damage"),
    (16_400_000, "GE_Player_Sagiri_Melee4_Damage"),
    (18_700_000, "GE_Player_Sagiri_Melee1_Damage"),
    (19_500_000, "GE_Player_Sagiri_Melee2_Damage"),
    (19_500_000, "GE_Player_Sagiri_Melee3_Damage"),
    (20_300_000, "GE_Player_Sagiri_Melee8_Damage"),
    (20_300_000, "GE_Player_Sagiri_Melee4_Damage"),
    (21_100_000, "GE_Player_Sagiri_Melee5_Damage"),
    (21_100_000, "GE_Player_Sagiri_Melee5_Damage"),
    (21_100_000, "GE_Player_Sagiri_Melee5_Damage"),
    (21_900_000, "GE_Player_Sagiri_Melee6_Damage"),
    (21_900_000, "GE_Player_Sagiri_Melee7_Damage"),
    (28_200_000, "GE_Player_Sagiri_Melee1_Damage"),
];
/// Frozen CT_ForkBuff 3 x buff_ThiefCandy_Up totals (8/9.6/11.2/12.8/14.40
/// percent per stack): the three-stack M_bonus zone per tier.
const TRIPLE_STACK_TIERS: [(u64, u64); 5] = [(6, 25), (36, 125), (42, 125), (48, 125), (54, 125)];
/// Native lvl80 ATK coefficients (AtkRateBaseArray[0]) at the static ATK 1000.
const MELEE_BASE_DAMAGE: [(&str, u64); 8] = [
    ("GE_Player_Sagiri_Melee1_Damage", 403),
    ("GE_Player_Sagiri_Melee2_Damage", 138),
    ("GE_Player_Sagiri_Melee3_Damage", 757),
    ("GE_Player_Sagiri_Melee4_Damage", 716),
    ("GE_Player_Sagiri_Melee5_Damage", 331),
    ("GE_Player_Sagiri_Melee6_Damage", 35),
    ("GE_Player_Sagiri_Melee7_Damage", 1062),
    ("GE_Player_Sagiri_Melee8_Damage", 16),
];
/// The swap-in Support packet rides the first window; the dash itself deals no
/// damage and grants none.
const PEER_BASE_DAMAGE: [(&str, u64); 8] = [
    ("GE_Player_Mint_Melee1_Damage_New", 104),
    ("GE_Player_Mint_Melee2_Damage_New", 231),
    ("GE_Player_Mint_Melee3_Damage_New", 170),
    ("GE_Player_Mint_Melee4_Damage_New", 262),
    ("GE_Player_Mint_Melee5_Damage_New", 341),
    ("GE_Player_Mint_Melee6_Damage_New", 184),
    ("GE_Player_Mint_Melee7_Damage_New", 255),
    ("GE_Player_Mint_Melee8_Damage_New", 523),
];
const QTE_TIME: u64 = 14_050_000;
const QTE_BASE_DAMAGE: u64 = 2000;
/// Whole-scenario packet multisets for the ChargeAdd energy oracle (dash
/// inputs deliver no packets; the Support packet pays its frozen charge).
const WEARER_ENERGY_PACKETS: [(&str, u32); 9] = [
    ("GE_Player_Sagiri_Melee1_Damage", 5),
    ("GE_Player_Sagiri_Melee2_Damage", 4),
    ("GE_Player_Sagiri_Melee3_Damage", 4),
    ("GE_Player_Sagiri_Melee4_Damage", 4),
    ("GE_Player_Sagiri_Melee5_Damage", 9),
    ("GE_Player_Sagiri_Melee6_Damage", 3),
    ("GE_Player_Sagiri_Melee7_Damage", 3),
    ("GE_Player_Sagiri_Melee8_Damage", 4),
    ("GE_Player_Sagiri_QTE1_Damage", 1),
];
/// Mint runs one four-attack chain (stages 0..3) and one five-attack chain.
const PEER_ENERGY_PACKETS: [(&str, u32); 8] = [
    ("GE_Player_Mint_Melee1_Damage_New", 2),
    ("GE_Player_Mint_Melee2_Damage_New", 2),
    ("GE_Player_Mint_Melee3_Damage_New", 2),
    ("GE_Player_Mint_Melee4_Damage_New", 2),
    ("GE_Player_Mint_Melee5_Damage_New", 2),
    ("GE_Player_Mint_Melee6_Damage_New", 2),
    ("GE_Player_Mint_Melee7_Damage_New", 1),
    ("GE_Player_Mint_Melee8_Damage_New", 1),
];
/// Peer packet ids by native chain stage order (Mint stages 1/1/1/1/2).
const PEER_PACKETS: [&str; 8] = [
    "GE_Player_Mint_Melee1_Damage_New",
    "GE_Player_Mint_Melee2_Damage_New",
    "GE_Player_Mint_Melee3_Damage_New",
    "GE_Player_Mint_Melee4_Damage_New",
    "GE_Player_Mint_Melee5_Damage_New",
    "GE_Player_Mint_Melee6_Damage_New",
    "GE_Player_Mint_Melee7_Damage_New",
    "GE_Player_Mint_Melee8_Damage_New",
];
/// Flat peer schedule (time, packet index): one four-attack chain (stages
/// 0..3) between the windows and one five-attack chain after them.
const PEER_SCHEDULE: [(u64, usize); 14] = [
    (10_750_000, 0),
    (10_750_000, 1),
    (11_550_000, 2),
    (12_350_000, 3),
    (12_350_000, 4),
    (13_150_000, 5),
    (22_950_000, 0),
    (22_950_000, 1),
    (23_750_000, 2),
    (24_550_000, 3),
    (24_550_000, 4),
    (25_350_000, 5),
    (26_150_000, 6),
    (26_150_000, 7),
];

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    // Flat ATK 1000 with zeroed static bonus zones keeps every packet oracle
    // on the bare native coefficient plus the Critical Dodge M_bonus zone.
    for actor in &mut req.actors {
        let stats = &mut actor.static_stats;
        stats.crit_rate = Ratio::zero();
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        stats.atk = Ratio {
            denominator: 1,
            numerator: 1000,
        };
        stats.base_atk = 1000;
    }
    req
}

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

/// Scenario ChargeAdd sum: each native packet pays once per multiset entry.
fn charge_sum(packets: &[(&str, u32)]) -> ExactFraction {
    packets
        .iter()
        .flat_map(|(effect, count)| {
            std::iter::repeat_n(packet_charge(effect), usize::try_from(*count).unwrap())
        })
        .fold(ExactFraction::new(0, 1).unwrap(), |sum, charge| {
            sum.add(charge).unwrap()
        })
}

/// M_bonus = 1 + the three-stack Critical Dodge zone while a window is live;
/// the retriggered window covers [0, 16s) and the fresh one [17.2s, 28.2s).
fn windowed(base: u64, time_us: u64, tier: u32) -> ExactFraction {
    let live = time_us < 16_000_000 || (17_200_000..28_200_000).contains(&time_us);
    if !live {
        return ExactFraction::new(u128::from(base), 1).unwrap();
    }
    let (numerator, denominator) = TRIPLE_STACK_TIERS[(tier - 1) as usize];
    ExactFraction::new(u128::from(base), 1)
        .unwrap()
        .mul(
            ExactFraction::new(1, 1)
                .unwrap()
                .add(ExactFraction::new(u128::from(numerator), u128::from(denominator)).unwrap())
                .unwrap(),
        )
        .unwrap()
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

fn base_damage(table: &[(&str, u64)], effect: &str) -> u64 {
    table
        .iter()
        .find(|(candidate, _)| *candidate == effect)
        .unwrap_or_else(|| panic!("unexpected packet {effect}"))
        .1
}

fn actor_hits<'a>(report: &'a Value, actor_index: u64, prefix: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["actor_index"] == actor_index
                && hit["damage_effect_id"]
                    .as_str()
                    .is_some_and(|fx| fx.starts_with(prefix))
        })
        .collect()
}

fn assert_critical_dash_receipts(report: &Value) {
    let receipts = report["player_input_receipts"].as_array().unwrap();
    assert_eq!(
        receipts.len(),
        8,
        "seven Critical Dodges and one plain dash"
    );
    let critical = [true, true, true, false, true, true, true, true];
    for (receipt, expected) in receipts.iter().zip(critical) {
        assert_eq!(receipt["critical_success"], expected);
        assert_eq!(
            super::support::receipt_assumption(
                report,
                receipt,
                crate::simulation::inputs::CRITICAL_DODGE_ID.as_str()
            )
            .is_some(),
            expected
        );
        assert_eq!(
            super::support::receipt_assumption(
                report,
                receipt,
                crate::weapons::thief_candy::HYPOTHESIS_ID.as_str()
            )
            .is_some(),
            expected
        );
    }
}

fn assert_wearer_windows(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Sagiri_Melee");
    assert_eq!(hits.len(), MELEE_SCHEDULE.len(), "wearer chain packets");
    for (hit, (time, effect)) in hits.iter().zip(MELEE_SCHEDULE.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        assert_eq!(hit["damage_effect_id"], *effect);
        assert_eq!(hit["critical"], false);
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            windowed(base_damage(&MELEE_BASE_DAMAGE, effect), *time, tier),
            "tier {tier} {effect} at {time}"
        );
    }
}

fn assert_support_packet(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Sagiri_QTE");
    assert_eq!(hits.len(), 1, "one switch-in Support packet");
    assert_eq!(hits[0]["time_us"].as_u64().unwrap(), QTE_TIME);
    assert_eq!(
        exact(&hits[0]["non_crit_damage"]),
        windowed(QTE_BASE_DAMAGE, QTE_TIME, tier),
        "tier {tier} Support packet in the first window"
    );
}

fn assert_peer_schedule(report: &Value) {
    let hits = actor_hits(report, 1, "GE_Player_Mint_Melee");
    assert_eq!(hits.len(), PEER_SCHEDULE.len(), "peer chain packets");
    for (hit, (time, packet)) in hits.iter().zip(PEER_SCHEDULE.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        let effect = PEER_PACKETS[*packet];
        assert_eq!(hit["damage_effect_id"], effect);
        // The Critical Dodge buff is wearer-only: the peer stays bare.
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            ExactFraction::new(u128::from(base_damage(&PEER_BASE_DAMAGE, effect)), 1).unwrap(),
            "bare peer {effect} at {time}"
        );
    }
}

/// Owner packets pay ChargeAdd x1 and every peer x3/5 (relations::DISTRIBUTION);
/// the expected sums re-read the frozen native ChargeAdd rows independently.
/// Nothing spends energy: both pools start full and only overflow.
fn assert_energy_oracle(report: &Value, tier: u32) {
    let wearer = charge_sum(&WEARER_ENERGY_PACKETS);
    let peer = charge_sum(&PEER_ENERGY_PACKETS);
    let share = ExactFraction::new(3, 5).unwrap();
    let expected = [
        wearer.add(peer.mul(share).unwrap()).unwrap(),
        peer.add(wearer.mul(share).unwrap()).unwrap(),
    ];
    for (actor, total) in expected.iter().enumerate() {
        let state = &report["final_actor_states"][actor];
        let granted = ExactFraction::new(state["granted_energy"].as_u64().unwrap().into(), 1)
            .unwrap()
            .add(exact(&state["charge_remainder"]))
            .unwrap();
        assert_eq!(granted, *total, "actor {actor} charge, tier {tier}");
        assert_eq!(state["spent_energy"].as_u64().unwrap(), 0, "tier {tier}");
    }
}

#[test]
fn thief_candy_continuing_cycles_pin_critical_dodge_windows_around_chain_visits() {
    for tier in 1..=5u32 {
        let req = cycles_request(tier);
        assert_eq!(req.actors[0].initial_hp, 16483, "native lvl80 wearer HP");
        assert_eq!(req.actors[1].initial_hp, 12120, "native lvl80 peer HP");
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_critical_dash_receipts(&report);
        assert_wearer_windows(&report, tier);
        assert_support_packet(&report, tier);
        assert_peer_schedule(&report);
        assert_energy_oracle(&report, tier);
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] != "ntesim.reaction.hexed_followup")
                .count(),
            51
        );
        assert_eq!(report["final_actor_states"][0]["hp"], 16483);
        assert_eq!(report["final_actor_states"][1]["hp"], 12120);
        assert!(
            crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .all(|buff| buff["id"] != "thief_candy.critical_dodge_damage_up"),
            "both windows expired before the horizon"
        );
        let replay: GameScenarioRequest =
            serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap();
        assert_eq!(
            report,
            run_game_scenario(&replay, &ExecutionLimits::default()).unwrap()
        );
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
    }
}
