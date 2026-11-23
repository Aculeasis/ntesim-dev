pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::CastleHealingBonus {
        healing_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Castle") {
        return Err(format!(
            "actor {i} Call of the Twisted City proc requires equipped weapon:fork_Castle"
        ));
    }
    ExactFraction::from_ratio(*healing_bonus)?;
    if *duration_us == 0 {
        return Err("Call of the Twisted City requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "castle_healing_bonus",
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
            .ok_or_else(|| format!("action {i} Call of the Twisted City deadline overflow"))?;
    }
    Ok(())
}
