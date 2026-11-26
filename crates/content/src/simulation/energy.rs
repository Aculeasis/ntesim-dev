use super::{
    ExactFraction, GameActorConfig, GameEnergyFanoutReceipt, GameEnergyFanoutRecipient,
    GameEnergyGrantReceipt, GameEnergyReceipt, GameEnergySource, GameEventReceipt,
    GameScenarioRequest, OrdinaryHitEnergyBasis, metadata::ActionPacketMetadata,
    state::ActorRuntime,
};
use crate::HypothesisId;
use crate::runtime_api::DirectPacketEnergyPolicy;
mod relations;

pub(crate) struct EnergySnapshot {
    current: u64,
    granted: u64,
    overflow: u64,
    charge_remainder: ExactFraction,
}

impl EnergySnapshot {
    pub(crate) fn capture(actor: &ActorRuntime) -> Self {
        Self {
            current: actor.energy.current(),
            granted: actor.granted_energy,
            overflow: actor.overflow_energy,
            charge_remainder: actor.charge_remainder,
        }
    }

    pub(crate) fn receipt(
        self,
        actor: &ActorRuntime,
        source: GameEnergySource,
        source_actor_index: usize,
        recipient_actor_index: usize,
        charge_basis: Option<ExactFraction>,
    ) -> Result<GameEnergyReceipt, String> {
        let gross = actor
            .granted_energy
            .checked_sub(self.granted)
            .ok_or("energy receipt grant counter decreased")?;
        let overflow = actor
            .overflow_energy
            .checked_sub(self.overflow)
            .ok_or("energy receipt overflow counter decreased")?;
        let retained = actor
            .energy
            .current()
            .checked_sub(self.current)
            .ok_or("energy receipt pool decreased during grant")?;
        if retained.checked_add(overflow) != Some(gross) {
            return Err("energy receipt does not conserve gross grant".into());
        }
        Ok(GameEnergyReceipt::Grant(GameEnergyGrantReceipt {
            source,
            source_actor_index,
            recipient_actor_index,
            energy_before: self.current,
            gross,
            retained,
            overflow,
            energy_after: actor.energy.current(),
            charge_remainder_before: (!self.charge_remainder.is_zero())
                .then_some(self.charge_remainder),
            charge_remainder_after: (!actor.charge_remainder.is_zero())
                .then_some(actor.charge_remainder),
            charge_basis,
            hit_basis: None,
        }))
    }
}

pub(crate) struct OrdinaryHitEnergyOwner {
    character_id: &'static str,
    display_name: &'static str,
    abilities: &'static [&'static str],
    pub(crate) hypothesis_id: HypothesisId,
    source: GameEnergySource,
    event_kind: &'static str,
}

