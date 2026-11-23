use super::*;
use crate::callbacks::ContentEvent;
use crate::simulation::state::GameSimulatorState;

fn count(report: &serde_json::Value, packet: &str) -> usize {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == packet)
        .count()
}

fn chains(req: &mut GameScenarioRequest, total: usize) {
    let mut time = 0;
    for _ in 0..total {
        for recovery in [923724, 840277, 1218687, 1706341, 820279] {
            req.actions.push(normal(time));
            time += recovery;
        }
    }
}

#[test]
fn word_storage_replaces_oldest_across_types_and_respects_selected_a3() {
    for (effects, expected) in [(vec![], 3), (vec![3], 4)] {
        let mut req = phase2_request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(effects).unwrap();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actions.clear();
        chains(&mut req, 3);
        req.actions.push(burst(17_000_000));
        let report = run(&req);
        assert_eq!(count(&report, "GE_Player_Cang_WorldC_Damage"), expected);
        assert_eq!(
            count(&report, "GE_Player_Cang_WorldC_1_Damage"),
            expected * 2
        );
        // Five creations cost HP, even when an older mark is discarded.
        // 10000 ->9700 ->9409 ->9127 ->8853 ->8587, then 500 per retained Bless.
        assert_eq!(
            report["final_actor_states"][0]["hp"],
            (8587 + 500 * expected).min(10000)
        );
    }
}

#[test]
fn word_expiry_is_half_open_and_is_not_a_free_heal() {
    for (burst_time, expected) in [(17_166_254, 1), (17_166_255, 0)] {
        let mut req = phase2_request();
        req.actors[0].initial_energy = 120;
        req.actors[0].energy_capacity = 120;
        req.actions = vec![
            normal(0),
            normal(1_000_000),
            normal(2_000_000),
            burst(burst_time),
        ];
        let report = run(&req);
        assert_eq!(count(&report, "GE_Player_Cang_WorldC_Damage"), expected);
        assert_eq!(
            report["final_actor_states"][0]["hp"],
            if expected == 0 { 9700 } else { 10000 }
        );
    }
}

#[test]
fn dead_mark_does_not_retarget_words_or_cancel_owner_healing_and_other_domain_targets() {
    let mut req = phase2_request();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.targets[0].hp_max = 1000;
    req.targets[0].hp_current = 1000;
    req.actions = vec![
        normal(0),
        normal(1_000_000),
        normal(2_000_000),
        burst(3_300_000),
    ];
    let report = run(&req);
    assert_eq!(count(&report, "GE_Player_Cang_WorldC_Damage"), 0);
    assert_eq!(count(&report, "GE_Player_Cang_WorldC_1_Damage"), 0);
    assert_eq!(count(&report, "GE_Player_Cang_UltraSkill_Damage"), 6);
    assert_eq!(report["final_actor_states"][0]["hp"], 10000);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
            .all(|h| h["target_index"] == 1)
    );
}

fn callback(
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    time: u64,
    event: ContentEvent,
) {
    state.advance_unpaused_test_clock(time);
    crate::callbacks::apply(
        event,
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue: &mut Default::default(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
        },
    )
    .unwrap();
}

