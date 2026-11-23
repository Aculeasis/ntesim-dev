use super::{GameActionCommand, GameScenarioRequest};
use crate::{execution::ExecutionLimits, rng::Rng};
mod plunge;

pub fn validate_game_request(
    request: &GameScenarioRequest,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    validate_and_resolve(request, limits).map(|_| ())
}

pub(crate) fn validate_party_size(count: usize) -> Result<(), String> {
    if count == 0 {
        return Err("team must contain at least one actor".into());
    }
    if count > 4 {
        return Err("team cannot exceed four actors".into());
    }
    Ok(())
}

pub(crate) fn validate_target_count(count: usize, max_work: u64) -> Result<(), String> {
    if count == 0 {
        return Err("scenario must contain at least one target".into());
    }
    if u64::try_from(count).map_err(|_| "target count overflow")? > max_work {
        return Err(format!(
            "target count {count} exceeds execution work limit {max_work}"
        ));
    }
    Ok(())
}

pub(super) fn validate_and_resolve(
    request: &GameScenarioRequest,
    limits: &ExecutionLimits,
) -> Result<Vec<Option<super::metadata::ActionPacketMetadata>>, String> {
    validate_request_options(request)?;
    if let Some(halt) = &request.source_halt {
        halt.validate(request)?;
    }
    super::model::GameTimeContext::validate_all(request)?;
    super::origin::Origins::validate(request)?;
    if let Some(profile) = &request.combat_stamina {
        profile.validate()?;
    }
    validate_model_assumptions(request)?;
    validate_incoming_damage_assumption(request)?;
    validate_stochastic_crit(request)?;
    validate_scenario_dimensions(request, limits)?;
    validate_incoming_damage_events(request)?;
    super::dummy::validate(request, limits)?;
    super::parry::validate(request)?;
    super::combat::validate(request)?;
    super::hp_cost::validate(request)?;
    super::interrupt::validate(request)?;
    validate_action_metadata(request)?;
    let packets = validate_actions(request)?;
    validate_actors(request, limits)?;
    validate_targets(request)?;
    super::incoming_defense::validate(request)?;
    plunge::validate_inputs(request, &packets)?;
    super::inputs::validate_dash_directions(request)?;
    validate_aim_targets(request)?;
    crate::registry::validate_action_inputs(request, &packets)?;
    Ok(packets)
}

fn validate_aim_targets(request: &GameScenarioRequest) -> Result<(), String> {
    for (&index, &target) in &request.aim_targets {
        if !matches!(
            request.actions.get(index),
            Some(
                GameActionCommand::InvokeAbility { .. }
                    | GameActionCommand::PlayerInput { .. }
                    | GameActionCommand::SwitchActor { .. }
            )
        ) {
            return Err(format!(
                "aim_targets index {index} must identify a combat input"
            ));
        }
        if target >= request.targets.len() {
            return Err(format!(
                "aim_targets index {index} selects unknown target {target}"
            ));
        }
    }
    Ok(())
}

fn validate_request_options(request: &GameScenarioRequest) -> Result<(), String> {
    if request.schema_version != 1 {
        return Err("schema_version must be 1".into());
    }
    if request.content_id != "ntesim-current" {
        return Err("unsupported content_id: expected ntesim-current".into());
    }
    if request.action_metadata.len() > 64 {
        return Err("action_metadata exceeds 64 declarations".into());
    }
    Ok(())
}

fn validate_incoming_damage_assumption(request: &GameScenarioRequest) -> Result<(), String> {
    if !request.incoming_damage_events.is_empty()
        && request
            .model_assumptions
            .get(super::events::INCOMING_DAMAGE_HYPOTHESIS_ID.as_str())
            .map(String::as_str)
            != Some(super::events::INCOMING_DAMAGE_HYPOTHESIS_ID.description())
    {
        return Err(format!(
            "incoming_damage_events require accepted model assumption {}",
            super::events::INCOMING_DAMAGE_HYPOTHESIS_ID
        ));
    }
    Ok(())
}

