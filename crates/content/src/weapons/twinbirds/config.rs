pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::Twinbirds {
        cycle_intensity_bonus,
        psyche_bonus,
        team_crit_damage_bonus,
        self_crit_damage_bonus,
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
        .eq_ignore_ascii_case("weapon:fork_twinbirds")
    {
        return Err(format!(
            "actor {i} Twinflight proc requires equipped weapon:fork_twinbirds"
        ));
    }
    ExactFraction::from_ratio(*psyche_bonus)?;
    ExactFraction::from_ratio(*cycle_intensity_bonus)?;
    ExactFraction::from_ratio(*team_crit_damage_bonus)?;
    ExactFraction::from_ratio(*self_crit_damage_bonus)?;
    if *duration_us == 0 {
        return Err("Twinflight requires positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "twinbirds",
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
