pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::Moon {
        psyche_bonus,
        crit_damage_per_stack,
        duration_us,
        icd_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_moon") {
        return Err(format!(
            "actor {i} Stellar Veil proc requires equipped weapon:fork_moon"
        ));
    }
    ExactFraction::from_ratio(*psyche_bonus)?;
    ExactFraction::from_ratio(*crit_damage_per_stack)?;
    if *duration_us == 0 {
        return Err("Stellar Veil requires positive duration".into());
    }
    if *icd_us == 0 {
        return Err("Stellar Veil requires positive cooldown".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "moon",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let t = ctx.time_us;
    let hit_delay_us = ctx.hit_delay_us;
    let i = ctx.action_index;
    t.checked_add(hit_delay_us)
        .and_then(|hit_t| hit_t.checked_add(*duration_us))
        .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    Ok(())
}
