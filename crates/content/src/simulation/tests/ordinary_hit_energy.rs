use super::*;
use serde_json::{Value, json};

struct Case {
    owner: &'static str,
    source: &'static str,
    melee: (&'static str, &'static str, &'static [u64]),
    skill: Option<(&'static str, &'static str, &'static [u64])>,
}

const EFFICIENCIES: [(u64, u64); 3] = [(28, 25), (3, 2), (7, 5)];

fn with_efficiencies(mut req: GameScenarioRequest) -> GameScenarioRequest {
    for (actor, (numerator, denominator)) in req.actors.iter_mut().zip(EFFICIENCIES) {
        actor.gear_procs.push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator,
                denominator,
            },
        });
    }
    req
}

const CASES: &[Case] = &[
    Case {
        owner: "character:1076",
        source: "shinku_hit",
        melee: (
            "GA_Shinku_Melee",
            "GE_Player_Shinku_Melee1_1_Damage",
            &[760],
        ),
        skill: Some((
            "GA_Shinku_Skill",
            "GE_Player_Shinku_Skill1_1_Damage",
            &[750, 1000, 3450],
        )),
    },
    Case {
        owner: "character:1004",
        source: "lacrimosa_hit",
        melee: (
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
            &[1428],
        ),
        skill: Some((
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
            &[6400],
        )),
    },
    Case {
        owner: "character:1033",
        source: "adler_hit",
        melee: ("GA_Adler_Melee", "GE_Player_Adler_Melee1_Damage", &[1254]),
        skill: Some(("GA_Adler_Skill", "GE_Player_Adler_Skill2_Damage", &[4000])),
    },
    Case {
        owner: "character:1019",
        source: "mint_hit",
        melee: (
            "GA_Mint019_Melee",
            "GE_Player_Mint_Melee1_Damage_New",
            &[578, 1283],
        ),
        skill: Some((
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
            &[2450; 2],
        )),
    },
    Case {
        owner: "character:1003",
        source: "sakiri_hit",
        melee: ("GA_Sagiri_Melee", "GE_Player_Sagiri_Melee1_Damage", &[2014]),
        skill: Some((
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            &[3902, 5398],
        )),
    },
    Case {
        owner: "character:1046",
        source: "zero_hit",
        melee: (
            "GA_Female046_Melee",
            "GE_Player_Female046_Melee1_Damage",
            &[1151],
        ),
        skill: Some((
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            &[587, 587, 3683, 3944],
        )),
    },
    Case {
        owner: "character:1075",
        source: "iroi_hit",
        melee: (
            "GA_Oneiroi_Melee",
            "GE_Player_Oneiroi_Melee1_Damage",
            &[2431],
        ),
        skill: Some((
            "GA_Oneiroi_Skill",
            "GE_Player_Oneiroi_Skill_Gun_Damage",
            &[300, 614, 614, 614, 614, 614, 614, 614, 200],
        )),
    },
    Case {
        owner: "character:1010",
        source: "nanally_hit",
        melee: (
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            &[1138],
        ),
        skill: Some((
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            &[1260; 5],
        )),
    },
    Case {
        owner: "character:1071",
        source: "chaos_hit",
        melee: ("GA_Chaos071_Melee", "GE_Player_Chaos_Melee1_Damage", &[913]),
        skill: Some((
            "GA_Chaos071_Skill",
            "GE_Player_Chaos_Skill1_Damage",
            &[2006, 5294],
        )),
    },
    Case {
        owner: "character:1021",
        source: "edgar_hit",
        melee: ("GA_Edgar_Melee", "GE_Player_Edgar_Melee1_Damage", &[1931]),
        skill: Some(("GA_Edgar_Skill", "GE_Player_Edgar_Skill_Damage", &[3200; 7])),
    },
    Case {
        owner: "character:1052",
        source: "hotori_hit",
        melee: ("GA_Jin_Melee", "GE_Player_Jin_Melee1_Damage", &[1251]),
        skill: Some((
            "GA_Jin_Skill",
            "GE_Player_Jin_Skill1_Damage",
            &[946, 947, 946, 947, 947, 947, 1421],
        )),
    },
    Case {
        owner: "character:1054",
        source: "daffodil_hit",
        melee: (
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Melee1_Damage",
            &[1041],
        ),
        skill: Some((
            "GA_Daffodill_Skill",
            "GE_Player_Daffodill_Skill1_Damage",
            &[831, 831, 685, 685, 685, 685, 685, 1012],
        )),
    },
    Case {
        owner: "character:1072",
        source: "linko_hit",
        melee: (
            "GA_Radio072_Melee",
            "GE_Player_Radio072_Melee1_Damage",
            &[3105],
        ),
        skill: Some((
            "GA_Radio072_Skill",
            "GE_Player_Radio072_Skill1_Damage",
            &[1493, 1162, 1996, 3749],
        )),
    },
    Case {
        owner: "character:1008",
        source: "skia_hit",
        melee: ("GA_Skia_Melee", "GE_Player_Skia_Melee1_Damage", &[2525]),
        skill: Some((
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            &[1080, 1080, 1080, 1080, 166, 914],
        )),
    },
    Case {
        owner: "character:1025",
        source: "hathor_hit",
        melee: ("GA_Hathor_Melee", "GE_Player_Hathor_Melee1_Damage", &[1219]),
        skill: Some((
            "GA_Hathor_Skill",
            "GE_Player_Hathor_Skill1_Damage",
            &[2833, 2833, 2833],
        )),
    },
    Case {
        owner: "character:1042",
        source: "blackbird_hit",
        melee: (
            "GA_BlackBird_Melee",
            "GE_Player_BlackBird_Melee1_Damage",
            &[838],
        ),
        skill: Some((
            "GA_BlackBird_Skill",
            "GE_Player_BlackBird_Skill_Damage",
            &[1890, 1890, 1890, 1890],
        )),
    },
    Case {
        owner: "character:1070",
        source: "aurelia_hit",
        melee: (
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_Melee1_Damage",
            &[2168],
        ),
        skill: Some((
            "GA_Mitsuki_Skill",
            "GE_Player_Mitsuki_Skill_Damage",
            &[6700],
        )),
    },
    Case {
        owner: "character:1020",
        source: "haniel_hit",
        melee: ("GA_Haniel_Melee", "GE_Player_Haniel_Melee1_Damage", &[2625]),
        skill: Some((
            "GA_Haniel_Skill",
            "GE_Player_Haniel_Skill1_Damage",
            &[1800, 1800],
        )),
    },
    Case {
        owner: "character:1055",
        source: "jiuyuan_hit",
        melee: ("GA_Kuhara_Melee", "GE_Player_Kuhara_Melee1_Damage", &[1365]),
        skill: Some((
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            &[852, 903, 903, 903, 903, 2636],
        )),
    },
    Case {
        owner: "character:1023",
        source: "baicang_hit",
        melee: ("GA_Cang_Melee", "GE_Player_Cang_Melee1_Damage", &[2257]),
        skill: Some(("GA_Cang_Skill", "GE_Player_Cang_Skill_Damage", &[3400])),
    },
    Case {
        owner: "character:1039",
        source: "fadia_hit",
        melee: ("GA_Fadia_Melee", "GE_Player_Fadia_Melee1_Damage", &[2707]),
        skill: None,
    },
    Case {
        owner: "character:1036",
        source: "zankou_hit",
        melee: (
            "GA_Zankou_Melee",
            "GE_Player_Zankou_Melee1_Damage",
            &[529, 915],
        ),
        skill: None,
    },
    Case {
        owner: "character:1073",
        source: "chiz_hit",
        melee: (
            "GA_Chiichan073_Melee",
            "GE_Player_Chiichan073_Melee1_Damage",
            &[1727],
        ),
        skill: None,
    },
    Case {
        owner: "character:1057",
        source: "akane_hit",
        melee: (
            "GA_Akane_Melee",
            "GE_Player_Akane_Melee1_Damage",
            &[1244, 1244],
        ),
        skill: Some(("GA_Akane_Skill", "GE_Player_Akane_Skill_Damage", &[4200])),
    },
    Case {
        owner: "character:1090",
        source: "elyms_hit",
        melee: (
            "GA_Elyms_Melee",
            "GE_Player_Elyms_Melee1_Damage",
            &[704, 704],
        ),
        skill: Some((
            "GA_Elyms_Skill",
            "GE_Player_Elyms_Skill1_Damage",
            &[808, 707, 707, 707, 707, 1464],
        )),
    },
];

