use super::*;
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../../../examples/prism-gravitide-cycles.nte");

/// Daffodil's five-stage native chain (1/2/6/3/7 category-A packets); every
/// frozen Daffodil packet is Chaos-typed, so the window's Lakshana zone has no
/// native packet to bind and stays covered by the synthetic unit tests.
const WEARER_STAGES: [&[(&str, u32)]; 5] = [
    &[("GE_Player_Daffodill_Melee1_Damage", 1)],
    &[
        ("GE_Player_Daffodill_Melee2_Damage", 1),
        ("GE_Player_Daffodill_Melee2_1_Damage", 1),
    ],
    &[
        ("GE_Player_Daffodill_Melee3_Damage", 2),
        ("GE_Player_Daffodill_Melee3_1_Damage", 2),
        ("GE_Player_Daffodill_Melee3_2_Damage", 1),
        ("GE_Player_Daffodill_Melee3_3_Damage", 1),
    ],
    &[
        ("GE_Player_Daffodill_Melee4_Damage", 1),
        ("GE_Player_Daffodill_Melee4_1_Damage", 1),
        ("GE_Player_Daffodill_Melee4_2_Damage", 1),
    ],
    &[
        ("GE_Player_Daffodill_Melee5_Damage", 2),
        ("GE_Player_Daffodill_Melee6_Damage", 4),
        ("GE_Player_Daffodill_Melee5_1_Damage", 1),
    ],
];
/// Two native Redirect admissions, with an authored wait for the notify CD.
const SKILL_CASTS: [u64; 2] = [0, 17_000_000];
const SKILL_PACKETS: [(&str, u32); 7] = [
    ("GE_Player_Daffodill_Skill1_Damage", 2),
    ("GE_Player_Daffodill_Skill2_Damage", 1),
    ("GE_Player_Daffodill_Skill6_Damage", 1),
    ("GE_Player_Daffodill_Skill7_Damage", 1),
    ("GE_Player_Daffodill_Skill8_Damage", 1),
    ("GE_Player_Daffodill_Skill9_Damage", 1),
    ("GE_Player_Daffodill_Skill5_Damage", 1),
];
/// Native lvl80 ATK coefficients (AtkRateBaseArray[0]) at the static ATK 1000.
const MELEE_BASE_DAMAGE: [(&str, u64); 13] = [
    ("GE_Player_Daffodill_Melee1_Damage", 167),
    ("GE_Player_Daffodill_Melee2_Damage", 153),
    ("GE_Player_Daffodill_Melee2_1_Damage", 153),
    ("GE_Player_Daffodill_Melee3_Damage", 132),
    ("GE_Player_Daffodill_Melee3_1_Damage", 62),
    ("GE_Player_Daffodill_Melee3_2_Damage", 141),
    ("GE_Player_Daffodill_Melee3_3_Damage", 263),
    ("GE_Player_Daffodill_Melee4_Damage", 524),
    ("GE_Player_Daffodill_Melee4_1_Damage", 193),
    ("GE_Player_Daffodill_Melee4_2_Damage", 127),
    ("GE_Player_Daffodill_Melee5_Damage", 101),
    ("GE_Player_Daffodill_Melee6_Damage", 101),
    ("GE_Player_Daffodill_Melee5_1_Damage", 473),
];
const SKILL_BASE_DAMAGE: [(&str, u64); 7] = [
    ("GE_Player_Daffodill_Skill1_Damage", 409),
    ("GE_Player_Daffodill_Skill2_Damage", 337),
    ("GE_Player_Daffodill_Skill6_Damage", 337),
    ("GE_Player_Daffodill_Skill7_Damage", 337),
    ("GE_Player_Daffodill_Skill8_Damage", 337),
    ("GE_Player_Daffodill_Skill9_Damage", 337),
    ("GE_Player_Daffodill_Skill5_Damage", 498),
];
/// Card table: DamageUp1 = DamageUp2 (Redirect Skill and Basic Attack zones)
/// 28/35/42/49/56 percent; CritUp = LakshanaUp 16/20/24/28/32 percent.
const DAMAGE_TIERS: [(u64, u64); 5] = [(7, 25), (7, 20), (21, 50), (49, 100), (14, 25)];
const CRIT_TIERS: [(u64, u64); 5] = [(4, 25), (1, 5), (6, 25), (7, 25), (8, 25)];
/// Whole-scenario packet multisets for the ChargeAdd energy oracle.
const WEARER_ENERGY_PACKETS: [(&str, u32); 20] = [
    ("GE_Player_Daffodill_Melee1_Damage", 3),
    ("GE_Player_Daffodill_Melee2_Damage", 3),
    ("GE_Player_Daffodill_Melee2_1_Damage", 3),
    ("GE_Player_Daffodill_Melee3_Damage", 4),
    ("GE_Player_Daffodill_Melee3_1_Damage", 4),
    ("GE_Player_Daffodill_Melee3_2_Damage", 2),
    ("GE_Player_Daffodill_Melee3_3_Damage", 2),
    ("GE_Player_Daffodill_Melee4_Damage", 2),
    ("GE_Player_Daffodill_Melee4_1_Damage", 2),
    ("GE_Player_Daffodill_Melee4_2_Damage", 2),
    ("GE_Player_Daffodill_Melee5_Damage", 4),
    ("GE_Player_Daffodill_Melee6_Damage", 8),
    ("GE_Player_Daffodill_Melee5_1_Damage", 2),
    ("GE_Player_Daffodill_Skill1_Damage", 4),
    ("GE_Player_Daffodill_Skill2_Damage", 2),
    ("GE_Player_Daffodill_Skill6_Damage", 2),
    ("GE_Player_Daffodill_Skill7_Damage", 2),
    ("GE_Player_Daffodill_Skill8_Damage", 2),
    ("GE_Player_Daffodill_Skill9_Damage", 2),
    ("GE_Player_Daffodill_Skill5_Damage", 2),
];
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
// Entry replaces stage1; only its first contact adds a pause before hit2.
// Later source inputs retain their own native section and settled start time.
const PEER_STAGES: [&[(&str, u64)]; 5] = [
    &[
        ("GE_Player_Mint_Melee1_Damage_New", 199890),
        ("GE_Player_Mint_Melee2_Damage_New", 366642),
    ],
    &[("GE_Player_Mint_Melee3_Damage_New", 89514)],
    &[
        ("GE_Player_Mint_Melee4_Damage_New", 241903),
        ("GE_Player_Mint_Melee5_Damage_New", 378755),
    ],
    &[("GE_Player_Mint_Melee6_Damage_New", 149935)],
    &[
        ("GE_Player_Mint_Melee7_Damage_New", 259318),
        ("GE_Player_Mint_Melee8_Damage_New", 600077),
    ],
];
const PEER_ENERGY_PACKETS: [(&str, u32); 8] = [
    ("GE_Player_Mint_Melee1_Damage_New", 2),
    ("GE_Player_Mint_Melee2_Damage_New", 2),
    ("GE_Player_Mint_Melee3_Damage_New", 2),
    ("GE_Player_Mint_Melee4_Damage_New", 2),
    ("GE_Player_Mint_Melee5_Damage_New", 2),
    ("GE_Player_Mint_Melee6_Damage_New", 2),
    ("GE_Player_Mint_Melee7_Damage_New", 2),
    ("GE_Player_Mint_Melee8_Damage_New", 2),
];

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

