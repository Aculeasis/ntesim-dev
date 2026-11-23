//! Typed stat provenance from the same native projection that builds the actor.
use super::{ExactFraction, GameActorStaticStats, Ratio, receipt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatSource {
    Character,
    Weapon,
    Set,
    Authored,
    CharacterPassive,
    WeaponPassive,
    BaseOverride,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatContribution {
    pub source: StatSource,
    pub attribute_id: String,
    pub amount: ExactFraction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildDecomposition {
    /// Add in order; BaseOverride replaces that attribute's preceding sum.
    pub contributions: Vec<StatContribution>,
    pub attributes: BTreeMap<String, ExactFraction>,
    /// Exact pre-quantization totals. Final HP/DEF integers remain in static_stats.
    pub atk: ExactFraction,
    pub hp: ExactFraction,
    pub defense: ExactFraction,
}

fn attributes(value: &Value, consumed: &[&str]) -> Result<BTreeMap<String, ExactFraction>, String> {
    value
        .as_object()
        .ok_or("missing stat decomposition attributes")?
        .iter()
        .filter(|(id, _)| consumed.contains(&id.as_str()))
        .map(|(id, value)| Ok((id.clone(), receipt(value)?)))
        .collect()
}

pub(super) fn project(
    native: &Value,
    authored: &BTreeMap<String, Ratio>,
    character_hp_bonus: Ratio,
    consumed: &[&str],
) -> Result<BuildDecomposition, String> {
    let zero = ExactFraction::new(0, 1)?;
    let sources = &native["stat_sources"];
    let mut contributions = Vec::new();
    for (key, source) in [
        ("character", StatSource::Character),
        ("weapon", StatSource::Weapon),
        ("set", StatSource::Set),
        ("base_override", StatSource::BaseOverride),
    ] {
        if let Some(values) = sources.get(key) {
            for (attribute_id, amount) in attributes(values, consumed)? {
                contributions.push(StatContribution {
                    source,
                    attribute_id,
                    amount,
                });
            }
        }
    }
    let mut additives = attributes(&sources["additive"], consumed)?;
    for (name, value) in authored {
        let attribute_id = super::studio_projection::additive_property(name)?;
        let amount = ExactFraction::from_ratio(*value)?;
        let remaining = additives
            .get(attribute_id)
            .copied()
            .unwrap_or(zero)
            .sub(amount)?;
        additives.insert(attribute_id.into(), remaining);
        contributions.push(StatContribution {
            source: StatSource::Authored,
            attribute_id: attribute_id.into(),
            amount,
        });
    }
    let hp_bonus = ExactFraction::from_ratio(character_hp_bonus)?;
    if !hp_bonus.is_zero() {
        let remaining = additives
            .get("HPMaxUp")
            .copied()
            .unwrap_or(zero)
            .sub(hp_bonus)?;
        additives.insert("HPMaxUp".into(), remaining);
        contributions.push(StatContribution {
            source: StatSource::CharacterPassive,
            attribute_id: "HPMaxUp".into(),
            amount: hp_bonus,
        });
    }
    for (attribute_id, amount) in additives {
        if !amount.is_zero() {
            contributions.push(StatContribution {
                source: StatSource::WeaponPassive,
                attribute_id,
                amount,
            });
        }
    }
    let report = BuildDecomposition {
        contributions,
        attributes: attributes(&native["attributes"], consumed)?,
        atk: receipt(&native["derived"]["atk_receipt"])?,
        hp: receipt(&native["derived"]["hp_receipt"])?,
        defense: receipt(&native["derived"]["def_receipt"])?,
    };
    report.validate()?;
    Ok(report)
}

impl BuildDecomposition {
    pub fn validate(&self) -> Result<(), String> {
        let zero = ExactFraction::new(0, 1)?;
        let mut sums = BTreeMap::new();
        for c in &self.contributions {
            let before = sums.get(&c.attribute_id).copied().unwrap_or(zero);
            sums.insert(
                c.attribute_id.clone(),
                if c.source == StatSource::BaseOverride {
                    c.amount
                } else {
                    before.add(c.amount)?
                },
            );
        }
        for (id, expected) in &self.attributes {
            if sums.remove(id).unwrap_or(zero) != *expected {
                return Err(format!("stat decomposition does not reconcile {id}"));
            }
        }
        if sums.values().any(|v| !v.is_zero()) {
            return Err("stat decomposition has an unconsumed contribution".into());
        }
        let value = |id: &str| self.attributes.get(id).copied().unwrap_or(zero);
        for (base, percent, flat, expected) in [
            ("AtkBase", "AtkUp", "AtkAdd", self.atk),
            ("HPMaxBase", "HPMaxUp", "HPMaxAdd", self.hp),
            ("DefBase", "DefUp", "DefAdd", self.defense),
        ] {
            let actual = value(base)
                .mul(ExactFraction::new(1, 1)?.add(value(percent))?)?
                .add(value(flat))?;
            if actual != expected {
                return Err(format!("stat decomposition derived {base} differs"));
            }
        }
        Ok(())
    }

    pub fn validate_static_stats(&self, stats: &GameActorStaticStats) -> Result<(), String> {
        self.validate()?;
        if self.atk != ExactFraction::from_ratio(stats.atk)?
            || self.hp.round_u64()? != stats.hp
            || self.defense.round_u64()? != stats.def
        {
            return Err("stat decomposition differs from compiled static stats".into());
        }
        let zero = ExactFraction::new(0, 1)?;
        for (id, value) in [
            ("HPMaxUp", stats.hp_percent),
            ("HPMaxAdd", stats.hp_flat),
            ("DefUp", stats.def_percent),
            ("DefAdd", stats.def_flat),
            ("CritBase", stats.crit_rate),
            ("CritDamageBase", stats.crit_damage_bonus),
            ("HealUp", stats.healing_bonus),
            ("ShieldEfficiency", stats.shield_bonus),
            ("UnbalIntensityBase", stats.break_intensity),
            ("MagBase", stats.cycle_intensity),
            (
                "UnbalAccrueEfficiencyBase",
                stats.break_efficiency_multiplier,
            ),
            ("DamageUpPsychicallyBase", stats.mental_damage_bonus),
            ("DamageUpGeneralBase", stats.damage_bonus),
        ] {
            if self.attributes.get(id).copied().unwrap_or(zero) != ExactFraction::from_ratio(value)?
            {
                return Err(format!("stat decomposition differs from compiled {id}"));
            }
        }
        for (element, id) in super::stats::ELEMENT_ATTRIBUTES {
            let value = stats
                .elemental_bonuses
                .get(element)
                .copied()
                .unwrap_or(Ratio::zero());
            if self.attributes.get(id).copied().unwrap_or(zero) != ExactFraction::from_ratio(value)?
            {
                return Err(format!(
                    "stat decomposition differs from compiled {element}"
                ));
            }
        }
        Ok(())
    }
}
