use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;

use crate::modifiers::{BuffGrantBuilder, TimedBuffGrant};
use crate::runtime_api::{
    BuffStatsView, CallbackContext, CastContext, DamageContext, HitModifiers,
};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const WEAPON_ID: &str = "weapon:fork_elyms";
pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsEchoesOnHoldV1;

const TEAM_DEF_IGNORE_BUFF_ID: &str = "echoes_on_hold.team_def_ignore";
const WEARER_DEF_IGNORE_BUFF_ID: &str = "echoes_on_hold.wearer_def_ignore";
const COSMOS_PEN_BUFF_ID: &str = "echoes_on_hold.cosmos_penetration";
const COSMOS_PEN_STAT: &str = "echoes_on_hold_cosmos_penetration";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    follow_up_packets: Option<u8>,
    recorded_packet_groups: BTreeSet<String>,
}

fn def_ignore_grant(
    actor_index: usize,
    team_def_ignore: Ratio,
    wearer_def_ignore: Ratio,
    duration_us: u64,
) -> Result<TimedBuffGrant, String> {
    let team_frac = ExactFraction::from_ratio(team_def_ignore)?;
    let wearer_frac = ExactFraction::from_ratio(wearer_def_ignore)?;
    Ok(
        BuffGrantBuilder::new(actor_index, TEAM_DEF_IGNORE_BUFF_ID, WEAPON_ID)
            .recipient(BuffRecipient::TeamAll)
            .duration_us(duration_us)
            .stacking_scope(BuffStackingScope::Shared)
            .stat("def_ignore", team_frac)
            .add_declaration(TimedBuffDeclaration::reset(
                WEARER_DEF_IGNORE_BUFF_ID,
                WEAPON_ID,
                BuffRecipient::SelfActor,
                duration_us,
                BuffStackingScope::Shared,
                BTreeMap::from([("def_ignore".into(), wearer_frac)]),
            ))
            .receipt(
                "echoes_on_hold_def_ignore",
                GameEventPayload {
                    def_ignore: Some(team_frac),
                    duration_us: Some(duration_us),
                    ..Default::default()
                },
            )
            .build(),
    )
}

fn cosmos_pen_grant(
    actor_index: usize,
    cosmos_penetration: Ratio,
    duration_us: u64,
) -> Result<TimedBuffGrant, String> {
    let pen_frac = ExactFraction::from_ratio(cosmos_penetration)?;
    Ok(
        BuffGrantBuilder::new(actor_index, COSMOS_PEN_BUFF_ID, WEAPON_ID)
            .recipient(BuffRecipient::TeamAll)
            .duration_us(duration_us)
            .stacking_scope(BuffStackingScope::Shared)
            .stat(COSMOS_PEN_STAT, pen_frac)
            .receipt(
                "echoes_on_hold_cosmos_penetration",
                GameEventPayload {
                    duration_us: Some(duration_us),
                    ..Default::default()
                },
            )
            .build(),
    )
}

pub(crate) fn initialize(ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    for (actor_index, config) in ctx.request.actors.iter().enumerate() {
        for proc in &config.gear_procs {
            let GearProcConfig::EchoesOnHold {
                team_def_ignore,
                wearer_def_ignore,
                cosmos_penetration,
                duration_us,
                ..
            } = proc
            else {
                continue;
            };
            let def_grant = def_ignore_grant(
                actor_index,
                *team_def_ignore,
                *wearer_def_ignore,
                *duration_us,
            )?;
            def_grant.apply(ctx)?;
            let pen_grant = cosmos_pen_grant(actor_index, *cosmos_penetration, *duration_us)?;
            pen_grant.apply(ctx)?;
        }
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::EchoesOnHold {
            cosmos_penetration,
            duration_us,
            ..
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        match ctx.action_metadata.input_role {
            Some(InputActionRole::Ultimate) => {
                let state = &mut ctx.actor.equipment.echoes_on_hold;
                state.follow_up_packets = Some(0);
                state.recorded_packet_groups.clear();
            }
            Some(InputActionRole::RedirectSkill) => {
                let grant = cosmos_pen_grant(ctx.actor_index, *cosmos_penetration, *duration_us)?;
                crate::gear::schedule_cast_buff(
                    ctx,
                    crate::callbacks::ContentEvent::TimedBuffGrant(Box::new(grant)),
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn on_hp_loss_hit(
    ctx: &mut CastContext<'_>,
    packet_group_id: Option<&str>,
) -> Result<(), String> {
    let state = &mut ctx.actor.equipment.echoes_on_hold;
    let Some(follow_up_packets) = &mut state.follow_up_packets else {
        return Ok(());
    };
    if !ctx.packet_metadata.is_some_and(|packet| {
        packet
            .semantic_tags
            .contains(crate::characters::packet_semantics::FOLLOW_UP_DAMAGE_TAG)
    }) {
        return Ok(());
    }
    if let Some(group) = packet_group_id
        && !state.recorded_packet_groups.insert(group.to_owned())
    {
        return Ok(());
    }
    *follow_up_packets += 1;
    if *follow_up_packets < 3 {
        return Ok(());
    }
    state.follow_up_packets = None;
    let Some(GearProcConfig::EchoesOnHold {
        team_def_ignore,
        wearer_def_ignore,
        duration_us,
        ..
    }) = ctx
        .config
        .gear_procs
        .iter()
        .find(|proc| matches!(proc, GearProcConfig::EchoesOnHold { .. }))
    else {
        return Err("armed Echoes on Hold is missing its gear proc".into());
    };
    let mut grant = def_ignore_grant(
        ctx.actor_index,
        *team_def_ignore,
        *wearer_def_ignore,
        *duration_us,
    )?;
    grant.phase = CausalPriority::ProcCallback;
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx.time_us,
        priority: grant.phase,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::TimedBuffGrant(Box::new(
            grant,
        ))),
    });
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if packet.element == Some(DamageElement::Cosmos)
        && let Some(ratio) = cosmos_penetration(&ctx.buff_stats)
    {
        modifiers = modifiers.add_resistance_reduction(ExactFraction::from_ratio(ratio)?)?;
    }
    Ok(modifiers)
}

pub(crate) fn cosmos_penetration(buff_stats: &impl BuffStatsView) -> Option<Ratio> {
    buff_stats.get(COSMOS_PEN_STAT).copied()
}
