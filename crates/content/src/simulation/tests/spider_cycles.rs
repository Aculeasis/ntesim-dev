use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/spider-knowledge-cycles.nte");

/// Sakiri's five-stage native chain: stage packet groups 1/2/2/3/2, every
/// packet category A. Stages run 0.8s apart, so each attack's primary packet
/// grants one Spider Knowledge stack and its same-instant siblings stay
/// ICD-rejected.
const WEARER_STAGES: [&[(&str, u32)]; 5] = [
    &[("GE_Player_Sagiri_Melee1_Damage", 1)],
    &[
        ("GE_Player_Sagiri_Melee2_Damage", 1),
        ("GE_Player_Sagiri_Melee3_Damage", 1),
    ],
    &[
        ("GE_Player_Sagiri_Melee8_Damage", 1),
        ("GE_Player_Sagiri_Melee4_Damage", 1),
    ],
    &[
        ("GE_Player_Sagiri_Melee5_Damage", 1),
        ("GE_Player_Sagiri_Melee5_Damage", 1),
        ("GE_Player_Sagiri_Melee5_Damage", 1),
    ],
    &[
        ("GE_Player_Sagiri_Melee6_Damage", 1),
        ("GE_Player_Sagiri_Melee7_Damage", 1),
    ],
];
/// Chain visits as (stage indices, stage times): the five-stack window
/// [4s, 14s) contains no wearer melee (only the Burst and Support packets),
/// the boundary hit at exactly 14s lands bare while it reopens the stack
/// count, and everything between the windows stays bare.
const WEARER_VISITS: [(&[usize], &[u64]); 6] = [
    (
        &[0, 1, 2, 3, 4],
        &[0, 800_000, 1_600_000, 2_400_000, 3_200_000],
    ),
    (&[0], &[16_000_000]),
    (&[1, 2, 3], &[16_800_000, 17_600_000, 18_400_000]),
    (&[4], &[19_200_000]),
    (
        &[0, 1, 2, 3, 4],
        &[20_000_000, 20_800_000, 21_600_000, 22_400_000, 23_200_000],
    ),
    (
        &[0, 1, 2, 3, 4, 0, 1],
        &[
            28_900_000, 29_700_000, 30_500_000, 31_300_000, 32_100_000, 32_900_000, 33_700_000,
        ],
    ),
];
/// Ultimate casts consume the stacks and open the windows: five stacks lift
/// the whole team by 5 x per-stack, eight stacks add the exact-eight bonus.
const BURSTS: [(u64, usize); 2] = [(4_000_000, 5), (34_500_000, 8)];
const BURST_PACKETS: [(&str, u32); 2] = [
    ("GE_Player_Sagiri_UltraSkill1_Damage", 5),
    ("GE_Player_Sagiri_UltraSkill2_Damage", 1),
];
/// Frozen CT_ForkBuff fractions: 8 x AtkUp totals plus AtkUp2 (hand-reduced in
/// spider.rs); five-stack windows carry 5 x AtkUp only.
const EIGHT_STACK_TIERS: [(u64, u64); 5] = [(1, 10), (23, 200), (13, 100), (29, 200), (4, 25)];
const FIVE_STACK_TIERS: [(u64, u64); 5] = [(1, 20), (23, 400), (13, 200), (29, 400), (2, 25)];
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
const BURST_BASE_DAMAGE: [(&str, u64); 2] = [
    ("GE_Player_Sagiri_UltraSkill1_Damage", 1200),
    ("GE_Player_Sagiri_UltraSkill2_Damage", 3000),
];
const QTE_PACKET: &str = "GE_Player_Sagiri_QTE1_Damage";
/// Swap-in Support dispatches (command time + 0.1s hit): the first lands in
/// the five-stack window, the second between the windows.
const QTE_HITS: [(u64, bool); 2] = [(10_600_000, true), (28_350_000, false)];
const QTE_BASE_DAMAGE: u64 = 2000;
/// Whole-scenario packet multisets for the ChargeAdd energy oracle.
const WEARER_ENERGY_PACKETS: [(&str, u32); 9] = [
    ("GE_Player_Sagiri_Melee1_Damage", 5),
    ("GE_Player_Sagiri_Melee2_Damage", 5),
    ("GE_Player_Sagiri_Melee3_Damage", 5),
    ("GE_Player_Sagiri_Melee4_Damage", 4),
    ("GE_Player_Sagiri_Melee5_Damage", 12),
    ("GE_Player_Sagiri_Melee6_Damage", 4),
    ("GE_Player_Sagiri_Melee7_Damage", 4),
    ("GE_Player_Sagiri_Melee8_Damage", 4),
    ("GE_Player_Sagiri_QTE1_Damage", 2),
];
const PEER_ENERGY_PACKETS: [(&str, u32); 8] = [
    ("GE_Player_Mint_Melee1_Damage_New", 3),
    ("GE_Player_Mint_Melee2_Damage_New", 3),
    ("GE_Player_Mint_Melee3_Damage_New", 3),
    ("GE_Player_Mint_Melee4_Damage_New", 3),
    ("GE_Player_Mint_Melee5_Damage_New", 3),
    ("GE_Player_Mint_Melee6_Damage_New", 3),
    ("GE_Player_Mint_Melee7_Damage_New", 3),
    ("GE_Player_Mint_Melee8_Damage_New", 3),
];
/// Three off-field Mint chains: the first rides the five-stack window, the
/// second sits between windows, the third rides the eight-stack window.
const PEER_CHAINS: [(u64, bool, bool); 3] = [
    (6_250_000, true, false),
    (24_250_000, false, false),
    (36_750_000, true, true),
];
const PEER_STAGES: [&[(&str, u32)]; 5] = [
    &[
        ("GE_Player_Mint_Melee1_Damage_New", 1),
        ("GE_Player_Mint_Melee2_Damage_New", 1),
    ],
    &[("GE_Player_Mint_Melee3_Damage_New", 1)],
    &[
        ("GE_Player_Mint_Melee4_Damage_New", 1),
        ("GE_Player_Mint_Melee5_Damage_New", 1),
    ],
    &[("GE_Player_Mint_Melee6_Damage_New", 1)],
    &[
        ("GE_Player_Mint_Melee7_Damage_New", 1),
        ("GE_Player_Mint_Melee8_Damage_New", 1),
    ],
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

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    // Flat ATK 1000 with zeroed static bonus zones keeps every packet oracle
    // on the bare native coefficient plus the consumed-stack team ATK zone.
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
    req
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

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

/// Live ATK zone: base packet coefficient times (1 + consumed-stack percent);
/// the buff stat adds base_atk x p, not a multiple of the static ATK.
fn atk_zone_damage(base: u64, zone: (u128, u128)) -> ExactFraction {
    ExactFraction::new(u128::from(base), 1)
        .unwrap()
        .mul(
            ExactFraction::new(1, 1)
                .unwrap()
                .add(ExactFraction::new(zone.0, zone.1).unwrap())
                .unwrap(),
        )
        .unwrap()
}

fn tier_zone(tiers: [(u64, u64); 5], tier: u32) -> (u128, u128) {
    let (numerator, denominator) = tiers[(tier - 1) as usize];
    (u128::from(numerator), u128::from(denominator))
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

fn assert_wearer_chains(report: &Value) {
    let hits = actor_hits(report, 0, "GE_Player_Sagiri_Melee");
    let mut expected = Vec::new();
    for (stages, times) in WEARER_VISITS.iter() {
        for (slot, time) in times.iter().enumerate() {
            for (effect, count) in WEARER_STAGES[stages[slot]] {
                for _ in 0..*count {
                    expected.push((*time, *effect));
                }
            }
        }
    }
    assert_eq!(hits.len(), expected.len(), "wearer chain packets");
    for (hit, (time, effect)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        assert_eq!(hit["damage_effect_id"], *effect);
        assert_eq!(hit["critical"], false);
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            ExactFraction::new(u128::from(base_damage(&MELEE_BASE_DAMAGE, effect)), 1).unwrap(),
            "bare {effect} at {time}"
        );
    }
}

fn assert_burst_windows(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Sagiri_UltraSkill");
    let mut expected = Vec::new();
    for (time, stacks) in BURSTS.iter() {
        for (effect, count) in BURST_PACKETS.iter() {
            for _ in 0..*count {
                expected.push((*time, *effect, *stacks == 8));
            }
        }
    }
    assert_eq!(hits.len(), expected.len(), "tier {tier} burst packets");
    for (hit, (time, effect, eight_stacks)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time, "tier {tier}");
        assert_eq!(hit["damage_effect_id"], *effect, "tier {tier}");
        let zone = if *eight_stacks {
            tier_zone(EIGHT_STACK_TIERS, tier)
        } else {
            tier_zone(FIVE_STACK_TIERS, tier)
        };
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            atk_zone_damage(base_damage(&BURST_BASE_DAMAGE, effect), zone),
            "tier {tier} burst {effect} at {time} eight stacks: {eight_stacks}"
        );
    }
}

