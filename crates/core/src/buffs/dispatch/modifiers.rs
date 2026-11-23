use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TargetRecipient {
    Index(usize),
    Name(String),
}

impl TargetRecipient {
    pub fn matches_str(&self, target_id: &str) -> bool {
        match self {
            Self::Index(idx) => target_id.parse::<usize>().ok() == Some(*idx),
            Self::Name(name) => name == target_id,
        }
    }

    pub fn matches_index(&self, index: usize) -> bool {
        match self {
            Self::Index(idx) => *idx == index,
            Self::Name(name) => name.parse::<usize>().ok() == Some(index),
        }
    }
}

impl From<usize> for TargetRecipient {
    fn from(idx: usize) -> Self {
        Self::Index(idx)
    }
}

impl From<&str> for TargetRecipient {
    fn from(s: &str) -> Self {
        if let Ok(idx) = s.parse::<usize>() {
            Self::Index(idx)
        } else {
            Self::Name(s.to_string())
        }
    }
}

impl From<String> for TargetRecipient {
    fn from(s: String) -> Self {
        if let Ok(idx) = s.parse::<usize>() {
            Self::Index(idx)
        } else {
            Self::Name(s)
        }
    }
}

impl std::fmt::Display for TargetRecipient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Index(idx) => write!(f, "{idx}"),
            Self::Name(name) => write!(f, "{name}"),
        }
    }
}

impl Serialize for TargetRecipient {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Index(idx) => serializer.serialize_str(&idx.to_string()),
            Self::Name(name) => serializer.serialize_str(name),
        }
    }
}

impl<'de> Deserialize<'de> for TargetRecipient {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = TargetRecipient;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a target identifier string or integer")
            }
            fn visit_str<E>(self, value: &str) -> Result<TargetRecipient, E>
            where
                E: serde::de::Error,
            {
                Ok(TargetRecipient::from(value))
            }
            fn visit_string<E>(self, value: String) -> Result<TargetRecipient, E>
            where
                E: serde::de::Error,
            {
                Ok(TargetRecipient::from(value))
            }
            fn visit_u64<E>(self, value: u64) -> Result<TargetRecipient, E>
            where
                E: serde::de::Error,
            {
                Ok(TargetRecipient::Index(value as usize))
            }
            fn visit_i64<E>(self, value: i64) -> Result<TargetRecipient, E>
            where
                E: serde::de::Error,
            {
                if value >= 0 {
                    Ok(TargetRecipient::Index(value as usize))
                } else {
                    Err(serde::de::Error::custom("target index cannot be negative"))
                }
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(
    deny_unknown_fields,
    tag = "type",
    content = "target",
    rename_all = "snake_case"
)]
pub enum BuffRecipient {
    SelfActor,
    ActiveActor,
    TeamAll,
    SpecificActor(usize),
    SpecificTarget(TargetRecipient),
}

impl BuffRecipient {
    pub fn matches_target_str(&self, target_id: &str) -> bool {
        match self {
            Self::SpecificTarget(target) => target.matches_str(target_id),
            _ => false,
        }
    }

