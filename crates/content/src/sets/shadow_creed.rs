use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::{ExactFraction, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::BuffRecipient;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsShadowCreedV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role
        != Some(crate::simulation::metadata::InputActionRole::RedirectSkill)
    {
        return Ok(());
    }
    let config = ctx.config;
    if ctx.config.character_id == "character:1070"
        && ctx
            .request
            .model_assumptions
            .contains_key(HypothesisId::AureliaShadowCreedSkillFinishV1.as_str())
    {
        return Ok(());
    }
    for proc in &config.gear_procs {
        if let GearProcConfig::ShadowCreedAtk {
            atk_bonus,
            duration_us,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            let atk_percent = ExactFraction::from_ratio(*atk_bonus)?;
            let grant = crate::modifiers::BuffGrantBuilder::new(
                ctx.actor_index,
                "shadow_creed.atk",
                "set:Suit8",
            )
            .recipient(BuffRecipient::SelfActor)
            .duration_us(*duration_us)
            .stat("atk_percent", atk_percent)
            .receipt(
                "shadow_creed_atk",
                GameEventPayload {
                    atk_percent: Some(atk_percent),
                    duration_us: Some(*duration_us),
                    ..Default::default()
                },
            )
            .into_event();
            crate::gear::schedule_cast_buff(ctx, grant)?;
        }
    }
    Ok(())
}

pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::ShadowCreedAtk {
        atk_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if actor.set_id.as_deref() != Some("set:Suit8") {
        return Err(format!(
            "actor {i} Shadow Creed proc requires equipped set:Suit8"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    if *duration_us == 0 {
        return Err("Shadow Creed requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "shadow_creed",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::RedirectSkill) {
        t.checked_add(*duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
