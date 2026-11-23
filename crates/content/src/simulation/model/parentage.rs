use super::GameScenarioReport;

impl GameScenarioReport {
    /// Validate concrete damage-parent links without inferring a missing cast.
    pub fn validate_damage_parentage(&self) -> Result<(), String> {
        for index in 0..self.hit_receipts.len() {
            validate_hit_parent(&self.hit_receipts, index)?;
        }
        Ok(())
    }
}

pub(crate) fn validate_hit_parent(
    hits: &[super::GameHitReceipt],
    index: usize,
) -> Result<(), String> {
    let hit = &hits[index];
    let Some(parent_index) = hit.parent_hit_index else {
        return Ok(());
    };
    if parent_index >= index {
        return Err("Damage parent must precede its follow-up receipt.".into());
    }
    let parent = &hits[parent_index];
    if parent.time_us > hit.time_us
        || parent.actor_index != hit.actor_index
        || parent.target_index != hit.target_index
        || parent.command_index != hit.command_index
        || parent.autonomous_event_index != hit.autonomous_event_index
        || parent.switch_origin != hit.switch_origin
    {
        return Err("Damage follow-up differs from its parent identity.".into());
    }
    Ok(())
}
