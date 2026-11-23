pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::WhaleAtk {
        atk_bonus,
        broken_damage_bonus,
        kill_heal,
        kill_heal_cooldown_us,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Whale") {
        return Err(format!(
            "actor {i} Song of the Whale proc requires equipped weapon:fork_Whale"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*broken_damage_bonus)?;
    ExactFraction::from_ratio(*kill_heal)?;
    if *kill_heal_cooldown_us == 0 {
        return Err("Song of the Whale requires a positive kill-heal cooldown".into());
    }
    Ok(crate::gear::validation::ProcValidation {
        kind: "whale_atk",
        duration_us: 0,
    })
}
