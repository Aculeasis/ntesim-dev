use super::actions::*;
use super::hypotheses::{record_hypothesis, record_hypothesis_id};
use super::modifiers::{HEALING_MODIFIERS_HYPOTHESIS_ID, HealingSettlement};
use crate::HypothesisId;
use crate::simulation::ExactFraction;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameActorConfig, GameEventReceipt, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::{ActorRuntime, GameSimulatorState, TargetRuntime},
};
pub struct CastContext<'a> {
    pub(crate) world_time_us: u64,
    pub(crate) origins: &'a mut crate::simulation::Origins,
    pub(crate) command_index: usize,
    /// Effective coordinate of the admitted native action, which may be an
    /// automatic Support owned by a switch rather than the source command.
    pub(crate) native_array_index: Option<usize>,
    pub(crate) execution_assumptions: &'a mut std::collections::BTreeMap<String, String>,
    pub(crate) config: &'a GameActorConfig,
    pub(crate) request: &'a crate::simulation::GameScenarioRequest,
    pub(crate) actor_index: usize,
    pub(crate) active_actor_index: usize,
    pub(crate) target_index: usize,
    pub(crate) ability_id: &'a str,
    pub(crate) action_metadata: &'a crate::simulation::metadata::ActionMetadata,
    pub(crate) packet_metadata: Option<&'a crate::simulation::metadata::PacketMetadata>,
    pub(crate) hit_time_us: u64,
    pub(crate) time_us: u64,
    pub(crate) buffs: &'a mut ntesim_core::buffs::dispatch::BuffManager,
    pub(crate) actor: &'a mut ActorRuntime,
    pub(crate) stamina: Option<&'a mut crate::simulation::CombatStaminaReport>,
    pub(crate) team_hp_snapshot: &'a [(u64, u64)],
    pub(crate) targets: &'a [crate::simulation::state::TargetRuntime],
    pub(crate) queue: &'a mut BinaryHeap,
    pub(crate) sequence: &'a mut u64,
}

use std::hash::BuildHasher;
use std::sync::{Arc, LazyLock, RwLock};

type ChildPacketCoordinates<'a> = (&'a str, &'a str, &'a str, &'a str, u32);

struct ChildPacketCacheKey {
    content_id: String,
    character_id: String,
    ability_id: String,
    damage_effect_id: String,
    array_index: u32,
    model_assumptions: std::collections::BTreeMap<String, String>,
}

impl ChildPacketCacheKey {
    fn coordinates(&self) -> ChildPacketCoordinates<'_> {
        (
            &self.content_id,
            &self.character_id,
            &self.ability_id,
            &self.damage_effect_id,
            self.array_index,
        )
    }
}

struct ChildPacketCacheEntry {
    base_atk_rate: ntesim_core::formula::Ratio,
    scaling_stat: crate::simulation::NativeScalingStat,
    packet: crate::simulation::metadata::PacketMetadata,
    binding: crate::simulation::metadata::ActionPacketBinding,
}

type CachedChild = Result<Arc<ChildPacketCacheEntry>, String>;

#[derive(Default)]
struct ChildPackets {
    candidates: ntesim_core::FastHashMap<u64, Vec<(ChildPacketCacheKey, CachedChild)>>,
    count: usize,
}

impl ChildPackets {
    fn get(
        &self,
        hash: u64,
        coordinates: ChildPacketCoordinates<'_>,
        assumptions: &std::collections::BTreeMap<String, String>,
    ) -> Option<&CachedChild> {
        self.candidates.get(&hash)?.iter().find_map(|(key, value)| {
            (key.coordinates() == coordinates && &key.model_assumptions == assumptions)
                .then_some(value)
        })
    }
}

static CHILD_PACKET_CACHE: LazyLock<RwLock<ChildPackets>> = LazyLock::new(RwLock::default);
const MAX_CACHED_CHILD_PACKETS: usize = 4_096;

