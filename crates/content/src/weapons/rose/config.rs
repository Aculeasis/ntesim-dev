pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::RoseThorn {
        atk_bonus,
        crit_damage_per_stack,
        duration_us,
        broken_extension_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Rose") {
        return Err(format!(
            "actor {i} Bohemian Rose proc requires equipped weapon:fork_Rose"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*crit_damage_per_stack)?;
    if *duration_us == 0 {
        return Err("Bohemian Rose requires a positive thorn duration".into());
    }
    if *broken_extension_us == 0 {
        return Err("Bohemian Rose requires a positive Broken extension".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "rose_thorn",
        duration_us: (*duration_us)
            .max(*broken_extension_us)
            .max(crate::weapons::rose::ICD_US),
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
    broken_extension_us: &u64,
) -> Result<(), String> {
    crate::gear::validation::validate_delivery_deadline(
        ctx,
        (*duration_us).max(*broken_extension_us).max(super::ICD_US),
    )
}
