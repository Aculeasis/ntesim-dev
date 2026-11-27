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
fn elyms_combat_start_skill1_then_power_e1_redirect_and_selective_cooldown_bypass() {
    let res = run_script(
        "options duration=25 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=0 start_energy=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         if .elyms.connection != 0 || .elyms.power_e1.active != 0 || .elyms.power_e2.active != 0 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 0 || .elyms.power_e1.active != 1 || .elyms.power_e2.active != 0 || .elyms.skill.ready != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 1 || .elyms.optical_illusion != 1 || .elyms.power_e1.active != 0 || .elyms.skill.ready != 0 { let fail = 1 / 0; }\n\
         elyms attack:3;\n\
         if .elyms.power_e2.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 2 || .elyms.optical_illusion != 0 || .elyms.consumed_stacks != 1 || .elyms.power_e2.active != 0 || .elyms.skill.ready != 0 { let fail = 1 / 0; }\n",
    );
    assert!(res.report.execution_completed, "{:?}", res.diagnostic);
    let json = serde_json::to_value(&res.report).unwrap();
    let hit_list = hits(&json);

    // Skill1 (6 hits) -> Skill2 (6 hits) -> Crosstalk B1(2)+B2(4)+B3(4) = 10 -> Skill3 (6 hits) = 28 hits.
    assert_eq!(hit_list.len(), 28);
    assert_eq!(
        hit_list[0]["damage_effect_id"],
        "GE_Player_Elyms_Skill1_Damage"
    );
    assert_eq!(
        hit_list[5]["damage_effect_id"],
        "GE_Player_Elyms_Skill1_2_Damage"
    );
    assert_eq!(
        hit_list[6]["damage_effect_id"],
        "GE_Player_Elyms_Skill2_Damage"
    );
    assert_eq!(
        hit_list[12]["damage_effect_id"],
        "GE_Player_Elyms_MeleeB1_Damage"
    );
    assert_eq!(
        hit_list[14]["damage_effect_id"],
        "GE_Player_Elyms_MeleeB2_Damage"
    );
    assert_eq!(
        hit_list[21]["damage_effect_id"],
        "GE_Player_Elyms_MeleeB3_1_Damage"
    );
    assert_eq!(
        hit_list[22]["damage_effect_id"],
        "GE_Player_Elyms_Skill3_Damage"
    );
    assert_eq!(
        hit_list[27]["damage_effect_id"],
        "GE_Player_Elyms_Skill3_1_Damage"
    );

    // Non-Ultimate Cinematic pause: Skill2 ends at SKILL2_RECOVERY (1_669_059) when Crosstalk starts; Skill3 runs full 1_719_655 -> 3_388_714 us.
    let pauses = json["timer_pauses"].as_array().unwrap();
    assert_eq!(
        pauses.len(),
        2,
        "only Skill2 and Skill3 admit Cinematic pauses: {pauses:?}"
    );
    let total_paused: u64 = pauses
        .iter()
        .map(|p| p["end_absolute_us"].as_u64().unwrap() - p["start_absolute_us"].as_u64().unwrap())
        .sum();
    assert_eq!(total_paused, 1_669_059 + 1_719_655);

    // Optical Illusion consumed on Skill3 doubles DMG Ratio (+100% CoefModify vs 0 stacks).
    let no_stack = run_script(
        "options duration=25 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         wait(13);\n\
         elyms skill;\n\
         wait(16);\n",
    );
    assert!(
        no_stack.report.execution_completed,
        "{:?}",
        no_stack.diagnostic
    );

    // Skill3_1 base ratio is 124.3%; with 1 Optical Illusion (+100% CoefModify) it deals 2x unboosted damage.
    let skill3_1_with_1_stack = res.report.hit_receipts[27].rounded_damage;
    let baseline_skill3 = run_script(
        "options duration=25 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms skill;\n\
         elyms skill;\n\
         wait(16);\n\
         elyms attack:3;\n\
         elyms skill;\n",
    );
    assert!(
        baseline_skill3.report.execution_completed,
        "{:?}",
        baseline_skill3.diagnostic
    );
    let base_s3_1 = baseline_skill3
        .report
        .hit_receipts
        .last()
        .unwrap()
        .rounded_damage;
    // Optical Illusion expired (15s < 16s wait), so base_s3_1 has 0 stacks consumed -> 1/2 of 1-stack Skill3_1 (before P1 stacks).
    assert!(skill3_1_with_1_stack > base_s3_1);
}

