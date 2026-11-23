use super::*;

fn validate_build_identity(
    actor: &crate::simulation::GameActorConfig,
    report: &Value,
) -> Result<(), String> {
    if report["character_id"] != actor.character_id
        || report["weapon_id"] != actor.weapon_id
        || report["weapon_tier"] != actor.weapon_tier
        || report["character_level"] != actor.character_level
        || actor.set_id.as_ref().is_none_or(|s| report["set_id"] != *s)
    {
        return Err("build/scenario character, level, weapon, tier or set mismatch".into());
    }
    Ok(())
}

fn project_awakening_hp(
    actor: &mut crate::simulation::GameActorConfig,
    stats: &mut GameActorStaticStats,
    owner_hp_bonus: Ratio,
    original_hp: u64,
    original_hp_max: u64,
) -> Result<Value, String> {
    let base_hp = stats
        .base_hp
        .ok_or("Edgar A5 build requires base-HP provenance")?;
    let final_hp = crate::characters::edgar::awakenings::project_max_hp(
        base_hp,
        ratio(
            ExactFraction::from_ratio(stats.hp_percent)?
                .add(ExactFraction::from_ratio(owner_hp_bonus)?)?,
        )?,
        stats.hp_flat,
    )?;
    stats.hp = final_hp;
    actor.initial_hp_max = final_hp;
    actor.initial_hp = if original_hp == original_hp_max {
        final_hp
    } else {
        original_hp
    };
    Ok(json!({
        "awakening": actor.awakening,
        "base_hp": base_hp,
        "hp_percent_excluding_edgar_a5": stats.hp_percent,
        "hp_flat": stats.hp_flat,
        "a5_hp_percent": owner_hp_bonus,
        "hp_before": original_hp_max,
        "hp_after": final_hp,
        "source": "1021AwakenEffect.json Effect5 / DT_EdgarEffectFigure Buff_Edgar_Star5_HPMaxUP",
        "settlement": "native build base HP, HP percent and flat contributions plus Edgar A5 in one additive percent settlement, rounded once to nearest integer, ties upward"
    }))
}

fn project_static_weapon_hp(
    tables: &BTreeMap<String, Value>,
    request: &Request,
    actor: &mut crate::simulation::GameActorConfig,
    stats: &mut GameActorStaticStats,
    report: &Value,
) -> Result<(Option<Value>, Option<Value>), String> {
    let owner_hp_bonus = crate::characters::edgar::awakenings::owned_max_hp_bonus(
        &actor.character_id,
        &actor.awakening,
    );
    let mut awakening_hp_receipt = None;
    let original_hp = actor.initial_hp;
    let full_hp = original_hp == actor.initial_hp_max;
    let static_hp_weapon = if request.gear_projection.as_ref().is_some_and(|projection| {
        matches!(projection.policy, GearProjectionPolicy::NativeWeaponCurveV1)
    }) {
        super::weapon_projection::compile_static_hp_weapon(
            tables,
            request.schema_version,
            &request.scenario.content_id,
            &actor.character_id,
            &actor.weapon_id,
            actor.weapon_tier,
        )?
    } else {
        None
    };
    let Some(mut weapon) = static_hp_weapon else {
        return Ok((None, None));
    };
    let bonus: Ratio =
        serde_json::from_value(weapon["hp_bonus"].clone()).map_err(|e| e.to_string())?;
    let total_bonus =
        ExactFraction::from_ratio(bonus)?.add(ExactFraction::from_ratio(owner_hp_bonus)?)?;
    let base_hp = receipt(&report["known_attributes"]["HPMaxBase"]["receipt"])?;
    let final_hp = receipt(&report["known_derived_stats"]["hp_receipt"])?
        .add(base_hp.mul(total_bonus)?)?
        .round_u64()?;
    stats.hp_percent = ratio(
        ExactFraction::from_ratio(stats.hp_percent)?.add(ExactFraction::from_ratio(bonus)?)?,
    )?;
    weapon["hp_before"] = json!(stats.hp);
    weapon["hp_after"] = json!(final_hp);
    stats.hp = final_hp;
    actor.initial_hp_max = final_hp;
    actor.initial_hp = if full_hp { final_hp } else { original_hp };
    if owner_hp_bonus.numerator != 0 {
        awakening_hp_receipt = Some(
            json!({"awakening":actor.awakening,"a5_hp_percent":owner_hp_bonus,"hp_after":final_hp,"settlement":"base HP percent contributions settle together once"}),
        );
    }
    Ok((Some(weapon), awakening_hp_receipt))
}

