pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::MarchingBeyondTime {
        atk_bonus,
        state_crit_damage_bonus,
        crit_damage_per_stack,
        def_ignore,
        def_ignore_duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Time") {
        return Err(format!(
            "actor {i} Marching Beyond Time proc requires equipped weapon:fork_Time"
        ));
    }
    for value in [
        atk_bonus,
        state_crit_damage_bonus,
        crit_damage_per_stack,
        def_ignore,
    ] {
        ExactFraction::from_ratio(*value)?;
    }
    if *def_ignore_duration_us == 0 {
        return Err("Marching Beyond Time requires a positive DEF ignore duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "marching_beyond_time",
        duration_us: *def_ignore_duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    def_ignore_duration_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::Ultimate) {
        t.checked_add(*def_ignore_duration_us)
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
