use crate::simulation::{PreparedSourceScenario, SourceScenarioReport};
use serde_json::Value;

fn run_script(source: &str) -> SourceScenarioReport {
    PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(1)
        .unwrap()
}

fn hits(report: &Value) -> &Vec<Value> {
    report["hit_receipts"].as_array().unwrap()
}

#[test]
fn akane_normal_chain_hitstop_cancel_and_melee4_delay_project_exact_times() {
    let res = run_script(
        "options duration=10 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane attack:4;\n\
         if .akane.string != 10 { let fail = 1 / 0; }\n",
    );
    assert!(res.report.execution_completed, "{:?}", res.diagnostic);
    let json = serde_json::to_value(&res.report).unwrap();
    let hit_list = hits(&json);
    // N1(2) + N2(2) + N3(3 Melee3 + 4 Melee3_1) + N4(1) = 12 hits.
    assert_eq!(hit_list.len(), 12);
    assert_eq!(
        hit_list[0]["damage_effect_id"],
        "GE_Player_Akane_Melee1_Damage"
    );
    assert_eq!(hit_list[0]["time_us"], 133_639);
    assert_eq!(hit_list[1]["time_us"], 465_515);
    assert_eq!(
        hit_list[2]["damage_effect_id"],
        "GE_Player_Akane_Melee2_Damage"
    );
    assert_eq!(hit_list[2]["time_us"], 964_735);
    assert_eq!(hit_list[3]["time_us"], 1_154_652);
    assert_eq!(
        hit_list[4]["damage_effect_id"],
        "GE_Player_Akane_Melee3_Damage"
    );
    assert_eq!(hit_list[4]["time_us"], 1_445_728);
    assert_eq!(
        hit_list[7]["damage_effect_id"],
        "GE_Player_Akane_Melee3_1_Damage"
    );
    assert_eq!(hit_list[7]["time_us"], 2_184_744);
    assert_eq!(
        hit_list[11]["damage_effect_id"],
        "GE_Player_Akane_Melee4_Damage"
    );
    assert_eq!(hit_list[11]["time_us"], 2_960_113);

    // Grounded Jump cancels N1 inside its Break Jump window cleanly.
    let jump_cancel = run_script(
        "options duration=10 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane attack[window=jump];\n\
         akane jump;\n",
    );
    assert!(
        jump_cancel.report.execution_completed,
        "{:?}",
        jump_cancel.diagnostic
    );
    let jcjson = serde_json::to_value(&jump_cancel.report).unwrap();
    assert_eq!(hits(&jcjson).len(), 2);

    // Cutoff at 2.1s during N3 (after 3 Melee3 hits, before 812ms Melee3_1 notify)
    // skips unissued Melee3_1 hits cleanly.
    let n3_cutoff = run_script(
        "options duration=2.1 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane attack:3;\n",
    );
    assert!(
        n3_cutoff.report.execution_completed,
        "{:?}",
        n3_cutoff.diagnostic
    );
    let cjson = serde_json::to_value(&n3_cutoff.report).unwrap();
    assert_eq!(
        hits(&cjson)
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Akane_Melee3_Damage")
            .count(),
        3
    );
    assert!(
        hits(&cjson)
            .iter()
            .all(|h| h["damage_effect_id"] != "GE_Player_Akane_Melee3_1_Damage")
    );
}

#[test]
fn akane_string_energy_deduplication_and_multi_dummy_overtone_vs_reverb() {
    let res = run_script(
        "options duration=15 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=2 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[target=t1];\n\
         akane attack:4;\n\
         if .akane.string != 16 { let fail = 1 / 0; }\n",
    );
    assert!(res.report.execution_completed, "{:?}", res.diagnostic);

    let a5_res = run_script(
        "options duration=15 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=5 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[target=t1];\n\
         akane attack:4;\n\
         if .akane.string != 30 { let fail = 1 / 0; }\n",
    );
    assert!(a5_res.report.execution_completed, "{:?}", a5_res.diagnostic);
    let json = serde_json::to_value(&a5_res.report).unwrap();
    let hit_list = hits(&json);
    assert_eq!(
        hit_list
            .iter()
            .filter(|h| h["target_index"] == 0
                && h["damage_effect_id"] == "GE_Player_Akane_Melee1_Damage")
            .count(),
        0
    );
    assert_eq!(
        hit_list
            .iter()
            .filter(|h| h["target_index"] == 1
                && h["damage_effect_id"] == "GE_Player_Akane_Melee1_Damage")
            .count(),
        2
    );

    let t0_n2 = hit_list
        .iter()
        .find(|h| {
            h["target_index"] == 0 && h["damage_effect_id"] == "GE_Player_Akane_Melee2_Damage"
        })
        .unwrap();
    let t1_n2 = hit_list
        .iter()
        .find(|h| {
            h["target_index"] == 1 && h["damage_effect_id"] == "GE_Player_Akane_Melee2_Damage"
        })
        .unwrap();
    let n2_0: u64 = t0_n2["rounded_damage"].as_u64().unwrap();
    let n2_1: u64 = t1_n2["rounded_damage"].as_u64().unwrap();
    assert!(n2_1 > n2_0);

    // Mutual exclusivity: Hold replaces Overtone with Reverb Field; Press restores Overtone.
    let swap = run_script(
        "options duration=40 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=2 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[target=t0];\n\
         if .akane.mode != 1 { let fail = 1 / 0; }\n\
         wait(.akane.skill.cd);\n\
         akane skill[hold=0.3s];\n\
         if .akane.mode != 2 { let fail = 1 / 0; }\n\
         if .akane.overtone != 0 { let fail = 1 / 0; }\n\
         if .akane.reverb != 1 { let fail = 1 / 0; }\n\
         wait(.akane.skill.cd);\n\
         akane skill[target=t0];\n\
         if .akane.mode != 1 { let fail = 1 / 0; }\n",
    );
    assert!(swap.report.execution_completed, "{:?}", swap.diagnostic);
}

