//! Paid finite Cyclone stages and their source-owned DEF carrier.
use crate::runtime_api::{CallbackContext, CastContext, DamageContext};
use crate::simulation::{ExactFraction as F, GameScenarioRequest, state::ActorRuntime};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorCycloneV1;
pub(super) const DEF: &str = "hathor.cyclone_def";
pub(super) const SOURCE: &str = "character:1025";
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if accepted(req) && !super::burst::accepted(req) {
        return Err("Hathor cyclone_v1 requires burst_emergency_v1".into());
    }
    Ok(())
}
pub(super) fn plan(stage: u8) -> super::held::Plan {
    use super::clock::Boundary;
    let (hits, recovery, success) = match stage {
        1 => (
            vec![
                ("GE_Player_Hathor_Skill5_Damage", 749_916),
                ("GE_Player_Hathor_Skill5_Damage", 800_024),
                ("GE_Player_Hathor_Skill5_Damage", 850_031),
                ("GE_Player_Hathor_Skill5_Damage", 900_033),
                ("GE_Player_Hathor_Skill5_Damage", 950_037),
                ("GE_Player_Hathor_Skill5_Damage", 1_000_019),
                ("GE_Player_Hathor_Skill5_Damage", 1_050_034),
            ],
            1_914_334,
            Some(1_066_650),
        ),
        2 => (
            vec![
                ("GE_Player_Hathor_Skill6_Damage", 516_624),
                ("GE_Player_Hathor_Skill7_Damage", 949_924),
                ("GE_Player_Hathor_Skill8_Damage", 1_266_449),
            ],
            2_169_196,
            Some(1_366_748),
        ),
        _ => (
            vec![
                ("GE_Player_Hathor_Skill9_Damage", 283_062),
                ("GE_Player_Hathor_Skill10_Damage", 766_666),
            ],
            1_499_682,
            None,
        ),
    };
    let mut notifies = vec![(recovery, Boundary::CycloneFinish)];
    if let Some(at) = success {
        notifies.push((at, Boundary::CycloneSuccess));
    }
    super::held::Plan {
        hits,
        recovery,
        notifies,
    }
}
pub(super) fn commit(ctx: &mut CastContext<'_>, stage: u8) -> Result<u8, String> {
    ctx.record_hypothesis(MODEL)?;
    let counter = super::burst::counter(ctx.actor);
    let state = ctx.actor.character.hathor_mut()?;
    if state.express_delivery_power < u32::from(stage) {
        return Err("Hathor Cyclone Power changed after admission".into());
    }
    let amount = F::new(stage.into(), 1)?;
    let paid = state
        .power
        .as_mut()
        .ok_or("Hathor Power missing")?
        .pool
        .spend_up_to(amount)?;
    if paid != amount {
        return Err("Hathor Cyclone Power ledger mismatch".into());
    }
    state.express_delivery_power -= u32::from(stage);
    ctx.actor.ability_cooldowns.remove("GA_Hathor_Skill");
    use ntesim_core::buffs::dispatch::*;
    ctx.buffs
        .apply_buff(
            BuffDefinition {
                id: DEF.into(),
                source: SOURCE.into(),
                recipient: BuffRecipient::SelfActor,
                duration_us: None,
                stacking_scope: BuffStackingScope::PerSource,
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                source_lifetime: BuffSourceLifetime::WhileAlive,
                semantic_tags: Default::default(),
                stat_modifiers: std::collections::BTreeMap::from([(
                    "def_percent".into(),
                    crate::formula::Ratio::one(),
                )]),
            },
            ctx.actor_index,
            ctx.world_time_us,
        )
        .map_err(|e| format!("Hathor Cyclone DEF: {e:?}"))?;
    Ok(counter)
}
pub(super) fn clear(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    ctx.state.buffs.remove_owned_buff(DEF, SOURCE, owner);
    super::burst::finish_cyclone(ctx, owner, "ModePause")
}
pub(crate) fn recovery(
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if ability != "GA_Hathor_Skill" || !super::burst::active(actor, now) {
        return None;
    }
    super::clock::combo_gate(actor, now).map(|gate| (gate, MODEL))
}
pub(crate) fn failure(
    actor: &ActorRuntime,
    now: u64,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if !super::burst::active(actor, now) {
        return None;
    }
    let stage = super::clock::combo_stage(actor, now);
    let power = match &actor.character {
        crate::characters::RuntimeState::Hathor(s) => s.express_delivery_power,
        _ => 0,
    };
    (power < u32::from(stage)).then(|| {
        let message = format!("Hathor Cyclone stage{stage} requires{stage} Power; current{power}");
        crate::simulation::rejection::AdmissionFailure {
            reason: crate::simulation::GameAdmissionReason::Other {
                message: message.clone(),
            },
            message,
        }
    })
}
pub(crate) fn crit(ctx: &DamageContext<'_>) -> Result<F, String> {
    if !packet(ctx.damage_effect_id)
        || !ctx.config.awakening.has_effect(4)
        || !super::burst::active(ctx.actor, ctx.time_us)
    {
        return F::new(0, 1);
    }
    F::new(
        super::clock::cyclone_counter(ctx.actor, ctx.command_index).into(),
        4,
    )
}
pub(crate) fn packet(effect: &str) -> bool {
    matches!(
        effect,
        "GE_Player_Hathor_Skill5_Damage"
            | "GE_Player_Hathor_Skill6_Damage"
            | "GE_Player_Hathor_Skill7_Damage"
            | "GE_Player_Hathor_Skill8_Damage"
            | "GE_Player_Hathor_Skill9_Damage"
            | "GE_Player_Hathor_Skill10_Damage"
    )
}
