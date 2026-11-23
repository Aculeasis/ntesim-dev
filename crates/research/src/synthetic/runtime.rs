use super::*;

fn runtime_entry<'a, T>(
    entries: &'a mut BTreeMap<String, T>,
    id: &str,
    kind: &str,
) -> std::result::Result<&'a mut T, String> {
    entries
        .get_mut(id)
        .ok_or_else(|| format!("internal synthetic runtime: unknown {kind} {id}"))
}

pub fn run(bytes: &[u8]) -> std::result::Result<Result, String> {
    let input: Fixture = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    execute(input, sha256(bytes))
}

pub(crate) fn execute(input: Fixture, input_sha256: String) -> std::result::Result<Result, String> {
    if input.schema_version != 1
        || input.content_id != "synthetic-v1"
        || input.data_status != "synthetic"
        || input.duration_us == 0
    {
        return Err(
            "requires schema 1, synthetic-v1 package, synthetic status and positive duration"
                .into(),
        );
    }
    let mut queue = Scheduler::default();
    let mut rng = Rng::new(input.seed, input.rng_streams.clone())?;
    for c in &input.clocks {
        queue
            .add_clock(c.id, c.rate_ppm)
            .map_err(|e| format!("{e:?}"))?;
    }
    queue
        .local_time_us(input.mode_clock)
        .map_err(|e| format!("{e:?}"))?;
    let mut targets = input
        .targets
        .iter()
        .map(|(id, t)| {
            VitalState::new(t.hp, t.max_hp, t.shield_order)
                .map(|s| (id.clone(), s))
                .map_err(|e| format!("target {id}: {e:?}"))
        })
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    let mut resources = input
        .resources
        .iter()
        .map(|(id, p)| {
            Pool::new(p.current, p.capacity)
                .map(|s| (id.clone(), s))
                .map_err(|e| format!("resource {id}: {e:?}"))
        })
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    let mut actors = BTreeMap::new();
    for (id, actor) in &input.actors {
        if id.trim().is_empty() {
            return Err("empty actor ID".into());
        }
        let initial_time = queue
            .local_time_us(actor.clock)
            .map_err(|e| format!("actor {id}: {e:?}"))?;
        if actor.initial_time_us != initial_time {
            return Err(format!(
                "actor {id}: initial_time_us must match its clock at combat start"
            ));
        }
        let states = actor
            .actions
            .iter()
            .map(|(id, input)| {
                (
                    id.clone(),
                    ActionState {
                        rule: input.rule.clone(),
                        ready_at_us: input.ready_at_us,
                    },
                )
            })
            .collect();
        actors.insert(
            id.clone(),
            Gate::new(actor.clock, initial_time, actor.locked_until_us, states)
                .map_err(|e| format!("actor {id}: {e:?}"))?,
        );
    }
    for (index, c) in input.commands.iter().enumerate() {
        match &c.effect {
            Effect::InvokeAction { actor, action } => {
                let definition = input
                    .actors
                    .get(actor)
                    .ok_or_else(|| format!("command {index}: unknown actor {actor}"))?;
                if definition.clock != c.clock {
                    return Err(format!("command {index}: actor clock mismatch"));
                }
                let action = definition
                    .actions
                    .get(action)
                    .ok_or_else(|| format!("command {index}: unknown actor action {action}"))?;
                if action.effects.is_empty() {
                    return Err(format!("command {index}: actor action has no effects"));
                }
                for pool in action.rule.costs.keys() {
                    if !resources.contains_key(pool) {
                        return Err(format!("command {index}: unknown cost pool {pool}"));
                    }
                }
                for effect in &action.effects {
                    validate_leaf(effect, &targets, &resources, &queue, &rng)
                        .map_err(|e| format!("command {index}: actor effect: {e}"))?;
                }
            }
            effect => validate_leaf(effect, &targets, &resources, &queue, &rng)
                .map_err(|e| format!("command {index}: {e}"))?,
        }
        queue
            .schedule_after(
                c.clock,
                c.delay_us,
                None,
                Task::Root {
                    index,
                    effect: c.effect.clone(),
                },
            )
            .map_err(|e| format!("{e:?}"))?;
    }
    let mut trace = Vec::new();
    let mut diagnostics = Vec::new();
    loop {
        let e = match queue.pop_before(input.duration_us) {
            Ok(Some(e)) => e,
            Ok(None) => break,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    code: "SCHEDULER".into(),
                    path: "clock".into(),
                    message: format!("{error:?}"),
                });
                break;
            }
        };
        let diagnostic_code = match &e.payload {
            Task::Apply {
                effect: Effect::InvokeAction { .. },
                ..
            } => "ACTION_ADMISSION",
            Task::Apply {
                effect: Effect::FormulaDamage { .. },
                ..
            } => "FORMULA_EFFECT",
            Task::Apply {
                effect: Effect::RandomDamage { .. },
                ..
            } => "RANDOM_EFFECT",
            _ => "STATE_TRANSITION",
        };
        let action_origin = match &e.payload {
            Task::Apply { origin, .. } | Task::Hit { origin, .. } => origin.clone(),
            Task::Root { .. } => None,
        };
        let (index, phase, change): (_, _, std::result::Result<Value, String>) = match e.payload {
            Task::Root { index, effect } => (
                index,
                "command",
                queue
                    .schedule_after(
                        e.clock,
                        0,
                        Some(e.event_id),
                        Task::Apply {
                            index,
                            effect,
                            origin: None,
                        },
                    )
                    .map(|_| Value::Null)
                    .map_err(|e| format!("{e:?}")),
            ),
            Task::Apply {
                index,
                effect,
                origin,
            } => {
                let change = match effect {
                    Effect::ObserveState { label } => queue
                        .clock_readings()
                        .map_err(|e| format!("snapshot clock: {e:?}"))
                        .and_then(|clocks| {
                            let observation = Observation {
                                version: "synthetic-state-observation-v1",
                                label: &label,
                                real_time_us: e.real_time_us,
                                clocks,
                                pending_events: queue.pending_count(),
                                targets: &targets,
                                resources: &resources,
                                actors: &actors,
                                rng: if input.rng_streams.is_empty() {
                                    None
                                } else {
                                    Some(&rng)
                                },
                            };
                            serde_json::to_value(observation)
                                .map(|value| json!({"observation":value}))
                                .map_err(|e| e.to_string())
                        }),
                    Effect::InvokeAction { actor, action } => invoke(
                        Invocation {
                            index,
                            clock: e.clock,
                            local_time_us: e.local_time_us,
                            cause: e.event_id,
                        },
                        &actor,
                        &action,
                        &input.actors[&actor].actions[&action].effects,
                        &mut actors,
                        &mut resources,
                        &mut queue,
                    ),
                    Effect::FormulaDamage {
                        target,
                        formula,
                        stats,
                        hit_delay_us,
                    } => formula
                        .evaluate(&stats)
                        .map_err(|error| format!("formula: {error:?}"))
                        .and_then(|evaluation| {
                            queue
                                .schedule_after(
                                    e.clock,
                                    hit_delay_us,
                                    Some(e.event_id),
                                    Task::Hit {
                                        index,
                                        target: target.clone(),
                                        amount: evaluation.amount,
                                        origin,
                                    },
                                )
                                .map(|_| json!({"target":target,"formula":evaluation}))
                                .map_err(|error| format!("scheduler: {error:?}"))
                        }),
                    Effect::RandomDamage {
                        target,
                        stream,
                        outcomes,
                        hit_delay_us,
                    } => random_hit(
                        Invocation {
                            index,
                            clock: e.clock,
                            local_time_us: e.local_time_us,
                            cause: e.event_id,
                        },
                        RandomHit {
                            target: &target,
                            stream: &stream,
                            outcomes: &outcomes,
                            delay_us: hit_delay_us,
                            origin,
                        },
                        &mut rng,
                        &mut queue,
                    ),
                    Effect::Damage {
                        target,
                        amount,
                        hit_delay_us,
                    } => queue
                        .schedule_after(
                            e.clock,
                            hit_delay_us,
                            Some(e.event_id),
                            Task::Hit {
                                index,
                                target,
                                amount,
                                origin,
                            },
                        )
                        .map(|_| Value::Null)
                        .map_err(|e| format!("{e:?}")),
                    Effect::Heal { target, amount } => {
                        runtime_entry(&mut targets, &target, "target")
                            .and_then(|state| state.heal(amount).map_err(|e| format!("{e:?}")))
                            .map(|v| json!({"target":target,"healing":v}))
                    }
                    Effect::Shield {
                        target,
                        id,
                        source,
                        amount,
                    } => runtime_entry(&mut targets, &target, "target")
                        .and_then(|state| {
                            state
                                .add_shield(id.clone(), source, amount)
                                .map_err(|e| format!("{e:?}"))
                        })
                        .map(|_| json!({"target":target,"shield":id,"amount":amount})),
                    Effect::Gain { pool, amount } => {
                        runtime_entry(&mut resources, &pool, "resource")
                            .map(|state| json!({"pool":pool,"gain":state.grant(amount)}))
                    }
                    Effect::Spend { pool, amount } => {
                        runtime_entry(&mut resources, &pool, "resource")
                            .and_then(|state| state.spend(amount).map_err(|e| format!("{e:?}")))
                            .map(|_| json!({"pool":pool,"spent":amount}))
                    }
                    Effect::SetRate { clock, rate_ppm } => queue
                        .set_rate(clock, rate_ppm)
                        .map(|_| json!({"clock":clock,"rate_ppm":rate_ppm}))
                        .map_err(|e| format!("{e:?}")),
                };
                (index, "effect", change)
            }
            Task::Hit {
                index,
                target,
                amount,
                origin: _,
            } => (
                index,
                "hit",
                runtime_entry(&mut targets, &target, "target")
                    .and_then(|state| state.damage(amount).map_err(|e| format!("{e:?}")))
                    .map(|v| json!({"target":target,"damage":v})),
            ),
        };
        let failed = change.is_err();
        let changes = match change {
            Ok(v) => v,
            Err(message) => {
                diagnostics.push(Diagnostic {
                    code: diagnostic_code.into(),
                    path: format!("commands/{index}"),
                    message: message.clone(),
                });
                json!({"error":message})
            }
        };
        trace.push(Trace {
            event_id: e.event_id,
            cause: e.cause,
            command_index: index,
            real_time_us: e.real_time_us,
            local_time_us: e.local_time_us,
            clock: e.clock,
            phase: phase.into(),
            changes,
            action_origin,
        });
        if failed {
            break;
        }
    }
    Ok(Result {
        engine_version: ENGINE_VERSION.into(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        content_id: input.content_id,
        input_sha256,
        seed: input.seed,
        rng_version: if input.rng_streams.is_empty() {
            "unused; no random operations"
        } else {
            rng::VERSION
        }
        .into(),
        rng: if input.rng_streams.is_empty() {
            None
        } else {
            Some(rng)
        },
        quality: Quality::Synthetic,
        success: diagnostics.is_empty(),
        requested_duration_us: input.duration_us,
        real_time_us: queue.real_time_us(),
        mode_time_us: queue
            .local_time_us(input.mode_clock)
            .map_err(|e| format!("{e:?}"))?,
        pending_events: queue.pending_count(),
        diagnostics,
        targets,
        resources,
        actors,
        trace,
    })
}

#[cfg(test)]
mod lookup_tests {
    use super::*;

    #[test]
    fn defensive_lookup_reports_target_and_resource_invariants() {
        let mut entries = BTreeMap::from([("known".into(), 7)]);
        for kind in ["target", "resource"] {
            assert_eq!(
                runtime_entry(&mut entries, "absent", kind).unwrap_err(),
                format!("internal synthetic runtime: unknown {kind} absent")
            );
        }
        *runtime_entry(&mut entries, "known", "resource").unwrap() = 8;
        assert_eq!(entries["known"], 8);
    }
}
