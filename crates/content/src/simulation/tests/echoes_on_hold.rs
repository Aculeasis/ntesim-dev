use super::*;
use crate::simulation::metadata::InputActionRole;

fn echoes_fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.seed = 42;
    req.stochastic_crit = None;
    req.relative_duration_us = 45_000_000;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_elyms".into();
    actor.gear_procs = vec![GearProcConfig::EchoesOnHold {
        atk_bonus: Ratio {
            numerator: 15,
            denominator: 100,
        },
        team_def_ignore: Ratio {
            numerator: 12,
            denominator: 100,
        },
        wearer_def_ignore: Ratio {
            numerator: 10,
            denominator: 100,
        },
        cosmos_penetration: Ratio {
            numerator: 8,
            denominator: 100,
        },
        duration_us: 20_000_000,
    }];
    actor.initial_energy = 120;
    actor.energy_capacity = 120;
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.crit_damage_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio {
        numerator: 20,
        denominator: 100,
    };
    req.targets[0].defense = 1080;
    req.actions.clear();
    req.action_metadata.clear();
    let id = crate::characters::haniel::hootie::HYPOTHESIS;
    req.model_assumptions
        .insert(id.as_str().to_string(), id.description().to_string());
    req
}

fn append_action(
    req: &mut GameScenarioRequest,
    actor: usize,
    time: u64,
    role: InputActionRole,
    element: &str,
    damage_effect_id: &str,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: "caller-test".into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let (damage_source_category, damage_type_ex, fixed_crit_rate, packet_ga_name_binding) =
        if damage_effect_id == "caller-effect" {
            (
                Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A".into()),
                Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}")),
                Some(crate::simulation::ResearchFixedCritRate::NoOverride),
                None,
            )
        } else {
            let ga_name = match damage_effect_id {
                "GE_Player_Haniel_Skill2_Damage" => Some("GA_Haniel_Skill".into()),
                "GE_Player_Haniel_Ultraskill2_Damage"
                | "GE_Player_Haniel_UltraSkill_Extra_Damage" => Some("GA_Haniel_UltraSkill".into()),
                _ => None,
            };
            (None, None, None, ga_name)
        };
    req.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: "test.synthetic_action".into(),
            ability_id: "caller-test".into(),
            damage_effect_id: damage_effect_id.into(),
            action_gameplay_tags: Some(vec!["Ability.Melee".into()]),
            input_action_role: Some(role),
            damage_type_ex,
            damage_source_category,
            fixed_crit_rate,
            unbal_value: None,
            packet_ga_name_binding,
        },
    );
}

#[test]
fn echoes_on_hold_combat_start_independent_windows_and_expiry() {
    let mut req = echoes_fixture();
    // Hit at 5s (Cosmos): receives 22% DEF ignore and 8% Cosmos penetration.
    append_action(
        &mut req,
        0,
        5_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );
    // Hit at 5s (Nature): receives 22% DEF ignore, but NOT Cosmos penetration.
    append_action(
        &mut req,
        0,
        5_000_000,
        InputActionRole::Normal,
        "NATURE",
        "caller-effect",
        0,
    );
    // Hit at 19.99s: both windows still active.
    append_action(
        &mut req,
        0,
        19_999_999,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );
    // Hit at 20.0s: both windows expired at their world-time deadline before damage.
    append_action(
        &mut req,
        0,
        20_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);

    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 4);

    let cosmos_hit_5s = hits[0]["damage"].as_str().unwrap();
    let nature_hit_5s = hits[1]["damage"].as_str().unwrap();
    let cosmos_hit_active = hits[2]["damage"].as_str().unwrap();
    let cosmos_hit_expired = hits[3]["damage"].as_str().unwrap();

    // Cosmos hit benefits from 8% Cosmos penetration, Nature hit does not:
    // With 20% base target resistance, Cosmos effective resistance is 12% (0.88 factor),
    // Nature effective resistance is 20% (0.80 factor).
    assert_ne!(cosmos_hit_5s, nature_hit_5s);
    let cosmos_dmg: f64 = cosmos_hit_5s
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / cosmos_hit_5s
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    let nature_dmg: f64 = nature_hit_5s
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / nature_hit_5s
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    assert!((cosmos_dmg / nature_dmg - 0.88 / 0.80).abs() < 1e-4);

    // Hit at 19.999999s has same damage as 5s hit (both windows active).
    assert_eq!(cosmos_hit_5s, cosmos_hit_active);

    // Hit at 20.0s has expired (no DEF ignore, no Cosmos penetration).
    let expired_dmg: f64 = cosmos_hit_expired
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / cosmos_hit_expired
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    assert!(expired_dmg < cosmos_dmg);
}

