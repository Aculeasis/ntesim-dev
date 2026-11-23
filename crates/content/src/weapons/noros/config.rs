pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::NorosAtk {
        atk_bonus,
        duration_us,
        cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_nonos") {
        return Err(format!(
            "actor {i} Noros proc requires equipped weapon:fork_nonos"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    if *duration_us == 0 || *cooldown_us == 0 {
        return Err("Noros requires positive duration and cooldown".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "noros_atk",
        duration_us: (*duration_us).max(*cooldown_us),
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
    cooldown_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::RedirectSkill) {
        t.checked_add((*duration_us).max(*cooldown_us))
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