fn assert_support_qte_packets(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Sagiri_QTE");
    assert_eq!(hits.len(), QTE_HITS.len(), "tier {tier} Support packets");
    for (hit, (time, boosted)) in hits.iter().zip(QTE_HITS.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        assert_eq!(hit["damage_effect_id"], QTE_PACKET);
        let expected_damage = if *boosted {
            atk_zone_damage(QTE_BASE_DAMAGE, tier_zone(FIVE_STACK_TIERS, tier))
        } else {
            ExactFraction::new(u128::from(QTE_BASE_DAMAGE), 1).unwrap()
        };
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            expected_damage,
            "tier {tier} Support packet at {time} in window: {boosted}"
        );
    }
}

fn assert_mint_chains(report: &Value, tier: u32) {
    let hits = actor_hits(report, 1, "GE_Player_Mint_Melee");
    let mut expected = Vec::new();
    for (chain, boosted, eight_stacks) in PEER_CHAINS.iter() {
        for (stage, delay) in [0u64, 800_000, 1_600_000, 2_400_000, 3_200_000]
            .into_iter()
            .enumerate()
        {
            for (effect, count) in PEER_STAGES[stage] {
                for _ in 0..*count {
                    expected.push((chain + delay, *effect, *boosted, *eight_stacks));
                }
            }
        }
    }
    assert_eq!(hits.len(), expected.len(), "tier {tier} peer packets");
    for (hit, (time, effect, boosted, eight_stacks)) in hits.iter().zip(expected.iter()) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time, "tier {tier}");
        assert_eq!(hit["damage_effect_id"], *effect, "tier {tier}");
        let base = u128::from(base_damage(&PEER_BASE_DAMAGE, effect));
        let zone = match (*boosted, *eight_stacks) {
            (true, false) => tier_zone(FIVE_STACK_TIERS, tier),
            (true, true) => tier_zone(EIGHT_STACK_TIERS, tier),
            (false, _) => (0, 1),
        };
        // Ordinary Burst adds source-base1000 * .30 for twenty seconds,
        // independently of Spider's ten-second per-recipient ATK window.
        let mut damage = atk_zone_damage(u64::try_from(base).unwrap(), zone);
        if BURSTS.iter().any(|(start, _)| {
            support::world_time_at(report, *start) <= support::world_time_at(report, *time)
                && support::world_time_at(report, *time)
                    < support::world_time_at(report, *start) + 20_000_000
        }) {
            damage = damage
                .add(ExactFraction::new(base * 3, 10).unwrap())
                .unwrap();
        }
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            damage,
            "tier {tier} peer {effect} at {time} in a window: {boosted}"
        );
    }
}