fn cycles(tier: u32) -> crate::simulation::SourceScenarioReport {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let (script, mut config) = crate::dsl::inspection::prepare(&source).unwrap();
    // Flat ATK 1000 with zeroed static bonus zones leaves the weapon zones as
    // the only damage modifiers; the crit-rate receipt keeps the weapon bonus.
    for compiled in &mut config.actors {
        let stats = &mut compiled.actor.static_stats;
        stats.elemental_bonuses.clear();
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        compiled.projection.static_stats = compiled.actor.static_stats.clone();
    }
    let result = crate::simulation::PreparedSourceScenario::from_configuration(
        &source,
        config,
        Default::default(),
        &ExecutionLimits::default(),
        &script.options[0].loc,
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    result
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

/// M_bonus = 1 + zone (zones 1/3); DEF and RES factors are 1 at def=0/resist=0.
fn zoned(base: u64, zone: (u64, u64)) -> ExactFraction {
    let bonus = ExactFraction::new(u128::from(zone.0), u128::from(zone.1)).unwrap();
    ExactFraction::new(u128::from(base), 1)
        .unwrap()
        .mul(ExactFraction::new(1, 1).unwrap().add(bonus).unwrap())
        .unwrap()
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

fn base_damage(table: &[(&str, u64)], effect: &str) -> u64 {
    table
        .iter()
        .find(|(candidate, _)| *candidate == effect)
        .unwrap_or_else(|| panic!("unexpected packet {effect}"))
        .1
}

fn assert_skill_packets(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Daffodill_Skill");
    assert_eq!(hits.len(), 16, "skill packets, tier {tier}");
    for (effect, count) in SKILL_PACKETS {
        assert_eq!(
            hits.iter()
                .filter(|hit| hit["damage_effect_id"] == effect)
                .count(),
            2 * count as usize,
            "{effect}"
        );
    }
    for time in SKILL_CASTS {
        assert!(hits.iter().any(|hit| hit["damage_effect_id"]
            == "GE_Player_Daffodill_Skill1_Damage"
            && hit["time_us"] == time + 196_675));
    }
    let zone = DAMAGE_TIERS[(tier - 1) as usize];
    for hit in hits {
        let effect = hit["damage_effect_id"].as_str().unwrap();
        let time = hit["time_us"].as_u64().unwrap();
        assert_eq!(hit["critical"], false);
        let expected = if time < 17_000_000 {
            ExactFraction::new(base_damage(&SKILL_BASE_DAMAGE, effect).into(), 1).unwrap()
        } else {
            zoned(
                base_damage(&SKILL_BASE_DAMAGE, effect),
                (zone.0, zone.1 * 2),
            )
        };
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            expected,
            "tier {tier} {effect} at {time}"
        );
    }
}

fn assert_wearer_windows(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Daffodill_Melee");
    let mut expected = std::collections::BTreeMap::new();
    for stages in [5, 2, 5] {
        for stage in &WEARER_STAGES[..stages] {
            for &(effect, count) in *stage {
                *expected.entry(effect).or_insert(0) += count;
            }
        }
    }
    assert_eq!(hits.len(), 41, "melee packets, tier {tier}");
    for (effect, count) in expected {
        assert_eq!(
            hits.iter()
                .filter(|hit| hit["damage_effect_id"] == effect)
                .count(),
            count as usize,
            "tier {tier} {effect}"
        );
    }
    let zone = DAMAGE_TIERS[(tier - 1) as usize];
    let mut expired = 0;
    for hit in hits {
        let time = hit["time_us"].as_u64().unwrap();
        let effect = hit["damage_effect_id"].as_str().unwrap();
        let inside = SKILL_CASTS
            .iter()
            .any(|start| time >= *start && time < start + 15_000_000);
        let expected = if inside {
            zoned(base_damage(&MELEE_BASE_DAMAGE, effect), zone)
        } else {
            expired += 1;
            ExactFraction::new(base_damage(&MELEE_BASE_DAMAGE, effect).into(), 1).unwrap()
        };
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            expected,
            "tier {tier} {effect} at {time}"
        );
    }
    assert_eq!(expired, 2);
    assert!(
        actor_hits(report, 0, "GE_Player_Daffodill_Melee")
            .iter()
            .any(|hit| hit["time_us"] == 15_000_000
                && hit["damage_effect_id"] == "GE_Player_Daffodill_Melee2_Damage")
    );
}

fn assert_window_trace(report: &Value) {
    let events: Vec<&Value> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "far_side_moon_redirect_window")
        .collect();
    assert_eq!(events.len(), SKILL_CASTS.len(), "window refresh events");
    for (event, time) in events.iter().zip(SKILL_CASTS) {
        assert_eq!(event["time_us"].as_u64().unwrap(), time);
        assert_eq!(event["from_actor"], 0);
        assert_eq!(event["data"]["expires_at_us"], time + 15_000_000);
    }
}

