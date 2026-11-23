pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::BitterCake {
        def_bonus,
        duration_us,
        cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_BitterCake")
    {
        return Err(format!(
            "actor {i} The Good, The Bad, The Bitter proc requires equipped weapon:fork_BitterCake"
        ));
    }
    ExactFraction::from_ratio(*def_bonus)?;
    if *duration_us == 0 || *cooldown_us == 0 {
        return Err("The Good, The Bad, The Bitter requires positive duration and cooldown".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "bittercake",
        duration_us: (*duration_us).max(*cooldown_us),
    })
}

pub(crate) fn validate_deadline(
    _ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    if *duration_us == 0 {
        return Err("Bittercake requires a positive duration".into());
    }
    Ok(())
}
