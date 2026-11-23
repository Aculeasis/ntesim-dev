use crate::simulation::{GameActionCommand, PreparedSourceScenario, SourceScenarioReport};

const PREFIX: &str = "options duration=10 crit=random;
    mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
    zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
    target hp=100000000 def=0 resist=0; active mint;";

fn run(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn source_readiness_is_available_off_field_and_cooldowns_expire_on_native_time() {
    let result = run(&format!(
        "{PREFIX}
        if !.mint.skill.ready || .mint.skill.cd!=0 || .mint.burst.ready
            || !.zero.burst.ready {{let assertion_failure=1/0;}}
        mint skill;
        if .mint.skill.ready || .mint.skill.cd<=0 || .mint.skill.cd+.time!=6.182791 {{let assertion_failure=1/0;}}
        zero attack;
        if .mint.skill.ready || !.zero.skill.ready {{let assertion_failure=1/0;}}
        wait_until(6.182790);
        if .mint.skill.ready || .mint.skill.cd!=0.000001 {{let assertion_failure=1/0;}}
        wait(0.000001);
        if !.mint.skill.ready || .mint.skill.cd!=0 {{let assertion_failure=1/0;}}
        mint skill;"
    ));
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let skills = result.request.actions.iter().filter(|command| matches!(command,
        GameActionCommand::InvokeAbility { ability_id, .. } if ability_id=="GA_Mint019_Skill"
    )).count();
    assert_eq!(skills, 2);
}

#[test]
fn source_hotori_readiness_uses_exact_owner_resource_prerequisites_off_field() {
    let result = run("options duration=22 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=0 resist=0; active mint;
        if .hotori.skill.ready || .hotori.burst.ready {let assertion_failure=1/0;}
        wait_until(10);
        if .hotori.skill.ready {let assertion_failure=1/0;}
        wait_until(10.000001);
        if !.hotori.skill.ready || .hotori.burst.ready {let assertion_failure=1/0;}
        wait_until(20);
        if !.hotori.burst.ready || .hotori.energy!=0 {let assertion_failure=1/0;}");
    // Native Timepiece grows at6/s: exactly60 rejects Skill, >60 permits it,
    // and exactly120 permits Burst despite zero conventional Ultimate energy.
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.total_events_processed, 0);
}

#[test]
fn source_support_queries_do_not_dispatch_spend_advance_or_draw_rng() {
    let prefix = PREFIX.replace("start_energy=0", "start_energy=0 start_cycle=100");
    let baseline = run(&format!("{prefix} zero attack;"));
    let observed = run(&format!(
        "{prefix}
        repeat 5 {{
            if !.zero.support.ready || .mint.support.ready {{let assertion_failure=1/0;}}
        }}
        zero attack;
        if .zero.support.ready {{let assertion_failure=1/0;}}"
    ));
    assert!(
        observed.report.success && observed.report.execution_completed,
        "{:?}",
        observed.diagnostic
    );
    super::assert_same_combat(&observed.report, &baseline.report);
    assert!(observed.report.action_metadata_ledger.iter().any(|entry| {
        observed
            .report
            .action_metadata(entry)
            .unwrap()
            .action
            .input_role
            == Some(crate::simulation::metadata::InputActionRole::Qte)
    }));
}

#[test]
fn source_rechecks_admission_after_an_implicit_support_and_incoming_event() {
    let prefix = PREFIX.replace("start_energy=0", "start_energy=0 start_cycle=100");
    let result = run(&format!(
        "{prefix}
        incoming zero at=0.3 damage=100000000;
        if .zero.skill.ready {{zero skill;}} else {{let assertion_failure=1/0;}}"
    ));
    assert!(!result.report.success && !result.report.execution_completed);
    let rejected = result.report.rejected_action.as_ref().unwrap();
    assert_eq!(rejected.actor_index, 1);
    assert!(result.report.final_actor_states[1].hp == 0);
    assert!(
        result
            .diagnostic
            .as_ref()
            .is_some_and(|diagnostic| diagnostic.message.contains("DEFEATED_ACTOR"))
    );
}

#[test]
fn source_initial_readiness_agrees_with_native_admission_for_all_released_owners() {
    let owners = crate::registry::character_descriptors().unwrap();
    assert_eq!(owners.len(), 23);
    let mut mismatches = Vec::new();
    for owner in owners {
        for action in ["skill", "burst"] {
            let name = owner.slug;
            let source = format!(
                "options duration=30 crit=fixed;
                {name} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
                target hp=100000000 def=0 resist=0; active {name};
                if 0 {{{name} {action};}}
                wait(.{name}.{action}.ready/1000000);"
            );
            let observed = run(&source);
            assert!(
                observed.report.success && observed.report.execution_completed,
                "{name} {action}: {:?}",
                observed.diagnostic
            );
            let query = &observed.occurrences[0];
            let ready = query.completed_us.unwrap() - query.started_us == 1;
            let (_, config) = crate::dsl::inspection::prepare(&source).unwrap();
            let mut request = observed.request;
            request.input_time_contexts.clear();
            request.actions = vec![
                config.actors[0].projection.public_actions[action]
                    .command
                    .clone(),
            ];
            let native =
                match crate::simulation::run_game_scenario_report(&request, &Default::default()) {
                    Ok(report) => report,
                    Err(error) => {
                        mismatches.push(format!("{name} {action}: {error}"));
                        continue;
                    }
                };
            let admitted = native
                .action_metadata_ledger
                .iter()
                .any(|entry| entry.command_indices.contains(&0));
            if ready != admitted {
                mismatches.push(format!(
                    "{name} {action}: ready={ready}, admitted={admitted}, {:?}",
                    native.halt_diagnostic
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn source_support_readiness_matches_the_actual_entry_for_all_released_owners() {
    use ntesim_data::character::Element;
    let mut mismatches = Vec::new();
    for owner in ntesim_data::character::definitions()
        .iter()
        .filter(|owner| owner.released)
    {
        let name = owner.slug;
        let old = match owner.element {
            Element::Chaos => "sakiri",
            Element::Incantation | Element::Cosmos => "mint",
            Element::Nature | Element::Lakshana => "zero",
            Element::Psyche => "daffodil",
        };
        let source = format!(
            "options duration=30 crit=fixed;
            {old} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
            {name} char lvl=80/80 awakening=0 talent=1,1,1 qte=8;
            target hp=100000000 def=0 resist=0 original_atk=1000;
            target hp=100000000 def=0 resist=0 original_atk=1000; active {old};
            wait(.{name}.support.ready/1000000);
            {name} swap[target=t1];"
        );
        let observed = run(&source);
        assert!(
            observed.report.success && observed.report.execution_completed,
            "{name}: {:?}",
            observed.diagnostic
        );
        let query = &observed.occurrences[0];
        let ready = query.completed_us.unwrap() - query.started_us == 1;
        let dispatched = observed.report.action_metadata_ledger.iter().any(|entry| {
            entry.actor_index == 1
                && observed
                    .report
                    .action_metadata(entry)
                    .unwrap()
                    .action
                    .input_role
                    == Some(crate::simulation::metadata::InputActionRole::Qte)
        });
        let ability =
            crate::simulation::talents::qte_ability(&format!("character:{}", owner.native_id))
                .unwrap();
        let delivered = observed
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.ability_id == ability);
        if !ready || !dispatched || !delivered {
            mismatches.push(format!(
                "{name}: ready={ready}, dispatched={dispatched}, delivered={delivered}"
            ));
        }
        assert_eq!(
            observed
                .request
                .aim_targets
                .values()
                .copied()
                .collect::<Vec<_>>(),
            vec![1]
        );
        super::assert_native_replay(&observed);
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn readiness_keeps_native_packet_mode_guards_without_recovery_or_field_mutation() {
    let source = "options duration=1 crit=fixed;
        fadia char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0; active fadia;
        if 0 {fadia skill;}";
    let initial = run(source);
    let mut state = crate::simulation::state::GameSimulatorState::new(&initial.request);
    // Internal settled-state fixture; source combat transitions remain Q4-L5.
    state.actors[0].character.fadia_mut().unwrap().full_kit = true;
    state.actors[0].locked_until_us = 1_000_000;
    let inspect = |state: &crate::simulation::state::GameSimulatorState| {
        crate::simulation::availability::ability(
            &initial.request,
            state,
            0,
            "GA_Fadia_Skill",
            "GE_Player_Fadia_Skill1_Damage",
            0,
        )
        .ready
    };
    assert!(
        inspect(&state),
        "readiness does not waive the later recovery admission"
    );
    state.actors[0].character.fadia_mut().unwrap().out_of_combat = true;
    assert!(
        !inspect(&state),
        "the owner's packet admission still applies"
    );
    assert_eq!(state.actors[0].locked_until_us, 1_000_000);
    assert_eq!(state.current_time_us, 0);
}

#[test]
fn source_fadia_skill_and_chiz_burst_initialize_their_existing_owner_dependencies() {
    for (name, action, model) in [
        ("fadia", "skill", crate::characters::fadia::effects::MODEL),
        ("chiz", "burst", crate::characters::chiz::market::MODEL),
    ] {
        let result = run(&format!(
            "options duration=30 crit=fixed;
            {name} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
            target hp=100000000 def=0 resist=0; active {name};
            if .{name}.{action}.ready {{{name} {action};}} else {{let assertion_failure=1/0;}}"
        ));
        assert!(
            result.report.success && result.report.execution_completed,
            "{name}: {:?}",
            result.diagnostic
        );
        assert!(result.report.hits_settled > 0);
        assert!(
            result
                .report
                .resolved_model_assumptions
                .contains_key(model.as_str())
        );
        let replay =
            crate::simulation::run_game_scenario_report(&result.request, &Default::default())
                .unwrap();
        let actual = serde_json::to_value(&result.report).unwrap();
        let replay = serde_json::to_value(replay).unwrap();
        for (key, value) in actual.as_object().unwrap() {
            assert!(value == &replay[key], "{name} replay field {key}");
        }
    }
}
