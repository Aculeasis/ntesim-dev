use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.relative_duration_us = 40_000_000;
    req.stochastic_crit = None;
    let mut wearer = req.actors[0].clone();
    wearer.slot = 0;
    wearer.character_id = "character:1052".into();
    wearer.weapon_id = "weapon:fork_Time".into();
    wearer.weapon_tier = 1;
    wearer.gear_procs = vec![GearProcConfig::MarchingBeyondTime {
        atk_bonus: Ratio {
            numerator: 4,
            denominator: 25,
        },
        state_crit_damage_bonus: Ratio {
            numerator: 6,
            denominator: 25,
        },
        crit_damage_per_stack: Ratio {
            numerator: 2,
            denominator: 25,
        },
        def_ignore: Ratio {
            numerator: 3,
            denominator: 25,
        },
        def_ignore_duration_us: 70_000_000,
    }];
    let mut teammate = wearer.clone();
    teammate.slot = 1;
    teammate.character_id = "character:1019".into();
    teammate.weapon_id = "weapon:none".into();
    teammate.gear_procs.clear();
    let mut second_wearer = teammate.clone();
    second_wearer.slot = 2;
    second_wearer.character_id = "character:1055".into();
    second_wearer.weapon_id = "weapon:fork_Time".into();
    second_wearer.gear_procs = wearer.gear_procs.clone();
    req.actors = vec![wearer, teammate, second_wearer];
    for actor in &mut req.actors {
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.crit_rate = Ratio::one();
        actor.static_stats.crit_damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        actor.static_stats.damage_bonus = Ratio::zero();
        actor.static_stats.elemental_bonus = Ratio::zero();
    }
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.active_actor_index = 0;
    req.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
        roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Test guaranteed hit criticals for Marching Beyond Time.".into(),
        eligible_effects: req
            .actors
            .iter()
            .map(|actor| {
                (
                    actor.character_id.clone(),
                    std::collections::BTreeSet::from(["*".into()]),
                )
            })
            .collect(),
        actor_sequences: req
            .actors
            .iter()
            .enumerate()
            .map(|(index, actor)| (actor.character_id.clone(), index as u64 + 1))
            .collect(),
    });
    req.actions.clear();
    let template = req.action_metadata[&0].clone();
    req.action_metadata.clear();
    req.action_metadata.insert(0, template);
    req
}

fn append_action(
    req: &mut GameScenarioRequest,
    actor: usize,
    time: u64,
    role: InputActionRole,
    delay: u64,
) {
    let index = req.actions.len();
    let ability = format!("caller-{index}");
    let packet = format!("caller-packet-{index}");
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: ability.clone(),
        array_index: 0,
        damage_effect_id: packet.clone(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let mut metadata = research_request().action_metadata[&0].clone();
    metadata.ability_id = ability;
    metadata.damage_effect_id = packet;
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec![
        match role {
            InputActionRole::RedirectSkill => "Ability.Skill",
            InputActionRole::Ultimate => "Ability.UltraSkill",
            InputActionRole::Qte => "Owner.QTE",
            _ => "Ability.Melee",
        }
        .into(),
    ]);
    req.action_metadata.insert(index, metadata);
}

fn append_hotori_action(req: &mut GameScenarioRequest, time: u64, burst: bool, delay: u64) {
    let index = req.actions.len();
    let (ability, packet) = if burst {
        ("GA_Jin_UltraSkill", "GE_Player_Jin_UltraSkill1_Damage")
    } else {
        ("GA_Jin_Skill", "GE_Player_Jin_Skill1_Damage")
    };
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: ability.into(),
        array_index: 0,
        damage_effect_id: packet.into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    req.action_metadata.remove(&index);
}

fn append_native_action(
    req: &mut GameScenarioRequest,
    actor: usize,
    time: u64,
    ability: &str,
    packet: &str,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: ability.into(),
        array_index: 0,
        damage_effect_id: packet.into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    req.action_metadata.remove(&index);
}

fn switch(req: &mut GameScenarioRequest, time: u64, target: usize) {
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: target,
        cooldown_us: 0,
    });
}

