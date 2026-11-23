use super::*;

pub(crate) fn total_weight(outcomes: &[WeightedDamage]) -> std::result::Result<u64, String> {
    if outcomes.is_empty() || outcomes.iter().any(|outcome| outcome.weight == 0) {
        return Err("random damage requires nonempty outcomes with positive weights".into());
    }
    let total = outcomes.iter().try_fold(0_u64, |total, outcome| {
        total
            .checked_add(outcome.weight)
            .ok_or("random damage weight overflow")
    })?;
    if total > rng::WORD_RANGE {
        return Err("random damage total weight exceeds 2^32".into());
    }
    Ok(total)
}

pub(crate) struct RandomHit<'a> {
    pub(crate) target: &'a str,
    pub(crate) stream: &'a str,
    pub(crate) outcomes: &'a [WeightedDamage],
    pub(crate) delay_us: u64,
    pub(crate) origin: Option<ActionOrigin>,
}

pub(crate) fn random_hit(
    context: Invocation,
    hit: RandomHit<'_>,
    rng: &mut Rng,
    queue: &mut Scheduler<Task>,
) -> std::result::Result<Value, String> {
    let bound = total_weight(hit.outcomes)?;
    let mut candidate = rng.clone();
    let sample = candidate.uniform(hit.stream, bound)?;
    let mut remaining = sample.value;
    let outcome_index = hit
        .outcomes
        .iter()
        .position(|outcome| {
            if remaining < outcome.weight {
                true
            } else {
                remaining -= outcome.weight;
                false
            }
        })
        .ok_or("random damage outcome mapping failed")?;
    let amount = hit.outcomes[outcome_index].amount;
    queue
        .schedule_after(
            context.clock,
            hit.delay_us,
            Some(context.cause),
            Task::Hit {
                index: context.index,
                target: hit.target.into(),
                amount,
                origin: hit.origin,
            },
        )
        .map_err(|e| format!("scheduler: {e:?}"))?;
    *rng = candidate;
    Ok(
        json!({"target":hit.target,"random_damage":{"sample":sample,"outcome_index":outcome_index,"amount":amount}}),
    )
}
