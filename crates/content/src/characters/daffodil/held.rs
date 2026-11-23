//! Canonical held Normal and paid Displaced Edge retain the owner montage.
use super::clock::{self, Kind};
use crate::runtime_api::NormalActionOverride;
use crate::simulation::{GameActionCommand, GameScenarioRequest, state::ActorRuntime};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::DaffodilHeldNormalV1;
pub(crate) const PRIMARY: &str = "GE_Player_Daffodill_Branch5_Damage";
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if accepted(req) && !super::secondary::accepted(req) {
        return Err("Daffodil held/suspended Normal requires secondary_actions_v1".into());
    }
    for (&command, hold) in &req.hold_inputs {
        let Some(GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            ..
        }) = req.actions.get(command)
        else {
            continue;
        };
        if req.actors[*actor_index].character_id != "character:1054" {
            continue;
        }
        if !accepted(req)
            || ability_id != "GA_Daffodill_Melee"
            || damage_effect_id != "GE_Player_Daffodill_Melee1_Damage"
            || hold.moving
            || hold.reserved_stocks.is_some()
            || req.plunge_inputs.contains_key(&command)
        {
            return Err("Daffodil held Normal requires its canonical Normal, held_normal_v1 and no moving/stocks/plunge branch".into());
        }
    }
    Ok(())
}
pub(crate) fn replacement(
    req: &GameScenarioRequest,
    _actor: &ActorRuntime,
    _now: u64,
) -> Result<Option<NormalActionOverride>, String> {
    if !accepted(req) {
        return Err("Daffodil held Normal requires held_normal_v1".into());
    }
    Ok(Some(NormalActionOverride {
        packet: PRIMARY,
        role: None,
        model: MODEL,
        timing: Some((1_433_323, 104_054)),
    }))
}
pub(crate) fn recovery(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    command: usize,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !accepted(req) || !req.hold_inputs.contains_key(&command) {
        return None;
    }
    clock::normal_jump_gate(actor, now).map(|gate| (gate, MODEL))
}
/// Observe the same combo gate as actual held-Normal admission.
pub(crate) fn input_window(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    now: u64,
) -> Result<u64, String> {
    if !accepted(req) {
        return Err("Daffodil hold input window requires held_normal_v1".into());
    }
    Ok(clock::normal_jump_gate(actor, now).unwrap_or(actor.locked_until_us))
}
pub(crate) fn selected(ctx: &crate::runtime_api::CastContext<'_>) -> bool {
    accepted(ctx.request)
        && ctx.request.hold_inputs.contains_key(&ctx.command_index)
        && ctx
            .packet_metadata
            .is_some_and(|p| p.damage_effect_id == PRIMARY)
}
pub(super) const HITS: &[clock::profile::Hit] = &[
    (PRIMARY, 104_054),
    ("GE_Player_Daffodill_Branch4_Damage", 400_088),
    ("GE_Player_Daffodill_Branch1_Damage", 400_780),
    ("GE_Player_Daffodill_Branch2_Damage", 784_106),
    ("GE_Player_Daffodill_Branch6_Damage", 833_130),
    ("GE_Player_Daffodill_Branch6_Damage", 892_724),
    ("GE_Player_Daffodill_Branch7_Damage", 935_062),
    ("GE_Player_Daffodill_Branch7_Damage", 1_007_713),
    ("GE_Player_Daffodill_Branch3_Damage", 1_208_015),
];
pub(crate) fn kind(
    ctx: &mut crate::runtime_api::CastContext<'_>,
) -> Result<Option<(Kind, &'static [clock::profile::Hit], u64)>, String> {
    if !selected(ctx) {
        return Ok(None);
    }
    ctx.record_hypothesis(MODEL)?;
    ctx.actor.character.daffodil_mut()?.riposte_ready = false;
    Ok(Some((Kind::Branch, HITS, 0)))
}
