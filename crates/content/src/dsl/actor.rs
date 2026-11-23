//! Build declarations delegate stat and passive derivation to the existing Rust projector.

use ntesim_core::formula::Ratio;
use std::collections::BTreeMap;

use super::{
    catalog::Catalog,
    configuration::failure,
    evaluate::{Bindings, Evaluator, Value},
    number::ExactNumber,
    validation::Diagnostic,
    *,
};
use crate::{
    build::{EquipmentProjection, studio_projection},
    simulation::GameActorConfig,
};

#[derive(Clone, Debug)]
pub struct CompiledActor {
    pub name: String,
    pub loc: SourceLocation,
    pub weapon_level: Option<u32>,
    pub weapon_max_level: Option<u32>,
    pub actor: GameActorConfig,
    pub projection: studio_projection::Response,
    pub contributions: BTreeMap<String, Ratio>,
    pub set_count: u32,
}

pub(super) struct ActorDraft {
    pub name: String,
    pub loc: SourceLocation,
    request: studio_projection::Request,
    talents: [u32; 3],
    qte_level: u32,
    start_hp: Option<u64>,
    start_hp_ratio: Option<ExactNumber>,
    start_energy: Option<u64>,
    start_cycle: Option<u64>,
    initial_resources: crate::simulation::GameInitialResources,
    totals: BTreeMap<String, ExactNumber>,
}

fn integer(value: &Value) -> Result<u64, String> {
    value.number()?.to_safe_integer()
}

fn evaluate(
    expression: &Expression,
    scope: &Bindings,
    evaluator: &mut Evaluator,
) -> Result<Value, Diagnostic> {
    evaluator.evaluate(expression, scope).map_err(Into::into)
}

