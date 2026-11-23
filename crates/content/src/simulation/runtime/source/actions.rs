use super::*;
use crate::dsl::configuration::merge_assumptions;
use crate::execution::CausalPriority;
use crate::simulation::GamePlungeInput;
use crate::simulation::events::{GameEventData, ScheduledGameEvent};

#[derive(Clone, Default)]
pub(crate) struct Parameters {
    pub(crate) window: Option<crate::simulation::SourceActionWindow>,
    pub(crate) target: Option<usize>,
    pub(crate) critical: bool,
    pub(crate) approach: bool,
    pub(crate) parry: bool,
    pub(crate) direction: Option<crate::simulation::GameDashDirection>,
    pub(crate) plunge: Option<GamePlungeInput>,
    pub(crate) hold: Option<crate::simulation::GameHoldInput>,
    pub(crate) branch: Option<crate::simulation::GameBranchInput>,
}

/// Owner clocks can depend on a declared model before the first cast. Bind
/// possible source inputs without evaluating control flow or scheduling them.
pub(super) fn declare_assumptions<'a>(
    config: &'a CompiledConfiguration,
    request: &mut GameScenarioRequest,
) -> Result<std::collections::BTreeSet<(usize, &'a str)>, Diagnostic> {
    use crate::dsl::StatementKind;
    for owner in &config.actors {
        merge_assumptions(
            &mut request.model_assumptions,
            &crate::characters::source_choices::initial_assumptions(&owner.actor.character_id),
        )
        .map_err(|message| failure(message, &owner.loc))?;
    }
    let mut required = std::collections::BTreeSet::new();
    let mut blocks = vec![config.rotation.as_slice()];
    while let Some(block) = blocks.pop() {
        for statement in block {
            match &statement.kind {
                StatementKind::ActionList { actor, actions } => {
                    let (actor_index, owner) = config
                        .actors
                        .iter()
                        .enumerate()
                        .find(|(_, owner)| owner.name == *actor)
                        .ok_or_else(|| {
                            failure(format!("Unknown actor '{actor}'."), &statement.loc)
                        })?;
                    for action in actions {
                        if let Some((_, expression)) =
                            action.params.iter().find(|(key, _)| key == "approach")
                        {
                            crate::characters::source_choices::approach(
                                &owner.actor.character_id,
                                &action.action,
                            )
                            .map_err(|message| failure(message, &expression.loc))?;
                        }
                        declare_window(action, &owner.actor.character_id, request)?;
                        declare_parry(action, &owner.actor.character_id, request)?;
                        merge_assumptions(
                            &mut request.model_assumptions,
                            &crate::characters::source_choices::assumptions(
                                &owner.actor.character_id,
                                &action.action,
                                action.params.iter().any(|(key, _)| key == "hold"),
                            ),
                        )
                        .map_err(|message| failure(message, &action.loc))?;
                        if owner.actor.character_id == "character:1025"
                            && action.action != "plunge"
                            && action.params.iter().any(|(key, _)| {
                                matches!(key.as_str(), "launchable" | "landing" | "height_bonus")
                            })
                        {
                            merge_assumptions(
                                &mut request.model_assumptions,
                                &crate::characters::hathor::branch::source_assumptions(),
                            )
                            .map_err(|message| failure(message, &action.loc))?;
                        }
                        for name in [action.action.as_str(), "swap"] {
                            required.insert((actor_index, name));
                            let descriptor =
                                owner.projection.public_actions.get(name).ok_or_else(|| {
                                    failure(
                                        format!("Unsupported {name} model for {actor}."),
                                        &action.loc,
                                    )
                                })?;
                            merge_assumptions(
                                &mut request.model_assumptions,
                                &descriptor.model_assumptions,
                            )
                            .map_err(|message| failure(message, &action.loc))?;
                        }
                    }
                }
                StatementKind::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    blocks.push(then_branch);
                    if let Some(branch) = else_branch {
                        blocks.push(branch);
                    }
                }
                StatementKind::Repeat { body, .. } | StatementKind::While { body, .. } => {
                    blocks.push(body)
                }
                StatementKind::For(loop_) => blocks.push(&loop_.body),
                _ => {}
            }
        }
    }
    Ok(required)
}

