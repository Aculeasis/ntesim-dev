use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicStats {
    pub base_atk: f64,
    pub base_hp: f64,
    pub base_def: f64,
    pub atk_percent: f64,
    pub hp_percent: f64,
    pub def_percent: f64,
    pub atk_flat: f64,
    pub hp_flat: f64,
    pub def_flat: f64,
    pub crit_rate: f64,
    pub crit_damage: f64,
    pub damage_bonus: f64,
    pub elemental_bonuses: BTreeMap<String, f64>,
    pub action_type_bonuses: BTreeMap<String, f64>,
    pub defense_shred: f64,
    pub defense_ignore: f64,
    pub vulnerability: f64,
    pub healing_bonus: f64,
    pub incoming_healing_bonus: f64,
    pub shield_bonus: f64,
    pub break_efficiency: f64,
    pub energy_efficiency: f64,
    pub cycle_intensity: f64,
}

impl DynamicStats {
    pub fn new(base_atk: f64, base_hp: f64, base_def: f64) -> Self {
        Self {
            base_atk,
            base_hp,
            base_def,
            atk_percent: 0.0,
            hp_percent: 0.0,
            def_percent: 0.0,
            atk_flat: 0.0,
            hp_flat: 0.0,
            def_flat: 0.0,
            crit_rate: 0.05,
            crit_damage: 0.50,
            damage_bonus: 0.0,
            elemental_bonuses: BTreeMap::new(),
            action_type_bonuses: BTreeMap::new(),
            defense_shred: 0.0,
            defense_ignore: 0.0,
            vulnerability: 0.0,
            healing_bonus: 0.0,
            incoming_healing_bonus: 0.0,
            shield_bonus: 0.0,
            break_efficiency: 0.0,
            energy_efficiency: 0.0,
            cycle_intensity: 0.0,
        }
    }

    pub fn total_atk(&self) -> f64 {
        self.base_atk * (1.0 + self.atk_percent) + self.atk_flat
    }

    pub fn total_hp(&self) -> f64 {
        self.base_hp * (1.0 + self.hp_percent) + self.hp_flat
    }

    pub fn total_def(&self) -> f64 {
        self.base_def * (1.0 + self.def_percent) + self.def_flat
    }

    pub fn get_stat(&self, name: &str) -> Option<f64> {
        match name {
            "atk" => Some(self.total_atk()),
            "hp" => Some(self.total_hp()),
            "def" => Some(self.total_def()),
            "base_atk" | "atk_base" => Some(self.base_atk),
            "base_hp" | "hp_base" => Some(self.base_hp),
            "base_def" | "def_base" => Some(self.base_def),
            "atk_percent" => Some(self.atk_percent),
            "hp_percent" => Some(self.hp_percent),
            "def_percent" => Some(self.def_percent),
            "atk_flat" => Some(self.atk_flat),
            "hp_flat" => Some(self.hp_flat),
            "def_flat" => Some(self.def_flat),
            "crit_rate" => Some(self.crit_rate),
            "crit_damage" => Some(self.crit_damage),
            "damage_bonus" => Some(self.damage_bonus),
            "defense_shred" => Some(self.defense_shred),
            "defense_ignore" => Some(self.defense_ignore),
            "vulnerability" => Some(self.vulnerability),
            "healing_bonus" => Some(self.healing_bonus),
            "incoming_healing_bonus" => Some(self.incoming_healing_bonus),
            "shield_bonus" => Some(self.shield_bonus),
            "break_efficiency" => Some(self.break_efficiency),
            "energy_efficiency" | "charge_efficiency" => Some(self.energy_efficiency),
            "cycle_intensity" => Some(self.cycle_intensity),
            other => self
                .elemental_bonuses
                .get(other)
                .copied()
                .or_else(|| self.action_type_bonuses.get(other).copied()),
        }
    }

    pub fn apply_modifier(&mut self, stat: &str, amount: f64) -> Result<(), CombatError> {
        if !amount.is_finite() {
            return Err(CombatError::NonFiniteParameter(stat.into()));
        }
        match stat {
            "atk_percent" => add_checked(&mut self.atk_percent, amount, stat)?,
            "hp_percent" => add_checked(&mut self.hp_percent, amount, stat)?,
            "def_percent" => add_checked(&mut self.def_percent, amount, stat)?,
            "atk_flat" => add_checked(&mut self.atk_flat, amount, stat)?,
            "hp_flat" => add_checked(&mut self.hp_flat, amount, stat)?,
            "def_flat" => add_checked(&mut self.def_flat, amount, stat)?,
            "crit_rate" => add_checked(&mut self.crit_rate, amount, stat)?,
            "crit_damage" => add_checked(&mut self.crit_damage, amount, stat)?,
            "damage_bonus" => add_checked(&mut self.damage_bonus, amount, stat)?,
            "defense_shred" => add_checked(&mut self.defense_shred, amount, stat)?,
            "defense_ignore" => add_checked(&mut self.defense_ignore, amount, stat)?,
            "vulnerability" => add_checked(&mut self.vulnerability, amount, stat)?,
            "healing_bonus" => add_checked(&mut self.healing_bonus, amount, stat)?,
            "incoming_healing_bonus" => {
                add_checked(&mut self.incoming_healing_bonus, amount, stat)?
            }
            "shield_bonus" => add_checked(&mut self.shield_bonus, amount, stat)?,
            "break_efficiency" => add_checked(&mut self.break_efficiency, amount, stat)?,
            "energy_efficiency" | "charge_efficiency" => {
                add_checked(&mut self.energy_efficiency, amount, stat)?
            }
            "cycle_intensity" => add_checked(&mut self.cycle_intensity, amount, stat)?,
            other if other.starts_with("elemental:") => {
                let el = &other["elemental:".len()..];
                add_checked_map(&mut self.elemental_bonuses, el, amount, stat)?;
            }
            other if other.starts_with("action:") => {
                let act = &other["action:".len()..];
                add_checked_map(&mut self.action_type_bonuses, act, amount, stat)?;
            }
            other => {
                if [
                    "Anima",
                    "Incantation",
                    "Lakshana",
                    "Psyche",
                    "Cosmos",
                    "Chaos",
                    "Physical",
                ]
                .contains(&other)
                {
                    add_checked_map(&mut self.elemental_bonuses, other, amount, stat)?;
                } else if ["normal", "skill", "ultimate", "dot", "counter", "switch"]
                    .contains(&other)
                {
                    add_checked_map(&mut self.action_type_bonuses, other, amount, stat)?;
                } else {
                    return Err(CombatError::UnknownModifier(other.into()));
                }
            }
        }
        Ok(())
    }
}

fn add_checked(value: &mut f64, amount: f64, stat: &str) -> Result<(), CombatError> {
    let updated = *value + amount;
    if !updated.is_finite() {
        return Err(CombatError::NonFiniteParameter(stat.into()));
    }
    *value = updated;
    Ok(())
}

fn add_checked_map(
    values: &mut BTreeMap<String, f64>,
    key: &str,
    amount: f64,
    stat: &str,
) -> Result<(), CombatError> {
    let updated = values.get(key).copied().unwrap_or_default() + amount;
    if !updated.is_finite() {
        return Err(CombatError::NonFiniteParameter(stat.into()));
    }
    values.insert(key.into(), updated);
    Ok(())
}
