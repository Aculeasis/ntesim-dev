pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::ApplianceDamage { damage_bonus } = proc else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_Appliance")
    {
        return Err(format!(
            "actor {i} Appliance proc requires equipped weapon:fork_Appliance"
        ));
    }
    ExactFraction::from_ratio(*damage_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "appliance",
        duration_us: 0,
    })
}