fn validate_stochastic_crit(request: &GameScenarioRequest) -> Result<(), String> {
    if let Some(policy) = &request.stochastic_crit {
        if policy.hypothesis.trim().is_empty() || policy.eligible_effects.is_empty() {
            return Err(
                "stochastic_crit requires a hypothesis and explicit effect eligibility".into(),
            );
        }
        if policy
            .eligible_effects
            .keys()
            .ne(policy.actor_sequences.keys())
        {
            return Err(
                "stochastic_crit actor sequences must exactly match eligible actor keys".into(),
            );
        }
        Rng::new(request.seed, policy.actor_sequences.clone())?;
        crate::gear::validate_rng_selectors(request)?;
        for (owner, effects) in &policy.eligible_effects {
            let matches: Vec<_> = request
                .actors
                .iter()
                .filter(|a| &a.character_id == owner)
                .collect();
            if matches.len() != 1
                || effects.is_empty()
                || effects.iter().any(|s| s.trim().is_empty())
            {
                return Err(
                    "stochastic_crit requires a unique configured actor and nonempty effects"
                        .into(),
                );
            }
            let rate = matches[0].static_stats.crit_rate;
            if rate.denominator == 0
                || rate.denominator > crate::rng::WORD_RANGE
                || rate.numerator > rate.denominator
            {
                return Err(
                    "stochastic crit rate must be in [0,1] with denominator in 1..=2^32".into(),
                );
            }
        }
    }
    Ok(())
}

fn validate_scenario_dimensions(
    request: &GameScenarioRequest,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    if request.source_exhausted
        && request.action_admission_order != super::GameActionAdmissionOrder::AfterSettledEvents
    {
        return Err("source_exhausted requires after_settled_events admission".into());
    }
    if request.relative_duration_us == 0 {
        return Err("relative_duration_us must be greater than zero".into());
    }
    if request.absolute_limit_us == 0 || request.absolute_limit_us > limits.max_horizon_us {
        return Err(
            "absolute_limit_us must be positive and within the caller's absolute ceiling".into(),
        );
    }
    validate_party_size(request.actors.len())?;
    if request.active_actor_index >= request.actors.len() {
        return Err(format!(
            "active_actor_index {} out of bounds for team size {}",
            request.active_actor_index,
            request.actors.len()
        ));
    }
    validate_target_count(request.targets.len(), limits.max_events)?;
    let requested_events = (request.actions.len() as u64)
        .checked_add(request.incoming_damage_events.len() as u64)
        .and_then(|count| count.checked_add(request.hp_cost_events.len() as u64))
        .and_then(|count| count.checked_add(request.interrupt_events.len() as u64))
        .and_then(|count| count.checked_add(request.tenacity_events.len() as u64))
        .and_then(|count| count.checked_add(request.parry_events.len() as u64))
        .and_then(|count| count.checked_add(request.combat_transition_events.len() as u64))
        .ok_or("scenario event count overflow")?;
    if requested_events > limits.max_events {
        return Err(format!(
            "scenario event count {requested_events} exceeds max limit {}",
            limits.max_events
        ));
    }
    Ok(())
}

