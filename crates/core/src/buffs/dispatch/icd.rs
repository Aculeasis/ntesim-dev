use super::*;

pub const STANDARD_ICD_WINDOW_US: u64 = 2_500_000;

pub const STANDARD_ICD_HIT_INTERVAL: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "rule", rename_all = "snake_case")]
pub enum IcdRule {
    None,
    TimerOnly { window_us: u64 },
    Standard3Hit2_5s,
    HitCountOnly { hit_interval: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(
    deny_unknown_fields,
    tag = "scope",
    content = "id",
    rename_all = "snake_case"
)]
pub enum IcdScope {
    Global,
    PerTarget(String),
    SharedGroup(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Default)]
pub struct IcdEntry {
    last_event_us: Option<u64>,
    last_trigger_us: Option<u64>,
    window_start_us: Option<u64>,
    pub(super) hits_since_trigger: u32,
}

impl IcdEntry {
    pub fn can_trigger(&self, rule: &IcdRule, now_us: u64) -> Result<bool, DispatchError> {
        self.clone().record_event(rule, now_us)
    }

    pub fn record_event(&mut self, rule: &IcdRule, now_us: u64) -> Result<bool, DispatchError> {
        if matches!(rule, IcdRule::HitCountOnly { hit_interval: 0 }) {
            return Err(DispatchError::InvalidIcd("zero hit interval".into()));
        }
        if let Some(last_us) = self.last_event_us
            && now_us < last_us
        {
            return Err(DispatchError::TimeReversal { last_us, now_us });
        }
        self.last_event_us = Some(now_us);
        Ok(match rule {
            IcdRule::None => {
                self.last_trigger_us = Some(now_us);
                true
            }
            IcdRule::TimerOnly { window_us } => {
                if self.last_trigger_us.is_some_and(|last| {
                    now_us
                        .checked_sub(last)
                        .is_none_or(|elapsed| elapsed < *window_us)
                }) {
                    return Ok(false);
                }
                self.last_trigger_us = Some(now_us);
                true
            }
            IcdRule::Standard3Hit2_5s => match self.window_start_us {
                None => {
                    self.window_start_us = Some(now_us);
                    self.last_trigger_us = Some(now_us);
                    self.hits_since_trigger = 1;
                    true
                }
                Some(start) => {
                    self.hits_since_trigger = self.hits_since_trigger.saturating_add(1);
                    let time_elapsed = now_us
                        .checked_sub(start)
                        .is_some_and(|elapsed| elapsed >= STANDARD_ICD_WINDOW_US);
                    let hits_elapsed = self.hits_since_trigger >= STANDARD_ICD_HIT_INTERVAL;

                    if time_elapsed || hits_elapsed {
                        self.window_start_us = Some(now_us);
                        self.last_trigger_us = Some(now_us);
                        self.hits_since_trigger = 1;
                        true
                    } else {
                        false
                    }
                }
            },
            IcdRule::HitCountOnly { hit_interval } => {
                self.hits_since_trigger = self.hits_since_trigger.saturating_add(1);
                if self.hits_since_trigger >= *hit_interval {
                    self.hits_since_trigger = 0;
                    self.last_trigger_us = Some(now_us);
                    true
                } else {
                    false
                }
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Default)]
pub struct IcdTracker {
    entries: BTreeMap<String, IcdEntry>,
}

impl IcdTracker {
    pub(crate) fn key(callback_id: &str, scope: &IcdScope) -> String {
        match scope {
            IcdScope::Global => format!("{callback_id}:g"),
            IcdScope::PerTarget(target) => format!("{callback_id}:t:{target}"),
            IcdScope::SharedGroup(group) => format!(":s:{group}"),
        }
    }

    pub fn can_trigger(
        &self,
        callback_id: &str,
        scope: &IcdScope,
        rule: &IcdRule,
        now_us: u64,
    ) -> Result<bool, DispatchError> {
        let k = Self::key(callback_id, scope);
        match self.entries.get(&k) {
            None => IcdEntry::default().can_trigger(rule, now_us),
            Some(entry) => entry.can_trigger(rule, now_us),
        }
    }

    pub fn record_event(
        &mut self,
        callback_id: &str,
        scope: &IcdScope,
        rule: &IcdRule,
        now_us: u64,
    ) -> Result<bool, DispatchError> {
        let k = Self::key(callback_id, scope);
        let mut entry = self.entries.get(&k).cloned().unwrap_or_default();
        let triggered = entry.record_event(rule, now_us)?;
        self.entries.insert(k, entry);
        Ok(triggered)
    }
}
