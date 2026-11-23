use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};
use serde_json::json;

fn source(initial: &str, awakening: u8, actions: &str) -> String {
    format!(
        "options duration=30 crit=random;
        haniel char lvl=80/80 awakening={awakening} talent=1,1,1 qte=1 {initial};
        target hp=100000000 def=0 resist=0; active haniel; {actions}"
    )
}
fn run(text: &str) -> SourceScenarioReport {
    let result = PreparedSourceScenario::new(text, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; actor={:?}; contacts={:?}",
        result.diagnostic,
        result.report.final_actor_states[0],
        result
            .report
            .trace
            .iter()
            .filter(
                |event| event.event_kind.contains("harmony") || event.event_kind.contains("plot")
            )
            .collect::<Vec<_>>()
    );
    result
}
fn replay(result: &SourceScenarioReport) {
    let native =
        crate::simulation::run_game_scenario_report(&result.request, &Default::default()).unwrap();
    let actual = serde_json::to_value(&result.report).unwrap();
    let native = serde_json::to_value(native).unwrap();
    for (key, value) in actual.as_object().unwrap() {
        assert_eq!(value, &native[key], "Plot native replay {key}");
    }
}

#[test]
fn source_haniel_native_cycles_fund_two_bursts_and_fast_cannons_from_empty_energy() {
    let result = run(include_str!(
        "../../../../../examples/haniel-continuing-energy.nte"
    ));
    assert_eq!(result.request.actors[0].initial_energy, 0);
    assert_eq!(result.request.actors[0].energy_capacity, 120);
    assert_eq!(result.report.final_actor_states[0].spent_energy, 240);
    assert_eq!(result.request.hold_inputs.len(), 2);
    for ability in ["GA_Haniel_Skill", "GA_Haniel_UltraSkill"] {
        assert_eq!(
            result
                .report
                .action_metadata_ledger
                .iter()
                .filter(|entry| {
                    result
                        .report
                        .action_metadata(entry)
                        .unwrap()
                        .action
                        .ability_id
                        == ability
                })
                .map(|entry| entry.command_indices.len())
                .sum::<usize>(),
            2
        );
    }
    let cannon: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == crate::characters::haniel::cannon::PACKET)
        .collect();
    // First capture is Plot0; the continuing speaker/Ensembles fill Plot4
    // before period two. One primary plus four two-dummy ricochets is nine.
    let counts: Vec<_> = result
        .request
        .hold_inputs
        .keys()
        .map(|command| {
            cannon
                .iter()
                .filter(|hit| hit.command_index == Some(*command))
                .count()
        })
        .collect();
    assert_eq!(counts, vec![1, 9]);
    for hit in cannon {
        // .2s Normal entry + first strict .400001s charge gate + .044596s emission.
        assert_eq!(
            hit.time_us,
            result.request.actions[hit.command_index.unwrap()].timestamp_us() + 644_597
        );
    }
    for effect in [
        crate::characters::haniel::hootie::ENSEMBLE,
        crate::characters::haniel::hootie::SYMPHONY,
        crate::characters::haniel::hootie::EXTRA,
    ] {
        assert!(
            result
                .report
                .hit_receipts
                .iter()
                .any(|hit| hit.damage_effect_id == effect),
            "{effect}"
        );
    }
    replay(&result);
}

#[test]
fn source_plot_preload_and_harmony_continue_through_two_speakers_without_rng_draws() {
    let actions = "if .haniel.resources.plot!=3 || .haniel.resources.harmony!=0
            || .haniel.hootie.active || .haniel.paranormal_ace.active
            || .haniel.symphony_token.active || .haniel.out_of_combat.active || .haniel.mods.hootie!=0
            || .haniel.mods.paranormal_ace!=0 || .haniel.mods.symphony_token!=0 {let assertion_failure=1/0;}
        haniel skill;
        if !.haniel.hootie.active {let assertion_failure=1/0;}
        haniel attack:4;
        if .haniel.resources.plot!=4 || .haniel.resources.harmony!=0 {let assertion_failure=1/0;}
        wait_until(10);
        if .haniel.hootie.active || .haniel.mods.hootie!=0 || .haniel.resources.plot!=4 {let assertion_failure=1/0;}
        wait_until(21); haniel skill; haniel attack:4;
        if .haniel.resources.plot!=4 || .haniel.resources.harmony!=0 {let assertion_failure=1/0;}";
    let result = run(&source("start_plot=3", 0, actions));
    let baseline = run(&source(
        "start_plot=3",
        0,
        "haniel skill; haniel attack:4; wait_until(10); wait_until(21); haniel skill; haniel attack:4;",
    ));
    super::assert_same_combat(&result.report, &baseline.report);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["final_actor_states"][0]["resources"]["haniel.plot_armor"]["current"],
        4
    );
    replay(&result);
}