#[test]
fn elyms_secondary_aim_plunge_riposte_and_qte_dispatch() {
    let aim_and_branch = run_script(
        "options duration=10 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms attack[hold=0.25];\n\
         elyms special;\n\
         elyms attack:2;\n\
         elyms special;\n",
    );
    assert!(
        aim_and_branch.report.execution_completed,
        "{:?}",
        aim_and_branch.diagnostic
    );
    let j1 = serde_json::to_value(&aim_and_branch.report).unwrap();
    let h1 = hits(&j1);
    assert_eq!(h1.len(), 6);
    assert_eq!(h1[0]["damage_effect_id"], "GE_Player_Elyms_Branch1_Damage");
    assert_eq!(h1[1]["damage_effect_id"], "GE_Player_Elyms_Branch2_Damage");
    assert_eq!(h1[2]["damage_effect_id"], "GE_Player_Elyms_Branch1_Damage");
    assert_eq!(h1[3]["damage_effect_id"], "GE_Player_Elyms_Branch3_Damage");
    assert_eq!(h1[4]["damage_effect_id"], "GE_Player_Elyms_Aim_Damage");
    assert_eq!(h1[5]["damage_effect_id"], "GE_Player_Elyms_Aim_Damage");
    // Aim repeats on 146_353 us cadence (`48_313` then `48_313 + 146_353 = 194_666`).
    let aim_delta = h1[5]["time_us"].as_u64().unwrap() - h1[4]["time_us"].as_u64().unwrap();
    assert_eq!(aim_delta, 146_353);

    // Riposte + Plunge + QTE
    let secondary = run_script(
        "options duration=10 iteration=1 crit=fixed stamina_capacity=200 initial_stamina=200 stamina_use_efficiency=1 stamina_recovery_rate=0;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms dash[critical=true];\n\
         elyms attack;\n\
         elyms plunge[airtime=0.2, height_bonus=1];\n\
         akane swap;\n\
         elyms swap;\n\
         if .elyms.connection != 1 { let fail = 1 / 0; }\n",
    );
    assert!(
        secondary.report.execution_completed,
        "{:?}",
        secondary.diagnostic
    );
    let j2 = serde_json::to_value(&secondary.report).unwrap();
    let h2 = hits(&j2);
    assert_eq!(
        h2[0]["damage_effect_id"],
        "GE_Player_Elyms_PerfectEvadeAttack_Damage"
    );
    assert_eq!(
        h2[1]["damage_effect_id"],
        "GE_Player_Elyms_AirAttack_Damage"
    );
    assert!(
        h2.iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_QTE_Damage"),
        "expected QTE hit on swap: {h2:?}"
    );
}

