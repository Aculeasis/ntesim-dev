use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Family {
    Jump,
    Skill,
    Movement,
}

fn jump_progress(kind: Kind, recovery: u64) -> u64 {
    match kind {
        Kind::Normal(4) => 769_057,
        Kind::Crosstalk(1) => 561_142,
        Kind::Crosstalk(2) => 933_511,
        Kind::Branch => 549_787,
        Kind::Aim => profile::AIM_RECOVERY,
        Kind::Plunge => 123_557,
        Kind::Riposte => 0,
        Kind::Dodge(true) => 383_426,
        Kind::Dodge(false) => 332_778,
        Kind::Qte => 1_034_234,
        Kind::Skill1 => 1_084_427,
        Kind::Skill2 => 1_666_549,
        Kind::Skill3 { .. } => 1_666_667,
        Kind::BurstA => 5_322_640,
        Kind::BurstB => 5_316_667,
        _ => recovery,
    }
}

fn skill_progress(kind: Kind, recovery: u64) -> u64 {
    match kind {
        Kind::Normal(0) => 421_018,
        Kind::Normal(1) => 459_984,
        Kind::Normal(2) => 1_274_223,
        Kind::Normal(3) => 812_723,
        Kind::Normal(4) => 1_038_993,
        Kind::Crosstalk(0) => 452_851,
        Kind::Crosstalk(1) => 383_873,
        Kind::Crosstalk(2) => 933_511,
        Kind::Branch => 650_035,
        Kind::Aim => profile::AIM_RECOVERY,
        Kind::Plunge => 255_141,
        Kind::Riposte => 0,
        Kind::Dodge(true) => 383_426,
        Kind::Dodge(false) => 332_778,
        Kind::Qte => 1_034_234,
        Kind::Skill1 => 483_052,
        Kind::Skill2 => 1_666_549,
        Kind::Skill3 { .. } => 1_666_667,
        Kind::BurstA => 5_322_640,
        Kind::BurstB => 5_316_667,
        _ => recovery,
    }
}

fn movement_progress(kind: Kind, recovery: u64) -> u64 {
    match kind {
        Kind::Normal(0) => 599_339,
        Kind::Normal(1) => 779_794,
        Kind::Normal(2) => 1_598_705,
        Kind::Normal(3) => 1_966_649,
        Kind::Normal(4) => 1_295_037,
        Kind::Crosstalk(0) => 984_953,
        Kind::Crosstalk(1) => 1_067_342,
        Kind::Crosstalk(2) => 1_415_258,
        Kind::Branch => 650_035,
        Kind::Aim => 0,
        Kind::Plunge => 749_160,
        Kind::Riposte => 652_065,
        Kind::Dodge(true) => 583_301,
        Kind::Dodge(false) => profile::EVADE_FORWARD_RECOVERY,
        Kind::Qte => 1_240_934,
        Kind::Skill1 => 1_003_819,
        Kind::Skill2 => 1_666_549,
        Kind::Skill3 { .. } => 1_666_667,
        Kind::BurstA => 5_484_152,
        Kind::BurstB => 5_483_333,
        _ => recovery,
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
        "GA_Elyms_Skill" | "GA_Elyms_UltraSkill" => Family::Skill,
        _ => return Ok(None),
    };
    let model = match family {
        Family::Movement => super::super::secondary::MODEL,
        Family::Jump | Family::Skill => super::MODEL,
    };
    Ok(gate(actor, now, family)?.map(|deadline| (deadline, model)))
}

pub(crate) fn gate(actor: &ActorRuntime, now: u64, family: Family) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Elyms(state) = &actor.character else {
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
        .map_err(|error| format!("Elyms gate end deadline: {error:?}"))?;
    if now >= end {
        return Ok(None);
    }
    let progress = match family {
        Family::Jump => jump_progress(action.kind, action.recovery),
        Family::Skill => skill_progress(action.kind, action.recovery),
        Family::Movement => movement_progress(action.kind, action.recovery),
    };
    let gate = clock
        .tracker
        .resolve_real_deadline(action.owner, action.start, progress.min(action.recovery))
        .map_err(|error| format!("Elyms gate deadline: {error:?}"))?;
    Ok(Some(gate))
}

pub(crate) fn on_jump(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    super::exit(ctx, owner)
}
