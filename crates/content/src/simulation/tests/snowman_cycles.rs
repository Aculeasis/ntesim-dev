use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/snowman-blessing-cycles.nte");

/// Adler's five-stage native chain (1/1/7/2/1 category-A packets). Adler is
/// the group-FIVE wearer whose own Blessing Skill shields the active actor,
/// so the Solifugae Vita ATK zone toggles with his shield windows.
const WEARER_STAGES: [&[(&str, u32)]; 5] = [
    &[("GE_Player_Adler_Melee1_Damage", 1)],
    &[("GE_Player_Adler_Melee2_Damage", 1)],
    &[("GE_Player_Adler_Melee3_Damage", 7)],
    &[
        ("GE_Player_Adler_Melee4_1_Damage", 1),
        ("GE_Player_Adler_Melee4_2_Damage", 1),
    ],
    &[("GE_Player_Adler_Melee5_Damage", 1)],
];
/// Two shielded chain visits (8s Blessing windows from the casts at 0s/8.8s).
const MELEE_VISIT_TIMES: [&[u64]; 2] = [
    &[1_200_000, 2_000_000, 2_800_000, 3_600_000, 4_400_000],
    &[10_000_000, 10_800_000, 11_600_000, 12_400_000, 13_200_000],
];
/// Chain hits landing exactly on each shield expiry instant settle at bare ATK
/// (the half-open shield interval removes the bonus before HitDelivery).
const BOUNDARY_HITS: [u64; 2] = [8_000_000, 16_800_000];
/// Native lvl80 ATK coefficients (AtkRateBaseArray[0]) at the static ATK 1000.
const MELEE_BASE_DAMAGE: [(&str, u64); 6] = [
    ("GE_Player_Adler_Melee1_Damage", 201),
    ("GE_Player_Adler_Melee2_Damage", 577),
    ("GE_Player_Adler_Melee3_Damage", 154),
    ("GE_Player_Adler_Melee4_1_Damage", 219),
    ("GE_Player_Adler_Melee4_2_Damage", 231),
    ("GE_Player_Adler_Melee5_Damage", 875),
];
/// Card table: 18/21/24/27/30 percent per tier (frozen curve keys 1800..3000).
const ATK_TIERS: [(u64, u64); 5] = [(9, 50), (21, 100), (6, 25), (27, 100), (3, 10)];
/// Each Blessing cast delivers its DEF-scaled Skill2 + two Skill3 packets; the
/// 1s tick train follows at 200 coefficient until superseded by a recast
/// (generation check), so cast one ticks 1..8s and cast two 9.8..18.8s.
const SKILL_CASTS: [(u64, (u64, u64)); 2] = [(0, (1, 1)), (8_800_000, (126, 100))];
const SKILL_BASE_DAMAGE: [(&str, u64); 2] = [
    ("GE_Player_Adler_Skill2_Damage", 840),
    ("GE_Player_Adler_Skill3_Damage", 840),
];
const TICK_TIMES: [&[u64]; 2] = [
    &[
        1_000_000, 2_000_000, 3_000_000, 4_000_000, 5_000_000, 6_000_000, 7_000_000, 8_000_000,
    ],
    &[
        9_800_000, 10_800_000, 11_800_000, 12_800_000, 13_800_000, 14_800_000, 15_800_000,
        16_800_000, 17_800_000, 18_800_000,
    ],
];
const TICK_BASE_DAMAGE: u64 = 240;
/// The second cast consumes 13 karma stacks (twelve cycle-one chain packets
/// plus the 8s boundary hit) for its 2%-per-stack final zone: 126/100.
const CAST_TWO_FINAL: (u64, u64) = (126, 100);
/// Whole-scenario packet multisets for the ChargeAdd energy oracle (Skill3 and
/// the ticks carry zero charge).
const WEARER_ENERGY_PACKETS: [(&str, u32); 7] = [
    ("GE_Player_Adler_Melee1_Damage", 4),
    ("GE_Player_Adler_Melee2_Damage", 2),
    ("GE_Player_Adler_Melee3_Damage", 14),
    ("GE_Player_Adler_Melee4_1_Damage", 2),
    ("GE_Player_Adler_Melee4_2_Damage", 2),
    ("GE_Player_Adler_Melee5_Damage", 2),
    ("GE_Player_Adler_Skill2_Damage", 2),
];
const PEER_ENERGY_PACKETS: [(&str, u32); 3] = [
    ("GE_Player_Mint_Melee1_Damage_New", 2),
    ("GE_Player_Mint_Melee2_Damage_New", 2),
    ("GE_Player_Mint_Melee3_Damage_New", 2),
];
/// Two off-field Mint chains (one cast each of stages 0 and 1) at 5.45s while
/// the first shield is live and 14.25s while the second is live.
const PEER_CHAINS: [u64; 2] = [5_450_000, 14_250_000];
const PEER_OPENING: [(&str, u32); 2] = [
    ("GE_Player_Mint_Melee1_Damage_New", 1),
    ("GE_Player_Mint_Melee2_Damage_New", 1),
];
const PEER_THIRD: &str = "GE_Player_Mint_Melee3_Damage_New";
const PEER_BASE_DAMAGE: [(&str, u64); 3] = [
    ("GE_Player_Mint_Melee1_Damage_New", 104),
    ("GE_Player_Mint_Melee2_Damage_New", 231),
    ("GE_Player_Mint_Melee3_Damage_New", 170),
];

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    // Flat ATK 1000 with zeroed static bonus zones keeps the shielded hits on
    // the bare native coefficient plus base_atk x tier ratio.
    for actor in &mut req.actors {
        let stats = &mut actor.static_stats;
        stats.elemental_bonuses.clear();
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
    }
    // DEF1200 with Righteous Heart snapshots each Blessing shield at 1960.
    // regardless of the compiled weapon DefUp secondary stat.
    let wearer = &mut req.actors[0];
    wearer.static_stats.base_def = Some(1000);
    wearer.static_stats.def = 1000;
    wearer.static_stats.def_percent = Ratio::zero();
    wearer.static_stats.def_flat = Ratio::zero();
    wearer.static_stats.shield_bonus = Ratio::zero();
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

