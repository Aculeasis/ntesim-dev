pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::TuansanlangTeam {
        break_intensity_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_tuansanlang")
    {
        return Err(format!(
            "actor {i} Danzaburou proc requires equipped weapon:fork_tuansanlang"
        ));
    }
    let bonus = ExactFraction::from_ratio(*break_intensity_bonus)?;
    if bonus.numerator == 0 {
        return Err("Danzaburou requires a positive Break Intensity bonus".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "tuansanlang",
        duration_us: 0,
    })
}
