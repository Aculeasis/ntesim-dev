//! Public live names are explicit owner bindings, never reflected state paths.
use super::RuntimeState;
use ntesim_core::kernel::state::wide_fraction;
use num_rational::BigRational;

impl crate::simulation::state::GameSimulatorState {
    pub(crate) fn live_owner_modifier_us(&self, index: usize, name: &str) -> Result<u64, String> {
        let actor = &self.actors[index];
        if actor.hp == 0
            && matches!(actor.character, RuntimeState::Zankou(_))
            && matches!(name, "hunt" | "delusion")
        {
            return Ok(0);
        }
        if let Some(id) = actor.character.managed_modifier(name) {
            use ntesim_core::buffs::dispatch::{BuffDuration, BuffRecipient, BuffSourceLifetime};
            return Ok(self
                .buffs
                .instances()
                .iter()
                .filter(|buff| {
                    buff.id == id
                        && buff.owner_actor == index
                        && buff.stacks > 0
                        && (buff.source_lifetime != BuffSourceLifetime::WhileAlive || actor.hp > 0)
                        && match buff.recipient {
                            BuffRecipient::SelfActor => actor.hp > 0,
                            BuffRecipient::SpecificActor(recipient) => self
                                .actors
                                .get(recipient)
                                .is_some_and(|recipient| recipient.hp > 0),
                            _ => false,
                        }
                })
                .filter_map(|buff| match buff.duration {
                    BuffDuration::Timed { expires_at_us, .. } => {
                        Some(expires_at_us.saturating_sub(self.mode_clock.relative_us()))
                    }
                    BuffDuration::Permanent => None,
                })
                .max()
                .unwrap_or(0));
        }
        if matches!(actor.character, RuntimeState::Adler(_)) && name == "blessing" {
            if actor.hp == 0 {
                return Ok(0);
            }
            let id = super::adler::skill::blessing_shield_id(index);
            return Ok(self
                .actors
                .iter()
                .filter(|recipient| recipient.hp > 0)
                .flat_map(|recipient| &recipient.shields)
                .filter(|shield| shield.id == id && shield.remaining_hp > 0)
                .map(|shield| shield.expires_at_us.saturating_sub(self.current_time_us))
                .max()
                .unwrap_or(0));
        }
        actor.character.live_modifier_us(name, self.current_time_us)
    }

    pub(crate) fn live_owner_state_active(&self, index: usize, name: &str) -> Result<bool, String> {
        let actor = &self.actors[index];
        if actor.character.managed_modifier(name).is_some() {
            return Ok(self.live_owner_modifier_us(index, name)? > 0);
        }
        if matches!(actor.character, RuntimeState::Adler(_)) && name == "blessing" {
            return Ok(self.live_owner_modifier_us(index, name)? > 0);
        }
        actor.live_state_active(name, self.current_time_us)
    }
}

impl crate::simulation::state::ActorRuntime {
    pub(crate) fn live_state_active(&self, name: &str, now: u64) -> Result<bool, String> {
        if let RuntimeState::Zankou(s) = &self.character {
            if name == "bloodfeast" {
                return Ok(self.hp > 0
                    && (s.bloodfeast_ready
                        || (s.a4
                            && self.energy.current() >= 100
                            && now >= s.paid_burst_until_us)));
            }
            if name == "free_inferno" {
                return Ok(self.hp > 0 && now < s.free_inferno_until_us);
            }
        }
        if self.hp == 0
            && matches!(self.character, RuntimeState::Zankou(_))
            && matches!(name, "hunt" | "delusion")
        {
            return Ok(false);
        }
        if name == "aiming"
            && matches!(
                self.character,
                RuntimeState::Adler(_) | RuntimeState::Nanally(_) | RuntimeState::Jiuyuan(_)
            )
        {
            return Ok(self.aiming);
        }
        self.character.live_state_active(name, now)
    }
}

