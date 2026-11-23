mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod callback;
pub(crate) mod hit;
pub(crate) mod projection;
use crate::simulation::GameFirefliesStack;

pub(crate) const BUFF_ID: &str = "fireflies.crit_damage";
const SOURCE: &str = "set:fireflies";

fn is_fireflies(buff: &ntesim_core::buffs::dispatch::ActiveBuffInstance, actor: usize) -> bool {
    use ntesim_core::buffs::dispatch::BuffRecipient;
    buff.id == BUFF_ID
        && buff.source == SOURCE
        && buff.owner_actor == actor
        && matches!(&buff.recipient, BuffRecipient::SpecificActor(index) if *index == actor)
}

pub(crate) fn stack_count(buffs: &ntesim_core::buffs::dispatch::BuffManager, actor: usize) -> u32 {
    buffs
        .instances()
        .iter()
        .filter(|buff| is_fireflies(buff, actor))
        .map(|buff| buff.stacks)
        .sum()
}

pub(crate) fn stacks(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    actor: usize,
) -> Result<Vec<GameFirefliesStack>, String> {
    use ntesim_core::buffs::dispatch::BuffDuration;
    buffs
        .instances()
        .iter()
        .filter(|buff| is_fireflies(buff, actor))
        .map(|buff| {
            let expires_at_us = match buff.duration {
                BuffDuration::Timed { expires_at_us, .. } => expires_at_us,
                BuffDuration::Permanent => return Err("Fireflies stacks must be timed".into()),
            };
            let value = buff
                .stat_modifiers
                .get("crit_damage_bonus")
                .copied()
                .ok_or("Fireflies stack lacks critical damage value")?;
            Ok(GameFirefliesStack {
                expires_at_us,
                crit_damage_bonus: crate::simulation::ExactFraction::from_ratio(value)?,
            })
        })
        .collect()
}
