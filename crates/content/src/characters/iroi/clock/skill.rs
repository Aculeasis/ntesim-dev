//! Tap Skill notifications create independent actors; no duration polling.
use super::*;
use crate::simulation::state::TimedBuffDeclaration;
use ntesim_core::buffs::dispatch::{BuffRecipient, BuffStackingScope};
pub(super) const ABILITY: &str = "GA_Oneiroi_Skill";
pub(super) const TINY: &str = "GE_Player_Oneiroi_Skill_Damage";
pub(super) fn schedule(ctx: &mut CastContext<'_>) -> Result<(), String> {
    for (progress, notify) in [
        (249777, Notify::SkillCooldown),
        (498799, Notify::SkillHeal),
        (498827, Notify::SkillGun),
        (551115, Notify::SkillSyringe),
        (601775, Notify::SkillTrumpet),
    ] {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us,
            Event::Montage {
                owner: ctx.actor_index,
                command: ctx.command_index,
                progress,
                notify,
            },
        )?;
    }
    Ok(())
}
pub(super) fn effect(kind: ActorKind) -> &'static str {
    match kind {
        ActorKind::NormalThree => profile::EFFECTS[2],
        ActorKind::SkillGun => "GE_Player_Oneiroi_Skill_Gun_Damage",
        ActorKind::SkillSyringe => "GE_Player_Oneiroi_Skill_Syringe_Damage",
        ActorKind::SkillTrumpet => "GE_Player_Oneiroi_Skill_Trumpet_Damage",
        ActorKind::BranchGun => super::super::branch::BRANCH1,
        ActorKind::BranchSyringe => super::super::branch::BRANCH2,
        ActorKind::BranchTrumpet => super::super::branch::BRANCH3,
    }
}
pub(super) fn heal(ctx: &mut CallbackContext<'_>, action: &Action) -> Result<(), String> {
    // Tiny resource carrier is issued before the same-time healing callbacks.
    // Its owner policy settles10 Charge once even if integer HP loss is zero.
    let grant = action.source.with_cast(ctx, action.owner, 0, |cast| {
        cast.schedule_native_aoe_packet(TINY, cast.time_us)?;
        super::super::skill::native_skill_heal(cast)
    })?;
    for recipient in 0..ctx.state.actors.len() {
        if recipient == action.owner || ctx.state.actors[recipient].hp == 0 {
            continue;
        }
        ctx.state.apply_buff(
            action.owner,
            TimedBuffDeclaration::reset(
                "Buff_Oneiroi_Skill_AtkUP",
                "character:1075",
                BuffRecipient::SpecificActor(recipient),
                20_000_000,
                BuffStackingScope::PerSource,
                std::collections::BTreeMap::from([("atk_add".into(), grant)]),
            ),
        )?;
    }
    Ok(())
}