fn resolve_child_packet(
    content_id: &str,
    character_id: &str,
    action_metadata: &crate::simulation::metadata::ActionMetadata,
    damage_effect_id: &str,
    array_index: u32,
    model_assumptions: &std::collections::BTreeMap<String, String>,
) -> CachedChild {
    let coordinates = (
        content_id,
        character_id,
        action_metadata.ability_id.as_str(),
        damage_effect_id,
        array_index,
    );
    let hash = ntesim_core::fast_hash::FxBuildHasher.hash_one(coordinates);

    if let Some(res) = CHILD_PACKET_CACHE
        .read()
        .ok()
        .and_then(|cache| cache.get(hash, coordinates, model_assumptions).cloned())
    {
        return res;
    }

    let compute = || -> Result<ChildPacketCacheEntry, String> {
        let tables = ntesim_data::skill_levels::frozen::tables();
        let projection = crate::build::project_native_packet_coefficient(
            tables,
            &crate::build::PacketCoefficientRequest {
                content_id,
                character_id,
                ability_id: &action_metadata.ability_id,
                damage_effect_id,
                array_index,
                research_metadata: None,
                model_assumptions,
            },
        )?;
        let base_atk_rate: ntesim_core::formula::Ratio =
            serde_json::from_value(projection["compiled_rate"].clone())
                .map_err(|error| format!("native child coefficient invalid: {error}"))?;
        let scaling_stat = serde_json::from_value(projection["compiled_scaling_stat"].clone())
            .map_err(|error| format!("native child scaling stat invalid: {error}"))?;

        let packet_meta = crate::simulation::metadata::resolve_packet_for_action(
            content_id,
            tables,
            character_id,
            action_metadata,
            damage_effect_id,
            model_assumptions,
        )?;

        Ok(ChildPacketCacheEntry {
            base_atk_rate,
            scaling_stat,
            packet: packet_meta.packet,
            binding: packet_meta.binding,
        })
    };

    let result = compute().map(Arc::new);
    if let Ok(mut cache) = CHILD_PACKET_CACHE.write() {
        if let Some(existing) = cache.get(hash, coordinates, model_assumptions) {
            return existing.clone();
        }
        if cache.count < MAX_CACHED_CHILD_PACKETS {
            let key = ChildPacketCacheKey {
                content_id: content_id.to_owned(),
                character_id: character_id.to_owned(),
                ability_id: action_metadata.ability_id.clone(),
                damage_effect_id: damage_effect_id.to_owned(),
                array_index,
                model_assumptions: model_assumptions.clone(),
            };
            cache
                .candidates
                .entry(hash)
                .or_default()
                .push((key, result.clone()));
            cache.count += 1;
        }
    }
    result
}

