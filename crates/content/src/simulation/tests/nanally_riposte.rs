use super::*;
use serde_json::{Value, json};

const SOURCE: &str = include_str!("../../../../../examples/nanally-riposte-cycles.nte");
const PACKET: &str = "GE_Player_Nanally_PerfectEvadeAttack_Damage";
const CAT: &str = "GE_Player_Nanally_Cat_PerfectEvadeAttack_Damage";

#[test]
fn nanally_riposte_opportunities_do_not_stack_and_source_death_cancels_pending_damage() {
    let prefix = SOURCE.split("# Two").next().unwrap();
    let source = format!(
        "{prefix} nanally dash[critical=true]; wait(0.4); nanally dash[critical=true]; nanally attack; wait(0.34); nanally attack;"
    );
    let report = run_game_scenario(&request(&source), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(packets(&report, PACKET).len(), 2);
    assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
    let source = SOURCE.replace(
        "active nanally;",
        "incoming nanally at=1.6 damage=100000000; active nanally;",
    );
    let mut req = request(&source);
    req.actions.truncate(3);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert!(packets(&report, PACKET).is_empty());
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
}

fn request(source: &str) -> GameScenarioRequest {
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req
}

fn packets<'a>(report: &'a Value, effect: &str) -> Vec<&'a Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == effect)
        .collect()
}

#[test]
fn nanally_paid_riposte_cycles_keep_normal_talent_aoe_charge_and_one_use_authorization() {
    for (normal, awakening, numerator, denominator) in
        [(1, 0, 2075, 1), (8, 0, 14225, 4), (8, 3, 15365, 4)]
    {
        let source = SOURCE
            .replace(
                "talent=1,1,1 qte=1 start_energy=0",
                &format!("talent={normal},1,1 qte=1 start_energy=0"),
            )
            .replacen("awakening=0", &format!("awakening={awakening}"), 1);
        let report = run_game_scenario(&request(&source), &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let hits = packets(&report, PACKET);
        assert_eq!(hits.len(), 4);
        for hit in hits {
            assert!([1_767_562, 4_767_562].contains(&hit["time_us"].as_u64().unwrap()));
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator))
            );
            let meta = super::support::hit_metadata(&report, hit);
            assert_eq!(meta["action"]["input_role"], "critical_riposte");
            assert_eq!(meta["action"]["primary_slot"], Value::Null);
            assert_eq!(
                meta["packet"]["native_ga_name"],
                "GA_Nanally_ExtremEvadeAtk"
            );
            assert_eq!(meta["binding"]["kind"], "owner_child");
        }
        assert_eq!(packets(&report, "GE_Player_Nanally_Melee1_Damage").len(), 4);
        assert_eq!(report["combat_stamina"]["total_spent"], json!("30/1"));
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "nanally_riposte_energy_grant")
            .collect();
        assert_eq!(grants.len(), 2); // Two packet groups, each fanned out to owner plus ally.
        for grant in grants {
            let fanout = &grant["energy_fanout"];
            assert_eq!(fanout["base"], json!("21/5"));
            assert_eq!(fanout["recipients"].as_array().unwrap().len(), 2);
        }
    }
}

#[test]
fn nanally_plain_dodge_and_other_admitted_inputs_do_not_leave_a_free_riposte() {
    let plain = run_game_scenario(
        &request(
            &SOURCE
                .replace("[critical=true]", "")
                .replace("wait(0.34)", "wait(0.7)")
                .replace("wait_until(4)", "wait_until(4.2)"),
        ),
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(
        plain["execution_completed"], true,
        "{}",
        plain["halt_diagnostic"]
    );
    assert!(packets(&plain, PACKET).is_empty());
    let prefix = SOURCE.split("# Two").next().unwrap();
    for action in [
        "nanally special;",
        "mint swap; wait(2); nanally swap; wait(2);",
    ] {
        let source =
            format!("{prefix} nanally dash[critical=true]; {action} wait(1); nanally attack;");
        let report = run_game_scenario(&request(&source), &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        assert!(packets(&report, PACKET).is_empty());
    }
    let mut same = request(SOURCE);
    same.actions.insert(
        1,
        GameActionCommand::SwitchActor {
            timestamp_us: 500_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    );
    let report = run_game_scenario(&same, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(packets(&report, PACKET).len(), 4);
}

#[test]
fn nanally_riposte_requires_paid_dodge_and_preserves_real_recovery() {
    let missing = SOURCE.replace(
        " stamina_capacity=60 initial_stamina=60 stamina_use_efficiency=1 stamina_recovery_rate=0",
        "",
    );
    assert!(crate::dsl::compiler::compile_scenario(&missing, Default::default()).is_err());
    let shortage = run_game_scenario(
        &request(&SOURCE.replace("initial_stamina=60", "initial_stamina=14")),
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(shortage["rejected_action"]["reason"]["kind"], "stamina");
    assert!(packets(&shortage, PACKET).is_empty());
    assert!(
        shortage["combat_stamina"]["spends"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let early = run_game_scenario(
        &request(&SOURCE.replace("wait(0.34);", "")),
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(early["rejected_action"]["reason"]["kind"], "recovery");
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 2_638_593);
    assert_eq!(packets(&early, PACKET).len(), 2);
}

#[test]
fn nanally_riposte_cat_reads_captured_burst_talent_and_expires_before_delayed_delivery() {
    let source = include_str!("../../../../../examples/nanally-riposte-underboss.nte");
    let prefix = source.split("# The first").next().unwrap();
    let report = run_game_scenario(&request(source), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let cat = packets(&report, CAT);
    assert_eq!(cat.len(), 2);
    for hit in &cat {
        assert_eq!(hit["non_crit_damage"], json!("7345/4"));
    }
    assert_eq!(
        cat.iter()
            .map(|hit| hit["target_index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        super::support::hit_metadata(&report, cat[0])["packet"]["native_ga_name"],
        "GA_Nanally_UltraSkill"
    );
    assert_eq!(packets(&report, PACKET).len(), 4);
    let expiry = format!(
        "{prefix} nanally burst; wait_until(9.2); nanally dash[critical=true]; nanally attack; wait(0.34);"
    );
    let report = run_game_scenario(&request(&expiry), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(packets(&report, PACKET).len(), 2);
    assert!(packets(&report, CAT).is_empty());
}

#[test]
fn nanally_riposte_cat_preserves_a4_scope_and_source_death_cancellation() {
    let source = include_str!("../../../../../examples/nanally-riposte-underboss.nte");
    let a4 = source.replacen("awakening=0", "awakening=[4]", 1);
    let report = run_game_scenario(&request(&a4), &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    for hit in packets(&report, CAT) {
        assert_eq!(hit["non_crit_damage"], json!("13221/4"));
    }
    for hit in packets(&report, PACKET) {
        assert_eq!(hit["non_crit_damage"], json!("2075/1"));
    }
    let source = source.replace(
        "active nanally;",
        "incoming nanally at=0.8 damage=100000000; active nanally;",
    );
    let mut req = request(&source);
    req.actions.truncate(4); // Burst, explicit movement wait, paid dash and admitted Riposte.
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(packets(&report, PACKET).is_empty());
    assert!(packets(&report, CAT).is_empty());
    assert_eq!(report["combat_stamina"]["total_spent"], json!("15/1"));
}