/// Stack grants: one per chain attack (same-instant siblings are ICD-rejected
/// without trace), capped at eight — capped attempts repeat the 8/8 detail.
/// The counter restarts after each consuming Ultimate (first grant at 14s).
fn assert_knowledge_trace(report: &Value) {
    let grant_times: Vec<u64> = WEARER_VISITS
        .iter()
        .flat_map(|(_, times)| times.iter().copied())
        .collect();
    let trace = report["trace"].as_array().unwrap();
    let events: Vec<&Value> = trace
        .iter()
        .filter(|event| event["event_kind"] == "spider_knowledge_stack")
        .collect();
    assert_eq!(
        events.len(),
        grant_times.len(),
        "one stack per chain attack"
    );
    let mut stacks = 0u32;
    for (event, time) in events.iter().zip(grant_times) {
        if time == 16_000_000 {
            stacks = 0;
        }
        stacks = (stacks + 1).min(8);
        assert_eq!(event["time_us"].as_u64().unwrap(), time);
        assert_eq!(event["from_actor"], 0);
        assert_eq!(event["data"]["stacks"], stacks);
        assert_eq!(event["data"]["max_stacks"], 8);
    }
}

fn assert_team_atk_trace(report: &Value, tier: u32) {
    let trace = report["trace"].as_array().unwrap();
    let events: Vec<&Value> = trace
        .iter()
        .filter(|event| event["event_kind"] == "spider_team_atk")
        .collect();
    assert_eq!(events.len(), BURSTS.len(), "one grant per consuming cast");
    for (event, (time, stacks)) in events.iter().zip(BURSTS.iter()) {
        assert_eq!(event["time_us"].as_u64().unwrap(), *time);
        let (numerator, denominator) = if *stacks == 8 {
            tier_zone(EIGHT_STACK_TIERS, tier)
        } else {
            tier_zone(FIVE_STACK_TIERS, tier)
        };
        let expected_atk = ExactFraction::new(numerator, denominator).unwrap();
        assert_eq!(
            event["data"]["atk_percent"],
            serde_json::to_value(expected_atk).unwrap()
        );
        assert_eq!(event["data"]["duration_us"], 10_000_000);
    }
}

