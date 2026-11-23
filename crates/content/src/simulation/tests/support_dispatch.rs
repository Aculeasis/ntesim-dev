use super::*;
use serde_json::{Value, json};

fn switch(timestamp_us: u64, target_actor_index: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index,
        cooldown_us: 0,
    }
}

fn count(report: &Value, kind: &str) -> usize {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .count()
}

fn request(owners: &[&str]) -> GameScenarioRequest {
    let mut request = research_request();
    let template = request.actors[0].clone();
    request.actors = owners
        .iter()
        .enumerate()
        .map(|(slot, &id)| {
            let mut actor = template.clone();
            actor.slot = slot;
            actor.character_id = id.into();
            actor
        })
        .collect();
    request.stochastic_crit = None;
    request.actions.clear();
    request.action_metadata.clear();
    request
}

#[test]
fn repeated_qualified_support_uses_two_paid_reactions_without_its_own_cooldown() {
    for (owner, outgoing, ability, first_delay) in [
        ("character:1003", "character:1010", "GA_Sagiri_QTE", 100_000),
        (
            "character:1010",
            "character:1003",
            "GA_Nanally_QTE",
            666_410,
        ),
        (
            "character:1046",
            "character:1010",
            "GA_Female046_QTE",
            100_000,
        ),
    ] {
        // Two different outgoing actors with a full pool can fund two
        // re-entries without recharging or resetting either account.
        let first = if owner == "character:1010" {
            "character:1046"
        } else {
            "character:1019"
        };
        let mut req = request(&[first, owner, outgoing]);
        req.actors[0].initial_cycle = 100;
        req.actors[2].initial_cycle = 100;
        let mut extra_target = req.targets[0].clone();
        extra_target.target_index = 1;
        req.targets.push(extra_target);
        req.actions = vec![
            switch(0, 1),
            switch(10, 1),
            switch(2_000_000, 2),
            switch(2_010_000, 1),
        ];
        req.aim_targets = [(0, 1), (3, 1)].into();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{owner}: {}",
            report["halt_diagnostic"]
        );
        assert_eq!(count(&report, "support_dispatch"), 2, "{owner}");
        assert_eq!(count(&report, "switch_actor_noop"), 1, "{owner}");
        assert_eq!(report["final_actor_states"][0]["cycle"]["spent"], 100);
        assert_eq!(report["final_actor_states"][2]["cycle"]["spent"], 100);
        let hits = report["hit_receipts"].as_array().unwrap();
        for time in [first_delay, 2_010_000 + first_delay] {
            assert!(
                hits.iter().any(|hit| {
                    hit["actor_index"] == 1
                        && hit["ability_id"] == ability
                        && hit["time_us"] == time
                        && hit["switch_origin"]
                            == json!({"kind":"support","incoming_actor_index":1})
                }),
                "{owner} has no Support hit at {time}"
            );
        }
    }
}

#[test]
fn ordinary_entry_has_no_support_even_for_the_previously_unconditional_owners() {
    for owner in ["character:1003", "character:1010", "character:1046"] {
        let mut req = request(&["character:1019", owner]);
        req.actions = vec![switch(0, 1)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true);
        assert_eq!(count(&report, "support_dispatch"), 0, "{owner}");
        assert!(
            report["hit_receipts"].as_array().unwrap().is_empty(),
            "{owner}"
        );
    }
}

#[test]
fn ordinary_switch_does_not_start_haniel_hathor_or_edgar_attacks() {
    for owner in ["haniel", "hathor", "edgar"] {
        let source = format!(
            "options duration=2 crit=fixed;
             mint char lvl=80/80 awakening=[] talent=1,1,1 qte=1;
             {owner} char lvl=80/80 awakening=[] talent=1,1,1 qte=1;
             target hp=100000 def=0 resist=0;
             active mint; {owner} swap; mint swap;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &ExecutionLimits::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            result.report.execution_completed,
            "{owner}: {:?}",
            result.diagnostic
        );
        assert!(result.report.hit_receipts.is_empty(), "{owner}");
        assert!(result.report.action_metadata_ledger.is_empty(), "{owner}");
        assert_eq!(result.report.final_buff_state.active_actor_index, 0);
    }
}

#[test]
fn conflicting_reached_support_model_cannot_produce_a_successful_run() {
    let mut req = request(&["character:1019", "character:1003"]);
    req.actors[0].initial_cycle = 100;
    req.actions = vec![switch(0, 1)];
    let id = crate::characters::sakiri::support::HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), "conflicting description".into());
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains(id.as_str()), "{error}");
}

#[test]
fn implicit_source_switch_admits_qualified_support_before_the_requested_attack() {
    let mut req = crate::dsl::compiler::compile_scenario(
        "options duration=2;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=0 resist=0;
         active mint; sakiri attack;",
        Default::default(),
    )
    .unwrap()
    .request;
    req.actors[0].initial_cycle = 100;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(count(&report, "support_dispatch"), 1);
    assert_eq!(count(&report, "switch_actor"), 1);
    assert_eq!(
        report["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":600_000})
    );
    assert_eq!(
        report["action_metadata_ledger"][0]["switch_origin"],
        json!({"kind":"support","incoming_actor_index":1})
    );
}

