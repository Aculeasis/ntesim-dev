//! Finite held actor, release sections and actual first-birth A1 reward.
use super::clock::Boundary;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::GameScenarioRequest;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorHeldSkillV1;
pub(super) const BIRTH: u64 = 686_667;
const FIRST_TICK: u64 = BIRTH + 250_000;
pub(super) struct Plan {
    pub(super) hits: Vec<(&'static str, u64)>,
    pub(super) recovery: u64,
    pub(super) notifies: Vec<(u64, Boundary)>,
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if accepted(req) && (!super::gaze::accepted(req) || !super::power::accepted(req)) {
        return Err("Hathor held_skill_v1 requires gaze_v1 and tap_power_v1".into());
    }
    for (&index, hold) in &req.hold_inputs {
        if let Some(crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            ..
        }) = req.actions.get(index)
            && req.actors[*actor_index].character_id == "character:1025"
        {
            let branch = ability_id == "GA_Hathor_Melee"
                && super::branch::accepted(req)
                && req.branch_inputs.contains_key(&index);
            if (!accepted(req) && !branch)
                || (ability_id != "GA_Hathor_Skill" && !branch)
                || hold.moving
                || hold.reserved_stocks.is_some()
            {
                return Err("Hathor held Skill requires held_skill_v1 and canonical stationary unreserved Skill; held Normal requires branch_v1 and branch_inputs".into());
            }
        }
    }
    Ok(())
}
pub(super) fn plan(ctx: &mut CastContext<'_>) -> Result<Option<Plan>, String> {
    let Some(hold) = ctx.request.hold_inputs.get(&ctx.command_index) else {
        return Ok(None);
    };
    ctx.record_hypothesis(MODEL)?;
    if hold.duration_us < 220_000 {
        return Ok(None);
    }
    let release = hold.duration_us.min(5_000_000);
    let mut hits = Vec::new();
    let mut notifies = Vec::new();
    if BIRTH < release {
        notifies.push((BIRTH, Boundary::HeldBirth));
    }
    for k in 0..17 {
        let at = FIRST_TICK + k * 250_000;
        if at >= release {
            break;
        }
        hits.push(("GE_Player_Hathor_Skill1Power_Damage", at));
        notifies.push((at, Boundary::HeldTick));
    }
    let recovery = if release <= 1_000_000 {
        for (effect, offset) in [
            ("GE_Player_Hathor_Skill1_Damage", 233_361),
            ("GE_Player_Hathor_Skill1_Damage", 416_613),
            ("GE_Player_Hathor_Skill11_Damage", 633_453),
        ] {
            hits.push((effect, release + offset));
        }
        notifies.push((release, Boundary::TapPower));
        release + 1_150_000
    } else {
        for (effect, offset) in [
            ("GE_Player_Hathor_Skill2_Damage", 349_932),
            ("GE_Player_Hathor_Skill2_Damage", 533_278),
            ("GE_Player_Hathor_Skill2_Damage", 733_218),
            ("GE_Player_Hathor_Skill2_Damage", 849_830),
            ("GE_Player_Hathor_Skill3_Damage", 1_049_849),
            ("GE_Player_Hathor_Skill4_Damage", 1_616_267),
            ("GE_Player_Hathor_Skill4_Damage", 1_649_862),
            ("GE_Player_Hathor_Skill4_Damage", 1_683_020),
            ("GE_Player_Hathor_Skill4_Damage", 1_716_322),
            ("GE_Player_Hathor_Skill4_Damage", 1_749_575),
        ] {
            hits.push((effect, release + offset));
        }
        release + 2_049_823
    };
    // Release ends the held section before its coincident marker. ShortEnd
    // supplies its own marker only when the earlier one never actually fired.
    notifies.push((
        if release > 236_652 {
            236_652
        } else {
            release + 16_368
        },
        Boundary::SkillCooldown,
    ));
    Ok(Some(Plan {
        hits,
        recovery,
        notifies,
    }))
}
pub(super) fn birth(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let power = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .power
        .as_mut()
        .ok_or("Hathor held Power state missing")?;
    if ctx.request.actors[owner].awakening.has_effect(1) && !power.a1_used {
        power.a1_used = true;
        super::power::grant(ctx, owner, super::power::Cause::FirstHeld)?;
    }
    Ok(())
}
pub(super) fn tick(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    for target in 0..ctx.state.targets.len() {
        if ctx.state.targets[target].hp_current > 0 {
            super::gaze::enhance(ctx, owner, target)?;
        }
    }
    Ok(())
}
