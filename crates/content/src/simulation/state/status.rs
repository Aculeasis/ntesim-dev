use super::target::TargetRuntime;
use crate::formula::Ratio;
use crate::simulation::ExactFraction;
use std::collections::BTreeMap;
/// Content-owned values for one timed stat application. Core owns its keying,
/// compatibility, refresh and expiry behavior after this declaration is made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TimedBuffDeclaration {
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) recipient: ntesim_core::buffs::dispatch::BuffRecipient,
    pub(crate) duration_us: u64,
    pub(crate) stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope,
    pub(crate) refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy,
    pub(crate) max_stacks: u32,
    pub(crate) decay_policy: ntesim_core::buffs::dispatch::DecayPolicy,
    pub(crate) stat_modifiers: BTreeMap<String, ExactFraction>,
}

impl TimedBuffDeclaration {
    pub(crate) fn reset(
        id: &str,
        source: &str,
        recipient: ntesim_core::buffs::dispatch::BuffRecipient,
        duration_us: u64,
        stacking_scope: ntesim_core::buffs::dispatch::BuffStackingScope,
        stat_modifiers: BTreeMap<String, ExactFraction>,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            recipient,
            duration_us,
            stacking_scope,
            refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
            max_stacks: 1,
            decay_policy: ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
            stat_modifiers,
        }
    }
}

pub(crate) struct TargetStatusDeclaration {
    pub(crate) source_actor: usize,
    pub(crate) source: String,
    pub(crate) target_index: usize,
    pub(crate) id: String,
    pub(crate) now_us: u64,
    pub(crate) duration_us: u64,
    pub(crate) reductions: BTreeMap<String, ExactFraction>,
}

struct ManagedBuffApplication {
    definition: ntesim_core::buffs::dispatch::BuffDefinition,
    owner_actor: usize,
    now_us: u64,
}

fn apply_managed_buff(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    application: ManagedBuffApplication,
) -> Result<(), String> {
    buffs
        .apply_buff(
            application.definition,
            application.owner_actor,
            application.now_us,
        )
        .map_err(|error| format!("buff application failed: {error:?}"))
}

pub(crate) fn apply_timed_buff(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    declaration: TimedBuffDeclaration,
    owner_actor: usize,
    now_us: u64,
) -> Result<(), String> {
    apply_timed_buff_with_lifecycle(
        buffs,
        declaration,
        owner_actor,
        now_us,
        ntesim_core::buffs::dispatch::BuffSourceLifetime::Snapshot,
    )
}