pub(crate) const ORDINARY_HIT_ENERGY_OWNERS: &[OrdinaryHitEnergyOwner] = &[
    OrdinaryHitEnergyOwner {
        character_id: "character:1076",
        display_name: "Shinku",
        abilities: &["GA_Shinku_Melee", "GA_Shinku_Skill"],
        hypothesis_id: SHINKU_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::ShinkuHit,
        event_kind: "shinku_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1004",
        display_name: "Lacrimosa",
        abilities: &["GA_Lacrimosa_Melee", "GA_Lacrimosa_Skill"],
        hypothesis_id: LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::LacrimosaHit,
        event_kind: "lacrimosa_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1033",
        display_name: "Adler",
        abilities: &["GA_Adler_Melee", "GA_Adler_Skill"],
        hypothesis_id: ADLER_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::AdlerHit,
        event_kind: "adler_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1019",
        display_name: "Mint",
        abilities: &["GA_Mint019_Melee", "GA_Mint019_Skill"],
        hypothesis_id: MINT_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::MintHit,
        event_kind: "mint_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1003",
        display_name: "Sakiri",
        abilities: &["GA_Sagiri_Melee", "GA_Sagiri_Skill"],
        hypothesis_id: SAKIRI_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::SakiriHit,
        event_kind: "sakiri_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1046",
        display_name: "Zero",
        abilities: &["GA_Female046_Melee", "GA_Female046_Skill"],
        hypothesis_id: ZERO_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::ZeroHit,
        event_kind: "zero_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1075",
        display_name: "Iroi",
        abilities: &["GA_Oneiroi_Melee", "GA_Oneiroi_Skill"],
        hypothesis_id: IROI_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::IroiHit,
        event_kind: "iroi_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1010",
        display_name: "Nanally",
        abilities: &["GA_Nanally_Melee", "GA_Nanally_Skill"],
        hypothesis_id: NANALLY_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::NanallyHit,
        event_kind: "nanally_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1071",
        display_name: "Chaos",
        abilities: &["GA_Chaos071_Melee", "GA_Chaos071_Skill"],
        hypothesis_id: CHAOS_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::ChaosHit,
        event_kind: "chaos_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1021",
        display_name: "Edgar",
        abilities: &["GA_Edgar_Melee", "GA_Edgar_Skill"],
        hypothesis_id: EDGAR_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::EdgarHit,
        event_kind: "edgar_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1052",
        display_name: "Hotori",
        abilities: &["GA_Jin_Melee", "GA_Jin_Skill"],
        hypothesis_id: HOTORI_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::HotoriHit,
        event_kind: "hotori_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1054",
        display_name: "Daffodil",
        abilities: &["GA_Daffodill_Melee", "GA_Daffodill_Skill"],
        hypothesis_id: DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::DaffodilHit,
        event_kind: "daffodil_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1072",
        display_name: "Linko",
        abilities: &["GA_Radio072_Melee", "GA_Radio072_Skill"],
        hypothesis_id: LINKO_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::LinkoHit,
        event_kind: "linko_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1008",
        display_name: "Skia",
        abilities: &["GA_Skia_Melee", "GA_Skia_Skill"],
        hypothesis_id: SKIA_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::SkiaHit,
        event_kind: "skia_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1025",
        display_name: "Hathor",
        abilities: &["GA_Hathor_Melee", "GA_Hathor_Skill"],
        hypothesis_id: HATHOR_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::HathorHit,
        event_kind: "hathor_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1042",
        display_name: "Blackbird",
        abilities: &[
            "GA_BlackBird_Melee",
            "GA_BlackBird_Skill",
            "GA_BlackBird_Evolve_Melee",
            "GA_BlackBird_Evolve_Skill",
        ],
        hypothesis_id: BLACKBIRD_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::BlackbirdHit,
        event_kind: "blackbird_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1070",
        display_name: "Aurelia",
        abilities: &["GA_Mitsuki_Melee", "GA_Mitsuki_Skill"],
        hypothesis_id: AURELIA_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::AureliaHit,
        event_kind: "aurelia_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1020",
        display_name: "Haniel",
        abilities: &["GA_Haniel_Melee", "GA_Haniel_Skill"],
        hypothesis_id: HANIEL_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::HanielHit,
        event_kind: "haniel_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1055",
        display_name: "Jiuyuan",
        abilities: &["GA_Kuhara_Melee", "GA_Kuhara_Skill"],
        hypothesis_id: JIUYUAN_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::JiuyuanHit,
        event_kind: "jiuyuan_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1023",
        display_name: "Baicang",
        abilities: &["GA_Cang_Melee", "GA_Cang_Skill"],
        hypothesis_id: BAICANG_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::BaicangHit,
        event_kind: "baicang_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1039",
        display_name: "Fadia",
        abilities: &["GA_Fadia_Melee"],
        hypothesis_id: FADIA_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::FadiaHit,
        event_kind: "fadia_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1036",
        display_name: "Zankou",
        abilities: &["GA_Zankou_Melee"],
        hypothesis_id: ZANKOU_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::ZankouHit,
        event_kind: "zankou_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1073",
        display_name: "Chiz",
        abilities: &["GA_Chiichan073_Melee"],
        hypothesis_id: CHIZ_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::ChizHit,
        event_kind: "chiz_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1057",
        display_name: "Akane Rin",
        abilities: &["GA_Akane_Melee", "GA_Akane_Skill"],
        hypothesis_id: AKANE_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::AkaneHit,
        event_kind: "akane_ordinary_hit_energy_grant",
    },
    OrdinaryHitEnergyOwner {
        character_id: "character:1090",
        display_name: "Elyms",
        abilities: &[
            "GA_Elyms_Melee",
            "GA_Elyms_Skill",
            "GA_Elyms_UltraSkill",
            "GA_Elyms_UltraSkillB",
        ],
        hypothesis_id: ELYMS_HIT_ENERGY_HYPOTHESIS_ID,
        source: GameEnergySource::ElymsHit,
        event_kind: "elyms_ordinary_hit_energy_grant",
    },
];

