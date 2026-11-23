use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePoolSpec {
    pub pool_id: String,
    pub initial_amount: u64,
    pub capacity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResourcePoolState {
    pub pool_id: String,
    pub current: u64,
    pub capacity: u64,
    pub initial: u64,
    pub total_granted: u64,
    pub total_spent: u64,
    pub total_overflow: u64,
}

impl ResourcePoolState {
    pub fn grant(&mut self, amount: u64) -> Result<(u64, u64), ExecutionError> {
        let mut pool = crate::state::Pool::new(self.current, self.capacity).map_err(|error| {
            ExecutionError::InvariantViolation {
                invariant_name: "resource_capacity".into(),
                details: format!("{error:?}"),
            }
        })?;
        let grant = pool.grant(amount);
        let total_granted = self.total_granted.checked_add(amount).ok_or_else(|| {
            ExecutionError::InvariantViolation {
                invariant_name: "resource_counter".into(),
                details: "grant counter overflow".into(),
            }
        })?;
        let total_overflow = self
            .total_overflow
            .checked_add(grant.excess)
            .ok_or_else(|| ExecutionError::InvariantViolation {
                invariant_name: "resource_counter".into(),
                details: "overflow counter overflow".into(),
            })?;
        self.current = pool.current();
        self.total_granted = total_granted;
        self.total_overflow = total_overflow;
        Ok((grant.applied, grant.excess))
    }

    pub fn spend(&mut self, amount: u64) -> Result<(), ExecutionError> {
        if amount > self.current {
            return Err(ExecutionError::ResourceShortage {
                pool: self.pool_id.clone(),
                available: self.current,
                required: amount,
            });
        }
        let mut pool = crate::state::Pool::new(self.current, self.capacity).map_err(|error| {
            ExecutionError::InvariantViolation {
                invariant_name: "resource_capacity".into(),
                details: format!("{error:?}"),
            }
        })?;
        pool.spend(amount)
            .map_err(|error| ExecutionError::InvariantViolation {
                invariant_name: "resource_spend".into(),
                details: format!("{error:?}"),
            })?;
        let total_spent = self.total_spent.checked_add(amount).ok_or_else(|| {
            ExecutionError::InvariantViolation {
                invariant_name: "resource_counter".into(),
                details: "spend counter overflow".into(),
            }
        })?;
        self.current = pool.current();
        self.total_spent = total_spent;
        Ok(())
    }

    pub fn is_conserved(&self) -> bool {
        let lhs = (self.initial as u128) + (self.total_granted as u128);
        let rhs =
            (self.current as u128) + (self.total_spent as u128) + (self.total_overflow as u128);
        lhs == rhs
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResourcePoolReport {
    pub pool_id: String,
    pub current: u64,
    pub capacity: u64,
    pub initial: u64,
    pub total_granted: u64,
    pub total_spent: u64,
    pub total_overflow: u64,
    pub conserved: bool,
}
