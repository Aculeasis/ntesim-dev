//! Shared character caps; costs, rewards and audio have no combat consumer.
pub const LEVEL_CAPS: [u32; 7] = [20, 30, 40, 50, 60, 70, 80];
pub fn max_level(stage: u32) -> Result<u32, String> {
    LEVEL_CAPS
        .get(stage as usize)
        .copied()
        .ok_or_else(|| "character breakthrough out of supported range".into())
}