fn action(ability: &str, effect: &str, delay: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: ability.into(),
        damage_effect_id: effect.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    }
}

fn mint_timed_action(
    timestamp_us: u64,
    ability: &str,
    effect: &str,
    energy_cost: u64,
    recovery_us: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    let mut command = action(ability, effect, 100);
    if let GameActionCommand::InvokeAbility {
        timestamp_us: command_time,
        energy_cost: command_cost,
        recovery_us: command_recovery,
        cooldown_us: command_cooldown,
        ..
    } = &mut command
    {
        *command_time = timestamp_us;
        *command_cost = energy_cost;
        *command_recovery = recovery_us;
        *command_cooldown = cooldown_us;
    }
    command
}

fn sakiri_timed_action(
    timestamp_us: u64,
    ability: &str,
    effect: &str,
    energy_cost: u64,
    recovery_us: u64,
    cooldown_us: u64,
) -> GameActionCommand {
    let mut command = action(ability, effect, 100);
    if let GameActionCommand::InvokeAbility {
        timestamp_us: command_time,
        energy_cost: command_cost,
        recovery_us: command_recovery,
        cooldown_us: command_cooldown,
        ..
    } = &mut command
    {
        *command_time = timestamp_us;
        *command_cost = energy_cost;
        *command_recovery = recovery_us;
        *command_cooldown = cooldown_us;
    }
    command
}

