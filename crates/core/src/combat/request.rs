use super::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealingTarget {
    pub current_hp: u64,
    pub max_hp: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    pub attacker_level: u32,
    pub attacker_stats: DynamicStats,
    pub target_defense: f64,
    pub target_resistance: f64,
    #[serde(default)]
    pub target_vulnerability: f64,
    #[serde(default)]
    pub buffs: Vec<Buff>,
    pub direct_hits: Vec<DirectHitSpec>,
    #[serde(default)]
    pub healings: Vec<HealingSpec>,
    #[serde(default)]
    pub healing_target: Option<HealingTarget>,
    #[serde(default)]
    pub shields: Vec<ShieldSpec>,
    #[serde(default)]
    pub negative_resistance_model: NegativeResistanceModel,
    #[serde(default = "default_rounding")]
    pub rounding: Rounding,
}

pub fn evaluate_request(request: &Request) -> Result<Value, String> {
    if request.schema_version != 1 {
        return Err(format!(
            "unsupported schema_version: {}",
            request.schema_version
        ));
    }
    if request.content_id != "ntesim-current" {
        return Err(format!("unsupported content_id: {}", request.content_id));
    }
    if request.attacker_level == 0 || request.attacker_level > 120 {
        return Err(format!(
            "attacker_level out of bounds: {}",
            request.attacker_level
        ));
    }

    let (current_stats, buffs_applied) =
        modifiers::project_active_buffs(&request.attacker_stats, &request.buffs)
            .map_err(|error| format!("buff projection: {error}"))?;

    let mut direct_reports = Vec::new();
    for hit in &request.direct_hits {
        let eval_req = DirectDamageEvaluationRequest {
            attacker_level: request.attacker_level,
            attacker_stats: &current_stats,
            target_defense: request.target_defense,
            target_resistance: request.target_resistance,
            target_vulnerability: request.target_vulnerability,
            hit,
            crit_mode: EvaluationCritMode::Expected,
            rounding: request.rounding,
            negative_res_model: request.negative_resistance_model,
        };
        let breakdown =
            evaluate_direct_damage(&eval_req).map_err(|e| format!("direct hit: {e}"))?;
        direct_reports.push(breakdown);
    }

    let mut healing_reports = Vec::new();
    if !request.healings.is_empty() && request.healing_target.is_none() {
        return Err("healing_target is required for healing evaluation".into());
    }
    let mut healing_target = request.healing_target.clone();
    for heal in &request.healings {
        let target = healing_target.as_mut().ok_or("healing_target missing")?;
        let breakdown = evaluate_healing(&current_stats, target.current_hp, target.max_hp, heal)
            .map_err(|e| format!("healing: {e}"))?;
        target.current_hp += breakdown.effective_healing;
        healing_reports.push(breakdown);
    }

    let mut shield_reports = Vec::new();
    for shld in &request.shields {
        let breakdown =
            evaluate_shield(&current_stats, shld).map_err(|e| format!("shield: {e}"))?;
        shield_reports.push(breakdown);
    }

    Ok(json!({
        "schema_version": 1,
        "kind": "combat-formula-evaluation-v1",
        "content_id": request.content_id,
        "attacker_level": request.attacker_level,
        "attacker_stats_initial": request.attacker_stats,
        "attacker_stats_final": current_stats,
        "buffs_requested": request.buffs.len(),
        "buffs_applied": buffs_applied,
        "direct_hits": direct_reports,
        "healings": healing_reports,
        "healing_target_final": healing_target,
        "shields": shield_reports,
        "quality": "incomplete",
        "combat_adapter_complete": false,
        "game_dps_computed": false
    }))
}
