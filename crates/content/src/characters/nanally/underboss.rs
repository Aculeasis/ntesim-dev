//! Owner-bound later Underboss attacks; the initiating action and Burst
//! coefficient coordinate remain separate throughout delivery.
use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::NanallyUnderbossCoordinationV2;
pub(crate) const SUMMON_HYPOTHESIS_ID: HypothesisId = HypothesisId::NanallyBurstSummonStateV1;
pub(crate) const BURST_COOLDOWN_US: u64 = 15_000_000;

const NORMAL_PACKETS: [&[&str]; 5] = [
    &["GE_Player_Nanally_Cat_Melee1_Damage"],
    &["GE_Player_Nanally_Cat_Melee2_Damage"],
    &[
        "GE_Player_Nanally_Cat_Melee3_Damage",
        "GE_Player_Nanally_Cat_Melee4_Damage",
    ],
    &["GE_Player_Nanally_Cat_Melee5_Damage"; 3],
    &[
        "GE_Player_Nanally_Cat_Melee6_Damage",
        "GE_Player_Nanally_Cat_Melee7_Damage",
    ],
];
const SKILL_PACKETS: &[&str] = &["GE_Player_Nanally_Cat_Skill_Damage"; 5];
const HEAVY_PACKETS: [&[&str]; 3] = [
    &["GE_Player_Nanally_Cat_Branch1_Damage"],
    &[
        "GE_Player_Nanally_Cat_Branch2_Damage",
        "GE_Player_Nanally_Cat_Branch3_Damage",
        "GE_Player_Nanally_Cat_Branch4_Damage",
    ],
    &["GE_Player_Nanally_Cat_Branch5_Damage"; 3],
];

pub(crate) fn is_coordinated_packet(packet: &str) -> bool {
    is_melee_child(packet)
        || SKILL_PACKETS.contains(&packet)
        || packet == super::support::CAT_PACKET
        || packet == super::riposte::CAT_PACKET
}

pub(crate) fn is_melee_child(packet: &str) -> bool {
    is_normal_packet(packet) || is_heavy_packet(packet) || packet == super::plunge::CAT_PACKET
}

fn is_heavy_packet(packet: &str) -> bool {
    HEAVY_PACKETS.iter().any(|group| group.contains(&packet))
}