pub(crate) const AKANE_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AkaneOrdinaryHitEnergyV1;
pub(crate) const ELYMS_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ElymsOrdinaryHitEnergyV1;

pub(crate) const SHINKU_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ShinkuOrdinaryHitEnergyV1;
pub(crate) const LACRIMOSA_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::LacrimosaOrdinaryHitEnergyV1;
pub(crate) const ADLER_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AdlerOrdinaryHitEnergyV1;
pub(crate) const CHIZ_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ChizOrdinaryHitEnergyV1;
pub(crate) const FADIA_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::FadiaOrdinaryHitEnergyV1;
pub(crate) const ZANKOU_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ZankouOrdinaryHitEnergyV1;
pub(crate) const JIUYUAN_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::JiuyuanOrdinaryHitEnergyV1;
pub(crate) const BAICANG_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::BaicangOrdinaryHitEnergyV1;
pub(crate) const MINT_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::MintOrdinaryHitEnergyV1;
pub(crate) const SAKIRI_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::SakiriOrdinaryHitEnergyV1;
pub(crate) const ZERO_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ZeroOrdinaryHitEnergyV1;
pub(crate) const IROI_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::IroiOrdinaryHitEnergyV1;
pub(crate) const NANALLY_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::NanallyOrdinaryHitEnergyV1;
pub(crate) const CHAOS_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ChaosOrdinaryHitEnergyV1;
pub(crate) const CHAOS_QTE_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::ChaosQteDirectPacketEnergyV1;
pub(crate) const EDGAR_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::EdgarOrdinaryHitEnergyV1;
pub(crate) const HOTORI_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HotoriOrdinaryHitEnergyV1;
pub(crate) const DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::DaffodilOrdinaryHitEnergyV1;
pub(crate) const LINKO_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::LinkoOrdinaryHitEnergyV1;
pub(crate) const SKIA_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::SkiaOrdinaryHitEnergyV1;
pub(crate) const HATHOR_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HathorOrdinaryHitEnergyV1;
pub(crate) const HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HathorBranchAirattackDirectPacketEnergyV1;
pub(crate) const DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::DaffodilBranchEntryDirectPacketEnergyV1;
pub(crate) const AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AureliaAirattackEvadebranchDirectPacketEnergyV1;
pub(crate) const SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::SakiriDirectPlungePacketEnergyV1;
pub(crate) const BLACKBIRD_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::BlackbirdOrdinaryHitEnergyV1;
pub(crate) const AURELIA_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::AureliaOrdinaryHitEnergyV1;
pub(crate) const HANIEL_HIT_ENERGY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::HanielOrdinaryHitEnergyV1;

