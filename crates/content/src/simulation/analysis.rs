//! Exact, causal views of settled native receipts. No combat rules are inferred.
use super::{ExactFraction, GameScenarioReport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageTotals {
    pub hits: usize,
    pub exact: ExactFraction,
    pub applied_hp_loss: u128,
    pub overkill: u128,
    /// Available only under the report's assessed completion/quality policy.
    pub dps: Option<ExactFraction>,
}

impl DamageTotals {
    fn empty() -> Result<Self, String> {
        Ok(Self {
            hits: 0,
            exact: ExactFraction::new(0, 1)?,
            applied_hp_loss: 0,
            overkill: 0,
            dps: None,
        })
    }

    fn add(&mut self, hit: &super::GameHitReceipt) -> Result<(), String> {
        let overkill = hit
            .rounded_damage
            .checked_sub(hit.applied_hp_loss)
            .ok_or("applied HP loss exceeds rounded damage")?;
        self.hits = self
            .hits
            .checked_add(1)
            .ok_or("damage hit count overflow")?;
        self.exact = ExactFraction::new(
            self.exact
                .numerator
                .checked_add(u128::from(hit.rounded_damage))
                .ok_or("rounded damage overflow")?,
            1,
        )?;
        self.applied_hp_loss = self
            .applied_hp_loss
            .checked_add(hit.applied_hp_loss.into())
            .ok_or("HP loss overflow")?;
        self.overkill = self
            .overkill
            .checked_add(overkill.into())
            .ok_or("overkill overflow")?;
        Ok(())
    }

    fn assess_dps(&mut self, report: &GameScenarioReport) -> Result<(), String> {
        if report.game_dps_computed
            && report.success
            && report.execution_completed
            && report.invariants_passed
            && report.relative_elapsed_us > 0
        {
            self.dps = Some(self.exact.mul(ExactFraction::new(
                1_000_000,
                report.relative_elapsed_us.into(),
            )?)?);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageContribution {
    pub actor_index: usize,
    pub target_index: usize,
    pub ability_id: String,
    pub damage_effect_id: String,
    pub on_field: bool,
    pub totals: DamageTotals,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageAnalysis {
    pub total: DamageTotals,
    /// Stable actor/target/ability/effect/field ordering, independent of hash maps.
    pub contributions: Vec<DamageContribution>,
}

impl GameScenarioReport {
    pub fn damage_analysis(&self) -> Result<DamageAnalysis, String> {
        self.validate_clocks()?;
        self.validate_damage_parentage()?;
        let mut total = DamageTotals::empty()?;
        let mut groups: BTreeMap<(usize, usize, &str, &str, bool), DamageTotals> = BTreeMap::new();
        let mut hp = self
            .state_timeline
            .first()
            .ok_or("missing initial state frame")?
            .targets
            .iter()
            .map(|t| t.hp_current)
            .collect::<Vec<_>>();
        for hit in &self.hit_receipts {
            if hit.damage.round_u64()? != hit.rounded_damage {
                return Err("damage receipt rounding differs from exact damage".into());
            }
            if hit.actor_index >= self.final_actor_states.len()
                || hit.active_actor_index >= self.final_actor_states.len()
            {
                return Err("damage receipt actor identity is invalid".into());
            }
            let remaining = hp
                .get_mut(hit.target_index)
                .ok_or("damage target identity is invalid")?;
            if hit.applied_hp_loss != (*remaining).min(hit.rounded_damage) {
                return Err("damage receipts do not reconcile applied target HP loss".into());
            }
            *remaining -= hit.applied_hp_loss;
            total.add(hit)?;
            let key = (
                hit.actor_index,
                hit.target_index,
                hit.ability_id.as_str(),
                hit.damage_effect_id.as_str(),
                hit.actor_index == hit.active_actor_index,
            );
            groups
                .entry(key)
                .or_insert(DamageTotals::empty()?)
                .add(hit)?;
        }
        if total.exact != self.total_damage_exact
            || total.hits != self.hits_settled
            || hp
                .iter()
                .zip(&self.final_target_states)
                .any(|(hp, t)| *hp != t.hp_current)
        {
            return Err("damage contributions differ from native totals/targets".into());
        }
        total.assess_dps(self)?;
        let contributions = groups
            .into_iter()
            .map(
                |(
                    (actor_index, target_index, ability_id, damage_effect_id, on_field),
                    mut totals,
                )| {
                    totals.assess_dps(self)?;
                    Ok(DamageContribution {
                        actor_index,
                        target_index,
                        ability_id: ability_id.to_owned(),
                        damage_effect_id: damage_effect_id.to_owned(),
                        on_field,
                        totals,
                    })
                },
            )
            .collect::<Result<Vec<_>, String>>()?;
        Ok(DamageAnalysis {
            total,
            contributions,
        })
    }
}