fn declare_window(
    action: &crate::dsl::ActionInvocation,
    character: &str,
    request: &mut GameScenarioRequest,
) -> Result<(), Diagnostic> {
    if let Some((_, expression)) = action.params.iter().find(|(key, _)| key == "window") {
        let window = crate::dsl::action_parameters::window(expression)
            .map_err(|message| failure(message, &expression.loc))?;
        if window == crate::simulation::SourceActionWindow::Swap {
            return if character == "character:1004" && action.action == "dash" {
                Ok(())
            } else {
                Err(failure(
                    "The swap window requires Lacrimosa's dash.".into(),
                    &expression.loc,
                ))
            };
        }
        let assumptions = crate::characters::source_choices::window_assumptions(character, window)
            .map_err(|message| failure(message, &expression.loc))?;
        merge_assumptions(&mut request.model_assumptions, &assumptions)
            .map_err(|message| failure(message, &expression.loc))?;
    }
    Ok(())
}

impl Trial<'_> {
    pub(super) fn action(
        &mut self,
        actor: &str,
        action: &str,
        params: &[(&str, Value)],
        loc: &SourceLocation,
    ) -> Result<(), String> {
        let params = parameters(action, params, self.request.targets.len())?;
        self.occurrences
            .last_mut()
            .expect("current occurrence")
            .action_window = params.window;
        let actor_index = self
            .program
            .config
            .actors
            .iter()
            .position(|config| config.name == actor)
            .ok_or_else(|| format!("Unknown actor '{actor}'."))?;
        let descriptors = &self.program.config.actors[actor_index]
            .projection
            .public_actions;
        let descriptor = descriptors
            .get(action)
            .ok_or_else(|| format!("Unsupported {action} model for {actor}."))?;
        if actor_index != self.execution.state.active_actor_index {
            let switch = descriptors
                .get("swap")
                .ok_or("Switch descriptor is missing.")?;
            let implicit = Parameters::default();
            self.admit_descriptor(
                actor_index,
                "swap",
                switch,
                if action == "swap" { &params } else { &implicit },
                loc,
            )?;
            if !self.execution.success
                || !self.execution.execution_completed
                || self.victory()
                || self.time_completed()
            {
                return Ok(());
            }
            if action == "swap" {
                return Ok(());
            }
        }
        // Same-actor swaps still pass through native no-op admission.
        self.admit_descriptor(actor_index, action, descriptor, &params, loc)?;
        Ok(())
    }

    fn admit_descriptor(
        &mut self,
        actor: usize,
        name: &str,
        descriptor: &crate::build::PublicAction,
        params: &Parameters,
        loc: &SourceLocation,
    ) -> Result<(), String> {
        self.command_budget()?;
        let mut candidate = descriptor.command.clone();
        candidate.set_timestamp_us(self.execution.state.current_time_us);
        let mut command = crate::registry::source_command(
            &self.request.actors[actor],
            &self.execution.state.actors[actor],
            &candidate,
        )?;
        let same_actor_switch = matches!(&command, GameActionCommand::SwitchActor { .. })
            && actor == self.execution.state.active_actor_index;
        let now = self.execution.state.current_time_us;
        match &mut command {
            GameActionCommand::InvokeAbility {
                timestamp_us,
                actor_index,
                ..
            }
            | GameActionCommand::PlayerInput {
                timestamp_us,
                actor_index,
                ..
            } => {
                *timestamp_us = 0;
                *actor_index = actor;
            }
            GameActionCommand::SwitchActor {
                timestamp_us,
                target_actor_index,
                ..
            } => {
                *timestamp_us = 0;
                *target_actor_index = actor;
            }
            _ => return Err("Invalid public action descriptor.".into()),
        }
        if let GameActionCommand::PlayerInput {
            critical_success, ..
        } = &mut command
        {
            *critical_success = params.critical;
        }

        let resolved = self
            .program
            .resolve_action(actor, name, descriptor, command, params)?;
        merge_assumptions(
            &mut self.request.model_assumptions,
            &resolved.model_assumptions,
        )?;
        let index = self.request.actions.len();
        let mut projected_command = resolved.projected_command.clone();
        projected_command.set_timestamp_us(now);

        let event = match &projected_command {
            GameActionCommand::SwitchActor {
                target_actor_index,
                cooldown_us,
                ..
            } => GameEventData::SwitchActor {
                command_index: index,
                target_actor_index: *target_actor_index,
                cooldown_us: *cooldown_us,
            },
            _ => GameEventData::ActionAdmission {
                command_index: index,
            },
        };
        self.request.actions.push(projected_command);
        if self.execution.state.record_detail {
            let mut replay_command = resolved.replay_command.clone();
            replay_command.set_timestamp_us(now);
            self.replay_actions.push(replay_command);
            self.locations.push(loc.clone());
        }
        self.packets.push(resolved.packet.clone());
        if params.parry {
            let event_index = self.request.parry_events.len();
            self.request
                .parry_events
                .push(crate::simulation::GameActorParryEvent::AfterAction {
                    command_index: index,
                });
            self.execution.register_action_parry(index, event_index)?;
        }
        if let Some(target) = params.target {
            self.request.aim_targets.insert(index, target);
        }
        if params.approach {
            self.request.approach_inputs.insert(index);
        }
        if let Some(plunge) = params.plunge {
            self.request.plunge_inputs.insert(index, plunge);
        }
        if let Some(hold) = params.hold {
            self.request.hold_inputs.insert(index, hold);
        }
        if let Some(direction) = params.direction {
            self.request.dash_directions.insert(index, direction);
        }
        if let Some(branch) = params.branch {
            self.request.branch_inputs.insert(index, branch);
        }
        if self.execution.state.record_detail {
            for receipt in &resolved.talent_receipts {
                crate::simulation::talents::push_receipt(
                    &mut self.talents,
                    receipt.clone(),
                    index,
                )?;
            }
        }
        self.execution.seq += 1;
        self.execution.queue.push(ScheduledGameEvent {
            time_us: now,
            priority: CausalPriority::Observation,
            seq: self.execution.seq,
            data: event,
        });
        self.advance(now)?;
        if same_actor_switch {
            return Ok(());
        }
        let context = self.begin_time_context(
            crate::simulation::GameTimeContextKind::Action {
                command_index: index,
            },
            loc,
        );
        let result = self.advance_action_window(actor, params.window);
        self.end_time_context(context);
        result
    }
}