/// Owner packets pay ChargeAdd x1 and every peer x3/5 (relations::DISTRIBUTION);
/// the expected sums re-read the frozen native ChargeAdd rows independently.
/// The two funded Ultimates spend 240 on the wearer; the peer never spends.
fn assert_energy_oracle(report: &Value, tier: u32) {
    let natives = [
        charge_sum(&WEARER_ENERGY_PACKETS),
        charge_sum(&PEER_ENERGY_PACKETS),
    ];
    let mut expected = [
        ExactFraction::new(0, 1).unwrap(),
        ExactFraction::new(0, 1).unwrap(),
    ];
    for (actor, native) in natives.iter().enumerate() {
        let share = natives[1 - actor]
            .mul(ExactFraction::new(3, 5).unwrap())
            .unwrap();
        expected[actor] = native.add(share).unwrap();
    }
    expected[0] = expected[0].mul(ExactFraction::new(6, 5).unwrap()).unwrap();
    for (actor, total) in expected.iter().enumerate() {
        let state = &report["final_actor_states"][actor];
        let remainder = exact(&state["charge_remainder"]);
        let granted = remainder
            .add(ExactFraction::new(state["granted_energy"].as_u64().unwrap().into(), 1).unwrap());
        assert_eq!(
            granted.unwrap(),
            *total,
            "actor {actor} charge, tier {tier}"
        );
        let spent = if actor == 0 { 240 } else { 0 };
        assert_eq!(
            state["spent_energy"].as_u64().unwrap(),
            spent,
            "actor {actor} Ultimate spends, tier {tier}"
        );
    }
}

#[test]
fn spider_continuing_cycles_consume_knowledge_stacks_into_team_atk_windows() {
    for tier in 1..=5u32 {
        let req = cycles_request(tier);
        assert_eq!(req.actors[0].initial_hp, 16483, "native lvl80 wearer HP");
        assert_eq!(req.actors[1].initial_hp, 12120, "native lvl80 peer HP");
        assert_eq!(req.actors[0].initial_energy, 120, "funded first Ultimate");
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_knowledge_trace(&report);
        assert_team_atk_trace(&report, tier);
        assert_wearer_chains(&report);
        assert_burst_windows(&report, tier);
        assert_support_qte_packets(&report, tier);
        assert_mint_chains(&report, tier);
        assert_energy_oracle(&report, tier);
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] != "ntesim.reaction.hexed_followup")
                .count(),
            81
        );
        assert_eq!(report["final_actor_states"][0]["hp"], 16483);
        assert_eq!(report["final_actor_states"][1]["hp"], 12120);
        let instances = crate::simulation::tests::support::final_buffs(&report);
        assert!(
            instances
                .iter()
                .all(|buff| buff["id"] != "spider.knowledge"),
            "the second Ultimate consumed every stack"
        );
        assert!(
            instances
                .iter()
                .any(|buff| buff["id"] == "buff_fork_spider_team_atk"
                    && buff["duration"]["expires_at_us"] == 42_500_000),
            "the second window outlives the horizon"
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
