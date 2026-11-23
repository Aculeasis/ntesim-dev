use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CastContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::{ExactFraction, GameActorConfig, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::BuffRecipient;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsGoldWoolV1;
pub(crate) const FINISH_MODEL: HypothesisId = HypothesisId::WeaponsGoldWoolChaosRealfinishV1;

fn grant(
    owner: usize,
    crit_damage_bonus: crate::formula::Ratio,
    duration_us: u64,
) -> Result<crate::modifiers::TimedBuffGrant, String> {
    Ok(crate::modifiers::BuffGrantBuilder::new(
        owner,
        "gold_wool.crit_damage",
        "weapon:fork_GoldWool",
    )
    .recipient(BuffRecipient::SelfActor)
    .duration_us(duration_us)
    .stat_ratio("crit_damage_bonus", crit_damage_bonus)?
    .receipt(
        "gold_wool_crit_damage",
        GameEventPayload {
            crit_damage_bonus: Some(ExactFraction::from_ratio(crit_damage_bonus)?),
            duration_us: Some(duration_us),
            ..Default::default()
        },
    )
    .build())
}

pub(crate) fn on_skill_finish(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
) -> Result<(), String> {
    let config = &ctx.request.actors[owner];
    if ctx.state.actors[owner].hp == 0
        || !crate::registry::skill_gear_trigger_on_finish(ctx.request, config)
    {
        return Ok(());
    }
    let grants: Vec<_> = config
        .gear_procs
        .iter()
        .filter_map(|proc| match proc {
            GearProcConfig::GoldWool {
                crit_damage_bonus,
                duration_us,
                ..
            } => Some((*crit_damage_bonus, *duration_us)),
            _ => None,
        })
        .collect();
    for (bonus, duration) in grants {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            FINISH_MODEL,
        )?;
        grant(owner, bonus, duration)?.apply(ctx)?;
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::GoldWool {
            crit_damage_bonus,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            let realfinish = crate::registry::skill_gear_trigger_on_finish(ctx.request, ctx.config);
            if realfinish {
                ctx.record_hypothesis(FINISH_MODEL)?;
            }
            if realfinish && ctx.action_metadata.input_role == Some(InputActionRole::RedirectSkill)
            {
                continue;
            }
            if matches!(
                ctx.action_metadata.input_role,
                Some(InputActionRole::RedirectSkill | InputActionRole::Ultimate)
            ) {
                let grant = grant(ctx.actor_index, *crit_damage_bonus, *duration_us)?;
                crate::gear::schedule_cast_buff(
                    ctx,
                    crate::callbacks::ContentEvent::TimedBuffGrant(Box::new(grant)),
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    config: &GameActorConfig,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if packet.element == Some(DamageElement::Lakshana) {
        for proc in &config.gear_procs {
            if let GearProcConfig::GoldWool { lakshana_bonus, .. } = proc {
                modifiers = modifiers.add_damage(ExactFraction::from_ratio(*lakshana_bonus)?)?;
            }
        }
    }
    Ok(modifiers)
}
