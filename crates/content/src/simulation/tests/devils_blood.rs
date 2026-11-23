use super::*;
use crate::formula::Ratio;
use crate::simulation::GameActionCommand;
use ntesim_core::buffs::dispatch::{DecayPolicy, RefreshPolicy};
use serde_json::json;

fn fixture() -> GameScenarioRequest {
    let mut req = crate::simulation::tests::research_request();
    req.stochastic_crit = None;
    req.relative_duration_us = 100;
    req.actors[0].set_id = Some("set:Suit3".into());
    req.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 4,
    };
    let (proc, _) = crate::sets::devils_blood::projection::compile(
        ntesim_data::stat_projection::frozen_tables(),
    )
    .unwrap();
    req.actors[0].gear_procs = vec![proc];
    req
}

fn with_status(
    req: &GameScenarioRequest,
    tags: &[&str],
    target: usize,
    source_actor: usize,
) -> Value {
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(req, &limits).unwrap();
    let (mut execution, assumptions) = PreparedExecution::new(req, req.seed, &[], &[]).unwrap();
    for _ in 0..4 {
        crate::simulation::state::apply_target_tagged_status(
            &mut execution.state.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor,
                source: "test.teammate_reaction".into(),
                target_index: target,
                id: "test.reaction_status".into(),
                now_us: 0,
                duration_us: 30,
                reductions: BTreeMap::new(),
            },
            tags.iter().map(|tag| (*tag).to_owned()).collect(),
            4,
            RefreshPolicy::Reset,
            DecayPolicy::AllAtOnce,
        )
        .unwrap();
    }
    execution
        .process_events(req, &limits, &packets, &[], &[])
        .unwrap();
    serde_json::to_value(execution.into_report(req, req.seed, assumptions).unwrap()).unwrap()
}

#[test]
fn devils_blood_target_tags_add_once_in_the_same_damage_zone_and_expire_before_equal_time_hits() {
    let mut req = fixture();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.set_id = None;
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    for tags in [
        vec!["Ability.Player.Buff.Reaction.AnHun"],
        vec!["Ability.Player.Buff.Reaction.HunXiang"],
        vec![
            "Ability.Player.Buff.Reaction.AnHun",
            "Ability.Player.Buff.Reaction.HunXiang",
        ],
    ] {
        let report = with_status(&req, &tags, 0, 1);
        // 1000 * 1/2 * 4/5 * (1 + .25 + .18 + .18) = 644.
        // At t=30 the conditional .18 expires: 400 * 1.43 = 572.
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|hit| hit["non_crit_damage"].clone())
                .collect::<Vec<_>>(),
            [644, 644, 644, 572, 572, 572].map(|n| json!(format!("{}/{}", n, 1)))
        );
        assert_eq!(
            report["resolved_model_assumptions"][crate::sets::devils_blood::HYPOTHESIS_ID.as_str()],
            crate::sets::devils_blood::HYPOTHESIS_ID.description()
        );
    }
    for (tags, target) in [
        (vec!["Ability.Player.Buff.Reaction.AnHun"], 1),
        (vec!["State.Debuff"], 0),
    ] {
        let report = with_status(&req, &tags, target, 1);
        assert_eq!(report["hit_receipts"][0]["non_crit_damage"], json!("572/1"));
    }
}

#[test]
fn devils_blood_pending_hit_reads_delivery_status_and_keeps_the_off_field_wearer() {
    let mut req = fixture();
    req.actions.truncate(1);
    req.action_metadata.retain(|index, _| *index == 0);
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.set_id = None;
    ally.gear_procs.clear();
    req.actors.push(ally);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 10,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let mut ally_action = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        actor_index,
        timestamp_us,
        ..
    } = &mut ally_action
    {
        *actor_index = 1;
        *timestamp_us = 40;
    }
    req.actions.push(ally_action);
    req.action_metadata
        .insert(2, req.action_metadata[&0].clone());
    for (delay, damage) in [(20, 644), (30, 572)] {
        if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
            *hit_delay_us = delay;
        }
        let report = with_status(&req, &["Ability.Player.Buff.Reaction.AnHun"], 0, 1);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            json!(format!("{}/{}", damage, 1))
        );
        assert_eq!(report["hit_receipts"][0]["actor_index"], 0);
        assert_eq!(report["hit_receipts"][1]["non_crit_damage"], json!("500/1"));
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    }
}