    pub fn matches_target_index(&self, index: usize) -> bool {
        match self {
            Self::SpecificTarget(target) => target.matches_index(index),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum BuffDuration {
    Permanent,
    Timed {
        duration_us: u64,
        expires_at_us: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "policy", rename_all = "snake_case")]
pub enum RefreshPolicy {
    Reset,
    Extend { max_duration_us: Option<u64> },
    Independent,
    Disallow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayPolicy {
    AllAtOnce,
    OneByOne,
}

/// Identity of overlapping applications. Shared applications replace the latest
/// source snapshot; PerSource applications retain independent carrier ownership.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuffStackingScope {
    #[default]
    PerSource,
    Shared,
}

/// Determines whether a managed effect follows its source actor's life state
/// or remains as a captured snapshot after application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuffSourceLifetime {
    #[default]
    Snapshot,
    WhileAlive,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuffDefinition {
    #[serde(default)]
    pub stacking_scope: BuffStackingScope,
    /// Domain-neutral labels for generic queries over managed state. The core
    /// stores and compares these labels but assigns no gameplay meaning to them.
    #[serde(default)]
    pub semantic_tags: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub source_lifetime: BuffSourceLifetime,
    pub id: String,
    pub source: String,
    pub recipient: BuffRecipient,
    pub duration_us: Option<u64>,
    pub refresh_policy: RefreshPolicy,
    pub max_stacks: u32,
    pub decay_policy: DecayPolicy,
    pub stat_modifiers: BTreeMap<String, Ratio>,
}

impl BuffDefinition {
    pub fn validate(&self) -> Result<(), DispatchError> {
        if self.id.is_empty()
            || self.source.is_empty()
            || self.duration_us == Some(0)
            || self.max_stacks == 0
            || self.semantic_tags.iter().any(String::is_empty)
            || self.stat_modifiers.values().any(|r| r.denominator == 0)
        {
            return Err(DispatchError::InvalidBuff(self.id.clone()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ActiveBuffInstance {
    pub instance_id: u64,
    #[serde(default)]
    pub stacking_scope: BuffStackingScope,
    #[serde(default)]
    pub semantic_tags: std::collections::BTreeSet<String>,
    #[serde(default)]
    pub source_lifetime: BuffSourceLifetime,
    pub id: String,
    pub source: String,
    pub owner_actor: usize,
    pub recipient: BuffRecipient,
    pub duration: BuffDuration,
    pub refresh_policy: RefreshPolicy,
    pub stacks: u32,
    pub max_stacks: u32,
    pub decay_policy: DecayPolicy,
    pub stat_modifiers: BTreeMap<String, Ratio>,
}

impl ActiveBuffInstance {
    pub fn effective_stacks_at(&self, now_us: u64) -> u32 {
        match self.duration {
            BuffDuration::Permanent => self.stacks,
            BuffDuration::Timed {
                expires_at_us,
                duration_us,
            } => {
                if now_us < expires_at_us {
                    self.stacks
                } else if self.decay_policy == DecayPolicy::OneByOne && duration_us > 0 {
                    let expired_intervals = (now_us - expires_at_us) / duration_us;
                    let decayed_stacks = 1 + expired_intervals;
                    (self.stacks as u64).saturating_sub(decayed_stacks) as u32
                } else {
                    0
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct BuffManager {
    pub(super) next_instance_id: u64,
    pub(super) active_buffs: Vec<ActiveBuffInstance>,
}

impl BuffManager {
    pub fn instances(&self) -> &[ActiveBuffInstance] {
        &self.active_buffs
    }
    pub fn remove_owned_buff(&mut self, id: &str, source: &str, owner_actor: usize) -> usize {
        let before = self.active_buffs.len();
        self.active_buffs.retain(|buff| {
            buff.id != id || buff.source != source || buff.owner_actor != owner_actor
        });
        before - self.active_buffs.len()
    }
    pub fn remove_target_buff(&mut self, id: &str, source: &str, target: &str) -> usize {
        let before = self.active_buffs.len();
        let recipient = BuffRecipient::SpecificTarget(target.into());
        self.active_buffs
            .retain(|buff| buff.id != id || buff.source != source || buff.recipient != recipient);
        before - self.active_buffs.len()
    }
    /// Application is atomic, including expiry and checked extension. Reset and
    /// Extend capture the new application's values; Disallow retains its snapshot.
    /// Lifecycle definitions sharing a key must agree, even at the stack cap.
    pub fn apply_buff(
        &mut self,
        def: BuffDefinition,
        owner_actor: usize,
        now_us: u64,
    ) -> Result<(), DispatchError> {
        def.validate()?;
        let deadline = def
            .duration_us
            .map(|dur| {
                now_us
                    .checked_add(dur)
                    .ok_or(DispatchError::ArithmeticOverflow)
            })
            .transpose()?;

        let same_key = |b: &ActiveBuffInstance| {
            b.id == def.id
                && b.recipient == def.recipient
                && (def.stacking_scope == BuffStackingScope::Shared
                    || b.stacking_scope == BuffStackingScope::Shared
                    || (b.source == def.source && b.owner_actor == owner_actor))
        };

        for buff in self.active_buffs.iter().filter(|b| same_key(b)) {
            if buff.effective_stacks_at(now_us) == 0 {
                continue;
            }
            let old_duration = match buff.duration {
                BuffDuration::Permanent => None,
                BuffDuration::Timed { duration_us, .. } => Some(duration_us),
            };
            if buff.stacking_scope != def.stacking_scope
                || buff.refresh_policy != def.refresh_policy
                || buff.max_stacks != def.max_stacks
                || buff.decay_policy != def.decay_policy
                || buff.semantic_tags != def.semantic_tags
                || buff.source_lifetime != def.source_lifetime
                || old_duration.is_some() != def.duration_us.is_some()
                || (def.decay_policy == DecayPolicy::OneByOne && old_duration != def.duration_us)
                || buff.stat_modifiers.keys().ne(def.stat_modifiers.keys())
            {
                return Err(DispatchError::InvalidBuff(def.id.clone()));
            }
        }

        if def.refresh_policy == RefreshPolicy::Independent {
            let stacks = self
                .active_buffs
                .iter()
                .filter(|b| same_key(b) && b.effective_stacks_at(now_us) > 0)
                .try_fold(0u32, |total, b| {
                    total
                        .checked_add(b.effective_stacks_at(now_us))
                        .ok_or(DispatchError::ArithmeticOverflow)
                })?;
            if stacks >= def.max_stacks {
                self.advance_time(now_us);
                return Ok(());
            }
        }

        let existing = self.active_buffs.iter().find(|b| {
            same_key(b)
                && b.effective_stacks_at(now_us) > 0
                && b.refresh_policy != RefreshPolicy::Independent
        });

        if let Some(buff) = existing {
            match &def.refresh_policy {
                RefreshPolicy::Extend { max_duration_us } => {
                    if let (BuffDuration::Timed { expires_at_us, .. }, Some(duration_us)) =
                        (&buff.duration, def.duration_us)
                    {
                        let expiry = expires_at_us
                            .checked_add(duration_us)
                            .ok_or(DispatchError::ArithmeticOverflow)?;
                        let expiry = match max_duration_us {
                            Some(cap) => expiry.min(
                                now_us
                                    .checked_add(*cap)
                                    .ok_or(DispatchError::ArithmeticOverflow)?,
                            ),
                            None => expiry,
                        };
                        let new_stacks = if buff.stacks < buff.max_stacks {
                            buff.stacks + 1
                        } else {
                            buff.stacks
                        };
                        if buff.decay_policy == DecayPolicy::OneByOne && new_stacks > 1 {
                            duration_us
                                .checked_mul(u64::from(new_stacks - 1))
                                .and_then(|tail| expiry.checked_add(tail))
                                .ok_or(DispatchError::ArithmeticOverflow)?;
                        }
                    }
                }
                RefreshPolicy::Reset => {
                    if let Some(expires_at_us) = deadline {
                        let duration_us = def.duration_us.unwrap();
                        let new_stacks = if buff.stacks < buff.max_stacks {
                            buff.stacks + 1
                        } else {
                            buff.stacks
                        };
                        if buff.decay_policy == DecayPolicy::OneByOne && new_stacks > 1 {
                            duration_us
                                .checked_mul(u64::from(new_stacks - 1))
                                .and_then(|tail| expires_at_us.checked_add(tail))
                                .ok_or(DispatchError::ArithmeticOverflow)?;
                        }
                    }
                }
                _ => {}
            }
        } else {
            self.next_instance_id
                .checked_add(1)
                .ok_or(DispatchError::ArithmeticOverflow)?;
        }

        self.advance_time(now_us);

        let existing = self
            .active_buffs
            .iter_mut()
            .find(|b| same_key(b) && b.refresh_policy != RefreshPolicy::Independent);

        if let Some(buff) = existing {
            match &def.refresh_policy {
                RefreshPolicy::Disallow => {}
                RefreshPolicy::Reset | RefreshPolicy::Extend { .. } => {
                    let refreshed_duration = match (&def.refresh_policy, &buff.duration, deadline) {
                        (
                            RefreshPolicy::Extend { max_duration_us },
                            BuffDuration::Timed { expires_at_us, .. },
                            Some(_),
                        ) => {
                            let expiry = expires_at_us
                                .checked_add(def.duration_us.unwrap())
                                .ok_or(DispatchError::ArithmeticOverflow)?;
                            let expiry = match max_duration_us {
                                Some(cap) => expiry.min(
                                    now_us
                                        .checked_add(*cap)
                                        .ok_or(DispatchError::ArithmeticOverflow)?,
                                ),
                                None => expiry,
                            };
                            BuffDuration::Timed {
                                duration_us: def.duration_us.unwrap(),
                                expires_at_us: expiry,
                            }
                        }
                        (_, _, Some(expiry)) => BuffDuration::Timed {
                            duration_us: def.duration_us.unwrap(),
                            expires_at_us: expiry,
                        },
                        _ => BuffDuration::Permanent,
                    };
                    buff.stacks = if buff.stacks < buff.max_stacks {
                        buff.stacks + 1
                    } else {
                        buff.stacks
                    };
                    buff.duration = refreshed_duration;
                    buff.stat_modifiers = def.stat_modifiers;
                    buff.semantic_tags = def.semantic_tags;
                    buff.source_lifetime = def.source_lifetime;
                    buff.source = def.source;
                    buff.owner_actor = owner_actor;
                }
                RefreshPolicy::Independent => unreachable!(),
            }
        } else {
            let duration = match deadline {
                None => BuffDuration::Permanent,
                Some(expires_at_us) => BuffDuration::Timed {
                    duration_us: def.duration_us.unwrap(),
                    expires_at_us,
                },
            };
            self.next_instance_id = self
                .next_instance_id
                .checked_add(1)
                .ok_or(DispatchError::ArithmeticOverflow)?;
            self.active_buffs.push(ActiveBuffInstance {
                instance_id: self.next_instance_id,
                stacking_scope: def.stacking_scope,
                semantic_tags: def.semantic_tags,
                source_lifetime: def.source_lifetime,
                id: def.id,
                source: def.source,
                owner_actor,
                recipient: def.recipient,
                duration,
                refresh_policy: def.refresh_policy,
                stacks: 1,
                max_stacks: def.max_stacks,
                decay_policy: def.decay_policy,
                stat_modifiers: def.stat_modifiers,
            });
        }
        Ok(())
    }

    pub fn advance_time(&mut self, now_us: u64) {
        let mut i = 0;
        while i < self.active_buffs.len() {
            let mut expired = false;
            let mut one_stack_decay = false;
            let mut dur_step = 0;

            let is_expired = match self.active_buffs[i].duration {
                BuffDuration::Timed {
                    expires_at_us,
                    duration_us,
                } => {
                    dur_step = duration_us;
                    now_us >= expires_at_us
                }
                BuffDuration::Permanent => false,
            };

            if is_expired {
                if self.active_buffs[i].decay_policy == DecayPolicy::OneByOne
                    && self.active_buffs[i].stacks > 1
                {
                    one_stack_decay = true;
                } else {
                    expired = true;
                }
            }

            if expired {
                self.active_buffs.remove(i);
            } else if one_stack_decay {
                let old_expiry = match self.active_buffs[i].duration {
                    BuffDuration::Timed { expires_at_us, .. } => expires_at_us,
                    _ => unreachable!(),
                };
                self.active_buffs[i].stacks -= 1;
                self.active_buffs[i].duration = BuffDuration::Timed {
                    duration_us: dur_step,
                    expires_at_us: old_expiry
                        .checked_add(dur_step)
                        .expect("validated decay horizon"),
                };
                // Revisit this stack when a sparse advance crossed several deadlines.
            } else {
                i += 1;
            }
        }
    }
}

fn gcd_u128(a: u128, b: u128) -> u128 {
    #[cfg(feature = "performance-diagnostics")]
    crate::performance::modifier_gcd(a, b);
    crate::formula::gcd(a, b)
}

impl BuffManager {
    pub fn get_stat_modifiers_for_actor(
        &self,
        actor_index: usize,
        active_actor: usize,
        now_us: u64,
    ) -> Result<BTreeMap<String, Ratio>, DispatchError> {
        self.aggregate_modifiers(now_us, None, |buff| match &buff.recipient {
            BuffRecipient::SelfActor => buff.owner_actor == actor_index,
            BuffRecipient::ActiveActor => actor_index == active_actor,
            BuffRecipient::TeamAll => true,
            BuffRecipient::SpecificActor(index) => *index == actor_index,
            BuffRecipient::SpecificTarget(_) => false,
        })
    }

    /// Query actor modifiers while applying explicit owner-life semantics.
    /// Buff owners absent from `living_sources` are inactive only for
    /// `WhileAlive`; snapshot modifiers keep their captured values.
    pub fn get_stat_modifiers_for_actor_with_living_sources(
        &self,
        actor_index: usize,
        active_actor: usize,
        now_us: u64,
        living_sources: &std::collections::BTreeSet<usize>,
    ) -> Result<BTreeMap<String, Ratio>, DispatchError> {
        self.aggregate_modifiers(now_us, Some(living_sources), |buff| match &buff.recipient {
            BuffRecipient::SelfActor => buff.owner_actor == actor_index,
            BuffRecipient::ActiveActor => actor_index == active_actor,
            BuffRecipient::TeamAll => true,
            BuffRecipient::SpecificActor(index) => *index == actor_index,
            BuffRecipient::SpecificTarget(_) => false,
        })
    }

    pub fn get_stat_modifiers_for_actor_with_living_sources_borrowed<'a>(
        &'a self,
        actor_index: usize,
        active_actor: usize,
        now_us: u64,
        living_sources: &std::collections::BTreeSet<usize>,
    ) -> Result<Vec<(&'a str, Ratio)>, DispatchError> {
        self.aggregate_modifiers_borrowed(now_us, Some(living_sources), |buff| {
            match &buff.recipient {
                BuffRecipient::SelfActor => buff.owner_actor == actor_index,
                BuffRecipient::ActiveActor => actor_index == active_actor,
                BuffRecipient::TeamAll => true,
                BuffRecipient::SpecificActor(index) => *index == actor_index,
                BuffRecipient::SpecificTarget(_) => false,
            }
        })
    }

    pub fn get_stat_modifiers_for_target(
        &self,
        target_id: &str,
        now_us: u64,
    ) -> Result<BTreeMap<String, Ratio>, DispatchError> {
        self.aggregate_modifiers(now_us, None, |buff| {
            buff.recipient.matches_target_str(target_id)
        })
    }

    /// Sum active stacks carrying an exact caller-defined tag on one target.
    /// The manager treats tags as opaque labels; domain policy belongs to the
    /// caller. Expired instances are excluded at the supplied simulation time.
    pub fn active_tagged_stacks_for_target(
        &self,
        target_id: &str,
        tag: &str,
        now_us: u64,
    ) -> Result<u32, DispatchError> {
        let mut total = 0u32;
        for buff in &self.active_buffs {
            let stacks = buff.effective_stacks_at(now_us);
            if stacks > 0
                && buff.semantic_tags.contains(tag)
                && buff.recipient.matches_target_str(target_id)
            {
                total = total
                    .checked_add(stacks)
                    .ok_or(DispatchError::ArithmeticOverflow)?;
            }
        }
        Ok(total)
    }

    pub fn active_tagged_stacks_for_target_index(
        &self,
        target_index: usize,
        tag: &str,
        now_us: u64,
    ) -> Result<u32, DispatchError> {
        let mut total = 0u32;
        for buff in &self.active_buffs {
            let stacks = buff.effective_stacks_at(now_us);
            if stacks > 0
                && buff.semantic_tags.contains(tag)
                && buff.recipient.matches_target_index(target_index)
            {
                total = total
                    .checked_add(stacks)
                    .ok_or(DispatchError::ArithmeticOverflow)?;
            }
        }
        Ok(total)
    }

    pub fn aggregate_modifiers_borrowed<'a>(
        &'a self,
        now_us: u64,
        living_sources: Option<&std::collections::BTreeSet<usize>>,
        applies: impl Fn(&ActiveBuffInstance) -> bool,
    ) -> Result<Vec<(&'a str, Ratio)>, DispatchError> {
        let mut result: Vec<(&'a str, Ratio)> = Vec::new();
        for buff in &self.active_buffs {
            let stacks = buff.effective_stacks_at(now_us);
            if stacks == 0 {
                continue;
            }

            if applies(buff) {
                let source_is_active = match (buff.source_lifetime, living_sources) {
                    (BuffSourceLifetime::WhileAlive, Some(living)) => {
                        living.contains(&buff.owner_actor)
                    }
                    (BuffSourceLifetime::WhileAlive, None) => {
                        return Err(DispatchError::MissingSourceLivenessContext);
                    }
                    (BuffSourceLifetime::Snapshot, _) => true,
                };
                if !source_is_active {
                    continue;
                }
                let stacks = stacks as u128;
                for (stat, ratio) in &buff.stat_modifiers {
                    let stat_str = stat.as_str();
                    let idx = result.iter().position(|(s, _)| *s == stat_str);
                    let added_num = ratio.numerator as u128 * stacks;
                    let added_denom = ratio.denominator as u128;
                    match idx {
                        Some(i) => {
                            let entry = &mut result[i].1;
                            let new_num = (entry.numerator as u128)
                                .checked_mul(added_denom)
                                .and_then(|left| {
                                    added_num
                                        .checked_mul(entry.denominator as u128)
                                        .and_then(|right| left.checked_add(right))
                                })
                                .ok_or(DispatchError::ArithmeticOverflow)?;
                            let new_denom = (entry.denominator as u128)
                                .checked_mul(added_denom)
                                .ok_or(DispatchError::ArithmeticOverflow)?;
                            let g = gcd_u128(new_num, new_denom);
                            let g = if g == 0 { 1 } else { g };
                            entry.numerator = u64::try_from(new_num / g)
                                .map_err(|_| DispatchError::ArithmeticOverflow)?;
                            entry.denominator = u64::try_from(new_denom / g)
                                .map_err(|_| DispatchError::ArithmeticOverflow)?;
                        }
                        None => {
                            let g = gcd_u128(added_num, added_denom);
                            let g = if g == 0 { 1 } else { g };
                            result.push((
                                stat_str,
                                Ratio {
                                    numerator: u64::try_from(added_num / g)
                                        .map_err(|_| DispatchError::ArithmeticOverflow)?,
                                    denominator: u64::try_from(added_denom / g)
                                        .map_err(|_| DispatchError::ArithmeticOverflow)?,
                                },
                            ));
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    fn aggregate_modifiers(
        &self,
        now_us: u64,
        living_sources: Option<&std::collections::BTreeSet<usize>>,
        applies: impl Fn(&ActiveBuffInstance) -> bool,
    ) -> Result<BTreeMap<String, Ratio>, DispatchError> {
        let borrowed = self.aggregate_modifiers_borrowed(now_us, living_sources, applies)?;
        let mut result = BTreeMap::new();
        for (stat, ratio) in borrowed {
            result.insert(stat.to_string(), ratio);
        }
        Ok(result)
    }
}
