//! Explicit shared-clock and character-clock probes; no inferred game timing.
//! Ordinary hit/recovery/switch timing, real vs game-mode vs actor clocks,
//! mode pause intervals (union of overlapping cutscene/burst stops),
//! hitlag dilation, action deadline conversion, and Hotori clock/cooldown branches.

use crate::formula::Ratio;
use serde::{Deserialize, Serialize};

pub use ntesim_content::characters::hotori::timing::TimeModel as HotoriTimeModel;
use ntesim_core::timing::*;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduledActionInput {
    pub actor_index: usize,
    pub start_real_us: u64,
    pub spec: ActionTimingSpec,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HotoriProbeInput {
    pub is_real_player: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TimingScenarioRequest {
    pub content_id: String,
    pub team_size: usize,
    pub horizon_real_us: u64,
    pub mode_pauses: Vec<ModePauseInterval>,
    pub hitlags: Vec<HitlagWindow>,
    pub actions: Vec<ScheduledActionInput>,
    pub hotori_probe: Option<HotoriProbeInput>,
}

pub fn evaluate_timing_scenario(
    request: &TimingScenarioRequest,
) -> Result<serde_json::Value, String> {
    if request.content_id != "ntesim-current" {
        return Err(format!("unexpected package ID {}", request.content_id));
    }
    if request.team_size == 0 || request.team_size > 4 {
        return Err(format!("invalid team size {}", request.team_size));
    }
    if request.horizon_real_us == 0 {
        return Err("horizon_real_us must be greater than zero".to_string());
    }

    let mut mode_clock = ModeClockManager::new();
    for p in &request.mode_pauses {
        mode_clock
            .add_pause(p.clone())
            .map_err(|error| format!("invalid mode pause: {error}"))?;
    }

    let mut hitlag_tracker = HitlagTracker::default();
    for h in &request.hitlags {
        if h.actor_index >= request.team_size {
            return Err(format!("hitlag actor out of bounds: {}", h.actor_index));
        }
        hitlag_tracker
            .add_window(h.clone())
            .map_err(|error| format!("invalid hitlag window: {error}"))?;
    }

    let mut resolved_actions = Vec::new();
    for act in &request.actions {
        if act.actor_index >= request.team_size {
            return Err(format!("action actor out of bounds: {}", act.actor_index));
        }
        let res = resolve_action_deadlines(
            act.actor_index,
            act.start_real_us,
            &act.spec,
            &hitlag_tracker,
            &mode_clock,
        )
        .map_err(|error| format!("failed to resolve action {}: {error}", act.spec.action_id))?;
        resolved_actions.push(res);
    }

    let mut actor_times_us = Vec::new();
    for i in 0..request.team_size {
        let t = hitlag_tracker
            .elapsed_actor_time_us(i, 0, request.horizon_real_us)
            .map_err(|error| format!("failed to resolve actor {i} clock: {error}"))?;
        actor_times_us.push(t);
    }

    let game_mode_horizon_us = mode_clock.real_to_mode_time(request.horizon_real_us);
    let total_paused_us = mode_clock.paused_time_up_to(request.horizon_real_us);

    let hotori_receipt = match &request.hotori_probe {
        None => None,
        Some(probe) => {
            let cd_us = HotoriTimeModel::additional_skill_cooldown_us(probe.is_real_player)
                .map_err(|e| e.to_string())?;
            Some(serde_json::json!({
                "model": HotoriTimeModel::default(),
                "is_real_player": probe.is_real_player,
                "additional_skill_cooldown_us": cd_us,
                "caveat": "A 35-second non-real-player extra cooldown must not be applied to an ordinary Hotori skill or interpreted as replay spacing.",
            }))
        }
    };

    Ok(serde_json::json!({
        "status": "ok",
        "engine": ntesim_contracts::ENGINE_VERSION,
        "engine_source_sha256": ntesim_contracts::ENGINE_SOURCE_SHA256,
        "content_id": request.content_id,
        "receipt": {
            "real_time_horizon_us": request.horizon_real_us,
            "game_mode_horizon_us": game_mode_horizon_us,
            "total_mode_paused_us": total_paused_us,
            "merged_mode_pauses": mode_clock.merged_intervals(),
            "actor_clocks_at_horizon_us": actor_times_us,
            "resolved_actions": resolved_actions,
            "hotori": hotori_receipt,
            "real_time_report": {
                "horizon_us": request.horizon_real_us,
                "horizon_s": Ratio { numerator: request.horizon_real_us, denominator: 1_000_000 },
            },
            "game_mode_report": {
                "horizon_us": game_mode_horizon_us,
                "horizon_s": Ratio { numerator: game_mode_horizon_us, denominator: 1_000_000 },
            }
        },
        "quality": {
            "timing_model_complete": true,
            "combat_adapter_complete": false,
            "game_dps_computed": false,
        }
    }))
}

#[cfg(test)]
mod tests;
