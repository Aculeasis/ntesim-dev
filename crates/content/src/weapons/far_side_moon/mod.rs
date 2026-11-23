use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::{DamageElement, InputActionRole, PacketMetadata};
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GameActorConfig, GameEventReceipt, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
use ntesim_core::execution::CausalPriority;
use ntesim_core::formula::Ratio;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsFarSideMoonV1;

const BUFF_ID: &str = "far_side_moon.redirect_window";
const SOURCE: &str = "weapon:fork_prism";

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        if let GearProcConfig::FarSideMoon {
            basic_attack_damage_bonus,
            lakshana_damage_bonus,
            duration_us,
            ..
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            if ctx.action_metadata.input_role == Some(InputActionRole::RedirectSkill) {
                crate::gear::schedule_cast_buff(
                    ctx,
                    ContentEvent::FarSideMoonGrant {
                        actor_index: ctx.actor_index,
                        basic_attack_damage_bonus: *basic_attack_damage_bonus,
                        lakshana_damage_bonus: *lakshana_damage_bonus,
                        duration_us: *duration_us,
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn crit_rate_bonus(config: &GameActorConfig) -> Result<ExactFraction, String> {
    let mut total = ExactFraction::new(0, 1)?;
    for proc in &config.gear_procs {
        if let GearProcConfig::FarSideMoon {
            crit_rate_bonus, ..
        } = proc
        {
            total = total.add(ExactFraction::from_ratio(*crit_rate_bonus)?)?;
        }
    }
    Ok(total)
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::FarSideMoon {
            skill_damage_bonus, ..
        } = proc
        else {
            continue;
        };
        let category = packet.native_damage_source_category.as_deref();
        if category == Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_E") {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*skill_damage_bonus)?)?;
        }
        if category != Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A")
            && packet.element != Some(DamageElement::Lakshana)
        {
            continue;
        }
        let active = ctx.buff_stats;
        if category == Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A")
            && let Some(value) = active.get("far_side_moon_basic_attack_damage")
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*value)?)?;
        }
        if packet.element == Some(DamageElement::Lakshana)
            && let Some(value) = active.get("far_side_moon_lakshana_damage")
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*value)?)?;
        }
    }
    Ok(modifiers)
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    basic_attack_damage_bonus: Ratio,
    lakshana_damage_bonus: Ratio,
    duration_us: u64,
) -> Result<(), String> {
    let state = &mut *ctx.state;
    state.max_proc_depth_reached = state.max_proc_depth_reached.max(1);
    state.apply_buff(
        actor_index,
        TimedBuffDeclaration::reset(
            BUFF_ID,
            SOURCE,
            BuffRecipient::SelfActor,
            duration_us,
            BuffStackingScope::PerSource,
            std::collections::BTreeMap::from([
                (
                    "far_side_moon_basic_attack_damage".into(),
                    ExactFraction::from_ratio(basic_attack_damage_bonus)?,
                ),
                (
                    "far_side_moon_lakshana_damage".into(),
                    ExactFraction::from_ratio(lakshana_damage_bonus)?,
                ),
            ]),
        ),
    )?;
    let expiry = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("Far Side of the Moon expiry overflow")?;
    if expiry <= ctx.request.absolute_limit_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue
            .push(crate::simulation::events::ScheduledGameEvent {
                time_us: expiry,
                priority: CausalPriority::BuffTickOrExpiry,
                seq: *ctx.sequence,
                data: crate::simulation::events::GameEventData::Content(
                    ContentEvent::BuffLifecycleExpiry,
                ),
            });
    }
    ctx.trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "far_side_moon_redirect_window".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: Some(actor_index),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            expires_at_us: Some(expiry),
            ..Default::default()
        }),
    });
    Ok(())
}
