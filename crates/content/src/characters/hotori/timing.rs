pub const MAX_TIME_VALUE: u64 = 120;
pub const REGEN_PER_SEC: u64 = 6;
pub const CONSUME_PER_SEC: u64 = 10;
pub const SKILL_COST: u64 = 60;
pub const ULTRA_SKILL_COST: u64 = 120;
pub const COPY_EXTRA_COOLDOWN_US: u64 = 35_000_000;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TimeModel {
    pub max_time_value: u64,
    pub regen_per_sec: u64,
    pub consume_per_sec: u64,
    pub skill_cost: u64,
    pub ultra_skill_cost: u64,
}

impl Default for TimeModel {
    fn default() -> Self {
        Self {
            max_time_value: MAX_TIME_VALUE,
            regen_per_sec: REGEN_PER_SEC,
            consume_per_sec: CONSUME_PER_SEC,
            skill_cost: SKILL_COST,
            ultra_skill_cost: ULTRA_SKILL_COST,
        }
    }
}

impl TimeModel {
    /// Native bytecode distinguishes the real player from a clone/copy.
    pub fn additional_skill_cooldown_us(is_real_player: Option<bool>) -> Result<u64, &'static str> {
        match is_real_player {
            Some(true) => Ok(0),
            Some(false) => Ok(COPY_EXTRA_COOLDOWN_US),
            None => Err("unsupported blueprint: is_real_player identity missing"),
        }
    }
}