fn declare_parry(
    action: &crate::dsl::ActionInvocation,
    character: &str,
    request: &mut GameScenarioRequest,
) -> Result<(), Diagnostic> {
    if let Some((_, expression)) = action.params.iter().find(|(name, _)| name == "parry") {
        let models = crate::characters::parry_inputs::Input::source(&action.action)
            .and_then(|input| crate::characters::parry_inputs::assumptions(character, input))
            .map_err(|message| failure(message, &expression.loc))?;
        merge_assumptions(&mut request.model_assumptions, &models)
            .map_err(|message| failure(message, &expression.loc))?;
    }
    Ok(())
}

fn parameters(
    action: &str,
    params: &[(&str, Value)],
    target_count: usize,
) -> Result<Parameters, String> {
    let window = params
        .iter()
        .find(|(key, _)| *key == "window")
        .map(|(_, value)| {
            let Value::Text(name) = value else {
                return Err("window requires a static input-kind identifier.".into());
            };
            crate::dsl::action_parameters::window_name(name)
        })
        .transpose()?;
    let target = params
        .iter()
        .find(|(key, _)| *key == "target")
        .map(|(_, value)| {
            let Value::Text(name) = value else {
                return Err("target requires a dummy identifier tN.".into());
            };
            crate::dsl::action_parameters::target_name(name, target_count)
        })
        .transpose()?;
    if action == "plunge" {
        return Ok(Parameters {
            window,
            target,
            plunge: Some(plunge(params)?),
            hold: params
                .iter()
                .find(|(key, _)| *key == "hold")
                .map(|(_, value)| {
                    Ok::<_, String>(crate::simulation::GameHoldInput {
                        duration_us: value.number()?.to_microseconds()?,
                        moving: false,
                        reserved_stocks: None,
                    })
                })
                .transpose()?,
            ..Default::default()
        });
    }
    let mut result = Parameters {
        window,
        target,
        branch: crate::dsl::action_parameters::branch(
            params,
            params.iter().any(|(key, _)| *key == "hold"),
        )?,
        ..Default::default()
    };
    let mut moving = None;
    let mut stocks = None;
    for (key, value) in params {
        if *key == "approach" {
            result.approach = crate::dsl::action_parameters::boolean(value, "approach")?;
            continue;
        }
        if *key == "parry" {
            result.parry = crate::dsl::action_parameters::boolean(value, "parry")?;
            continue;
        }
        if matches!(
            *key,
            "target" | "window" | "launchable" | "landing" | "height_bonus"
        ) {
            continue;
        }
        if *key == "stocks" {
            stocks = Some(crate::dsl::action_parameters::stocks(value)?);
            continue;
        }
        if *key == "moving" {
            moving = Some(crate::dsl::action_parameters::boolean(
                value,
                &format!("{action} moving"),
            )?);
            continue;
        }
        if *key == "hold" {
            result.hold = Some(crate::simulation::GameHoldInput {
                duration_us: value.number()?.to_microseconds()?,
                moving: false,
                reserved_stocks: None,
            });
            continue;
        }
        if action != "dash" || !matches!(*key, "critical" | "back") {
            return Err(format!("Unsupported {action} action parameter '{key}'."));
        }
        let flag = crate::dsl::action_parameters::boolean(value, &format!("dash {key}"))?;
        if *key == "critical" {
            result.critical = flag;
        } else {
            result.direction = Some(if flag {
                crate::simulation::GameDashDirection::Back
            } else {
                crate::simulation::GameDashDirection::Front
            });
        }
    }
    if let Some(moving) = moving {
        result
            .hold
            .as_mut()
            .ok_or("moving requires an explicit hold duration.")?
            .moving = moving;
    }
    if let Some(stocks) = stocks {
        result
            .hold
            .as_mut()
            .ok_or("stocks require an explicit hold duration.")?
            .reserved_stocks = Some(stocks);
    }
    Ok(result)
}

fn plunge(params: &[(&str, Value)]) -> Result<GamePlungeInput, String> {
    let mut landing = None;
    let mut height = None;
    for (key, value) in params {
        match *key {
            "target" | "window" | "hold" => {}
            "airtime" if landing.is_none() => {
                let duration = value.number()?.to_microseconds()?;
                if duration == 0 {
                    return Err("plunge airtime must be positive.".into());
                }
                landing = Some(duration);
            }
            "height_bonus" if height.is_none() => {
                let (numerator, denominator) = value.number()?.to_safe_ratio()?;
                if numerator > denominator {
                    return Err("plunge height_bonus must be within0..1.".into());
                }
                height = Some(crate::formula::Ratio {
                    numerator,
                    denominator,
                });
            }
            _ => return Err(format!("Unsupported or repeated plunge parameter '{key}'.")),
        }
    }
    Ok(GamePlungeInput {
        landing_delay_us: landing.ok_or("plunge requires airtime.")?,
        height_bonus: height.ok_or("plunge requires height_bonus.")?,
    })
}