/// Base packet coefficient times an exact (numerator, denominator) multiplier:
/// the shielded ATK factor (50+9)/50 and the karma final zone (126, 100).
fn ratio_damage(base: u128, multiplier: (u128, u128)) -> ExactFraction {
    ExactFraction::new(base, 1)
        .unwrap()
        .mul(ExactFraction::new(multiplier.0, multiplier.1).unwrap())
        .unwrap()
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

fn assert_wearer_melee_windows(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Adler_Melee");
    let mut expected = Vec::new();
    for times in MELEE_VISIT_TIMES {
        for (stage, time) in times.iter().enumerate() {
            for (effect, count) in WEARER_STAGES[stage] {
                for _ in 0..*count {
                    expected.push((*time, *effect, true));
                }
            }
        }
    }
    for time in BOUNDARY_HITS {
        expected.push((time, "GE_Player_Adler_Melee1_Damage", false));
    }
    expected.sort_by_key(|entry| entry.0);
    assert_eq!(
        hits.len(),
        expected.len(),
        "tier {tier} wearer melee packets"
    );
    let zone = ATK_TIERS[(tier - 1) as usize];
    for (hit, (time, effect, shielded)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time, "tier {tier}");
        assert_eq!(hit["damage_effect_id"], *effect, "tier {tier}");
        assert_eq!(hit["critical"], false, "tier {tier}");
        let base = u128::from(base_damage(&MELEE_BASE_DAMAGE, effect));
        let expected_damage = if *shielded {
            ratio_damage(base, (u128::from(zone.0 + zone.1), u128::from(zone.1)))
        } else {
            ExactFraction::new(base, 1).unwrap()
        };
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            expected_damage,
            "tier {tier} {effect} at {time} shielded: {shielded}"
        );
    }
}

fn assert_def_scaled_skill_packets(report: &Value) {
    let hits = actor_hits(report, 0, "GE_Player_Adler_Skill");
    let mut expected = Vec::new();
    for (time, multiplier) in SKILL_CASTS {
        for (effect, _) in SKILL_BASE_DAMAGE.iter() {
            let repeats = if *effect == "GE_Player_Adler_Skill2_Damage" {
                1
            } else {
                2
            };
            for _ in 0..repeats {
                expected.push((time, *effect, multiplier));
            }
        }
    }
    for (times, multiplier) in [
        (&TICK_TIMES[0], SKILL_CASTS[0].1),
        (&TICK_TIMES[1], CAST_TWO_FINAL),
    ] {
        for time in times.iter() {
            expected.push((*time, "GE_Player_Adler_Skill_Damage", multiplier));
        }
    }
    expected.sort_by_key(|entry| entry.0);
    assert_eq!(hits.len(), expected.len(), "def-scaled skill packets");
    for (hit, (time, effect, multiplier)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        assert_eq!(hit["damage_effect_id"], *effect);
        assert_eq!(hit["critical"], false);
        let base = if *effect == "GE_Player_Adler_Skill_Damage" {
            TICK_BASE_DAMAGE
        } else {
            base_damage(&SKILL_BASE_DAMAGE, effect)
        };
        // The shielded ATK zone never touches DEF-scaled packets; only the
        // consumed karma final zone scales the second cast's train.
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            ratio_damage(
                u128::from(base),
                (u128::from(multiplier.0), u128::from(multiplier.1))
            ),
            "{effect} at {time} with final zone {multiplier:?}"
        );
    }
}

