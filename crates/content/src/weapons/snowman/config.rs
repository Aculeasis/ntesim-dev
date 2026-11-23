pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::SnowmanShieldAtk { atk_bonus } = proc else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_snowman") {
        return Err(format!(
            "actor {i} Solifugae Vita proc requires equipped weapon:fork_snowman"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "snowman",
        duration_us: 0,
    })
}