impl CastContext<'_> {
    pub(crate) fn scale_primary_coefficient(
        &mut self,
        effect: &str,
        factor: ExactFraction,
    ) -> Result<(), String> {
        let (sequence, rate) = self
            .queue
            .iter()
            .find_map(|event| match &event.data {
                GameEventData::HitDelivery {
                    command_index,
                    actor_index,
                    target_index,
                    damage_effect_id,
                    atk_rate,
                    ..
                } if *command_index == self.command_index
                    && *actor_index == self.actor_index
                    && *target_index == self.target_index
                    && damage_effect_id == effect =>
                {
                    Some((event.seq, *atk_rate))
                }
                _ => None,
            })
            .ok_or("native primary coefficient missing")?;
        let scaled = ExactFraction::from_ratio(rate)?.mul(factor)?;
        let replacement = ntesim_core::formula::Ratio {
            numerator: scaled
                .numerator
                .try_into()
                .map_err(|_| "primary coefficient numerator overflow")?,
            denominator: scaled
                .denominator
                .try_into()
                .map_err(|_| "primary coefficient denominator overflow")?,
        };
        let mut events = self.queue.drain().collect::<Vec<_>>();
        for event in &mut events {
            if event.seq == sequence
                && let GameEventData::HitDelivery { atk_rate, .. } = &mut event.data
            {
                *atk_rate = replacement;
            }
        }
        self.queue.replace(events);
        Ok(())
    }
    /// Replace the admitted primary with an owner-selected native chain packet.
    /// Preserve its causal sequence; admission metadata continues to describe the input.
    pub(crate) fn replace_native_primary_packet(
        &mut self,
        damage_effect_id: &str,
    ) -> Result<(), String> {
        let old_sequence = self
            .queue
            .iter()
            .find_map(|event| match &event.data {
                crate::simulation::events::GameEventData::HitDelivery {
                    command_index,
                    actor_index,
                    target_index,
                    damage_effect_id,
                    ..
                } if *command_index == self.command_index
                    && *actor_index == self.actor_index
                    && *target_index == self.target_index
                    && self
                        .packet_metadata
                        .is_some_and(|packet| packet.damage_effect_id == *damage_effect_id) =>
                {
                    Some(event.seq)
                }
                _ => None,
            })
            .ok_or("admitted primary delivery missing")?;
        self.schedule_native_child_packet(damage_effect_id, self.target_index, self.hit_time_us)?;
        let replacement_sequence = *self.sequence;
        let mut events = self.queue.drain().collect::<Vec<_>>();
        events.retain(|event| event.seq != old_sequence);
        let replacement = events
            .iter_mut()
            .find(|event| event.seq == replacement_sequence)
            .ok_or("replacement primary delivery missing")?;
        replacement.seq = old_sequence;
        self.queue.replace(events);
        Ok(())
    }

    pub(crate) fn cancel_primary_delivery(&mut self) -> Result<(), String> {
        let mut events = self.queue.drain().collect::<Vec<_>>();
        events.retain(|event| {
            !matches!(&event.data,
                crate::simulation::events::GameEventData::HitDelivery {
                    command_index,
                    actor_index,
                    target_index,
                    damage_effect_id,
                    ..
                } if *command_index == self.command_index
                    && *actor_index == self.actor_index
                    && *target_index == self.target_index
                    && self
                        .packet_metadata
                        .is_some_and(|packet| packet.damage_effect_id == *damage_effect_id)
            )
        });
        self.queue.replace(events);
        Ok(())
    }

    /// Deliver an owner-selected child using the admitted action's native coordinate.
    /// Talent projection has already resolved purchased levels and resonance here.
    pub(crate) fn schedule_native_child_packet(
        &mut self,
        damage_effect_id: &str,
        target_index: usize,
        time_us: u64,
    ) -> Result<(), String> {
        self.schedule_native_packet(damage_effect_id, target_index, time_us, None, false)
    }

    pub(crate) fn schedule_scaled_native_child(
        &mut self,
        effect: &str,
        target: usize,
        time: u64,
        doubled: bool,
    ) -> Result<(), String> {
        self.schedule_native_packet(effect, target, time, None, doubled)
    }

    fn schedule_native_packet(
        &mut self,
        damage_effect_id: &str,
        target_index: usize,
        time_us: u64,
        energy_group_id: Option<&str>,
        doubled: bool,
    ) -> Result<(), String> {
        let array_index = self
            .native_array_index
            .ok_or("native child requires an admitted native coordinate")?;
        if target_index >= self.request.targets.len() || time_us < self.time_us {
            return Err("native child target/time is outside the admitted scope".into());
        }
        let array_index_u32 = array_index
            .try_into()
            .map_err(|_| "native child coordinate exceeds u32")?;
        let entry = resolve_child_packet(
            &self.request.content_id,
            &self.config.character_id,
            self.action_metadata,
            damage_effect_id,
            array_index_u32,
            &self.request.model_assumptions,
        )?;
        let mut atk_rate = entry.base_atk_rate;
        if doubled {
            atk_rate.numerator = atk_rate
                .numerator
                .checked_mul(2)
                .ok_or("native child rate overflow")?;
        }
        let scaling_stat = entry.scaling_stat;
        let mut metadata = crate::simulation::metadata::ActionPacketMetadata {
            action: self.action_metadata.clone(),
            packet: entry.packet.clone(),
            binding: entry.binding.clone(),
        };
        metadata.binding.energy_group_id = energy_group_id.map(str::to_owned);
        increment_sequence(self.sequence)?;
        self.queue.push(ScheduledGameEvent {
            time_us,
            priority: ntesim_core::execution::CausalPriority::HitDelivery,
            seq: *self.sequence,
            data: crate::simulation::events::GameEventData::HitDelivery {
                command_index: self.command_index,
                actor_index: self.actor_index,
                target_index,
                ability_id: self.ability_id.to_owned(),
                damage_effect_id: damage_effect_id.to_owned(),
                atk_rate,
                scaling_stat,
                metadata: Box::new(metadata),
            },
        });
        Ok(())
    }

    /// One energy opportunity across all recipients of a native AoE packet.
    pub(crate) fn schedule_native_aoe_packet(
        &mut self,
        effect: &str,
        time_us: u64,
    ) -> Result<(), String> {
        self.schedule_native_aoe_group(effect, time_us).map(|_| ())
    }

    /// Capped AoE packet aiming at self.target_index first, then other targets up to max_targets.
    pub(crate) fn schedule_native_capped_aoe_packet(
        &mut self,
        effect: &str,
        time_us: u64,
        max_targets: usize,
    ) -> Result<String, String> {
        increment_sequence(self.sequence)?;
        let group = self.next_packet_group()?;
        let mut count = 0;
        if self.target_index < self.targets.len() && self.targets[self.target_index].hp_current > 0
        {
            self.schedule_native_packet(effect, self.target_index, time_us, Some(&group), false)?;
            count += 1;
        }
        for target_index in 0..self.targets.len() {
            if count >= max_targets {
                break;
            }
            if target_index != self.target_index && self.targets[target_index].hp_current > 0 {
                self.schedule_native_packet(effect, target_index, time_us, Some(&group), false)?;
                count += 1;
            }
        }
        Ok(group)
    }

    /// Autonomous capped AoE packet hitting living targets in declaration order up to max_targets.
    pub(crate) fn schedule_native_autonomous_capped_aoe_packet(
        &mut self,
        effect: &str,
        time_us: u64,
        max_targets: usize,
    ) -> Result<String, String> {
        increment_sequence(self.sequence)?;
        let group = self.next_packet_group()?;
        let mut count = 0;
        for target_index in 0..self.targets.len() {
            if count >= max_targets {
                break;
            }
            if self.targets[target_index].hp_current > 0 {
                self.schedule_native_packet(effect, target_index, time_us, Some(&group), false)?;
                count += 1;
            }
        }
        Ok(group)
    }

    /// Return the causal identity even when this finite request has no living recipient.
    pub(crate) fn schedule_native_aoe_group(
        &mut self,
        effect: &str,
        time_us: u64,
    ) -> Result<String, String> {
        increment_sequence(self.sequence)?;
        let group = self.next_packet_group()?;
        for target_index in 0..self.targets.len() {
            if self.targets[target_index].hp_current > 0 {
                self.schedule_native_packet(effect, target_index, time_us, Some(&group), false)?;
            }
        }
        Ok(group)
    }

    /// One projectile contact on its selected living recipient, with an identity
    /// shared by native damage/resource/caster-pause callbacks.
    pub(crate) fn schedule_native_target_group(
        &mut self,
        effect: &str,
        target: usize,
        time_us: u64,
    ) -> Result<String, String> {
        if target >= self.targets.len() {
            return Err("native projectile target is outside the admitted scope".into());
        }
        increment_sequence(self.sequence)?;
        let group = self.next_packet_group()?;
        if self.targets[target].hp_current > 0 {
            self.schedule_native_packet(effect, target, time_us, Some(&group), false)?;
        }
        Ok(group)
    }

    /// Retain the admitted primary's causal sequence and expand its live scope.
    pub(crate) fn scope_primary_aoe(&mut self, effect: &str) -> Result<(), String> {
        self.scope_primary_aoe_group(effect).map(|_| ())
    }

    pub(crate) fn scope_primary_aoe_group(&mut self, effect: &str) -> Result<String, String> {
        self.scope_primary_group(effect, true)
    }

    pub(crate) fn scope_primary_target_group(&mut self, effect: &str) -> Result<String, String> {
        self.scope_primary_group(effect, false)
    }

    fn scope_primary_group(&mut self, effect: &str, all_targets: bool) -> Result<String, String> {
        let sequence = self
            .queue
            .iter()
            .find_map(|event| match &event.data {
                GameEventData::HitDelivery {
                    command_index,
                    actor_index,
                    target_index,
                    damage_effect_id,
                    ..
                } if *command_index == self.command_index
                    && *actor_index == self.actor_index
                    && *target_index == self.target_index
                    && damage_effect_id == effect =>
                {
                    Some(event.seq)
                }
                _ => None,
            })
            .ok_or("native AoE primary delivery missing")?;
        increment_sequence(self.sequence)?;
        let group = self.next_packet_group()?;
        let mut events = self.queue.drain().collect::<Vec<_>>();
        let primary = events
            .iter_mut()
            .find(|event| event.seq == sequence)
            .ok_or("native AoE primary delivery missing")?;
        if let GameEventData::HitDelivery { metadata, .. } = &mut primary.data {
            metadata.binding.energy_group_id = Some(group.clone());
        }
        let primary = primary.clone();
        self.queue.replace(events);
        for target_index in 0..self.targets.len() {
            if target_index != self.target_index
                && all_targets
                && self.targets[target_index].hp_current > 0
            {
                // Expand this packet's recipients without compiling a different
                // coefficient from baseline tables or replacing captured metadata.
                increment_sequence(self.sequence)?;
                let mut recipient = primary.clone();
                recipient.seq = *self.sequence;
                if let GameEventData::HitDelivery {
                    target_index: target,
                    ..
                } = &mut recipient.data
                {
                    *target = target_index;
                }
                self.queue.push(recipient);
            }
        }
        Ok(group)
    }

    fn next_packet_group(&mut self) -> Result<String, String> {
        next_packet_group(self.actor_index, self.actor)
    }

    pub(crate) fn record_hypothesis(&mut self, id: impl AsRef<str>) -> Result<(), String> {
        record_hypothesis_id(self.request, self.execution_assumptions, id)
    }

    #[allow(dead_code)]
    pub(crate) fn record_hypothesis_with_desc(
        &mut self,
        id: impl AsRef<str>,
        description: impl AsRef<str>,
    ) -> Result<(), String> {
        record_hypothesis(self.request, self.execution_assumptions, id, description)
    }
    pub(crate) fn child_packet_metadata(
        &self,
        damage_effect_id: &str,
    ) -> Result<crate::simulation::metadata::ActionPacketMetadata, String> {
        crate::simulation::metadata::resolve_packet_for_action(
            &self.request.content_id,
            ntesim_data::skill_levels::frozen::tables(),
            &self.config.character_id,
            self.action_metadata,
            damage_effect_id,
            &self.request.model_assumptions,
        )
    }
}