fn validate_incoming_damage_events(request: &GameScenarioRequest) -> Result<(), String> {
    let mut previous_damage_time = 0;
    for (index, event) in request.incoming_damage_events.iter().enumerate() {
        if event.timestamp_us < previous_damage_time {
            return Err(format!(
                "incoming damage event {index} timestamp {} violates monotonicity (previous was {previous_damage_time})",
                event.timestamp_us
            ));
        }
        if event.timestamp_us > request.absolute_limit_us {
            return Err(format!(
                "incoming damage event {index} timestamp {} exceeds scenario horizon {}",
                event.timestamp_us, request.absolute_limit_us
            ));
        }
        if event.actor_index == usize::MAX {
            if request.actors.is_empty() {
                return Err(format!(
                    "incoming damage event {index} requires at least one actor"
                ));
            }
            for actor in &request.actors {
                for proc in &actor.gear_procs {
                    if let super::GearProcConfig::BitterCake {
                        duration_us,
                        cooldown_us,
                        ..
                    } = proc
                    {
                        event
                            .timestamp_us
                            .checked_add(*duration_us)
                            .and_then(|_| event.timestamp_us.checked_add(*cooldown_us))
                            .ok_or_else(|| {
                                format!(
                                    "incoming damage event {index} Bittercake deadline overflow"
                                )
                            })?;
                    }
                }
            }
            previous_damage_time = event.timestamp_us;
            continue;
        }
        let actor = request
            .actors
            .get(event.actor_index)
            .ok_or_else(|| format!("incoming damage event {index} actor index out of bounds"))?;
        for proc in &actor.gear_procs {
            if let super::GearProcConfig::BitterCake {
                duration_us,
                cooldown_us,
                ..
            } = proc
            {
                event
                    .timestamp_us
                    .checked_add(*duration_us)
                    .and_then(|_| event.timestamp_us.checked_add(*cooldown_us))
                    .ok_or_else(|| {
                        format!("incoming damage event {index} Bittercake deadline overflow")
                    })?;
            }
        }
        previous_damage_time = event.timestamp_us;
    }
    Ok(())
}

fn validate_action_metadata(request: &GameScenarioRequest) -> Result<(), String> {
    for (index, declaration) in &request.action_metadata {
        let Some(GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            ..
        }) = request.actions.get(*index)
        else {
            return Err(format!(
                "action_metadata index {index} must identify an InvokeAbility action"
            ));
        };
        if declaration.ability_id != *ability_id
            || declaration.damage_effect_id != *damage_effect_id
        {
            return Err(format!(
                "action_metadata identity at action {index} does not match configured ability/effect"
            ));
        }
    }
    Ok(())
}

fn validate_actions(
    request: &GameScenarioRequest,
) -> Result<Vec<Option<super::metadata::ActionPacketMetadata>>, String> {
    let mut prev_t = 0;
    let mut packets = vec![None; request.actions.len()];
    for (i, action) in request.actions.iter().enumerate() {
        let t = action.timestamp_us();
        if t < prev_t {
            return Err(format!(
                "action {i} timestamp {t} violates monotonicity (previous was {prev_t})"
            ));
        }
        if t > request.absolute_limit_us {
            return Err(format!(
                "action {i} timestamp {t} exceeds scenario horizon {}",
                request.relative_duration_us
            ));
        }
        prev_t = t;
        packets[i] = validate_action_contents(request, i, action, t)?;
    }
    Ok(packets)
}

fn validate_action_contents(
    request: &GameScenarioRequest,
    index: usize,
    action: &GameActionCommand,
    timestamp_us: u64,
) -> Result<Option<super::metadata::ActionPacketMetadata>, String> {
    match action {
        GameActionCommand::TargetGauge {
            target_index,
            gauge,
            ..
        } => {
            let target = request.targets.get(*target_index).ok_or_else(|| {
                format!("target gauge action {index} target index is out of bounds")
            })?;
            let profile = target.break_profile.as_ref().ok_or_else(|| {
                format!("target gauge action {index} requires an explicit Break gauge")
            })?;
            if super::ExactFraction::from_ratio(*gauge)?.is_zero() {
                timestamp_us
                    .checked_add(profile.broken_duration_us)
                    .ok_or("target Broken expiry overflow")?;
            }
            Ok(None)
        }
        GameActionCommand::PlayerInput {
            actor_index,
            input,
            critical_success,
            ..
        } => {
            validate_player_input_action(
                request,
                *actor_index,
                *input,
                *critical_success,
                timestamp_us,
            )?;
            Ok(None)
        }
        GameActionCommand::InvokeAbility {
            timestamp_us,
            actor_index,
            ability_id,
            damage_effect_id,
            atk_rate,
            scaling_stat,
            recovery_us,
            cooldown_us,
            hit_delay_us,
            ..
        } => {
            scaling_stat.validate()?;
            super::ExactFraction::from_ratio(*atk_rate)?;
            validate_invoke_ability_action(
                request,
                index,
                *timestamp_us,
                *actor_index,
                ability_id,
                damage_effect_id,
                *recovery_us,
                *cooldown_us,
                *hit_delay_us,
            )
        }
        GameActionCommand::SwitchActor {
            target_actor_index,
            cooldown_us,
            ..
        } => {
            if *target_actor_index >= request.actors.len() {
                return Err(format!(
                    "switch {index} target_actor_index {target_actor_index} out of bounds for team size {}",
                    request.actors.len()
                ));
            }
            timestamp_us
                .checked_add(*cooldown_us)
                .ok_or_else(|| format!("switch {index} deadline overflow"))?;
            Ok(None)
        }
        GameActionCommand::Wait { duration_us, .. } => {
            timestamp_us
                .checked_add(*duration_us)
                .ok_or_else(|| format!("wait {index} deadline overflow"))?;
            Ok(None)
        }
    }
}

