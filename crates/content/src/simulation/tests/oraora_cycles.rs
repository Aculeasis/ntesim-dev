use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/oraora-puncher-cycles.nte");

/// Three native Nanally chain visits: five legal inputs whose stages deliver
/// 1/1/3/7/2 category-A packets, each granting one independent 10s stack.
const WEARER_STAGES: [&[(&str, u32)]; 5] = [
    &[("GE_Player_Nanally_Melee1_Damage", 1)],
    &[("GE_Player_Nanally_Melee2_Damage", 1)],
    &[
        ("GE_Player_Nanally_Melee3_Damage", 1),
        ("GE_Player_Nanally_Melee4_Damage", 1),
        ("GE_Player_Nanally_Melee5_Damage", 1),
    ],
    &[("GE_Player_Nanally_Melee6_Damage", 7)],
    &[
        ("GE_Player_Nanally_Melee9_Damage", 1),
        ("GE_Player_Nanally_Melee10_Damage", 1),
    ],
];
const WEARER_STARTS: [u64; 3] = [0, 14_000_000, 35_000_000];
// Independent rate-one markers for inputs1.5s apart, including Normal5's first pause.
const WEARER_OFFSETS: [u64; 14] = [
    191054, 1709023, 3232926, 3316815, 3508139, 4849478, 4899464, 5008732, 5130868, 5217125,
    5270502, 5333821, 6249362, 6860864,
];
// Packets now settle separately: earlier markers grant stacks before later damage.
const WEARER_STACKS: [[u32; 14]; 3] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10],
    [5, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10],
];
/// Native lvl80 ATK coefficients (AtkRateBaseArray[0]) at the static ATK 1000.
const WEARER_BASE_DAMAGE: [(&str, u64); 8] = [
    ("GE_Player_Nanally_Melee1_Damage", 296),
    ("GE_Player_Nanally_Melee2_Damage", 388),
    ("GE_Player_Nanally_Melee3_Damage", 122),
    ("GE_Player_Nanally_Melee4_Damage", 210),
    ("GE_Player_Nanally_Melee5_Damage", 1270),
    ("GE_Player_Nanally_Melee6_Damage", 69),
    ("GE_Player_Nanally_Melee9_Damage", 771),
    ("GE_Player_Nanally_Melee10_Damage", 773),
];
const WEARER_QTE: (&str, [u64; 2], u64) = (
    "GE_Player_Nanally_QTE1_Damage",
    [12_666_410, 33_666_410],
    2000,
);
/// Two identical off-field Mint chains (stage packets 2/1/2/1/2 at the same
/// 0.8s cadence); the peer holds no Oraora! proc, so damage stays constant.
const PEER_CHAINS: [u64; 2] = [7_750_000, 27_250_000];
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
/// Whole-scenario packet multisets for the ChargeAdd energy oracle: three
/// wearer chains (Melee6 seven times each) plus two switch-in Support QTEs,
/// and two peer chains.
const WEARER_ENERGY_PACKETS: [(&str, u32); 9] = [
    ("GE_Player_Nanally_Melee1_Damage", 3),
    ("GE_Player_Nanally_Melee2_Damage", 3),
    ("GE_Player_Nanally_Melee3_Damage", 3),
    ("GE_Player_Nanally_Melee4_Damage", 3),
    ("GE_Player_Nanally_Melee5_Damage", 3),
    ("GE_Player_Nanally_Melee6_Damage", 21),
    ("GE_Player_Nanally_Melee9_Damage", 3),
    ("GE_Player_Nanally_Melee10_Damage", 3),
    ("GE_Player_Nanally_QTE1_Damage", 2),
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

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    // Flat ATK 1000 with no crit and no static bonus zones keeps every packet
    // oracle on the bare native coefficient plus the passive's stack zone.
    for actor in &mut req.actors {
        let stats = &mut actor.static_stats;
        stats.elemental_bonuses = Default::default();
        stats.elemental_bonus = Ratio::zero();
        stats.damage_bonus = Ratio::zero();
        stats.crit_rate = Ratio::zero();
        stats.atk = Ratio {
            denominator: 1,
            numerator: 1000,
        };
        stats.base_atk = 1000;
    }
    req
}