impl CastContext<'_> {
    pub(crate) fn refresh_buff(
        &mut self,
        id: &str,
        recipient: ntesim_core::buffs::dispatch::BuffRecipient,
        duration_us: u64,
        stat_modifiers: std::collections::BTreeMap<String, ExactFraction>,
    ) -> Result<(), String> {
        crate::simulation::state::apply_timed_buff(
            self.buffs,
            crate::simulation::state::TimedBuffDeclaration::reset(
                id,
                &self.config.character_id,
                recipient,
                duration_us,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                stat_modifiers,
            ),
            self.actor_index,
            self.world_time_us,
        )
        .map_err(|error| format!("buff refresh failed: {error}"))
    }

    pub(crate) fn refresh_shared_buff_while_alive(
        &mut self,
        id: &str,
        recipient: ntesim_core::buffs::dispatch::BuffRecipient,
        duration_us: u64,
        stat_modifiers: std::collections::BTreeMap<String, ExactFraction>,
    ) -> Result<(), String> {
        let declaration = crate::simulation::state::TimedBuffDeclaration::reset(
            id,
            &self.config.character_id,
            recipient,
            duration_us,
            ntesim_core::buffs::dispatch::BuffStackingScope::Shared,
            stat_modifiers,
        );
        crate::simulation::state::apply_timed_buff_with_lifecycle(
            self.buffs,
            declaration,
            self.actor_index,
            self.world_time_us,
            ntesim_core::buffs::dispatch::BuffSourceLifetime::WhileAlive,
        )
        .map_err(|error| format!("buff refresh failed: {error}"))
    }
}
pub(crate) struct BreakContext<'a> {
    pub world_time_us: u64,
    pub config: &'a GameActorConfig,
    pub actor_index: usize,
    pub target_index: usize,
    pub buffs: &'a ntesim_core::buffs::dispatch::BuffManager,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BuffStats<'a> {
    entries: &'a [(&'a str, ntesim_core::formula::Ratio)],
    map: Option<&'a std::collections::BTreeMap<String, ntesim_core::formula::Ratio>>,
}