pub(crate) fn is_normal_packet(packet: &str) -> bool {
    NORMAL_PACKETS.iter().any(|group| group.contains(&packet))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Packet {
    effect: String,
    rate: crate::formula::Ratio,
    scaling: crate::simulation::NativeScalingStat,
    metadata: crate::simulation::metadata::ActionPacketMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CoordinatedAttack {
    actor_index: usize,
    command_index: usize,
    generation: u64,
    packets: Vec<Packet>,
    all_targets: bool,
}

impl CoordinatedAttack {
    pub(crate) fn belongs_to_command(&self, owner: usize, command: usize) -> bool {
        self.actor_index == owner && self.command_index == command
    }
}

pub(crate) fn start(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(SUMMON_HYPOTHESIS_ID)?;
    let duration_us = if ctx.config.awakening.has_effect(2) {
        PARAMETERS.a2_underboss_duration_us
    } else {
        PARAMETERS.underboss_duration_us
    };
    let coordinate = match ctx.request.actions.get(ctx.command_index) {
        Some(crate::simulation::GameActionCommand::InvokeAbility { array_index, .. }) => {
            u32::try_from(*array_index).map_err(|_| "Nanally Underboss coordinate exceeds u32")?
        }
        _ => return Err("Nanally Underboss requires an admitted Burst coordinate".into()),
    };
    let state = ctx.actor.character.nanally_mut()?;
    state.underboss_coordinate = Some(coordinate);
    state.underboss_generation = state
        .underboss_generation
        .checked_add(1)
        .ok_or("Nanally Underboss generation overflow")?;
    state.underboss_until_us = ctx
        .time_us
        .checked_add(duration_us)
        .ok_or("Nanally Underboss deadline overflow")?;
    Ok(())
}

pub(crate) fn schedule(ctx: &mut CastContext<'_>, stage: Option<usize>) -> Result<(), String> {
    let selected = stage.map_or(SKILL_PACKETS, |index| NORMAL_PACKETS[index]);
    schedule_packets(ctx, selected).map(|_| ())
}

pub(crate) fn schedule_support(ctx: &mut CastContext<'_>) -> Result<(), String> {
    schedule_packets(ctx, &[super::support::CAT_PACKET]).map(|_| ())
}

pub(crate) fn schedule_riposte(ctx: &mut CastContext<'_>) -> Result<(), String> {
    schedule_packets(ctx, &[super::riposte::CAT_PACKET]).map(|_| ())
}

pub(crate) fn schedule_plunge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    schedule_packets(ctx, &[super::plunge::CAT_PACKET]).map(|_| ())
}

pub(crate) fn schedule_heavy(ctx: &mut CastContext<'_>, stage: usize) -> Result<(), String> {
    schedule_packets(ctx, HEAVY_PACKETS[stage]).map(|_| ())
}

fn schedule_packets(ctx: &mut CastContext<'_>, selected: &[&str]) -> Result<bool, String> {
    let state = ctx.actor.character.nanally_mut()?;
    if state.underboss_until_us <= ctx.time_us {
        return Ok(false);
    }
    let coordinate = state
        .underboss_coordinate
        .ok_or("Nanally Underboss talent snapshot missing")?;
    let generation = state.underboss_generation;
    ctx.record_hypothesis(if ctx.ability_id == "GA_Nanally_QTE" {
        super::support::HYPOTHESIS_ID
    } else if ctx
        .packet_metadata
        .is_some_and(|packet| packet.damage_effect_id == super::riposte::PACKET)
    {
        super::riposte::HYPOTHESIS_ID
    } else {
        HYPOTHESIS_ID
    })?;
    let mut packets = Vec::with_capacity(selected.len());
    for effect in selected {
        let projection = crate::build::project_native_packet_coefficient(
            ntesim_data::skill_levels::frozen::tables(),
            &crate::build::PacketCoefficientRequest {
                content_id: &ctx.request.content_id,
                character_id: &ctx.config.character_id,
                ability_id: ctx.ability_id,
                damage_effect_id: effect,
                array_index: coordinate,
                research_metadata: None,
                model_assumptions: &ctx.request.model_assumptions,
            },
        )?;
        packets.push(Packet {
            effect: (*effect).into(),
            rate: serde_json::from_value(projection["compiled_rate"].clone())
                .map_err(|error| format!("Nanally coordinated rate invalid: {error}"))?,
            scaling: serde_json::from_value(projection["compiled_scaling_stat"].clone())
                .map_err(|error| format!("Nanally coordinated scaling invalid: {error}"))?,
            metadata: ctx.child_packet_metadata(effect)?,
        });
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.hit_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::Content(ContentEvent::NanallyUnderboss(Box::new(
            CoordinatedAttack {
                actor_index: ctx.actor_index,
                command_index: ctx.command_index,
                generation,
                packets,
                all_targets: selected
                    .first()
                    .is_some_and(|packet| is_heavy_packet(packet))
                    || ctx.packet_metadata.is_some_and(|packet| {
                        matches!(
                            packet.damage_effect_id.as_str(),
                            super::riposte::PACKET | super::plunge::PACKET
                        )
                    }),
            },
        ))),
    });
    Ok(true)
}

pub(crate) fn deliver(
    ctx: &mut CallbackContext<'_>,
    attack: CoordinatedAttack,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[attack.actor_index];
    if actor.hp == 0 {
        return Ok(());
    }
    let targets: Vec<_> = ctx
        .state
        .targets
        .iter()
        .enumerate()
        .filter_map(|(index, target)| {
            (target.hp_current > 0 && (attack.all_targets || index == 0)).then_some(index)
        })
        .collect();
    if targets.is_empty() {
        return Ok(());
    }
    let state = actor.character.nanally_mut()?;
    if state.underboss_generation != attack.generation
        || state.underboss_until_us <= ctx.state.current_time_us
    {
        return Ok(());
    }
    for packet in attack.packets {
        for target_index in &targets {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.state.current_time_us,
                priority: CausalPriority::HitDelivery,
                seq: *ctx.sequence,
                data: GameEventData::HitDelivery {
                    command_index: attack.command_index,
                    actor_index: attack.actor_index,
                    target_index: *target_index,
                    ability_id: packet.metadata.action.ability_id.clone(),
                    damage_effect_id: packet.effect.clone(),
                    atk_rate: packet.rate,
                    scaling_stat: packet.scaling,
                    metadata: Box::new(packet.metadata.clone()),
                },
            });
        }
    }
    super::callback::schedule_awakenings(
        &ctx.request.actors[attack.actor_index],
        attack.actor_index,
        ctx.state.current_time_us,
        ctx.sequence,
        ctx.queue,
    )?;

    Ok(())
}
