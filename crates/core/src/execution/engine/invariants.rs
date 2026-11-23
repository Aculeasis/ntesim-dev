use super::*;

pub(crate) fn verify_state_invariants(
    state: &SimState,
    limits: &ExecutionLimits,
) -> Result<usize, ExecutionError> {
    let mut checks = 0usize;

    for (id, pool) in &state.pools {
        checks += 1;
        if !pool.is_conserved() {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "resource_conservation".into(),
                details: format!(
                    "pool {id} not conserved: initial({}) + granted({}) != current({}) + spent({}) + overflow({})",
                    pool.initial,
                    pool.total_granted,
                    pool.current,
                    pool.total_spent,
                    pool.total_overflow
                ),
            });
        }
    }

    for a in &state.actors {
        checks += 2;
        if a.current_hp > a.max_hp {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "actor_vital_bounds".into(),
                details: format!(
                    "actor {} current_hp {} > max_hp {}",
                    a.actor_index, a.current_hp, a.max_hp
                ),
            });
        }
        if (a.current_hp > 0) != a.is_alive {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "actor_vital_consistency".into(),
                details: format!(
                    "actor {} is_alive {} inconsistent with hp {}",
                    a.actor_index, a.is_alive, a.current_hp
                ),
            });
        }
        checks += 1;
        if a.active_shields.len() > limits.max_active_shields {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "shield_count_limit".into(),
                details: format!(
                    "actor {} active shields {} > limit {}",
                    a.actor_index,
                    a.active_shields.len(),
                    limits.max_active_shields
                ),
            });
        }
        for s in &a.active_shields {
            checks += 1;
            if s.remaining_hp > s.max_hp {
                return Err(ExecutionError::InvariantViolation {
                    invariant_name: "shield_vital_bounds".into(),
                    details: format!(
                        "actor {} shield {} remaining {} > max {}",
                        a.actor_index, s.id, s.remaining_hp, s.max_hp
                    ),
                });
            }
        }
    }

    for t in &state.targets {
        checks += 2;
        if t.current_hp > t.max_hp {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "target_vital_bounds".into(),
                details: format!(
                    "target {} current_hp {} > max_hp {}",
                    t.target_id, t.current_hp, t.max_hp
                ),
            });
        }
        if (t.current_hp > 0) != t.is_alive {
            return Err(ExecutionError::InvariantViolation {
                invariant_name: "target_vital_consistency".into(),
                details: format!(
                    "target {} is_alive {} inconsistent with hp {}",
                    t.target_id, t.is_alive, t.current_hp
                ),
            });
        }
    }

    checks += 1;
    let any_alive = state.actors.iter().any(|a| a.is_alive);
    if any_alive
        && (state.active_actor >= state.actors.len() || !state.actors[state.active_actor].is_alive)
    {
        return Err(ExecutionError::InvariantViolation {
            invariant_name: "active_actor_validity".into(),
            details: format!(
                "active actor {} is invalid or dead while alive actors remain",
                state.active_actor
            ),
        });
    }

    checks += 1;
    if state.max_proc_depth_reached > limits.max_proc_depth {
        return Err(ExecutionError::InvariantViolation {
            invariant_name: "max_proc_depth".into(),
            details: format!(
                "max proc depth {} exceeded limit {}",
                state.max_proc_depth_reached, limits.max_proc_depth
            ),
        });
    }

    Ok(checks)
}
