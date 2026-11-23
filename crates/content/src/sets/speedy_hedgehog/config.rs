pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::SpeedyHedgehogTeamAtk {
        team_atk_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if actor.set_id.as_deref() != Some("set:Suit11") {
        return Err(format!(
            "actor {i} Speedy Hedgehog proc requires equipped set:Suit11"
        ));
    }
    ExactFraction::from_ratio(*team_atk_bonus)?;
    if *duration_us == 0 {
        return Err("Speedy Hedgehog requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "speedy_hedgehog",
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
