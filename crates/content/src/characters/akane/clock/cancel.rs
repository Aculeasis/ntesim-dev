use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Family {
    Jump,
    Skill,
    Movement,
}

fn jump_progress(kind: Kind, recovery: u64) -> u64 {
    match kind {
        Kind::Normal(0) | Kind::AppearNormal | Kind::MeleePre => 527_595,
        Kind::Normal(1) => 436_721,
        Kind::Normal(2) => 915_880,
        Kind::Normal(3) => 657_890,
        Kind::SkillPress => 422_076,
        Kind::SkillHold => 1_229_058,
        Kind::SyncedOvertone => 588_414,
        Kind::SyncedReverb => 1_229_058,
        Kind::BurstStage1 => 4_068_734,
        Kind::BurstStage2 => 1_526_601,
        Kind::Qte => 1_291_110,
        Kind::Plunge => 216_571,
        Kind::Riposte => 652_000,
        Kind::Dodge(true) => 166_629,
        Kind::Dodge(false) => 82_544,
        _ => recovery,
    }
}

fn skill_progress(kind: Kind, recovery: u64) -> u64 {
    match kind {
        Kind::Normal(0) | Kind::AppearNormal => 777_845,
        Kind::MeleePre => 886_565,
        Kind::Normal(1) => 492_799,
        Kind::Normal(2) => 1_024_168,
        Kind::Normal(3) => 672_824,
        Kind::Plunge => 216_571,
        Kind::Riposte => 652_000,
        Kind::Qte => 1_291_110,
        Kind::Dodge(true) => 383_426,
        Kind::Dodge(false) => 332_778,
        Kind::BurstStage1 => 4_050_620,
        Kind::BurstStage2 => 1_568_717,
        Kind::SyncedOvertone => 611_652,
        Kind::SyncedReverb => 1_309_354,
        Kind::SkillPress => 908_030,
        Kind::SkillHold => 1_309_354,
        _ => recovery,
    }
}

fn movement_progress(kind: Kind) -> u64 {
    match kind {
        Kind::Normal(0) | Kind::AppearNormal | Kind::MeleePre => 756_796,
        Kind::Normal(1) => 639_644,
        Kind::Normal(2) => 965_366,
        Kind::Normal(3) => 1_532_618,
        Kind::Plunge => 831_852,
        Kind::Riposte => 652_000,
        Kind::Qte => 1_291_110,
        Kind::SkillPress => 1_712_056,
        Kind::SkillHold => 1_422_213,
        Kind::SyncedOvertone => 646_508,
        Kind::SyncedReverb => 1_335_262,
        Kind::BurstStage1 => 4_068_734,
        Kind::BurstStage2 => 1_568_717,
        Kind::Dodge(true) => 583_301,
        Kind::Dodge(false) => profile::EVADE_FORWARD_RECOVERY,
        _ => 781_518,
    }
}

pub(crate) fn recovery(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    if !super::accepted(request) {
        return Ok(None);
    }
    let family = match ability {
        "jump" => Family::Jump,
        "dash" if super::super::secondary::accepted(request) => Family::Movement,
        "GA_Akane_Skill" | "GA_Akane_UltraSkill" => Family::Skill,
        _ => return Ok(None),
    };
    let model = match family {
        Family::Movement => super::super::secondary::MODEL,
        Family::Jump | Family::Skill => super::MODEL,
    };
    Ok(gate(actor, now, family)?.map(|deadline| (deadline, model)))
}

pub(crate) fn gate(actor: &ActorRuntime, now: u64, family: Family) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Akane(state) = &actor.character else {
        return Ok(None);
    };
    let Some(clock) = &state.clock else {
        return Ok(None);
    };
    let Some(action) = &clock.action else {
        return Ok(None);
    };
    let end = clock
        .tracker
        .resolve_real_deadline(action.owner, action.start, action.recovery)
        .map_err(|error| format!("Akane gate end deadline: {error:?}"))?;
    if now >= end {
        return Ok(None);
    }
    let progress = match family {
        Family::Jump => jump_progress(action.kind, action.recovery),
        Family::Skill => skill_progress(action.kind, action.recovery),
        Family::Movement => movement_progress(action.kind),
    };
    let gate = clock
        .tracker
        .resolve_real_deadline(action.owner, action.start, progress.min(action.recovery))
        .map_err(|error| format!("Akane gate deadline: {error:?}"))?;
    Ok(Some(gate))
}

pub(crate) fn on_jump(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    super::exit(ctx, owner)
}
