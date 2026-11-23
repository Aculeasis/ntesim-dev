use crate::formula::Ratio;
use crate::simulation::ExactFraction;
#[derive(Clone, Debug)]
pub(crate) struct ActorRuntime {
    pub(crate) combat_hp_losses: Option<Vec<crate::simulation::vitals::CombatHpLoss>>,
    pub(crate) aiming: bool,
    pub(crate) hp: u64,
    pub(crate) hp_max: u64,
    pub(crate) hp_max_base: ExactFraction,
    pub(crate) hp_max_static_bonus: crate::simulation::ExactFraction,
    pub(crate) shields: Vec<ActorShieldRuntime>,
    pub(crate) energy: ntesim_core::kernel::state::Pool,
    pub(crate) cycle: ntesim_core::kernel::state::Pool,
    pub(crate) cycle_ledger: crate::simulation::cycle::Ledger,
    pub(crate) locked_until_us: u64,
    pub(crate) action_clock: Option<ntesim_core::kernel::timing::ClockDomain>,
    pub(crate) world_time_us: u64,
    /// Shared switch cooldown, measured on paused World time.
    pub(crate) switch_cooldown_until_us: u64,
    /// Finite owner action switch gate, measured on the current action clock.
    pub(crate) switch_action_locked_until_us: u64,
    pub(crate) ability_cooldowns: ntesim_core::kernel::timing::ProjectedDeadlines<String>,
    pub(crate) last_action_start_us: u64,
    pub(crate) field_time_absolute_us: u64,
    pub(crate) field_time_relative_us: u64,
    pub(crate) character: crate::characters::RuntimeState,
    pub(crate) equipment: crate::gear::RuntimeState,
    pub(crate) initial_energy: u64,
    pub(crate) granted_energy: u64,
    pub(crate) spent_energy: u64,
    pub(crate) overflow_energy: u64,
    pub(crate) charge_multiplier: Ratio,
    pub(crate) charge_remainder: ExactFraction,
    pub(crate) packet_energy_groups: std::collections::BTreeSet<String>,
    pub(crate) single_support_contact: Option<usize>,
    pub(crate) single_support_start_us: Option<u64>,
    pub(crate) packet_group_ordinal: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ActorShieldRuntime {
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) remaining_hp: u64,
    pub(crate) max_hp: u64,
    pub(crate) expires_at_us: u64,
}

impl ActorRuntime {
    /// Combat-entry owner overrides establish the actual ledger origin before
    /// any grants/spends; this is not a continuing-combat resource reset.
    pub(crate) fn initialize_cycle(&mut self, value: u64) -> Result<(), String> {
        self.cycle = ntesim_core::kernel::state::Pool::new(value, self.cycle.capacity())
            .map_err(|error| format!("Cycle initialization failed: {error:?}"))?;
        self.cycle_ledger = crate::simulation::cycle::Ledger::new(value);
        Ok(())
    }

    /// Called immediately after an actual combat damage/paid-cost settlement.
    /// Keep each loss-time MaxHP even when a later heal or stat change shares its event.
    pub(crate) fn record_hp_loss(&mut self, amount: u64, max_hp: u64) {
        if amount > 0
            && let Some(losses) = &mut self.combat_hp_losses
        {
            losses.push(crate::simulation::vitals::CombatHpLoss {
                hp_before: self
                    .hp
                    .checked_add(amount)
                    .expect("settled HP loss must preserve its pre-loss HP"),
                hp_after: self.hp,
                max_hp,
            });
        }
    }
    pub(crate) fn restore_vitals(
        &self,
        max_hp: u64,
        now_us: u64,
        excluded_shield: Option<&str>,
    ) -> Result<ntesim_core::state::VitalState, String> {
        let mut vitals = ntesim_core::state::VitalState::new(
            self.hp,
            max_hp,
            ntesim_core::state::ShieldOrder::OldestFirst,
        )
        .map_err(|error| format!("actor vitals failed: {error:?}"))?;
        for shield in self.shields.iter().filter(|shield| {
            shield.expires_at_us > now_us && Some(shield.id.as_str()) != excluded_shield
        }) {
            vitals
                .add_shield(
                    shield.id.clone(),
                    shield.source.clone(),
                    shield.remaining_hp,
                )
                .map_err(|error| format!("actor shield restore failed: {error:?}"))?;
        }
        Ok(vitals)
    }

