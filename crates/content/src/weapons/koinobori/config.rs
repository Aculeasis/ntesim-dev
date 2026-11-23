pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::KoinoboriStats {
        atk_bonus,
        def_bonus,
        hp_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_koinobori")
    {
        return Err(format!(
            "actor {i} A Time Will Come proc requires equipped weapon:fork_koinobori"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*def_bonus)?;
    ExactFraction::from_ratio(*hp_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "koinobori",
        duration_us: 0,
    })
}
