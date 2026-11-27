use super::*;
use crate::simulation::PlayerInput;
use serde_json::json;

fn reaction_actor(
    req: &mut GameScenarioRequest,
    slot: usize,
    character_id: &str,
    cycle: u64,
    lvl: u32,
) {
    let mut base = req.actors[0].clone();
    base.slot = slot;
    base.character_id = character_id.into();
    base.character_level = lvl;
    base.character_max_level = Some(lvl);
    base.awakening = ntesim_data::AwakeningSelection::prefix(0);
    base.talent_levels = Some([1, 1, 1]);
    base.purchased_qte_level = Some(1);
    base.initial_energy = 0;
    base.energy_capacity = 100;
    base.initial_cycle = cycle;
    base.initial_resources = Default::default();
    base.static_stats.base_atk = 1000;
    base.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    base.static_stats.damage_bonus = Ratio::zero();
    base.static_stats.elemental_bonus = Ratio::zero();
    base.static_stats.elemental_bonuses.clear();
    base.static_stats.crit_rate = Ratio::zero();
    base.weapon_id.clear();
    base.set_id = None;
    base.gear_procs.clear();
    while req.actors.len() <= slot {
        req.actors.push(base.clone());
    }
    req.actors[slot] = base;
}

fn switch(timestamp_us: u64, target: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index: target,
        cooldown_us: 0,
    }
}

fn dsl_run(script: &str) -> crate::simulation::SourceScenarioReport {
    crate::simulation::PreparedSourceScenario::new(script, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap()
}

#[test]
fn shinku_defiant_spirit_ignores_charge_efficiency_off_field_and_charge_reaction() {
    // Off-field ally attacks grant 0 Defiant Spirit; Shinku's own hits (Normal = 18,
    // Skill = 5, Watch = 3.75, QTE = 5.3) are unscaled by ce=1 (CE = 2.0).
    let result = dsl_run(
        "options duration=25 crit=fixed;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        shinku add stats ce=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_cycle=0;
        target hp=100000000 def=0 resist=0;
        active mint;
        mint attack:5;
        if .shinku.energy!=0 {let assertion_failure=1/0;}
        shinku swap;
        if .shinku.energy!=0 {let assertion_failure=1/0;}
        shinku attack:5;
        if .shinku.energy!=18 {let assertion_failure=1/0;}
        shinku skill;
        if .shinku.energy!=23 {let assertion_failure=1/0;}
        wait(2.5);
        while .shinku.resources.watch < 1 { wait(0.1); }
        if .shinku.resources.watch!=1 {let assertion_failure=1/0;}
        shinku attack;
        // Normal(18) + Skill(5) + Watch(3.75) + Melee1_1(0.76 * 45/44) = 27 + 29/55.
        if .shinku.energy!=27 {let assertion_failure=1/0;}",
    );
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(
        serde_json::to_value(&result.report).unwrap()["final_actor_states"][0]["charge_remainder"],
        "29/55"
    );
    // Verify QTE grants exact 5.3 = 53/10 Defiant Spirit, even with ce=1.
    let mut qte_req = request();
    qte_req.actors[0].initial_energy = 0;
    qte_req.actors[0]
        .gear_procs
        .push(crate::simulation::GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 2,
                denominator: 1,
            },
        });
    qte_req.actions = vec![make_action(
        0,
        0,
        "GA_Shinku_QTE",
        "GE_Player_Shinku_QTE_Damage",
        0,
        0,
    )];
    let qte_report = run_game_scenario(&qte_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(qte_report["execution_completed"], true, "{qte_report}");
    assert_eq!(qte_report["final_actor_states"][0]["energy"], 5);
    assert_eq!(
        qte_report["final_actor_states"][0]["charge_remainder"],
        "3/10"
    );
    // Burst must reject when Defiant Spirit < 60.
    let reject = dsl_run(
        "options duration=5 crit=fixed;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=59;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;",
    );
    assert!(!reject.report.execution_completed);
    let diag = reject.diagnostic.expect("burst admission rejection");
    assert!(
        diag.message.contains("energy") || diag.message.contains("60"),
        "unexpected rejection: {diag:?}"
    );
}