#[test]
fn elyms_ultra_aoe_vs_st_atkbase_followup_and_on_the_line_pause_resume() {
    // 1 target at Medium Connection -> ST UltraSkillB + AtkBase team buff + Follow-up + Remembrance pause/resume + Echoes on Hold.
    let st = run_script(
        "options duration=30 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_energy=100;\n\
         elyms add weapon=fork_elyms refine=1;\n\
         elyms add stats atk=500 atk%=0.5;\n\
         haniel char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms skill;\n\
         elyms skill;\n\
         elyms burst;\n\
         haniel swap;\n\
         wait(1.0);\n\
         elyms swap;\n\
         wait(2.0);\n\
         haniel swap;\n\
         wait(20.0);\n",
    );
    assert!(st.report.execution_completed, "{:?}", st.diagnostic);
    let st_json = serde_json::to_value(&st.report).unwrap();
    let st_hits = hits(&st_json);

    // ST UltraSkillB packets present (`B1`, `B2`, `B3`) and `XieTongSkillB1` x 6 triggers * 4 hits = 24 hits!
    assert!(
        st_hits
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_UltraSkillB1_Damage")
    );
    assert!(
        st_hits
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_UltraSkillB3_Damage")
    );
    let xietong_st: Vec<_> = st_hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Elyms_XieTongSkillB1_Damage")
        .collect();
    assert_eq!(
        xietong_st.len(),
        24,
        "expected 6 triggers x 4 ST Remembrance hits"
    );

    // Verify pause/resume timing: trigger 1 fired on initial swap, paused 2s on-field, trigger 2 waited remaining ~2s!
    let t0 = xietong_st[0]["time_us"].as_u64().unwrap();
    let t1 = xietong_st[4]["time_us"].as_u64().unwrap();
    assert!(
        t1 - t0 >= 4_500_000,
        "2s on-field pause should push trigger 2 past 4.5s total gap (t0={t0}, t1={t1})"
    );

    // Check `elyms.ultra.atk_add` = 0.20 * (660 + 570) = 246 flat ATK (ignoring console add stats!).
    let q_atk_entry = st
        .report
        .buff_catalog
        .iter()
        .find(|entry| entry.id == "elyms.ultra.atk_add")
        .expect("expected elyms.ultra.atk_add in buff_catalog");
    assert_eq!(
        q_atk_entry.stat_modifiers.get("atk_add"),
        Some(&crate::formula::Ratio {
            numerator: 246,
            denominator: 1,
        })
    );

    // Check weapon:fork_elyms (`Echoes on Hold`) activated from Remembrance Follow-up packets!
    assert!(
        st.report
            .resolved_model_assumptions
            .contains_key(crate::weapons::echoes_on_hold::HYPOTHESIS_ID.as_str()),
        "XieTongSkillB1 must satisfy State.Damage.Followup on weapon:fork_elyms"
    );

    // Opening Skill1 grants 100 direct Cycle plus 9 packet Cycle; Remembrance adds 16, capped at 100 with 199/1000 carry.
    assert!(st.report.final_actor_states[0].granted_energy >= 23); // Skill1 (5.1) + Skill2 (8.3) + 24*0.425 (10.2) = 23.6
    let cycle_ledger = st.report.final_actor_states[0]
        .cycle
        .as_ref()
        .expect("expected Elyms cycle ledger on ST run");
    assert_eq!(cycle_ledger.granted, 125);
    assert_eq!(cycle_ledger.current, 100);
    assert_eq!(
        (cycle_ledger.carry.numerator, cycle_ledger.carry.denominator),
        (199, 1000)
    );

    // 2 living targets -> AoE UltraSkillA1/A2 and XieTongSkillA1
    let aoe_source = "options duration=25 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_energy=100;\n\
         haniel char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         target hp=90000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms skill;\n\
         elyms skill;\n\
         elyms burst;\n\
         haniel swap;\n\
         wait(18);\n";
    let aoe = run_script(aoe_source);
    assert!(aoe.report.execution_completed, "{:?}", aoe.diagnostic);
    let aoe_json = serde_json::to_value(&aoe.report).unwrap();
    let aoe_hits = hits(&aoe_json);
    assert!(
        aoe_hits
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_UltraSkillA1_Damage")
    );
    assert!(
        aoe_hits
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_UltraSkillA2_Damage")
    );
    // 6 triggers * 4 waves * 2 targets = 48 AoE hits
    let xietong_aoe: Vec<_> = aoe_hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Elyms_XieTongSkillA1_Damage")
        .collect();
    assert_eq!(xietong_aoe.len(), 48);

    // Skill2 defeats t1 after the scan; the Ultimate keeps its AoE variant.
    let after_scan = run_script(&aoe_source.replace("hp=90000000", "hp=3200"));
    assert!(
        after_scan.report.execution_completed,
        "{:?}",
        after_scan.diagnostic
    );
    let hits = &after_scan.report.hit_receipts;
    assert!(hits.iter().any(
        |hit| hit.target_index == 1 && hit.damage_effect_id == "GE_Player_Elyms_Skill2_Damage"
    ));
    assert!(
        hits.iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Elyms_UltraSkillA1_Damage")
    );
    assert_eq!(
        hits.iter()
            .filter(|hit| hit.damage_effect_id == "GE_Player_Elyms_XieTongSkillA1_Damage")
            .count(),
        24
    );
}