    pub(crate) fn save_vitals(&mut self, vitals: &ntesim_core::state::VitalState, now_us: u64) {
        self.hp = vitals.hp();
        for shield in &mut self.shields {
            shield.remaining_hp = vitals
                .shields()
                .iter()
                .find(|active| active.id == shield.id)
                .map_or(0, ntesim_core::state::Shield::remaining);
        }
        self.shields
            .retain(|shield| shield.remaining_hp > 0 && shield.expires_at_us > now_us);
    }

    pub(crate) fn expire_shields(&mut self, now_us: u64) -> Vec<(String, u64)> {
        let expired = self
            .shields
            .iter()
            .filter(|shield| shield.expires_at_us <= now_us)
            .map(|shield| (shield.id.clone(), shield.expires_at_us))
            .collect();
        self.shields.retain(|shield| shield.expires_at_us > now_us);
        if self.hp == 0 {
            self.shields.clear();
        }
        expired
    }
}

pub(crate) fn replace_actor_shield(
    actor: &mut ActorRuntime,
    max_hp: u64,
    now_us: u64,
    shield_id: &str,
    source: &str,
    amount: u64,
    duration_us: u64,
) -> Result<u64, String> {
    let expires_at_us = now_us
        .checked_add(duration_us)
        .ok_or("actor shield expiry overflow")?;
    if actor.hp == 0 {
        return Ok(expires_at_us);
    }
    let old_shields = actor.shields.clone();
    let mut vitals = actor
        .restore_vitals(max_hp, now_us, Some(shield_id))
        .map_err(|error| format!("actor shield vitals failed: {error}"))?;
    if amount > 0 {
        vitals
            .add_shield(shield_id.into(), source.into(), amount)
            .map_err(|error| format!("actor shield grant failed: {error:?}"))?;
    }
    actor.shields = vitals
        .shields()
        .iter()
        .map(|shield| {
            let is_new = shield.id == shield_id;
            let old = old_shields.iter().find(|old| old.id == shield.id);
            ActorShieldRuntime {
                id: shield.id.clone(),
                source: shield.source.clone(),
                remaining_hp: shield.remaining(),
                max_hp: if is_new {
                    amount
                } else {
                    old.map_or(shield.remaining(), |old| old.max_hp)
                },
                expires_at_us: if is_new {
                    expires_at_us
                } else {
                    old.map_or(expires_at_us, |old| old.expires_at_us)
                },
            }
        })
        .collect();
    Ok(expires_at_us)
}

impl ActorRuntime {
    pub(super) fn energy_conserved(&self) -> bool {
        let expected = (self.initial_energy as u128) + (self.granted_energy as u128);
        let actual = (self.energy.current() as u128)
            + (self.spent_energy as u128)
            + (self.overflow_energy as u128);
        expected == actual
    }

    /// Scaled generation retains sub-unit carry; explicit pool fills bypass this.
    pub(crate) fn grant_energy(&mut self, amount: u64) -> Result<u64, String> {
        self.grant_fractional_energy(ExactFraction::new(amount.into(), 1)?)
    }

    pub(crate) fn grant_fractional_energy(&mut self, amount: ExactFraction) -> Result<u64, String> {
        let generated = amount
            .mul(ExactFraction::from_ratio(self.charge_multiplier)?)?
            .add(self.charge_remainder)?;
        let whole = u64::try_from(generated.numerator / generated.denominator)
            .map_err(|_| "scaled energy exceeds u64")?;
        let remainder = ExactFraction::new(
            generated.numerator % generated.denominator,
            generated.denominator,
        )?;
        self.restore_energy(whole)?;
        self.charge_remainder = remainder;
        Ok(whole)
    }

