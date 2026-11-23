pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::Nakupeda {
        hp_bonus,
        lowest_hp_heal_ratio,
        shield_ratio,
        shield_duration_us,
        team_heal_ratio,
        cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Nakupeda") {
        return Err(format!(
            "actor {i} Nacupeda proc requires equipped weapon:fork_Nakupeda"
        ));
    }
    if ntesim_data::character::definition(&actor.character_id)?
        .weapon_group
        .native_key()
        != super::GROUP_ID
    {
        return Err("Nacupeda group is incompatible with the wearer".into());
    }
    ExactFraction::from_ratio(*hp_bonus)?;
    for ratio in [lowest_hp_heal_ratio, shield_ratio, team_heal_ratio] {
        let value = ExactFraction::from_ratio(*ratio)?;
        if value.numerator == 0 || value.numerator > value.denominator {
            return Err("Nacupeda effect ratios must be in (0, 1]".into());
        }
    }
    if *shield_duration_us == 0 || *cooldown_us == 0 {
        return Err("Nacupeda durations must be positive".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "nakupeda",
        duration_us: 0,
    })
}