fn request(case: &Case, skill: bool) -> GameScenarioRequest {
    let mut req = sample_s1_request();
    let actor = &mut req.actors[0];
    actor.character_id = case.owner.into();
    actor.weapon_id.clear();
    actor.gear_procs.clear();
    actor.set_id = None;
    actor.talent_levels = Some([1; 3]);
    actor.character_max_level = Some(80);
    actor.initial_energy = 0;
    actor.energy_capacity = 100;
    actor.awakening = ntesim_data::AwakeningSelection::prefix(0);
    for (slot, id) in [(1, "character:1019"), (2, "character:1003")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = id.into();
        req.actors.push(ally);
    }
    req.targets[0].resistance = Ratio::zero();
    let (ability, effect) = if skill {
        {
            let skill = case.skill.expect("supported Skill fixture");
            (skill.0, skill.1)
        }
    } else {
        (case.melee.0, case.melee.1)
    };
    req.actions = vec![action(ability, effect, 0)];
    if case.owner == "character:1052" && skill {
        // Present Replay needs more than 60 Timepiece and its 5.167475s
        // rewind must settle within the default 20s horizon.
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
            *timestamp_us = 11_000_000;
        }
    }
    if case.owner == "character:1057" {
        for (k, v) in crate::characters::akane::source_assumptions() {
            req.model_assumptions.insert(k, v);
        }
    }
    if case.owner == "character:1090" {
        for (k, v) in crate::characters::elyms::source_assumptions() {
            req.model_assumptions.insert(k, v);
        }
    }
    if case.owner == "character:1033" && skill {
        req.relative_duration_us = 10_000_100;
        req.model_assumptions.insert(
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
                .as_str()
                .into(),
            crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
                .description()
                .into(),
        );
        req.model_assumptions.insert(
            crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID.into(),
            crate::characters::adler::skill::SKILL_TRAIN_HYPOTHESIS_ID
                .description()
                .into(),
        );
        req.model_assumptions.insert(
            crate::characters::adler::skill::KARMA_HYPOTHESIS_ID.into(),
            crate::characters::adler::skill::KARMA_HYPOTHESIS_ID
                .description()
                .into(),
        );
        req.model_assumptions.insert(
            crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID.into(),
            crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID
                .description()
                .into(),
        );
    }
    req
}

fn grants<'a>(report: &'a Value, case: &Case) -> Vec<&'a Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["energy_fanout"]["source"] == case.source
                || event["energy_change"]["source"] == case.source
        })
        .collect()
}