#[test]
fn echoes_on_hold_redirect_refreshes_cosmos_pen_and_leaves_def_ignore_unchanged() {
    let mut req = echoes_fixture();
    req.relative_duration_us = 45_000_001;
    append_action(
        &mut req,
        0,
        24_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );
    // The first damage from Redirect uses the refreshed penetration.
    append_action(
        &mut req,
        0,
        25_000_000,
        InputActionRole::RedirectSkill,
        "COSMOS",
        "caller-effect",
        0,
    );
    // Nature still uses the original resistance and expired DEF Ignore.
    append_action(
        &mut req,
        0,
        25_000_000,
        InputActionRole::Normal,
        "NATURE",
        "caller-effect",
        0,
    );
    append_action(
        &mut req,
        0,
        45_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);

    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 4);

    // Hit at 25s Cosmos vs Nature: Cosmos gets penetration (0.88 factor), Nature does not (0.80 factor).
    let cosmos_25s = hits[1]["damage"].as_str().unwrap();
    let nature_25s = hits[2]["damage"].as_str().unwrap();
    let cosmos_25s_val: f64 = cosmos_25s
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / cosmos_25s
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    let nature_25s_val: f64 = nature_25s
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / nature_25s
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    assert!((cosmos_25s_val / nature_25s_val - 0.88 / 0.80).abs() < 1e-4);

    assert_eq!(hits[0]["damage"], hits[3]["damage"]);
    assert_eq!(hits[2]["damage"], hits[3]["damage"]);
}

