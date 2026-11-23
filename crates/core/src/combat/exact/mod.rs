mod damage;
mod factors;
mod fraction;

#[cfg(test)]
mod tests;

pub use damage::{ExactDirectDamageBreakdown, ExactDirectDamageRequest, evaluate_direct_damage};
pub use factors::{
    defense_factor, direct_damage, exact_healing, resistance_factor, try_defense_factor,
    try_direct_damage, try_exact_healing, try_resistance_factor,
};
pub use fraction::{ExactArithmeticError, ExactFraction};
