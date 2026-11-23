//! Exact bounded resources and health settlement. Callers supply units and shield
//! order explicitly; this module does not assert NTE rounding or shield rules.
use serde::{Deserialize, Serialize};

mod exact_pool;
pub use exact_pool::{ExactPool, ExactPoolSummary};
mod wide_pool;
pub use wide_pool::{ResourcePoolSummary, WideFraction, WidePool, WidePoolSummary, wide_fraction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum StateError {
    AboveCapacity,
    InsufficientResource { required: u64, available: u64 },
    ZeroHealthCapacity,
    DeadRecipient,
    DuplicateShield(String),
    UnknownShield(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pool {
    current: u64,
    capacity: u64,
}

impl<'de> Deserialize<'de> for Pool {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Input {
            current: u64,
            capacity: u64,
        }
        let input = Input::deserialize(deserializer)?;
        Pool::new(input.current, input.capacity)
            .map_err(|error| serde::de::Error::custom(format!("{error:?}")))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grant {
    pub applied: u64,
    pub excess: u64,
}

impl Pool {
    pub fn new(current: u64, capacity: u64) -> Result<Self, StateError> {
        if current > capacity {
            Err(StateError::AboveCapacity)
        } else {
            Ok(Self { current, capacity })
        }
    }
    pub fn current(&self) -> u64 {
        self.current
    }
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
    pub fn grant(&mut self, amount: u64) -> Grant {
        let applied = amount.min(self.capacity - self.current);
        self.current += applied;
        Grant {
            applied,
            excess: amount - applied,
        }
    }
    pub fn spend(&mut self, amount: u64) -> Result<(), StateError> {
        if amount > self.current {
            Err(StateError::InsufficientResource {
                required: amount,
                available: self.current,
            })
        } else {
            self.current -= amount;
            Ok(())
        }
    }
    pub fn set_capacity(&mut self, capacity: u64) -> u64 {
        let lost = self.current.saturating_sub(capacity);
        self.capacity = capacity;
        self.current = self.current.min(capacity);
        lost
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShieldOrder {
    OldestFirst,
    NewestFirst,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Shield {
    pub id: String,
    pub source: String,
    remaining: u64,
}
impl Shield {
    pub fn remaining(&self) -> u64 {
        self.remaining
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Absorption {
    pub shield: String,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DamageSettlement {
    pub absorbed: Vec<Absorption>,
    pub hp_lost: u64,
    pub overkill: u64,
    pub died: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VitalState {
    hp: Pool,
    shields: Vec<Shield>,
    order: ShieldOrder,
}

impl VitalState {
    pub fn new(hp: u64, max_hp: u64, order: ShieldOrder) -> Result<Self, StateError> {
        if max_hp == 0 {
            return Err(StateError::ZeroHealthCapacity);
        }
        Ok(Self {
            hp: Pool::new(hp, max_hp)?,
            shields: Vec::new(),
            order,
        })
    }
    pub fn hp(&self) -> u64 {
        self.hp.current()
    }
    pub fn is_alive(&self) -> bool {
        self.hp() > 0
    }
    pub fn shields(&self) -> &[Shield] {
        &self.shields
    }
    pub fn add_shield(
        &mut self,
        id: String,
        source: String,
        amount: u64,
    ) -> Result<(), StateError> {
        if !self.is_alive() {
            return Err(StateError::DeadRecipient);
        }
        if self.shields.iter().any(|s| s.id == id) {
            return Err(StateError::DuplicateShield(id));
        }
        self.shields.push(Shield {
            id,
            source,
            remaining: amount,
        });
        Ok(())
    }
    pub fn remove_shield(&mut self, id: &str) -> Result<Shield, StateError> {
        let index = self
            .shields
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| StateError::UnknownShield(id.to_owned()))?;
        Ok(self.shields.remove(index))
    }
    pub fn heal(&mut self, amount: u64) -> Result<Grant, StateError> {
        if !self.is_alive() {
            Err(StateError::DeadRecipient)
        } else {
            Ok(self.hp.grant(amount))
        }
    }
    pub fn damage(&mut self, amount: u64) -> Result<DamageSettlement, StateError> {
        if !self.is_alive() {
            return Err(StateError::DeadRecipient);
        }
        let absorbed = self.absorb_shields(amount)?;
        let remaining = amount - absorbed.iter().map(|entry| entry.amount).sum::<u64>();
        let (hp_lost, overkill, died) = self.settle_hp_damage(remaining)?;
        Ok(DamageSettlement {
            absorbed,
            hp_lost,
            overkill,
            died,
        })
    }

    pub fn absorb_shields(&mut self, amount: u64) -> Result<Vec<Absorption>, StateError> {
        if !self.is_alive() {
            return Err(StateError::DeadRecipient);
        }
        let mut remaining = amount;
        let mut absorbed = Vec::new();
        let indices: Box<dyn Iterator<Item = usize>> = match self.order {
            ShieldOrder::OldestFirst => Box::new(0..self.shields.len()),
            ShieldOrder::NewestFirst => Box::new((0..self.shields.len()).rev()),
        };
        for index in indices {
            let shield = &mut self.shields[index];
            let taken = remaining.min(shield.remaining);
            if taken > 0 {
                shield.remaining -= taken;
                remaining -= taken;
                absorbed.push(Absorption {
                    shield: shield.id.clone(),
                    amount: taken,
                });
            }
            if remaining == 0 {
                break;
            }
        }
        self.shields.retain(|s| s.remaining > 0);
        Ok(absorbed)
    }

    pub fn settle_hp_damage(&mut self, amount: u64) -> Result<(u64, u64, bool), StateError> {
        if !self.is_alive() {
            return Err(StateError::DeadRecipient);
        }
        let hp_lost = amount.min(self.hp());
        self.hp.spend(hp_lost)?;
        let overkill = amount - hp_lost;
        Ok((hp_lost, overkill, !self.is_alive()))
    }
}

#[cfg(test)]
mod tests;
