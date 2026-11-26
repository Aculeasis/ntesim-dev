use super::*;
use crate::simulation::GameActionCommand;
use serde_json::json;

pub(super) fn request(remora_at: u64) -> GameScenarioRequest {
    let source = "options duration=12; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100; hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100; hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; wait(1);";
    let mut request = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    for actor in &mut request.actors {
        actor.initial_energy = 0;
    }
    // Baseline shared Remora excludes Hathor's separately tested B2 extension.
    request.actors[2].character_level = 20;
    request.actors[2].character_max_level = Some(20);
    // Hotori's paid Support has a 1.283615s switch gate and .32s on contact.
    request.actions = [(0, 1), (remora_at.max(1_603_615), 2)]
        .map(
            |(timestamp_us, target_actor_index)| GameActionCommand::SwitchActor {
                timestamp_us,
                target_actor_index,
                cooldown_us: 0,
            },
        )
        .into();
    request
}

pub(super) fn run(request: &GameScenarioRequest, extra_remora: bool) -> Value {
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(request, &limits).unwrap();
    let (mut execution, assumptions) =
        PreparedExecution::new(request, request.seed, &[], &[]).unwrap();
    if extra_remora {
        use ntesim_core::buffs::dispatch::{
            BuffDefinition, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
        };
        execution
            .state
            .buffs
            .apply_buff(
                BuffDefinition {
                    source: "test.remora".into(),
                    id: "test.second_target_remora".into(),
                    recipient: BuffRecipient::SpecificTarget(1usize.into()),
                    stacking_scope: BuffStackingScope::Shared,
                    source_lifetime: Default::default(),
                    duration_us: Some(12_000_000),
                    max_stacks: 1,
                    refresh_policy: RefreshPolicy::Reset,
                    decay_policy: DecayPolicy::AllAtOnce,
                    stat_modifiers: BTreeMap::new(),
                    semantic_tags: std::collections::BTreeSet::from(
                        ["ntesim.status.remora".into()],
                    ),
                },
                0,
                0,
            )
            .unwrap();
    }
    execution
        .process_events(request, &limits, &packets, &[], &[])
        .unwrap();
    serde_json::to_value(
        execution
            .into_report(request, request.seed, assumptions)
            .unwrap(),
    )
    .unwrap()
}

pub(super) fn charges(report: &Value) -> Vec<&Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "reaction_charge_energy")
        .collect()
}

#[test]
fn remora_charge_observes_half_open_five_seconds_and_the_fifth_pistil() {
    for (remora_at, times) in [
        (3_000_000, vec![4_000_000, 6_000_000]),
        (7_000_000, vec![8_000_000, 10_000_000]),
    ] {
        let req = request(remora_at);
        let report = run(&req, false);
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let grants = charges(&report);
        assert_eq!(
            grants
                .iter()
                .map(|e| e["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            times
        );
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().len(),
            8,
            "five bud packets and three native Support packets; Charge adds none"
        );
        assert_eq!(report["final_actor_states"][2]["granted_energy"], 33);
        assert_eq!(
            report["final_actor_states"][2]["charge_remainder"],
            json!("9/10")
        );
        for (peer, energy, remainder) in [(0, 10, "31/50"), (1, 14, "21/50")] {
            assert_eq!(report["final_actor_states"][peer]["granted_energy"], energy);
            assert_eq!(
                report["final_actor_states"][peer]["charge_remainder"],
                json!(remainder)
            );
        }
        for grant in grants {
            assert_eq!(grant["energy_change"]["source"], "reaction_charge");
            assert_eq!(
                grant["energy_change"]["source_actor_index"], 0,
                "the bud retains its outgoing tie owner"
            );
            assert_eq!(grant["energy_change"]["recipient_actor_index"], 2);
            assert_eq!(grant["energy_change"]["charge_basis"], json!("10/1"));
        }
    }
}

#[test]
fn charge_is_once_per_emission_across_remora_targets_and_preserves_efficiency_carry_overflow() {
    let mut req = request(0);
    let mut second = req.targets[0].clone();
    second.target_index = 1;
    req.targets.push(second);
    req.actors[2].initial_energy = 95;
    req.actors[2]
        .gear_procs
        .push(crate::simulation::GearProcConfig::ChargeEfficiency {
            multiplier: crate::formula::Ratio {
                numerator: 4,
                denominator: 3,
            },
        });
    let report = run(&req, true);
    // The injected second-target status lasts through every emission. Ten HP
    // packets from five pistils still give only five Charge grants. Hotori
    // contributes 5.7 off field and Hathor contributes 8.2 on field.
    let grants = charges(&report);
    assert_eq!(grants.len(), 5);
    assert_eq!(
        grants
            .iter()
            .map(|g| g["energy_change"]["gross"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [13, 14, 13, 13, 14]
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 16);
    assert_eq!(report["final_actor_states"][2]["granted_energy"], 85);
    assert_eq!(report["final_actor_states"][2]["overflow_energy"], 80);
    assert_eq!(
        report["final_actor_states"][2]["charge_remainder"],
        json!("1/5")
    );
}

#[test]
fn charge_uses_live_active_recipient_skips_dead_and_retains_an_independent_bud() {
    let mut req = request(0);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 3_500_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    let report = run(&req, false);
    let grants = charges(&report);
    assert_eq!(grants.len(), 3);
    assert_eq!(grants[0]["energy_change"]["recipient_actor_index"], 2);
    assert_eq!(grants[1]["energy_change"]["recipient_actor_index"], 0);
    assert_eq!(grants[2]["energy_change"]["recipient_actor_index"], 0);
    req.actions.pop();
    let hypothesis = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(hypothesis.as_str().into(), hypothesis.description().into());
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 3_500_000,
        actor_index: 2,
        delivered_damage: req.actors[2].initial_hp,
    }];
    let report = run(&req, false);
    assert_eq!(charges(&report).len(), 1);
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 8);
    assert_eq!(report["final_actor_states"][2]["hp"], 0);
    for source_actor in [0, 1] {
        req.incoming_damage_events[0].actor_index = source_actor;
        req.incoming_damage_events[0].delivered_damage = req.actors[source_actor].initial_hp;
        let report = run(&req, false);
        assert_eq!(
            charges(&report).len(),
            3,
            "bud and Remora survive their independent appliers' deaths"
        );
        assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 8);
    }
}