fn fraction(mut numerator: u64, mut denominator: u64) -> Value {
    let (mut a, mut b) = (numerator, denominator);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    numerator /= a;
    denominator /= a;
    json!(format!("{}/{}", numerator, denominator))
}

fn check_flow(
    case: &Case,
    req: &GameScenarioRequest,
    packets: &[u64],
    efficiencies: &[(u64, u64)],
) -> Value {
    check_flow_with_spend(case, req, packets, efficiencies, None)
}

fn check_flow_with_spend(
    case: &Case,
    req: &GameScenarioRequest,
    packets: &[u64],
    efficiencies: &[(u64, u64)],
    spend_before_packet: Option<(usize, u64, u64)>,
) -> Value {
    let report = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_flow_with_spend(
        case,
        req,
        &report,
        packets,
        efficiencies,
        spend_before_packet,
    );
    report
}

fn assert_flow_with_spend(
    case: &Case,
    req: &GameScenarioRequest,
    report: &Value,
    packets: &[u64],
    efficiencies: &[(u64, u64)],
    spend_before_packet: Option<(usize, u64, u64)>,
) {
    assert_eq!(report["execution_completed"], true, "{}", case.source);
    let events = grants(report, case);
    assert_eq!(events.len(), packets.len(), "{}", case.source);
    for (recipient, actor) in req.actors.iter().enumerate() {
        let (distribution_n, distribution_d) = if recipient == 0 { (1, 1) } else { (3, 5) };
        let (efficiency_n, efficiency_d) = efficiencies[recipient];
        let denominator = 1000 * distribution_d * efficiency_d * 5;
        let mut cumulative = 0;
        let mut granted = 0;
        let mut energy = actor.initial_energy;
        let mut overflow = 0;
        for (packet, base) in packets.iter().enumerate() {
            if recipient == 0
                && let Some((spend_packet, expected_energy, amount)) = spend_before_packet
                && packet == spend_packet
            {
                assert_eq!(energy, expected_energy, "{} pre-spend energy", case.source);
                energy -= amount;
            }
            let fanout = &events[packet]["energy_fanout"];
            assert_eq!(fanout["source"], case.source);
            assert_eq!(fanout["source_actor_index"], 0);
            assert_eq!(fanout["base"], fraction(*base, 1000));
            let recipients = fanout["recipients"].as_array().unwrap();
            assert_eq!(recipients.len(), req.actors.len());
            // The Lv80/80 Chiz fixture has B4: add .20 only while the
            // receiver is on field, including delivery after a real switch.
            let hoarding = recipient == 0
                && case.owner == "character:1073"
                && events[packet]["active_actor"] == 0;
            let scaled_increment = base
                * distribution_n
                * (efficiency_n * 5 + if hoarding { efficiency_d } else { 0 });
            let carry_before = cumulative % denominator;
            cumulative += scaled_increment;
            let next_granted = cumulative / denominator;
            let gross = next_granted - granted;
            let retained = gross.min(actor.energy_capacity - energy);
            let entry = &recipients[recipient];
            let mut expected = json!({
                "recipient_actor_index": recipient,
                "distribution": fraction(distribution_n, distribution_d),
                "energy_before": energy, "gross": gross, "retained": retained,
                "overflow": gross - retained, "energy_after": energy + retained,
                "charge_basis": fraction(*base * distribution_n, 1000 * distribution_d),
            });
            let expected_object = expected.as_object_mut().unwrap();
            if carry_before != 0 {
                expected_object.insert(
                    "charge_remainder_before".into(),
                    fraction(carry_before, denominator),
                );
            }
            if cumulative % denominator != 0 {
                expected_object.insert(
                    "charge_remainder_after".into(),
                    fraction(cumulative % denominator, denominator),
                );
            }
            assert_eq!(
                entry, &expected,
                "{} packet {packet} recipient {recipient}",
                case.source
            );
            let typed: crate::simulation::model::GameEnergyFanoutReceipt =
                serde_json::from_value(fanout.clone()).unwrap();
            assert_eq!(typed.grants().count(), req.actors.len());
            granted = next_granted;
            energy += retained;
            overflow += gross - retained;
        }
        let state = &report["final_actor_states"][recipient];
        assert_eq!(
            state["energy"], energy,
            "{} recipient {recipient}",
            case.source
        );
        assert_eq!(state["granted_energy"], granted);
        assert_eq!(state["overflow_energy"], overflow);
        if cumulative % denominator == 0 {
            assert!(state["charge_remainder"].is_null());
        } else {
            assert_eq!(
                state["charge_remainder"],
                fraction(cumulative % denominator, denominator)
            );
        }
    }
    let hypothesis = format!(
        "{}.ordinary_hit_energy_v1",
        case.source.trim_end_matches("_hit")
    );
    assert!(
        report["resolved_model_assumptions"]
            .get(hypothesis)
            .is_some()
    );
}

