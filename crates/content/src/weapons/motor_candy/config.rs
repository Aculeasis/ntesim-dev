pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::MotorCandyAtk {
        atk_per_stack,
        interval_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_MotorCandy")
    {
        return Err(format!(
            "actor {i} Motor Candy proc requires equipped weapon:fork_MotorCandy"
        ));
    }
    ExactFraction::from_ratio(*atk_per_stack)?;
    if *interval_us == 0 {
        return Err("Motor Candy requires a positive interval".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "motor_candy",
        duration_us: 0,
    })
}