impl RuntimeState {
    /// Closed public names for native managed owner effects. These are owned
    /// effects, including a team carrier, rather than raw BuffManager IDs.
    fn managed_modifier(&self, name: &str) -> Option<&'static str> {
        match (self, name) {
            (Self::Mint(_), "attendance") => Some("mint.a3_attendance_bonus"),
            (Self::Mint(_), "crit_damage") => Some("mint.a5_crit_damage"),
            (Self::Sakiri(_), "team_atk") => Some("Sagiri_QDamUp_AtkAdd"),
            (Self::Zero(_), "support_atk") => Some("Buff_Female051_Level5"),
            _ => None,
        }
    }

    pub(crate) fn live_resource(&self, name: &str, now: u64) -> Result<BigRational, String> {
        match self {
            Self::Zankou(state) if name == "kindling" => wide_fraction(
                crate::simulation::ExactFraction::new(u128::from(state.kindling_stacks), 1)?,
            ),
            Self::Baicang(state) if name == "words" => {
                wide_fraction(crate::simulation::ExactFraction::new(
                    state
                        .stored_words
                        .iter()
                        .filter(|word| word.expiry_us > now)
                        .count() as u128,
                    1,
                )?)
            }
            Self::Edgar(state) if name == "keys" => wide_fraction(
                crate::simulation::ExactFraction::new(u128::from(state.keys_of_truth), 1)?,
            ),
            Self::Adler(state) if name == "karma" => wide_fraction(
                crate::simulation::ExactFraction::new(u128::from(state.karma_stacks), 1)?,
            ),
            Self::Aurelia(state) if name == "crescendo" => {
                wide_fraction(crate::simulation::ExactFraction::new(
                    u128::from(if state.crescendo_expires_at_us > now {
                        state.crescendo_stacks
                    } else {
                        0
                    }),
                    1,
                )?)
            }
            Self::Daffodil(state) if name == "resonance" || name == "phantom_steps" => {
                wide_fraction(crate::simulation::ExactFraction::new(
                    u128::from(if name == "resonance" {
                        state.resonance_stacks
                    } else {
                        state.phantom.charges
                    }),
                    1,
                )?)
            }
            Self::Jiuyuan(state) if name == "bullets" => wide_fraction(
                crate::simulation::ExactFraction::new(u128::from(state.bullets), 1)?,
            ),
            Self::Haniel(state) if name == "plot" || name == "harmony" => {
                wide_fraction(crate::simulation::ExactFraction::new(
                    u128::from(if name == "plot" {
                        state.plot
                    } else {
                        state.harmony
                    }),
                    1,
                )?)
            }
            Self::Iroi(state) if name == "moon" => {
                wide_fraction(super::iroi::resource::remaining(state, now)?)
            }
            Self::Hathor(state) if name == "power" => wide_fraction(
                state
                    .power
                    .as_ref()
                    .ok_or("Hathor Power requires its resource model")?
                    .pool
                    .current(),
            ),
            Self::Hotori(state) if name == "timepiece" => {
                wide_fraction(state.timepiece.projected(now)?.pool.current())
            }
            Self::Blackbird(state) if name == "malice" => wide_fraction(
                state
                    .malice
                    .as_ref()
                    .ok_or("Blackbird Malice requires its resource model")?
                    .current(),
            ),
            Self::Chaos(state) if name == "crime" => wide_fraction(
                state
                    .crime
                    .as_ref()
                    .ok_or("Chaos Crime requires its resource model")?
                    .current(),
            ),
            Self::Chiz(state) if name == "grains" => {
                if let Some(finance) = &state.finance {
                    Ok(finance.balance().clone())
                } else {
                    wide_fraction(
                        state
                            .grain
                            .as_ref()
                            .ok_or("Chiz Grain is not initialized")?
                            .current(),
                    )
                }
            }
            Self::Chiz(state) if name == "skill_stocks" || name == "stocks" => {
                wide_fraction(crate::simulation::ExactFraction::new(
                    u128::from(
                        state
                            .pressed
                            .as_ref()
                            .ok_or("Chiz Skill stocks require the pressed Skill model")?
                            .stocks
                            .current(),
                    ),
                    1,
                )?)
            }
            Self::Chiz(state) if name == "loans" => Ok(state
                .finance
                .as_ref()
                .and_then(|finance| finance.debt.as_ref())
                .ok_or("Chiz Loans require the financial model")?
                .current()
                .clone()),
            Self::Zankou(state) if name == "kindling" => wide_fraction(
                crate::simulation::ExactFraction::new(u128::from(state.kindling_stacks), 1)?,
            ),
            _ => Err(format!(
                "Unknown resource '{name}' for this character model."
            )),
        }
    }

    pub(crate) fn live_modifier_us(&self, name: &str, now: u64) -> Result<u64, String> {
        match self {
            Self::Zankou(state) if name == "free_inferno" => {
                Ok(state.free_inferno_until_us.saturating_sub(now))
            }
            Self::Zankou(state) if matches!(name, "hunt" | "delusion") => Ok(if state.a1 {
                u64::MAX
            } else {
                (if name == "hunt" {
                    state.hunt_until_us
                } else {
                    state.delusion_until_us
                })
                .saturating_sub(now)
            }),
            Self::Zankou(state) if name == "illusion" => Ok(
                if state.form_at(now) == super::zankou::state::Form::Illusion {
                    state.form_until_us.saturating_sub(now)
                } else {
                    0
                },
            ),
            Self::Sakiri(state) if name == "defeat_bonus" => Ok(state
                .a3_kill_bonus_deadlines
                .iter()
                .map(|deadline| deadline.saturating_sub(now))
                .max()
                .unwrap_or(0)),
            Self::Mint(state) if name == "riposte_guard" => {
                Ok(state.a1_guard_until_us.unwrap_or(0).saturating_sub(now))
            }
            Self::Skia(state) if name == "gnaw" => Ok(state.gnaw_until_us.saturating_sub(now)),
            Self::Zankou(state) if name == "scorch_enhancement" => {
                Ok(state.r6_enhancement_until_us.saturating_sub(now))
            }
            Self::Shinku(state) if name == "surging_crimson" => {
                Ok(state.surging_crimson_until_us.saturating_sub(now))
            }
            Self::Linko(state) if name == "oversync" => {
                Ok(super::linko::skill::oversync_remaining_us(state, now))
            }
            Self::Fadia(state) if name == "lilith" => {
                Ok(state.lilith_until_us.unwrap_or(0).saturating_sub(now))
            }
            Self::Baicang(state) if name == "domain" => Ok(if state.domain_start_us <= now {
                state.domain_until_us.saturating_sub(now)
            } else {
                0
            }),
            Self::Edgar(state) if name == "finnegans_wake" => Ok(state
                .field
                .as_ref()
                .map_or(0, |field| field.until.saturating_sub(now))),
            Self::Aurelia(state) if matches!(name, "cadenza" | "crescendo") => {
                Ok(if name == "cadenza" {
                    state.cadenza_until_us
                } else {
                    state.crescendo_expires_at_us
                }
                .saturating_sub(now))
            }
            Self::Nanally(state) if matches!(name, "authority" | "underboss") => {
                Ok(if name == "authority" {
                    state.authority_until_us
                } else {
                    state.underboss_until_us
                }
                .saturating_sub(now))
            }
            Self::Daffodil(state) if name == "eyes" || name == "phantom_handle" => {
                let until = if name == "eyes" {
                    state.insight_until_us
                } else {
                    state.phantom.expires_at_us.unwrap_or(0)
                };
                Ok(until.saturating_sub(now))
            }
            Self::Haniel(state)
                if matches!(name, "hootie" | "paranormal_ace" | "symphony_token") =>
            {
                let until = match name {
                    "hootie" => state.hootie_until_us,
                    "paranormal_ace" => state.paranormal_ace_until_us,
                    _ => state.token.as_ref().map(|token| token.until),
                };
                Ok(until.unwrap_or(0).saturating_sub(now))
            }
            Self::Iroi(state) if name == "dream" => {
                Ok(if super::iroi::resource::dream_active(state, now) {
                    state.dream_end_us.saturating_sub(now)
                } else {
                    0
                })
            }
            Self::Hotori(state) if name == "momentary_gaze" => {
                Ok(state.timepiece.a3_until_us.unwrap_or(0).saturating_sub(now))
            }
            Self::Blackbird(state) if name == "skill_permission" => Ok(state
                .skill_permission_until_us
                .unwrap_or(0)
                .saturating_sub(now)),
            Self::Chiz(state) if name == "market" => Ok(state
                .finance
                .as_ref()
                .and_then(|finance| finance.market.as_ref())
                .and_then(|market| market.session.as_ref())
                .map_or(0, |session| session.ends_at_us.saturating_sub(now))),
            _ => Err(format!(
                "Unknown timed modifier '{name}' for this character model."
            )),
        }
    }

    fn timed_modifier_state(&self, name: &str) -> bool {
        matches!(
            (self, name),
            (Self::Sakiri(_), "defeat_bonus")
                | (Self::Mint(_), "riposte_guard")
                | (Self::Skia(_), "gnaw")
                | (Self::Zankou(_), "scorch_enhancement")
                | (Self::Shinku(_), "surging_crimson")
                | (Self::Linko(_), "oversync")
                | (Self::Fadia(_), "lilith")
                | (Self::Baicang(_), "domain")
                | (Self::Edgar(_), "finnegans_wake")
                | (Self::Aurelia(_), "cadenza" | "crescendo")
                | (Self::Nanally(_), "authority" | "underboss")
                | (Self::Daffodil(_), "eyes" | "phantom_handle")
                | (
                    Self::Haniel(_),
                    "hootie" | "paranormal_ace" | "symphony_token"
                )
        )
    }

    pub(crate) fn live_state_active(&self, name: &str, now: u64) -> Result<bool, String> {
        if self.timed_modifier_state(name) {
            return Ok(self.live_modifier_us(name, now)? > 0);
        }
        match self {
            Self::Zankou(state) if name == "hunt" => Ok(state.is_in_hunt(now)),
            Self::Zankou(state) if name == "delusion" => {
                Ok(state.a1 || state.delusion_until_us > now)
            }
            Self::Zankou(state) if name == "illusion" => {
                Ok(state.form_at(now) == super::zankou::state::Form::Illusion)
            }
            Self::Zankou(state) if name == "reality" => {
                Ok(state.form_at(now) == super::zankou::state::Form::Reality)
            }
            Self::Zankou(state) if name == "enhanced_entry" => {
                Ok(state.skill_variant(now) == super::zankou::forms::Variant::Entry(true))
            }
            Self::Zankou(state) if name == "enhanced_soulcross" => {
                Ok(state.skill_variant(now) == super::zankou::forms::Variant::Return(true))
            }
            Self::Zankou(state) if name == "bloodfeast" => Ok(state.bloodfeast_ready),
            Self::Zankou(state) if name == "free_inferno" => Ok(now < state.free_inferno_until_us),
            Self::Linko(state) if name == "xiaozhen" => Ok(state.pet_active),
            Self::Fadia(state) if name == "out_of_combat" => Ok(state.out_of_combat),
            Self::Edgar(state) if name == "out_of_combat" => Ok(state.out_of_combat),
            Self::Lacrimosa(state) if name == "melee" || name == "ranged" => {
                Ok(if name == "melee" {
                    state.melee
                } else {
                    !state.melee
                })
            }
            Self::Iroi(state) if matches!(name, "dream" | "ivory_dream" | "horn_dream") => {
                Ok(super::iroi::resource::dream_active(state, now)
                    && match name {
                        "ivory_dream" => state.dream_is_ivory,
                        "horn_dream" => !state.dream_is_ivory,
                        _ => true,
                    })
            }
            Self::Hathor(state) if name == "emergency_delivery" => {
                Ok(super::hathor::burst::state_active(state, now))
            }
            Self::Hathor(state) if name == "out_of_combat" => Ok(state.out_of_combat),
            Self::Haniel(state) if name == "out_of_combat" => Ok(state.out_of_combat),
            Self::Hotori(state) if name == "time_stop" => Ok(state.timepiece.mode_active(now)),
            Self::Hotori(state) if name == "rewind" => Ok(state.timepiece.rewinding(now)),
            Self::Blackbird(state) if name == "transformed" => Ok(state.form.is_some()),
            Self::Blackbird(state) if name == "out_of_combat" => Ok(state.out_of_combat),
            Self::Chaos(state) if name == "dread_echo" => Ok(state.form.is_some()),
            Self::Chiz(state) if matches!(name, "market" | "debt" | "surplus") => {
                let finance = state
                    .finance
                    .as_ref()
                    .ok_or("Chiz state requires the financial model")?;
                Ok(match name {
                    "market" => finance
                        .market
                        .as_ref()
                        .is_some_and(|market| market.session.is_some()),
                    "debt" => finance.in_debt(),
                    "surplus" => finance.in_surplus(),
                    _ => unreachable!(),
                })
            }
            Self::Chiz(state) if name == "hoarding" => Ok(state.hoarding_active),
            Self::Chiz(state) if name == "out_of_combat" => Ok(state.out_of_combat),
            _ => Err(format!("Unknown state '{name}' for this character model.")),
        }
    }
}
