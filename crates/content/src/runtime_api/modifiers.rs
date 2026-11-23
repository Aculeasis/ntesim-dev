use crate::simulation::ExactFraction;
use crate::{HypothesisDescription, HypothesisId};

/// Supported additive content sources saturate at the documented 100% bound.
/// Individual exact-formula requests still validate their own input range.
pub(crate) fn aggregate_defense_ignore(
    owner: ExactFraction,
    buffs: ExactFraction,
) -> Result<ExactFraction, String> {
    let total = owner.add(buffs)?;
    if total.numerator >= total.denominator {
        ExactFraction::new(1, 1)
    } else {
        Ok(total)
    }
}

#[derive(Clone, Debug)]
pub struct HitModifiers {
    /// An owner-resolved HP-loss packet; bypass every ordinary damage zone/crit.
    pub(crate) resolved_damage: Option<ExactFraction>,
    /// Owner-captured replacement of the full stat*coefficient base.
    pub(crate) damage_base_override: Option<ExactFraction>,
    pub(crate) damage_bonus: ExactFraction,
    pub(crate) atk_percent: ExactFraction,
    pub(crate) def_percent: ExactFraction,
    pub(crate) ratio_add: ExactFraction,
    pub(crate) ratio_modify: ExactFraction,
    pub(crate) final_bonus: ExactFraction,
    pub(crate) crit_damage_bonus: ExactFraction,
    pub(crate) resistance_reduction: ExactFraction,
    pub(crate) assumptions: std::collections::BTreeMap<String, String>,
}

impl HitModifiers {
    pub(crate) fn new(damage_bonus: ExactFraction) -> Result<Self, String> {
        let zero = ExactFraction::new(0, 1)?;
        Ok(Self {
            resolved_damage: None,
            damage_base_override: None,
            damage_bonus,
            atk_percent: zero,
            def_percent: zero,
            ratio_add: zero,
            ratio_modify: zero,
            final_bonus: zero,
            crit_damage_bonus: zero,
            resistance_reduction: zero,
            assumptions: std::collections::BTreeMap::new(),
        })
    }
    pub(crate) fn record_hypothesis(
        &mut self,
        request: Option<&crate::simulation::GameScenarioRequest>,
        id: HypothesisId,
    ) -> Result<(), String> {
        if let Some(req) = request {
            crate::runtime_api::record_hypothesis_id(req, &mut self.assumptions, id)
        } else {
            self.assumptions
                .insert(id.as_str().to_string(), id.description().to_string());
            Ok(())
        }
    }
    pub(crate) fn add_damage(mut self, value: ExactFraction) -> Result<Self, String> {
        self.damage_bonus = self.damage_bonus.add(value)?;
        Ok(self)
    }
    pub(crate) fn add_atk_percent(mut self, value: ExactFraction) -> Result<Self, String> {
        self.atk_percent = self.atk_percent.add(value)?;
        Ok(self)
    }
    pub(crate) fn add_def_percent(mut self, value: ExactFraction) -> Result<Self, String> {
        self.def_percent = self.def_percent.add(value)?;
        Ok(self)
    }
    pub(crate) fn add_ratio(mut self, value: ExactFraction) -> Result<Self, String> {
        self.ratio_add = self.ratio_add.add(value)?;
        Ok(self)
    }
    pub(crate) fn modify_ratio(mut self, value: ExactFraction) -> Result<Self, String> {
        self.ratio_modify = self.ratio_modify.add(value)?;
        Ok(self)
    }
    pub(crate) fn add_final(mut self, value: ExactFraction) -> Result<Self, String> {
        self.final_bonus = self.final_bonus.add(value)?;
        Ok(self)
    }
    pub(crate) fn add_crit_damage(mut self, value: ExactFraction) -> Result<Self, String> {
        self.crit_damage_bonus = self.crit_damage_bonus.add(value)?;
        Ok(self)
    }
    pub(crate) fn add_resistance_reduction(mut self, value: ExactFraction) -> Result<Self, String> {
        self.resistance_reduction = self.resistance_reduction.add(value)?;
        Ok(self)
    }
}

pub(crate) struct BreakModifiers {
    pub damage_bonus: ExactFraction,
    pub defense_ignore: ExactFraction,
    pub extra_contributions: u32,
}

impl BreakModifiers {
    pub(crate) fn neutral() -> Result<Self, String> {
        Ok(Self {
            damage_bonus: ExactFraction::new(0, 1)?,
            defense_ignore: ExactFraction::new(0, 1)?,
            extra_contributions: 0,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ScorchEnhancement {
    pub(crate) owner_actor_index: usize,
    pub(crate) stack_cap: u32,
    pub(crate) set_max_stacks: bool,
}

pub(crate) struct StainEnhancement {
    pub(crate) duration_us: u64,
    pub(crate) base_bonus: crate::formula::Ratio,
}

pub(crate) struct RemoraEnhancement {
    pub(crate) duration_us: u64,
    pub(crate) crit_rate_bonus: crate::formula::Ratio,
}

pub(crate) struct NovaEndEffect {
    pub(crate) owner_actor_index: usize,
    pub(crate) pending_damage: ExactFraction,
    pub(crate) damage_effect_id: &'static str,
    pub(crate) ability_id: &'static str,
    pub(crate) hypothesis: HypothesisId,
    pub(crate) reapply: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct DirectPacketEnergyPolicy {
    pub(crate) hypothesis_id: HypothesisId,
    pub(crate) event_kind: &'static str,
    pub(crate) trigger: PacketEnergyTrigger,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PacketEnergyTrigger {
    AppliedHpLoss,
    EffectApplication,
}

pub(crate) struct PacketCycleGrant {
    pub raw: ExactFraction,
    pub hypothesis: HypothesisId,
}

pub(crate) const HEALING_MODIFIERS_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::CombatLiveHealingModifiersV1;
#[allow(dead_code)]
pub(crate) const HEALING_MODIFIERS_HYPOTHESIS: HypothesisDescription =
    HypothesisDescription(HEALING_MODIFIERS_HYPOTHESIS_ID);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HealingSettlement {
    pub(crate) gross: u64,
    pub(crate) applied: u64,
    pub(crate) excess: u64,
    pub(crate) before_hp: u64,
    pub(crate) after_hp: u64,
}
