use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicBuffAmountRule {
    pub stat: String,
    pub coefficient: f64,
    pub maximum: Option<f64>,
    pub snapshot: SnapshotPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buff {
    pub id: String,
    pub stat: String,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(default)]
    pub amount_from: Option<DynamicBuffAmountRule>,
    pub stacking: StackingPolicy,
    #[serde(default)]
    pub duration_us: u64,
}

impl Buff {
    pub fn compute_amount(&self, source_stats: &DynamicStats) -> Result<f64, CombatError> {
        if self.amount.is_some() && self.amount_from.is_some() {
            return Err(CombatError::NonFiniteParameter(
                "buff must select one amount source".into(),
            ));
        }
        if let Some(amt) = self.amount {
            if !amt.is_finite() {
                return Err(CombatError::NonFiniteParameter("buff amount".into()));
            }
            return Ok(amt);
        }
        if let Some(rule) = &self.amount_from {
            if !rule.coefficient.is_finite()
                || rule
                    .maximum
                    .is_some_and(|cap| !cap.is_finite() || cap < 0.0)
            {
                return Err(CombatError::NonFiniteParameter(
                    "buff coefficient/cap".into(),
                ));
            }
            let base_val = source_stats
                .get_stat(&rule.stat)
                .ok_or_else(|| CombatError::MissingStat(rule.stat.clone()))?;
            let mut calculated = base_val * rule.coefficient;
            if let Some(cap) = rule.maximum {
                calculated = calculated.min(cap);
            }
            if !calculated.is_finite() {
                return Err(CombatError::NonFiniteParameter(
                    "buff computed amount".into(),
                ));
            }
            return Ok(calculated);
        }
        Err(CombatError::NonFiniteParameter(
            "buff has neither amount nor amount_from".into(),
        ))
    }
}

fn project_stats(
    initial: &DynamicStats,
    active: &[(&Buff, f64)],
) -> Result<DynamicStats, CombatError> {
    let mut projected = initial.clone();
    for (buff, amount) in active {
        projected.apply_modifier(&buff.stat, *amount)?;
    }
    for (stat, aggregate) in [
        ("defense_shred", &mut projected.defense_shred),
        ("defense_ignore", &mut projected.defense_ignore),
    ] {
        if active.iter().any(|(buff, _)| buff.stat == stat) {
            if !aggregate.is_finite() {
                return Err(CombatError::NonFiniteParameter(stat.into()));
            }
            *aggregate = aggregate.clamp(0.0, 1.0);
        }
    }
    Ok(projected)
}

/// Resolve an explicitly active stat projection. Identity is the caller's buff ID;
/// distinct carriers must use distinct IDs. Live dynamic snapshots need a runtime.
pub(crate) fn project_active_buffs(
    initial: &DynamicStats,
    buffs: &[Buff],
) -> Result<(DynamicStats, usize), CombatError> {
    let mut active: Vec<(&Buff, f64)> = Vec::new();
    for buff in buffs {
        if matches!(&buff.amount_from, Some(rule) if rule.snapshot == SnapshotPolicy::Dynamic) {
            return Err(CombatError::NonFiniteParameter("dynamic buff snapshots require a live lifecycle; static projection cannot evaluate them".into()));
        }
        let at_application = project_stats(initial, &active)?;
        let source = match &buff.amount_from {
            Some(rule) if rule.snapshot == SnapshotPolicy::AtApplication => &at_application,
            _ => initial,
        };
        let amount = buff.compute_amount(source)?;
        match buff.stacking {
            StackingPolicy::ReplaceBySource => {
                active.retain(|(existing, _)| existing.id != buff.id || existing.stat != buff.stat)
            }
            StackingPolicy::ReplaceByTarget => {
                active.retain(|(existing, _)| existing.stat != buff.stat)
            }
            StackingPolicy::AddStacks { max_stacks } => {
                if max_stacks == 0 {
                    return Err(CombatError::NonFiniteParameter(
                        "buff max_stacks must be positive".into(),
                    ));
                }
                if active
                    .iter()
                    .filter(|(existing, _)| existing.id == buff.id && existing.stat == buff.stat)
                    .count()
                    >= max_stacks as usize
                {
                    continue;
                }
            }
        }
        active.push((buff, amount));
    }
    let applied = active.len();
    let projected = project_stats(initial, &active)?;
    Ok((projected, applied))
}