#[test]
fn ordinary_hit_energy_records_and_validates_conflicts() {
    for case in CASES {
        let mut req = request(case, case.skill.is_some());
        for actor in &mut req.actors {
            actor.talent_levels = None;
            actor.purchased_skill_level = 1;
        }
        let modeled = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        req.model_assumptions =
            serde_json::from_value(modeled["resolved_model_assumptions"].clone()).unwrap();
        let hypothesis = format!(
            "{}.ordinary_hit_energy_v1",
            case.source.trim_end_matches("_hit")
        );
        let description = req.model_assumptions.get(&hypothesis).cloned().unwrap();
        req.model_assumptions
            .insert(hypothesis.clone(), "conflicting description".into());
        let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
        assert!(
            error.contains("conflicting reached model hypothesis"),
            "{error}"
        );
        req.model_assumptions.insert(hypothesis, description);
        check_flow(case, &req, case.skill.unwrap_or(case.melee).2, &[(1, 1); 3]);
    }
}

#[test]
fn ordinary_melee_grants_keep_owner_distribution_exact_carry_and_replay() {
    for case in CASES {
        let req = request(case, false);
        let report = check_flow(case, &req, case.melee.2, &[(1, 1); 3]);
        assert_eq!(
            report,
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
        );
        check_flow(case, &with_efficiencies(req), case.melee.2, &EFFICIENCIES);
    }
}

#[test]
fn ordinary_skill_grants_each_native_packet_with_recipient_efficiency() {
    for case in CASES.iter().filter(|case| case.skill.is_some()) {
        let req = request(case, true);
        let packets = case.skill.unwrap().2;
        check_flow(case, &req, packets, &[(1, 1); 3]);
        check_flow(case, &with_efficiencies(req), packets, &EFFICIENCIES);
    }
}

#[test]
fn chaos_qte_direct_packet_energy_uses_its_own_policy_and_can_fund_burst() {
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1071")
        .unwrap();
    let mut req = with_efficiencies(request(case, false));
    req.actors[0].initial_energy = 93;
    let mut qte = action("GA_Chaos071_QTE", "GE_Player_Chaos_QTE1_Damage", 100_000);
    let mut burst = action(
        "GA_Chaos071_UltraSkill",
        "GE_Player_Chaos_UltraSkill1_Damage",
        100_000,
    );
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        recovery_us,
        cooldown_us,
        ..
    } = &mut qte
    {
        *timestamp_us = 0;
        *recovery_us = 500_000;
        *cooldown_us = 15_000_000;
    }
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        energy_cost,
        cooldown_us,
        ..
    } = &mut burst
    {
        *timestamp_us = 600_000;
        *energy_cost = 100;
        *cooldown_us = 20_000_000;
    }
    req.actions = vec![qte, burst];
    req.relative_duration_us = 2_000_000;

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][0]["damage_effect_id"],
        "GE_Player_Chaos_QTE1_Damage"
    );
    let events: Vec<&Value> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "chaos_qte_direct_packet_energy_grant")
        .collect();
    assert_eq!(events.len(), 1);
    let fanout = &events[0]["energy_fanout"];
    assert_eq!(fanout["source"], "chaos_hit");
    assert_eq!(fanout["base"], fraction(6400, 1000));
    let recipients = fanout["recipients"].as_array().unwrap();
    assert_eq!(recipients.len(), 3);
    for (recipient, expected_gross, distribution) in [
        (0, 7, fraction(1, 1)),
        (1, 5, fraction(3, 5)),
        (2, 5, fraction(3, 5)),
    ] {
        let entry = &recipients[recipient];
        assert_eq!(entry["recipient_actor_index"], recipient);
        assert_eq!(entry["gross"], expected_gross);
        assert_eq!(entry["distribution"], distribution);
    }
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(21, 125)
    );
    assert_eq!(report["final_actor_states"][1]["energy"], 5);
    assert_eq!(
        report["final_actor_states"][1]["charge_remainder"],
        fraction(19, 25)
    );
    assert_eq!(report["final_actor_states"][2]["energy"], 5);
    assert_eq!(
        report["final_actor_states"][2]["charge_remainder"],
        fraction(47, 125)
    );
    assert!(
        report["resolved_model_assumptions"]
            .get("chaos.qte_direct_packet_energy_v1")
            .is_some()
    );
    assert!(
        report["resolved_model_assumptions"]
            .get("chaos.ordinary_hit_energy_v1")
            .is_none()
    );

    let mut conflicting = req;
    conflicting.actions.truncate(1);
    for actor in &mut conflicting.actors {
        actor.talent_levels = None;
        actor.purchased_skill_level = 1;
        actor.gear_procs.clear();
    }
    conflicting.model_assumptions =
        serde_json::from_value(report["resolved_model_assumptions"].clone()).unwrap();
    conflicting.model_assumptions.insert(
        "chaos.qte_direct_packet_energy_v1".into(),
        "conflicting description".into(),
    );
    let error = run_game_scenario(&conflicting, &ExecutionLimits::default()).unwrap_err();
    assert!(
        error.contains("conflicting reached model hypothesis"),
        "{error}"
    );
}

