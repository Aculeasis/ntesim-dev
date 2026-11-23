use super::*;
use crate::simulation::{GameActionCommand, GameActorDamageEvent};
use serde_json::{Value, json};

fn request(enhanced: bool) -> GameScenarioRequest {
    let source = "options duration=30 crit=fixed;
        hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        chaos char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0;
        target hp=100000000 def=1080 resist=0; active hotori;";
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    if !enhanced {
        req.actors[2].character_level = 20;
        req.actors[2].character_max_level = Some(20);
    }
    for actor in &mut req.actors {
        actor.initial_energy = 0;
    }
    req.actors[1].static_stats.atk = crate::formula::Ratio {
        numerator: 2000,
        denominator: 1,
    };
    req.actors[1].static_stats.crit_rate = crate::formula::Ratio::zero();
    req.actors[1].static_stats.damage_bonus = crate::formula::Ratio::zero();
    req.actors[1].static_stats.elemental_bonus = crate::formula::Ratio::zero();
    req.actors[1].static_stats.elemental_bonuses.clear();
    req.actions = vec![swap(0, 1)];
    req
}
fn swap(timestamp_us: u64, target_actor_index: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index,
        cooldown_us: 0,
    }
}
fn ability(time: u64, skill: bool) -> GameActionCommand {
    serde_json::from_value(json!({
        "kind":"invoke_ability","timestamp_us":time,"actor_index":1,
        "ability_id":if skill {"GA_Chaos071_Skill"} else {"GA_Chaos071_Melee"},
        "damage_effect_id":if skill {"GE_Player_Chaos_Skill1_Damage"} else {"GE_Player_Chaos_Melee1_Damage"},
        "array_index":0,"atk_rate":"1/1","scaling_stat":"atk",
        "energy_cost":0,"recovery_us":0,"cooldown_us":0,"hit_delay_us":0
    })).unwrap()
}
fn hits(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == crate::characters::chaos::remora::PACKET)
        .collect()
}
fn run(req: &GameScenarioRequest, removal: Option<u64>) -> Value {
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(req, &limits).unwrap();
    let (mut execution, assumptions) = PreparedExecution::new(req, req.seed, &[], &[]).unwrap();
    if let Some(time) = removal {
        execution.advance_to(req, &limits, &packets, time).unwrap();
        crate::simulation::reactions::remove_remora(&mut execution.event_context(req, &packets), 0)
            .unwrap();
        // A duplicate cue/removal does not produce another owner packet.
        crate::simulation::reactions::remove_remora(&mut execution.event_context(req, &packets), 0)
            .unwrap();
    }
    execution
        .process_events(req, &limits, &packets, &[], &[])
        .unwrap();
    let state = execution.state.actors[1].character.chaos_mut().unwrap();
    assert!(state.remora_bonuses.is_empty());
    assert!(state.remoras.is_empty());
    serde_json::to_value(execution.into_report(req, req.seed, assumptions).unwrap()).unwrap()
}

#[test]
fn chaos_remora_end_preserves_native_eight_atk_and_separate_exact_elapsed_factor() {
    for (enhanced, removal, time, damage) in [
        (false, None, 5_000_000, 8000),
        (true, Some(4_000_000), 4_000_000, 8000),
        (true, Some(10_000_000), 10_000_000, 26000),
        (true, None, 12_000_000, 32000),
    ] {
        let req = request(enhanced);
        let report = run(&req, removal);
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let packets = hits(&report);
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0]["time_us"], time);
        assert_eq!(packets[0]["target_index"], 0);
        assert_eq!(packets[0]["scaling_rate"], json!("8/1"));
        assert_eq!(
            packets[0]["non_crit_damage"],
            json!(format!("{}/{}", damage, 1))
        );
        let packet_meta = crate::simulation::tests::support::hit_metadata(&report, packets[0]);
        assert_eq!(packet_meta["action"]["input_role"], "other");
        assert_eq!(
            packet_meta["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
        );
        assert_eq!(
            packet_meta["binding"]["packet_ga_name"],
            "GA_Chaos071_UltraSkill"
        );
        for (index, actor) in report["final_actor_states"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            // Qualified entry's 6.4 ChargeAdd is credited once across both
            // dummies; the later Remora-end packet adds nothing.
            assert_eq!(actor["energy"], if index == 1 { 6 } else { 3 });
            assert_eq!(
                actor["charge_remainder"],
                if index == 1 {
                    json!("2/5")
                } else {
                    json!("21/25")
                }
            );
        }
        let chaos = &report["final_actor_states"][1];
        assert!(chaos["cycle"].is_null()); // No Cycle grant creates an otherwise untouched ledger.
        assert_eq!(
            chaos["fractional_resources"]["chaos.crime"]["current"],
            json!("0/1")
        );
        assert_eq!(
            report["autonomous_events"][0]["trigger"],
            json!({"kind":"field_entry","command_index":0})
        );
    }
}

