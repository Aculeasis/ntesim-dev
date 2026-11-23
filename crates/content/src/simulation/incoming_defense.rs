//! Optional explicit DEF stage before the existing incoming/share/shield settlement.
use super::{
    ExactFraction as F, GameIncomingDamageBasis, GameIncomingDefenseReceipt, GameScenarioRequest,
};
use crate::runtime_api::CallbackContext;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::CombatIncomingDefenseV1;
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(super) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    for (&index, basis) in &req.incoming_damage_basis {
        if index >= req.incoming_damage_events.len() {
            return Err("incoming damage basis event index out of bounds".into());
        }
        if *basis == GameIncomingDamageBasis::BeforeDefense {
            if !accepted(req) {
                return Err(
                    "before_defense incoming damage requires combat.incoming_defense_v1".into(),
                );
            }
            let target = *req
                .incoming_dummy_sources
                .get(&index)
                .ok_or("before_defense incoming damage requires an explicit dummy source")?;
            if req.targets.get(target).is_none_or(|t| t.level == 0) {
                return Err(
                    "before_defense incoming dummy requires a positive declared level".into(),
                );
            }
        }
    }
    Ok(())
}
pub(super) fn evaluate(
    ctx: &mut CallbackContext<'_>,
    index: usize,
    actor: usize,
    amount: u64,
) -> Result<Option<(GameIncomingDefenseReceipt, u64)>, String> {
    if ctx.request.incoming_damage_basis.get(&index)
        != Some(&GameIncomingDamageBasis::BeforeDefense)
    {
        return Ok(None);
    }
    let target = *ctx
        .request
        .incoming_dummy_sources
        .get(&index)
        .ok_or("incoming defense source missing")?;
    let level = ctx.request.targets[target].level;
    let defense = ctx.live_def(actor)?;
    let zero = F::new(0, 1)?;
    let factor = ntesim_core::combat::exact::defense_factor(level, defense, zero, zero)?;
    let delivered = F::new(amount.into(), 1)?.mul(factor)?.round_u64()?;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    Ok(Some((
        GameIncomingDefenseReceipt {
            damage_before_defense: amount,
            source_target_index: target,
            source_level: level,
            recipient_defense: defense,
        },
        delivered,
    )))
}