pub(crate) fn apply_timed_buff_with_lifecycle(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    declaration: TimedBuffDeclaration,
    owner_actor: usize,
    now_us: u64,
    source_lifetime: ntesim_core::buffs::dispatch::BuffSourceLifetime,
) -> Result<(), String> {
    use ntesim_core::buffs::dispatch::BuffDefinition;
    let stat_modifiers = declaration
        .stat_modifiers
        .into_iter()
        .map(|(key, value)| {
            Ok((
                key,
                Ratio {
                    numerator: u64::try_from(value.numerator)
                        .map_err(|_| "buff numerator exceeds u64")?,
                    denominator: u64::try_from(value.denominator)
                        .map_err(|_| "buff denominator exceeds u64")?,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    apply_managed_buff(
        buffs,
        ManagedBuffApplication {
            definition: BuffDefinition {
                stacking_scope: declaration.stacking_scope,
                semantic_tags: Default::default(),
                source_lifetime,
                id: declaration.id,
                source: declaration.source,
                recipient: declaration.recipient,
                duration_us: Some(declaration.duration_us),
                refresh_policy: declaration.refresh_policy,
                max_stacks: declaration.max_stacks,
                decay_policy: declaration.decay_policy,
                stat_modifiers,
            },
            owner_actor,
            now_us,
        },
    )
}

pub(crate) fn apply_target_status(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    declaration: TargetStatusDeclaration,
) -> Result<(), String> {
    use ntesim_core::buffs::dispatch::{
        BuffDefinition, BuffRecipient, BuffStackingScope, DecayPolicy, RefreshPolicy,
    };
    let stat_modifiers = declaration
        .reductions
        .into_iter()
        .map(|(element, value)| {
            Ok((
                format!("resistance_reduction:{element}"),
                Ratio {
                    numerator: u64::try_from(value.numerator)
                        .map_err(|_| "status numerator exceeds u64")?,
                    denominator: u64::try_from(value.denominator)
                        .map_err(|_| "status denominator exceeds u64")?,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    apply_managed_buff(
        buffs,
        ManagedBuffApplication {
            definition: BuffDefinition {
                stacking_scope: BuffStackingScope::PerSource,
                semantic_tags: Default::default(),
                source_lifetime: Default::default(),
                id: declaration.id,
                source: declaration.source,
                recipient: BuffRecipient::SpecificTarget(declaration.target_index.into()),
                duration_us: Some(declaration.duration_us),
                refresh_policy: RefreshPolicy::Reset,
                max_stacks: 1,
                decay_policy: DecayPolicy::AllAtOnce,
                stat_modifiers,
            },
            owner_actor: declaration.source_actor,
            now_us: declaration.now_us,
        },
    )
    .map_err(|error| format!("target status application failed: {error}"))
}

/// Apply an owner-declared target status carrying opaque semantic labels and a
/// caller-specified stack cap. Core owns lifecycle and storage; content owns
/// status meaning and the evidence-backed application policy.
pub(crate) fn apply_target_tagged_status(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    declaration: TargetStatusDeclaration,
    semantic_tags: std::collections::BTreeSet<String>,
    max_stacks: u32,
    refresh_policy: ntesim_core::buffs::dispatch::RefreshPolicy,
    decay_policy: ntesim_core::buffs::dispatch::DecayPolicy,
) -> Result<(), String> {
    use ntesim_core::buffs::dispatch::{BuffDefinition, BuffRecipient, BuffStackingScope};
    if max_stacks == 0 {
        return Err("target status max_stacks must be positive".into());
    }
    let stat_modifiers = declaration
        .reductions
        .into_iter()
        .map(|(element, value)| {
            Ok((
                format!("resistance_reduction:{element}"),
                Ratio {
                    numerator: u64::try_from(value.numerator)
                        .map_err(|_| "status numerator exceeds u64")?,
                    denominator: u64::try_from(value.denominator)
                        .map_err(|_| "status denominator exceeds u64")?,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let definition = BuffDefinition {
        stacking_scope: BuffStackingScope::PerSource,
        semantic_tags,
        source_lifetime: Default::default(),
        id: declaration.id,
        source: declaration.source,
        recipient: BuffRecipient::SpecificTarget(declaration.target_index.into()),
        duration_us: Some(declaration.duration_us),
        refresh_policy,
        max_stacks,
        decay_policy,
        stat_modifiers,
    };
    let mut next = buffs.clone();
    next.apply_buff(definition, declaration.source_actor, declaration.now_us)
        .map_err(|error| format!("tagged target status application failed: {error:?}"))?;
    *buffs = next;
    Ok(())
}

pub(crate) fn target_has_status(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    source_actor: usize,
    source: &str,
    target_index: usize,
    id: &str,
) -> bool {
    buffs.instances().iter().any(|buff| {
        buff.owner_actor == source_actor
            && buff.source == source
            && buff.id == id
            && buff.recipient.matches_target_index(target_index)
    })
}

pub(crate) fn target_status_count(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
) -> usize {
    buffs
        .instances()
        .iter()
        .filter(|buff| buff.recipient.matches_target_index(target_index))
        .count()
}

pub(crate) fn target_tagged_status_stacks(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    semantic_tag: &str,
    now_us: u64,
) -> Result<u32, String> {
    buffs
        .active_tagged_stacks_for_target_index(target_index, semantic_tag, now_us)
        .map_err(|error| format!("target status stack query failed: {error:?}"))
}

pub(crate) fn owned_status_counts(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    targets: &[TargetRuntime],
    source_actor: usize,
) -> BTreeMap<String, usize> {
    use ntesim_core::buffs::dispatch::{BuffRecipient, TargetRecipient};
    let mut counts = BTreeMap::new();
    for buff in buffs.instances() {
        if buff.owner_actor == source_actor
            && match &buff.recipient {
                BuffRecipient::SpecificTarget(TargetRecipient::Index(idx)) => {
                    targets.get(*idx).is_some_and(|t| t.hp_current > 0)
                }
                BuffRecipient::SpecificTarget(TargetRecipient::Name(name)) => name
                    .parse::<usize>()
                    .ok()
                    .is_some_and(|idx| targets.get(idx).is_some_and(|t| t.hp_current > 0)),
                _ => false,
            }
        {
            *counts.entry(buff.id.clone()).or_insert(0) += 1;
        }
    }
    counts
}

pub(crate) fn target_resistance_reduction(
    buffs: &ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    element: &str,
) -> Result<ExactFraction, String> {
    let key = format!("resistance_reduction:{element}");
    buffs
        .instances()
        .iter()
        .filter(|buff| buff.recipient.matches_target_index(target_index))
        .filter_map(|buff| buff.stat_modifiers.get(&key))
        .try_fold(ExactFraction::new(0, 1)?, |sum, value| {
            sum.add(ExactFraction::from_ratio(*value)?)
        })
}
