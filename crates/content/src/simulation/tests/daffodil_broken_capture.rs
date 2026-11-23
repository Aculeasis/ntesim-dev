use super::*;
use crate::characters::daffodil::insight;
use crate::simulation::{
    break_damage, break_state, events::GameEventData, state::GameSimulatorState,
};

#[test]
fn daffodil_r6_a4_a5_capture_coexists_with_a6_and_other_elemental_contributors() {
    let req = super::daffodil_broken::scenario();
    let mut state = GameSimulatorState::new(&req);
    for _ in 0..2 {
        insight::apply_target(&mut state.buffs, 0, 0, true, 0).unwrap();
    }
    let mut queue = Default::default();
    let mut sequence = 0;
    let mut trace = Vec::new();
    assert!(
        break_state::apply_unbalance(
            &req,
            &mut state,
            &mut queue,
            &mut sequence,
            &mut trace,
            1,
            0,
            ExactFraction::new(1, 1).unwrap()
        )
        .unwrap()
    );
    break_damage::schedule(&req, &mut state, &mut queue, &mut sequence, 0, 0).unwrap();
    let packets: Vec<_> = std::iter::from_fn(|| queue.pop())
        .filter_map(|event| match event.data {
            GameEventData::TargetBreakDamage(packet) => Some(packet),
            _ => None,
        })
        .collect();
    assert_eq!(
        packets
            .iter()
            .map(|packet| packet.actor_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 0, 0]
    );
    // 3603 * (1 + .20 + 2*.15) * 10 * (20/37) * .95 = 1026855/37.
    for packet in packets.iter().filter(|packet| packet.actor_index == 0) {
        assert_eq!(packet.damage, ExactFraction::new(1_026_855, 37).unwrap());
        assert_eq!(packet.contribution.bonus, ExactFraction::new(1, 2).unwrap());
        assert_eq!(
            packet.contribution.defense_ignore,
            ExactFraction::new(3, 20).unwrap()
        );
    }
    assert_eq!(packets[1].damage, ExactFraction::new(68457, 4).unwrap());
    assert_eq!(packets[2].damage, ExactFraction::new(14412, 1).unwrap());
}
