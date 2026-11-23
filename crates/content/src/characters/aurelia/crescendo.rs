use crate::HypothesisId;
use crate::runtime_api::ActorHitContext;
use crate::simulation::ExactFraction;
use ntesim_core::buffs::dispatch::BuffRecipient;
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::AureliaCrescendoB4V1;

pub(crate) fn is_bullet_packet(packet_id: &str) -> bool {
    matches!(
        packet_id,
        "GE_Player_Mitsuki_Melee3_Damage"
            | "GE_Player_Mitsuki_Melee3_Power_Damage"
            | "GE_Player_Mitsuki_EvadeBranch_Damage"
            | "GE_Player_Mitsuki_EvadeBranch_Power_Damage"
            | "GE_Player_Mitsuki_PerfectAtkBullet_Damage"
            | "GE_Player_Mitsuki_PerfectAtkBullet_Power_Damage"
    )
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !is_bullet_packet(ctx.damage_effect_id) {
        return Ok(());
    }
    if !ctx
        .request
        .model_assumptions
        .contains_key(HYPOTHESIS_ID.as_str())
    {
        return Ok(());
    }
    if ctx.config.character_id != "character:1070" {
        return Ok(());
    }

    let has_a6 = ctx.config.awakening.has_effect(6);
    let max_stacks = if has_a6 { 20 } else { 10 };
    let now_us = ctx.time_us;

    let state = ctx.actor.character.aurelia_mut()?;
    if state.crescendo_expires_at_us <= now_us {
        state.crescendo_stacks = 0;
    }

    let old_stacks = state.crescendo_stacks;
    let next_stacks = (old_stacks + 1).min(max_stacks);
    let duration_us = if !has_a6 && old_stacks < 10 && next_stacks == 10 {
        if !state.crescendo_extended_once {
            state.crescendo_extended_once = true;
            7_000_000
        } else {
            5_000_000
        }
    } else {
        5_000_000
    };

    state.crescendo_stacks = next_stacks;
    state.crescendo_expires_at_us = now_us + duration_us;

    ctx.refresh_buff(
        "aurelia.crescendo",
        BuffRecipient::SelfActor,
        duration_us,
        BTreeMap::from([(
            "atk_percent".into(),
            ExactFraction::new(next_stacks as u128, 100)?,
        )]),
    )?;

    Ok(())
}

pub(crate) fn apply_a6_skill_finish(
    state: &mut super::awakenings::State,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    actor_index: usize,
    character_id: &str,
    now_us: u64,
    world_time_us: u64,
) -> Result<(), String> {
    if state.crescendo_expires_at_us <= now_us {
        state.crescendo_stacks = 0;
    }
    let next_stacks = (state.crescendo_stacks + 10).min(20);
    state.crescendo_stacks = next_stacks;
    state.crescendo_expires_at_us = now_us + 5_000_000;

    crate::simulation::state::apply_timed_buff(
        buffs,
        crate::simulation::state::TimedBuffDeclaration::reset(
            "aurelia.crescendo",
            character_id,
            BuffRecipient::SelfActor,
            5_000_000,
            ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
            BTreeMap::from([(
                "atk_percent".into(),
                ExactFraction::new(next_stacks as u128, 100)?,
            )]),
        ),
        actor_index,
        world_time_us,
    )?;

    Ok(())
}
