use crate::HypothesisId;
use crate::runtime_api::CastContext;
use crate::simulation::{ExactFraction, GameEventPayload, GearProcConfig};
use ntesim_core::buffs::dispatch::BuffRecipient;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::SetsLostRadianceV1;

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.action_metadata.input_role
        != Some(crate::simulation::metadata::InputActionRole::Ultimate)
    {
        return Ok(());
    }
    let config = ctx.config;
    for proc in &config.gear_procs {
        if let GearProcConfig::LostRadianceDefIgnore {
            def_ignore,
            duration_us,
        } = proc
        {
            ctx.record_hypothesis(HYPOTHESIS_ID)?;
            let def_ignore_exact = ExactFraction::from_ratio(*def_ignore)?;
            let grant = crate::modifiers::BuffGrantBuilder::new(
                ctx.actor_index,
                "lost_radiance.def_ignore",
                "set:Suit6",
            )
            .recipient(BuffRecipient::SelfActor)
            .duration_us(*duration_us)
            .stat("def_ignore", def_ignore_exact)
            .receipt(
                "lost_radiance_def_ignore",
                GameEventPayload {
                    def_ignore: Some(def_ignore_exact),
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
    let crate::simulation::GearProcConfig::LostRadianceDefIgnore {
        def_ignore,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if actor.set_id.as_deref() != Some("set:Suit6") {
        return Err(format!(
            "actor {i} Lost Radiance proc requires equipped set:Suit6"
        ));
    }
    ExactFraction::from_ratio(*def_ignore)?;
    if *duration_us == 0 {
        return Err("Lost Radiance requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "lost_radiance",
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
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::Ultimate) {
        t.checked_add(*duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