impl<'a> BuffStats<'a> {
    #[inline]
    pub fn new(entries: &'a [(&'a str, ntesim_core::formula::Ratio)]) -> Self {
        Self { entries, map: None }
    }

    #[inline]
    pub fn get(&self, key: &str) -> Option<&ntesim_core::formula::Ratio> {
        if let Some(map) = self.map {
            return map.get(key);
        }
        self.entries
            .iter()
            .find_map(|(s, r)| if *s == key { Some(r) } else { None })
    }
}

impl<'a> From<&'a [(&'a str, ntesim_core::formula::Ratio)]> for BuffStats<'a> {
    fn from(entries: &'a [(&'a str, ntesim_core::formula::Ratio)]) -> Self {
        Self { entries, map: None }
    }
}

impl<'a> From<&'a std::collections::BTreeMap<String, ntesim_core::formula::Ratio>>
    for BuffStats<'a>
{
    fn from(map: &'a std::collections::BTreeMap<String, ntesim_core::formula::Ratio>) -> Self {
        Self {
            entries: &[],
            map: Some(map),
        }
    }
}

pub trait BuffStatsView {
    fn get(&self, key: &str) -> Option<&ntesim_core::formula::Ratio>;
}

impl BuffStatsView for std::collections::BTreeMap<String, ntesim_core::formula::Ratio> {
    fn get(&self, key: &str) -> Option<&ntesim_core::formula::Ratio> {
        self.get(key)
    }
}

impl BuffStatsView for BuffStats<'_> {
    fn get(&self, key: &str) -> Option<&ntesim_core::formula::Ratio> {
        self.get(key)
    }
}

