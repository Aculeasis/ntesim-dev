use super::*;
use serde_json::json;

/// Documented Katana input starts for the Burst at 36s fixture.
const STAGE_STARTS: [u64; 10] = [
    40_604_238, 40_928_818, 41_411_492, 42_203_310, 43_018_384, 43_490_588, 43_815_169, 44_297_843,
    45_089_661, 45_904_735,
];

const FINISHER_START: u64 = 46_376_939;

const STAGE_PACKETS: [&str; 13] = [
    "GE_Player_Jin_UltraSkillMelee1_Damage",
    "GE_Player_Jin_UltraSkillMelee2_Damage",
    "GE_Player_Jin_UltraSkillMelee2_1_Damage",
    "GE_Player_Jin_UltraSkillMelee3_Damage",
    "GE_Player_Jin_UltraSkillMelee3_Damage",
    "GE_Player_Jin_UltraSkillMelee3_Damage",
    "GE_Player_Jin_UltraSkillMelee3_1_Damage",
    "GE_Player_Jin_UltraSkillMelee4_Damage",
    "GE_Player_Jin_UltraSkillMelee4_1_Damage",
    "GE_Player_Jin_UltraSkillMelee4_2_Damage",
    "GE_Player_Jin_UltraSkillMelee4_3_Damage",
    "GE_Player_Jin_UltraSkillMelee5_Damage",
    "GE_Player_Jin_UltraSkillMelee5_1_Damage",
];

fn action(
    timestamp_us: u64,
    ability_id: &str,
    damage_effect_id: &str,
    recovery_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us,
        cooldown_us: 0,
        hit_delay_us: 100,
    }
}

fn basic(timestamp_us: u64) -> GameActionCommand {
    action(
        timestamp_us,
        "GA_Jin_Melee",
        "GE_Player_Jin_Melee1_Damage",
        0,
    )
}

fn burst(timestamp_us: u64) -> GameActionCommand {
    action(
        timestamp_us,
        "GA_Jin_UltraSkill",
        "GE_Player_Jin_UltraSkill1_Damage",
        crate::characters::hotori::skill::public_burst_timing().0,
    )
}

fn chain_actions() -> Vec<GameActionCommand> {
    let mut actions = vec![burst(36_000_000)];
    actions.extend(STAGE_STARTS.into_iter().map(basic));
    actions.push(basic(FINISHER_START));
    actions
}

fn trace_times(report: &serde_json::Value, kind: &str) -> Vec<u64> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect()
}