fn validate_player_input_action(
    request: &GameScenarioRequest,
    actor_index: usize,
    input: crate::simulation::PlayerInput,
    critical_success: bool,
    timestamp_us: u64,
) -> Result<(), String> {
    let actor = request
        .actors
        .get(actor_index)
        .ok_or("action actor index out of bounds")?;
    if input == crate::simulation::PlayerInput::Arc {
        let arc = crate::gear::arc::Arc::project(actor)?;
        arc.validate_deadline(timestamp_us)?;
    } else {
        crate::characters::player_inputs::validate(&actor.character_id, input)?;
    }
    if critical_success && input != crate::simulation::PlayerInput::Dash {
        return Err("critical_success is only valid for dash inputs".into());
    }
    if critical_success
        && crate::registry::critical_dodge_model(request, actor).is_some()
        && request.combat_stamina.is_none()
    {
        return Err("owner Critical Dodge requires an explicit combat stamina profile".into());
    }
    timestamp_us
        .checked_add(super::inputs::recovery(input))
        .ok_or("player input deadline overflow")?;
    Ok(())
}

fn validate_invoke_ability_action(
    request: &GameScenarioRequest,
    index: usize,
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    recovery_us: u64,
    cooldown_us: u64,
    hit_delay_us: u64,
) -> Result<Option<super::metadata::ActionPacketMetadata>, String> {
    for duration in [recovery_us, cooldown_us, hit_delay_us] {
        timestamp_us
            .checked_add(duration)
            .ok_or_else(|| format!("action {index} deadline overflow"))?;
    }
    let actor = request
        .actors
        .get(actor_index)
        .ok_or("action actor index out of bounds")?;
    let duration = crate::registry::cast_buff_duration(actor, ability_id);
    if let Some(duration) = duration {
        timestamp_us
            .checked_add(duration)
            .ok_or_else(|| format!("action {index} configured kit buff deadline overflow"))?;
    }
    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &actor.character_id,
        ability_id,
        damage_effect_id,
        request.action_metadata.get(&index),
        &request.model_assumptions,
    )?;
    crate::gear::validate_action_deadlines(
        &actor.gear_procs,
        &metadata.action,
        timestamp_us,
        hit_delay_us,
        index,
    )?;
    Ok(Some(metadata))
}

