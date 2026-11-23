use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};

fn run(source: &str) -> SourceScenarioReport {
    let program =
        PreparedSourceScenario::new(source, Default::default(), &Default::default()).unwrap();
    let program = PreparedSourceScenario::from_export(
        &serde_json::from_slice(&serde_json::to_vec(&program.export()).unwrap()).unwrap(),
        &Default::default(),
    )
    .unwrap();
    let result = program.run(42).unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    result
}

#[test]
fn source_aim_keeps_edgar_held_skill_single_target_and_native_pulse_times() {
    let source = "options duration=80 crit=random;
        mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        edgar char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        target hp=100000000 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active mint;
        repeat 2 {edgar skill[target=t1,hold=4.9]; wait(25);}";
    let result = run(source);
    assert_eq!(result.request.aim_targets.len(), 2);
    for (&command, &target) in &result.request.aim_targets {
        assert_eq!(target, 1);
        let hits: Vec<_> = result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.command_index == Some(command))
            .collect();
        assert_eq!(hits.len(), 7);
        for (pulse, hit) in hits.iter().enumerate() {
            assert_eq!(hit.target_index, 1);
            assert_eq!(
                hit.time_us,
                result.request.actions[command].timestamp_us() + 821_980 + 600_000 * pulse as u64
            );
        }
    }
    super::assert_native_replay(&result);
    let scheduled = crate::dsl::compiler::compile_scenario(source, Default::default()).unwrap();
    assert_eq!(scheduled.request.aim_targets, result.request.aim_targets);
}

#[test]
fn source_aim_preserves_ordinary_owner_packet_trains_and_area_counts() {
    for owner in [
        "adler",
        "aurelia",
        "baicang",
        "blackbird",
        "chaos",
        "chiz",
        "daffodil",
        "edgar",
        "fadia",
        "haniel",
        "hathor",
        "hotori",
        "iroi",
        "jiuyuan",
        "lacrimosa",
        "linko",
        "mint",
        "nanally",
        "sakiri",
        "shinku",
        "skia",
        "zankou",
        "zero",
    ] {
        let source = format!(
            "options duration=60 crit=random;
            {owner} char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
            target hp=100000000 def=0 resist=0;
            target hp=100000000 def=0 resist=0; active {owner};
            repeat 2 {{{owner} attack[target=t1]:5; wait(2);}}"
        );
        let aimed = run(&source);
        let default = run(&source.replace("[target=t1]", ""));
        let packets = |result: &SourceScenarioReport| {
            let mut packets: Vec<_> = result
                .report
                .hit_receipts
                .iter()
                .map(|hit| {
                    (
                        hit.time_us,
                        hit.damage_effect_id.clone(),
                        hit.damage.numerator,
                        hit.damage.denominator,
                    )
                })
                .collect();
            packets.sort();
            packets
        };
        assert_eq!(packets(&aimed), packets(&default), "{owner}");
        assert_eq!(
            serde_json::to_value(&aimed.report.final_actor_states).unwrap(),
            serde_json::to_value(&default.report.final_actor_states).unwrap(),
            "{owner}"
        );
        for &command in aimed.request.aim_targets.keys() {
            assert!(
                aimed
                    .report
                    .hit_receipts
                    .iter()
                    .any(|hit| hit.command_index == Some(command) && hit.target_index == 1),
                "{owner}: {command}"
            );
        }
        super::assert_native_replay(&aimed);
    }
}