#[test]
fn hotori_katana_two_trains_and_breakthrough_finisher_end_time_stop() {
    let mut req = super::hotori_actions::request();
    req.relative_duration_us = 55_000_000;
    req.actions = chain_actions();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["rejected_action"], serde_json::Value::Null);
    assert_eq!(report["hits_settled"], 37);
    let hits = report["hit_receipts"].as_array().unwrap();

    // The initial Burst train is unchanged.
    assert_eq!(
        hits[0]["damage_effect_id"],
        "GE_Player_Jin_UltraSkill1_Damage"
    );
    assert_eq!(
        hits[9]["damage_effect_id"],
        "GE_Player_Jin_UltraSkill3_Damage"
    );

    // Two complete five-input trains deliver the recorded 26 preliminary hits.
    for train in 0..2 {
        for (index, effect) in STAGE_PACKETS.iter().enumerate() {
            let hit = &hits[10 + train * 13 + index];
            assert_eq!(
                hit["damage_effect_id"], *effect,
                "train {train} packet {index}"
            );
            assert_eq!(
                super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"],
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_Q"
            );
        }
    }
    let expected_times: [u64; 26] = [
        40_787_669, 41_086_924, 41_261_911, 41_611_294, 41_679_269, 41_744_391, 41_961_327,
        42_287_054, 42_470_086, 42_693_555, 42_836_152, 43_231_056, 43_290_963, 43_674_019,
        43_973_275, 44_148_262, 44_497_645, 44_565_620, 44_630_742, 44_847_678, 45_173_405,
        45_356_437, 45_579_906, 45_722_503, 46_117_407, 46_177_314,
    ];
    for (index, time_us) in expected_times.into_iter().enumerate() {
        assert_eq!(
            hits[10 + index]["time_us"],
            time_us,
            "preliminary hit {index}"
        );
    }

    // Level-1 talent Katana coefficients, ATK 1000, DEF factor 1/2.
    let expected_damage = [
        (381, 2),
        (117, 1),
        (637, 2),
        (46, 1),
        (46, 1),
        (46, 1),
        (445, 2),
        (255, 2),
        (297, 2),
        (127, 1),
        (212, 1),
        (85, 2),
        (2211, 2),
    ];
    for (index, (numerator, denominator)) in expected_damage.into_iter().enumerate() {
        for hit in [&hits[10 + index], &hits[23 + index]] {
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator)),
                "packet {index}"
            );
        }
    }

    // B6 replaces the ordinary finisher with the empowered row.
    assert_eq!(
        hits[36]["damage_effect_id"],
        "GE_Player_Jin_UltraSkillMelee6Pas2_Damage"
    );
    assert_eq!(hits[36]["non_crit_damage"], json!("3000/1"));
    assert_eq!(hits[36]["time_us"], 47_834_796);

    // Ten count markers increment once per admitted input, never per hit.
    assert_eq!(
        trace_times(&report, "hotori_katana_count"),
        vec![
            40_804_050, 41_111_984, 41_636_929, 42_302_823, 43_258_695, 43_690_400, 43_998_335,
            44_523_280, 45_189_174, 46_145_046,
        ]
    );
    // The finisher ends Time Stop at its damage delivery; the later clear
    // marker is idempotent cleanup.
    assert_eq!(
        trace_times(&report, "hotori_time_stop_finisher"),
        vec![47_834_796]
    );
    assert_eq!(
        trace_times(&report, "hotori_time_stop_clear"),
        vec![48_128_948]
    );
    assert!(trace_times(&report, "hotori_time_stop_expired").is_empty());

    // The55-relative-second endpoint is absolute66.834796: nineteen world
    // seconds after the finisher recover19*6=114 Timepiece.
    let timepiece = &report["final_actor_states"][0]["fractional_resources"]["hotori.timepiece"];
    assert_eq!(timepiece["current"], json!("114/1"));
    assert_eq!(timepiece["spent"], json!("120/1"));
    assert_eq!(timepiece["overflow"], json!("96/1"));
    assert_eq!(timepiece["conserved"], true);
}

/// The finisher ends Time Stop at its 47.834796 hit, releasing a paused enemy
/// schedule after its remaining delay and shifting later schedules by only the
/// 9.651463s actual interval instead of the 12s resource deadline.
#[test]
fn hotori_katana_finisher_releases_paused_enemy_schedules_at_its_hit() {
    let mut req = super::hotori_actions::request();
    req.relative_duration_us = 66_000_000;
    req.actions = chain_actions();
    req.incoming_damage_events = vec![
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 42_000_000,
            actor_index: 0,
            delivered_damage: 100,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 50_000_000,
            actor_index: 0,
            delivered_damage: 100,
        },
    ];
    let id = crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let receipts = report["incoming_damage_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 2);
    // Commitment at36s freezes the six seconds remaining to the42s input.
    // Setup plus active stop is11.834796s; both inputs shift by that union.
    assert_eq!(receipts[0]["time_us"], 53_834_796);
    assert_eq!(receipts[1]["time_us"], 61_834_796);
    // The resource deadline is consumed without ending the already-finished
    // mode again.
    assert_eq!(
        trace_times(&report, "hotori_time_stop_finisher"),
        vec![47_834_796]
    );
    assert!(trace_times(&report, "hotori_time_stop_expired").is_empty());
}

#[test]
fn hotori_katana_without_breakthrough_four_uses_ordinary_finisher() {
    let mut req = super::hotori_actions::request();
    req.relative_duration_us = 55_000_000;
    req.actors[0].character_level = 50;
    req.actors[0].purchased_qte_level = Some(1);
    req.actors[0].character_max_level = Some(50);
    req.targets[0].defense = 0;
    req.actions = chain_actions();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 37);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(
        hits[36]["damage_effect_id"],
        "GE_Player_Jin_UltraSkillMelee6_Damage"
    );
    assert_eq!(hits[36]["non_crit_damage"], json!("3000/1"));
}