fn validate_actors(request: &GameScenarioRequest, limits: &ExecutionLimits) -> Result<(), String> {
    for (i, a) in request.actors.iter().enumerate() {
        crate::registry::validate_awakening_selection(&a.character_id, &a.awakening)?;
        if a.purchased_qte_level.is_some() {
            super::talents::qte_coordinate(a).map_err(|error| format!("actor {i}: {error}"))?;
        }
        super::ExactFraction::from_ratio(a.static_stats.damage_bonus)?;
        super::ExactFraction::from_ratio(a.static_stats.healing_bonus)?;
        super::ExactFraction::from_ratio(a.static_stats.shield_bonus)?;
        super::ExactFraction::from_ratio(a.static_stats.break_intensity)?;
        super::ExactFraction::from_ratio(a.static_stats.break_efficiency_multiplier)?;
        super::ExactFraction::from_ratio(a.static_stats.mental_damage_bonus)?;
        super::ExactFraction::from_ratio(a.static_stats.hp_percent)?;
        super::ExactFraction::from_ratio(a.static_stats.hp_flat)?;
        super::state::initial_hp_projection(a)
            .map_err(|error| format!("actor {i} HP projection: {error}"))?;
        for (element, bonus) in &a.static_stats.elemental_bonuses {
            if !matches!(
                element.as_str(),
                "CHAOS" | "COSMOS" | "INCANTATION" | "LAKSHANA" | "NATURE" | "PSYCHE"
            ) {
                return Err(format!(
                    "actor {i} unsupported damage bonus element {element}"
                ));
            }
            super::ExactFraction::from_ratio(*bonus)?;
        }
        if a.initial_hp_max == 0 {
            return Err(format!("actor {i} initial_hp_max cannot be zero"));
        }
        if a.initial_hp > a.initial_hp_max {
            return Err(format!("actor {i} initial_hp exceeds initial_hp_max"));
        }
        if a.energy_capacity == 0 {
            return Err(format!("actor {i} energy_capacity cannot be zero"));
        }
        if a.initial_energy > a.energy_capacity {
            return Err(format!("actor {i} initial_energy exceeds energy_capacity"));
        }
        crate::registry::validate_initial_resources(
            &a.character_id,
            a.character_level,
            &a.awakening,
            &a.initial_resources,
        )
        .map_err(|error| format!("actor {i}: {error}"))?;
        for (id, description) in crate::registry::initial_resource_assumptions(&a.initial_resources)
        {
            if request.model_assumptions.get(&id) != Some(&description) {
                return Err(format!("actor {i} initial resources require model {id}"));
            }
        }
        if a.initial_cycle > crate::simulation::reactions::CYCLE_CAPACITY {
            return Err(format!(
                "actor {i} initial_cycle exceeds the nominal {} Cycle capacity",
                crate::simulation::reactions::CYCLE_CAPACITY
            ));
        }
        super::ExactFraction::from_ratio(a.cycle_gain_efficiency)?;
        if a.energy_capacity > limits.max_pool_capacity {
            return Err(format!("actor {i} energy capacity exceeds execution limit"));
        }
        crate::gear::validate_config(a, i, request.absolute_limit_us, limits)?;
    }
    Ok(())
}

fn validate_targets(request: &GameScenarioRequest) -> Result<(), String> {
    for (j, t) in request.targets.iter().enumerate() {
        if t.target_index != j {
            return Err(format!(
                "target index {} must match its contiguous position {j}",
                t.target_index
            ));
        }
        if t.hp_max == 0 {
            return Err(format!("target {j} hp_max cannot be zero"));
        }
        if t.hp_current > t.hp_max {
            return Err(format!("target {j} hp_current exceeds hp_max"));
        }
        for (element, resistance) in &t.elemental_resistances {
            if !matches!(
                element.as_str(),
                "NATURE" | "CHAOS" | "COSMOS" | "INCANTATION" | "LAKSHANA" | "PSYCHE"
            ) {
                return Err(format!(
                    "target {j} has unsupported resistance element '{element}'"
                ));
            }
            crate::simulation::ExactFraction::from_ratio(*resistance)
                .map_err(|error| format!("target {j} {element} resistance is invalid: {error}"))?;
        }
        crate::simulation::ExactFraction::from_ratio(t.resistance)
            .map_err(|error| format!("target {j} common resistance is invalid: {error}"))?;
        if let Some(profile) = &t.break_profile {
            validate_target_break_profile(profile, j)?;
        }
    }
    Ok(())
}

