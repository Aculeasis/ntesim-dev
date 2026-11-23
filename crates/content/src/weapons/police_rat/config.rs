pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::PoliceRatBoss {
        atk_bonus,
        boss_damage_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_PoliceRat")
    {
        return Err(format!(
            "actor {i} Justice Executioner proc requires equipped weapon:fork_PoliceRat"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*boss_damage_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "police_rat_boss",
        duration_us: 0,
    })
}
