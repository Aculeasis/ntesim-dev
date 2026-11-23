use super::*;
use crate::simulation::tests::support::formula_damage_total;

fn scenario(periods: u64) -> GameScenarioRequest {
    let mut req = phase2_request();
    req.actions.clear();
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].initial_hp = 22204;
    req.actors[0].initial_hp_max = 22204;
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].hp_max = 24_000_000;
    req.targets[0].hp_current = 24_000_000;
    req.relative_duration_us = periods * 60_000_000;
    for period in 0..periods {
        let base = period * 60_000_000;
        let mut cursor = base;
        for _ in 0..6 {
            chain(&mut req, &mut cursor);
        }
        wait(&mut req, cursor, base + 35_000_000);
        req.actions.push(make_action(
            base + 35_000_000,
            0,
            "GA_Cang_Skill",
            "GE_Player_Cang_Skill_Damage",
            0,
            14_000_000,
        ));
        wait(&mut req, base + 35_961_682, base + 37_000_000);
        req.actions.push(burst(base + 37_000_000));
        wait(&mut req, base + 39_871_112, base + 41_000_000);
        cursor = base + 41_000_000;
        chain(&mut req, &mut cursor);
        wait(&mut req, cursor, base + 60_000_000);
    }
    req
}

fn chain(req: &mut GameScenarioRequest, time: &mut u64) {
    for recovery in [923724, 840277, 1218687, 1706341, 820279] {
        req.actions.push(normal(*time));
        *time += recovery;
    }
}

fn wait(req: &mut GameScenarioRequest, start: u64, end: u64) {
    req.actions.push(GameActionCommand::Wait {
        timestamp_us: start,
        duration_us: end - start,
    });
}

#[test]
fn two_cycles_keep_hp_words_energy_cycle_carry_and_real_field_clocks() {
    let first = run(&scenario(1));
    assert_eq!(first["hits_settled"], 97);
    assert_eq!(first["final_actor_states"][0]["hp"], 20223);
    let request = scenario(2);
    let report = run(&request);
    let actor = &report["final_actor_states"][0];
    // The second cinematic preserves the third retained mark: one Bless,
    // two missiles and one 5%-max-HP heal, followed by current-HP costs.
    assert_eq!(report["hits_settled"], 194 + 3);
    assert_eq!(report["relative_duration_us"], 120_000_000);
    assert_eq!(report["pending_events"], 0);
    assert_eq!(actor["hp"], 19352);
    assert_eq!(actor["energy"], 23);
    assert_eq!(actor["granted_energy"], 320);
    assert_eq!(actor["spent_energy"], 240);
    assert_eq!(actor["overflow_energy"], 57);
    assert_eq!(actor["charge_remainder"], json!("1/125"));
    assert_eq!(actor["energy_conserved"], true);
    assert_eq!(
        actor["cycle"],
        json!({"initial":0,"current":100,"capacity":100,"granted":534,"spent":0,"overflow":434,"carry":"51/125","conserved":true})
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    let coefficient = hits
        .iter()
        .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(
                ExactFraction::from_ratio(
                    serde_json::from_value(hit["scaling_rate"].clone()).unwrap(),
                )
                .unwrap(),
            )
        })
        .unwrap();
    // Additional .514 + 2*.257 ATK from the retained Bless and its missiles.
    assert_eq!(coefficient, ExactFraction::new(73609 + 514, 500).unwrap());
    // Controlled ATK1000+B4 200, half DEF, no bonuses or sampled crit.
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("444738/5")
    );
    let blessings = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Cang_WorldC_Damage")
        .collect::<Vec<_>>();
    assert_eq!(blessings.len(), 9);
    let fresh_words = blessings
        .iter()
        .filter(|hit| hit.get("autonomous_event_index").is_some())
        .collect::<Vec<_>>();
    assert!(
        !fresh_words.is_empty(),
        "continuing domains generate fresh Blesses"
    );
    for hit in fresh_words {
        assert!(hit.get("command_index").is_none());
        let event =
            &report["autonomous_events"][hit["autonomous_event_index"].as_u64().unwrap() as usize];
        assert_eq!(event["ability_id"], "GA_Cang_UltraSkill");
        assert_eq!(event["trigger"]["kind"], "action");
        let command = event["trigger"]["command_index"].as_u64().unwrap() as usize;
        assert!(
            matches!(&request.actions[command], GameActionCommand::InvokeAbility { ability_id, .. } if ability_id == "GA_Cang_Melee")
        );
        assert!(event["time_us"].as_u64().unwrap() < hit["time_us"].as_u64().unwrap());
    }
    assert_eq!(
        blessings
            .iter()
            .filter(|h| h["time_us"] == 100_664_744)
            .count(),
        3
    ); // All three marks retain their World-clock remainder through the cinematic.
    let ticks = hits
        .iter()
        .filter(|h| h["damage_effect_id"] == "GE_Player_Cang_UltraSkill_Damage")
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        ticks,
        [
            41_854_959,
            43_854_959,
            45_854_959,
            47_854_959,
            49_854_959,
            51_854_959,
            101_854_959,
            103_854_959,
            105_854_959,
            107_854_959,
            109_854_959,
            111_854_959
        ]
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["event_kind"] == "baicang_word_heal")
            .count(),
        9
    );
}