#[test]
fn echoes_on_hold_post_ultimate_followup_counting_and_refresh() {
    let mut req = echoes_fixture();

    // At 10s: Ultimate cast arms the Follow-up counter.
    append_action(
        &mut req,
        0,
        10_000_000,
        InputActionRole::Ultimate,
        "COSMOS",
        "caller-effect",
        0,
    );

    // Packet 1 at 12s: GE_Player_Haniel_Skill2_Damage (tagged State.Damage.Followup).
    append_action(
        &mut req,
        0,
        12_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "GE_Player_Haniel_Skill2_Damage",
        0,
    );

    // Packet 2 at 14s: GE_Player_Haniel_Ultraskill2_Damage (tagged State.Damage.Followup).
    append_action(
        &mut req,
        0,
        14_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "GE_Player_Haniel_Ultraskill2_Damage",
        0,
    );

    // Untagged damage at 16s: does NOT advance the counter.
    append_action(
        &mut req,
        0,
        16_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );

    // Packet 3 at 18s: GE_Player_Haniel_UltraSkill_Extra_Damage (tagged State.Damage.Followup).
    // On the 3rd qualifying packet, DEF ignore window refreshes for 20s (until 38s).
    append_action(
        &mut req,
        0,
        18_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "GE_Player_Haniel_UltraSkill_Extra_Damage",
        0,
    );

    // At 25s: Original combat-start window expired at 20s, but refreshed DEF ignore is active until 38s!
    append_action(
        &mut req,
        0,
        25_000_000,
        InputActionRole::Normal,
        "NATURE",
        "caller-effect",
        0,
    );

    // At 38s: Refreshed DEF ignore window expires.
    append_action(
        &mut req,
        0,
        38_000_000,
        InputActionRole::Normal,
        "NATURE",
        "caller-effect",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);
    let refreshes: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "echoes_on_hold_def_ignore")
        .map(|event| event["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(refreshes, [0, 18_000_000]);

    let hits = report["hit_receipts"].as_array().unwrap();
    // The hit at 25s (hits index for the 25s action) has DEF ignore active!
    // The hit at 38s has DEF ignore expired!
    let hit_25s = hits
        .iter()
        .find(|h| h["time_us"] == 25_000_000 && h["target_index"] == 0)
        .unwrap();
    let hit_38s = hits
        .iter()
        .find(|h| h["time_us"] == 38_000_000 && h["target_index"] == 0)
        .unwrap();

    let dmg_25s: f64 = hit_25s["damage"]
        .as_str()
        .unwrap()
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / hit_25s["damage"]
            .as_str()
            .unwrap()
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    let dmg_38s: f64 = hit_38s["damage"]
        .as_str()
        .unwrap()
        .split('/')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap()
        / hit_38s["damage"]
            .as_str()
            .unwrap()
            .split('/')
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
    assert!(dmg_25s > dmg_38s);
}

#[test]
fn echoes_on_hold_replacement_ultimate_resets_counter() {
    let mut req = echoes_fixture();
    // Ultimate 1 at 5s arms counter.
    append_action(
        &mut req,
        0,
        5_000_000,
        InputActionRole::Ultimate,
        "COSMOS",
        "caller-effect",
        0,
    );
    // Packet 1 at 6s.
    append_action(
        &mut req,
        0,
        6_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "GE_Player_Haniel_Skill2_Damage",
        0,
    );
    // Packet 2 at 7s.
    append_action(
        &mut req,
        0,
        7_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "GE_Player_Haniel_Skill2_Damage",
        0,
    );

    // Replacement Ultimate at 8s resets counter to 0!
    append_action(
        &mut req,
        0,
        8_000_000,
        InputActionRole::Ultimate,
        "COSMOS",
        "caller-effect",
        0,
    );
    // Packet 1 after replacement at 9s (counter becomes 1, NOT 3).
    append_action(
        &mut req,
        0,
        9_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "GE_Player_Haniel_Skill2_Damage",
        0,
    );

    // At 25s: start DEF ignore expired at 20s. Since counter only reached 1, NO refresh occurred!
    append_action(
        &mut req,
        0,
        25_000_000,
        InputActionRole::Normal,
        "NATURE",
        "caller-effect",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);

    let trace = report["trace"].as_array().unwrap();
    // Only 1 echoes_on_hold_def_ignore receipt occurred (at combat start t=0).
    let def_ignore_receipts = trace
        .iter()
        .filter(|r| r["event_kind"] == "echoes_on_hold_def_ignore")
        .count();
    assert_eq!(def_ignore_receipts, 1);
}

#[test]
fn echoes_on_hold_competing_copies_share_windows_without_stacking() {
    let mut req = echoes_fixture();
    req.relative_duration_us = 10_000_000;
    // Add actor 1 also equipped with Echoes on Hold.
    let mut actor1 = req.actors[0].clone();
    actor1.slot = 1;
    req.actors.push(actor1);

    append_action(
        &mut req,
        0,
        5_000_000,
        InputActionRole::Normal,
        "COSMOS",
        "caller-effect",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], true);

    let buffs = super::support::final_buffs(&report);
    for id in [
        "echoes_on_hold.team_def_ignore",
        "echoes_on_hold.wearer_def_ignore",
        "echoes_on_hold.cosmos_penetration",
    ] {
        let owners: Vec<_> = buffs
            .iter()
            .filter(|buff| buff["id"] == id)
            .map(|buff| buff["owner_actor"].as_u64().unwrap())
            .collect();
        assert_eq!(owners, [1], "{id}");
    }

    // The first wearer loses the extra DEF Ignore when the second copy takes over.
    let mut single_req = echoes_fixture();
    single_req.actions = req.actions.clone();
    single_req.action_metadata = req.action_metadata.clone();
    let single_report = run_game_scenario(&single_req, &ExecutionLimits::default()).unwrap();
    assert_ne!(
        report["hit_receipts"][0]["damage"],
        single_report["hit_receipts"][0]["damage"]
    );
}