#[test]
fn remora_refresh_preserves_one_status_and_target_death_cleans_it() {
    let mut req = request(4_000_000);
    req.actors.swap(0, 2);
    for (index, actor) in req.actors.iter_mut().enumerate() {
        actor.slot = index;
        actor.initial_cycle = 100;
    }
    req.relative_duration_us = 8_000_000;
    if let GameActionCommand::SwitchActor {
        target_actor_index, ..
    } = &mut req.actions[1]
    {
        *target_actor_index = 0;
    }
    let report = run(&req, false);
    assert!(
        charges(&report).is_empty(),
        "Remora without a bud emits no Charge events"
    );
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 3);
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    let statuses: Vec<_> = buffs
        .iter()
        .filter(|buff| buff["id"] == "ntesim.reaction.remora")
        .collect();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0]["stacks"], 1);
    assert_eq!(statuses[0]["duration"]["expires_at_us"], 9_000_000);
    let mut killed = request(3_000_000);
    // Keep the dummy alive through both Supports and the first bud. The
    // second bud supplies the killing connected hit during live Remora.
    let prefix = run(&killed, false);
    let support_loss: u64 = prefix["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["time_us"].as_u64().unwrap() < 4_000_000)
        .map(|hit| hit["applied_hp_loss"].as_u64().unwrap())
        .sum();
    killed.targets[0].hp_current = support_loss + 1;
    killed.relative_duration_us = 4_000_001;
    let report = run(&killed, false);
    assert_eq!(
        charges(&report).len(),
        1,
        "the killing hit still connected to a living Remora target"
    );
    assert_eq!(report["final_target_states"][0]["hp_current"], 0);
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| buff["id"] != "ntesim.reaction.remora")
    );
}

#[test]
fn hathor_enhanced_remora_extends_charge_eligibility_without_extra_emissions() {
    let mut req = request(0);
    req.actors[2].character_level = 80;
    req.actors[2].character_max_level = Some(80);
    let report = run(&req, false);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let grants = charges(&report);
    assert_eq!(grants.len(), 5);
    assert_eq!(grants.last().unwrap()["time_us"], 10_000_000);
    assert_eq!(report["final_actor_states"][2]["granted_energy"], 63);
    assert_eq!(
        report["final_actor_states"][2]["charge_remainder"],
        json!("9/10")
    );
    assert_eq!(report["hits_settled"], 8);
}

#[test]
fn apply_remora_from_owner_with_retrigger_control_applies_remora_status() {
    let req = request(0);
    let limits = ExecutionLimits::default();
    let _ = crate::simulation::validation::validate_and_resolve(&req, &limits).unwrap();
    let (mut exec, _) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
    let mut trace = Vec::new();
    let mut cb = crate::runtime_api::CallbackContext {
        state: &mut exec.state,
        request: &req,
        queue: &mut exec.queue,
        sequence: &mut exec.seq,
        trace: &mut trace,
    };
    crate::simulation::reactions::apply_remora_from_owner_with_retrigger_control(
        &mut cb, 0, 0, 0, false,
    )
    .unwrap();
    assert_eq!(
        crate::simulation::state::target_tagged_status_stacks(
            &exec.state.buffs,
            0,
            crate::simulation::reactions::REMORA_STATUS_TAG,
            0,
        )
        .unwrap(),
        1
    );
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].event_kind, "reaction_remora_applied");
}