pub struct DamageContext<'a> {
    pub(crate) world_time_us: u64,
    /// Live attached DoT stacks held by owners rather than BuffManager statuses.
    pub(crate) foreign_dot_stacks: u32,
    pub(crate) packet_group_id: Option<&'a str>,
    pub(crate) command_index: usize,
    pub(crate) config: &'a GameActorConfig,
    pub(crate) start_fons: Option<u64>,
    pub(crate) ability_id: &'a str,
    pub(crate) action_metadata: &'a crate::simulation::metadata::ActionMetadata,
    pub(crate) target: &'a TargetRuntime,
    pub(crate) actor: &'a ActorRuntime,
    pub(crate) time_us: u64,
    pub(crate) debuff_count: usize,
    pub(crate) damage_effect_id: &'a str,
    pub(crate) actor_index: usize,
    pub(crate) active_actor_index: usize,
    pub(crate) target_count: usize,
    pub(crate) non_boss_target_count: usize,
    pub(crate) target_index: usize,
    pub(crate) buffs: &'a ntesim_core::buffs::dispatch::BuffManager,
    pub(crate) buff_stats: BuffStats<'a>,
    /// Counts of live target statuses owned by this actor. Content owners query
    /// the status IDs they define; shared settlement does not know kit IDs.
    pub(crate) owned_status_counts: &'a std::collections::BTreeMap<String, usize>,
    pub(crate) team_has_three_elements: bool,
}

pub struct RecipientContext<'a> {
    pub source_index: usize,
    pub source_config: &'a GameActorConfig,
    pub source_actor: &'a ActorRuntime,
    pub team_actors: &'a [ActorRuntime],
    pub time_us: u64,
    pub target_count: usize,
    pub packet: Option<&'a crate::simulation::metadata::PacketMetadata>,
    pub request: Option<&'a crate::simulation::GameScenarioRequest>,
}
pub struct HitContext<'a> {
    pub(crate) world_time_us: u64,
    pub(crate) request: &'a GameScenarioRequest,
    pub(crate) config: &'a GameActorConfig,
    pub(crate) actor_index: usize,
    pub(crate) ability_id: &'a str,
    pub(crate) damage_effect_id: &'a str,
    pub(crate) target_index: usize,
    pub(crate) buffs: &'a mut ntesim_core::buffs::dispatch::BuffManager,
}
pub(crate) struct CallbackContext<'a> {
    pub(crate) state: &'a mut GameSimulatorState,
    pub(crate) request: &'a GameScenarioRequest,
    pub(crate) queue: &'a mut BinaryHeap,
    pub(crate) sequence: &'a mut u64,
    pub(crate) trace: &'a mut Vec<GameEventReceipt>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IncomingDamageShare {
    pub(crate) actor_index: usize,
    pub(crate) amount: u64,
    pub(crate) hp_floor: u64,
}

pub(crate) struct IncomingDamagePlan {
    pub(crate) primary: IncomingDamageShare,
    pub(crate) redirected: Vec<IncomingDamageShare>,
}
impl IncomingDamagePlan {
    pub(crate) fn single(actor_index: usize, amount: u64) -> Self {
        Self {
            primary: IncomingDamageShare {
                actor_index,
                amount,
                hp_floor: 0,
            },
            redirected: Vec::new(),
        }
    }
}

pub(crate) struct IncomingHpContext<'a> {
    pub request: &'a GameScenarioRequest,
    pub state: &'a mut GameSimulatorState,
    pub queue: &'a mut BinaryHeap,
    pub sequence: &'a mut u64,
    pub trace: &'a mut Vec<GameEventReceipt>,
    pub actor_index: usize,
    pub hp_before: u64,
    pub residual: u64,
}

pub(crate) struct IncomingHpSettlement {
    pub actual_hp_loss: u64,
    pub overflow: u64,
    pub defeated: bool,
}

pub(crate) struct FinishHitContext<'a> {
    pub request: &'a GameScenarioRequest,
    pub state: &'a GameSimulatorState,
    pub parent_actor: usize,
    pub target_index: usize,
    pub applied_hp_loss: u64,
    pub metadata: &'a crate::simulation::metadata::ActionPacketMetadata,
}

pub(crate) struct FinishDamage {
    pub owner: usize,
    pub hypothesis: HypothesisId,
    pub metadata: crate::simulation::metadata::ActionPacketMetadata,
    pub effect_id: &'static str,
    pub ability_id: &'static str,
}

