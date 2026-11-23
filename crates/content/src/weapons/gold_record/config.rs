pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::GoldRecord {
        atk_bonus,
        qte_crit_damage_bonus,
        ult_crit_damage_per_stack,
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
        .eq_ignore_ascii_case("weapon:fork_GoldRecord")
    {
        return Err(format!(
            "actor {i} Voice of the Voyager proc requires equipped weapon:fork_GoldRecord"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*qte_crit_damage_bonus)?;
    ExactFraction::from_ratio(*ult_crit_damage_per_stack)?;
    if *duration_us == 0 {
        return Err("Voice of the Voyager requires a positive duration".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "gold_record",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    duration_us: &u64,
) -> Result<(), String> {
    let action = ctx.action;
    let t = ctx.time_us;
    let hit_delay_us = ctx.hit_delay_us;
    let i = ctx.action_index;
    if action.input_role == Some(crate::simulation::metadata::InputActionRole::Qte) {
        t.checked_add(hit_delay_us)
            .and_then(|hit_t| hit_t.checked_add(*duration_us))
            .ok_or_else(|| format!("action {i} gear proc deadline overflow"))?;
    }
    Ok(())
}