/// With crit=fixed and no research roll policy the tier CritUp never fires a
/// critical, but every owner packet receipt still carries the added rate.
fn assert_crit_rate_receipts(report: &Value, tier: u32) {
    let (numerator, denominator) = CRIT_TIERS[(tier - 1) as usize];
    for hit in report["hit_receipts"].as_array().unwrap() {
        assert_eq!(hit["critical"], false);
        let expected = if hit["actor_index"] == 0 {
            json!(format!("{}/{}", numerator, denominator))
        } else {
            json!("0/1")
        };
        assert_eq!(hit["crit_rate"], expected, "tier {tier}");
    }
}

fn assert_peer_chains(report: &Value, req: &GameScenarioRequest, tier: u32) {
    let hits = actor_hits(report, 1, "GE_Player_Mint_Melee");
    let expected = peer_schedule(req);
    assert_eq!(hits.len(), expected.len(), "tier {tier} peer packets");
    for (hit, (time, effect)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time, "tier {tier}");
        assert_eq!(hit["damage_effect_id"], *effect, "tier {tier}");
        let base =
            ExactFraction::new(u128::from(base_damage(&PEER_BASE_DAMAGE, effect)), 1).unwrap();
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            base,
            "tier {tier} peer {effect}"
        );
    }
    let midpoint = expected.len() / 2;
    for (early, late) in expected[..midpoint].iter().zip(expected[midpoint..].iter()) {
        assert_eq!(late.0 - early.0, 17_000_000, "identical peer chain offset");
        assert_eq!(early.1, late.1, "identical peer chain packet");
    }
}

