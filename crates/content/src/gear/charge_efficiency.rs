pub(crate) fn validate_config(
    proc: &crate::simulation::GearProcConfig,
    _ctx: &crate::gear::validation::ConfigContext<'_>,
) -> Result<crate::gear::validation::ProcValidation, String> {
    let crate::simulation::GearProcConfig::ChargeEfficiency { multiplier } = proc else {
        return Err("equipment validator received the wrong proc kind".into());
    };
    validate_multiplier(*multiplier)?;
    Ok(crate::gear::validation::ProcValidation {
        kind: "charge_efficiency",
        duration_us: 0,
    })
}

pub(crate) fn validate_multiplier(multiplier: crate::formula::Ratio) -> Result<(), String> {
    let value = crate::simulation::ExactFraction::from_ratio(multiplier)?;
    if value.numerator == 0 || value.numerator > 100 * value.denominator {
        return Err("Charge efficiency requires a positive multiplier <= 100".into());
    }
    Ok(())
}