/// Exact receipt fractions deserialize straight from numerator/denominator.
fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
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

/// Card table 2.0/2.4/2.8/3.2/3.6 percent per stack at T1..T5.
fn per_stack(tier: u32) -> ExactFraction {
    let (numerator, denominator) = match tier {
        1 => (1, 50),
        2 => (3, 125),
        3 => (7, 250),
        4 => (4, 125),
        5 => (9, 250),
        other => panic!("unsupported tier {other}"),
    };
    ExactFraction::new(numerator, denominator).unwrap()
}

/// M_bonus = 1 + n stacks of the tier's per-stack damage bonus (zones 1/3);
/// DEF and RES factors are 1 at def=0/resist=0 and crit is fixed off.
fn expected_stack_damage(base: u64, stacks: u32, tier: u32) -> ExactFraction {
    let bonus = per_stack(tier)
        .mul(ExactFraction::new(u128::from(stacks), 1).unwrap())
        .unwrap();
    ExactFraction::new(u128::from(base), 1)
        .unwrap()
        .mul(ExactFraction::new(1, 1).unwrap().add(bonus).unwrap())
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

fn assert_wearer_stack_windows(report: &Value, tier: u32) {
    let hits = actor_hits(report, 0, "GE_Player_Nanally_Melee");
    let mut expected = Vec::new();
    let effects: Vec<_> = WEARER_STAGES
        .iter()
        .flat_map(|stage| {
            stage
                .iter()
                .flat_map(|(effect, count)| std::iter::repeat_n(*effect, *count as usize))
        })
        .collect();
    for (visit, start) in WEARER_STARTS.into_iter().enumerate() {
        for (index, effect) in effects.iter().enumerate() {
            expected.push((
                start + WEARER_OFFSETS[index],
                *effect,
                WEARER_STACKS[visit][index],
            ));
        }
    }
    assert_eq!(hits.len(), expected.len(), "wearer packets, tier {tier}");
    for (hit, (time, effect, stacks)) in hits.iter().zip(&expected) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time, "tier {tier}");
        assert_eq!(hit["damage_effect_id"], *effect, "tier {tier}");
        assert_eq!(hit["critical"], false, "tier {tier}");
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            expected_stack_damage(base_damage(&WEARER_BASE_DAMAGE, effect), *stacks, tier),
            "tier {tier} {effect} at {time} with {stacks} live stacks"
        );
    }
}

/// Each actual packet attempts one stack with its independent ten-second expiry;
/// a capped attempt does not refresh an earlier expiry.
fn admission_schedule() -> (Vec<(u64, u64, u64)>, Vec<u64>) {
    let mut admitted = Vec::new();
    let mut capped = Vec::new();
    for (visit, start) in WEARER_STARTS.into_iter().enumerate() {
        for (index, offset) in WEARER_OFFSETS.into_iter().enumerate() {
            let time = start + offset;
            let live = WEARER_STACKS[visit][index];
            if live < 10 {
                admitted.push((time, u64::from(live) + 1, time + 10_000_000));
            } else {
                capped.push(time);
            }
        }
    }
    (admitted, capped)
}

fn assert_independent_expiry_and_cap(report: &Value) {
    let (admitted, capped) = admission_schedule();
    let trace = report["trace"].as_array().unwrap();
    let stack_events: Vec<&Value> = trace
        .iter()
        .filter(|event| event["event_kind"] == "oraora_basic_attack_stack")
        .collect();
    let cap_events: Vec<&Value> = trace
        .iter()
        .filter(|event| event["event_kind"] == "oraora_stack_capped")
        .collect();
    assert_eq!(stack_events.len(), admitted.len(), "admitted grants");
    for (event, (time, stacks, expiry)) in stack_events.iter().zip(&admitted) {
        assert_eq!(event["time_us"].as_u64().unwrap(), *time);
        assert_eq!(event["from_actor"], 0);
        assert_eq!(event["data"]["admitted"], true);
        assert_eq!(event["data"]["stacks"], *stacks);
        assert_eq!(event["data"]["expires_at_us"], *expiry);
    }
    let cap_total = capped.len();
    assert_eq!(cap_events.len(), cap_total);
    for (event, time) in cap_events.iter().zip(&capped) {
        assert_eq!(event["time_us"].as_u64().unwrap(), *time);
        assert_eq!(event["from_actor"], 0);
        assert_eq!(event["data"]["admitted"], false);
        assert_eq!(event["data"]["stacks"], 10);
        assert!(event["data"].get("expires_at_us").is_none());
    }
}