fn run_with_stacks(stacks: usize) -> serde_json::Value {
    let mut req = fixture();
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    for i in 0..stacks {
        append_action(
            &mut req,
            1,
            20 + i as u64 * 10,
            if i % 2 == 0 {
                InputActionRole::RedirectSkill
            } else {
                InputActionRole::Qte
            },
            0,
        );
        if stacks == 1 && i == 0 {
            let GameActionCommand::InvokeAbility { atk_rate, .. } = &mut req.actions[2] else {
                unreachable!()
            };
            *atk_rate = Ratio::zero();
        }
    }
    switch(&mut req, 100, 0);
    append_action(&mut req, 0, 110, InputActionRole::Ultimate, 0);
    run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn marching_beyond_time_stack_counts_have_independent_exact_crit_oracles() {
    for (count, expected) in [(0, 2018), (1, 2111), (3, 2297), (7, 2297)] {
        let report = run_with_stacks(count);
        let ult = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["actor_index"] == 0 && hit["command_index"] == 3 + count)
            .unwrap();
        assert_eq!(ult["rounded_damage"], expected);
        let buffs = crate::simulation::tests::support::final_buffs(&report);
        let def_buffs: Vec<_> = buffs
            .iter()
            .filter(|buff| buff["id"] == "marching_beyond_time.def_ignore")
            .collect();
        assert_eq!(def_buffs.len(), usize::from(count >= 3));
        if let Some(buff) = def_buffs.first() {
            assert_eq!(buff["owner_actor"], 0);
            assert_eq!(buff["source"], "weapon:fork_Time");
            assert_eq!(buff["duration"]["expires_at_us"], 70_000_110);
            assert_eq!(
                buff["stat_modifiers"]["def_ignore"],
                serde_json::json!("3/25")
            );
        }
        assert_eq!(
            report["resolved_model_assumptions"]
                [crate::weapons::marching_beyond_time::HYPOTHESIS_ID.as_str()],
            crate::weapons::marching_beyond_time::HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn marching_beyond_time_ultimate_bonus_stays_with_its_delayed_command_train() {
    let mut req = fixture();
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    for i in 0..3 {
        append_action(&mut req, 1, 20 + i * 10, InputActionRole::Qte, 0);
    }
    switch(&mut req, 100, 0);
    append_action(&mut req, 0, 110, InputActionRole::Ultimate, 1_000_000);
    append_action(&mut req, 0, 200, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let delayed = hits.iter().find(|hit| hit["command_index"] == 6).unwrap();
    let later = hits.iter().find(|hit| hit["command_index"] == 7).unwrap();
    assert_eq!(delayed["rounded_damage"], 2297);
    assert_eq!(later["rounded_damage"], 1740);
}

#[test]
fn marching_beyond_time_native_hotori_burst_train_keeps_crit_and_samples_def_ignore_at_expiry() {
    let mut req = fixture();
    req.relative_duration_us = 121_000_000;
    req.targets[0].defense = 300;
    req.targets[0].resistance = Ratio::zero();
    req.actors[0].talent_levels = Some([8, 8, 8]);
    // 11s of passive Timepiece (>60) funds Present Replay; the wearer's Skill
    // opens Wastelab and the rewind permits the Mint QTE trio that stacks it.
    append_hotori_action(&mut req, 11_000_000, false, 0);
    switch(&mut req, 11_500_000, 1);
    for time in [12_000_000, 12_500_000, 13_000_000] {
        append_native_action(
            &mut req,
            1,
            time,
            "GA_Mint019_QTE",
            "GE_Player_Mint_QTE1_Damage",
            0,
        );
    }
    switch(&mut req, 16_200_000, 0);
    // Passive recovery reaches exactly 120 Timepiece at 35.167475s.
    append_hotori_action(&mut req, 36_000_000, true, 0);
    // The70-world-second DEF-ignore buff pauses for the14.183333s first
    // mode and expires at120.183333 absolute; the second Burst can
    // only be admitted after the first mode ends and 120 Timepiece rebuilds.
    append_hotori_action(&mut req, 120_183_334, true, 0);
    let _prepared =
        crate::simulation::validation::validate_and_resolve(&req, &ExecutionLimits::default())
            .unwrap();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(report["rejected_action"].is_null());
    assert!(
        crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .all(|buff| buff["id"] != "marching_beyond_time.def_ignore")
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let first: Vec<_> = hits
        .iter()
        .filter(|hit| hit["command_index"] == 6)
        .collect();
    let second: Vec<_> = hits
        .iter()
        .filter(|hit| hit["command_index"] == 7)
        .collect();
    assert_eq!(first.len(), 10);
    assert_eq!(second.len(), 10);
    let expected_packets = [
        "GE_Player_Jin_UltraSkill1_Damage",
        "GE_Player_Jin_UltraSkill1_1_Damage",
        "GE_Player_Jin_UltraSkill1_2_Damage",
        "GE_Player_Jin_UltraSkill1_3_Damage",
        "GE_Player_Jin_UltraSkill1_4_Damage",
        "GE_Player_Jin_UltraSkill1_5_Damage",
        "GE_Player_Jin_UltraSkill2_Damage",
        "GE_Player_Jin_UltraSkill4_Damage",
        "GE_Player_Jin_UltraSkill3_Damage",
        "GE_Player_Jin_UltraSkill3_Damage",
    ];
    for ((first_hit, second_hit), packet) in first.iter().zip(&second).zip(expected_packets) {
        assert_eq!(first_hit["damage_effect_id"], packet);
        assert_eq!(second_hit["damage_effect_id"], packet);
        assert_eq!(first_hit["critical"], true);
        assert_eq!(second_hit["critical"], true);
        let first_crit = serde_json::from_value::<ExactFraction>(first_hit["crit_damage"].clone())
            .unwrap()
            .to_f64();
        let first_noncrit =
            serde_json::from_value::<ExactFraction>(first_hit["non_crit_damage"].clone())
                .unwrap()
                .to_f64();
        let second_crit =
            serde_json::from_value::<ExactFraction>(second_hit["crit_damage"].clone())
                .unwrap()
                .to_f64();
        let second_noncrit =
            serde_json::from_value::<ExactFraction>(second_hit["non_crit_damage"].clone())
                .unwrap()
                .to_f64();
        // 3 stacks add .24 state + .24 per-stack Crit DMG on the first Burst.
        assert!((first_crit / first_noncrit - 1.98).abs() < 1e-12);
        assert!((second_crit / second_noncrit - 1.5).abs() < 1e-12);
        // DEF factor 1080/1344 inside the buff vs 1080/1380 after expiry.
        assert!((second_noncrit / first_noncrit - 1344.0 / 1380.0).abs() < 1e-12);
    }
}

#[test]
fn marching_beyond_time_wastetime_crit_covers_the_katana_family_until_mode_end() {
    let mut req = fixture();
    req.relative_duration_us = 55_000_000;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    append_hotori_action(&mut req, 11_000_000, false, 0);
    switch(&mut req, 11_500_000, 1);
    for time in [12_000_000, 12_500_000, 13_000_000] {
        append_native_action(
            &mut req,
            1,
            time,
            "GA_Mint019_QTE",
            "GE_Player_Mint_QTE1_Damage",
            0,
        );
    }
    switch(&mut req, 16_200_000, 0);
    append_hotori_action(&mut req, 36_000_000, true, 0);
    for time in [
        40_604_238, 40_928_818, 41_411_492, 42_203_310, 43_018_384, 43_490_588, 43_815_169,
        44_297_843, 45_089_661, 45_904_735,
    ] {
        append_native_action(
            &mut req,
            0,
            time,
            "GA_Jin_Melee",
            "GE_Player_Jin_Melee1_Damage",
            100,
        );
    }
    append_native_action(
        &mut req,
        0,
        46_376_939,
        "GA_Jin_Melee",
        "GE_Player_Jin_Melee1_Damage",
        100,
    );
    append_native_action(
        &mut req,
        0,
        48_600_000,
        "GA_Jin_Melee",
        "GE_Player_Jin_Melee1_Damage",
        100,
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let crit_ratio = |hit: &serde_json::Value| {
        let crit = serde_json::from_value::<ExactFraction>(hit["crit_damage"].clone())
            .unwrap()
            .to_f64();
        let non_crit = serde_json::from_value::<ExactFraction>(hit["non_crit_damage"].clone())
            .unwrap()
            .to_f64();
        crit / non_crit
    };
    // The consumed three-stack value covers the whole Burst command family:
    // ten initial packets, both Katana trains and the breakthrough finisher.
    let family: Vec<_> = hits
        .iter()
        .filter(|hit| {
            hit["actor_index"] == 0 && (6..=17).contains(&hit["command_index"].as_u64().unwrap())
        })
        .collect();
    assert_eq!(family.len(), 10 + 26 + 1);
    for hit in &family {
        assert_eq!(hit["critical"], true);
        assert!(
            (crit_ratio(hit) - 1.98).abs() < 1e-12,
            "packet {} misses the consumed family bonus",
            hit["damage_effect_id"]
        );
    }
    // A Basic after the finisher ended the mode samples no family bonus.
    let next = hits.iter().find(|hit| hit["command_index"] == 18).unwrap();
    assert_eq!(next["damage_effect_id"], "GE_Player_Jin_Melee1_Damage");
    assert!((crit_ratio(next) - 1.5).abs() < 1e-12);
}

#[test]
fn marching_beyond_time_wearers_observe_team_actions_independently_while_off_field() {
    let mut req = fixture();
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    append_action(&mut req, 1, 20, InputActionRole::Qte, 0);
    switch(&mut req, 30, 2);
    append_action(&mut req, 2, 40, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 50, 1);
    append_action(&mut req, 1, 60, InputActionRole::Qte, 0);
    switch(&mut req, 100, 0);
    append_action(&mut req, 0, 110, InputActionRole::Ultimate, 0);
    switch(&mut req, 200, 2);
    append_action(&mut req, 2, 210, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let wearer0 = hits
        .iter()
        .find(|hit| hit["actor_index"] == 0 && hit["command_index"] == 8)
        .unwrap();
    let wearer2 = hits
        .iter()
        .find(|hit| hit["actor_index"] == 2 && hit["command_index"] == 10)
        .unwrap();
    assert_eq!(wearer0["rounded_damage"], 2297);
    assert_eq!(wearer2["rounded_damage"], 2111);
}

#[test]
fn marching_beyond_time_def_ignore_is_wearer_only_and_expires_at_the_exact_boundary() {
    let mut req = fixture();
    req.relative_duration_us = 80_000_000;
    req.targets[0].defense = 300;
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    for i in 0..3 {
        append_action(&mut req, 1, 20 + i * 10, InputActionRole::RedirectSkill, 0);
    }
    switch(&mut req, 100, 0);
    append_action(&mut req, 0, 110, InputActionRole::Ultimate, 0);
    append_action(&mut req, 0, 2_000_000, InputActionRole::Normal, 0);
    switch(&mut req, 2_000_010, 1);
    append_action(&mut req, 1, 2_000_020, InputActionRole::Normal, 0);
    switch(&mut req, 2_000_030, 0);
    append_action(&mut req, 0, 2_000_040, InputActionRole::Normal, 0);
    append_action(&mut req, 0, 72_000_000, InputActionRole::Normal, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits.iter().find(|hit| hit["command_index"] == 7).unwrap()["rounded_damage"],
        1398
    );
    assert_eq!(
        hits.iter().find(|hit| hit["command_index"] == 9).unwrap()["rounded_damage"],
        1174
    );
    assert_eq!(
        hits.iter().find(|hit| hit["command_index"] == 11).unwrap()["rounded_damage"],
        1398
    );
    assert_eq!(
        hits.iter().find(|hit| hit["command_index"] == 12).unwrap()["rounded_damage"],
        1362
    );
}

#[test]
fn marching_beyond_time_skill_restarts_wastelab_and_rejected_commands_do_not_add_stacks() {
    let mut req = fixture();
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    append_action(&mut req, 1, 20, InputActionRole::RedirectSkill, 0);
    append_action(&mut req, 1, 30, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 100, 0);
    append_action(&mut req, 0, 110, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 200, 1);
    append_action(&mut req, 1, 210, InputActionRole::Qte, 0);
    switch(&mut req, 300, 0);
    append_action(&mut req, 0, 310, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ult = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 0 && hit["command_index"] == 9)
        .unwrap();
    assert_eq!(ult["rounded_damage"], 2111);

    let mut rejected = fixture();
    append_action(&mut rejected, 0, 0, InputActionRole::RedirectSkill, 0);
    append_action(&mut rejected, 1, 10, InputActionRole::Qte, 0);
    let prepared =
        crate::simulation::validation::validate_and_resolve(&rejected, &ExecutionLimits::default())
            .unwrap();
    let mut state = crate::simulation::state::GameSimulatorState::new(&rejected);
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    let mut rng = None;
    let mut hits = Vec::new();
    let mut ledger = Vec::new();
    let mut ctx = crate::simulation::context::EventContext {
        request: &rejected,
        prepared_packets: &prepared,
        state: &mut state,
        queue: &mut queue,
        sequence: &mut sequence,
        trace: &mut trace,
        rng: &mut rng,
        hits: &mut hits,
        action_metadata_ledger: &mut ledger,
    };
    assert!(
        crate::simulation::admission::admit_ability(
            &mut ctx,
            0,
            &rejected.actions[0],
            prepared[0].clone(),
            crate::simulation::admission::AdmissionOrigin::Authored,
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(ctx.state.actors[0].equipment.marching_beyond_time.stacks, 0);
    assert!(
        crate::simulation::admission::admit_ability(
            &mut ctx,
            1,
            &rejected.actions[1],
            prepared[1].clone(),
            crate::simulation::admission::AdmissionOrigin::Authored,
        )
        .unwrap()
        .is_some()
    );
    let runtime = &ctx.state.actors[0].equipment.marching_beyond_time;
    assert!(runtime.wastelab_active);
    assert_eq!(runtime.stacks, 0);
}

#[test]
fn marching_beyond_time_normal_and_switch_inputs_do_not_add_stacks() {
    let mut req = fixture();
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    append_action(&mut req, 1, 20, InputActionRole::Normal, 0);
    switch(&mut req, 30, 0);
    append_action(&mut req, 0, 40, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ult = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 0 && hit["command_index"] == 4)
        .unwrap();
    assert_eq!(ult["rounded_damage"], 2018);
}

#[test]
fn marching_beyond_time_hand_derived_t5_crit_math() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // Frozen T5 curves: buff_Time_AtkUp 8/25, buff_Time_stateCritDamageUp 12/25,
    // buff_Time_CritDamageUp 4/25 per stack.
    // ATK_live = 1000 * (1 + 8/25) = 1000 * 33/25 = 1320 exact.
    assert_eq!(reduced(25 + 8, 25), (33, 25));
    assert_eq!(1000u128 * 33 / 25, 1320);
    // C at 3 stacks = 1 + 1/2 + 12/25 + 3 * 4/25 = 123/50 = 2.46 exact.
    assert_eq!(reduced(50 + 25 + 24 + 24, 50), (123, 50));
    // D = 1320 * 123/50 = 3247.2 -> nearest-integer 3247 (fraction .2 < .5).
    let exact_num = 1320u128 * 123;
    let exact_den = 50u128;
    assert_eq!(exact_num / exact_den, 3247);
    assert!(exact_num % exact_den * 2 < exact_den);
    // Misreading rejected: dropping the state bonus gives C = 99/50 -> 2613.6 -> 2614.
    assert_ne!(reduced(50 + 25 + 24, 50), (123, 50));

    // Real T5 scenario: 3 teammate casts -> 3 stacks -> wearer Ultimate crits 3247.
    let mut req = fixture();
    for actor in req.actors.iter_mut().take(1) {
        actor.weapon_tier = 5;
        if let GearProcConfig::MarchingBeyondTime {
            atk_bonus,
            state_crit_damage_bonus,
            crit_damage_per_stack,
            def_ignore,
            ..
        } = &mut actor.gear_procs[0]
        {
            *atk_bonus = Ratio {
                numerator: 8,
                denominator: 25,
            };
            *state_crit_damage_bonus = Ratio {
                numerator: 12,
                denominator: 25,
            };
            *crit_damage_per_stack = Ratio {
                numerator: 4,
                denominator: 25,
            };
            *def_ignore = Ratio {
                numerator: 6,
                denominator: 25,
            };
        }
    }
    append_action(&mut req, 0, 0, InputActionRole::RedirectSkill, 0);
    switch(&mut req, 10, 1);
    for i in 0..3 {
        append_action(&mut req, 1, 20 + i as u64 * 10, InputActionRole::Qte, 0);
    }
    switch(&mut req, 100, 0);
    append_action(&mut req, 0, 110, InputActionRole::Ultimate, 0);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let ult = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 0 && hit["command_index"] == 6)
        .unwrap();
    assert_eq!(ult["rounded_damage"], 3247);
}
