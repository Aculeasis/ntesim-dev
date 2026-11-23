use super::*;

#[derive(Clone, Copy)]
pub(crate) enum Family {
    Jump,
    Skill,
    Movement,
}

pub(crate) fn gate(
    actor: &ActorRuntime,
    now: u64,
    continuation: bool,
    family: Family,
) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Chaos(state) = &actor.character else {
        return Ok(None);
    };
    let Some(clock) = &state.clock else {
        return Ok(None);
    };
    let Some(action) = &clock.action else {
        return Ok(None);
    };
    let end_progress = if continuation {
        continuation_progress(action).unwrap_or(action.recovery)
    } else {
        action.recovery
    };
    let end = deadline(clock, action, end_progress)?;
    if now >= end {
        return Ok(None);
    }
    let gate = match family {
        Family::Jump => jump_progress(action.kind, action.recovery)?,
        Family::Skill => match action.kind {
            Kind::Dodge(back) => {
                if back {
                    590_149
                } else {
                    333_716
                }
            }
            Kind::Riposte => 100,
            _ => action.recovery,
        },
        Family::Movement => match action.kind {
            Kind::Normal(stage, _) => *[473_455, 1_148_914, 587_265, 1_227_315, 1_238_200]
                .get(stage)
                .ok_or("Chaos movement stage missing")?,
            Kind::AppearNormal => 473_455,
            Kind::VerdictPlain(transformed) => {
                if transformed {
                    1_116_171
                } else {
                    1_108_887
                }
            }
            Kind::VerdictFull(transformed) => {
                if transformed {
                    626_785
                } else {
                    651_287
                }
            }
            Kind::Skill => 1_209_865,
            Kind::Burst => 5_412_379,
            Kind::Support => 1_291_110,
            Kind::Plunge => 825_108,
            Kind::Riposte => 366_207,
            Kind::Dodge(back) => {
                if back {
                    599_939
                } else {
                    action.recovery
                }
            }
        },
    };
    Ok(Some(deadline(clock, action, gate.min(end_progress))?))
}

fn jump_progress(kind: Kind, recovery: u64) -> Result<u64, String> {
    Ok(match kind {
        Kind::Normal(stage, transformed) => match stage {
            0 => 263_727,
            1 => 464_920,
            2 => {
                if transformed {
                    483_244
                } else {
                    502_412
                }
            }
            3 => 1_209_621,
            4 => 1_061_822,
            _ => return Err("Chaos cancel stage missing".into()),
        },
        Kind::VerdictPlain(transformed) => {
            if transformed {
                182_860
            } else {
                183_883
            }
        }
        Kind::VerdictFull(transformed) => {
            if transformed {
                632_402
            } else {
                674_412
            }
        }
        Kind::AppearNormal => 263_727,
        Kind::Support => super::super::support::RECOVERY,
        Kind::Dodge(back) => {
            if back {
                100
            } else {
                82_544
            }
        }
        Kind::Riposte => 100,
        Kind::Plunge => recovery,
        Kind::Skill => 1_324_696,
        Kind::Burst => 5_412_379,
    })
}

fn continuation_progress(action: &Action) -> Option<u64> {
    match action.kind {
        Kind::VerdictFull(true) => Some(1_102_773),
        Kind::VerdictFull(false) => Some(1_088_377),
        _ => None,
    }
}

pub(super) fn continuation_end(clock: &Clock, action: &Action) -> Result<Option<u64>, String> {
    continuation_progress(action)
        .map(|progress| deadline(clock, action, progress))
        .transpose()
}

fn deadline(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .tracker
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|error| format!("Chaos cancel deadline: {error:?}"))
}

/// A combo reads the preceding admitted Normal's captured form and progress.
/// Hold duration overlaps that montage on the real axis, never a new idle lead-in.
pub(crate) fn combo(
    actor: &ActorRuntime,
    now: u64,
    hold: u64,
    normal: bool,
    continuation: bool,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    let crate::characters::RuntimeState::Chaos(state) = &actor.character else {
        return Ok(None);
    };
    let Some(clock) = &state.clock else {
        return Ok(None);
    };
    let Some(action) = &clock.action else {
        return Ok(None);
    };
    let minimum_hold = minimum_hold(action.kind);
    if let Kind::VerdictFull(_) = action.kind
        && continuation
    {
        // The reached EndAbilitySkill marker bounds the live branch after its
        // earlier AnySkill admission gate. Unused pose-tail length is not retained.
        let end = continuation_end(clock, action)?.ok_or("Chaos continuation endpoint missing")?;
        if now >= end {
            return Ok(None);
        }
        if hold < minimum_hold {
            return Err(
                "Chaos full Verdict continuation requires .1s transformed or .2s ordinary hold"
                    .into(),
            );
        }
        let gate = deadline(clock, action, 1_060_001)?.max(
            action
                .start
                .checked_add(hold)
                .ok_or("Chaos continuation hold overflow")?,
        );
        if gate >= end {
            return Err("Chaos continuation hold cannot fit before EndAbilitySkill".into());
        }
        return Ok(Some((gate, super::super::cancel::CONTINUATION)));
    }
    if !normal || now >= deadline(clock, action, action.recovery)? {
        return Ok(None);
    }
    let progress = match action.kind {
        Kind::Normal(stage, transformed) => {
            if hold < minimum_hold {
                return Err("Chaos Normal combo hold requires at least.2 real seconds".into());
            }
            normal_progress(stage, transformed)?
        }
        Kind::AppearNormal => {
            if hold < minimum_hold {
                return Err(
                    "Chaos Appear Normal combo requires at least.25 real seconds held".into(),
                );
            }
            300_000
        }
        Kind::Plunge => {
            if hold < minimum_hold {
                return Err("Chaos plunge combo requires at least.2 real seconds held".into());
            }
            120_000
        }
        _ => return Ok(None),
    };
    let held = action
        .input_start
        .checked_add(hold)
        .ok_or("Chaos combo hold overflow")?;
    if held >= deadline(clock, action, action.recovery)? {
        return Err("Chaos combo hold cannot fit inside its preceding montage".into());
    }
    Ok(Some((
        deadline(clock, action, progress + 1)?.max(held),
        super::super::cancel::COMBO,
    )))
}

fn normal_progress(stage: usize, transformed: bool) -> Result<u64, String> {
    Ok(match stage {
        0 => 300_000,
        1 => {
            if transformed {
                640_000
            } else {
                600_000
            }
        }
        2 => 400_000,
        3 => 1_090_000,
        4 => {
            if transformed {
                900_000
            } else {
                850_000
            }
        }
        _ => return Err("Chaos combo stage missing".into()),
    })
}

pub(crate) fn input_window(
    actor: &ActorRuntime,
    now: u64,
    normal: bool,
    continuation: bool,
) -> Result<u64, String> {
    let crate::characters::RuntimeState::Chaos(state) = &actor.character else {
        return Err("Chaos hold window requires its native state.".into());
    };
    let Some(action) = state.clock.as_ref().and_then(|clock| clock.action.as_ref()) else {
        return Ok(actor.locked_until_us);
    };
    // A plain stage can still commit a funded full stage at its boundary.
    // Advance through native events and re-evaluate instead of predicting Crime.
    Ok(
        combo(actor, now, minimum_hold(action.kind), normal, continuation)?
            .map_or(actor.locked_until_us, |(deadline, _)| deadline),
    )
}

fn minimum_hold(kind: Kind) -> u64 {
    match kind {
        Kind::AppearNormal => 250_000,
        Kind::VerdictFull(true) => 100_000,
        _ => 200_000,
    }
}
