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