pub(crate) fn notify_active_actor_energy_gain(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    recipient: usize,
    positive_grant: bool,
) -> Result<(), String> {
    if positive_grant
        && recipient == ctx.state.active_actor_index
        && ctx.state.actors[recipient].hp > 0
    {
        #[cfg(test)]
        {
            ctx.state.active_actor_energy_gain_observations = ctx
                .state
                .active_actor_energy_gain_observations
                .saturating_add(1);
        }
        crate::registry::on_active_actor_energy_gain(ctx, recipient)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn ordinary_hit_grants(
    request: &GameScenarioRequest,
    actor_configs: &[GameActorConfig],
    actors: &mut [ActorRuntime],
    active_actor_index: usize,
    command_index: usize,
    hit_index: usize,
    actor_index: usize,
    target_index: usize,
    applied_hp_loss: u64,
    metadata: &ActionPacketMetadata,
    hit_assumptions: &mut std::collections::BTreeMap<String, String>,
    trace: &mut Vec<GameEventReceipt>,
    time_us: u64,
) -> Result<bool, String> {
    let owner = match ORDINARY_HIT_ENERGY_OWNERS
        .iter()
        .find(|owner| actor_configs[actor_index].character_id == owner.character_id)
    {
        Some(owner) => owner,
        None => return Ok(false),
    };
    let direct_packet_policy = direct_packet_energy_policy(
        request,
        &actor_configs[actor_index].character_id,
        &metadata.action.ability_id,
        &metadata.packet.damage_effect_id,
    );
    let energy_group = metadata.binding.energy_group_id.as_deref();
    if (target_index != request.aim_target(command_index) && energy_group.is_none())
        || energy_group.is_some_and(|id| actors[actor_index].packet_energy_groups.contains(id))
        || (applied_hp_loss == 0
            && !direct_packet_policy.is_some_and(|policy| {
                policy.trigger == crate::runtime_api::PacketEnergyTrigger::EffectApplication
            }))
        || (direct_packet_energy_is_unmodeled(
            &actor_configs[actor_index].character_id,
            &metadata.action.ability_id,
            &metadata.packet.damage_effect_id,
        ) && direct_packet_policy.is_none())
        || (!owner
            .abilities
            .contains(&metadata.action.ability_id.as_str())
            && direct_packet_policy.is_none())
        || !matches!(
            metadata.binding.kind,
            super::metadata::ActionPacketBindingKind::NativeDirect
                | super::metadata::ActionPacketBindingKind::OwnerChild
        )
    {
        return Ok(false);
    }
    let hypothesis_id = direct_packet_policy
        .map(|policy| policy.hypothesis_id)
        .unwrap_or(owner.hypothesis_id);
    crate::runtime_api::record_hypothesis_id(request, hit_assumptions, hypothesis_id)?;

    let row = ntesim_data::native_action_metadata::resolve_damage_effect(
        &request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &metadata.packet.damage_effect_id,
    )?;
    let charge_value = row
        .raw_row
        .as_ref()
        .and_then(|row| row.get("ChargeAdd"))
        .ok_or_else(|| {
            format!(
                "native {} packet {} has no ChargeAdd scalar",
                owner.display_name, metadata.packet.damage_effect_id
            )
        })?;
    let native_charge = ntesim_data::native::parse_native_scalar(charge_value)?;
    let base = ExactFraction::new(native_charge.numerator(), native_charge.denominator())?;
    if base.is_zero() {
        return Ok(false);
    }

    let source_element = actor_element(&actor_configs[actor_index].character_id)?;
    let event_kind = if let Some(policy) = direct_packet_policy {
        policy.event_kind
    } else {
        owner.event_kind
    };
    let mut active_gained = false;
    let mut grants = Vec::new();
    for recipient_index in 0..actors.len() {
        if actors[recipient_index].hp == 0 {
            continue;
        }
        let coefficient = if recipient_index == actor_index {
            relations::DISTRIBUTION.owner
        } else {
            let recipient_element = actor_element(&actor_configs[recipient_index].character_id)?;
            relations::peer(source_element, recipient_element)
        };
        let gross = base.mul(coefficient)?;
        if recipient_index == active_actor_index && !gross.is_zero() {
            active_gained = true;
        }
        let snapshot = EnergySnapshot::capture(&actors[recipient_index]);
        actors[recipient_index].grant_fractional_energy(gross)?;
        let receipt = snapshot.receipt(
            &actors[recipient_index],
            owner.source,
            actor_index,
            recipient_index,
            Some(gross),
        )?;
        if let GameEnergyReceipt::Grant(mut grant) = receipt {
            grant.hit_basis = Some(OrdinaryHitEnergyBasis {
                base,
                distribution: coefficient,
            });
            grants.push(grant);
        }
    }
    if grants.len() == 1 {
        let grant = grants.into_iter().next().expect("single grant");
        trace.push(GameEventReceipt {
            time_us,
            phase: "HitDelivery".into(),
            event_kind: event_kind.into(),
            active_actor: active_actor_index,
            hit_index: Some(hit_index),
            from_actor: Some(actor_index),
            to_actor: Some(grant.recipient_actor_index),
            energy_change: Some(GameEnergyReceipt::Grant(grant)),
            energy_fanout: None,
            data: None,
        });
    } else if !grants.is_empty() {
        let recipients = grants
            .into_iter()
            .map(|grant| GameEnergyFanoutRecipient {
                recipient_actor_index: grant.recipient_actor_index,
                distribution: grant.hit_basis.map(|basis| basis.distribution),
                energy_before: grant.energy_before,
                gross: grant.gross,
                retained: grant.retained,
                overflow: grant.overflow,
                energy_after: grant.energy_after,
                charge_remainder_before: grant.charge_remainder_before,
                charge_remainder_after: grant.charge_remainder_after,
                charge_basis: grant.charge_basis,
            })
            .collect();
        trace.push(GameEventReceipt {
            time_us,
            phase: "HitDelivery".into(),
            event_kind: event_kind.into(),
            active_actor: active_actor_index,
            hit_index: Some(hit_index),
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: Some(GameEnergyFanoutReceipt {
                source: owner.source,
                source_actor_index: actor_index,
                base: Some(base),
                recipients,
            }),
            data: None,
        });
    }
    if let Some(group) = energy_group {
        actors[actor_index]
            .packet_energy_groups
            .insert(group.to_owned());
    }
    Ok(active_gained)
}

fn direct_packet_energy_policy(
    request: &GameScenarioRequest,
    character_id: &str,
    ability_id: &str,
    effect_id: &str,
) -> Option<DirectPacketEnergyPolicy> {
    if let Some(policy) =
        crate::registry::owned_direct_packet_energy(request, character_id, ability_id, effect_id)
    {
        return Some(policy);
    }
    if matches!(
        (character_id, ability_id, effect_id),
        (
            "character:1071",
            "GA_Chaos071_QTE",
            "GE_Player_Chaos_QTE1_Damage"
        )
    ) {
        return Some(DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: CHAOS_QTE_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID,
            event_kind: "chaos_qte_direct_packet_energy_grant",
        });
    }
    if matches!(
        (character_id, ability_id, effect_id),
        (
            "character:1025",
            "GA_Hathor_Melee",
            "GE_Player_Hathor_Branch_Damage" | "GE_Player_Hathor_AirAttack_Damage"
        )
    ) {
        return Some(DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID,
            event_kind: "hathor_direct_packet_energy_grant",
        });
    }
    if matches!(
        (character_id, ability_id, effect_id),
        (
            "character:1054",
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Branch1_Damage"
                | "GE_Player_Daffodill_Branch2_Damage"
                | "GE_Player_Daffodill_Branch3_Damage"
                | "GE_Player_Daffodill_Branch4_Damage"
                | "GE_Player_Daffodill_Branch5_Damage"
                | "GE_Player_Daffodill_Branch6_Damage"
                | "GE_Player_Daffodill_Branch7_Damage"
                | "GE_Player_Daffodill_EntryAttack1_Damage"
                | "GE_Player_Daffodill_EntryAttack2_Damage"
                | "GE_Player_Daffodill_EntryAttack4_Damage"
        )
    ) {
        return Some(DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID,
            event_kind: "daffodil_direct_packet_energy_grant",
        });
    }
    if matches!(
        (character_id, ability_id, effect_id),
        (
            "character:1070",
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_AirAttack_Damage" | "GE_Player_Mitsuki_EvadeBranch_Damage"
        ) | (
            "character:1003",
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_AirAttack1_Damage"
        )
    ) {
        let (hypothesis_id, event_kind) = match character_id {
            "character:1070" => (
                AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID,
                "aurelia_direct_packet_energy_grant",
            ),
            "character:1003" => (
                SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID,
                "sakiri_direct_packet_energy_grant",
            ),
            _ => unreachable!("direct packet match restricts owner identity"),
        };
        return Some(DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id,
            event_kind,
        });
    }
    None
}

