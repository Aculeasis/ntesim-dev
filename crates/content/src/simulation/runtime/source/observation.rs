use super::*;
use crate::dsl::evaluate::ExpressionContext;
use crate::simulation::state::GameSimulatorState;

pub(super) struct Observation<'a> {
    pub state: &'a GameSimulatorState,
    pub request: &'a GameScenarioRequest,
    pub actors: &'a [crate::dsl::configuration::CompiledActor],
    pub options: &'a crate::dsl::options::CompiledOptions,
}

impl ExpressionContext for Observation<'_> {
    fn identifier(&self, name: &str) -> Result<Value, String> {
        Err(format!("Unknown variable '{name}'."))
    }

    fn field(&self, parts: &[String]) -> Result<Value, String> {
        let path = parts.iter().map(String::as_str).collect::<Vec<_>>();
        let value = match path.as_slice() {
            ["time"] => return seconds(self.state.mode_clock.relative_us()),
            ["duration"] => {
                if !self.options.duration_specified
                    && self.options.stop != crate::simulation::GameStopPolicy::Victory
                {
                    return Err(
                        "Field .duration is unavailable without a declared duration.".into(),
                    );
                }
                return seconds(
                    if self.options.stop == crate::simulation::GameStopPolicy::Victory {
                        self.request.absolute_limit_us
                    } else {
                        self.request.relative_duration_us
                    },
                );
            }
            ["actor", "energy"] => self.state.actors[self.state.active_actor_index]
                .energy
                .current(),
            ["target", "hp"] => self.state.targets[0].hp_current,
            ["team", "fons"] => self
                .request
                .start_fons
                .ok_or("Missing options start_fons required by .team.fons.")?,
            ["team", field @ ("stamina" | "staminamax")] => {
                let stamina = self.state.combat_stamina.as_ref().ok_or_else(|| {
                    format!("Missing combat stamina profile required by .team.{field}.")
                })?;
                let value = if *field == "stamina" {
                    stamina.current
                } else {
                    crate::simulation::ExactFraction::from_ratio(stamina.profile.capacity)?
                };
                return ntesim_core::kernel::state::wide_fraction(value)
                    .and_then(ExactNumber::from_rational)
                    .map(Value::Number);
            }
            [
                name,
                action @ ("skill" | "burst"),
                property @ ("ready" | "cd"),
            ] => {
                return self.ability_field(name, action, property);
            }
            [name, "support", "ready"] => {
                let index = self.actor_index(name, "support.ready")?;
                u64::from(crate::simulation::support::ready(
                    self.request,
                    self.state,
                    index,
                )?)
            }
            [name, kind @ ("resources" | "mods"), field] => {
                return self.owner_field(name, kind, field);
            }
            [name, state, "active"] => return self.owner_field(name, "state", state),
            [name, field] => return self.actor_field(name, field),
            _ => return Err(format!("Unsupported live field .{}.", parts.join("."))),
        };
        number(value).map(Value::Number)
    }
}

impl Observation<'_> {
    fn owner_field(&self, name: &str, kind: &str, field: &str) -> Result<Value, String> {
        let path = if kind == "state" {
            format!("{field}.active")
        } else {
            format!("{kind}.{field}")
        };
        let index = self.actor_index(name, &path)?;
        let actor = &self.state.actors[index];
        let now = self.state.current_time_us;
        let result = match kind {
            "resources" if field == "cycle" => number(actor.cycle.current()).map(Value::Number),
            "resources" => actor
                .character
                .live_resource(field, now)
                .and_then(ExactNumber::from_rational)
                .map(Value::Number),
            "mods" => self
                .state
                .live_owner_modifier_us(index, field)
                .and_then(seconds),
            "state" => self
                .state
                .live_owner_state_active(index, field)
                .and_then(|active| number(u64::from(active)).map(Value::Number)),
            _ => unreachable!("closed native query kind"),
        };
        result.map_err(|message| format!(".{name}.{path}: {message}"))
    }

    fn actor_index(&self, name: &str, field: &str) -> Result<usize, String> {
        self.actors
            .iter()
            .position(|actor| actor.name == name)
            .ok_or_else(|| format!("Unknown actor '{name}' in live field .{name}.{field}."))
    }

    fn ability_field(&self, name: &str, action: &str, property: &str) -> Result<Value, String> {
        let index = self.actor_index(name, &format!("{action}.{property}"))?;
        let descriptor = self.actors[index]
            .projection
            .public_actions
            .get(action)
            .ok_or_else(|| format!("Unsupported {action} model for {name}."))?;
        let mut candidate = descriptor.command.clone();
        candidate.set_timestamp_us(self.state.current_time_us);
        let command = crate::registry::source_command(
            &self.request.actors[index],
            &self.state.actors[index],
            &candidate,
        )?;
        let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            energy_cost,
            ..
        } = &command
        else {
            return Err(format!("Unsupported {action} ability for {name}."));
        };
        let availability = crate::simulation::availability::ability(
            self.request,
            self.state,
            index,
            ability_id,
            damage_effect_id,
            *energy_cost,
        );
        if property == "cd" {
            seconds(availability.cooldown_us)
        } else {
            number(u64::from(availability.ready)).map(Value::Number)
        }
    }

    fn actor_field(&self, name: &str, field: &str) -> Result<Value, String> {
        let index = self.actor_index(name, field)?;
        let actor = &self.state.actors[index];
        let value = match field {
            "hp" => actor.hp,
            "hpmax" => self.hp_max(index)?,
            "hpratio" => {
                return number(actor.hp)?
                    .divide(&number(self.hp_max(index)?)?)
                    .map(Value::Number);
            }
            "energy" => actor.energy.current(),
            "energymax" => actor.energy.capacity(),
            "onfield" => u64::from(index == self.state.active_actor_index),
            "normal" => actor.character.next_normal(self.state.current_time_us)?,
            "string" => return self.owner_field(name, "resources", "string"),
            "mode" => {
                if self.state.live_owner_state_active(index, "overtone")? {
                    1
                } else if self.state.live_owner_state_active(index, "reverb")? {
                    2
                } else {
                    0
                }
            }
            "overtone" | "reverb" | "cadenza" | "anthem" | "perfect_pitch" => {
                return self.owner_field(name, "state", field);
            }
            _ => return Err(format!("Unsupported live field .{name}.{field}.")),
        };
        number(value).map(Value::Number)
    }

    fn hp_max(&self, index: usize) -> Result<u64, String> {
        let actor = &self.state.actors[index];
        crate::simulation::state::effective_hp_max(
            &self.state.buffs,
            actor.hp_max_base,
            &actor.hp_max_static_bonus,
            index,
            actor.hp_max,
        )
    }
}

fn number(value: u64) -> Result<ExactNumber, String> {
    ExactNumber::decimal(&value.to_string())
}

fn seconds(value: u64) -> Result<Value, String> {
    number(value)?
        .divide(&ExactNumber::from(1_000_000))
        .map(Value::Number)
}