pub(crate) fn validate_target_break_profile(
    profile: &crate::simulation::GameTargetBreakProfileConfig,
    target_index: usize,
) -> Result<(), String> {
    if profile.broken_duration_us == 0 {
        return Err(format!(
            "target {target_index} break_profile broken_duration_us cannot be zero"
        ));
    }
    let capacity =
        crate::simulation::ExactFraction::from_ratio(profile.capacity).map_err(|error| {
            format!("target {target_index} break_profile capacity is invalid: {error}")
        })?;
    if capacity.numerator == 0 {
        return Err(format!(
            "target {target_index} break_profile capacity must be positive"
        ));
    }
    let gauge =
        crate::simulation::ExactFraction::from_ratio(profile.current_gauge).map_err(|error| {
            format!("target {target_index} break_profile current_gauge is invalid: {error}")
        })?;
    if gauge.numerator * capacity.denominator > capacity.numerator * gauge.denominator {
        return Err(format!(
            "target {target_index} break_profile current_gauge exceeds capacity"
        ));
    }
    if let Some(fraction) = profile.discord_fraction {
        let fraction = crate::simulation::ExactFraction::from_ratio(fraction).map_err(|error| {
            format!("target {target_index} break_profile discord_fraction is invalid: {error}")
        })?;
        if fraction.numerator == 0 || fraction.numerator > fraction.denominator {
            return Err(format!(
                "target {target_index} break_profile discord_fraction must be in (0, 1]"
            ));
        }
    }
    Ok(())
}

pub fn validate_model_assumptions(request: &GameScenarioRequest) -> Result<(), String> {
    validate_model_assumption_entries(request)
}

fn validate_model_assumption_entries(request: &GameScenarioRequest) -> Result<(), String> {
    // Canonical declarations are bounded by the finite engine registry. A
    // four-member party can legitimately project more than 32 owner models.
    if request.model_assumptions.len() > 32
        && request
            .model_assumptions
            .iter()
            .filter(|(id, description)| {
                ntesim_contracts::get_hypothesis_description(id) != Some(description.as_str())
            })
            .count()
            > 32
    {
        return Err("custom model_assumptions exceeds 32 entries".into());
    }
    for (id, description) in &request.model_assumptions {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_:-./".contains(&b))
            || description.trim().is_empty()
            || description.len() > 4096
        {
            return Err("model_assumptions requires a bounded ASCII identity and nonempty description of at most 4096 UTF-8 bytes".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod target_break_profile_tests {
    use super::validate_target_break_profile;
    use crate::formula::Ratio;
    use crate::simulation::GameTargetBreakProfileConfig;

    fn ratio(numerator: u64, denominator: u64) -> Ratio {
        Ratio {
            numerator,
            denominator,
        }
    }

    fn profile(
        capacity: Ratio,
        current_gauge: Ratio,
        broken_duration_us: u64,
    ) -> GameTargetBreakProfileConfig {
        GameTargetBreakProfileConfig {
            capacity,
            current_gauge,
            broken_duration_us,
            discord_fraction: None,
        }
    }

    #[test]
    fn accepts_exact_gauge_at_or_below_positive_capacity() {
        assert!(validate_target_break_profile(&profile(ratio(10, 3), ratio(5, 2), 1), 0).is_ok());
        assert!(validate_target_break_profile(&profile(ratio(3, 2), ratio(3, 2), 1), 0).is_ok());
    }

    #[test]
    fn rejects_nonpositive_capacity_zero_duration_and_gauge_above_capacity() {
        assert!(validate_target_break_profile(&profile(ratio(0, 1), ratio(0, 1), 1), 0).is_err());
        assert!(validate_target_break_profile(&profile(ratio(1, 1), ratio(0, 1), 0), 0).is_err());
        assert!(validate_target_break_profile(&profile(ratio(2, 3), ratio(3, 4), 1), 0).is_err());
        assert!(validate_target_break_profile(&profile(ratio(1, 0), ratio(0, 1), 1), 0).is_err());
    }

    #[test]
    fn discord_fraction_requires_an_exact_value_in_open_zero_to_one_range() {
        for fraction in [ratio(1, 1), ratio(1, 3)] {
            let mut profile = profile(ratio(1, 1), ratio(1, 1), 1);
            profile.discord_fraction = Some(fraction);
            assert!(validate_target_break_profile(&profile, 0).is_ok());
        }
        for fraction in [ratio(0, 1), ratio(2, 1), ratio(1, 0)] {
            let mut profile = profile(ratio(1, 1), ratio(1, 1), 1);
            profile.discord_fraction = Some(fraction);
            assert!(validate_target_break_profile(&profile, 0).is_err());
        }
    }
}