fn project_build_skills(
    tables: &BTreeMap<String, Value>,
    actor: &crate::simulation::GameActorConfig,
    actor_build: &ActorBuild,
    report: &Value,
    skill_policy: Option<&ActionProjectionPolicy>,
) -> Result<Option<BTreeMap<String, Value>>, String> {
    if matches!(
        skill_policy,
        Some(
            ActionProjectionPolicy::NativeMintSkillLevelV1
                | ActionProjectionPolicy::NativeSkillLevelV1
        )
    ) {
        if matches!(
            skill_policy,
            Some(ActionProjectionPolicy::NativeMintSkillLevelV1)
        ) && actor.character_id != "character:1019"
        {
            return Err("effective skill projection currently supports Mint only".into());
        }
        let levels = actor_build
            .purchased_skill_levels
            .as_ref()
            .ok_or("effective skill projection requires purchased_skill_levels for each build")?;
        if levels.is_empty() || levels.len() > 4 {
            if matches!(
                skill_policy,
                Some(ActionProjectionPolicy::NativeMintSkillLevelV1)
            ) {
                return Err(
                    "Mint purchased_skill_levels requires 1..=4 configured abilities".into(),
                );
            } else {
                return Err("purchased_skill_levels requires 1..=4 configured abilities".into());
            }
        }
        let mut canonical = BTreeMap::new();
        for (ability, level) in levels {
            let breakthrough = report["character_breakthrough"]
                .as_u64()
                .ok_or("compiled breakthrough missing")?
                .try_into()
                .map_err(|_| "breakthrough overflow")?;
            let skill_receipt = super::preview_projection::effective_skill_level(
                tables,
                &actor.character_id,
                ability,
                *level,
                breakthrough,
                &actor.awakening,
            )?;
            let ability = skill_receipt["ability_id"]
                .as_str()
                .ok_or("canonical skill ability missing")?
                .to_owned();
            if canonical.insert(ability, skill_receipt).is_some() {
                return Err("duplicate canonical purchased skill ability".into());
            }
        }
        return Ok(Some(canonical));
    } else if actor_build.purchased_skill_levels.is_some() {
        return Err(
            "purchased_skill_levels requires native_mint_skill_level_v1 action projection".into(),
        );
    }
    Ok(None)
}