#[test]
fn hotori_katana_expiry_finishes_admitted_stage_without_live_bonuses() {
    let mut req = super::hotori_actions::request();
    req.relative_duration_us = 36_000_000;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(2);
    req.actions = vec![
        burst(20_000_000),
        basic(30_000_000),
        basic(33_190_000),
        basic(33_700_000),
        basic(35_200_000),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 18);
    let hits = report["hit_receipts"].as_array().unwrap();

    // Stage 3 delivers three Melee3 rows before the 34.183333s deadline with
    // the A2 Time Stop bonus and Melee3_1 after it without the bonus.
    let stage3 = &hits[13..17];
    assert_eq!(
        stage3[0]["damage_effect_id"],
        "GE_Player_Jin_UltraSkillMelee3_Damage"
    );
    for hit in &stage3[..3] {
        assert_eq!(hit["non_crit_damage"], json!("276/5"));
    }
    assert_eq!(
        stage3[3]["damage_effect_id"],
        "GE_Player_Jin_UltraSkillMelee3_1_Damage"
    );
    assert_eq!(stage3[3]["time_us"], 34_249_835);
    assert_eq!(stage3[3]["non_crit_damage"], json!("445/2"));
    assert_eq!(
        trace_times(&report, "hotori_time_stop_expired"),
        vec![34_183_333]
    );
    assert!(trace_times(&report, "hotori_time_stop_finisher").is_empty());
    assert_eq!(trace_times(&report, "hotori_katana_count").len(), 3);

    // No subsequent Katana input exists after the expiry: the next Basic is an
    // ordinary Entry-stage attack.
    assert_eq!(hits[17]["damage_effect_id"], "GE_Player_Jin_Melee1_Damage");
    assert_eq!(hits[17]["time_us"], 35_200_100);
    assert_eq!(hits[17]["non_crit_damage"], json!("100/1"));
}

#[test]
fn hotori_katana_finisher_recovery_switch_and_next_basic_boundaries() {
    let mut req = super::hotori_actions::request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.relative_duration_us = 55_000_000;
    req.actions = chain_actions();
    req.actions.push(basic(48_513_663));
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 48_537_040,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 38);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits[37]["damage_effect_id"], "GE_Player_Jin_Melee1_Damage");
    assert_eq!(hits[37]["time_us"], 48_513_763);

    // The finisher's 2.160101s switch lock spans its 2.136724s Any recovery.
    let mut locked = req.clone();
    *locked.actions.last_mut().unwrap() = GameActionCommand::SwitchActor {
        timestamp_us: 48_537_039,
        target_actor_index: 1,
        cooldown_us: 0,
    };
    let rejected = run_game_scenario(&locked, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(
        rejected["rejected_action"]["reason"]["until_us"],
        48_537_040
    );

    // The Any recovery itself admits the next Basic exactly at its end.
    let mut early = req.clone();
    early.actions.pop();
    early.actions.pop();
    early.actions.push(basic(48_513_662));
    let rejected = run_game_scenario(&early, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["execution_completed"], false);
    assert_eq!(
        rejected["rejected_action"]["reason"]["until_us"],
        48_513_663
    );
}

#[test]
fn hotori_katana_direct_packet_requires_an_active_time_stop() {
    let mut req = super::hotori_actions::request();
    req.actions = vec![action(
        0,
        "GA_Jin_Melee",
        "GE_Player_Jin_UltraSkillMelee1_Damage",
        0,
    )];
    let err = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(
        err.contains("Hotori Katana inputs require an active World's Tide Time Stop"),
        "unexpected error: {err}"
    );
}

#[test]
fn hotori_katana_strict_mode_accepts_the_chain_hypothesis() {
    let mut req = super::hotori_actions::request();
    req.actors[0].talent_levels = None;
    req.actors[0].purchased_skill_level = 1;
    req.model_assumptions.insert(
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID.into(),
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    for model in [
        crate::characters::hotori::timepiece::MODEL,
        crate::characters::hotori::timepiece::CLOCK_MODEL,
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID,
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID,
        crate::characters::hotori::katana::MODEL,
    ] {
        req.model_assumptions
            .insert(model.as_str().into(), model.description().into());
    }
    req.relative_duration_us = 55_000_000;
    req.actions = chain_actions();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 37);
}
