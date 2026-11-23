pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::ForgottenHpStats {
        atk_bonus,
        def_bonus,
    } = proc
    else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    let actor = ctx.actor;
    let i = ctx.actor_index;
    use crate::simulation::ExactFraction;
    if !actor
        .weapon_id
        .eq_ignore_ascii_case("weapon:fork_wuhuakuang")
    {
        return Err(format!(
            "actor {i} The Forgotten proc requires equipped weapon:fork_wuhuakuang"
        ));
    }
    ExactFraction::from_ratio(*atk_bonus)?;
    ExactFraction::from_ratio(*def_bonus)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "forgotten",
        duration_us: 0,
    })
}
