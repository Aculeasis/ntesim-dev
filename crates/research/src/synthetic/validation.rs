use super::*;

pub(crate) fn validate_leaf(
    effect: &Effect,
    targets: &BTreeMap<String, VitalState>,
    resources: &BTreeMap<String, Pool>,
    queue: &Scheduler<Task>,
    rng: &Rng,
) -> std::result::Result<(), String> {
    match effect {
        Effect::ObserveState { label } if label.trim().is_empty() || label.len() > 128 => {
            Err("observation label must be nonblank and at most 128 bytes".into())
        }
        Effect::Damage { target, .. }
        | Effect::FormulaDamage { target, .. }
        | Effect::RandomDamage { target, .. }
        | Effect::Heal { target, .. }
        | Effect::Shield { target, .. }
            if !targets.contains_key(target) =>
        {
            Err(format!("unknown target {target}"))
        }
        Effect::FormulaDamage { formula, .. } if formula.unit != "health_quantum" => {
            Err("formula damage requires health_quantum output".into())
        }
        Effect::RandomDamage {
            stream, outcomes, ..
        } => {
            if !rng.contains(stream) {
                return Err(format!("unknown RNG stream {stream}"));
            }
            total_weight(outcomes).map(|_| ())
        }
        Effect::Gain { pool, .. } | Effect::Spend { pool, .. } if !resources.contains_key(pool) => {
            Err(format!("unknown resource {pool}"))
        }
        Effect::SetRate { clock, .. } => queue
            .local_time_us(*clock)
            .map(|_| ())
            .map_err(|e| format!("{e:?}")),
        Effect::InvokeAction { .. } => Err("nested actor invocations are unsupported".into()),
        _ => Ok(()),
    }
}