#[test]
fn source_aim_is_retained_by_skia_delayed_gnaw_and_its_command_identity() {
    let result = run("options duration=40 crit=random;
        mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        skia char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        target hp=100000000 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active mint;
        repeat 2 {skia skill[target=t1]; wait(13);}");
    let shadows: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Skia_SkillShadowAtk_Damage")
        .collect();
    assert_eq!(shadows.len(), 2);
    for hit in shadows {
        assert_eq!(hit.target_index, 1);
        assert_eq!(result.request.aim_targets[&hit.command_index.unwrap()], 1);
    }
    super::assert_native_replay(&result);
}

#[test]
fn source_target_identifiers_are_static_typed_and_have_exact_utf16_diagnostics() {
    use crate::dsl::{compiler::SourceRequest, source::compile_response};
    for target in ["t2", "t01", "t", "mint", "1", "\"t1\"", "t0+1"] {
        let source = format!(
            "# 🐦\noptions duration=10 crit=random; mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
            target hp=100000000 def=0 resist=0;
            target hp=100000000 def=0 resist=0; active mint;
            if 0 {{mint attack[target={target}];}}"
        );
        let response = compile_response(&SourceRequest {
            source: source.clone(),
            limits: Default::default(),
        });
        assert!(response.compiled.is_none(), "{target}");
        let diagnostic = &response.diagnostics[0];
        assert_eq!(
            diagnostic.code,
            crate::dsl::validation::DiagnosticCode::Configuration
        );
        let span = source
            .encode_utf16()
            .skip(diagnostic.loc.offset)
            .take(diagnostic.loc.length)
            .collect::<Vec<_>>();
        assert_eq!(
            String::from_utf16(&span).unwrap(),
            if target == "t0+1" { "+" } else { target }
        );
        assert_eq!(diagnostic.loc.line, 5);
        assert!(diagnostic.loc.column > 1);
    }
}

#[test]
fn source_aim_preserves_iroi_twenty_target_cap_and_autonomous_pulse_selection() {
    let source = format!(
        "options duration=20 crit=random;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_moon=32;
        {} active iroi; iroi burst[target=t20];",
        "target hp=100000000 def=0 resist=0;".repeat(21)
    );
    let result = run(&source);
    let recipients = |packet: &str| {
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == packet)
            .map(|hit| hit.target_index)
            .collect::<std::collections::BTreeSet<_>>()
    };
    let expected_direct = (0..19).chain([20]).collect();
    assert_eq!(
        recipients("GE_Player_Oneiroi_UltraSkill_Damage"),
        expected_direct
    );
    assert_eq!(
        recipients("GE_Player_Oneiroi_UltraSkill1_Damage"),
        (0..20).collect()
    );
    super::assert_native_replay(&result);
}

#[test]
fn native_aim_binding_rejects_missing_wait_commands_and_out_of_range_dummies() {
    let base = super::research_request();
    let mut request = base.clone();
    request.action_metadata.clear();
    request.actions = vec![crate::simulation::GameActionCommand::Wait {
        timestamp_us: 0,
        duration_us: 1,
    }];
    for (command, target) in [(0, 0), (1, 0)] {
        request.aim_targets = [(command, target)].into();
        assert!(
            crate::simulation::validate_game_request(&request, &Default::default())
                .unwrap_err()
                .contains("must identify a combat input")
        );
    }
    request = base;
    request.aim_targets = [(0, request.targets.len())].into();
    assert!(
        crate::simulation::validate_game_request(&request, &Default::default())
            .unwrap_err()
            .contains("unknown target")
    );
}

#[test]
fn source_switch_aim_retains_two_paid_cycle_generations_and_reaction_recipient_policies() {
    for (old, new, event, targeted) in [
        ("mint", "hotori", "reaction_blossom_created", false),
        ("mint", "adler", "reaction_hexed_applied", true),
        ("adler", "lacrimosa", "reaction_scorch_applied", false),
        ("lacrimosa", "blackbird", "", false),
        ("blackbird", "hathor", "reaction_stain_applied", true),
        ("hotori", "hathor", "reaction_remora_applied", true),
    ] {
        let source = format!(
            "options duration=50 crit=random;
            {old} char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
            {new} char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
            target hp=100000000 def=0 resist=0;
            target hp=100000000 def=0 resist=0; active {old};
            {new} swap[target=t1]; {new} attack[target=t1]; wait(16);
            {old} swap[target=t1]; {old} attack[target=t1]; wait(16);"
        );
        let result = run(&source);
        let control = run(&source.replace("target=t1", "target=t0"));
        let damage = |report: &SourceScenarioReport| {
            let mut values: Vec<_> = report
                .report
                .hit_receipts
                .iter()
                .map(|hit| {
                    (
                        hit.time_us,
                        hit.damage_effect_id.clone(),
                        hit.damage.numerator,
                        hit.damage.denominator,
                    )
                })
                .collect();
            values.sort();
            values
        };
        assert_eq!(damage(&result), damage(&control), "{old}/{new}");
        let events: Vec<_> = result
            .report
            .trace
            .iter()
            .filter(|trace| trace.event_kind == event)
            .collect();
        if !event.is_empty() {
            assert_eq!(events.len(), 2, "{old}/{new}: {event}");
        }
        if targeted {
            assert!(
                events
                    .iter()
                    .all(|trace| trace.data.as_ref().and_then(|d| d.target_index) == Some(1)),
                "{old}/{new}"
            );
        }
        for actor in &result.report.final_actor_states {
            assert_eq!(actor.cycle.as_ref().unwrap().spent, 100, "{old}/{new}");
        }
        for packet in ["ntesim.reaction.scorch", "ntesim.reaction.nova"] {
            assert!(
                result
                    .report
                    .hit_receipts
                    .iter()
                    .filter(|hit| hit.damage_effect_id == packet)
                    .all(|hit| hit.target_index == 1),
                "{old}/{new}: {packet}"
            );
        }
        if event == "reaction_blossom_created" {
            let hits: Vec<_> = result
                .report
                .hit_receipts
                .iter()
                .filter(|hit| hit.damage_effect_id == "ntesim.reaction.blossom")
                .collect();
            assert_eq!(hits.len(), 20);
            assert_eq!(hits.iter().filter(|hit| hit.target_index == 0).count(), 10);
        }
        super::assert_native_replay(&result);
    }
}