pub fn compile(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    request: &Request,
    limits: &ExecutionLimits,
) -> Result<Compiled, String> {
    if request.schema_version != 1 {
        return Err("build projection requires schema 1".into());
    }
    if request.hypothesis.trim().is_empty() {
        return Err("static build projection requires an explicit hypothesis".into());
    }
    if request.scenario.actions.len() as u64 > limits.max_events {
        return Err("build scenario action count exceeds execution event budget".into());
    }
    if request.scenario.actors.is_empty()
        || request.scenario.actors.len() > 4
        || request.builds.len() != request.scenario.actors.len()
    {
        return Err("build projection requires exactly one build for each of 1..4 actors".into());
    }
    let builds = request
        .builds
        .iter()
        .map(|b| (b.slot, &b.build))
        .collect::<BTreeMap<_, _>>();
    if builds.len() != request.builds.len() {
        return Err("duplicate build slot".into());
    }
    let mut scenario = request.scenario.clone();
    for actor in &scenario.actors {
        crate::registry::validate_awakening_selection(&actor.character_id, &actor.awakening)?;
    }
    assumptions::attach(&mut scenario, request)?;
    let slots = scenario
        .actors
        .iter()
        .map(|a| a.slot)
        .collect::<BTreeSet<_>>();
    let characters = scenario
        .actors
        .iter()
        .map(|a| &a.character_id)
        .collect::<BTreeSet<_>>();
    if slots.len() != scenario.actors.len() || characters.len() != scenario.actors.len() {
        return Err("duplicate actor slot or character".into());
    }
    let mut receipts = Vec::new();
    let mut skill_receipts = BTreeMap::new();
    for actor in &mut scenario.actors {
        let build = builds
            .get(&actor.slot)
            .ok_or("missing build for actor slot")?;
        if build.content_id != scenario.content_id {
            return Err("build/scenario package mismatch".into());
        }
        let released = coverage["entities"]
            .as_array()
            .ok_or("coverage entities missing")?
            .iter()
            .any(|e| e["id"] == actor.character_id && e["availability"] == "released");
        if !released {
            return Err("build actor is not in the released character inventory".into());
        }
        let report = equipment::compile(tables, build)?;
        let actor_build = request
            .builds
            .iter()
            .find(|b| b.slot == actor.slot)
            .unwrap();
        if let Some(levels) = project_build_skills(
            tables,
            actor,
            actor_build,
            &report,
            request.action_projection.as_ref().map(|p| &p.policy),
        )? {
            skill_receipts.insert(actor.slot, levels);
        }
        validate_build_identity(actor, &report)?;
        let native_element = ntesim_data::character::definition(&actor.character_id)?
            .element
            .native_key();
        let property = element_attribute(native_element)?;
        let mut stats = project(&report, property)?;
        if actor.initial_hp_max != stats.hp || actor.initial_hp > stats.hp {
            return Err(format!(
                "initial vitals disagree with compiled build HP: expected {}, got {}",
                stats.hp, actor.initial_hp_max
            ));
        }
        let original_hp = actor.initial_hp;
        let original_hp_max = actor.initial_hp_max;
        let full_hp = original_hp == original_hp_max;
        let owner_hp_bonus = crate::characters::edgar::awakenings::owned_max_hp_bonus(
            &actor.character_id,
            &actor.awakening,
        );
        let (static_hp_weapon, mut awakening_hp_receipt) =
            project_static_weapon_hp(tables, request, actor, &mut stats, &report)?;
        let static_gear_receipt = if static_hp_weapon.is_some() {
            static_hp_weapon
        } else if request.gear_projection.as_ref().is_some_and(|projection| {
            matches!(projection.policy, GearProjectionPolicy::NativeWeaponCurveV1)
                && actor.weapon_id.eq_ignore_ascii_case("weapon:fork_Arachne")
        }) {
            let native = crate::weapons::eternal_waltz::projection::compile_native(
                tables,
                request.schema_version,
                &scenario.content_id,
                actor,
            )?;
            let parameters = native["parameters"]
                .as_array()
                .ok_or("Eternal Waltz parameters missing")?;
            let hp_bonus = receipt(&parameters[0]["selected"]["value"])?;
            let base_hp = receipt(&report["known_attributes"]["HPMaxBase"]["receipt"])?;
            let current_hp_exact = receipt(&report["known_derived_stats"]["hp_receipt"])?;
            let compiled_hp = crate::weapons::eternal_waltz::projection::add_base_hp_ratio(
                current_hp_exact,
                base_hp,
                hp_bonus,
            )?;
            let compiled_hp = crate::weapons::eternal_waltz::projection::add_base_hp_ratio(
                compiled_hp,
                base_hp,
                ExactFraction::from_ratio(owner_hp_bonus)?,
            )?;
            let final_hp = compiled_hp.round_u64()?;
            stats.hp_percent = ratio(ExactFraction::from_ratio(stats.hp_percent)?.add(hp_bonus)?)?;
            let current_hp_after = if full_hp { final_hp } else { original_hp };
            if owner_hp_bonus.numerator != 0 {
                awakening_hp_receipt = Some(json!({
                    "awakening": actor.awakening,
                    "base_hp": {"numerator":base_hp.numerator.to_string(),"denominator":base_hp.denominator.to_string()},
                    "hp_percent_excluding_edgar_a5": stats.hp_percent,
                    "hp_flat": stats.hp_flat,
                    "a5_hp_percent": owner_hp_bonus,
                    "hp_before": original_hp_max,
                    "hp_after": final_hp,
                    "source": "1021AwakenEffect.json Effect5 / DT_EdgarEffectFigure Buff_Edgar_Star5_HPMaxUP",
                    "settlement": "native build HP plus base HP times Arachne HP ratio and Edgar A5 HP ratio, rounded once to nearest integer, ties upward"
                }));
            }
            let receipt = json!({
                "weapon_id": actor.weapon_id,
                "refinement": actor.weapon_tier,
                "native_parameter": parameters[0],
                "base_hp": {"numerator":base_hp.numerator.to_string(),"denominator":base_hp.denominator.to_string()},
                "compiled_hp_before_settlement": {"numerator":compiled_hp.numerator.to_string(),
                    "denominator":compiled_hp.denominator.to_string()},
                "hp_before": stats.hp,
                "hp_after": final_hp,
                "current_hp_before": original_hp,
                "current_hp_after": current_hp_after,
                "current_hp_policy": if full_hp {"full build vitals remain full"} else {"explicit current HP is preserved"},
                "settlement": "native derived HP plus base HP times the Arachne HP ratio, rounded once to nearest integer, ties upward"
            });
            stats.hp = final_hp;
            actor.initial_hp_max = final_hp;
            actor.initial_hp = current_hp_after;
            Some(receipt)
        } else if request.gear_projection.as_ref().is_some_and(|projection| {
            matches!(projection.policy, GearProjectionPolicy::NativeWeaponCurveV1)
                && actor
                    .weapon_id
                    .eq_ignore_ascii_case("weapon:fork_koinobori")
        }) {
            let distinct_elements = request
                .scenario
                .actors
                .iter()
                .map(|a| {
                    Ok(ntesim_data::character::definition(&a.character_id)?
                        .element
                        .key())
                })
                .collect::<Result<BTreeSet<_>, String>>()?;
            if distinct_elements.len() >= 3 {
                let native = crate::weapons::koinobori::projection::compile_native(
                    tables,
                    request.schema_version,
                    &scenario.content_id,
                    actor,
                )?;
                let parameters = native["parameters"]
                    .as_array()
                    .ok_or("A Time Will Come parameters missing")?;
                let hp_param = parameters
                    .iter()
                    .find(|p| p["name_id"].as_str() == Some("buff_koinobori_HpMaxUp"))
                    .ok_or("buff_koinobori_HpMaxUp parameter missing")?;
                let hp_bonus = receipt(&hp_param["selected"]["value"])?;
                let base_hp = receipt(&report["known_attributes"]["HPMaxBase"]["receipt"])?;
                let current_hp_exact = receipt(&report["known_derived_stats"]["hp_receipt"])?;
                let compiled_hp = crate::weapons::koinobori::projection::add_base_hp_ratio(
                    current_hp_exact,
                    base_hp,
                    hp_bonus,
                )?;
                let compiled_hp = crate::weapons::koinobori::projection::add_base_hp_ratio(
                    compiled_hp,
                    base_hp,
                    ExactFraction::from_ratio(owner_hp_bonus)?,
                )?;
                let final_hp = compiled_hp.round_u64()?;
                stats.hp_percent =
                    ratio(ExactFraction::from_ratio(stats.hp_percent)?.add(hp_bonus)?)?;
                let current_hp_after = if full_hp { final_hp } else { original_hp };
                if owner_hp_bonus.numerator != 0 {
                    awakening_hp_receipt = Some(json!({
                        "awakening": actor.awakening,
                        "base_hp": {"numerator":base_hp.numerator.to_string(),"denominator":base_hp.denominator.to_string()},
                        "hp_percent_excluding_edgar_a5": stats.hp_percent,
                        "hp_flat": stats.hp_flat,
                        "a5_hp_percent": owner_hp_bonus,
                        "hp_before": original_hp_max,
                        "hp_after": final_hp,
                        "source": "1021AwakenEffect.json Effect5 / DT_EdgarEffectFigure Buff_Edgar_Star5_HPMaxUP",
                        "settlement": "native build HP plus base HP times Koinobori team HP ratio and Edgar A5 HP ratio, rounded once to nearest integer, ties upward"
                    }));
                }
                let receipt = json!({
                    "weapon_id": actor.weapon_id,
                    "refinement": actor.weapon_tier,
                    "native_parameter": hp_param,
                    "base_hp": {"numerator":base_hp.numerator.to_string(),"denominator":base_hp.denominator.to_string()},
                    "compiled_hp_before_settlement": {"numerator":compiled_hp.numerator.to_string(),
                        "denominator":compiled_hp.denominator.to_string()},
                    "hp_before": stats.hp,
                    "hp_after": final_hp,
                    "current_hp_before": original_hp,
                    "current_hp_after": current_hp_after,
                    "current_hp_policy": if full_hp {"full build vitals remain full"} else {"explicit current HP is preserved"},
                    "settlement": "native derived HP plus base HP times the koinobori HP ratio when team has at least 3 distinct elements, rounded once to nearest integer, ties upward"
                });
                stats.hp = final_hp;
                actor.initial_hp_max = final_hp;
                actor.initial_hp = current_hp_after;
                Some(receipt)
            } else {
                None
            }
        } else {
            None
        };
        if owner_hp_bonus.numerator != 0 && static_gear_receipt.is_none() {
            awakening_hp_receipt = Some(project_awakening_hp(
                actor,
                &mut stats,
                owner_hp_bonus,
                original_hp,
                original_hp_max,
            )?);
        }
        stats.hp_percent = ratio(
            ExactFraction::from_ratio(stats.hp_percent)?
                .add(ExactFraction::from_ratio(owner_hp_bonus)?)?,
        )?;
        let charge_efficiency = super::stats::project_equipped_charge_efficiency(
            tables,
            &scenario.content_id,
            actor,
            &report,
        )?;
        let set_proc_receipt =
            crate::sets::quiet_manor::projection::project_equipped(tables, actor, &report)?;
        let hp_suit_receipt =
            crate::sets::hp_suit::projection::project_equipped(tables, actor, &report)?;
        let crimson_receipt =
            crate::sets::crimson::projection::project_equipped(tables, actor, &report)?;
        let devils_blood_receipt =
            crate::sets::devils_blood::projection::project_equipped(tables, actor, &report)?;
        let diabolos_receipt =
            crate::sets::diabolos::projection::project_equipped(tables, actor, &report)?;
        let street_boxer_receipt =
            crate::sets::street_boxer::projection::project_equipped(tables, actor, &report)?;
        let fireflies_receipt = crate::sets::fireflies::projection::project_equipped(
            tables,
            actor,
            &report,
            scenario.absolute_limit_us,
            request.gear_projection.as_ref(),
        )?;
        receipts.push(json!({
            "slot": actor.slot, "native_element": native_element,
            "element_bonus_attribute": property,
            "replaced_static_stats": actor.static_stats,
            "charge_efficiency_projection": charge_efficiency,
            "static_gear_projection": static_gear_receipt,
            "awakening_hp_projection": awakening_hp_receipt,
            "native_set_projection": set_proc_receipt,
            "native_hp_suit_projection": hp_suit_receipt,
            "native_crimson_projection": crimson_receipt,
            "native_devils_blood_projection": devils_blood_receipt,
            "native_diabolos_projection": diabolos_receipt,
            "native_street_boxer_projection": street_boxer_receipt,
            "native_fireflies_projection": fireflies_receipt,
            "compiled_static_stats": stats, "build_report": report
        }));
        actor.static_stats = stats;
    }
    let action_receipts = request
        .action_projection
        .as_ref()
        .map(|projection| {
            project_actions_with_levels(tables, &mut scenario, projection, &skill_receipts)
        })
        .transpose()?;
    let gear_receipts = request
        .gear_projection
        .as_ref()
        .map(|projection| {
            let mut receipts = project_gear(tables, &mut scenario, projection)?;
            if matches!(
                projection.policy,
                GearProjectionPolicy::NativeMindRoyaleFirefliesSoloV1
            ) {
                if request.action_projection.is_none() {
                    return Err(
                        "Fireflies native projection requires native action projection".into(),
                    );
                }
                receipts.push(crate::sets::fireflies::projection::project(
                    tables,
                    &mut scenario,
                    &request.builds,
                )?);
            }
            Ok::<_, String>(receipts)
        })
        .transpose()?;
    let receipt = json!({
        "kind": "compiled-game-research-v1", "schema_version": 1,
        "content_id": scenario.content_id, "status": "INCOMPLETE", "quality": "incomplete",
        "game_dps_computed": false, "acceptance_ready": false,
        "projection_policy": request.projection_policy, "hypothesis": request.hypothesis,
        "actor_build_receipts": receipts, "compiled_scenario": scenario,
        "action_projection": request.action_projection, "action_receipts": action_receipts,
        "skill_level_receipts": skill_receipts,
        "gear_projection": request.gear_projection, "gear_receipts": gear_receipts,
        "blockers": ["Native static attribute application and passive execution remain unresolved",
            "Caller actions, timings, procs and all-hit native element selection remain partial hypotheses",
            "Complete recommended published builds and adapters remain unavailable"]
    });
    Ok(Compiled { scenario, receipt })
}

pub fn run(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    request: &Request,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    let mut compiled = compile(tables, coverage, request, limits)?;
    compiled.receipt["kind"] = json!("built-game-research-v1");
    compiled.receipt["execution"] = run_game_scenario(&compiled.scenario, limits)?;
    Ok(compiled.receipt)
}