#[test]
fn shinku_tidal_assault_scarlet_descent_cd_and_verdict_end_of_stance() {
    let result = dsl_run(
        "options duration=25 crit=fixed;
        shinku char lvl=80/80 awakening=5 talent=8,8,8 qte=8 start_energy=60;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;
        if !.shinku.surging_crimson.active || .shinku.energy!=0 {let assertion_failure=1/0;}
        // 5 stages of Tidal Assault fill Fading Reason cleanly to 12.
        shinku attack:5;
        if .shinku.resources.fading_reason!=12 {let assertion_failure=1/0;}
        shinku skill;
        if .shinku.resources.fading_reason!=0 || .shinku.resources.crimson_judgment!=1 {let assertion_failure=1/0;}
        // Wait inside Surging Crimson until <= 4s remain, then consume Crimson Judgment & cast Verdict.
        if .shinku.mods.surging_crimson > 4 { wait(.shinku.mods.surging_crimson - 4); }
        shinku burst;
        if .shinku.resources.crimson_judgment!=0 || !.shinku.surging_crimson.active {let assertion_failure=1/0;}
        shinku burst;
        // Immediately after Verdict settles, Surging Crimson and A5 DEF are cleared.
        if .shinku.surging_crimson.active || .shinku.mods.surging_crimson!=0 {let assertion_failure=1/0;}",
    );
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    let report = serde_json::to_value(&result.report).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let verdict_flurry = hits
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_UltraSkill_Rage_Damage")
        .expect("Verdict flurry hit");
    // Lv9 UltraSkill_Rage (370/1000) * 636 ATK * (1 + 0.30 Surging Crimson + 0.30 R3 + 0.50 Passive 2 = 21/10)
    // * (12/11 A2 -10% Cosmos RES) = 741258/1375!
    assert_eq!(verdict_flurry["scaling_rate"], json!("37/100"));
    assert_eq!(verdict_flurry["non_crit_damage"], json!("741258/1375"));

    // Verify Scarlet Descent rejects within 3.0s cooldown.
    let cd_reject = dsl_run(
        "options duration=10 crit=fixed;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=60;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;
        shinku attack:5;
        shinku skill;
        shinku skill;",
    );
    assert!(!cd_reject.report.execution_completed);
    let diag = cd_reject
        .diagnostic
        .expect("Scarlet Descent cooldown rejection");
    assert!(diag.message.contains("cooldown"), "{diag:?}");
}