#[test]
fn chaos_remora_refresh_replaces_elapsed_without_explosion_and_preserves_off_field_owner() {
    let mut req = request(true);
    req.actors[1].initial_cycle = 100;
    req.actions.push(swap(4_000_000, 0));
    let report = run(&req, None);
    let packets = hits(&report);
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0]["time_us"], 16_000_000);
    assert_eq!(packets[0]["non_crit_damage"], json!("32000/1"));
    assert_eq!(packets[0]["actor_index"], 1);
    assert_eq!(
        report["autonomous_events"][0]["trigger"]["command_index"],
        1
    );
    let mut req = request(false);
    req.actions.push(swap(2_000_000, 0)); // After native QTE recovery; no Cycle to refresh Remora.
    assert_eq!(hits(&run(&req, None))[0]["time_us"], 5_000_000);
}

#[test]
fn chaos_remora_owner_death_cancels_but_reaction_applier_death_preserves_explosion() {
    for victim in [0, 1] {
        let mut req = request(false);
        req.incoming_damage_events.push(GameActorDamageEvent {
            timestamp_us: 2_000_000,
            actor_index: victim,
            delivered_damage: req.actors[victim].initial_hp,
        });
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        assert_eq!(hits(&run(&req, None)).len(), usize::from(victim == 0));
    }
}

#[test]
fn chaos_remora_uses_live_warrant_b4_bonus_and_misses_grant_no_resources() {
    let mut req = request(false);
    req.actions.push(ability(2_000_000, true));
    let report = run(&req, None);
    assert_eq!(hits(&report)[0]["non_crit_damage"], json!("10400/1"));
    assert_eq!(
        report["final_actor_states"][1]["fractional_resources"]["chaos.crime"]["current"],
        json!("100/1")
    );
    let mut req = request(false);
    for target in &mut req.targets {
        target.resistance = crate::formula::Ratio::one();
    }
    let report = run(&req, None);
    assert_eq!(hits(&report)[0]["applied_hp_loss"], 0);
    assert_eq!(report["final_actor_states"][1]["energy"], 0);
}

#[test]
fn chaos_remora_target_death_retires_callback_before_short_horizon_and_b2_is_required() {
    let mut req = request(false);
    req.relative_duration_us = 3_000_000;
    req.targets[0].hp_current = 1;
    req.targets[0].hp_max = 1;
    req.actions.push(ability(2_000_000, false));
    let report = run(&req, None);
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(hits(&report).is_empty());
    let mut req = request(false);
    req.actors[1].character_level = 20;
    req.actors[1].character_max_level = Some(20);
    req.relative_duration_us = 2_000_000;
    assert_eq!(run(&req, None)["execution_completed"], true);
    let mut command = ability(0, true);
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut command
    {
        *damage_effect_id = crate::characters::chaos::remora::PACKET.into();
    }
    req.actions = vec![command];
    assert!(crate::simulation::run_game_scenario(&req, &ExecutionLimits::default()).is_err());
}

#[test]
fn chaos_remora_samples_live_atk_after_its_real_listener_was_armed() {
    use ntesim_core::buffs::dispatch::{
        BuffDefinition, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
    };
    let mut req = request(true);
    req.actors[1].static_stats.base_atk = 1000;
    let limits = ExecutionLimits::default();
    let packets = crate::simulation::validation::validate_and_resolve(&req, &limits).unwrap();
    let (mut execution, assumptions) = PreparedExecution::new(&req, req.seed, &[], &[]).unwrap();
    let event = execution.queue.pop().unwrap();
    assert!(
        execution
            .process_event(event.data, &req, &packets, &[], &[])
            .unwrap()
    );
    assert!(
        !execution.state.actors[1]
            .character
            .chaos_mut()
            .unwrap()
            .remoras
            .is_empty()
    );
    execution
        .state
        .buffs
        .apply_buff(
            BuffDefinition {
                source: "test.live_atk".into(),
                id: "test.live_atk".into(),
                recipient: BuffRecipient::SelfActor,
                stacking_scope: BuffStackingScope::Shared,
                source_lifetime: Default::default(),
                duration_us: Some(20_000_000),
                max_stacks: 1,
                refresh_policy: RefreshPolicy::Reset,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: [(
                    "atk_percent".into(),
                    crate::formula::Ratio {
                        numerator: 1,
                        denominator: 2,
                    },
                )]
                .into(),
                semantic_tags: Default::default(),
            },
            1,
            0,
        )
        .unwrap();
    execution
        .process_events(&req, &limits, &packets, &[], &[])
        .unwrap();
    let report =
        serde_json::to_value(execution.into_report(&req, req.seed, assumptions).unwrap()).unwrap();
    // Live2500 ATK = compiled2000 + base1000*.5;32*2500*.5 =40000.
    assert_eq!(hits(&report)[0]["non_crit_damage"], json!("40000/1"));
}
