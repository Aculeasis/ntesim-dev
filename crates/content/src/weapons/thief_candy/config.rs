pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::ThiefCandyDodge {
        damage_bonus,
        duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_ThiefCandy")
    {
        return Err(format!(
            "actor {i} Thief's Candy proc requires equipped weapon:fork_ThiefCandy"
        ));
    }
    ExactFraction::from_ratio(*damage_bonus)?;
    if *duration_us == 0 {
        return Err("Thief's Candy requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "thief_candy",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    crate::gear::validation::validate_cast_deadline(ctx, *duration_us)
}