#[test]
fn shinku_passive1_lone_stride_back_charge_aoe_and_atk_stacks() {
    // Two dummies, Skia -> Mint -> Shinku triggers Charge twice >= 1.0s apart.
    // BT >= 2 Shinku: BackChargeMul = 1.0 grants 10 energy to living off-field teammates,
    // emits ReactionAOE across both dummies once per 1.0s ICD, and applies +5% ATK stacks (max 10).
    let mut req = crate::simulation::tests::research_request();
    req.targets[0].resistance = Ratio::zero();
    req.action_metadata.clear();
    let mut target2 = req.targets[0].clone();
    target2.target_index = 1;
    req.targets.push(target2);
    reaction_actor(&mut req, 0, "character:1008", 100, 80); // Skia (Lakshana)
    reaction_actor(&mut req, 1, "character:1019", 0, 80); // Mint (Nature, no QTE into Shinku)
    reaction_actor(&mut req, 2, "character:1076", 100, 80); // Shinku (Cosmos, BT 5 >= 2)
    req.actors[2].talent_levels = Some([8, 1, 1]); // Normal Lv8 -> ReactionAOE 685.5%
    req.active_actor_index = 0;
    // 0->2 triggers Remora (+ Shinku QTE 5.3, recovery 0.6s); 2->1 triggers Blossom; 1->2 puts Shinku on-field before Charge lands.
    req.actions = vec![switch(0, 2), switch(1_000_000, 1), switch(1_800_000, 2)];
    req.relative_duration_us = 10_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let aoe_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Shinku_ReactionAOE_Damage")
        .collect();
    // 2 living targets hit cleanly on the first Charge while Shinku is active.
    assert_eq!(aoe_hits.len(), 2);
    assert_eq!(aoe_hits[0]["target_index"], 0);
    assert_eq!(aoe_hits[1]["target_index"], 1);
    // Lv8 ReactionAOE = 6.855 ATK, +5% ATK stack (1 stack active -> 1050 ATK),
    // target defense=1080 -> 1/2 DEF, resistance=0 -> 1050 * 6.855 * 1/2 = 28791/8!
    assert_eq!(aoe_hits[0]["non_crit_damage"], json!("28791/8"));

    // Verify BackChargeMul = 1.0 granted off-field Charge energy to Skia and Mint,
    // and ReactionAOE (ChargeAdd = 10.0, shared energy group) distributed 0.6x = 6 once (not per target).
    let back_charge_count = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "shinku_passive1_back_charge")
        .count();
    assert_eq!(back_charge_count, 2);
    // Skia & Mint each got 3 (Shinku QTE 0.6*5) + 10 (BackChargeMul) + 6 (ReactionAOE 0.6*10) = 19 energy!
    assert_eq!(report["final_actor_states"][0]["energy"], 19);
    assert_eq!(report["final_actor_states"][1]["energy"], 19);
    // Shinku's Defiant Spirit received 0 from Charge and 0 from ReactionAOE (only 5.3 from initial QTE).
    assert_eq!(report["final_actor_states"][2]["energy"], 5);
    assert_eq!(report["final_actor_states"][2]["charge_remainder"], "3/10");

    // Breakthrough < 2 (level 1/20): neither BackChargeMul nor ReactionAOE/ATK buff triggers.
    let mut low_bt = req.clone();
    reaction_actor(&mut low_bt, 2, "character:1076", 100, 1);
    low_bt.actors[2].character_max_level = Some(20);
    let low_report = run_game_scenario(&low_bt, &ExecutionLimits::default()).unwrap();
    assert_eq!(low_report["execution_completed"], true);
    let low_aoe = low_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Shinku_ReactionAOE_Damage")
        .count();
    assert_eq!(low_aoe, 0);
    assert_eq!(low_report["final_actor_states"][0]["energy"], 3);
    assert_eq!(low_report["final_actor_states"][1]["energy"], 3);
}

#[test]
fn shinku_passive2_wrathscale_applies_only_bt4_and_non_boss() {
    for (lvl, is_boss, expected_num, expected_den) in [
        // Lv80/80 (BT 5 >= 4), non-boss: +30% Surging Crimson + +50% Wrathscale = 9/5 -> 185 * 9 / 10
        (80, false, 333, 2),
        // Lv80/80 (BT 5 >= 4), boss: +30% Surging Crimson only -> 185 * 13 / 20 = 481/4
        (80, true, 481, 4),
        // Lv40/40 (BT < 4), non-boss: +30% Surging Crimson only; level DEF = 140/(140+180) = 7/16 -> 3367/32
        (40, false, 3367, 32),
    ] {
        let mut req = request();
        req.actors[0].character_level = lvl;
        req.actors[0].character_max_level = Some(lvl);
        req.actors[0].purchased_qte_level = Some(1);
        req.actors[0].initial_energy = 60;
        req.targets[0].is_boss = is_boss;
        req.actions = vec![make_action(
            0,
            0,
            "GA_Shinku_UltraSkill",
            "GE_Player_Shinku_UltraSkill1_Damage",
            0,
            20_000_000,
        )];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            fraction_json(expected_num, expected_den),
            "lvl={lvl} is_boss={is_boss}"
        );
    }
}