fn assert_switch_in_support_packets(report: &Value) {
    let (effect, times, damage) = WEARER_QTE;
    let hits = actor_hits(report, 0, "GE_Player_Nanally_QTE");
    assert_eq!(hits.len(), times.len(), "switch-in Support QTE packets");
    for (hit, time) in hits.iter().zip(times) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), time);
        assert_eq!(hit["damage_effect_id"], effect);
        // The QTE is category H: no Oraora! stack and no stack damage bonus.
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            ExactFraction::new(u128::from(damage), 1).unwrap(),
            "constant QTE packet at {time}"
        );
    }
}

fn assert_peer_chain_windows(report: &Value, tier: u32) {
    let hits = actor_hits(report, 1, "GE_Player_Mint_Melee");
    let mut expected = Vec::new();
    for chain in PEER_CHAINS {
        for (stage, delay) in [0u64, 800_000, 1_600_000, 2_400_000, 3_200_000]
            .into_iter()
            .enumerate()
        {
            for (effect, count) in PEER_STAGES[stage] {
                for _ in 0..*count {
                    expected.push((chain + delay, *effect));
                }
            }
        }
    }
    assert_eq!(hits.len(), expected.len(), "peer packets, tier {tier}");
    for (hit, (time, effect)) in hits.iter().zip(&expected) {
        assert_eq!(hit["time_us"].as_u64().unwrap(), *time);
        assert_eq!(hit["damage_effect_id"], *effect);
        assert_eq!(
            exact(&hit["non_crit_damage"]),
            ExactFraction::new(u128::from(base_damage(&PEER_BASE_DAMAGE, effect)), 1).unwrap(),
            "tier {tier} peer {effect} at {time}"
        );
    }
    let (first, second) = expected.split_at(expected.len() / 2);
    assert!(
        first
            .iter()
            .zip(second)
            .all(|((early, _), (late, _))| late - early == 19_500_000)
    );
}

/// Owner packets pay ChargeAdd x1 and every peer x3/5 (relations::DISTRIBUTION);
/// the expected sums re-read the frozen native ChargeAdd rows independently.
fn assert_charge_add_energy_oracle(report: &Value, tier: u32) {
    let wearer = charge_sum(&WEARER_ENERGY_PACKETS);
    let peer = charge_sum(&PEER_ENERGY_PACKETS);
    let distribution = ExactFraction::new(3, 5).unwrap();
    let expected_wearer = wearer.add(peer.mul(distribution).unwrap()).unwrap();
    let expected_peer = peer.add(wearer.mul(distribution).unwrap()).unwrap();
    for (actor, expected) in [(0usize, expected_wearer), (1usize, expected_peer)] {
        let state = &report["final_actor_states"][actor];
        let granted = ExactFraction::new(state["granted_energy"].as_u64().unwrap().into(), 1)
            .unwrap()
            .add(exact(&state["charge_remainder"]))
            .unwrap();
        assert_eq!(
            granted, expected,
            "actor {actor} continuing charge, tier {tier}"
        );
        assert_eq!(state["spent_energy"], 0, "no bursts spend energy");
    }
}

#[test]
fn oraora_continuing_cycles_settle_stacks_cap_expiry_and_native_energy() {
    for tier in 1..=5u32 {
        let req = cycles_request(tier);
        assert_eq!(req.actors[0].initial_hp, 15998, "native lvl80 wearer HP");
        assert_eq!(req.actors[1].initial_hp, 12120, "native lvl80 peer HP");
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_wearer_stack_windows(&report, tier);
        assert_independent_expiry_and_cap(&report);
        assert_switch_in_support_packets(&report);
        assert_peer_chain_windows(&report, tier);
        assert_charge_add_energy_oracle(&report, tier);
        assert_eq!(report["final_actor_states"][0]["hp"], 15998);
        assert_eq!(report["final_actor_states"][0]["hp_max"], 15998);
        assert_eq!(report["final_actor_states"][1]["hp"], 12120);
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