impl CallbackContext<'_> {
    pub(crate) fn settle_healing(
        &mut self,
        source_actor: usize,
        recipient_actor: usize,
        base_heal: ExactFraction,
    ) -> Result<HealingSettlement, String> {
        let (healing_bonus, incoming_healing_bonus) =
            self.live_healing_bonuses(source_actor, recipient_actor)?;
        let gross = ntesim_core::combat::exact::exact_healing(
            base_heal,
            healing_bonus,
            incoming_healing_bonus,
        )?
        .round_u64()?;
        let recipient = self
            .state
            .actors
            .get(recipient_actor)
            .ok_or("healing recipient actor is out of bounds")?;
        let before_hp = recipient.hp;
        if before_hp == 0 {
            self.state
                .healing_receipts
                .push(crate::simulation::GameHealingReceipt {
                    time_us: self.state.current_time_us,
                    source_actor_index: source_actor,
                    recipient_actor_index: recipient_actor,
                    base: base_heal,
                    outgoing_bonus: healing_bonus,
                    incoming_bonus: incoming_healing_bonus,
                    gross,
                    applied: 0,
                    excess: gross,
                    hp_before: before_hp,
                    hp_after: before_hp,
                });
            return Ok(HealingSettlement {
                gross,
                applied: 0,
                excess: gross,
                before_hp,
                after_hp: before_hp,
            });
        }
        let recipient_hp_max = crate::simulation::state::effective_hp_max(
            &self.state.buffs,
            recipient.hp_max_base,
            &recipient.hp_max_static_bonus,
            recipient_actor,
            recipient.hp_max,
        )?;
        let recipient = self
            .state
            .actors
            .get_mut(recipient_actor)
            .ok_or("healing recipient actor is out of bounds")?;
        let mut vitals = ntesim_core::state::VitalState::new(
            before_hp,
            recipient_hp_max,
            ntesim_core::state::ShieldOrder::OldestFirst,
        )
        .map_err(|error| format!("healing recipient vitals failed: {error:?}"))?;
        let settlement = vitals
            .heal(gross)
            .map_err(|error| format!("healing settlement failed: {error:?}"))?;
        recipient.hp = vitals.hp();
        let after_hp = recipient.hp;
        self.state
            .healing_receipts
            .push(crate::simulation::GameHealingReceipt {
                time_us: self.state.current_time_us,
                source_actor_index: source_actor,
                recipient_actor_index: recipient_actor,
                base: base_heal,
                outgoing_bonus: healing_bonus,
                incoming_bonus: incoming_healing_bonus,
                gross,
                applied: settlement.applied,
                excess: settlement.excess,
                hp_before: before_hp,
                hp_after: after_hp,
            });
        Ok(HealingSettlement {
            gross,
            applied: settlement.applied,
            excess: settlement.excess,
            before_hp,
            after_hp,
        })
    }

    pub(crate) fn live_shield_bonus(
        &mut self,
        source_actor: usize,
    ) -> Result<ExactFraction, String> {
        let living_sources = self
            .state
            .actors
            .iter()
            .enumerate()
            .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
            .collect::<std::collections::BTreeSet<_>>();
        let modifiers = self
            .state
            .buffs
            .get_stat_modifiers_for_actor_with_living_sources(
                source_actor,
                self.state.active_actor_index,
                self.state.mode_clock.relative_us(),
                &living_sources,
            )
            .map_err(|error| format!("shield modifier query failed: {error:?}"))?;
        let bonus = modifiers
            .get("shield_bonus")
            .copied()
            .map(ExactFraction::from_ratio)
            .transpose()?;
        Ok(bonus.unwrap_or(ExactFraction {
            numerator: 0,
            denominator: 1,
        }))
    }

    pub(crate) fn outgoing_shield_bonus(
        &mut self,
        source_actor: usize,
        recipient_actor: usize,
    ) -> Result<ExactFraction, String> {
        let source = self
            .state
            .actors
            .get(source_actor)
            .ok_or("shield granter actor is out of bounds")?;
        let max_hp = crate::simulation::state::effective_hp_max(
            &self.state.buffs,
            source.hp_max_base,
            &source.hp_max_static_bonus,
            source_actor,
            source.hp_max,
        )?;
        let rainman_bonus = crate::weapons::yuren::grant_time_shield_bonus(
            self.request
                .actors
                .get(source_actor)
                .ok_or("shield granter config is out of bounds")?,
            source.hp,
            max_hp,
            source_actor,
            recipient_actor,
        )?;
        let static_bonus =
            ExactFraction::from_ratio(self.request.actors[source_actor].static_stats.shield_bonus)?;
        static_bonus
            .add(self.live_shield_bonus(source_actor)?)?
            .add(rainman_bonus)
    }

    pub(crate) fn live_healing_bonuses(
        &mut self,
        source_actor: usize,
        recipient_actor: usize,
    ) -> Result<(ExactFraction, ExactFraction), String> {
        let living_sources = self
            .state
            .actors
            .iter()
            .enumerate()
            .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
            .collect::<std::collections::BTreeSet<_>>();
        let query = |actor_index, key| -> Result<ExactFraction, String> {
            let modifiers = self
                .state
                .buffs
                .get_stat_modifiers_for_actor_with_living_sources(
                    actor_index,
                    self.state.active_actor_index,
                    self.state.mode_clock.relative_us(),
                    &living_sources,
                )
                .map_err(|error| format!("healing modifier query failed: {error:?}"))?;
            modifiers
                .get(key)
                .copied()
                .map(ExactFraction::from_ratio)
                .transpose()
                .map(|value| {
                    value.unwrap_or(ExactFraction {
                        numerator: 0,
                        denominator: 1,
                    })
                })
        };
        let outgoing = query(source_actor, "healing_bonus")?.add(ExactFraction::from_ratio(
            self.request.actors[source_actor].static_stats.healing_bonus,
        )?)?;
        let incoming = query(recipient_actor, "incoming_healing_bonus")?;
        if !outgoing.is_zero() || !incoming.is_zero() {
            record_hypothesis_id(
                self.request,
                &mut self.state.execution_model_assumptions,
                HEALING_MODIFIERS_HYPOTHESIS_ID,
            )?;
        }
        Ok((outgoing, incoming))
    }
}
pub struct ActorHitContext<'a> {
    pub(crate) world_time_us: u64,
    pub(crate) hit_index: usize,
    pub(crate) packet_group_id: Option<&'a str>,
    pub(crate) authored_command_index: Option<usize>,
    pub(crate) autonomous_event_index: Option<usize>,
    pub(crate) target_alive: bool,
    pub(crate) control_applications: &'a mut Vec<crate::simulation::GameControlApplication>,
    pub(crate) applied_hp_loss: u64,
    pub(crate) active_actor_index: usize,
    pub(crate) command_index: usize,
    pub(crate) request: &'a GameScenarioRequest,
    pub(crate) execution_assumptions: &'a mut std::collections::BTreeMap<String, String>,
    pub(crate) action_metadata: &'a crate::simulation::metadata::ActionMetadata,
    pub(crate) target_level: u32,
    pub(crate) queue: &'a mut BinaryHeap,
    pub(crate) sequence: &'a mut u64,
    pub(crate) config: &'a GameActorConfig,
    pub(crate) actor_index: usize,
    pub(crate) ability_id: &'a str,
    pub(crate) damage_effect_id: &'a str,
    pub(crate) time_us: u64,
    pub(crate) target_index: usize,
    pub(crate) target_airborne: bool,
    pub(crate) buffs: &'a mut ntesim_core::buffs::dispatch::BuffManager,
    pub(crate) actor: &'a mut ActorRuntime,
    pub(crate) trace: &'a mut Vec<GameEventReceipt>,
}