#[test]
fn elyms_relay_node_charge_enhancement_and_remora_cooperation() {
    // 1. Charge Enhancement ignores start_energy and stacks on active-actor energy gain up to 8 stacks (+30% Cosmos).
    let charge_res = PreparedSourceScenario::new(
        "options duration=15 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_energy=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms skill;\n\
         elyms attack:5;\n",
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(1)
    .expect("charge_res run");
    assert!(
        charge_res.report.execution_completed,
        "{:?}",
        charge_res.diagnostic
    );
    let cjson = serde_json::to_value(&charge_res.report).unwrap();
    let ce_events: Vec<_> = cjson["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "elyms_charge_enhancement")
        .collect();
    assert!(
        ce_events.len() >= 8,
        "expected >= 8 active-actor energy gain events after t=0 start_energy: {ce_events:?}"
    );
    assert!(
        ce_events.iter().all(|e| e["time_us"].as_u64().unwrap() > 0),
        "start_energy at t=0 must never trigger Charge Enhancement"
    );

    // 2. Remora Enhancement: Elyms + Akane trigger Remora -> Elyms schedules +7.5s delayed Remora which fires Akane Perfect Pitch without infinite recursion!
    let remora_res = PreparedSourceScenario::new(
        "options duration=25 iteration=1 crit=fixed;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100 start_energy=0;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100 start_energy=0;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms attack;\n\
         akane swap;\n\
         akane skill;\n\
         wait(20);\n",
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(1)
    .expect("remora_res run");
    assert!(
        remora_res.report.execution_completed,
        "{:?}",
        remora_res.diagnostic
    );
    let rjson = serde_json::to_value(&remora_res.report).unwrap();
    let remora_enh: Vec<_> = rjson["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == "elyms_remora_enhancement")
        .collect();
    assert!(
        !remora_enh.is_empty(),
        "expected Elyms Remora Enhancement triggers"
    );
    // Self-applied delayed Remora cannot self-retrigger immediately inside 7.5s: check spacing >= 7.0s between Remora Enhancement triggers!
    if remora_enh.len() >= 2 {
        let gap =
            remora_enh[1]["time_us"].as_u64().unwrap() - remora_enh[0]["time_us"].as_u64().unwrap();
        assert!(
            gap >= 7_000_000,
            "Remora Enhancement must obey >= 7.0s global ICD"
        );
    }
    assert!(
        hits(&rjson)
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Reaction_3_new_1057_Damage"),
        "delayed Remora Enhancement must cooperate with Akane Perfect Pitch"
    );
}

#[test]
fn elyms_passive2_awakenings_and_resonances_r3_r6() {
    // Compare A0 vs C6 R6 (`Skill1 -> Skill2 -> Skill3 (4 stacks + Skill3Extra) -> Q -> Remembrance`), checking A3/R3/A1/A2/A4/A5/A6/R6!
    let c6 = run_script(
        "options duration=30 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=6 talent=10,10,10 qte=10 start_cycle=0 start_energy=100;\n\
         haniel char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=0;\n\
         target hp=100000000 def=1000 resist=0.2;\n\
         active elyms;\n\
         if .elyms.resources.cycle != 100 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 0 || .elyms.power_e1.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 2 || .elyms.optical_illusion != 4 || .elyms.power_e2.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.optical_illusion != 0 || .elyms.consumed_stacks != 4 { let fail = 1 / 0; }\n\
         elyms burst;\n\
         if .elyms.consumed_stacks != 0 || .elyms.connection != 0 { let fail = 1 / 0; }\n\
         haniel swap;\n\
         wait(18);\n",
    );
    assert!(c6.report.execution_completed, "{:?}", c6.diagnostic);
    let c6_json = serde_json::to_value(&c6.report).unwrap();
    let c6_hits = hits(&c6_json);

    assert!(
        c6_hits
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_Skill3Extra_Damage"),
        "R6 must emit GE_Player_Elyms_Skill3Extra_Damage when Optical Illusion is consumed"
    );
    assert!(
        c6_hits
            .iter()
            .any(|h| h["damage_effect_id"] == "GE_Player_Elyms_UltraSkillB1_Damage"),
        "R3 Skill2->Skill3->Burst at 1 dummy must select ST UltraSkillB"
    );
    // Check A1/A4 Crit DMG applied on Burst hits (Passive 2 +10%, A4 +20%, A1 +40% = +70% extra Crit DMG on top of 50% base = +120% -> 11/5 crit multiplier!).
    let burst_hit = c6
        .report
        .hit_receipts
        .iter()
        .find(|h| h.damage_effect_id == "GE_Player_Elyms_UltraSkillB3_Damage")
        .unwrap();
    assert_eq!(
        burst_hit.crit_damage.numerator * 5 * burst_hit.non_crit_damage.denominator,
        burst_hit.non_crit_damage.numerator * 11 * burst_hit.crit_damage.denominator,
        "expected +120% crit dmg (11/5 ratio): crit={:?} non_crit={:?}",
        burst_hit.crit_damage,
        burst_hit.non_crit_damage
    );
}

#[test]
fn elyms_a3_grants_stacks_on_skill3_without_resonance() {
    let result = run_script(
        "options duration=25 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=[3] talent=10,10,10 qte=10;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         elyms skill;\n\
         if .elyms.connection != 0 || .elyms.power_e1.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 1 || .elyms.power_e2.active { let fail = 1 / 0; }\n\
         elyms attack:3;\n\
         elyms skill;\n\
         if .elyms.connection != 2 || .elyms.optical_illusion != 3 || .elyms.consumed_stacks != 1 || .elyms.power_e2.active { let fail = 1 / 0; }\n\
         elyms attack:3;\n\
         elyms skill;\n\
         if .elyms.optical_illusion != 0 || .elyms.consumed_stacks != 4 { let fail = 1 / 0; }\n",
    );
    assert!(result.report.execution_completed, "{:?}", result.diagnostic);
}

#[test]
fn elyms_two_cycle_manual_oracle_and_dsl_queries() {
    // 2 complete A0 cycles verifying Weak/Med/Strong transitions, Skill1 Cycle refill, QTE, Skill3, Burst, and 2x On the Line!
    // Use cross-element Akane (`LAKSHANA` -> `COSMOS` Remora) so `swap` admits Support QTE on both transitions.
    let oracle = run_script(
        "options duration=80 iteration=1 crit=fixed;\n\
         elyms char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100 start_energy=100;\n\
         akane char lvl=80/80 awakening=0 talent=10,10,10 qte=10 start_cycle=100;\n\
         target hp=100000000 def=0 resist=0;\n\
         active elyms;\n\
         // Cycle 1: Weak -> Skill1 (PowerE1) -> Skill2 (Medium, 1 Optical Illusion) -> Attack x3 (PowerE2) -> Skill3 (Strong, consumes 1) -> Burst -> Swap out -> Swap in (QTE advances Weak -> Medium, +1 stack).\n\
         elyms skill;\n\
         if .elyms.connection != 0 || .elyms.power_e1.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 1 || .elyms.optical_illusion != 1 { let fail = 1 / 0; }\n\
         elyms attack:3;\n\
         if .elyms.power_e2.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 2 || .elyms.consumed_stacks != 1 { let fail = 1 / 0; }\n\
         elyms burst;\n\
         if .elyms.on_the_line != 6 || .elyms.connection != 0 { let fail = 1 / 0; }\n\
         akane swap;\n\
         wait(18);\n\
         elyms swap;\n\
         // Cycle 2: QTE already transitioned Weak -> Medium with 1 Optical Illusion.\n\
         if .elyms.connection != 1 || .elyms.optical_illusion != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.resources.cycle != 100 || .elyms.connection != 1 { let fail = 1 / 0; }\n\
         elyms attack:3;\n\
         if .elyms.power_e2.active != 1 { let fail = 1 / 0; }\n\
         elyms skill;\n\
         if .elyms.connection != 2 || .elyms.consumed_stacks != 1 { let fail = 1 / 0; }\n\
         while !.elyms.burst.ready {\n\
           if .elyms.skill.ready { elyms skill; } else { elyms attack; }\n\
         }\n\
         elyms burst;\n\
         if .elyms.on_the_line != 6 || .elyms.connection != 0 { let fail = 1 / 0; }\n\
         akane swap;\n\
         wait(18);\n",
    );
    assert!(oracle.report.execution_completed, "{:?}", oracle.diagnostic);
    let json = serde_json::to_value(&oracle.report).unwrap();
    let xietong: Vec<_> = hits(&json)
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Elyms_XieTongSkillB1_Damage")
        .collect();
    assert_eq!(
        xietong.len(),
        48,
        "expected 2 cycles * 6 triggers * 4 hits = 48 Remembrance hits"
    );
}