fn assert_shield_trace(report: &Value) {
    let trace = report["trace"].as_array().unwrap();
    let grants: Vec<&Value> = trace
        .iter()
        .filter(|event| event["event_kind"] == "adler_blessing_shield")
        .collect();
    let expiries: Vec<&Value> = trace
        .iter()
        .filter(|event| event["event_kind"] == "actor_shield_expired")
        .collect();
    assert_eq!(grants.len(), SKILL_CASTS.len(), "Blessing grants");
    assert_eq!(expiries.len(), BOUNDARY_HITS.len(), "shield expiries");
    for (event, (time, _)) in grants.iter().zip(SKILL_CASTS) {
        assert_eq!(event["time_us"].as_u64().unwrap(), time);
        assert_eq!(event["from_actor"], 0);
        assert_eq!(event["to_actor"], 0);
        assert_eq!(event["data"]["amount"], 1960);
        assert_eq!(event["data"]["expires_at_us"], time + 8_000_000);
    }
    for (event, time) in expiries.iter().zip(BOUNDARY_HITS) {
        assert_eq!(event["time_us"].as_u64().unwrap(), time);
        assert_eq!(event["to_actor"], 0);
        assert_eq!(event["data"]["shield_id"], "adler.blessing:0");
    }
}

fn assert_peer_chains(report: &Value) {
    let hits = actor_hits(report, 1, "GE_Player_Mint_Melee");
    let mut expected = Vec::new();
    for chain in PEER_CHAINS {
        for (effect, count) in PEER_OPENING.iter() {
            for _ in 0..*count {
                expected.push((chain, *effect));
            }
        }
        expected.push((chain + 800_000, PEER_THIRD));
    }
    assert_eq!(hits.len(), expected.len(), "peer packets");
    for (hit, (time, effect)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        assert_eq!(hit["damage_effect_id"], *effect);
        assert_eq!(hit["critical"], false);
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            ExactFraction::new(u128::from(base_damage(&PEER_BASE_DAMAGE, effect)), 1).unwrap(),
            "peer {effect} at {time}"
        );
    }
}

/// Owner packets pay ChargeAdd x1 and every peer x3/5 (relations::DISTRIBUTION);
/// the expected sums re-read the frozen native ChargeAdd rows independently.
fn assert_charge_add_energy_oracle(report: &Value, tier: u32) {
    let wearer_native = charge_sum(&WEARER_ENERGY_PACKETS);
    let peer_native = charge_sum(&PEER_ENERGY_PACKETS);
    let share = ExactFraction::new(3, 5).unwrap();
    let peer_share = peer_native.mul(share).unwrap();
    let wearer_share = wearer_native.mul(share).unwrap();
    let expected = [
        wearer_native.add(peer_share).unwrap(),
        peer_native.add(wearer_share).unwrap(),
    ];
    for actor in 0..2 {
        let state = &report["final_actor_states"][actor];
        let granted = ExactFraction::new(state["granted_energy"].as_u64().unwrap().into(), 1)
            .unwrap()
            .add(exact(&state["charge_remainder"]))
            .unwrap();
        assert_eq!(
            granted, expected[actor],
            "actor {actor} charge, tier {tier}"
        );
        assert_eq!(state["spent_energy"], 0, "no bursts spend energy");
    }
}

#[test]
fn snowman_continuing_cycles_toggle_the_shielded_atk_zone_across_blessing_windows() {
    for tier in 1..=5u32 {
        let req = cycles_request(tier);
        assert_eq!(req.actors[0].initial_hp, 14302, "native lvl80 wearer HP");
        assert_eq!(req.actors[1].initial_hp, 12120, "native lvl80 peer HP");
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_shield_trace(&report);
        assert_wearer_melee_windows(&report, tier);
        assert_def_scaled_skill_packets(&report);
        assert_peer_chains(&report);
        assert_charge_add_energy_oracle(&report, tier);
        assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 56);
        assert_eq!(report["final_actor_states"][0]["hp"], 14302);
        assert_eq!(report["final_actor_states"][1]["hp"], 12120);
        assert!(
            report["final_actor_states"][0]
                .get("active_shields")
                .and_then(serde_json::Value::as_array)
                .is_none_or(Vec::is_empty),
            "both Blessing shields expired before the horizon"
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
