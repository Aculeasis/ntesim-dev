pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::TigerTallyAtk {
        atk_bonus,
        normal_damage_bonus,
        duration_us,
        talisman_window_us,
        boss_damage_bonus,
        boss_damage_duration_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_TigerTally")
    {
        return Err(format!(
            "actor {i} TigerTally proc requires equipped weapon:fork_TigerTally"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*normal_damage_bonus)?;
    ExactFraction::from_ratio(*boss_damage_bonus)?;
    if [*duration_us, *talisman_window_us, *boss_damage_duration_us].contains(&0) {
        return Err("Commander Tiger Talisman requires positive durations".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "tiger_tally",
        duration_us: *duration_us,
    })
}

pub(crate) fn validate_deadline(
    ctx: &crate::gear::validation::DeadlineContext<'_>,
    proc: &crate::simulation::GearProcConfig,
) -> Result<(), String> {
    let crate::simulation::GearProcConfig::TigerTallyAtk {
        duration_us,
        talisman_window_us,
        boss_damage_duration_us,
        ..
    } = proc
    else {
        return Err("deadline validator received the wrong proc kind".into());
    };
    for duration in [duration_us, talisman_window_us, boss_damage_duration_us] {
        crate::gear::validation::validate_cast_deadline(ctx, *duration)?;
    }
    Ok(())
}
