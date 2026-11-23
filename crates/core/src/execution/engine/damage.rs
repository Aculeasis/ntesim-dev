use super::*;

pub(crate) fn compute_damage_mitigation(
    base_damage: u64,
    kind: DamageKind,
    target_defense: u64,
    attacker_level: u32,
) -> Result<u64, ExecutionError> {
    crate::combat::mitigate_integer_damage(base_damage, kind, target_defense, attacker_level)
        .map_err(|details| ExecutionError::InvariantViolation {
            invariant_name: "damage_mitigation".into(),
            details,
        })
}

pub(crate) fn settle_target_damage(
    target: &mut TargetState,
    effective_damage: u64,
) -> Result<(u64, u64, u64, bool), ExecutionError> {
    let error = |error: crate::state::StateError| ExecutionError::InvariantViolation {
        invariant_name: "vital_settlement".into(),
        details: format!("{error:?}"),
    };
    let mut vitals = crate::state::VitalState::new(
        target.current_hp,
        target.max_hp,
        crate::state::ShieldOrder::OldestFirst,
    )
    .map_err(error)?;
    for shield in &target.active_shields {
        vitals
            .add_shield(
                shield.id.clone(),
                shield.source.clone(),
                shield.remaining_hp,
            )
            .map_err(error)?;
    }
    let settlement = vitals.damage(effective_damage).map_err(error)?;
    let absorbed: u64 = settlement
        .absorbed
        .iter()
        .map(|absorption| absorption.amount)
        .sum();
    target.total_damage_taken = target
        .total_damage_taken
        .checked_add(effective_damage)
        .ok_or_else(|| ExecutionError::InvariantViolation {
            invariant_name: "damage_counter".into(),
            details: "damage counter overflow".into(),
        })?;
    target.total_absorbed = target.total_absorbed.checked_add(absorbed).ok_or_else(|| {
        ExecutionError::InvariantViolation {
            invariant_name: "damage_counter".into(),
            details: "absorption counter overflow".into(),
        }
    })?;
    target.total_hp_lost = target
        .total_hp_lost
        .checked_add(settlement.hp_lost)
        .ok_or_else(|| ExecutionError::InvariantViolation {
            invariant_name: "damage_counter".into(),
            details: "HP counter overflow".into(),
        })?;
    target.current_hp = vitals.hp();
    target.is_alive = vitals.is_alive();
    target.active_shields.retain_mut(|shield| {
        if let Some(remaining) = vitals
            .shields()
            .iter()
            .find(|remaining| remaining.id == shield.id)
        {
            shield.remaining_hp = remaining.remaining();
            true
        } else {
            false
        }
    });
    Ok((
        absorbed,
        settlement.hp_lost,
        settlement.overkill,
        settlement.died,
    ))
}
