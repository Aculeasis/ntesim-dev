use super::*;
use crate::simulation::{
    break_damage::{self, BreakPacket},
    state::GameSimulatorState,
};
use ntesim_core::buffs::dispatch::*;

fn request() -> GameScenarioRequest {
    let mut request = research_request();
    request.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 30,
            denominator: 1,
        },
        current_gauge: Ratio::one(),
        broken_duration_us: 1000,
        discord_fraction: None,
    });
    request
}

fn snapshot(request: &GameScenarioRequest, state: &mut GameSimulatorState) -> Vec<BreakPacket> {
    let mut queue = crate::simulation::timer_queue::EventQueue::new();
    break_damage::schedule(request, state, &mut queue, &mut 0, 0, 0).unwrap();
    queue
        .into_sorted_vec()
        .into_iter()
        .map(|event| match event.data {
            crate::simulation::events::GameEventData::TargetBreakDamage(packet) => *packet,
            _ => unreachable!(),
        })
        .collect()
}

#[test]
fn break_capture_respects_buff_expiry_living_carriers_and_original_not_reduced_capacity() {
    let mut request = request();
    let mut ally = request.actors[0].clone();
    ally.slot = 1;
    request.actors.push(ally);
    for (time, alive, expected) in [(99, true, 28824), (100, true, 14412), (99, false, 14412)] {
        let mut state = GameSimulatorState::new(&request);
        state.advance_unpaused_test_clock(time);
        state.targets[0].break_state.as_mut().unwrap().capacity = ExactFraction::new(3, 1).unwrap();
        if !alive {
            state.actors[1].hp = 0;
        }
        state
            .buffs
            .apply_buff(
                BuffDefinition {
                    id: "test.break_intensity".into(),
                    source: "test".into(),
                    stacking_scope: BuffStackingScope::PerSource,
                    semantic_tags: Default::default(),
                    source_lifetime: BuffSourceLifetime::WhileAlive,
                    recipient: BuffRecipient::SpecificActor(0),
                    duration_us: Some(100),
                    refresh_policy: RefreshPolicy::Reset,
                    max_stacks: 1,
                    decay_policy: DecayPolicy::AllAtOnce,
                    stat_modifiers: BTreeMap::from([(
                        "break_intensity".into(),
                        Ratio {
                            numerator: 300,
                            denominator: 1,
                        },
                    )]),
                },
                1,
                0,
            )
            .unwrap();
        let packet = snapshot(&request, &mut state)
            .into_iter()
            .find(|p| p.actor_index == 0)
            .unwrap();
        assert_eq!(packet.damage, ExactFraction::new(expected, 1).unwrap());
        assert_eq!(
            packet.contribution.original_capacity,
            ExactFraction::new(30, 1).unwrap()
        );
    }
}

#[test]
fn equipped_break_bonuses_reach_hp_and_danzaburou_uses_one_strongest_same_element_aura() {
    for (weapon, proc, expected) in [
        (
            "weapon:fork_BlackBook",
            GearProcConfig::BlackBookBreak {
                break_intensity_bonus: Ratio {
                    numerator: 60,
                    denominator: 1,
                },
            },
            (86472, 5),
        ),
        (
            "weapon:fork_jiaojuan",
            GearProcConfig::JiaojuanBreak {
                break_intensity_bonus: Ratio {
                    numerator: 48,
                    denominator: 1,
                },
                broken_damage_bonus: Ratio {
                    numerator: 1,
                    denominator: 10,
                },
            },
            (417948, 25),
        ),
    ] {
        let mut request = request();
        request.actors[0].weapon_id = weapon.into();
        request.actors[0].gear_procs = vec![proc];
        let mut state = GameSimulatorState::new(&request);
        assert_eq!(
            snapshot(&request, &mut state)[0].damage,
            ExactFraction::new(expected.0, expected.1).unwrap()
        );
    }
    let mut request = request();
    let identities = crate::registry::character_descriptors()
        .unwrap()
        .into_iter()
        .filter(|d| d.element == "NATURE")
        .take(3)
        .collect::<Vec<_>>();
    assert_eq!(identities.len(), 3);
    let template = request.actors[0].clone();
    request.actors = identities
        .iter()
        .enumerate()
        .map(|(slot, d)| {
            let mut actor = template.clone();
            actor.slot = slot;
            actor.character_id = d.character_id.into();
            if slot > 0 {
                actor.weapon_id = "weapon:fork_tuansanlang".into();
                actor.gear_procs = vec![GearProcConfig::TuansanlangTeam {
                    break_intensity_bonus: Ratio {
                        numerator: if slot == 1 { 70 } else { 126 },
                        denominator: 1,
                    },
                }];
            }
            actor
        })
        .collect();
    let mut state = GameSimulatorState::new(&request);
    let packets = snapshot(&request, &mut state);
    assert!(
        packets
            .iter()
            .all(|p| p.contribution.intensity == ExactFraction::new(126, 1).unwrap())
    );
    state.actors[2].hp = 0;
    assert!(
        snapshot(&request, &mut state)
            .iter()
            .all(|p| p.contribution.intensity == ExactFraction::new(70, 1).unwrap())
    );
    state.actors[1].hp = 0;
    assert_eq!(
        snapshot(&request, &mut state)[0].contribution.intensity,
        ExactFraction::new(0, 1).unwrap()
    );
}

#[test]
fn captured_break_packet_survives_source_death_and_strict_admission_requires_the_new_model() {
    let request = request();
    let mut state = GameSimulatorState::new(&request);
    let packet = snapshot(&request, &mut state).remove(0);
    state.actors[0].hp = 0;
    let mut hits = Vec::new();
    break_damage::settle(
        &mut crate::simulation::context::EventContext {
            request: &request,
            prepared_packets: &[],
            state: &mut state,
            queue: &mut Default::default(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
            rng: &mut None,
            hits: &mut hits,
            action_metadata_ledger: &mut Vec::new(),
        },
        packet,
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].rounded_damage, 14412);
    let mut state = GameSimulatorState::new(&request);
    snapshot(&request, &mut state);
}
