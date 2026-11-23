//! Exact per-owner Cycle accounting, separate from Ultimate distribution.
use super::{ExactFraction, GameCycleLedgerReport, GameEventPayload, GameEventReceipt};

#[derive(Clone, Debug)]
pub(crate) struct Ledger {
    active: bool,
    initial: u64,
    granted: u64,
    spent: u64,
    overflow: u64,
    carry: ExactFraction,
    groups: std::collections::BTreeSet<String>,
}

impl Ledger {
    /// Owner activation fills the integer pool without multiplying by gain
    /// efficiency, discarding exact carry, or rewriting its accounting origin.
    pub(crate) fn fill(
        &mut self,
        pool: &mut ntesim_core::kernel::state::Pool,
    ) -> Result<(), String> {
        let missing = pool.capacity() - pool.current();
        let granted = self
            .granted
            .checked_add(missing)
            .ok_or("Cycle fill accounting overflow")?;
        pool.grant(missing);
        self.granted = granted;
        self.active = true;
        Ok(())
    }

    pub(crate) fn new(initial: u64) -> Self {
        Self {
            active: false,
            initial,
            granted: 0,
            spent: 0,
            overflow: 0,
            carry: ExactFraction {
                numerator: 0,
                denominator: 1,
            },
            groups: Default::default(),
        }
    }

    pub(crate) fn spend(&mut self, amount: u64) -> Result<(), String> {
        self.spent = self
            .spent
            .checked_add(amount)
            .ok_or("Cycle spend accounting overflow")?;
        // An authored full pool can be spent before this owner's accrual is bound.
        // Its real spend must still appear in the conserved native ledger.
        self.active |= amount > 0;
        Ok(())
    }

    pub(crate) fn grant_direct(
        &mut self,
        pool: &mut ntesim_core::kernel::state::Pool,
        amount: u64,
    ) -> Result<(), String> {
        self.granted = self
            .granted
            .checked_add(amount)
            .ok_or("Cycle grant accounting overflow")?;
        let excess = pool.grant(amount).excess;
        self.overflow = self
            .overflow
            .checked_add(excess)
            .ok_or("Cycle overflow accounting overflow")?;
        self.active = true;
        Ok(())
    }

    pub(crate) fn summary(
        &self,
        pool: &ntesim_core::kernel::state::Pool,
    ) -> Option<GameCycleLedgerReport> {
        self.active.then(|| GameCycleLedgerReport {
            initial: self.initial,
            current: pool.current(),
            capacity: pool.capacity(),
            granted: self.granted,
            spent: self.spent,
            overflow: self.overflow,
            carry: self.carry,
            conserved: u128::from(self.initial) + u128::from(self.granted)
                == u128::from(pool.current()) + u128::from(self.spent) + u128::from(self.overflow),
        })
    }
}

pub(super) fn on_packet(
    ctx: &mut super::context::EventContext<'_>,
    command_index: usize,
    actor_index: usize,
    target_index: usize,
    hp_loss: u64,
    metadata: &super::metadata::ActionPacketMetadata,
) -> Result<(), String> {
    if hp_loss == 0
        || ctx.state.actors[actor_index].hp == 0
        || !matches!(
            metadata.binding.kind,
            super::metadata::ActionPacketBindingKind::NativeDirect
                | super::metadata::ActionPacketBindingKind::OwnerChild
        )
    {
        return Ok(());
    }
    let group = metadata.binding.energy_group_id.as_deref();
    if (target_index != ctx.request.aim_target(command_index) && group.is_none())
        || group.is_some_and(|id| {
            ctx.state.actors[actor_index]
                .cycle_ledger
                .groups
                .contains(id)
        })
    {
        return Ok(());
    }
    let config = &ctx.request.actors[actor_index];
    let Some(grant) = crate::registry::packet_cycle_grant(
        ctx.request,
        config,
        &metadata.packet.damage_effect_id,
    )?
    else {
        return Ok(());
    };
    if grant.raw.is_zero() {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        grant.hypothesis,
    )?;
    let actor = &mut ctx.state.actors[actor_index];
    let ledger = &mut actor.cycle_ledger;
    let gross = grant
        .raw
        .mul(ExactFraction::from_ratio(config.cycle_gain_efficiency)?)?;
    let accrued = gross.add(ledger.carry)?;
    let whole = u64::try_from(accrued.numerator / accrued.denominator)
        .map_err(|_| "Cycle grant exceeds u64")?;
    let granted = ledger
        .granted
        .checked_add(whole)
        .ok_or("Cycle grant accounting overflow")?;
    let mut pool = actor.cycle.clone();
    let overflow = ledger
        .overflow
        .checked_add(pool.grant(whole).excess)
        .ok_or("Cycle overflow accounting overflow")?;
    ledger.carry =
        ExactFraction::new(accrued.numerator % accrued.denominator, accrued.denominator)?;
    ledger.granted = granted;
    ledger.overflow = overflow;
    ledger.active = true;
    actor.cycle = pool;
    if let Some(id) = group {
        ledger.groups.insert(id.to_owned());
    }
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "HitDelivery".into(),
        event_kind: "packet_cycle_grant".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            damage_effect_id: Some(metadata.packet.damage_effect_id.clone()),
            exact_raw: Some(grant.raw),
            exact_scaled: Some(gross),
            gross: Some(whole),
            pool: Some(actor.cycle.current()),
            overflow: Some(overflow),
            exact_carry: Some(ledger.carry),
            ..Default::default()
        }),
    });
    Ok(())
}
