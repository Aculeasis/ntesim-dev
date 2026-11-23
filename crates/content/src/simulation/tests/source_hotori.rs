use super::*;

fn run(rotation: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(
        &format!(
            "options duration=80 iteration=1 crit=fixed;
            hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active hotori; {rotation}"
        ),
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap()
}

fn times(result: &SourceScenarioReport, kind: &str) -> Vec<u64> {
    result
        .report
        .trace
        .iter()
        .filter(|event| event.event_kind == kind)
        .map(|event| event.time_us)
        .collect()
}

fn completed(result: &SourceScenarioReport) {
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_native_replay(result);
}

#[test]
fn source_hotori_guarded_two_cycles_keep_katana_finisher_and_timepiece_carry() {
    let result = run("repeat 2 {
        let start=.time;
        if .hotori.resources.timepiece!=120 {let assertion_failure=1/0;}
        hotori burst;
        while .hotori.time_stop.active {hotori attack[window=attack];}
        if .hotori.normal!=1 {let assertion_failure=1/0;}
        wait(40-(.time-start));
    }");
    completed(&result);
    assert_eq!(result.report.hits_settled, 74);
    assert_eq!(
        times(&result, "hotori_time_stop_enter"),
        [2_183_333, 54_018_126]
    );
    assert_eq!(
        times(&result, "hotori_time_stop_finisher"),
        [11_834_793, 63_669_586]
    );
    assert!(times(&result, "hotori_time_stop_expired").is_empty());
    let timepiece = &serde_json::to_value(&result.report.final_actor_states[0]).unwrap()["fractional_resources"]
        ["hotori.timepiece"];
    assert_eq!(timepiece["spent"], serde_json::json!("240/1"));
    assert_eq!(timepiece["conserved"], true);
    assert_eq!(result.report.absolute_elapsed_us, 103_669_586);
}

#[test]
fn source_hotori_fixed_repeat_spills_into_ordinary_mode_after_finisher() {
    let result = run("hotori burst; repeat 12 {hotori attack[window=attack];} mint swap;");
    completed(&result);
    assert_eq!(times(&result, "hotori_time_stop_finisher"), [11_834_793]);
    let ordinary: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Jin_Melee1_Damage")
        .collect();
    assert_eq!(ordinary.len(), 1);
    assert!(ordinary[0].time_us > 11_834_793);
    assert_eq!(result.report.final_buff_state.active_actor_index, 1);
    let full_recovery = run("hotori burst; hotori attack:12; mint swap;");
    completed(&full_recovery);
    assert_eq!(
        times(&full_recovery, "hotori_time_stop_expired"),
        [14_183_333]
    );
    assert!(
        full_recovery
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Jin_Melee1_Damage")
    );
}

#[test]
fn source_hotori_switch_failure_is_immediate_and_an_explicit_wait_reaches_timeout() {
    for next in ["mint swap;", "mint attack;"] {
        let result = run(&format!("hotori burst; {next}"));
        assert!(!result.report.success && !result.report.execution_completed);
        assert_eq!(result.occurrences[1].started_us, 4_604_238);
        assert!(result.occurrences[1].completed_us.is_none());
        assert!(result.report.rejected_action.is_some());
        assert_eq!(
            result.diagnostic.as_ref().unwrap().code,
            crate::dsl::validation::DiagnosticCode::Admission
        );
        let context = result.failure_context.as_ref().unwrap();
        assert_eq!(
            (context.occurrence_index, context.command_index),
            (1, Some(1))
        );
        assert_eq!(
            result.diagnostic.as_ref().unwrap().loc,
            result.occurrences[1].loc
        );
    }
    let result = run("incoming hotori at=3 damage=100;
        incoming hotori at=16 damage=100;
        hotori burst; wait_until(14.183333);
        if .hotori.time_stop.active || .hotori.resources.timepiece!=0 {let assertion_failure=1/0;}
        mint swap; wait_until(28);");
    completed(&result);
    assert_eq!(times(&result, "hotori_time_stop_expired"), [14_183_333]);
    assert_eq!(
        result
            .report
            .incoming_damage_receipts
            .iter()
            .map(|event| event.time_us)
            .collect::<Vec<_>>(),
        [17_183_333, 30_183_333]
    );
}

#[test]
fn source_hotori_explicit_waits_consume_real_mode_budget_without_extra_attacks() {
    let result = run("hotori burst;
        while .hotori.time_stop.active {hotori attack; wait(3);}
        mint swap;");
    completed(&result);
    assert_eq!(times(&result, "hotori_time_stop_expired"), [14_183_333]);
    assert!(times(&result, "hotori_time_stop_finisher").is_empty());
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit
                .damage_effect_id
                .starts_with("GE_Player_Jin_UltraSkillMelee"))
            .count(),
        7
    );
}

#[test]
fn source_hotori_buds_preserve_their_owner_pause_policy_and_restored_lifetimes() {
    for level in [30, 80] {
        let source = format!(
            "options duration=30 iteration=1 crit=fixed;
            mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
            hotori char lvl={level}/{level} awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
            target hp=100000000 def=0 resist=0; active mint;
            hotori swap; hotori burst;"
        );
        let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
        completed(&result);
        let entry = times(&result, "hotori_time_stop_enter")[0];
        let setup = entry - result.occurrences[1].started_us;
        let pistils: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "ntesim.reaction.blossom")
            .map(|hit| hit.time_us)
            .collect();
        let expected: Vec<_> = if level == 30 {
            vec![]
        } else {
            [2_000_000, 4_000_000, 6_000_000, 8_000_000, 10_000_000]
                .into_iter()
                .map(|time| time + setup)
                .collect()
        };
        assert_eq!(pistils, expected, "level {level}, entry {entry}");
    }
}

#[test]
fn source_hotori_object_lifetime_pauses_in_setup_and_progresses_in_active_stop() {
    let source = "options duration=30 iteration=1 crit=fixed;
        haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
        target hp=100000000 def=0 resist=0; active haniel;
        haniel skill; let speaker_remaining=.haniel.mods.hootie;
        hotori burst;
        if !.haniel.hootie.active || .haniel.mods.hootie>=speaker_remaining {let assertion_failure=1/0;}
        wait(.haniel.mods.hootie);
        if .haniel.hootie.active || !.hotori.time_stop.active {let assertion_failure=1/0;}";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    completed(&result);
    let entry = times(&result, "hotori_time_stop_enter")[0];
    let expiry = result.occurrences.last().unwrap().completed_us.unwrap();
    assert!(expiry > entry && expiry < entry + 12_000_000);
}