#[test]
fn source_plot_ace_extension_and_unconsumed_token_expire_at_native_boundaries() {
    let actions = "haniel skill;
        let speaker=.haniel.mods.hootie;
        let cast=.time;
        let ace_end=cast+10;
        let speaker_end=cast+speaker+10;
        haniel burst;
        if !.haniel.paranormal_ace.active || .haniel.mods.paranormal_ace!=ace_end-.time
            || .haniel.mods.hootie!=speaker_end-.time {let assertion_failure=1/0;}
        # Burst supplied one contact; retain its half-second World gate through release.
        while .time<cast+0.5 {wait(cast+0.5-.time);}
        haniel attack:3;
        if .haniel.resources.plot!=2 || .haniel.resources.harmony!=0
            || !.haniel.symphony_token.active {let assertion_failure=1/0;}
        let token_end=.time+.haniel.mods.symphony_token;
        wait(token_end-.time);
        if .haniel.symphony_token.active || .haniel.mods.symphony_token!=0 {let assertion_failure=1/0;}
        wait(ace_end-.time);
        if .haniel.paranormal_ace.active || .haniel.mods.paranormal_ace!=0
            || !.haniel.hootie.active || .haniel.mods.hootie!=speaker_end-.time {let assertion_failure=1/0;}
        wait(speaker_end-.time);
        if .haniel.hootie.active || .haniel.resources.harmony!=0 || .haniel.resources.plot!=2 {let assertion_failure=1/0;}";
    let result = run(&source("start_plot=0", 0, actions));
    replay(&result);
}

#[test]
fn source_plot_capacity_integrality_owner_and_zero_validate() {
    for (awakening, capacity) in [(0, 4), (1, 6)] {
        run(&source(
            &format!("start_plot={capacity}"),
            awakening,
            &format!("if .haniel.resources.plot!={capacity} {{let assertion_failure=1/0;}}"),
        ));
        for value in ["-1".into(), "0.5".into(), (capacity + 1).to_string()] {
            assert!(
                PreparedSourceScenario::new(
                    &source(&format!("start_plot={value}"), awakening, ""),
                    Default::default(),
                    &Default::default()
                )
                .is_err()
            );
        }
    }
    let wrong = source("start_plot=0", 0, "").replace("haniel", "mint");
    assert!(PreparedSourceScenario::new(&wrong, Default::default(), &Default::default()).is_err());
    let result = run(&source(
        "start_plot=0",
        1,
        "if .haniel.resources.plot!=0 {let assertion_failure=1/0;}",
    ));
    assert_eq!(
        serde_json::to_value(&result.request).unwrap()["actors"][0]["initial_resources"]["plot"],
        json!("0/1")
    );
    let mut request = result.request;
    request.actors[0].initial_resources.plot = Some(crate::formula::Ratio {
        numerator: 7,
        denominator: 1,
    });
    assert!(crate::simulation::PreparedGameScenario::new(&request, &Default::default()).is_err());
}

#[test]
fn source_plot_owner_death_clears_preload_harmony_and_timed_states() {
    let text = source("start_plot=3", 0, "haniel skill; wait_until(2);
        if .haniel.hp!=0 || .haniel.resources.plot!=0 || .haniel.resources.harmony!=0
            || .haniel.hootie.active || .haniel.paranormal_ace.active || .haniel.symphony_token.active
            || .haniel.mods.hootie!=0 || .haniel.mods.paranormal_ace!=0 || .haniel.mods.symphony_token!=0 {let assertion_failure=1/0;}")
        .replace("active haniel;", "mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; incoming haniel at=2 damage=1000000; active haniel;");
    let result = run(&text);
    replay(&result);
}
