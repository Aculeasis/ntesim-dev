use super::*;

fn required_attribute(attributes: &BTreeMap<String, Scalar>, name: &str) -> Result<Scalar, String> {
    attributes
        .get(name)
        .copied()
        .ok_or_else(|| format!("required source-bound attribute missing: {name}"))
}

fn modifier_or_zero(attributes: &BTreeMap<String, Scalar>, name: &str) -> Scalar {
    // These are additive modifiers. An omitted modifier pack entry means no
    // contribution; base values below are never inferred from absence.
    attributes.get(name).copied().unwrap_or(Scalar::zero())
}

pub(crate) fn derive_stats(
    known_attributes: &BTreeMap<String, Scalar>,
    total_base_atk_scalar: Scalar,
) -> Result<Value, String> {
    let hp_base = required_attribute(known_attributes, "HPMaxBase")?;
    let hp_up = modifier_or_zero(known_attributes, "HPMaxUp");
    let hp_add = modifier_or_zero(known_attributes, "HPMaxAdd");
    let hp_derived = hp_base.mul(Scalar::one().add(hp_up)?)?.add(hp_add)?;

    let atk_up = modifier_or_zero(known_attributes, "AtkUp");
    let atk_add = modifier_or_zero(known_attributes, "AtkAdd");
    let atk_derived = total_base_atk_scalar
        .mul(Scalar::one().add(atk_up)?)?
        .add(atk_add)?;

    let def_base = required_attribute(known_attributes, "DefBase")?;
    let def_up = modifier_or_zero(known_attributes, "DefUp");
    let def_add = modifier_or_zero(known_attributes, "DefAdd");
    let def_derived = def_base.mul(Scalar::one().add(def_up)?)?.add(def_add)?;

    let unbal_base = required_attribute(known_attributes, "UnbalIntensityBase")?;
    let unbal_up = modifier_or_zero(known_attributes, "UnbalIntensityUp");
    let unbal_add = modifier_or_zero(known_attributes, "UnbalIntensityAdd");
    let unbal_derived = unbal_base
        .mul(Scalar::one().add(unbal_up)?)?
        .add(unbal_add)?;

    let crit_rate_derived = required_attribute(known_attributes, "CritBase")?;
    let crit_dmg_derived = required_attribute(known_attributes, "CritDamageBase")?;
    let charge_eff_derived = required_attribute(known_attributes, "ChargeGetEfficiencyBase")?;

    Ok(json!({
        "hp": hp_derived.to_f64(),
        "atk": atk_derived.to_f64(),
        "def": def_derived.to_f64(),
        "break_intensity": unbal_derived.to_f64(),
        "crit_rate": crit_rate_derived.to_f64(),
        "crit_damage": crit_dmg_derived.to_f64(),
        "charge_efficiency": charge_eff_derived.to_f64(),
        "hp_receipt": hp_derived.receipt(),
        "atk_receipt": atk_derived.receipt(),
        "def_receipt": def_derived.receipt(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_attributes() -> BTreeMap<String, Scalar> {
        [
            ("HPMaxBase", Scalar::new(1000, 1)),
            ("AtkUp", Scalar::new(1, 10)),
            ("DefBase", Scalar::new(100, 1)),
            ("UnbalIntensityBase", Scalar::new(30, 1)),
            ("CritBase", Scalar::new(1, 4)),
            ("CritDamageBase", Scalar::new(1, 2)),
            ("ChargeGetEfficiencyBase", Scalar::new(1, 1)),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
    }

    #[test]
    fn omitted_additive_modifiers_mean_zero_but_missing_source_base_fails_closed() {
        let attributes = complete_attributes();
        let result = derive_stats(&attributes, Scalar::new(100, 1)).unwrap();
        assert_eq!(result["hp"], 1000.0);
        assert_eq!(result["atk"], 110.0);
        assert_eq!(result["def"], 100.0);
        assert_eq!(result["break_intensity"], 30.0);
        assert_eq!(result["charge_efficiency"], 1.0);

        for missing in [
            "HPMaxBase",
            "DefBase",
            "UnbalIntensityBase",
            "CritBase",
            "CritDamageBase",
            "ChargeGetEfficiencyBase",
        ] {
            let mut incomplete = attributes.clone();
            incomplete.remove(missing);
            assert!(
                derive_stats(&incomplete, Scalar::new(100, 1))
                    .unwrap_err()
                    .contains(missing)
            );
        }
    }
}
