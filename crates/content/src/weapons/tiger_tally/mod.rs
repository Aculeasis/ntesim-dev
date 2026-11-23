use crate::HypothesisId;
mod config;
pub(crate) use config::{validate_config, validate_deadline};
pub(crate) mod projection;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::TimedBuffDeclaration;
use crate::simulation::{ExactFraction, GearProcConfig};
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::WeaponsTigerTallyV2;

pub(crate) const BUFF_ID: &str = "tiger_tally.normal_damage_up";
pub(crate) const ARC_BUFF_ID: &str = "tiger_tally.arc_boss_damage_up";
pub(crate) const SOURCE: &str = "weapon:fork_TigerTally";
pub(crate) const NORMAL_DAMAGE_BONUS_STAT: &str = "tiger_tally_normal_damage_bonus";
pub(crate) const BOSS_DAMAGE_BONUS_STAT: &str = "tiger_tally_boss_damage_bonus";

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) left_talisman_until_us: Option<u64>,
    pub(crate) right_talisman_until_us: Option<u64>,
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::TigerTallyAtk {
            normal_damage_bonus,
            duration_us,
            talisman_window_us,
            boss_damage_bonus,
            boss_damage_duration_us,
            ..
        } = proc
        else {
            continue;
        };
        ctx.record_hypothesis(HYPOTHESIS_ID)?;
        if !matches!(
            ctx.action_metadata.input_role,
            Some(InputActionRole::RedirectSkill) | Some(InputActionRole::Ultimate)
        ) {
            continue;
        }
        let bonus = ExactFraction::from_ratio(*normal_damage_bonus)?;
        crate::simulation::state::apply_timed_buff(
            ctx.buffs,
            TimedBuffDeclaration {
                id: BUFF_ID.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us: *duration_us,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Independent,
                max_stacks: 2,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers: BTreeMap::from([(NORMAL_DAMAGE_BONUS_STAT.into(), bonus)]),
            },
            ctx.actor_index,
            ctx.world_time_us,
        )
        .map_err(|error| format!("Commander Tiger Talisman buff grant failed: {error}"))?;

        let state = &mut ctx.actor.equipment.tiger_tally;
        let opposite_until_us = match ctx.action_metadata.input_role {
            Some(InputActionRole::RedirectSkill) => state.right_talisman_until_us,
            Some(InputActionRole::Ultimate) => state.left_talisman_until_us,
            _ => continue,
        };
        let activates_arc = opposite_until_us.is_some_and(|deadline| ctx.time_us < deadline);
        if activates_arc {
            state.left_talisman_until_us = None;
            state.right_talisman_until_us = None;
            crate::simulation::state::apply_timed_buff(
                ctx.buffs,
                TimedBuffDeclaration {
                    id: ARC_BUFF_ID.into(),
                    source: SOURCE.into(),
                    recipient: BuffRecipient::SelfActor,
                    duration_us: *boss_damage_duration_us,
                    stacking_scope: BuffStackingScope::PerSource,
                    refresh_policy: RefreshPolicy::Reset,
                    max_stacks: 1,
                    decay_policy: DecayPolicy::AllAtOnce,
                    stat_modifiers: BTreeMap::from([(
                        BOSS_DAMAGE_BONUS_STAT.into(),
                        ExactFraction::from_ratio(*boss_damage_bonus)?,
                    )]),
                },
                ctx.actor_index,
                ctx.world_time_us,
            )
            .map_err(|error| format!("Arc Commander Tiger Talisman grant failed: {error}"))?;
        } else {
            if state
                .left_talisman_until_us
                .is_some_and(|deadline| ctx.time_us >= deadline)
            {
                state.left_talisman_until_us = None;
            }
            if state
                .right_talisman_until_us
                .is_some_and(|deadline| ctx.time_us >= deadline)
            {
                state.right_talisman_until_us = None;
            }
            let deadline = ctx
                .time_us
                .checked_add(*talisman_window_us)
                .ok_or("Commander Tiger Talisman deadline overflow")?;
            match ctx.action_metadata.input_role {
                Some(InputActionRole::RedirectSkill) => {
                    state.left_talisman_until_us = Some(deadline);
                }
                Some(InputActionRole::Ultimate) => {
                    state.right_talisman_until_us = Some(deadline);
                }
                _ => unreachable!("talisman role was checked above"),
            }
        }
    }
    Ok(())
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    packet: &crate::simulation::metadata::PacketMetadata,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    for proc in &ctx.config.gear_procs {
        let GearProcConfig::TigerTallyAtk { .. } = proc else {
            continue;
        };
        if let Some(stacked) = ctx.buff_stats.get(NORMAL_DAMAGE_BONUS_STAT)
            && receives_normal_bonus(packet)?
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*stacked)?)?;
        }
        if ctx.target.is_boss
            && let Some(bonus) = ctx.buff_stats.get(BOSS_DAMAGE_BONUS_STAT)
        {
            modifiers = modifiers.add_damage(ExactFraction::from_ratio(*bonus)?)?;
        }
    }
    Ok(modifiers)
}

fn receives_normal_bonus(
    packet: &crate::simulation::metadata::PacketMetadata,
) -> Result<bool, String> {
    let source = packet
        .native_damage_source_category
        .as_deref()
        .ok_or_else(|| {
            format!(
                "Commander Tiger Talisman requires damage source classification for {}",
                packet.damage_effect_id
            )
        })?;
    match source.strip_prefix("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_") {
        Some("A" | "Z") => Ok(true),
        Some("B" | "E" | "F" | "H" | "P" | "Q" | "R" | "T" | "X") => Ok(false),
        _ => Err(format!(
            "unsupported Commander Tiger Talisman damage source classification {source} for {}",
            packet.damage_effect_id
        )),
    }
}