#[test]
fn devils_blood_projection_binds_both_curves_and_validates_native_equipment() {
    let tables = ntesim_data::stat_projection::frozen_tables();
    let (proc, native) = crate::sets::devils_blood::projection::compile(tables).unwrap();
    assert_eq!(
        serde_json::to_value(&proc).unwrap(),
        json!({"kind":"devils_blood_damage",
        "damage_bonus":"9/50","affected_bonus":"9/50"})
    );
    assert_eq!(
        native["parameters"]["Psyche_4_2"]["selected"],
        json!({"numerator":"9","denominator":"50"})
    );
    let mut actor = fixture().actors.remove(0);
    actor.gear_procs.clear();
    let equipped =
        json!({"set_id":"set:Suit3","set_count":4,"unresolved_set_buffs":[{"threshold":4}]});
    let project = crate::sets::devils_blood::projection::project_equipped;
    assert!(project(tables, &mut actor, &equipped).unwrap().is_some());
    assert!(project(tables, &mut actor, &equipped).unwrap().is_some());
    assert_eq!(actor.gear_procs.len(), 1);
    actor.gear_procs.push(proc);
    assert!(project(tables, &mut actor, &equipped).is_err());
    actor.gear_procs.clear();
    let two_piece = json!({"set_id":"set:Suit3","set_count":2,"unresolved_set_buffs":[]});
    assert!(project(tables, &mut actor, &two_piece).unwrap().is_none());
    assert!(
        project(
            tables,
            &mut actor,
            &json!({"set_id":"set:Suit3","set_count":4,"unresolved_set_buffs":[]})
        )
        .is_err()
    );
    for value in [json!(-0.18), json!(18)] {
        let mut changed = tables.clone();
        changed.insert(
            "CT_Equipmentbuff".into(),
            ntesim_data::set_parameters::overlay(),
        );
        changed.get_mut("CT_Equipmentbuff").unwrap()["rows"]["Psyche_4_2"]["Keys"][0]["Value"] =
            value;
        assert!(crate::sets::devils_blood::projection::compile(&changed).is_err());
    }
    let mut invalid = fixture();
    invalid.actors[0].set_id = None;
    assert!(
        crate::simulation::validate_game_request(&invalid, &ExecutionLimits::default())
            .unwrap_err()
            .contains("set:Suit3")
    );
}

#[test]
fn devils_blood_source_build_enables_four_piece_without_multiplying_or_reapplying_static_stats() {
    let run = |count| {
        let compiled = crate::dsl::compiler::compile_scenario(&format!(
            "options iteration=1 duration=2 crit=fixed; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; \
             mint add cartridge=Suit3 count={count}; mint add stats dmg%=0.25; \
             target hp=100000000 def=0 resist=0; active mint; mint attack;"
        ), Default::default()).unwrap();
        crate::simulation::run_game_scenario(&compiled.request, &ExecutionLimits::default())
            .unwrap()
    };
    let two = run(2);
    let four = run(4);
    let value = |report: &Value| {
        serde_json::from_value::<crate::simulation::ExactFraction>(
            report["hit_receipts"][0]["non_crit_damage"].clone(),
        )
        .unwrap()
    };
    // Mint's Anima packet receives no two-piece Psyche bonus. 1.43 / 1.25 = 143/125.
    assert_eq!(
        value(&four),
        value(&two)
            .mul(crate::simulation::ExactFraction::new(143, 125).unwrap())
            .unwrap()
    );
}
