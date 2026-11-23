pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::BlackBookBreak {
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
        .eq_ignore_ascii_case("weapon:fork_BlackBook")
    {
        return Err(format!(
            "actor {i} Black Tome proc requires equipped weapon:fork_BlackBook"
        ));
    }
    ExactFraction::from_ratio(*break_intensity_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "black_book",
        duration_us: 0,
    })
}