#[test]
fn selected_a1_callbacks_cap_credits_and_reduce_only_the_shared_skill_clock() {
    let mut req = phase2_request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected(vec![1]).unwrap();
    let mut state = GameSimulatorState::new(&req);
    state.actors[0]
        .ability_cooldowns
        .insert("GA_Cang_WorldPas_Fadia".into(), 30_000_000);
    for _ in 0..6 {
        callback(
            &req,
            &mut state,
            0,
            ContentEvent::BaicangWordObjurgateA1 { actor_index: 0 },
        );
        callback(
            &req,
            &mut state,
            0,
            ContentEvent::BaicangWordSilenceA1 { actor_index: 0 },
        );
    }
    let words = state.actors[0].character.baicang_mut().unwrap();
    assert_eq!(
        (words.a1_silence_stacks, words.a1_objurgate_credits),
        (4, 4)
    );
    state.actors[0]
        .ability_cooldowns
        .insert("GA_Cang_Skill".into(), 3_000_000);
    callback(
        &req,
        &mut state,
        2_000_000,
        ContentEvent::BaicangWordObjurgateA1 { actor_index: 0 },
    );
    assert_eq!(
        state.actors[0]
            .ability_cooldowns
            .get("GA_Cang_Skill")
            .copied()
            .unwrap(),
        2_000_000
    );
    assert_eq!(
        state.actors[0]
            .ability_cooldowns
            .get("GA_Cang_WorldPas_Fadia")
            .copied()
            .unwrap(),
        30_000_000
    );
    req.actors[0].awakening = Default::default();
    let mut plain = GameSimulatorState::new(&req);
    callback(
        &req,
        &mut plain,
        0,
        ContentEvent::BaicangWordSilenceA1 { actor_index: 0 },
    );
    callback(
        &req,
        &mut plain,
        0,
        ContentEvent::BaicangWordObjurgateA1 { actor_index: 0 },
    );
    let words = plain.actors[0].character.baicang_mut().unwrap();
    assert_eq!(
        (words.a1_silence_stacks, words.a1_objurgate_credits),
        (0, 0)
    );
}

#[test]
fn word_healing_uses_live_max_hp_and_outgoing_and_incoming_zones_once() {
    let mut req = phase2_request();
    req.actors[0].initial_hp = 1000;
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    let id = crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    let mut state = GameSimulatorState::new(&req);
    crate::simulation::state::apply_timed_buff(
        &mut state.buffs,
        crate::simulation::state::TimedBuffDeclaration::reset(
            "test.word.incoming",
            "test",
            ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
            2_000_000,
            ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([(
                "incoming_healing_bonus".into(),
                ExactFraction::new(1, 5).unwrap(),
            )]),
        ),
        0,
        0,
    )
    .unwrap();
    callback(
        &req,
        &mut state,
        1_000_000,
        ContentEvent::BaicangWordHeal {
            actor_index: 0,
            percent: 5,
        },
    );
    assert_eq!(state.actors[0].hp, 1900); // 10000*.05*1.5*1.2 =900
    callback(
        &req,
        &mut state,
        2_000_000,
        ContentEvent::BaicangWordHeal {
            actor_index: 0,
            percent: 5,
        },
    );
    assert_eq!(state.actors[0].hp, 2650); // incoming zone expired exactly
    state.actors[0].hp = 0;
    callback(
        &req,
        &mut state,
        3_000_000,
        ContentEvent::BaicangWordHeal {
            actor_index: 0,
            percent: 10,
        },
    );
    assert_eq!(state.actors[0].hp, 0);
}

#[test]
fn defeat_clears_owner_ledgers_and_invalidates_pending_field_callbacks() {
    let req = phase2_request();
    let mut state = GameSimulatorState::new(&req);
    let words = state.actors[0].character.baicang_mut().unwrap();
    words.normal_word_counter = 2;
    words.a1_silence_stacks = 4;
    words.a1_objurgate_credits = 3;
    words.domain_generation = 1;
    words.domain_until_us = 20_000_000;
    words.fatal_hit_debt_total = 1200;
    words.a2_stack_expiries.push(10_000_000);
    words.riposte_ready = true;
    state.actors[0].hp = 0;
    crate::registry::on_actor_defeated(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut Default::default(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
        },
        0,
    )
    .unwrap();
    callback(
        &req,
        &mut state,
        1_000_000,
        ContentEvent::BaicangWordSilenceA1 { actor_index: 0 },
    );
    let words = state.actors[0].character.baicang_mut().unwrap();
    assert_eq!(
        (
            words.normal_word_counter,
            words.a1_silence_stacks,
            words.a1_objurgate_credits
        ),
        (0, 0, 0)
    );
    assert!(words.a2_stack_expiries.is_empty());
    assert!(!words.riposte_ready);
    assert_eq!(words.domain_until_us, 0);
    assert_eq!(words.fatal_hit_debt_total, 0);
}