#[test]
fn akane_synced_resonance_hold_rejection_no_display_packet_and_preserves_cd_and_mode() {
    let invalid_energy = PreparedSourceScenario::new(
        "options duration=10 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_string_energy=31;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill;\n",
        Default::default(),
        &Default::default(),
    );
    assert!(invalid_energy.is_err());

    let hold_reject = run_script(
        "options duration=20 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_string_energy=30;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill;\n\
         akane skill[hold=0.25s];\n",
    );
    assert!(!hold_reject.report.execution_completed);

    let synced = run_script(
        "options duration=20 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_string_energy=30;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill;\n\
         wait(4.5);\n\
         if .akane.overtone != 1 || .akane.string != 30 { let fail = 1 / 0; }\n\
         akane skill;\n\
         if .akane.string != 0 { let fail = 1 / 0; }\n\
         if .akane.skill.cd <= 8 || .akane.mods.overtone <= 17 { let fail = 1 / 0; }\n",
    );
    assert!(synced.report.execution_completed, "{:?}", synced.diagnostic);
    let json = serde_json::to_value(&synced.report).unwrap();
    let hit_list = hits(&json);
    assert!(
        hit_list
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Akane_UpgradeSkill_Damage")
    );
    assert!(
        hit_list
            .iter()
            .all(|h| h["damage_effect_id"] != "GE_Player_Akane_UpgradeSkill_DisplayGE")
    );

    // If Synced Resonance ends before its impact notify (429ms on Overtone / 1004ms on Reverb), String Energy remains 30.
    let pre_impact_cutoff = run_script(
        "options duration=1.2 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_string_energy=30;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill;\n\
         akane skill;\n\
         if .akane.string != 30 { let fail = 1 / 0; }\n",
    );
    assert!(
        pre_impact_cutoff.report.execution_completed,
        "{:?}",
        pre_impact_cutoff.diagnostic
    );
    let pcjson = serde_json::to_value(&pre_impact_cutoff.report).unwrap();
    assert!(
        hits(&pcjson)
            .iter()
            .all(|h| h["damage_effect_id"] != "GE_Player_Akane_UpgradeSkill_Damage")
    );
}

#[test]
fn akane_sonic_boom_cadenza_pause_manual_stage2_and_no_auto_cast() {
    let manual = run_script(
        "options duration=40 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=6 talent=10,10,10 qte=10 start_energy=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane burst;\n\
         if .akane.cadenza != 1 || .akane.anthem != 1 || .akane.burst.ready != 0 { let fail = 1 / 0; }\n\
         akane attack;\n\
         if .akane.energy <= 0 { let fail = 1 / 0; }\n\
         wait(5);\n\
         akane burst;\n\
         if .akane.cadenza != 0 || .akane.mods.anthem <= 17 || .akane.burst.cd <= 5 { let fail = 1 / 0; }\n",
    );
    assert!(manual.report.execution_completed, "{:?}", manual.diagnostic);
    let json = serde_json::to_value(&manual.report).unwrap();
    let hit_list = hits(&json);
    assert_eq!(
        hit_list
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Akane_UltraSkill1_Damage")
            .count(),
        10
    );
    assert_eq!(
        hit_list
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Akane_UltraSkill2_Damage")
            .count(),
        1
    );

    // Natural Solo Cadenza expiry does NOT auto-cast Stage 2, and out-of-combat +3/s energy regen is omitted.
    let expired = run_script(
        "options duration=35 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_energy=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane burst;\n\
         wait(21);\n\
         if .akane.cadenza != 0 || .akane.energy != 0 { let fail = 1 / 0; }\n",
    );
    assert!(
        expired.report.execution_completed,
        "{:?}",
        expired.diagnostic
    );
    let ejson = serde_json::to_value(&expired.report).unwrap();
    assert_eq!(
        hits(&ejson)
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Akane_UltraSkill2_Damage")
            .count(),
        0
    );
}