#[test]
fn adler_skill_charge_add_uses_efficiency_carry_and_energy_spending() {
    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1033")
        .unwrap();
    let mut req = with_efficiencies(request(case, true));
    req.actors[0].initial_energy = 20;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 20;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let events = grants(&report, case);
    assert_eq!(events.len(), 1);
    let recipients = events[0]["energy_fanout"]["recipients"].as_array().unwrap();
    assert_eq!(recipients.len(), 3);
    assert_eq!(recipients[0]["gross"], 4);
    assert_eq!(recipients[1]["gross"], 3);
    assert_eq!(recipients[2]["gross"], 3);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 20);
    assert_eq!(report["final_actor_states"][0]["energy"], 4);
    assert_eq!(report["final_actor_states"][1]["energy"], 3);
    assert_eq!(report["final_actor_states"][2]["energy"], 3);
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn ordinary_grants_conserve_partial_headroom_and_full_pool_overflow() {
    for case in CASES {
        let mut req = request(case, case.skill.is_some());
        req.actors[0].initial_energy = 99;
        req.actors[1].initial_energy = 100;
        req.actors[2].initial_energy = 98;
        check_flow(case, &req, case.skill.unwrap_or(case.melee).2, &[(1, 1); 3]);
    }
}

#[test]
fn ordinary_grants_skip_dead_recipients_dead_targets_and_zero_hp_loss() {
    for case in CASES {
        let mut req = request(case, false);
        req.actors[1].initial_hp = 0;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let events = grants(&report, case);
        assert_eq!(events.len(), case.melee.2.len(), "{}", case.source);
        let recipients = events[0]["energy_fanout"]["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), 2, "{}", case.source);
        assert_eq!(recipients[0]["recipient_actor_index"], 0);
        assert_eq!(recipients[1]["recipient_actor_index"], 2);
        for dead in [true, false] {
            let mut req = request(case, false);
            if dead {
                req.targets[0].hp_current = 0;
            } else {
                req.actors[0].static_stats.atk = Ratio::zero();
            }
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert!(
                grants(&report, case).is_empty(),
                "{} dead={dead}",
                case.source
            );
            if dead {
                assert_eq!(report["hits_settled"], 0);
            }
        }
    }
}

#[test]
fn ordinary_delayed_grants_keep_source_after_switch_and_include_fatal_damage() {
    for case in CASES {
        let mut req = request(case, false);
        req.actions = vec![
            action(case.melee.0, case.melee.1, 1000),
            GameActionCommand::SwitchActor {
                timestamp_us: 500,
                target_actor_index: 1,
                cooldown_us: 0,
            },
        ];
        req.relative_duration_us = 2000;
        req.targets[0].hp_current = 1;
        if matches!(
            case.owner,
            "character:1010" | "character:1036" | "character:1057" | "character:1090"
        ) {
            // These finite Normals cannot switch before their owner recovery gate.
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(report["execution_completed"], false);
            assert!(
                report["halt_diagnostic"]
                    .as_str()
                    .unwrap()
                    .starts_with("ACTOR_LOCKED")
            );
            continue;
        }
        let report = check_flow(case, &req, &case.melee.2[..1], &[(1, 1); 3]);
        for event in grants(&report, case) {
            assert_eq!(event["active_actor"], 1);
            assert_eq!(event["energy_fanout"]["source_actor_index"], 0);
        }
        assert_eq!(report["hit_receipts"][0]["applied_hp_loss"], 1);
    }
}

#[test]
fn lacrimosa_melee_projectile_energy_retains_owner_after_switch() {
    let case = CASES
        .iter()
        .find(|case| case.source == "lacrimosa_hit")
        .unwrap();
    let mut req = with_efficiencies(request(case, false));
    req.relative_duration_us = 3_000_000;
    req.actions = vec![GameActionCommand::PlayerInput {
        timestamp_us: 0,
        actor_index: 0,
        input: PlayerInput::Special,
        critical_success: false,
    }];
    for index in 1..=3 {
        let mut command = action(case.melee.0, case.melee.1, 1000);
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut command {
            *timestamp_us = index * 800_000;
        }
        req.actions.push(command);
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_400_500,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    let report = check_flow(
        case,
        &req,
        &[4000, 2471, 893, 766, 766, 945, 567, 567, 567, 748],
        &EFFICIENCIES,
    );
    let last = grants(&report, case).last().unwrap().to_owned();
    assert_eq!(last["active_actor"], 1);
    assert_eq!(last["energy_fanout"]["source_actor_index"], 0);
    assert_eq!(
        report["hit_receipts"].as_array().unwrap().last().unwrap()["damage_effect_id"],
        "GE_Player_Lacrimosa_B_Melee3_Explode_Damage"
    );
}

#[test]
fn ordinary_basis_accepts_shared_receipts_and_rejects_obsolete_fields() {
    for case in CASES {
        let report = run_game_scenario(&request(case, false), &ExecutionLimits::default()).unwrap();
        let mut fanout_json = grants(&report, case)[0]["energy_fanout"].clone();
        let typed_fanout: crate::simulation::model::GameEnergyFanoutReceipt =
            serde_json::from_value(fanout_json.clone()).unwrap();
        let first_grant = typed_fanout.grants().next().unwrap();
        let mut receipt = serde_json::to_value(GameEnergyReceipt::Grant(first_grant)).unwrap();
        let obsolete_key = format!("{}_basis", case.source);
        let typed: GameEnergyReceipt = serde_json::from_value(receipt.clone()).unwrap();
        let canonical = serde_json::to_value(typed).unwrap();
        assert_eq!(canonical["hit_basis"], receipt["hit_basis"]);
        assert!(canonical.get(&obsolete_key).is_none());
        let object = receipt.as_object_mut().unwrap();
        let basis = object.remove("hit_basis").unwrap();
        object.insert(obsolete_key.clone(), basis.clone());
        assert!(serde_json::from_value::<GameEnergyReceipt>(receipt.clone()).is_err());
        receipt
            .as_object_mut()
            .unwrap()
            .insert("hit_basis".into(), basis.clone());
        assert!(serde_json::from_value::<GameEnergyReceipt>(receipt).is_err());
        fanout_json
            .as_object_mut()
            .unwrap()
            .insert(obsolete_key, basis);
        assert!(
            serde_json::from_value::<crate::simulation::model::GameEnergyFanoutReceipt>(
                fanout_json
            )
            .is_err()
        );
    }
}

mod continuing_rotations;

mod aurelia_continuation;

mod owner_rotations;

mod iroi_continuation;

mod jiuyuan_continuation;

mod zero_continuation;

mod haniel_continuation;

mod adler_continuation;

mod hotori_continuation;

mod fadia_continuation;