impl DamageContext<'_> {
    pub(crate) fn has_status(&self, id: &str) -> bool {
        crate::simulation::state::target_has_status(
            self.buffs,
            self.actor_index,
            &self.config.character_id,
            self.target_index,
            id,
        )
    }
}

impl HitContext<'_> {
    pub(crate) fn refresh_status(
        &mut self,
        id: &str,
        duration_us: u64,
        reductions: std::collections::BTreeMap<String, ExactFraction>,
    ) -> Result<(), String> {
        crate::simulation::state::apply_target_status(
            self.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor: self.actor_index,
                source: self.config.character_id.clone(),
                target_index: self.target_index,
                id: id.into(),
                now_us: self.world_time_us,
                duration_us,
                reductions,
            },
        )
    }
}

impl ActorHitContext<'_> {
    pub(crate) fn refresh_buff(
        &mut self,
        id: &str,
        recipient: ntesim_core::buffs::dispatch::BuffRecipient,
        duration_us: u64,
        stat_modifiers: std::collections::BTreeMap<String, ExactFraction>,
    ) -> Result<(), String> {
        crate::simulation::state::apply_timed_buff(
            self.buffs,
            crate::simulation::state::TimedBuffDeclaration::reset(
                id,
                &self.config.character_id,
                recipient,
                duration_us,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                stat_modifiers,
            ),
            self.actor_index,
            self.world_time_us,
        )
    }

    pub(crate) fn refresh_status(
        &mut self,
        id: &str,
        duration_us: u64,
        reductions: std::collections::BTreeMap<String, ExactFraction>,
    ) -> Result<(), String> {
        crate::simulation::state::apply_target_status(
            self.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor: self.actor_index,
                source: self.config.character_id.clone(),
                target_index: self.target_index,
                id: id.into(),
                now_us: self.world_time_us,
                duration_us,
                reductions,
            },
        )
    }
}