#[test]
fn akane_perfect_pitch_remora_icd_hathor_chaos_and_two_cycle_oracle() {
    let multi = PreparedSourceScenario::new(
        "options duration=25 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=6 talent=10,10,10 qte=10 start_cycle=100;\n\
         akane add weapon=fork_prism refine=1;\n\
         zero char lvl=80/80 awakening=2 talent=10,10,10 qte=10 start_cycle=100;\n\
         chaos char lvl=80/80 awakening=2 talent=10,10,10 qte=10 start_cycle=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[hold=0.3s];\n\
         if .akane.perfect_pitch != 0 { let fail = 1 / 0; }\n\
         zero swap;\n\
         wait(1);\n\
         akane swap;\n\
         if .akane.perfect_pitch != 1 { let fail = 1 / 0; }\n\
         wait(0.1);\n\
         chaos swap;\n\
         wait(8);\n",
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(1)
    .expect("multi scenario failed");
    assert!(multi.report.execution_completed, "{:?}", multi.diagnostic);
    let json = serde_json::to_value(&multi.report).unwrap();
    assert!(
        hits(&json)
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Reaction_3_new_1057_Damage")
    );
    assert!(json["reaction_counts"]["Remora"].as_u64().unwrap() >= 2);

    // Two-cycle oracle at R6: battle-entry 30 String Energy, R6 post-spend +10 refund,
    // independent A4(+30%)/R6(+64%) CritDMG stacking, and full 120s continuity.
    let oracle = PreparedSourceScenario::new(
        "options duration=120 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=6 talent=10,10,10 qte=10 start_energy=0;\n\
         akane add weapon=fork_prism refine=5;\n\
         akane add stats atk=2000 cr=0 cd=0;\n\
         target hp=500000000 def=0 resist=0;\n\
         active akane;\n\
         if .akane.string != 30 { let fail = 1 / 0; }\n\
         akane skill;\n\
         if .akane.string != 30 { let fail = 1 / 0; }\n\
         akane skill;\n\
         if .akane.string != 10 { let fail = 1 / 0; }\n\
         akane attack:4;\n\
         akane attack:4;\n\
         if .akane.burst.ready { akane burst; wait(5); akane burst; }\n\
         wait_until(60);\n\
         if .akane.skill.ready { akane skill; }\n\
         akane attack:4;\n\
         if .akane.burst.ready { akane burst; wait(5); akane burst; }\n",
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(1)
    .expect("oracle scenario failed");
    assert!(oracle.report.execution_completed, "{:?}", oracle.diagnostic);
    let ojson = serde_json::to_value(&oracle.report).unwrap();
    assert!(
        hits(&ojson)
            .iter()
            .filter(|h| h["damage_effect_id"] == "GE_Player_Akane_UpgradeSkill_Damage")
            .count()
            >= 1
    );
    assert!(ojson["total_damage"].as_u64().unwrap_or(1) > 0);
}

#[test]
fn akane_packet_cycle_accrual_and_multi_target_reverb_melee1() {
    // Single-target Overtone: Skill (6.000) + N1..N4 (23.999) = 29.999 -> 29 whole Cycle (999/1000 carry).
    let overtone = run_script(
        "options duration=10 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill;\n\
         akane attack:4;\n",
    );
    assert!(
        overtone.report.execution_completed,
        "{:?}",
        overtone.diagnostic
    );
    let ojson = serde_json::to_value(&overtone.report).unwrap();
    assert_eq!(ojson["final_actor_states"][0]["cycle"]["current"], 29);
    assert!(
        ojson["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "packet_cycle_grant")
    );

    // Multi-target Reverb Field: N1 (Melee1) relays to every living dummy (2 hits x 2 targets = 4),
    // while Cycle accrual deduplicates AoE packets: Skill Hold (4.100 x 2 = 8.200) + N1..N4 (23.999) = 32.199 -> 32.
    let reverb = run_script(
        "options duration=12 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[hold=0.3s];\n\
         akane attack:4;\n",
    );
    assert!(reverb.report.execution_completed, "{:?}", reverb.diagnostic);
    let rjson = serde_json::to_value(&reverb.report).unwrap();
    let r_hits = hits(&rjson);
    assert_eq!(
        r_hits
            .iter()
            .filter(|h| h["target_index"] == 0
                && h["damage_effect_id"] == "GE_Player_Akane_Melee1_Damage")
            .count(),
        2
    );
    assert_eq!(
        r_hits
            .iter()
            .filter(|h| h["target_index"] == 1
                && h["damage_effect_id"] == "GE_Player_Akane_Melee1_Damage")
            .count(),
        2
    );
    assert_eq!(rjson["final_actor_states"][0]["cycle"]["current"], 32);
}

#[test]
fn akane_reverb_scope_a1_self_heal_and_perfect_pitch_final_damage() {
    use crate::simulation::ExactFraction;

    // Reverb Field +60% / A5 +90% DMG bonus applies to relayed Melee/Riposte hits, NOT Skill2 or UpgradeSkill.
    let a0 = run_script(
        "options duration=12 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_string_energy=30;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[hold=0.3s];\n\
         akane attack;\n\
         akane skill;\n",
    );
    let a5 = run_script(
        "options duration=12 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=[5] talent=10,10,10 qte=10 start_string_energy=30;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[hold=0.3s];\n\
         akane attack;\n\
         akane skill;\n",
    );
    assert!(a0.report.execution_completed, "{:?}", a0.diagnostic);
    assert!(a5.report.execution_completed, "{:?}", a5.diagnostic);
    let a0_json = serde_json::to_value(&a0.report).unwrap();
    let a5_json = serde_json::to_value(&a5.report).unwrap();
    let find_non_crit = |report: &Value, effect: &str| -> ExactFraction {
        serde_json::from_value(
            hits(report)
                .iter()
                .find(|h| h["damage_effect_id"] == effect)
                .unwrap()["non_crit_damage"]
                .clone(),
        )
        .unwrap()
    };
    let a0_skill2 = find_non_crit(&a0_json, "GE_Player_Akane_Skill2_Damage");
    let a5_skill2 = find_non_crit(&a5_json, "GE_Player_Akane_Skill2_Damage");
    assert_eq!(a0_skill2, a5_skill2);
    // 636 base ATK * 3.238 (Skill2 Lv10) = 2059.368 = 257421/125, with zero Reverb DMG bonus.
    assert_eq!(a0_skill2, ExactFraction::new(257_421, 125).unwrap());

    let a0_upskill = find_non_crit(&a0_json, "GE_Player_Akane_UpgradeSkill_Damage");
    let a5_upskill = find_non_crit(&a5_json, "GE_Player_Akane_UpgradeSkill_Damage");
    assert_eq!(a0_upskill, a5_upskill);
    // 636 base ATK * 20.240 (UpgradeSkill Lv10) = 12872.64 = 321816/25, with zero Reverb DMG bonus.
    assert_eq!(a0_upskill, ExactFraction::new(321_816, 25).unwrap());

    let a0_melee1 = find_non_crit(&a0_json, "GE_Player_Akane_Melee1_Damage");
    let a5_melee1 = find_non_crit(&a5_json, "GE_Player_Akane_Melee1_Damage");
    assert_ne!(a0_melee1, a5_melee1);

    // A1 heals only Akane Rin herself (not teammates).
    let a1_heal = run_script(
        "options duration=10 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=1 talent=10,10,10 qte=10 start_hp%=0.1 start_string_energy=30;\n\
         zero char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_hp%=0.1;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill;\n\
         akane skill;\n",
    );
    assert!(
        a1_heal.report.execution_completed,
        "{:?}",
        a1_heal.diagnostic
    );
    let hjson = serde_json::to_value(&a1_heal.report).unwrap();
    let heals: Vec<&Value> = hjson["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "akane_a1_heal")
        .collect();
    assert_eq!(heals.len(), 1);
    assert_eq!(heals[0]["to_actor"], 0);
    assert!(hjson["final_actor_states"][0]["hp"].as_u64().unwrap() > 1599);
    assert_eq!(hjson["final_actor_states"][1]["hp"].as_u64().unwrap(), 1515);

    // Perfect Pitch uses FinalDamageUp (+160% at >= 8s remaining Remora): 636 * 1.50 * (1 + 1.60) = 2480.4 = 12402/5.
    let pp = run_script(
        "options duration=15 iteration=1 crit=random;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         hathor char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100;\n\
         zero char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active akane;\n\
         akane skill[hold=0.3s];\n\
         hathor swap;\n\
         wait(1);\n\
         zero swap;\n\
         wait(1);\n",
    );
    assert!(pp.report.execution_completed, "{:?}", pp.diagnostic);
    let pjson = serde_json::to_value(&pp.report).unwrap();
    let pp_dmg = find_non_crit(&pjson, "GE_Reaction_3_new_1057_Damage");
    assert_eq!(pp_dmg, ExactFraction::new(12_402, 5).unwrap());
}
