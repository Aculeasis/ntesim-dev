use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/silent-garden-cycles.nte");

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let mut req = live_source_request(&source);
    for actor in &mut req.actors {
        let stats = &mut actor.static_stats;
        stats.base_atk = 1000;
        stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
    }
    req
}

fn fraction(value: &Value) -> ExactFraction {
    let ratio: Ratio = serde_json::from_value(value.clone()).unwrap();
    ExactFraction::from_ratio(ratio).unwrap()
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

fn charge_total(packets: &[(&str, u32)]) -> ExactFraction {
    let mut total = ExactFraction::new(0, 1).unwrap();
    for (effect_id, count) in packets {
        for _ in 0..*count {
            total = total.add(packet_charge(effect_id)).unwrap();
        }
    }
    total
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

const CHAIN: [(&str, u64, u64); 5] = [
    ("GE_Player_Cang_Melee1_Damage", 475, 184_377),
    ("GE_Player_Cang_Melee2_Damage", 423, 1_123_478),
    ("GE_Player_Cang_Melee3_Damage", 1190, 1_930_256),
    ("GE_Player_Cang_Melee4_Damage", 251, 3_116_386),
    ("GE_Player_Cang_Melee5_Damage", 863, 5_053_095),
];
const MINT_CYCLE: [&str; 8] = [
    "GE_Player_Mint_Melee1_Damage_New",
    "GE_Player_Mint_Melee2_Damage_New",
    "GE_Player_Mint_Melee3_Damage_New",
    "GE_Player_Mint_Melee4_Damage_New",
    "GE_Player_Mint_Melee5_Damage_New",
    "GE_Player_Mint_Melee6_Damage_New",
    "GE_Player_Mint_Melee7_Damage_New",
    "GE_Player_Mint_Melee8_Damage_New",
];
const BAICANG_NATIVE_PACKETS: [(&str, u32); 5] = [
    ("GE_Player_Cang_Melee1_Damage", 3),
    ("GE_Player_Cang_Melee2_Damage", 2),
    ("GE_Player_Cang_Melee3_Damage", 2),
    ("GE_Player_Cang_Melee4_Damage", 14),
    ("GE_Player_Cang_Melee5_Damage", 2),
];

fn assert_continuing_rng_stream(report: &Value) {
    let selections = report["proc_selection_receipts"].as_array().unwrap();
    assert_eq!(selections.len(), 2);
    for (index, selection) in selections.iter().enumerate() {
        assert_eq!(selection["time_us"], [0, 25_000_000][index]);
        assert_eq!(selection["stream"], "weapon:jingmotingyuan:0");
        assert_eq!(selection["sample"]["draws"][0]["index"], index as u64);
        assert_eq!(
            selection["cooldown_until_us"],
            [25_000_000, 50_000_000][index]
        );
    }
}

fn assert_arc_trains(report: &Value, tier: u32) {
    // Exact refinement coefficients 24..40% / 18..30% at ATK1000; Nagging's
    // sixth packet is doubled; every train packet shares the admission time.
    let selections = report["proc_selection_receipts"].as_array().unwrap();
    let arc_hits = actor_hits(report, 0, "GE_Fork_serenity_Skill");
    let expected_count: usize = selections
        .iter()
        .map(|selection| usize::from(selection["outcome_index"] != 0) + 5)
        .sum();
    assert_eq!(arc_hits.len(), expected_count, "tier {tier}");
    for hit in &arc_hits {
        let time = hit["time_us"].as_u64().unwrap();
        let arc = usize::from(time >= 25_000_000);
        let expected = match hit["damage_effect_id"].as_str().unwrap() {
            crate::weapons::jingmotingyuan::arc::HARSH => 200 + 40 * u64::from(tier),
            crate::weapons::jingmotingyuan::arc::NAGGING => 150 + 30 * u64::from(tier),
            _ => 2 * (150 + 30 * u64::from(tier)),
        };
        assert_eq!(
            super::support::hit_metadata(report, hit)["action"]["ability_id"],
            "weapon:fork_jingmotingyuan:arc"
        );
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::to_value(
                ExactFraction::new(expected.into(), 1)
                    .unwrap()
                    .mul(ExactFraction::new(6, 5).unwrap())
                    .unwrap()
            )
            .unwrap()
        );
        assert_eq!(hit["time_us"], [0, 25_000_000][arc]);
    }
}

fn assert_continuing_chain_cycles(report: &Value, tier: u32) {
    // The native five-stage chain repeats identically in both cycles; the
    // The31s step follows the completed chain wrap; re-entry adds no input.
    let cang = actor_hits(report, 0, "GE_Player_Cang_Melee");
    assert_eq!(cang.len(), 23, "two chains plus the31s step");
    for cycle in 0..2u64 {
        let offset = cycle * 25_000_000;
        let mut hits = cang.iter().filter(|hit| {
            let time = hit["time_us"].as_u64().unwrap();
            if cycle == 0 {
                time < 20_000_000
            } else {
                (25_000_000..31_000_000).contains(&time)
            }
        });
        for (fx, damage, delay) in CHAIN {
            let repeat = if fx.ends_with("Melee4_Damage") { 7 } else { 1 };
            for slot in 0..repeat {
                let hit = hits.next().expect("chain packet present");
                assert_eq!(hit["damage_effect_id"], fx, "tier {tier} cycle {cycle}");
                assert_eq!(hit["time_us"], delay + offset + slot * 115_738);
                assert_eq!(
                    hit["non_crit_damage"],
                    serde_json::to_value(
                        ExactFraction::new(damage.into(), 1)
                            .unwrap()
                            .mul(ExactFraction::new(6, 5).unwrap())
                            .unwrap()
                    )
                    .unwrap()
                );
            }
        }
        assert!(hits.next().is_none());
    }
    assert_eq!(cang[22]["time_us"], 31_184_377);
    assert_eq!(cang[22]["damage_effect_id"], "GE_Player_Cang_Melee1_Damage");
    // The off-field peer keeps its own identical cycles in both windows.
    let mint = actor_hits(report, 1, "GE_Player_Mint_Melee");
    assert_eq!(mint.len(), 16, "tier {tier}");
    for (first, second) in mint[..8].iter().zip(&mint[8..]) {
        assert_eq!(first["damage_effect_id"], second["damage_effect_id"]);
        assert_eq!(
            second["time_us"].as_u64().unwrap() - first["time_us"].as_u64().unwrap(),
            44_490_692
        );
        assert_eq!(first["non_crit_damage"], second["non_crit_damage"]);
    }
}

fn assert_charge_add_energy_oracle(report: &Value, tier: u32) {
    // Owner packets pay ChargeAdd x1 and every element-pair peer x3/5
    // (relations::DISTRIBUTION); weapon packets never join hit energy.
    let baicang_native = charge_total(&BAICANG_NATIVE_PACKETS);
    let mint_native = charge_total(&MINT_CYCLE.iter().map(|fx| (*fx, 2)).collect::<Vec<_>>());
    let peer = ExactFraction::new(3, 5).unwrap();
    let expected_baicang = baicang_native.add(mint_native.mul(peer).unwrap()).unwrap();
    let expected_mint = mint_native.add(baicang_native.mul(peer).unwrap()).unwrap();
    for (actor, expected) in [(0usize, expected_baicang), (1, expected_mint)] {
        let state = &report["final_actor_states"][actor];
        let granted = ExactFraction::new(state["granted_energy"].as_u64().unwrap().into(), 1)
            .unwrap()
            .add(fraction(&state["charge_remainder"]))
            .unwrap();
        assert_eq!(
            granted, expected,
            "actor {actor} continuing charge, tier {tier}"
        );
        assert_eq!(state["spent_energy"], 0);
    }
}

#[test]
fn silent_garden_continuing_cycles_settle_arc_payments_packets_and_native_energy() {
    for tier in 1..=5u32 {
        let req = cycles_request(tier);
        assert_eq!(req.actors[0].initial_hp, 16604, "native lvl80 wearer HP");
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        assert_continuing_rng_stream(&report);
        assert_arc_trains(&report, tier);
        // Arc5% and Word3% current-HP costs, each nearest/ties-up, without resets:
        // 16604 ->15774 ->15301 ->14536 ->14100 ->13677.
        assert_eq!(report["final_actor_states"][0]["hp"], 13677);
        assert_eq!(report["final_actor_states"][0]["hp_max"], 16604);
        // Arc and actual Word costs both notify the equipped weapon once.
        let stacks: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "jingmotingyuan_crit_stack")
            .collect();
        assert_eq!(stacks.len(), 5, "tier {tier}");
        for (row, (time, cost)) in stacks.iter().zip([
            (0, 830),
            (1_930_256, 473),
            (25_000_000, 765),
            (25_184_377, 436),
            (28_116_386, 423),
        ]) {
            assert_eq!(row["time_us"], time);
            assert_eq!(row["data"]["consumed"], cost);
            assert_eq!(row["data"]["expires_at_us"], time + 5_000_000);
        }
        assert_continuing_chain_cycles(&report, tier);
        assert_charge_add_energy_oracle(&report, tier);
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

#[test]
fn silent_garden_cycles_sample_both_trains_across_seeds() {
    let mut seen = BTreeSet::new();
    for seed in 0..8u64 {
        let mut req = cycles_request(1);
        req.seed = seed;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let selections = report["proc_selection_receipts"].as_array().unwrap();
        for (arc, time) in selections.iter().zip([0, 25_000_000]) {
            assert_eq!(arc["time_us"], time);
            seen.insert(arc["outcome_index"].as_u64().unwrap());
            let count = if arc["outcome_index"] == 0 { 5 } else { 6 };
            let packets = report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| {
                    hit["time_us"] == time
                        && hit["damage_effect_id"]
                            .as_str()
                            .unwrap()
                            .starts_with("GE_Fork_serenity_Skill")
                })
                .count();
            assert_eq!(packets, count, "seed {seed}");
        }
        assert_eq!(report["final_actor_states"][0]["hp"], 13677);
    }
    assert_eq!(seen, BTreeSet::from([0, 1]));
}

#[test]
fn silent_garden_cycles_cost_stacks_buff_only_live_packets_in_each_window() {
    let mut req = cycles_request(1);
    // Native lvl80 Baicang carries a static +50% crit damage zone entry.
    assert_eq!(
        req.actors[0].static_stats.crit_damage_bonus,
        Ratio {
            numerator: 1,
            denominator: 2
        }
    );
    let mut policy = research_request().stochastic_crit.unwrap();
    policy.actor_sequences = BTreeMap::from([("character:1023".into(), 54)]);
    policy.eligible_effects =
        BTreeMap::from([("character:1023".into(), BTreeSet::from(["*".into()]))]);
    req.stochastic_crit = Some(policy);
    req.actors[0].static_stats.crit_rate = Ratio::one();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    for hit in report["hit_receipts"].as_array().unwrap() {
        let time = hit["time_us"].as_u64().unwrap();
        let stacks = [
            (0, 1),
            (1_930_256, 2),
            (25_000_000, 1),
            (25_184_377, 2),
            (28_116_386, 3),
        ]
        .into_iter()
        .rfind(|(at, _)| *at < time || (*at == time && matches!(*at, 0 | 25_000_000)))
        .filter(|(at, _)| time < at + 5_000_000)
        .map_or(0, |(_, count)| count);
        if hit["actor_index"] == 0 {
            assert_eq!(hit["critical"], true);
            // Crit zone sums static1/2 and12% per live actual-cost stack.
            let expected = fraction(&hit["non_crit_damage"])
                .mul(ExactFraction::new(75 + 6 * stacks, 50).unwrap())
                .unwrap();
            assert_eq!(fraction(&hit["damage"]), expected, "time {time}");
        } else {
            assert_eq!(hit["critical"], false);
        }
    }
    let arcs = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .unwrap()
                .starts_with("GE_Fork_serenity_Skill")
        })
        .count();
    assert!(arcs > 0);
}
