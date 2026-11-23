use super::daffodil_actions::{make_action, request};
use super::*;
use serde_json::json;

fn with_recovery(mut action: GameActionCommand, duration: u64) -> GameActionCommand {
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut action {
        *recovery_us = duration;
    }
    action
}

#[test]
fn daffodil_insight_two_funded_bursts_preserve_marks_energy_and_break_generations() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([3, 4, 5]).unwrap();
    req.targets[0].hp_current = 10_000_000;
    req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio::one(),
        current_gauge: Ratio::one(),
        broken_duration_us: 1000,
        discord_fraction: None,
    });
    req.relative_duration_us = 152_000_000;
    req.actions.clear();
    let mut burst_commands = Vec::new();
    for cycle in 0..2 {
        let start = cycle * 76_000_000;
        burst_commands.push(req.actions.len());
        req.actions.push(with_recovery(
            make_action(
                start,
                0,
                "GA_Daffodill_UltraSkill",
                "GE_Player_Daffodill_UltraSkill1_Damage",
                120,
                20_000_000,
            ),
            5_087_548,
        ));
        // Seven whole normal chains fund the next 120 energy without resetting state.
        for normal in 0..35 {
            req.actions.push(with_recovery(
                make_action(
                    start + 6_000_000 + normal * 2_000_000,
                    0,
                    "GA_Daffodill_Melee",
                    "GE_Player_Daffodill_Melee1_Damage",
                    0,
                    0,
                ),
                2_000_000,
            ));
        }
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true, "{report}");
    for (cycle, command) in burst_commands.into_iter().enumerate() {
        let contributions: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| {
                hit["command_index"] == command && hit.get("break_contribution").is_some()
            })
            .collect();
        assert_eq!(contributions.len(), cycle + 2);
        // cap1: 3603/6 * (1+.20+.15*n), n1 then n2.
        let (numerator, denominator) = if cycle == 0 { (32427, 40) } else { (3603, 4) };
        for hit in contributions {
            assert_eq!(
                hit["damage"],
                json!(format!("{}/{}", numerator, denominator))
            );
            assert_eq!(hit["crit_sample"], serde_json::Value::Null);
        }
    }
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["spent_energy"], 240);
    assert_eq!(actor["energy"], 120);
    assert_eq!(actor["energy_conserved"], true);
    let mark = report["final_target_states"][0]["statuses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|status| {
            status["id"]
                .as_str()
                .unwrap()
                .starts_with("ntesim.daffodil.insight")
        })
        .unwrap();
    assert_eq!(mark["stacks"], 2);
    assert_eq!(
        mark["expires_at_us"],
        9_999_000_000u64 + 76_000_000 - 5_099_006
    );
}
