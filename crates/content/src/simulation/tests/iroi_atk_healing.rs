use super::*;
use crate::simulation::state::GameSimulatorState;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use std::collections::BTreeMap;

fn request() -> GameScenarioRequest {
    let source = "options duration=2 crit=fixed; iroi char lvl=80/80 awakening=0 talent=1,1,1 qte=1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active iroi; iroi skill;";
    let mut req = crate::dsl::compiler::compile_scenario(source, Default::default())
        .unwrap()
        .request;
    req.actors[0].static_stats.base_atk = 700;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[1].initial_hp = 1000;
    req
}

fn heal(req: &GameScenarioRequest, state: &mut GameSimulatorState) -> Vec<GameEventReceipt> {
    let mut queue = BinaryHeap::new();
    let mut sequence = 0;
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::IroiSkillHeal {
            source_actor_index: 0,
            recipient_actor_index: 1,
            atk_ratio: ExactFraction::new(943, 1000).unwrap(),
        },
        &mut crate::runtime_api::CallbackContext {
            state,
            request: req,
            queue: &mut queue,
            sequence: &mut sequence,
            trace: &mut trace,
        },
    )
    .unwrap();
    trace
}

#[test]
fn iroi_healing_reads_live_atk_zones_and_weapon_stats_without_hp_or_damage_zones() {
    for hp_max in [5000, 40_000] {
        let mut req = request();
        req.actors[0].initial_hp_max = hp_max;
        req.actors[0].initial_hp = hp_max;
        req.actors[0].static_stats.damage_bonus = Ratio {
            numerator: 5,
            denominator: 1,
        };
        req.actors[0].static_stats.crit_rate = Ratio::one();
        req.actors[0].gear_procs = vec![GearProcConfig::WrongGate {
            atk_bonus: Ratio {
                numerator: 4,
                denominator: 25,
            },
            anima_damage_bonus: Ratio::zero(),
            ally_damage_bonus: Ratio::zero(),
            duration_us: 20_000_000,
        }];
        let mut state = GameSimulatorState::new(&req);
        state
            .apply_buff(
                0,
                crate::simulation::state::TimedBuffDeclaration::reset(
                    "test.live_atk",
                    "test",
                    ntesim_core::buffs::dispatch::BuffRecipient::SelfActor,
                    10_000_000,
                    ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                    BTreeMap::from([
                        ("atk_percent".into(), ExactFraction::new(1, 5).unwrap()),
                        ("atk_add".into(), ExactFraction::new(30, 1).unwrap()),
                    ]),
                ),
            )
            .unwrap();
        let trace = heal(&req, &mut state);
        // 1000 compiled ATK + 700 * (.20 + .16) + 30 flat = 1282.
        // 1282 * .943 = 1208.926 -> 1209, independent of HP/crit/damage zones.
        let payload = trace[0].data.as_ref().unwrap();
        assert_eq!(
            payload.exact_raw,
            Some(ExactFraction::new(1282, 1).unwrap())
        );
        assert_eq!(
            payload.atk_ratio,
            Some(ExactFraction::new(943, 1000).unwrap())
        );
        assert_eq!((payload.gross, payload.applied), (Some(1209), Some(1209)));
        assert_eq!(state.actors[1].hp, 2209);
        state.advance_unpaused_test_clock(10_000_000);
        let trace = heal(&req, &mut state);
        // The expired .20/flat30 buff leaves weapon .16: 1112 * .943 -> 1049.
        let payload = trace[0].data.as_ref().unwrap();
        assert_eq!(
            payload.exact_raw,
            Some(ExactFraction::new(1112, 1).unwrap())
        );
        assert_eq!(
            payload.atk_ratio,
            Some(ExactFraction::new(943, 1000).unwrap())
        );
        assert_eq!(payload.gross, Some(1049));
    }
}

#[test]
fn iroi_healing_uses_the_same_strict_half_hp_weapon_predicate_as_damage() {
    for (hp, gross) in [(10_001, 1075), (10_000, 943), (9999, 943)] {
        let mut req = request();
        req.actors[0].initial_hp_max = 20_000;
        req.actors[0].initial_hp = hp;
        req.actors[0].gear_procs = vec![GearProcConfig::ForgottenHpStats {
            atk_bonus: Ratio {
                numerator: 1,
                denominator: 5,
            },
            def_bonus: Ratio {
                numerator: 1,
                denominator: 4,
            },
        }];
        let mut state = GameSimulatorState::new(&req);
        let trace = heal(&req, &mut state);
        assert_eq!(trace[0].data.as_ref().unwrap().gross, Some(gross));
    }
}

#[test]
fn iroi_healing_queries_living_team_atk_and_never_revives_or_heals_from_a_dead_source() {
    let mut req = request();
    let mut hotori = req.actors[0].clone();
    hotori.character_id = "character:1052".into();
    hotori.slot = 2;
    hotori.awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors.push(hotori);
    let mut state = GameSimulatorState::new(&req);
    assert_eq!(
        heal(&req, &mut state)[0].data.as_ref().unwrap().exact_raw,
        Some(ExactFraction::new(1210, 1).unwrap())
    );
    state.actors[2].hp = 0;
    assert_eq!(
        heal(&req, &mut state)[0].data.as_ref().unwrap().exact_raw,
        Some(ExactFraction::new(1000, 1).unwrap())
    );
    state.actors[0].hp = 0;
    let hp = state.actors[1].hp;
    assert!(heal(&req, &mut state).is_empty());
    assert_eq!(state.actors[1].hp, hp);
    state.actors[0].hp = 1;
    state.actors[1].hp = 0;
    assert!(heal(&req, &mut state).is_empty());
    assert_eq!(state.actors[1].hp, 0);
}