    pub(crate) fn restore_energy(&mut self, amount: u64) -> Result<(), String> {
        let next_granted = self
            .granted_energy
            .checked_add(amount)
            .ok_or("energy grant accounting overflow")?;
        let mut next_energy = self.energy.clone();
        let grant = next_energy.grant(amount);
        let next_overflow = self
            .overflow_energy
            .checked_add(grant.excess)
            .ok_or("energy overflow accounting overflow")?;
        self.energy = next_energy;
        self.granted_energy = next_granted;
        self.overflow_energy = next_overflow;
        Ok(())
    }

    pub(crate) fn spend_energy(&mut self, amount: u64) -> Result<(), String> {
        let next_spent = self
            .spent_energy
            .checked_add(amount)
            .ok_or("energy spend accounting overflow")?;
        self.energy
            .spend(amount)
            .map_err(|error| format!("energy spend failed: {error:?}"))?;
        self.spent_energy = next_spent;
        Ok(())
    }
}

/// Effective actor Max HP per EQUIPMENT_CONTRACT.md: set packs use the additive
/// pack interpreter, so static HPMaxUp (e.g. Suit10 2pc) and dynamic stack
/// bonuses (e.g. Suit10 4pc) sum in one pool: base * (1 + static + dynamic),
/// rounded once to nearest integer.
pub(crate) fn effective_hp_max(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    base_hp_max: ExactFraction,
    static_bonus: &ExactFraction,
    actor_index: usize,
    current_hp_max: u64,
) -> Result<u64, String> {
    let mut bonus = *static_bonus;
    let mut has_dynamic = false;
    for buff in buffs.instances() {
        if buff.id == crate::sets::hp_suit::BUFF_ID
            && buff.owner_actor == actor_index
            && let Some(ratio) = buff
                .stat_modifiers
                .get(crate::sets::hp_suit::MAX_HP_BONUS_KEY)
        {
            has_dynamic = true;
            let per_stack = ExactFraction::from_ratio(*ratio)?;
            let stacks = ExactFraction::new(buff.stacks.into(), 1)?;
            bonus = bonus.add(per_stack.mul(stacks)?)?;
        }
    }
    // Without dynamic stacks, return the live hp_max as-is so manual
    // overrides (e.g. test fixtures) are respected.
    if !has_dynamic {
        return Ok(current_hp_max);
    }
    base_hp_max
        .mul(ExactFraction::new(1, 1)?.add(bonus)?)?
        .round_u64()
}

pub(crate) fn initial_hp_projection(
    config: &super::super::GameActorConfig,
) -> Result<(ExactFraction, ExactFraction), String> {
    let zero = ExactFraction::new(0, 1)?;
    let Some(base) = config.static_stats.base_hp.filter(|base| *base > 0) else {
        return Ok((ExactFraction::new(config.initial_hp_max.into(), 1)?, zero));
    };
    let base = ExactFraction::new(base.into(), 1)?;
    let declared = ExactFraction::from_ratio(config.static_stats.hp_percent)?
        .add(ExactFraction::from_ratio(config.static_stats.hp_flat)?.div(base)?)?;
    if config.static_stats.hp == config.initial_hp_max
        && base
            .mul(ExactFraction::new(1, 1)?.add(declared)?)?
            .round_u64()?
            == config.initial_hp_max
    {
        // Preserve unrounded static percentage/flat composition for later live HP changes.
        return Ok((base, declared));
    }
    // Historical manually overridden totals remain explicit authoritative initial values.
    let baseline = ExactFraction::new(config.initial_hp_max.into(), 1)?;
    let collapsed = if baseline.try_cmp(base).map_err(String::from)?.is_lt() {
        zero
    } else {
        baseline.sub(base)?.div(base)?
    };
    Ok((base, collapsed))
}