#[test]
fn source_targeted_switch_admits_blackbird_support_when_only_aimed_dummy_is_alive() {
    let result = run("options duration=20 crit=random;
        lacrimosa char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        blackbird char lvl=80/80 awakening=0 talent=8,8,8 qte=8;
        target hp=1 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active lacrimosa;
        lacrimosa attack;
        if .blackbird.support.ready {let assertion_failure=1/0;}
        blackbird swap[target=t1]; wait(12);");
    assert_eq!(result.report.final_target_states[0].hp_current, 0);
    let support = result
        .report
        .action_metadata_ledger
        .iter()
        .find(|entry| {
            entry.actor_index == 1
                && result
                    .report
                    .action_metadata(entry)
                    .unwrap()
                    .action
                    .input_role
                    == Some(crate::simulation::metadata::InputActionRole::Qte)
        })
        .unwrap();
    assert_eq!(support.command_indices.len(), 1);
    assert_eq!(result.request.aim_targets[&support.command_indices[0]], 1);
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "ntesim.reaction.nova" && hit.target_index == 1)
    );
    super::assert_native_replay(&result);
}

#[test]
fn source_switch_aim_keeps_overlapping_remora_removal_callbacks_per_target() {
    let result = run("options duration=30 crit=random;
        hotori char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        hathor char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_cycle=100;
        chaos char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_crime=0;
        target hp=100000000 def=0 resist=0;
        target hp=100000000 def=0 resist=0; active hotori;
        hathor swap[target=t0]; wait(5);
        hotori swap[target=t1]; wait(13);");
    let applications: Vec<_> = result
        .report
        .trace
        .iter()
        .filter(|trace| trace.event_kind == "reaction_remora_applied")
        .collect();
    assert_eq!(applications.len(), 2);
    assert!(applications[1].time_us < applications[0].time_us + 12_000_000);
    let hits: Vec<_> = result
        .report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == crate::characters::chaos::remora::PACKET)
        .collect();
    assert_eq!(hits.len(), 2);
    for (target, hit) in hits.iter().enumerate() {
        assert_eq!(hit.target_index, target);
        assert_eq!(hit.time_us, applications[target].time_us + 12_000_000);
    }
    super::assert_native_replay(&result);
}

#[test]
fn stop_time_halts_with_success_on_target_defeat() {
    let result = run("options duration=120 stop=time crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=10 def=0 resist=0; active mint;
        while .time + 5 < .duration {
            mint attack;
        }
        wait_until(.duration);");
    assert!(result.report.execution_completed);
    assert_eq!(result.report.final_target_states[0].hp_current, 0);
    assert!(result.report.absolute_elapsed_us < 120_000_000);
}

#[test]
fn stop_victory_ignores_duration_and_runs_until_all_targets_defeated() {
    let result = run("options duration=1 stop=victory crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=1000 def=0 resist=0; active mint;
        while 1 {
            mint attack;
        }");
    assert!(result.report.execution_completed);
    assert_eq!(result.report.final_target_states[0].hp_current, 0);
}

#[test]
fn stop_victory_without_duration_allows_evaluating_duration_field() {
    let result = run("options stop=victory crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=1000 def=0 resist=0; active mint;
        while .time < .duration {
            mint attack;
        }");
    assert!(result.report.execution_completed);
    assert_eq!(result.report.final_target_states[0].hp_current, 0);
}
