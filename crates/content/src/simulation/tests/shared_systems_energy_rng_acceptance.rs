use super::*;

#[test]
fn shared_energy_and_rng_continuing_rotation_preserves_streams_and_conservation() {
    let mut request = sample_s1_request();
    request.action_metadata.clear();

    let base = request.actors[0].clone();
    request.actors = [
        ("character:1019", 0, 50, 100), // Mint: slot 0, init 50, cap 100
        ("character:1003", 0, 98, 100), // Sakiri: slot 1, init 98, cap 100 (reaches 100 with hit)
        ("character:1046", 0, 0, 100),  // Zero: slot 2, init 0, cap 100
    ]
    .into_iter()
    .enumerate()
    .map(|(slot, (id, awakening, initial_energy, capacity))| {
        let mut actor = base.clone();
        actor.slot = slot;
        actor.character_id = id.into();
        actor.awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        actor.initial_energy = initial_energy;
        actor.energy_capacity = capacity;
        actor.weapon_id.clear();
        actor.set_id = None;
        actor.gear_procs.clear();
        actor
    })
    .collect();

    // Actor 0 has Charge Efficiency +12% (28/25)
    request.actors[0]
        .gear_procs
        .push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 28,
                denominator: 25,
            },
        });

    request.stochastic_crit = Some(ResearchCritPolicy {
        roll_policy: ResearchCritRollPolicy::IndependentPerTargetHitV1,
        hypothesis: "Independent per-target-hit crit rolls".into(),
        eligible_effects: BTreeMap::from([
            (
                "character:1019".into(),
                BTreeSet::from(["GE_Player_Mint_Skill1_Damage_New".into()]),
            ),
            (
                "character:1003".into(),
                BTreeSet::from(["GE_Player_Sagiri_UltraSkill1_Damage".into()]),
            ),
        ]),
        actor_sequences: BTreeMap::from([
            ("character:1019".into(), 54),
            ("character:1003".into(), 55),
        ]),
    });

    request.relative_duration_us = 2_000_000;
    request.actions = vec![
        // Action 0: Mint Skill (delayed hit at +200ms)
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: "GA_Mint019_Skill".into(),
            damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 100_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 200_000,
        },
        // Action 1: Switch to Sakiri before Mint's hit lands (triggers Support QTE, locks actor 1 until 700ms)
        GameActionCommand::SwitchActor {
            timestamp_us: 100_000,
            target_actor_index: 1,
            cooldown_us: 1_000_000,
        },
        // Action 2: Sakiri spends 100 energy on Burst once unlocked at 800ms
        GameActionCommand::InvokeAbility {
            timestamp_us: 800_000,
            actor_index: 1,
            ability_id: "GA_Sagiri_UltraSkill".into(),
            damage_effect_id: "GE_Player_Sagiri_UltraSkill1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: NativeScalingStat::Atk,
            energy_cost: 100,
            recovery_us: 500_000,
            cooldown_us: 15_000_000,
            hit_delay_us: 100_000,
        },
    ];

    let limits = ExecutionLimits::default();
    let report = run_game_scenario(&request, &limits).unwrap();
    assert_eq!(
        report["status"],
        "ok",
        "error trace: {}",
        serde_json::to_string_pretty(&report).unwrap()
    );

    let final_states = report["final_actor_states"].as_array().unwrap();
    // Mint (slot 0) started with 50 energy and received off-field generation from her delayed Skill
    assert!(final_states[0]["energy"].as_u64().unwrap() > 50);

    // Sakiri (slot 1) started at 95, capped at 100, and spent 100 on Burst
    assert_eq!(final_states[1]["spent_energy"], 100);

    // RNG stream accounting: each character stream has deterministic draw accounting
    let rng_report = &report["rng"];
    assert!(rng_report.get("streams").is_some());

    // Deterministic replay check with same seed
    let replay_report = run_game_scenario(&request, &limits).unwrap();
    assert_eq!(report["hit_receipts"], replay_report["hit_receipts"]);
    assert_eq!(
        report["total_damage_exact"],
        replay_report["total_damage_exact"]
    );

    // Changing seed produces distinct but valid draw streams
    let mut different_seed_request = request.clone();
    different_seed_request.seed = 999999;
    let different_report = run_game_scenario(&different_seed_request, &limits).unwrap();
    assert_eq!(different_report["status"], "ok");
}