impl ActorDraft {
    pub fn new(
        declaration: &Declaration,
        slot: usize,
        catalog: &Catalog,
        scope: &Bindings,
        evaluator: &mut Evaluator,
    ) -> Result<Self, Diagnostic> {
        let fail = |message| failure(message, &declaration.loc);
        let DeclarationKind::Character {
            actor,
            level,
            awakening,
            talents,
            qte,
            extra_fields,
        } = &declaration.kind
        else {
            return Err(fail("Expected a character declaration.".into()));
        };
        let level = level
            .as_ref()
            .ok_or_else(|| fail("char requires lvl=current/cap.".into()))?;
        let selection = match awakening
            .as_ref()
            .ok_or_else(|| fail("char requires awakening.".into()))?
        {
            AwakeningSelection::Rank(rank) if *rank <= 6 => {
                ntesim_data::AwakeningSelection::prefix(*rank)
            }
            AwakeningSelection::Explicit(indices) => {
                let indices = indices
                    .iter()
                    .map(|i| u8::try_from(*i).map_err(|_| "Invalid awakening index.".to_owned()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(fail)?;
                ntesim_data::AwakeningSelection::selected(indices).map_err(fail)?
            }
            _ => return Err(fail("Awakening rank must be in 0..6.".into())),
        };
        let talents = talents
            .as_ref()
            .ok_or_else(|| fail("char requires three talent levels.".into()))?;
        let mut purchased = [0; 3];
        for (index, expression) in [&talents.normal, &talents.skill, &talents.burst]
            .into_iter()
            .enumerate()
        {
            let value = integer(&evaluate(expression, scope, evaluator)?)
                .map_err(|message| failure(message, &expression.loc))?;
            purchased[index] = u32::try_from(value)
                .map_err(|_| fail("Talent level exceeds its integer range.".into()))?;
        }
        let qte = qte
            .as_ref()
            .ok_or_else(|| fail("char requires qte=<level>.".into()))?;
        let qte_level = integer(&evaluate(qte, scope, evaluator)?)
            .and_then(|value| {
                u32::try_from(value).map_err(|_| "QTE level exceeds its integer range.".into())
            })
            .map_err(|message| failure(message, &qte.loc))?;
        let mut draft = Self {
            name: actor.clone(),
            loc: declaration.loc.clone(),
            talents: purchased,
            qte_level,
            request: studio_projection::Request {
                slot,
                content_id: "ntesim-current".into(),
                character_id: catalog.resolve("character", actor).map_err(fail)?,
                level: level.current,
                max_level: Some(level.cap),
                awakening: selection,
                weapon_id: None,
                weapon_level: None,
                weapon_max_level: None,
                refinement: 1,
                set_id: None,
                set_count: 0,
                base_overrides: Default::default(),
                additive_stats: BTreeMap::new(),
            },
            start_hp: None,
            start_hp_ratio: None,
            start_energy: None,
            start_cycle: None,
            initial_resources: Default::default(),
            totals: BTreeMap::new(),
        };
        for (key, expression) in extra_fields {
            let value = evaluate(expression, scope, evaluator)?;
            let result = match key.as_str() {
                "start_hp" => integer(&value).map(|v| draft.start_hp = Some(v)),
                "start_energy" => integer(&value).map(|v| draft.start_energy = Some(v)),
                "start_timepiece" | "start_grains" | "start_loans" | "start_malice"
                | "start_crime" | "start_power" | "start_moon" | "start_plot" | "start_bullets"
                | "start_resonance" | "start_karma" | "start_keys" | "start_stocks"
                | "start_skill_stocks" | "start_kindling" => value.number().and_then(|v| {
                    let (numerator, denominator) = v.to_safe_ratio()?;
                    let initial = Some(Ratio {
                        numerator,
                        denominator,
                    });
                    match key.as_str() {
                        "start_timepiece" => draft.initial_resources.timepiece = initial,
                        "start_grains" => draft.initial_resources.grains = initial,
                        "start_loans" => draft.initial_resources.loans = initial,
                        "start_malice" => draft.initial_resources.malice = initial,
                        "start_crime" => draft.initial_resources.crime = initial,
                        "start_power" => draft.initial_resources.power = initial,
                        "start_moon" => draft.initial_resources.moon = initial,
                        "start_plot" => draft.initial_resources.plot = initial,
                        "start_bullets" => draft.initial_resources.bullets = initial,
                        "start_resonance" => draft.initial_resources.resonance = initial,
                        "start_karma" => draft.initial_resources.karma = initial,
                        "start_keys" => draft.initial_resources.keys = initial,
                        "start_stocks" | "start_skill_stocks" => {
                            draft.initial_resources.stocks = initial
                        }
                        "start_kindling" => draft.initial_resources.kindling = initial,
                        _ => unreachable!(),
                    }
                    crate::registry::validate_initial_resources(
                        &draft.request.character_id,
                        draft.request.level,
                        &draft.request.awakening,
                        &draft.initial_resources,
                    )
                }),
                "start_cycle" => integer(&value).and_then(|v| {
                    if v > 100 {
                        return Err("start_cycle exceeds the nominal 100 Cycle capacity.".into());
                    }
                    draft.start_cycle = Some(v);
                    Ok(())
                }),
                "start_hp%" => value.number().and_then(|v| {
                    if v <= &ExactNumber::from(0) || v > &ExactNumber::from(1) {
                        return Err("start_hp% must be greater than 0 and at most 1.".into());
                    }
                    draft.start_hp_ratio = Some(v.clone());
                    Ok(())
                }),
                _ => Err(format!("Unknown character parameter '{key}'.")),
            };
            result.map_err(|message| failure(message, &expression.loc))?;
        }
        if draft.start_hp.is_some() && draft.start_hp_ratio.is_some() {
            return Err(fail(
                "start_hp and start_hp% are mutually exclusive.".into(),
            ));
        }
        Ok(draft)
    }

    pub fn equip(
        &mut self,
        declaration: &Declaration,
        catalog: &Catalog,
    ) -> Result<(), Diagnostic> {
        let fail = |message| failure(message, &declaration.loc);
        match &declaration.kind {
            DeclarationKind::Weapon {
                weapon_id,
                refinement,
                level,
                extra_fields,
                ..
            } => {
                if self.request.weapon_id.is_some() {
                    return Err(fail("Duplicate weapon declaration.".into()));
                }
                if let Some((key, _)) = extra_fields.first() {
                    return Err(fail(format!("Unknown weapon parameter '{key}'.")));
                }
                let refinement =
                    refinement.ok_or_else(|| fail("weapon requires refine.".into()))?;
                if !(1..=5).contains(&refinement) {
                    return Err(fail("refine must be in 1..5.".into()));
                }
                self.request.weapon_id = Some(catalog.resolve("weapon", weapon_id).map_err(fail)?);
                self.request.refinement = refinement;
                self.request.weapon_level = Some(level.as_ref().map_or(80, |l| l.current));
                self.request.weapon_max_level = Some(level.as_ref().map_or(80, |l| l.cap));
            }
            DeclarationKind::Cartridge {
                cartridge_id,
                count,
                ..
            } => {
                if self.request.set_id.is_some() {
                    return Err(fail("Multiple cartridge sets are not supported.".into()));
                }
                if !(1..=4).contains(count) {
                    return Err(fail("cartridge count must be in 1..4.".into()));
                }
                self.request.set_id =
                    Some(catalog.resolve("cartridge", cartridge_id).map_err(fail)?);
                self.request.set_count = *count;
            }
            _ => return Err(fail("Expected equipment declaration.".into())),
        }
        Ok(())
    }

    pub fn stats(
        &mut self,
        pairs: &[(String, Expression)],
        scope: &Bindings,
        evaluator: &mut Evaluator,
    ) -> Result<(), Diagnostic> {
        for (key, expression) in pairs {
            let fail = |message| failure(message, &expression.loc);
            let native =
                stat_key(key).ok_or_else(|| fail(format!("Unsupported combat stat '{key}'.")))?;
            let value = evaluate(expression, scope, evaluator)?;
            let value = value.number().map_err(fail)?;
            if value < &ExactNumber::from(0) {
                return Err(fail("Stat contributions must be nonnegative.".into()));
            }
            let total = self
                .totals
                .entry(native.into())
                .or_insert_with(|| ExactNumber::from(0));
            *total = total.add(value).map_err(fail)?;
        }
        Ok(())
    }

    pub fn finish(
        mut self,
        tables: &BTreeMap<String, serde_json::Value>,
    ) -> Result<CompiledActor, Diagnostic> {
        let fail = |message| failure(message, &self.loc);
        for (key, value) in &self.totals {
            let (numerator, denominator) = value.to_safe_ratio().map_err(fail)?;
            self.request.additive_stats.insert(
                key.clone(),
                Ratio {
                    numerator,
                    denominator,
                },
            );
        }
        let projection = {
            #[cfg(feature = "performance-diagnostics")]
            let _span = crate::performance::Span::new("source.actor_projection");
            studio_projection::project(tables, &self.request).map_err(fail)?
        };
        let gear_procs = match &projection.equipment_projection {
            EquipmentProjection::Supported { gear_procs, .. } => gear_procs.clone(),
            EquipmentProjection::Unsupported { diagnostic } => {
                return Err(fail(diagnostic.clone()));
            }
        };
        let max_hp = projection.static_stats.hp;
        let hp = if let Some(ratio) = self.start_hp_ratio {
            ratio.floor_product(max_hp).map_err(fail)?
        } else {
            self.start_hp.unwrap_or(max_hp)
        };
        if hp == 0 || hp > max_hp {
            return Err(fail(
                "start_hp must be positive and at most maximum HP.".into(),
            ));
        }
        let energy = self.start_energy.unwrap_or(projection.energy_capacity);
        if energy > projection.energy_capacity {
            return Err(fail("start_energy exceeds energy capacity.".into()));
        }
        let actor = GameActorConfig {
            slot: self.request.slot,
            character_id: self.request.character_id,
            character_level: self.request.level,
            awakening: self.request.awakening,
            purchased_skill_level: self.talents[1],
            purchased_qte_level: Some(self.qte_level),
            talent_levels: Some(self.talents),
            character_max_level: self.request.max_level,
            initial_hp: hp,
            initial_hp_max: max_hp,
            initial_energy: energy,
            energy_capacity: projection.energy_capacity,
            initial_cycle: self.start_cycle.unwrap_or(0),
            initial_resources: self.initial_resources,
            cycle_gain_efficiency: ntesim_core::formula::Ratio::one(),
            weapon_id: self.request.weapon_id.unwrap_or_default(),
            weapon_tier: self.request.refinement,
            set_id: self.request.set_id,
            static_stats: projection.static_stats.clone(),
            gear_procs,
        };
        crate::simulation::qte_coordinate(&actor).map_err(fail)?;
        Ok(CompiledActor {
            name: self.name,
            loc: self.loc,
            weapon_level: self.request.weapon_level,
            weapon_max_level: self.request.weapon_max_level,
            actor,
            projection,
            contributions: self.request.additive_stats,
            set_count: self.request.set_count,
        })
    }
}

fn stat_key(key: &str) -> Option<&str> {
    Some(match key {
        "hp" | "atk" | "def" => key,
        "hp%" => "hp_percent",
        "atk%" => "atk_percent",
        "def%" => "def_percent",
        "cr" => "crit_rate",
        "cd" => "crit_damage_bonus",
        "dmg%" => "damage_bonus",
        "anima%" => "anima_bonus",
        "chaos%" => "chaos_bonus",
        "cosmos%" => "cosmos_bonus",
        "incantation%" => "incantation_bonus",
        "lakshana%" => "lakshana_bonus",
        "psyche%" => "psyche_bonus",
        "heal%" => "healing_bonus",
        "mental%" => "mental_damage_bonus",
        "shield%" => "shield_bonus",
        "ce" => "charge_efficiency",
        "ci" => "cycle_intensity",
        "bi" => "break_intensity",
        _ => return None,
    })
}

pub(super) fn stat_label(key: &str) -> &str {
    [
        "hp",
        "atk",
        "def",
        "hp%",
        "atk%",
        "def%",
        "cr",
        "cd",
        "dmg%",
        "anima%",
        "chaos%",
        "cosmos%",
        "incantation%",
        "lakshana%",
        "psyche%",
        "heal%",
        "mental%",
        "shield%",
        "ce",
        "ci",
        "bi",
    ]
    .into_iter()
    .find(|name| stat_key(name) == Some(key))
    .unwrap_or(key)
}
