pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::WorldrainDamage {
        damage_bonus,
        cycle_intensity_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_Worldrain")
    {
        return Err(format!(
            "actor {i} Worldrain proc requires equipped weapon:fork_Worldrain"
        ));
    }
    ExactFraction::from_ratio(*damage_bonus)?;
    ExactFraction::from_ratio(*cycle_intensity_bonus)?;
    if *duration_us == 0 {
        return Err("Worldrain requires positive CI duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "worldrain",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    if ctx.action.input_role == Some(crate::simulation::metadata::InputActionRole::RedirectSkill) {
        ctx.time_us
            .checked_add(*duration_us)
            .ok_or_else(|| format!("action {} Worldrain deadline overflow", ctx.action_index))?;
    }
    Ok(())
}