#[test]
fn shinku_passive3_menacing_gaze_critical_dodge_a3_and_a4_watch_ex() {
    // 1. A0: 2s idle -> Menacing Gaze active -> 3 ticks + 1 Critical Dodge = 4 stacks.
    // Switching off-field clears A0 stacks.
    let mut a0_req = request();
    a0_req.actors[0].initial_energy = 0;
    a0_req.combat_stamina = Some(crate::simulation::CombatStaminaProfile {
        capacity: Ratio {
            numerator: 200,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 200,
            denominator: 1,
        },
        use_efficiency: Ratio::one(),
        recovery_rate: Ratio {
            numerator: 10,
            denominator: 1,
        },
    });
    a0_req.model_assumptions.insert(
        crate::simulation::inputs::CRITICAL_DODGE_ID.into(),
        crate::simulation::inputs::CRITICAL_DODGE_ID
            .description()
            .into(),
    );
    let mut ally = a0_req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.initial_resources = Default::default();
    a0_req.actors.push(ally);
    a0_req.actions = vec![
        // WatchStart at 2.0s; ticks at 3.0, 4.0, 5.0 -> 3 stacks; Dash(crit) at 5.2s -> 4 stacks.
        GameActionCommand::PlayerInput {
            timestamp_us: 5_200_000,
            actor_index: 0,
            input: PlayerInput::Dash,
            critical_success: true,
        },
        make_action(
            5_800_000,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let a0_report = run_game_scenario(&a0_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(a0_report["execution_completed"], true, "{a0_report}");
    let watch = a0_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_Watch_Damage")
        .expect("Watch hit present");
    // 4 stacks * 1.0 Lv1 ATK * 1000 * 1/2 DEF = 2000/1.
    assert_eq!(watch["non_crit_damage"], json!("2000/1"));
    assert_eq!(a0_report["final_actor_states"][0]["cycle"]["current"], 7); // 1.266 + 6.250 = 7.516
    assert_eq!(a0_report["final_actor_states"][0]["energy"], 4);
    assert_eq!(
        a0_report["final_actor_states"][0]["charge_remainder"],
        "29/55" // 0.76 * 45/44 + 3.75 = 171/220 + 15/4 = 996/220 = 249/55 = 4 + 29/55
    );

    // 2. A3 retains stacks & ticks off-field; A4 raises cap to 16 and triggers WatchEx at 16 stacks.
    let mut a4_req = request();
    a4_req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(4);
    let mut ally2 = a4_req.actors[0].clone();
    ally2.slot = 1;
    ally2.character_id = "character:1019".into();
    ally2.awakening = ntesim_data::AwakeningSelection::prefix(0);
    ally2.initial_resources = Default::default();
    a4_req.actors.push(ally2);
    a4_req.actions = vec![
        switch(100_000, 1), // Mint on-field while Shinku keeps ticking off-field via A3!
        switch(18_500_000, 0), // 16 stacks built while off-field [2s + 16s = 18s]
        make_action(
            18_600_000,
            0,
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            0,
            0,
        ),
    ];
    let a4_report = run_game_scenario(&a4_req, &ExecutionLimits::default()).unwrap();
    assert_eq!(a4_report["execution_completed"], true, "{a4_report}");
    let watch_16 = a4_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_Watch_Damage")
        .expect("Watch 16-stack hit");
    let watch_ex = a4_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_WatchEx_Damage")
        .expect("WatchEx 16-stack A4 hit");
    // A4 has A1 (12% DEF ignore -> 25/47) + R3 (+1 level: Watch Lv2 = 1.08, WatchEx Lv2 = 0.216).
    assert_eq!(watch_16["scaling_rate"], json!("432/25")); // 1.08 * 16 = 17.28
    assert_eq!(watch_ex["scaling_rate"], json!("432/125")); // 0.216 * 16 = 3.456
    assert_eq!(watch_ex["non_crit_damage"], json!("86400/47")); // 1000 * (432/125) * (25/47)

    // 3. DSL A0 switch clears Menacing Gaze whereas A3 preserves it.
    let swap_check = dsl_run(
        "options duration=10 crit=fixed;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        active shinku;
        wait(4.5);
        if !.shinku.menacing_gaze.active || .shinku.resources.watch!=2 {let assertion_failure=1/0;}
        mint swap;
        if .shinku.menacing_gaze.active || .shinku.resources.watch!=0 {let assertion_failure=1/0;}",
    );
    assert!(
        swap_check.report.success && swap_check.report.execution_completed,
        "{:?}",
        swap_check.diagnostic
    );

    // 4. Rage bonus class split: Watch inside Surging Crimson gets +0% without R3 and +30% with R3.
    let watch_rage_a0 = dsl_run(
        "options duration=15 crit=fixed;
        shinku char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=60;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;
        wait(3.5);
        shinku attack;",
    );
    let watch_rage_r3 = dsl_run(
        "options duration=15 crit=fixed;
        shinku char lvl=80/80 awakening=3 talent=1,1,1 qte=1 start_energy=60;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;
        wait(3.5);
        shinku attack;",
    );
    let r0_hit = serde_json::to_value(&watch_rage_a0.report).unwrap()["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_Watch_Damage")
        .unwrap()["non_crit_damage"]
        .clone();
    let r3_hit = serde_json::to_value(&watch_rage_r3.report).unwrap()["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["damage_effect_id"] == "GE_Player_Shinku_Watch_Damage")
        .unwrap()["non_crit_damage"]
        .clone();
    assert_eq!(r0_hit, json!("636/1")); // 636 * 1.0 * (1 + 0% Rage)
    // 636 * 1.08 (Lv2) * (1 + 0.30 R3 Rage) * (12/11 A2 -10% Cosmos RES) = 1339416/1375
    assert_eq!(r3_hit, json!("1339416/1375"));
}

#[test]
fn shinku_a5_def_percent_in_surging_crimson_and_scarlet_descent_heal() {
    let mid_surge = dsl_run(
        "options duration=5 crit=fixed;
        shinku char lvl=80/80 awakening=5 talent=1,1,1 qte=1 start_energy=60 start_hp=1000;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;",
    );
    assert!(
        mid_surge.report.execution_completed,
        "{:?}; {:?}",
        mid_surge.diagnostic, mid_surge.report.halt_diagnostic
    );
    let mid_report = serde_json::to_value(&mid_surge.report).unwrap();
    let a5_buff = crate::simulation::tests::support::final_buffs(&mid_report)
        .into_iter()
        .find(|b| b["id"] == "shinku.a5_def")
        .expect("shinku.a5_def active during Surging Crimson");
    assert_eq!(a5_buff["stat_modifiers"]["def_percent"], json!("1/2"));

    let result = dsl_run(
        "options duration=15 crit=fixed;
        shinku char lvl=80/80 awakening=5 talent=1,1,1 qte=1 start_energy=60 start_hp=1000;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        active shinku;
        shinku burst;
        shinku attack:5;
        shinku skill;
        mint swap;",
    );
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    let report = serde_json::to_value(&result.report).unwrap();
    // Scarlet Descent healed 3.0 * BaseATK!
    let heals = report["healing_receipts"].as_array().unwrap();
    assert_eq!(heals.len(), 1);
    let a5_heal = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_kind"] == "shinku_a5_scarlet_descent_heal")
        .expect("A5 heal trace present");
    assert_eq!(a5_heal["data"]["gross"], heals[0]["gross"]);
    // mint swap cleared Surging Crimson & shinku.a5_def buff.
    assert!(
        !crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|b| b["id"] == "shinku.a5_def")
    );
}