#[test]
fn echoes_on_hold_counts_native_off_field_packets_once_per_fanout() {
    use crate::characters::haniel::hootie::{ENSEMBLE, EXTRA, SYMPHONY};

    let source = "
        options iteration=1 duration=35 crit=fixed;
        haniel char lvl=80/80 awakening=4 talent=1,8,10 qte=8 start_energy=120;
        haniel add weapon=fork_elyms refine=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0 boss=false;
        target hp=100000000 def=0 resist=0 boss=false;
        active haniel;
        haniel skill; haniel burst; mint swap;
        mint attack; mint attack; mint attack;
        mint attack; mint attack; mint attack;
    ";
    let result = crate::simulation::PreparedSourceScenario::new(
        source,
        Default::default(),
        &ExecutionLimits::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    let report = serde_json::to_value(result.report).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let follow_ups: Vec<_> = hits
        .iter()
        .filter(|hit| {
            matches!(
                hit["damage_effect_id"].as_str(),
                Some(ENSEMBLE | SYMPHONY | EXTRA)
            )
        })
        .collect();
    assert_eq!(follow_ups[0]["damage_effect_id"], ENSEMBLE);
    assert_eq!(follow_ups[1]["damage_effect_id"], ENSEMBLE);
    assert_eq!(follow_ups[2]["damage_effect_id"], SYMPHONY);
    assert_eq!(follow_ups[3]["damage_effect_id"], SYMPHONY);
    assert_eq!(follow_ups[4]["damage_effect_id"], EXTRA);
    assert_eq!(follow_ups[0]["time_us"], follow_ups[3]["time_us"]);
    assert_ne!(follow_ups[0]["target_index"], follow_ups[1]["target_index"]);
    assert_ne!(follow_ups[2]["target_index"], follow_ups[3]["target_index"]);
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "echoes_on_hold_def_ignore")
        .collect();
    assert_eq!(grants.len(), 2);
    assert_eq!(grants[1]["time_us"], follow_ups[4]["time_us"]);
    assert_eq!(grants[1]["from_actor"], 0);
    assert_eq!(grants[1]["active_actor"], 1);
    assert_eq!(grants[1]["phase"], "ProcCallback");
}

#[test]
fn echoes_on_hold_penetration_applies_to_cosmos_break_contribution() {
    let mut req = echoes_fixture();
    req.relative_duration_us = 1;
    let mut teammate = req.actors[0].clone();
    teammate.slot = 1;
    teammate.character_id = crate::registry::get_character_descriptor("zero")
        .unwrap()
        .character_id
        .into();
    teammate.weapon_id.clear();
    teammate.gear_procs.clear();
    req.actors.push(teammate);
    req.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio::one(),
        current_gauge: Ratio::one(),
        broken_duration_us: 100,
        discord_fraction: None,
    });
    append_action(
        &mut req,
        0,
        0,
        InputActionRole::Normal,
        "NATURE",
        "caller-effect",
        0,
    );
    req.action_metadata.get_mut(&0).unwrap().unbal_value = Some(Ratio::one());

    let break_damage = |request: &GameScenarioRequest| {
        let report = run_game_scenario(request, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true);
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit.get("break_contribution").is_some())
            .map(|hit| serde_json::from_value::<ExactFraction>(hit["damage"].clone()).unwrap())
            .collect::<Vec<_>>()
    };
    let t1 = break_damage(&req);
    let GearProcConfig::EchoesOnHold {
        cosmos_penetration, ..
    } = &mut req.actors[0].gear_procs[0]
    else {
        unreachable!()
    };
    *cosmos_penetration = Ratio {
        numerator: 1,
        denominator: 5,
    };
    let higher_penetration = break_damage(&req);
    assert_eq!(t1.len(), 2);
    assert_eq!(higher_penetration.len(), 2);
    assert_eq!(t1[0], higher_penetration[0]);
    // Only Cosmos sees resistance move from .12 to 0.
    assert_eq!(
        t1[1],
        higher_penetration[1]
            .mul(ExactFraction::new(22, 25).unwrap())
            .unwrap()
    );
}
