pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::NestBirdGaze {
        damage_reduction,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_NestBird") {
        return Err(format!(
            "actor {i} Tears Beneath the Mask proc requires equipped weapon:fork_NestBird"
        ));
    }
    let reduction = ExactFraction::from_ratio(*damage_reduction)?;
    if reduction.is_zero() || reduction.numerator > reduction.denominator {
        return Err("Tears Beneath the Mask requires a damage reduction ratio in (0, 1]".into());
    }
    if *duration_us == 0 {
        return Err("Tears Beneath the Mask requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "nest_bird",
        duration_us: *duration_us,
    })
}
