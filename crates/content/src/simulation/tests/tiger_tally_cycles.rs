use super::*;
use serde_json::Value;

const SOURCE: &str = include_str!("../../../../../examples/tiger-tally-cycles.nte");

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

fn fraction(n: u128, d: u128) -> ExactFraction {
    ExactFraction::new(n, d).unwrap()
}

fn request() -> GameScenarioRequest {
    let executed = PreparedSourceScenario::new(SOURCE, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        executed.report.success && executed.report.execution_completed,
        "{:?}",
        executed.diagnostic
    );
    super::assert_native_replay(&executed);
    let mut req = executed.request;
    // Preserve legal native builds/procs, isolating the live base-ATK and bonus zones.
    for actor in &mut req.actors {
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.elemental_bonus = Ratio::zero();
        actor.static_stats.elemental_bonuses.clear();
    }
    req
}

fn grants(req: &GameScenarioRequest, burst_only: bool) -> Vec<u64> {
    req.actions
        .iter()
        .filter_map(|action| match action {
            GameActionCommand::InvokeAbility {
                actor_index: 0,
                ability_id,
                timestamp_us,
                ..
            } if ability_id == "GA_Hathor_UltraSkill"
                || !burst_only && ability_id == "GA_Hathor_Skill" =>
            {
                Some(*timestamp_us)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn tiger_tally_continuing_native_cycles_preserve_zones_independent_expiry_and_energy() {
    // Refinement changes the native damage/ATK zones, not these legal clocks or
    // resource grants. Reuse the source-validated timeline and bind each actual
    // refinement through the production projection instead of executing it twice.
    let timeline = request();
    for tier in [1, 5] {
        let mut original = timeline.clone();
        let actor = &mut original.actors[0];
        actor.weapon_tier = tier;
        let positions = actor
            .gear_procs
            .iter()
            .enumerate()
            .filter_map(|(index, proc)| {
                matches!(proc, GearProcConfig::TigerTallyAtk { .. }).then_some(index)
            })
            .collect::<Vec<_>>();
        crate::weapons::tiger_tally::projection::project(
            &BTreeMap::new(),
            1,
            &original.content_id,
            actor,
            &positions,
        )
        .unwrap();
        for boss in [false, true] {
            let mut req = original.clone();
            req.targets[0].is_boss = boss;
            // Isolate the two timed damage zones while retaining the same
            // permanent ATK, Power snapshots and native action/packet clocks.
            let mut control = req.clone();
            for proc in &mut control.actors[0].gear_procs {
                if let GearProcConfig::TigerTallyAtk {
                    normal_damage_bonus,
                    boss_damage_bonus,
                    ..
                } = proc
                {
                    *normal_damage_bonus = Ratio::zero();
                    *boss_damage_bonus = Ratio::zero();
                }
            }
            let plain = run_game_scenario(&control, &ExecutionLimits::default()).unwrap();
            let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{}",
                report["rejected_action"]
            );
            let mut boundary_hits = Vec::new();
            let mut burst_hits = 0;
            let starts = grants(&req, false);
            let bursts = grants(&req, true);
            assert_eq!(starts.len(), 4);
            assert_eq!(bursts.len(), 2);
            let boundaries = starts
                .iter()
                .map(|time| support::world_time_at(&report, *time) + 15_000_000)
                .chain(
                    bursts
                        .iter()
                        .map(|time| support::world_time_at(&report, *time) + 10_000_000),
                )
                .collect::<Vec<_>>();
            assert_eq!(
                report["hit_receipts"].as_array().unwrap().len(),
                plain["hit_receipts"].as_array().unwrap().len()
            );
            for (hit, reference) in report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .zip(plain["hit_receipts"].as_array().unwrap())
            {
                let effect = hit["damage_effect_id"].as_str().unwrap();
                let time = hit["time_us"].as_u64().unwrap();
                if hit["actor_index"] != 0 {
                    // Peer damage cannot receive either wearer-only zone.
                    let base = match effect {
                        "GE_Player_Mint_Melee1_Damage_New" => 104,
                        "GE_Player_Mint_Melee2_Damage_New" => 231,
                        "GE_Player_Mint_Melee3_Damage_New" => 170,
                        "GE_Player_Mint_Melee4_Damage_New" => 262,
                        "GE_Player_Mint_Melee5_Damage_New" => 341,
                        other => panic!("unexpected peer packet {other}"),
                    };
                    assert_eq!(exact(&hit["non_crit_damage"]), fraction(base, 1));
                    continue;
                }
                let normal = effect.contains("_Melee");
                let stacks = starts
                    .iter()
                    .filter(|start| {
                        support::world_time_at(&report, **start)
                            <= support::world_time_at(&report, time)
                            && support::world_time_at(&report, time)
                                < support::world_time_at(&report, **start) + 15_000_000
                    })
                    .count();
                let arc = boss
                    && bursts.iter().any(|start| {
                        support::world_time_at(&report, *start)
                            <= support::world_time_at(&report, time)
                            && support::world_time_at(&report, time)
                                < support::world_time_at(&report, *start) + 10_000_000
                    });
                let zone = fraction(5, 4)
                    .add(fraction(
                        u128::from(normal) * stacks as u128 * u128::from(tier + 3) * 3,
                        80,
                    ))
                    .unwrap()
                    .add(fraction(u128::from(arc) * u128::from(tier + 3), 40))
                    .unwrap();
                let expected = exact(&reference["non_crit_damage"])
                    .mul(zone.div(fraction(5, 4)).unwrap())
                    .unwrap();
                assert_eq!(
                    exact(&hit["non_crit_damage"]),
                    expected,
                    "T{tier} boss={boss} at {time} {effect}"
                );
                if boundaries.contains(&support::world_time_at(&report, time)) {
                    boundary_hits.push(support::world_time_at(&report, time));
                }
                burst_hits += usize::from(effect.contains("_UltraSkill"));
            }
            for boundary in boundaries {
                assert!(
                    boundary_hits.contains(&boundary),
                    "missing exact expiry {boundary}"
                );
            }
            for (period, start) in bursts.iter().enumerate() {
                let first = plain["hit_receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|hit| {
                        hit["damage_effect_id"] == "GE_Player_Hathor_Melee1_Damage"
                            && hit["time_us"].as_u64().unwrap() > *start
                    })
                    .unwrap();
                // .243 ATK, permanent tier ATK and actual Power2/4 snapshots.
                let atk = fraction(1, 1)
                    .add(fraction(u128::from(tier + 3) * 3, 80))
                    .unwrap()
                    .add(fraction((period as u128 + 1) * 3, 10))
                    .unwrap();
                assert_eq!(
                    exact(&first["non_crit_damage"]),
                    fraction(243, 1)
                        .mul(atk)
                        .unwrap()
                        .mul(fraction(5, 4))
                        .unwrap()
                );
            }
            assert_eq!(burst_hits, 6);
            assert_eq!(report["final_actor_states"][0]["spent_energy"], 200);
            assert_eq!(report["final_actor_states"][0]["energy_conserved"], true);
            assert!(
                report["final_actor_states"][0]["granted_energy"]
                    .as_u64()
                    .unwrap()
                    > 100
            );
            assert!(
                crate::simulation::tests::support::final_buffs(&report)
                    .iter()
                    .all(|buff| !buff["id"].as_str().unwrap().starts_with("tiger_tally."))
            );
        }
    }
}

#[test]
fn tiger_tally_native_plunge_does_not_inherit_basic_attack_bonus_from_melee_input() {
    use super::hathor_actions::make_action;
    let mut original = crate::dsl::compiler::compile_scenario(
        "options duration=8 crit=fixed;
        hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        hathor add weapon=fork_TigerTally refine=1;
        hathor add stats dmg%=0.25;
        target hp=100000000 def=0 resist=0 boss=false; active hathor;",
        Default::default(),
    )
    .unwrap()
    .request;
    original.actors[0].static_stats.base_atk = 1000;
    original.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    original
        .model_assumptions
        .extend(crate::characters::hathor::source_assumptions());
    original
        .model_assumptions
        .extend(crate::characters::hathor::branch::source_assumptions());
    super::blackbird_secondary::stamina(&mut original, 100);
    for (effect, base, normal) in [
        ("GE_Player_Hathor_Branch_Damage", 2400, true),
        ("GE_Player_Hathor_AirAttack_Damage", 1250, false),
    ] {
        let mut req = original.clone();
        req.actions = vec![
            make_action(
                0,
                0,
                "GA_Hathor_Skill",
                "GE_Player_Hathor_Skill1_Damage",
                0,
                0,
            ),
            make_action(
                2_000_000,
                0,
                "GA_Hathor_Melee",
                if normal {
                    "GE_Player_Hathor_Melee1_Damage"
                } else {
                    effect
                },
                0,
                0,
            ),
        ];
        if normal {
            req.hold_inputs.insert(
                1,
                GameHoldInput {
                    duration_us: 1_500_000,
                    moving: false,
                    reserved_stocks: None,
                },
            );
            req.branch_inputs.insert(
                1,
                GameBranchInput {
                    can_be_launched: true,
                    landing_delay_us: 2_000_000,
                    height_bonus: Ratio::zero(),
                },
            );
        } else {
            req.plunge_inputs.insert(
                1,
                GamePlungeInput {
                    landing_delay_us: 1_000_000,
                    height_bonus: Ratio::zero(),
                },
            );
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hit = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["damage_effect_id"] == effect)
            .unwrap();
        // Static 25% + one 15% stack only for category A, live ATK +15%.
        let expected = fraction(base, 1)
            .mul(fraction(23, 20))
            .unwrap()
            .mul(if normal {
                fraction(7, 5)
            } else {
                fraction(5, 4)
            })
            .unwrap();
        assert_eq!(exact(&hit["non_crit_damage"]), expected);
    }
}