#[test]
fn dispatched_support_feeds_team_admission_observers_like_a_manual_cast() {
    let mut req = request(&["character:1019", "character:1054", "character:1003"]);
    req.actors[0].initial_cycle = 100;
    req.actions = vec![switch(0, 2)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(count(&report, "support_dispatch"), 1);
    assert_eq!(count(&report, "daffodil_resonance_granted"), 1);
}

#[test]
fn lacrimosa_a6_allows_a_free_reaction_and_preserves_standard_paid_support() {
    for initial_cycle in [0, 100] {
        let source = format!(
            "options duration=12 crit=fixed;
        lacrimosa char lvl=80/80 awakening=[6] talent=1,1,1 qte=1;
        sakiri char lvl=80/80 awakening=[] talent=1,1,1 qte=1 start_cycle={initial_cycle};
        aurelia char lvl=80/80 awakening=[] talent=1,1,1 qte=1 start_cycle=100;
        target hp=100000000 def=0 resist=0;
        active lacrimosa;
        lacrimosa skill; wait_until(0.9);
        sakiri swap; wait(0.1); lacrimosa swap;
        if .sakiri.resources.cycle!={initial_cycle} {{let error=1/0;}}
        wait(0.7);
        sakiri swap; wait(0.1); lacrimosa swap; wait(0.7);
        aurelia swap; wait(0.1); lacrimosa swap;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &ExecutionLimits::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(result.report.execution_completed, "{:?}", result.diagnostic);
        let report = serde_json::to_value(&result.report).unwrap();
        let paid = u64::from(initial_cycle == 100);
        assert_eq!(count(&report, "support_dispatch") as u64, 2 + paid);
        assert_eq!(report["reaction_counts"]["Scorch"], 1 + paid);
        assert_eq!(report["reaction_counts"]["Nova"], 1);
        assert_eq!(report["final_actor_states"][2]["cycle"]["spent"], 100);
        let spent = result.report.final_actor_states[1]
            .cycle
            .as_ref()
            .map_or(0, |cycle| cycle.spent);
        assert_eq!(spent, initial_cycle);
    }
}

#[test]
fn lacrimosa_a6_entry_window_expires_at_five_seconds() {
    for (when, cycle, expected_spend, expected_dispatches) in
        [("4.999999", 100, 0, 1), ("5", 100, 100, 1), ("5", 0, 0, 0)]
    {
        let source = format!(
            "options duration=8;
            lacrimosa char lvl=80/80 awakening=[6] talent=1,1,1 qte=1;
            sakiri char lvl=80/80 awakening=[] talent=1,1,1 qte=1 start_cycle={cycle};
            target hp=100000000 def=0 resist=0; active lacrimosa;
            lacrimosa skill; wait_until(0.9); sakiri swap;
            wait_until({when}); lacrimosa swap;"
        );
        let result = crate::simulation::PreparedSourceScenario::new(
            &source,
            Default::default(),
            &ExecutionLimits::default(),
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(result.report.execution_completed, "{:?}", result.diagnostic);
        let report = serde_json::to_value(&result.report).unwrap();
        let spent = result.report.final_actor_states[1]
            .cycle
            .as_ref()
            .map_or(0, |cycle| cycle.spent);
        assert_eq!(spent, expected_spend, "return at {when}");
        assert_eq!(
            count(&report, "support_dispatch"),
            expected_dispatches,
            "return at {when}"
        );
    }
}

#[test]
fn lacrimosa_a6_free_nova_can_trigger_discord_against_live_scorch() {
    let source = "options duration=8;
        lacrimosa char lvl=80/80 awakening=[6] talent=1,1,1 qte=1;
        sakiri char lvl=80/80 awakening=[] talent=1,1,1 qte=1 start_cycle=100;
        aurelia char lvl=80/80 awakening=[] talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active sakiri;
        lacrimosa swap; lacrimosa skill; aurelia swap; wait(0.1); lacrimosa swap;";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(result.report.execution_completed, "{:?}", result.diagnostic);
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(count(&report, "support_dispatch"), 2);
    for reaction in ["Scorch", "Nova", "Discord"] {
        assert_eq!(report["reaction_counts"][reaction], 1, "{reaction}");
    }
    assert_eq!(report["final_actor_states"][1]["cycle"]["spent"], 100);
    assert_eq!(
        result.report.final_actor_states[2]
            .cycle
            .as_ref()
            .map_or(0, |cycle| cycle.spent),
        0
    );
}

#[test]
fn iroi_support_damage_cap_and_heal_share_qte_level_but_use_separate_events() {
    let source = format!(
        "options duration=2 crit=fixed;
        zero char lvl=80/80 awakening=[] talent=1,1,1 qte=1 start_cycle=100 start_hp%=0.5;
        iroi char lvl=80/80 awakening=[] talent=1,1,1 qte=8 start_hp%=0.5;
        {} active zero; iroi swap[target=t12];",
        "target hp=100000000 def=0 resist=0;".repeat(13)
    );
    let result = crate::simulation::PreparedSourceScenario::new(
        &source,
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(result.report.execution_completed, "{:?}", result.diagnostic);
    let report = serde_json::to_value(&result.report).unwrap();
    let hits = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Oneiroi_QTE")
        .collect::<Vec<_>>();
    assert_eq!(hits.len(), 12);
    assert!(hits.iter().any(|hit| hit["target_index"] == 12));
    assert!(hits.iter().all(|hit| hit["time_us"] == 784_391));
    let heals = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "iroi_support_heal")
        .collect::<Vec<_>>();
    assert_eq!(heals.len(), 2);
    for (recipient, heal) in heals.into_iter().enumerate() {
        assert_eq!(heal["to_actor"], recipient);
        // The heal is 284 montage microseconds after contact, during the 30% rate window.
        assert_eq!(heal["time_us"], 785_338);
        assert_eq!(heal["data"]["atk_ratio"], "1029/1000");
    }
}
