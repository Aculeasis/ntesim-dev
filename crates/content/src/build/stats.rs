use super::*;

pub(super) const STATIC_ATTRIBUTES: &[&str] = &[
    "AtkBase",
    "AtkUp",
    "AtkAdd",
    "HPMaxBase",
    "HPMaxUp",
    "HPMaxAdd",
    "DefBase",
    "DefUp",
    "DefAdd",
    "CritBase",
    "CritDamageBase",
    "HealUp",
    "ShieldEfficiency",
    "UnbalAccrueEfficiencyBase",
    "UnbalIntensityBase",
    "MagBase",
    "DamageUpPsychicallyBase",
    "DamageUpGeneralBase",
    "DamageUpChaosBase",
    "DamageUpCosmosBase",
    "DamageUpIncantationBase",
    "DamageUpLakshanaBase",
    "DamageUpNatureBase",
    "DamageUpPsycheBase",
    "ChargeMax",
    "ChargeGetEfficiencyBase",
];

pub const ELEMENT_ATTRIBUTES: [(&str, &str); 6] = [
    ("CHAOS", "DamageUpChaosBase"),
    ("COSMOS", "DamageUpCosmosBase"),
    ("INCANTATION", "DamageUpIncantationBase"),
    ("LAKSHANA", "DamageUpLakshanaBase"),
    ("NATURE", "DamageUpNatureBase"),
    ("PSYCHE", "DamageUpPsycheBase"),
];

pub fn element_attribute(element: &str) -> Result<&'static str, String> {
    match element {
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS" => Ok("DamageUpChaosBase"),
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS" => Ok("DamageUpCosmosBase"),
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_INCANTATION" => {
            Ok("DamageUpIncantationBase")
        }
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA" => Ok("DamageUpLakshanaBase"),
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE" => Ok("DamageUpNatureBase"),
        "ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE" => Ok("DamageUpPsycheBase"),
        _ => Err("unsupported or missing native character element".into()),
    }
}

pub fn project(report: &Value, element: &str) -> Result<GameActorStaticStats, String> {
    if !ELEMENT_ATTRIBUTES
        .iter()
        .any(|(_, property)| *property == element)
    {
        return Err("unsupported native element attribute".into());
    }
    let attribute = |name: &str| receipt(&report["known_attributes"][name]["receipt"]);
    if !attribute("DamageUpNormalBase")?.is_zero() {
        return Err("unsupported typed build damage bonus DamageUpNormalBase".into());
    }
    let derived = &report["known_derived_stats"];
    let hp_percent = attribute("HPMaxUp")?;
    let hp_flat = attribute("HPMaxAdd")?;
    Ok(GameActorStaticStats {
        base_atk: integer(&report["base_atk"], "base ATK")?,
        atk: ratio(receipt(&derived["atk_receipt"])?)?,
        hp: nearest_integer(&derived["hp_receipt"], "HP")?,
        base_hp: Some(integer(
            &report["known_attributes"]["HPMaxBase"]["receipt"],
            "base HP",
        )?),
        hp_percent: ratio(hp_percent)?,
        hp_flat: ratio(hp_flat)?,
        def: nearest_integer(&derived["def_receipt"], "DEF")?,
        base_def: Some(integer(
            &report["known_attributes"]["DefBase"]["receipt"],
            "base DEF",
        )?),
        def_percent: ratio(attribute("DefUp")?)?,
        def_flat: ratio(attribute("DefAdd")?)?,
        crit_rate: ratio(attribute("CritBase")?)?,
        crit_damage_bonus: ratio(attribute("CritDamageBase")?)?,
        mental_damage_bonus: attribute("DamageUpPsychicallyBase").and_then(ratio)?,
        healing_bonus: report["known_attributes"]
            .get("HealUp")
            .map(|value| ratio(receipt(&value["receipt"])?))
            .transpose()?
            .unwrap_or(Ratio::zero()),
        shield_bonus: project_shield_bonus(report)?,
        break_intensity: ratio(attribute("UnbalIntensityBase")?)?,
        cycle_intensity: report["known_attributes"]
            .get("MagBase")
            .map(|value| ratio(receipt(&value["receipt"])?))
            .transpose()?
            .unwrap_or(Ratio::zero()),
        break_efficiency_multiplier: ratio(attribute("UnbalAccrueEfficiencyBase")?)?,
        damage_bonus: ratio(attribute("DamageUpGeneralBase")?)?,
        elemental_bonuses: ELEMENT_ATTRIBUTES
            .iter()
            .map(|(element, property)| Ok(((*element).into(), ratio(attribute(property)?)?)))
            .collect::<Result<_, String>>()?,
        elemental_bonus: Ratio::zero(),
    })
}

pub(crate) fn project_shield_bonus(report: &Value) -> Result<Ratio, String> {
    let Some(attribute) = report["known_attributes"].get("ShieldEfficiency") else {
        return Ok(Ratio::zero());
    };
    ratio(receipt(&attribute["receipt"])?)
}

pub(super) fn project_equipped_charge_efficiency(
    tables: &BTreeMap<String, Value>,
    content_id: &str,
    actor: &mut crate::simulation::GameActorConfig,
    report: &Value,
) -> Result<Value, String> {
    let static_charge = crate::weapons::good_boy::projection::static_charge(
        tables,
        content_id,
        Some(&actor.weapon_id),
        actor.weapon_tier,
    )?;
    project_charge_efficiency(actor, report, static_charge)
}

pub(super) fn project_charge_efficiency(
    actor: &mut crate::simulation::GameActorConfig,
    report: &Value,
    static_weapon: Option<Value>,
) -> Result<Value, String> {
    let attribute = &report["known_attributes"]["ChargeGetEfficiencyBase"];
    let mut exact = receipt(&attribute["receipt"])?;
    if let Some(weapon) = &static_weapon {
        let bonus: Ratio = serde_json::from_value(weapon["charge_efficiency_bonus"].clone())
            .map_err(|e| e.to_string())?;
        exact = exact.add(ExactFraction::from_ratio(bonus)?)?;
    }
    let multiplier = ratio(exact)?;
    let proc = GearProcConfig::ChargeEfficiency { multiplier };
    crate::gear::validate_charge_multiplier(multiplier)?;
    let previous = actor
        .gear_procs
        .iter()
        .filter(|proc| matches!(proc, GearProcConfig::ChargeEfficiency { .. }))
        .cloned()
        .collect::<Vec<_>>();
    if previous.len() > 1 {
        return Err("duplicate charge efficiency proc in native build projection".into());
    }
    actor
        .gear_procs
        .retain(|proc| !matches!(proc, GearProcConfig::ChargeEfficiency { .. }));
    if multiplier.numerator != multiplier.denominator {
        actor.gear_procs.push(proc);
    }
    let mut result = json!({
        "multiplier": multiplier,
        "native_attribute": attribute,
        "replaced_procs": previous,
    });
    if let Some(weapon) = static_weapon {
        result["static_weapon"] = weapon;
    }
    Ok(result)
}
