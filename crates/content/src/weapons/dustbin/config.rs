pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::DustbinBreak {
        break_intensity_bonus,
        duration_us,
        cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_dustbin") {
        return Err(format!(
            "actor {i} Decomposer proc requires equipped weapon:fork_dustbin"
        ));
    }
    ExactFraction::from_ratio(*break_intensity_bonus)?;
    if *duration_us == 0 {
        return Err("Decomposer requires a positive duration".into());
    }
    if *cooldown_us == 0 {
        return Err("Decomposer requires a positive trigger cooldown".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "dustbin",
        duration_us: (*duration_us).max(*cooldown_us),
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
    cooldown_us: &u64,
) -> Result<(), String> {
    crate::gear::validation::validate_delivery_deadline(ctx, (*duration_us).max(*cooldown_us))
}