fn direct_packet_energy_is_unmodeled(
    character_id: &str,
    ability_id: &str,
    effect_id: &str,
) -> bool {
    matches!(
        (character_id, ability_id, effect_id),
        (
            "character:1025",
            "GA_Hathor_Melee",
            "GE_Player_Hathor_Branch_Damage" | "GE_Player_Hathor_AirAttack_Damage"
        ) | (
            "character:1054",
            "GA_Daffodill_Melee",
            "GE_Player_Daffodill_Branch1_Damage"
                | "GE_Player_Daffodill_Branch2_Damage"
                | "GE_Player_Daffodill_Branch3_Damage"
                | "GE_Player_Daffodill_Branch4_Damage"
                | "GE_Player_Daffodill_Branch5_Damage"
                | "GE_Player_Daffodill_Branch6_Damage"
                | "GE_Player_Daffodill_Branch7_Damage"
                | "GE_Player_Daffodill_EntryAttack1_Damage"
                | "GE_Player_Daffodill_EntryAttack2_Damage"
                | "GE_Player_Daffodill_EntryAttack4_Damage"
        ) | (
            "character:1070",
            "GA_Mitsuki_Melee",
            "GE_Player_Mitsuki_AirAttack_Damage" | "GE_Player_Mitsuki_EvadeBranch_Damage"
        ) | (
            "character:1003",
            "GA_Sagiri_Melee",
            "GE_Player_Sagiri_AirAttack1_Damage"
        )
    )
}

fn actor_element(character_id: &str) -> Result<ntesim_data::character::Element, String> {
    Ok(ntesim_data::character::definition(character_id)?.element)
}
