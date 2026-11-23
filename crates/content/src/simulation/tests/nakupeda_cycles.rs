use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/nakupeda-support-cycles.nte");

// The passive HP curve (24..40%) scales the lvl80 wearer Max HP per tier;
// start_hp%=0.5 floors the halves, so 8491.5 -> 8491.
const WEARER_MAX_HP: [u64; 5] = [16983, 17531, 18079, 18627, 19174];
const WEARER_INITIAL_HP: [u64; 5] = [8491, 8765, 9039, 9313, 9587];

fn cycles_request(tier: u32) -> GameScenarioRequest {
    let source = SOURCE.replace("refine=1", &format!("refine={tier}"));
    let result = crate::simulation::PreparedSourceScenario::new(
        &source,
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    super::assert_native_replay(&result);
    result.request
}

fn outcomes(report: &Value) -> Vec<u64> {
    report["proc_selection_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|selection| selection["outcome_index"].as_u64().unwrap())
        .collect()
}

fn rounded(base: u64, numerator: u128, denominator: u128) -> u64 {
    ExactFraction::new(base.into(), 1)
        .unwrap()
        .mul(ExactFraction::new(numerator, denominator).unwrap())
        .unwrap()
        .round_u64()
        .unwrap()
}

#[test]
fn nakupeda_continuing_cycles_settle_both_rolls_and_keep_team_vitals() {
    // Frozen buff_Nakipeda curves: heal/shield 20..40% of the recipient's Max HP.
    let support_ratio = |tier: u32| (20 + 5 * u128::from(tier - 1), 100u128);
    for tier in 1..=5u32 {
        let req = cycles_request(tier);
        let tier_index = tier as usize - 1;
        assert_eq!(
            req.actors[0].initial_hp, WEARER_INITIAL_HP[tier_index],
            "native lvl80 wearer 50% HP, tier {tier}"
        );
        assert_eq!(req.actors[0].initial_hp_max, WEARER_MAX_HP[tier_index]);
        assert_eq!(
            req.actors[1].initial_hp, 1212,
            "peer 10% HP stays the lowest ratio"
        );
        assert_eq!(req.actors[1].initial_hp_max, 12120);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        // Seed 42 rolls the lowest-HP heal first and the wearer shield second;
        // one continuing stream pays draw 0 before and draw 1 after the switch.
        let selections = report["proc_selection_receipts"].as_array().unwrap();
        assert_eq!(outcomes(&report), vec![0, 1], "tier {tier}");
        for (index, selection) in selections.iter().enumerate() {
            assert_eq!(selection["time_us"], [0, 62_698_647][index]);
            assert_eq!(selection["stream"], "weapon:nakupeda:0");
            assert_eq!(selection["sample"]["draws"][0]["index"], index as u64);
            assert_eq!(
                selection["cooldown_until_us"],
                [30_000_000, 92_698_647][index]
            );
        }
        // Commitment is at 0 and 30s + nine native
        // 3.633183s chains = 62.698647s; primary contact is +3.843514s.
        let bursts: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Haniel_Ultraskill1_Damage")
            .collect();
        assert_eq!(bursts.len(), 2, "tier {tier}");
        assert_eq!(bursts[0]["time_us"], 3_843_514);
        assert_eq!(bursts[1]["time_us"], 66_542_161);
        // Outcome 0 heals the lowest-ratio peer by the peer's own Max HP ratio.
        let (numerator, denominator) = support_ratio(tier);
        assert_eq!(
            report["final_actor_states"][1]["hp"],
            1212 + rounded(12120, numerator, denominator)
        );
        // The wearer never gains or loses HP through outcomes 0 and 1.
        assert_eq!(
            report["final_actor_states"][0]["hp"],
            WEARER_INITIAL_HP[tier_index]
        );
        // Outcome 1 shields the wearer for 15s; it expires before the horizon.
        let shield_amount = rounded(WEARER_MAX_HP[tier_index], numerator, denominator);
        let trace = report["trace"].as_array().unwrap();
        let granted = trace
            .iter()
            .find(|event| event["event_kind"] == "nakupeda_shield")
            .unwrap();
        assert_eq!(granted["time_us"], 62_698_647);
        assert_eq!(granted["data"]["amount"], shield_amount);
        assert_eq!(granted["data"]["expires_at_us"], 77_698_647);
        assert!(
            trace
                .iter()
                .any(|event| event["event_kind"] == "actor_shield_expired"
                    && support::world_time_at(&report, event["time_us"].as_u64().unwrap())
                        == support::world_time_at(&report, 62_698_647) + 15_000_000)
        );
        assert!(
            report["final_actor_states"][0]
                .get("active_shields")
                .is_none()
        );
        // Both Ultimates spend full energy; refills never reset the pool.
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
        assert_eq!(report["final_actor_states"][1]["spent_energy"], 0);
        assert_eq!(
            trace
                .iter()
                .filter(|event| event["energy_change"]["source"] == "ability_cost"
                    && event["energy_change"]["spent"] == 120)
                .count(),
            2
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

#[test]
fn nakupeda_cycles_roll_all_three_outcomes_with_exact_team_vital_oracles() {
    let mut seen = BTreeSet::new();
    for seed in 0..12u64 {
        let mut req = cycles_request(1);
        req.seed = seed;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let rolls = outcomes(&report);
        assert_eq!(rolls.len(), 2, "seed {seed}");
        assert_eq!(report["proc_selection_receipts"][1]["time_us"], 62_698_647);
        seen.extend(rolls.iter().copied());
        // T1: outcome 0 heals the peer 12120*1/5, outcome 2 heals everyone by
        // their own Max HP (12120*1/10 and 16983*1/10), outcome 1 keeps HP.
        let peer_heal = rolls
            .iter()
            .filter(|&&outcome| outcome == 0 || outcome == 2)
            .map(|&outcome| rounded(12120, if outcome == 0 { 20 } else { 10 }, 100))
            .sum::<u64>();
        let wearer_heal = rolls
            .iter()
            .filter(|&&outcome| outcome == 2)
            .map(|_| rounded(16983, 10, 100))
            .sum::<u64>();
        assert_eq!(
            report["final_actor_states"][1]["hp"],
            1212 + peer_heal,
            "seed {seed} rolls {rolls:?}"
        );
        assert_eq!(
            report["final_actor_states"][0]["hp"],
            8491 + wearer_heal,
            "seed {seed} rolls {rolls:?}"
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 240);
    }
    assert_eq!(seen, BTreeSet::from([0, 1, 2]));
}
