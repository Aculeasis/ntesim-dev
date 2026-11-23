use super::{ExactFraction, GameAdmissionReason};
use crate::formula::Ratio;
use serde::{Deserialize, Serialize};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::CombatStaminaV1;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CombatStaminaProfile {
    pub capacity: Ratio,
    pub initial: Ratio,
    pub use_efficiency: Ratio,
    pub recovery_rate: Ratio,
}

impl CombatStaminaProfile {
    pub fn validate(&self) -> Result<(), String> {
        let capacity = ExactFraction::from_ratio(self.capacity)?;
        let initial = ExactFraction::from_ratio(self.initial)?;
        let efficiency = ExactFraction::from_ratio(self.use_efficiency)?;
        ExactFraction::from_ratio(self.recovery_rate)?;
        if capacity.is_zero() || efficiency.is_zero() || less(capacity, initial)? {
            return Err(
                "combat stamina requires positive capacity/use efficiency and initial <= capacity"
                    .into(),
            );
        }
        // A real admitted dodge must have an exactly representable scaled cost.
        ExactFraction::new(15, 1)?.mul(efficiency)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CombatStaminaSpend {
    pub command_index: usize,
    pub actor_index: usize,
    pub time_us: u64,
    pub amount: ExactFraction,
    pub before: ExactFraction,
    pub after: ExactFraction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CombatStaminaReport {
    pub profile: CombatStaminaProfile,
    pub current: ExactFraction,
    pub total_spent: ExactFraction,
    /// Actual recovered amount, excluding recovery discarded at capacity.
    pub total_recovered: ExactFraction,
    pub recovery_ready_at_us: u64,
    pub updated_at_us: u64,
    pub spends: Vec<CombatStaminaSpend>,
    /// Source-owned recovery gates. Removing one cannot release another owner.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub recovery_suppressions: std::collections::BTreeSet<(usize, usize)>,
}

fn less(a: ExactFraction, b: ExactFraction) -> Result<bool, String> {
    match a.try_sub(b) {
        Ok(_) => Ok(false),
        Err(ntesim_core::combat::exact::ExactArithmeticError::SubtractionUnderflow) => Ok(true),
        Err(error) => Err(error.to_string()),
    }
}

impl CombatStaminaReport {
    pub(crate) fn verify(&self) -> Result<(), String> {
        let initial = ExactFraction::from_ratio(self.profile.initial)?;
        if initial.add(self.total_recovered)? != self.current.add(self.total_spent)?
            || less(
                ExactFraction::from_ratio(self.profile.capacity)?,
                self.current,
            )?
        {
            return Err("combat stamina conservation/capacity invariant failed".into());
        }
        Ok(())
    }

    pub(crate) fn new(profile: &CombatStaminaProfile) -> Self {
        Self {
            profile: profile.clone(),
            current: ExactFraction::from_ratio(profile.initial).expect("validated stamina"),
            total_spent: ExactFraction::new(0, 1).unwrap(),
            total_recovered: ExactFraction::new(0, 1).unwrap(),
            recovery_ready_at_us: 0,
            updated_at_us: 0,
            spends: Vec::new(),
            recovery_suppressions: Default::default(),
        }
    }

    pub(crate) fn advance(&mut self, time_us: u64) -> Result<(), String> {
        if time_us < self.updated_at_us {
            return Err("combat stamina clock reversed".into());
        }
        let elapsed = if self.recovery_suppressions.is_empty() {
            time_us.saturating_sub(self.updated_at_us.max(self.recovery_ready_at_us))
        } else {
            0
        };
        let recovery = ExactFraction::from_ratio(self.profile.recovery_rate)?
            .mul(ExactFraction::new(elapsed.into(), 1_000_000)?)?;
        let capacity = ExactFraction::from_ratio(self.profile.capacity)?;
        let room = capacity.sub(self.current)?;
        let applied = if less(room, recovery)? {
            room
        } else {
            recovery
        };
        let current = self.current.add(applied)?;
        let total = self.total_recovered.add(applied)?;
        self.current = current;
        self.total_recovered = total;
        self.updated_at_us = time_us;
        Ok(())
    }

    pub(crate) fn prepare_dodge(
        &self,
        time_us: u64,
    ) -> Result<Option<GameAdmissionReason>, String> {
        time_us
            .checked_add(2_000_000)
            .ok_or("combat stamina recovery deadline overflow")?;
        let required = self.dodge_cost()?;
        Ok(
            less(self.current, required)?.then_some(GameAdmissionReason::Stamina {
                required,
                available: self.current,
            }),
        )
    }

    fn dodge_cost(&self) -> Result<ExactFraction, String> {
        ExactFraction::new(15, 1)?.mul(ExactFraction::from_ratio(self.profile.use_efficiency)?)
    }

    pub(crate) fn spend_dodge(
        &mut self,
        command_index: usize,
        actor_index: usize,
        time_us: u64,
    ) -> Result<(), String> {
        let amount = self.dodge_cost()?;
        self.spend(command_index, actor_index, time_us, amount)
    }

    pub(crate) fn suppress_recovery(
        &mut self,
        actor: usize,
        command: usize,
        now: u64,
        suppressed: bool,
    ) -> Result<(), String> {
        self.advance(now)?;
        if suppressed {
            self.recovery_suppressions.insert((actor, command));
        } else {
            self.recovery_suppressions.remove(&(actor, command));
        }
        Ok(())
    }

    /// A paired damage opportunity is funded in full or leaves the account intact.
    pub(crate) fn try_spend(
        &mut self,
        command: usize,
        actor: usize,
        now: u64,
        base_cost: u64,
    ) -> Result<bool, String> {
        self.advance(now)?;
        let amount = ExactFraction::new(base_cost.into(), 1)?
            .mul(ExactFraction::from_ratio(self.profile.use_efficiency)?)?;
        if less(self.current, amount)? {
            return Ok(false);
        }
        self.spend(command, actor, now, amount)?;
        Ok(true)
    }

    /// Continuing owner costs can exhaust the remaining account; admission fees
    /// still use their separate full-affordability check.
    pub(crate) fn spend_up_to(
        &mut self,
        command_index: usize,
        actor_index: usize,
        time_us: u64,
        base_cost: u64,
    ) -> Result<(), String> {
        self.advance(time_us)?;
        let scaled = ExactFraction::new(base_cost.into(), 1)?
            .mul(ExactFraction::from_ratio(self.profile.use_efficiency)?)?;
        let amount = if less(self.current, scaled)? {
            self.current
        } else {
            scaled
        };
        self.spend(command_index, actor_index, time_us, amount)
    }

    fn spend(
        &mut self,
        command_index: usize,
        actor_index: usize,
        time_us: u64,
        amount: ExactFraction,
    ) -> Result<(), String> {
        let after = self.current.sub(amount)?;
        let total = self.total_spent.add(amount)?;
        let deadline = time_us
            .checked_add(2_000_000)
            .ok_or("combat stamina recovery deadline overflow")?;
        self.spends.push(CombatStaminaSpend {
            command_index,
            actor_index,
            time_us,
            amount,
            before: self.current,
            after,
        });
        self.current = after;
        self.total_spent = total;
        self.recovery_ready_at_us = deadline;
        Ok(())
    }
}
