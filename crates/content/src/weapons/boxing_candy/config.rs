pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::BoxingCandyDamage {
        damage_bonus,
        low_hp_damage_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_BoxingCandy")
    {
        return Err(format!(
            "actor {i} Boxing Candy proc requires equipped weapon:fork_BoxingCandy"
        ));
    }
    let ordinary = ExactFraction::from_ratio(*damage_bonus)?;
    let enhanced = ExactFraction::from_ratio(*low_hp_damage_bonus)?;
    enhanced
        .sub(ordinary)
        .map_err(|_| "Boxing Candy enhanced bonus cannot be below its ordinary bonus")?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "boxing_candy",
        duration_us: 0,
    })
}
