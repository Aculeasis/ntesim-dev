use super::*;
use crate::formula::Ratio;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{NativeScalingStat, state::GameSimulatorState};

#[test]
fn reached_normal4_actor_survives_montage_exit_but_not_later_source_death() {
    let req = crate::dsl::compiler::compile_scenario(
        "options duration=3;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0; active chiz; chiz attack;",
        Default::default(),
    )
    .unwrap()
    .request;
    let packet = crate::simulation::metadata::resolve_requested_action_packet(
        &req.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "character:1073",
        "GA_Chiichan073_Melee",
        "GE_Player_Chiichan073_Melee4_Damage",
        None,
        &req.model_assumptions,
    )
    .unwrap();
    for (now, remaining) in [(236_105, 0), (236_106, 4), (400_000, 3)] {
        let mut state = GameSimulatorState::new(&req);
        let groups: BTreeMap<_, _> = (0..5)
            .map(|index| (format!("test.normal4.{index}"), (236_106, index * 150_000)))
            .collect();
        let mut queue = BinaryHeap::new();
        for (group, (spawn, delay)) in &groups {
            let time = spawn + delay;
            if time <= now {
                continue;
            }
            let mut metadata = packet.clone();
            metadata.binding.energy_group_id = Some(group.clone());
            queue.push(ScheduledGameEvent {
                time_us: time,
                priority: crate::execution::CausalPriority::HitDelivery,
                seq: time,
                data: GameEventData::HitDelivery {
                    command_index: 0,
                    actor_index: 0,
                    target_index: 0,
                    ability_id: "GA_Chiichan073_Melee".into(),
                    damage_effect_id: "GE_Player_Chiichan073_Melee4_Damage".into(),
                    atk_rate: Ratio::one(),
                    scaling_stat: NativeScalingStat::Atk,
                    metadata: Box::new(metadata),
                },
            });
        }
        state.actors[0].character.chiz_mut().unwrap().clock = Some(Box::new(Clock {
            tracker: HitlagTracker::default(),
            action: Some(Action {
                command: 0,
                owner: 0,
                start: 0,
                kind: Kind::Normal(3),
                groups,
            }),
            connected: BTreeSet::new(),
        }));
        stop(&mut state.actors[0], &mut queue, now).unwrap();
        assert_eq!(queue.len(), remaining);
        // The finite producer's already-authored real pulse times remain exact.
        assert!(
            queue
                .iter()
                .all(|event| [386_106, 536_106, 686_106, 836_106].contains(&event.time_us))
        );
        state.actors[0].hp = 0;
        state.current_time_us = now + 1;
        let mut request = req.clone();
        request
            .model_assumptions
            .insert(MODEL.into(), MODEL.description().into());
        let mut seq = 10;
        let mut trace = Vec::new();
        exit(
            &mut CallbackContext {
                state: &mut state,
                request: &request,
                queue: &mut queue,
                sequence: &mut seq,
                trace: &mut trace,
            },
            0,
        )
        .unwrap();
        assert!(queue.is_empty());
    }
}