fn peer_schedule(req: &GameScenarioRequest) -> Vec<(u64, &'static str)> {
    let actions: Vec<_> = req
        .actions
        .iter()
        .filter_map(|action| match action {
            GameActionCommand::InvokeAbility {
                actor_index: 1,
                timestamp_us,
                ..
            } => Some(*timestamp_us),
            _ => None,
        })
        .collect();
    assert_eq!(actions.len(), 10);
    let mut expected = Vec::new();
    for (input, time) in actions.into_iter().enumerate() {
        for &(effect, delay) in PEER_STAGES[input % 5] {
            expected.push((time + delay, effect));
        }
    }
    expected
}

/// Owner packets pay ChargeAdd x1 and every peer x3/5 (relations::DISTRIBUTION);
/// the expected sums re-read the frozen native ChargeAdd rows independently.
fn assert_charge_add_energy_oracle(report: &Value, tier: u32) {
    let wearer = charge_sum(&WEARER_ENERGY_PACKETS);
    let peer = charge_sum(&PEER_ENERGY_PACKETS);
    let share = ExactFraction::new(3, 5).unwrap();
    let expected = [
        wearer.add(peer.mul(share).unwrap()).unwrap(),
        peer.add(wearer.mul(share).unwrap()).unwrap(),
    ];
    for (actor, total) in expected.iter().enumerate() {
        let state = &report["final_actor_states"][actor];
        assert_eq!(
            granted_total(state),
            *total,
            "actor {actor} continuing charge, tier {tier}"
        );
        assert_eq!(state["spent_energy"], 0, "no bursts spend energy");
    }
}

fn granted_total(state: &Value) -> ExactFraction {
    ExactFraction::new(state["granted_energy"].as_u64().unwrap().into(), 1)
        .unwrap()
        .add(exact(&state["charge_remainder"]))
        .unwrap()
}

#[test]
fn prism_continuing_cycles_refresh_and_expire_the_redirect_window_with_native_energy() {
    for tier in 1..=5u32 {
        let source = cycles(tier);
        let req = source.request;
        assert_eq!(req.actors[0].initial_hp, 16180, "native lvl80 wearer HP");
        assert_eq!(req.actors[1].initial_hp, 12120, "native lvl80 peer HP");
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_eq!(
            report,
            serde_json::to_value(source.report).unwrap(),
            "Prism source/native T{tier}"
        );
        assert_skill_packets(&report, tier);
        assert_wearer_windows(&report, tier);
        assert_window_trace(&report);
        assert_crit_rate_receipts(&report, tier);
        assert_peer_chains(&report, &req, tier);
        assert_charge_add_energy_oracle(&report, tier);
        assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 73);
        assert_eq!(report["final_actor_states"][0]["hp"], 16180);
        assert_eq!(report["final_actor_states"][1]["hp"], 12120);
        assert!(
            crate::simulation::tests::support::final_buffs(&report)
                .iter()
                .all(|buff| buff["id"] != "far_side_moon.redirect_window"
                    && buff["id"] != "far_side_moon.skill_stacks"),
            "the refreshed window and stacks expired before the horizon"
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
