pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::YurenHp {
        hp_bonus,
        shield_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_yuren") {
        return Err(format!(
            "actor {i} Rainman proc requires equipped weapon:fork_yuren"
        ));
    }
    ExactFraction::from_ratio(*hp_bonus)?;
    ExactFraction::from_ratio(*shield_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "yuren_hp",
        duration_us: 0,
    })
}
