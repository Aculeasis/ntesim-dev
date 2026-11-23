use super::kits::*;
use super::traits::{CharacterKit, SetEffect, WeaponPassive};
use crate::runtime_api::{
    ActorHitContext, BreakContext, BreakModifiers, CastContext, DamageContext, HitContext,
    HitModifiers, RecipientContext,
};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::state::{ActorRuntime, TargetRuntime};
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest};
pub(crate) fn lookup_weapon_passive(weapon_id: &str) -> Option<&'static dyn WeaponPassive> {
    let _ = weapon_id;
    None
}

pub(crate) fn lookup_set_effect(set_id: &str) -> Option<&'static dyn SetEffect> {
    let _ = set_id;
    None
}

pub(crate) fn weapon_passive_on_cast(
    weapon_id: &str,
    ctx: &mut CastContext<'_>,
) -> Result<(), String> {
    lookup_weapon_passive(weapon_id).map_or(Ok(()), |p| p.on_cast(ctx))
}

pub(crate) fn weapon_passive_on_hit(
    weapon_id: &str,
    ctx: &mut CastContext<'_>,
    critical: bool,
) -> Result<(), String> {
    lookup_weapon_passive(weapon_id).map_or(Ok(()), |p| p.on_hit(ctx, critical))
}

pub(crate) fn weapon_passive_damage_bonus(
    weapon_id: &str,
    ctx: &DamageContext<'_>,
    bonus: HitModifiers,
) -> Result<HitModifiers, String> {
    match lookup_weapon_passive(weapon_id) {
        Some(p) => p.damage_bonus(ctx, bonus),
        None => Ok(bonus),
    }
}

pub(crate) fn set_effect_on_cast(set_id: &str, ctx: &mut CastContext<'_>) -> Result<(), String> {
    lookup_set_effect(set_id).map_or(Ok(()), |s| s.on_cast(ctx))
}

pub(crate) fn set_effect_on_hit(
    set_id: &str,
    ctx: &mut CastContext<'_>,
    critical: bool,
) -> Result<(), String> {
    lookup_set_effect(set_id).map_or(Ok(()), |s| s.on_hit(ctx, critical))
}

pub(crate) fn set_effect_damage_bonus(
    set_id: &str,
    ctx: &DamageContext<'_>,
    bonus: HitModifiers,
) -> Result<HitModifiers, String> {
    match lookup_set_effect(set_id) {
        Some(s) => s.damage_bonus(ctx, bonus),
        None => Ok(bonus),
    }
}

pub(crate) fn weapon_passive_id(weapon_id: &str) -> Option<&'static str> {
    lookup_weapon_passive(weapon_id).map(|p| p.weapon_id())
}

pub(crate) fn set_effect_id(set_id: &str) -> Option<&'static str> {
    lookup_set_effect(set_id).map(|s| s.set_id())
}

static ADLER_KIT: AdlerKit = AdlerKit;
static AURELIA_KIT: AureliaKit = AureliaKit;
static BAICANG_KIT: BaicangKit = BaicangKit;
static BLACKBIRD_KIT: BlackbirdKit = BlackbirdKit;
static CHAOS_KIT: ChaosKit = ChaosKit;
static CHIZ_KIT: ChizKit = ChizKit;
static DAFFODIL_KIT: DaffodilKit = DaffodilKit;
static EDGAR_KIT: EdgarKit = EdgarKit;
static FADIA_KIT: FadiaKit = FadiaKit;
static HANIEL_KIT: HanielKit = HanielKit;
static HATHOR_KIT: HathorKit = HathorKit;
static HOTORI_KIT: HotoriKit = HotoriKit;
static IROI_KIT: IroiKit = IroiKit;
static JIUYUAN_KIT: JiuyuanKit = JiuyuanKit;
static LACRIMOSA_KIT: LacrimosaKit = LacrimosaKit;
static LINKO_KIT: LinkoKit = LinkoKit;
static MINT_KIT: MintKit = MintKit;
static NANALLY_KIT: NanallyKit = NanallyKit;
static SAKIRI_KIT: SakiriKit = SakiriKit;
static SHINKU_KIT: ShinkuKit = ShinkuKit;
static SKIA_KIT: SkiaKit = SkiaKit;
static ZANKOU_KIT: ZankouKit = ZankouKit;
static ZERO_KIT: ZeroKit = ZeroKit;

pub(crate) fn lookup_kit(character_id: &str) -> Option<&'static dyn CharacterKit> {
    match character_id {
        "character:1033" => Some(&ADLER_KIT),
        "character:1070" => Some(&AURELIA_KIT),
        "character:1023" => Some(&BAICANG_KIT),
        "character:1042" => Some(&BLACKBIRD_KIT),
        "character:1071" => Some(&CHAOS_KIT),
        "character:1073" => Some(&CHIZ_KIT),
        "character:1054" => Some(&DAFFODIL_KIT),
        "character:1021" => Some(&EDGAR_KIT),
        "character:1039" => Some(&FADIA_KIT),
        "character:1020" => Some(&HANIEL_KIT),
        "character:1025" => Some(&HATHOR_KIT),
        "character:1052" => Some(&HOTORI_KIT),
        "character:1075" => Some(&IROI_KIT),
        "character:1055" => Some(&JIUYUAN_KIT),
        "character:1004" => Some(&LACRIMOSA_KIT),
        "character:1072" => Some(&LINKO_KIT),
        "character:1019" => Some(&MINT_KIT),
        "character:1010" => Some(&NANALLY_KIT),
        "character:1003" => Some(&SAKIRI_KIT),
        "character:1076" => Some(&SHINKU_KIT),
        "character:1008" => Some(&SKIA_KIT),
        "character:1036" => Some(&ZANKOU_KIT),
        "character:1046" => Some(&ZERO_KIT),
        _ => None,
    }
}

pub(crate) fn skill_gear_trigger_on_finish(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
) -> bool {
    lookup_kit(&config.character_id).is_some_and(|kit| kit.skill_gear_trigger_on_finish(request))
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let Some(kit) = lookup_kit(&ctx.config.character_id) else {
        return Ok(());
    };
    if let Some(profile) = kit.single_packet_support()
        && ctx.ability_id == profile.entry.ability_id
        && matches!(
            ctx.request.actions.get(ctx.command_index),
            Some(crate::simulation::GameActionCommand::SwitchActor { .. })
        )
    {
        ctx.record_hypothesis(profile.entry.dispatch_model)?;
        ctx.scope_primary_aoe(profile.entry.damage_effect_id)?;
        ctx.actor.single_support_contact = Some(ctx.command_index);
        if ctx.actor_index == ctx.active_actor_index {
            ctx.actor.single_support_start_us = Some(ctx.time_us);
            ctx.actor.switch_action_locked_until_us = ctx
                .time_us
                .checked_add(profile.switch_gate_us)
                .ok_or("Support switch gate overflow")?;
        }
    }
    kit.on_cast(ctx)
}

pub(crate) fn utility_invocation(
    config: &GameActorConfig,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<crate::runtime_api::UtilityInvocation> {
    lookup_kit(&config.character_id).and_then(|kit| kit.utility_invocation(actor, ability, now))
}

pub(crate) fn source_command(
    config: &GameActorConfig,
    actor: &ActorRuntime,
    template: &crate::simulation::GameActionCommand,
) -> Result<crate::simulation::GameActionCommand, String> {
    let mut command = template.clone();
    if let Some(kit) = lookup_kit(&config.character_id) {
        kit.select_source_command(actor, &mut command)?;
    }
    Ok(command)
}

pub(crate) fn entry_action(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    old: usize,
    new: usize,
    qualified_support: bool,
) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
    lookup_kit(&request.actors[new].character_id).map_or(Ok(None), |kit| {
        kit.entry_action(request, state, old, new, qualified_support)
    })
}

// Ability provenance precedes packet resolution; packet legality follows it.
// Keeping both stages preserves the caller's diagnostic and mutation order.
pub(crate) fn normal_action_override(
    config: &GameActorConfig,
    actor: &ActorRuntime,
) -> Option<crate::runtime_api::NormalActionOverride> {
    if let Some(value) =
        lookup_kit(&config.character_id).and_then(|kit| kit.normal_action_override(actor))
    {
        return Some(value);
    }
    actor
        .aiming
        .then(|| crate::characters::player_inputs::aim_packet(&config.character_id))
        .flatten()
        .map(|packet| crate::runtime_api::NormalActionOverride {
            packet,
            role: None,
            model: crate::characters::player_inputs::AIM_HYPOTHESIS_ID,
            timing: None,
        })
}

pub(crate) fn held_normal_override(
    request: &GameScenarioRequest,
    command_index: usize,
    actor: &ActorRuntime,
    time_us: u64,
) -> Result<Option<crate::runtime_api::NormalActionOverride>, String> {
    request
        .hold_inputs
        .get(&command_index)
        .map_or(Ok(None), |input| match &actor.character {
            crate::characters::RuntimeState::Zankou(_) => {
                crate::characters::zankou::normal::held(actor, *input, time_us)
            }
            crate::characters::RuntimeState::Edgar(_)
            | crate::characters::RuntimeState::Mint(_)
            | crate::characters::RuntimeState::Haniel(_)
            | crate::characters::RuntimeState::Hathor(_)
            | crate::characters::RuntimeState::Iroi(_) => Ok(None),
            crate::characters::RuntimeState::Daffodil(_) => {
                crate::characters::daffodil::held::replacement(request, actor, time_us)
            }
            crate::characters::RuntimeState::Nanally(_) => {
                crate::characters::nanally::clock::held::replacement(actor, *input, time_us)
            }
            crate::characters::RuntimeState::Chiz(_) => {
                crate::characters::chiz::barren::replacement(request, actor, command_index, time_us)
            }
            crate::characters::RuntimeState::Blackbird(_) => {
                crate::characters::blackbird::branches::replacement(request, actor, *input, time_us)
            }
            crate::characters::RuntimeState::Chaos(_) => {
                crate::characters::chaos::verdict::replacement(request, actor, *input, time_us)
            }
            _ => crate::characters::baicang::hold::replacement(actor, *input, time_us),
        })
}

pub(crate) fn held_recovery_override(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    command_index: usize,
    now: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    if matches!(
        &actor.character,
        crate::characters::RuntimeState::Daffodil(_)
    ) {
        return Ok(crate::characters::daffodil::held::recovery(
            request,
            actor,
            command_index,
            now,
        ));
    }
    if matches!(&actor.character, crate::characters::RuntimeState::Chiz(_)) {
        return Ok(crate::characters::chiz::barren::recovery(
            request,
            actor,
            command_index,
        ));
    }
    if matches!(&actor.character, crate::characters::RuntimeState::Chaos(_)) {
        return crate::characters::chaos::verdict::recovery_override(
            request,
            actor,
            command_index,
            now,
        );
    }
    if matches!(
        &actor.character,
        crate::characters::RuntimeState::Blackbird(_)
    ) {
        crate::characters::blackbird::branches::recovery_override(
            request,
            actor,
            command_index,
            now,
        )
    } else {
        Ok(None)
    }
}

pub(crate) fn held_input_window(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    now: u64,
) -> Result<u64, String> {
    match &actor.character {
        crate::characters::RuntimeState::Zankou(_) => Ok(actor.locked_until_us.max(now)),
        crate::characters::RuntimeState::Blackbird(_) => {
            crate::characters::blackbird::branches::input_window(request, actor, now)
        }
        crate::characters::RuntimeState::Chaos(_) => {
            crate::characters::chaos::cancel::input_window(request, actor, now)
        }
        crate::characters::RuntimeState::Daffodil(_) => {
            crate::characters::daffodil::held::input_window(request, actor, now)
        }
        _ => Err("Unsupported owner hold input window.".into()),
    }
}

pub(crate) fn validate_back_dash_binding(
    request: &crate::simulation::GameScenarioRequest,
    config: &GameActorConfig,
) -> Result<(), String> {
    let model = match config.character_id.as_str() {
        "character:1019" => crate::characters::mint::clock::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1036" => crate::characters::zankou::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1025" => crate::characters::hathor::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1020" => crate::characters::haniel::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1039" => crate::characters::fadia::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1021" => lookup_kit(&config.character_id)
            .unwrap()
            .validate_player_input(request, crate::simulation::PlayerInput::Dash)?,
        "character:1054" => crate::characters::daffodil::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1073" => crate::characters::chiz::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1071" => crate::characters::chaos::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        "character:1042" => crate::characters::blackbird::secondary::validate_input(
            request,
            crate::simulation::PlayerInput::Dash,
        )?,
        _ => return Err("no owner Back Dodge binding".into()),
    };
    model.ok_or("Back Dodge requires the owner's accepted secondary model")?;
    Ok(())
}

pub(crate) fn on_committed_player_input(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    command_index: usize,
    actor: usize,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    if ctx.request.actors[actor].character_id == "character:1036" {
        crate::characters::zankou::secondary::committed(ctx, actor, command_index, input)?;
    }
    if ctx.request.actors[actor].character_id == "character:1075"
        && crate::characters::iroi::clock::accepted(ctx.request)
    {
        crate::characters::iroi::clock::stop(
            &mut ctx.state.actors[actor],
            ctx.queue,
            ctx.state.current_time_us,
        )?;
    }
    if ctx.request.actors[actor].character_id == "character:1019" {
        crate::characters::mint::clock::controls::committed(ctx, actor, command_index, input)?;
    }
    if ctx.request.actors[actor].character_id == "character:1010" {
        crate::characters::nanally::clock::input(ctx, actor, command_index, input)?;
    }
    if ctx.request.actors[actor].character_id == "character:1020" {
        crate::characters::haniel::clock::committed(ctx, actor, command_index, input)?;
    }
    if ctx.request.actors[actor].character_id == "character:1025" {
        crate::characters::hathor::clock::controls::committed(ctx, actor, command_index, input)?;
    }
    if ctx.request.actors[actor].character_id == "character:1039" {
        crate::characters::fadia::clock::controls::committed(ctx, actor, command_index, input)?;
    }
    if ctx.request.actors[actor].character_id == "character:1021" {
        crate::characters::edgar::clock::committed(ctx, actor, command_index, input)?;
    }
    if matches!(
        &ctx.state.actors[actor].character,
        crate::characters::RuntimeState::Daffodil(_)
    ) {
        crate::characters::daffodil::clock::cancel::committed(ctx, actor, input)?;
        crate::characters::daffodil::secondary::committed(ctx, actor, command_index, input)?;
    }
    if matches!(
        &ctx.state.actors[actor].character,
        crate::characters::RuntimeState::Blackbird(_)
    ) {
        crate::characters::blackbird::clock::on_input(ctx, actor, command_index, input)?;
    }
    if matches!(
        &ctx.state.actors[actor].character,
        crate::characters::RuntimeState::Chaos(_)
    ) {
        crate::characters::chaos::secondary::committed_input(ctx, actor, command_index, input)?;
    }
    if matches!(
        &ctx.state.actors[actor].character,
        crate::characters::RuntimeState::Chiz(_)
    ) {
        crate::characters::chiz::cancel::committed_input(ctx, actor, input)?;
        crate::characters::chiz::secondary::committed(ctx, actor, command_index, input)?;
    }
    Ok(())
}

pub(crate) fn recovery_lock_override(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &ActorRuntime,
    ability_id: &str,
    now_us: u64,
) -> Result<Option<(u64, crate::HypothesisId)>, String> {
    if ability_id == "switch_actor"
        && actor.single_support_start_us == Some(actor.last_action_start_us)
        && let Some(profile) =
            lookup_kit(&config.character_id).and_then(|kit| kit.single_packet_support())
    {
        return Ok(Some((
            actor.switch_action_locked_until_us,
            profile.entry.dispatch_model,
        )));
    }
    if config.character_id == "character:1071"
        && let Some(value) =
            crate::characters::chaos::cancel::recovery(request, actor, ability_id, now_us)?
    {
        return Ok(Some(value));
    }
    if config.character_id == "character:1042"
        && let Some(value) = crate::characters::blackbird::cancel::recovery_override(
            request, actor, ability_id, now_us,
        )?
    {
        return Ok(Some(value));
    }
    Ok(lookup_kit(&config.character_id)
        .and_then(|kit| kit.recovery_lock_override(request, actor, ability_id, now_us)))
}

pub(crate) fn validate_parry(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
) -> Result<(), String> {
    if config.character_id == "character:1019" {
        return crate::characters::mint::clock::controls::validate_parry(request);
    }
    if config.character_id == "character:1025" {
        return if crate::characters::hathor::secondary::accepted(request)
            && request.combat_stamina.is_some()
        {
            Ok(())
        } else {
            Err("Hathor parry requires secondary_actions_v1 and paid Dodge stamina".into())
        };
    }
    if config.character_id == "character:1020" {
        return if crate::characters::haniel::secondary::accepted(request)
            && request.combat_stamina.is_some()
        {
            Ok(())
        } else {
            Err("Haniel parry requires full_kit_v1 and explicit paid Dodge stamina".into())
        };
    }
    if config.character_id == "character:1054" {
        return crate::characters::daffodil::phantom::validate_parry(request, config);
    }
    crate::characters::blackbird::combat::validate(
        request,
        config,
        crate::HypothesisId::BlackbirdExplicitParryV1,
    )
}

pub(crate) fn parry_actor(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
    event: usize,
) -> Result<crate::simulation::GameActorParryReceipt, String> {
    if ctx.request.actors[actor].character_id == "character:1019" {
        return crate::characters::mint::clock::controls::parry(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1025" {
        return crate::characters::hathor::secondary::parry(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1020" {
        return crate::characters::haniel::secondary::parry(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1054" {
        return crate::characters::daffodil::phantom::parry(ctx, actor, event);
    }
    crate::characters::blackbird::combat::parry(ctx, actor, event)
}

pub(crate) fn validate_combat_transition(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
) -> Result<(), String> {
    if config.character_id == "character:1036" {
        return Ok(());
    }
    if config.character_id == "character:1010" {
        return crate::characters::nanally::combat::validate(request);
    }
    if config.character_id == "character:1025" {
        return if crate::characters::hathor::combat::accepted(request) {
            Ok(())
        } else {
            Err("Hathor combat transitions require combat_lifecycle_v1".into())
        };
    }
    if config.character_id == "character:1020" {
        return if crate::characters::haniel::stats::accepted(request) {
            Ok(())
        } else {
            Err("Haniel combat transitions require nova_extraction_v1".into())
        };
    }
    if config.character_id == "character:1039" {
        return if crate::characters::fadia::effects::accepted(request) {
            Ok(())
        } else {
            Err("Fadia combat transitions require full_kit_v1".into())
        };
    }
    if config.character_id == "character:1021" {
        return if crate::characters::edgar::clock::accepted(request) {
            Ok(())
        } else {
            Err("Edgar combat transitions require full_kit_v1".into())
        };
    }
    if config.character_id == "character:1073" {
        return crate::characters::chiz::combat::validate(request);
    }
    crate::characters::blackbird::combat::validate(
        request,
        config,
        crate::HypothesisId::BlackbirdCombatTransitionsV1,
    )
}

pub(crate) fn combat_transition(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
    event: usize,
) -> Result<crate::simulation::GameActorCombatTransitionReceipt, String> {
    if ctx.request.actors[actor].character_id == "character:1036" {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            crate::characters::zankou::combat_passives::MODEL,
        )?;
        ctx.state.actors[actor]
            .character
            .zankou_mut()?
            .a5_targets
            .clear();
        return Ok(crate::simulation::GameActorCombatTransitionReceipt {
            event_index: event,
            actor_index: actor,
            time_us: ctx.state.current_time_us,
            leaving: ctx.request.combat_transition_events[event].leaving,
            malice_granted: crate::simulation::ExactFraction::new(0, 1)?,
            leave_grant_due_us: None,
            resource_deposits: Default::default(),
        });
    }
    if ctx.request.actors[actor].character_id == "character:1010" {
        return crate::characters::nanally::combat::transition(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1025" {
        return crate::characters::hathor::combat::transition(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1020" {
        return crate::characters::haniel::stats::transition(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1039" {
        return crate::characters::fadia::stats::transition(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1021" {
        return crate::characters::edgar::effects::transition(ctx, actor, event);
    }
    if ctx.request.actors[actor].character_id == "character:1073" {
        return crate::characters::chiz::combat::transition(ctx, actor, event);
    }
    crate::characters::blackbird::combat::transition(ctx, actor, event)
}

pub(crate) fn validate_interrupt(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
) -> Result<(), String> {
    lookup_kit(&config.character_id)
        .ok_or("unsupported interruption owner")?
        .validate_interrupt(request)
}

pub(crate) fn interrupt_actor(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    lookup_kit(&ctx.request.actors[actor_index].character_id)
        .ok_or("unsupported interruption owner")?
        .interrupt(ctx, actor_index, event_index)
}

pub(crate) fn critical_dodge_model(
    request: &crate::simulation::GameScenarioRequest,
    config: &GameActorConfig,
) -> Option<crate::HypothesisId> {
    lookup_kit(&config.character_id).and_then(|kit| kit.critical_dodge_model(request))
}

pub(crate) fn on_critical_dodge(ctx: &mut CastContext<'_>) -> Result<(), String> {
    lookup_kit(&ctx.config.character_id).map_or(Ok(()), |kit| kit.on_critical_dodge(ctx))
}

pub(crate) fn validate_ability_admission(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Result<(), String> {
    if config.character_id == "character:1042"
        && ability == "GA_BlackBird_Evolve_Skill"
        && actor
            .character
            .blackbird()?
            .skill_permission_until_us
            .is_some_and(|until| now > until)
    {
        return Err("Blackbird Evolve Skill permission has expired".into());
    }
    lookup_kit(&config.character_id).map_or(Ok(()), |kit| {
        kit.validate_ability_admission(request, config, actor, ability)
    })
}

/// Located owner rule and packet description of one admitted action; recording
/// owners consume this without the simulation layer naming them.
pub(crate) struct RecordedAdmission<'a> {
    pub(crate) owner: usize,
    pub(crate) command_index: usize,
    pub(crate) ability_id: &'a str,
    pub(crate) array_index: usize,
    pub(crate) damage_effect_id: &'a str,
    pub(crate) atk_rate: ntesim_core::formula::Ratio,
    pub(crate) scaling_stat: crate::simulation::NativeScalingStat,
    pub(crate) hit_delay_us: u64,
    pub(crate) metadata: &'a crate::simulation::metadata::ActionPacketMetadata,
}

pub(crate) fn observe_recorded_admission(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    trace: &mut Vec<crate::simulation::GameEventReceipt>,
    origin: crate::simulation::admission::AdmissionOrigin,
    admission: RecordedAdmission<'_>,
) -> Result<(), String> {
    crate::characters::hotori::replay::observe_admission(request, state, trace, origin, admission)
}

pub(crate) fn resource_admission_failure(
    request: &GameScenarioRequest,
    command: usize,
    config: &GameActorConfig,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
    stamina: Option<&crate::simulation::CombatStaminaReport>,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if config.character_id == "character:1025"
        && let Some(failure) = crate::characters::hathor::air::admission(request, command, stamina)
    {
        return Some(failure);
    }
    lookup_kit(&config.character_id)
        .and_then(|kit| kit.resource_admission_failure(request, command, actor, ability, now))
}

pub(crate) fn fixed_crit_encoding(
    effect: &str,
) -> Option<(ExactFraction, ExactFraction, crate::HypothesisId)> {
    crate::characters::packet_semantics::fixed_crit_encoding(effect)
}

pub(crate) fn validate_packet_admission(
    config: &GameActorConfig,
    actor: &ActorRuntime,
    ability: &str,
    packet_effect: &str,
    time_us: u64,
) -> Result<(), String> {
    lookup_kit(&config.character_id).map_or(Ok(()), |kit| {
        kit.validate_packet_admission(config, actor, ability, packet_effect, time_us)
    })
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    bonus: HitModifiers,
) -> Result<HitModifiers, String> {
    match lookup_kit(&ctx.config.character_id) {
        Some(kit) => kit.damage_bonus(ctx, bonus),
        None => Ok(bonus),
    }
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    lookup_kit(&ctx.config.character_id)
        .map_or_else(|| ExactFraction::new(0, 1), |kit| kit.crit_bonus(ctx))
}

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    lookup_kit(&ctx.config.character_id)
        .map_or_else(|| ExactFraction::new(0, 1), |kit| kit.crit_rate_bonus(ctx))
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    lookup_kit(&ctx.config.character_id)
        .map_or_else(|| ExactFraction::new(0, 1), |kit| kit.defense_ignore(ctx))
}

pub(crate) fn break_efficiency_bonus(
    config: &GameActorConfig,
    actor: &ActorRuntime,
    command_index: usize,
    ability: &str,
) -> Result<ExactFraction, String> {
    lookup_kit(&config.character_id).map_or(ExactFraction::new(0, 1), |kit| {
        kit.break_efficiency_bonus(config, actor, command_index, ability)
    })
}

pub(crate) fn break_modifiers(ctx: &BreakContext<'_>) -> Result<BreakModifiers, String> {
    lookup_kit(&ctx.config.character_id)
        .map_or_else(BreakModifiers::neutral, |kit| kit.break_modifiers(ctx))
}

pub(crate) fn cast_buff_duration(config: &GameActorConfig, ability: &str) -> Option<u64> {
    lookup_kit(&config.character_id).and_then(|kit| kit.cast_buff_duration(config, ability))
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let Some(kit) = lookup_kit(&ctx.config.character_id) else {
        return Ok(());
    };
    if let Some(profile) = kit.single_packet_support()
        && ctx.damage_effect_id == profile.entry.damage_effect_id
        && ctx.actor.single_support_contact == Some(ctx.command_index)
        && ctx.applied_hp_loss > 0
    {
        ctx.actor.single_support_contact = None;
        if ctx.actor_index == ctx.active_actor_index
            && ctx.actor.single_support_start_us == Some(ctx.actor.last_action_start_us)
            && ctx.queue.event_replay().is_none()
        {
            ctx.actor.locked_until_us = ctx
                .actor
                .locked_until_us
                .checked_add(profile.connected_pause_us)
                .ok_or("Support recovery overflow")?;
            ctx.actor.switch_action_locked_until_us = ctx
                .actor
                .switch_action_locked_until_us
                .checked_add(profile.connected_pause_us)
                .ok_or("Support switch gate overflow")?;
        }
        kit.on_single_support_contact(ctx.actor)?;
    }
    kit.on_hit(ctx)
}

pub(crate) fn validate_packet_delivery(
    source_config: &GameActorConfig,
    source_actor: &ActorRuntime,
    packet: &PacketMetadata,
    target_has_scorch: bool,
    scorch_replacement_modeled: bool,
    target_would_be_defeated: bool,
) -> Result<(), String> {
    lookup_kit(&source_config.character_id).map_or(Ok(()), |kit| {
        kit.validate_packet_delivery(
            source_config,
            source_actor,
            packet,
            target_has_scorch,
            scorch_replacement_modeled,
            target_would_be_defeated,
        )
    })
}

pub(crate) fn on_hp_loss_hit(ctx: &mut CastContext<'_>) -> Result<(), String> {
    lookup_kit(&ctx.config.character_id).map_or(Ok(()), |kit| kit.on_hp_loss_hit(ctx))
}

pub(crate) fn packet_cycle_grant(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    effect: &str,
) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
    lookup_kit(&config.character_id).map_or(Ok(None), |kit| kit.packet_cycle_grant(request, effect))
}

pub(crate) fn validate_player_input(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    input: crate::simulation::PlayerInput,
) -> Result<Option<crate::HypothesisId>, String> {
    lookup_kit(&config.character_id)
        .map_or(Ok(None), |kit| kit.validate_player_input(request, input))
}

pub(crate) fn validate_hit_target(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    ability_id: &str,
    packet: &PacketMetadata,
    target: &TargetRuntime,
) -> Result<(), String> {
    lookup_kit(&config.character_id).map_or(Ok(()), |kit| {
        kit.validate_hit_target(request, config, ability_id, packet, target)
    })
}

pub(crate) fn has_before_hit(character_id: &str) -> bool {
    lookup_kit(character_id).is_some_and(|kit| kit.has_before_hit())
}

pub(crate) fn before_hit(ctx: &mut HitContext<'_>) -> Result<(), String> {
    lookup_kit(&ctx.config.character_id).map_or(Ok(()), |kit| kit.before_hit(ctx))
}

pub(crate) fn validate_action_inputs(
    request: &GameScenarioRequest,
    packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
) -> Result<(), String> {
    for (&command, input) in &request.hold_inputs {
        if input.reserved_stocks.is_none() {
            continue;
        }
        let Some(crate::simulation::GameActionCommand::InvokeAbility { actor_index, .. }) =
            request.actions.get(command)
        else {
            return Err("hold reserved_stocks requires an InvokeAbility".into());
        };
        lookup_kit(&request.actors[*actor_index].character_id)
            .ok_or("hold reserved_stocks has no supported owner")?
            .validate_hold_stock_selection(input)?;
    }
    crate::characters::blackbird::form::validate_inputs(request)?;
    crate::characters::chaos::form::validate(request)?;
    crate::characters::chaos::support::validate(request)?;
    crate::characters::chaos::secondary::validate(request)?;
    crate::characters::chaos::remora::validate_inputs(request)?;
    crate::characters::chaos::warrant::validate(request)?;
    crate::characters::chaos::verdict::validate(request, packets)?;
    crate::characters::baicang::cancel::validate(request)?;
    crate::characters::blackbird::branches::validate_inputs(request, packets)?;
    crate::characters::chiz::pressed::held::validate(request, packets)?;
    crate::characters::chiz::cancel::validate(request)?;
    crate::characters::chiz::market::validate(request)?;
    crate::characters::chiz::secondary::validate(request)?;
    crate::characters::chiz::barren::validate(request)?;
    crate::characters::daffodil::clock::cancel::validate(request)?;
    crate::characters::mint::clock::validate(request)?;
    crate::characters::daffodil::secondary::validate(request)?;
    crate::characters::daffodil::held::validate(request)?;
    crate::characters::daffodil::phantom::validate(request)?;
    crate::characters::nanally::plunge::validate(request)?;
    crate::characters::nanally::clock::held::validate(request)?;
    crate::characters::edgar::clock::validate(request)?;
    crate::characters::fadia::clock::validate(request)?;
    crate::characters::haniel::clock::validate(request)?;
    crate::characters::hathor::clock::validate(request)?;
    crate::characters::hathor::gaze::validate(request)?;
    crate::characters::hathor::held::validate(request)?;
    crate::characters::hathor::branch::validate(request)?;
    crate::characters::hathor::burst::validate(request)?;
    crate::characters::hathor::cyclone::validate(request)?;
    crate::characters::hathor::combat::validate(request)?;
    crate::characters::hathor::secondary::validate(request)?;
    crate::characters::hathor::air::validate(request)?;
    if !crate::characters::hathor::held::accepted(request)
        && request.incoming_dummy_sources.keys().any(|index| {
            request.incoming_damage_basis.get(index)
                != Some(&crate::simulation::GameIncomingDamageBasis::BeforeDefense)
        })
    {
        return Err("incoming dummy source requires a supported contact observer or explicit before_defense basis".into());
    }
    crate::characters::baicang::hold::validate_inputs(request, packets)
}

pub(crate) fn validate_plunge_binding(
    character: &str,
    ctx: &crate::runtime_api::PlungeInputContext<'_>,
) -> Result<(), String> {
    lookup_kit(character)
        .ok_or("no supported owner plunge binding")?
        .validate_plunge(ctx)
}

pub(crate) fn on_player_input(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    lookup_kit(&config.character_id)
        .map_or(Ok(()), |kit| kit.on_player_input(request, actor, input))
}

pub(crate) fn validate_player_input_state(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &ActorRuntime,
) -> Result<(), String> {
    lookup_kit(&config.character_id).map_or(Ok(()), |kit| {
        kit.validate_player_input_state(request, actor)
    })
}

pub(crate) fn recipient_modifiers(
    ctx: &RecipientContext<'_>,
    value: HitModifiers,
) -> Result<HitModifiers, String> {
    match lookup_kit(&ctx.source_config.character_id) {
        Some(kit) => kit.recipient_modifiers(ctx, value),
        None => Ok(value),
    }
}

pub(crate) fn on_target_defeated(
    config: &GameActorConfig,
    actor: &mut crate::simulation::state::ActorRuntime,
    time_us: u64,
) -> Result<(), String> {
    match &actor.character {
        crate::characters::RuntimeState::Sakiri(_) => {
            crate::characters::sakiri::awakenings::on_team_defeat(config, actor, time_us)
        }
        _ => Ok(()),
    }
}
pub(crate) fn owned_direct_packet_energy(
    request: &GameScenarioRequest,
    character: &str,
    ability: &str,
    packet: &str,
) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
    if character == "character:1025" && !crate::characters::hathor::secondary::accepted(request) {
        return None;
    }
    if character == "character:1020" && !crate::characters::haniel::secondary::accepted(request) {
        return None;
    }
    if character == "character:1021" && !crate::characters::edgar::clock::accepted(request) {
        return None;
    }
    if character == "character:1039" && !crate::characters::fadia::effects::accepted(request) {
        return None;
    }
    let kit = lookup_kit(character)?;
    kit.direct_packet_energy(ability, packet)
        .or_else(|| {
            let support = &kit.single_packet_support()?.entry;
            (ability == support.ability_id && packet == support.damage_effect_id).then_some(
                crate::runtime_api::DirectPacketEnergyPolicy {
                    trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                    hypothesis_id: support.dispatch_model,
                    event_kind: "support_hit_energy_grant",
                },
            )
        })
        .filter(|policy| {
            policy.trigger == crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss
                || request
                    .model_assumptions
                    .get(policy.hypothesis_id.as_str())
                    .map(String::as_str)
                    == Some(policy.hypothesis_id.description())
        })
}

pub(crate) fn cancel_owned_packet_after_defeat(config: &GameActorConfig, effect_id: &str) -> bool {
    if config.character_id == "character:1025" && effect_id.starts_with("GE_Player_Hathor_Melee") {
        return true;
    }
    if config.character_id == "character:1071"
        && matches!(
            effect_id,
            crate::characters::chaos::secondary::RIPOSTE | crate::characters::chaos::remora::PACKET
        )
    {
        return true;
    }
    (config.character_id == "character:1042"
        && crate::characters::blackbird::secondary::cancel_after_defeat(effect_id))
        || (config.character_id == "character:1036"
            && crate::characters::zankou::vile_ash::is_vile_ash_packet(effect_id))
        || (config.character_id == "character:1055"
            && crate::characters::jiuyuan::pact::is_packet(effect_id))
        || (config.character_id == "character:1010"
            && crate::characters::nanally::riposte::cancel_after_defeat(effect_id))
        || (config.character_id == "character:1003"
            && crate::characters::sakiri::skill::cancel_after_defeat(effect_id))
        || (config.character_id == "character:1019"
            && crate::characters::mint::riposte::cancel_after_defeat(effect_id))
        || (config.character_id == "character:1046"
            && crate::characters::zero::skill::cancel_after_defeat(effect_id))
}

pub(crate) fn on_zero_periodic_delivery(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor_index: usize,
    target_index: usize,
    effect_id: &str,
) -> Result<(), String> {
    if ctx.request.actors[actor_index].character_id == "character:1036" {
        crate::characters::zankou::vile_ash::continue_zero_packet(
            ctx,
            actor_index,
            target_index,
            effect_id,
        )?;
    }
    Ok(())
}

pub(crate) fn scorch_enhancement(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<Option<crate::runtime_api::ScorchEnhancement>, String> {
    crate::characters::zankou::scorch::policy(request, state, assumptions)
}

pub(crate) fn stain_enhancement(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<Option<crate::runtime_api::StainEnhancement>, String> {
    crate::characters::blackbird::stain::policy(request, state, assumptions)
}

pub(crate) fn remora_enhancement(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<Option<crate::runtime_api::RemoraEnhancement>, String> {
    crate::characters::hathor::remora::policy(request, state, assumptions)
}

pub(crate) fn charge_reaction_bonus(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<u64, String> {
    crate::characters::chiz::passives::charge_bonus(request, state, assumptions)
}

pub(crate) fn charge_reaction_override(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<Option<u64>, String> {
    crate::characters::edgar::effects::charge(request, state, assumptions)
}

pub(crate) fn on_charge_grant(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    recipient: usize,
) -> Result<(), String> {
    crate::characters::zero::callback::on_charge_grant(ctx, recipient)
}

pub(crate) fn validate_discord_extensions(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    if request
        .actors
        .iter()
        .enumerate()
        .any(|(index, actor)| actor.character_id == "character:1054" && state.actors[index].hp > 0)
    {
        crate::runtime_api::record_hypothesis_id(
            request,
            assumptions,
            crate::HypothesisId::DaffodilDiscordCapReductionV1,
        )?;
    }
    Ok(())
}

pub(crate) fn on_discord(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    source_actor: usize,
    target_index: usize,
    source_command_key: usize,
) -> Result<bool, String> {
    let daffodil_broke = crate::characters::daffodil::discord::on_discord(
        ctx.request,
        ctx.state,
        ctx.queue,
        ctx.sequence,
        ctx.trace,
        source_actor,
        target_index,
    )?;
    crate::characters::lacrimosa::skill::on_discord(ctx, target_index, source_command_key)?;
    Ok(daffodil_broke)
}

pub(crate) fn validate_remora_extensions(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    crate::characters::chaos::remora::validate(request, state, assumptions)
}
pub(crate) fn on_remora_created(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target: usize,
    generation: u64,
) -> Result<bool, String> {
    crate::characters::chaos::remora::created(ctx, target, generation)
}
pub(crate) fn on_remora_removed(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target: usize,
    generation: u64,
    elapsed: u64,
    command: usize,
) -> Result<(), String> {
    crate::characters::chaos::remora::removed(ctx, target, generation, elapsed, command)
}
pub(crate) fn has_remora_listener(
    state: &crate::simulation::state::GameSimulatorState,
    target: usize,
    generation: u64,
) -> bool {
    crate::characters::chaos::remora::has_listener(state, target, generation)
}

pub(crate) fn scorch_dot_eligible(
    request: &crate::simulation::GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    owner_actor_index: usize,
    damage_effect_id: &str,
    semantic_tags: &std::collections::BTreeSet<String>,
) -> bool {
    crate::characters::zankou::scorch::dot_eligible(
        request,
        state,
        owner_actor_index,
        damage_effect_id,
        semantic_tags,
    )
}

pub(crate) fn blossom_bud_copy_eligible(request: &crate::simulation::GameScenarioRequest) -> bool {
    crate::characters::iroi::bud_copy::iroi_has_b2(request).is_some()
}

pub(crate) fn validate_nova_extensions(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    target: usize,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<bool, String> {
    crate::characters::haniel::stats::validate_nova(request, state, target)?;
    crate::characters::blackbird::nova::validate(request, state, assumptions)
}

pub(crate) fn on_nova_created(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target: usize,
    generation: u64,
    command: usize,
    until_us: u64,
    reapply: bool,
) -> Result<(), String> {
    crate::characters::blackbird::nova::create(
        ctx.request,
        ctx.state,
        target,
        generation,
        command,
        until_us,
        reapply,
    )
}

pub(crate) fn observe_applied_hp(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    target: usize,
    command: usize,
    metadata: &crate::simulation::metadata::ActionPacketMetadata,
    applied_hp: u64,
) -> Result<(), String> {
    crate::characters::blackbird::nova::observe(
        request, state, target, command, metadata, applied_hp,
    )?;
    crate::characters::jiuyuan::pact::observe_damage(
        state,
        target,
        &metadata.packet.damage_effect_id,
        applied_hp,
    );
    Ok(())
}

pub(crate) fn on_nova_removed(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target_index: usize,
    generation: u64,
    command_index: usize,
) -> Result<Vec<crate::runtime_api::NovaEndEffect>, String> {
    let mut effects = Vec::new();
    for owner in 0..ctx.request.actors.len() {
        match ctx.request.actors[owner].character_id.as_str() {
            "character:1020" => {
                crate::characters::haniel::stats::nova_end(ctx, owner, target_index, generation)?
            }
            "character:1039" => {
                crate::characters::fadia::stats::nova_end(ctx, owner, target_index)?
            }
            "character:1042" => {
                if let Some(effect) = crate::characters::blackbird::nova::end(
                    ctx.request,
                    ctx.state,
                    owner,
                    target_index,
                    generation,
                )? {
                    effects.push(effect);
                }
            }
            "character:1070" => {
                crate::characters::aurelia::harmonics::schedule_harmonics_if_eligible(
                    ctx,
                    owner,
                    target_index,
                    command_index,
                )?
            }
            _ => {}
        }
    }
    Ok(effects)
}

pub(crate) fn initialize_actor(
    config: &GameActorConfig,
    actor: &mut crate::simulation::state::ActorRuntime,
    request: &GameScenarioRequest,
) -> Result<(), String> {
    if let Some(kit) = lookup_kit(&config.character_id) {
        kit.initialize(request, config, actor)?;
    }
    match &actor.character {
        crate::characters::RuntimeState::Adler(_) => {
            let state = actor.character.adler_mut()?;
            state.karma_stacks = crate::characters::adler::awakenings::initial_karma(
                config.initial_resources.karma,
            )?;
            Ok(())
        }
        crate::characters::RuntimeState::Edgar(_) => {
            let state = actor.character.edgar_mut()?;
            state.keys_of_truth =
                crate::characters::edgar::awakenings::initial_keys(config.initial_resources.keys)?;
            Ok(())
        }
        crate::characters::RuntimeState::Haniel(_) => {
            crate::characters::haniel::resource::initialize(config, actor)
        }
        crate::characters::RuntimeState::Zankou(_) => {
            let state = actor.character.zankou_mut()?;
            state.kindling_stacks = crate::characters::zankou::passives::initial_kindling(
                config.initial_resources.kindling,
            )?;
            crate::characters::zankou::passives::initialize_actor(actor)
        }
        crate::characters::RuntimeState::Hotori(_) => {
            crate::characters::hotori::initialize(config, actor)
        }
        crate::characters::RuntimeState::Chiz(_) => {
            crate::characters::chiz::resource::initialize(config, actor)?;
            crate::characters::chiz::finance::initialize(request, config, actor)?;
            crate::characters::chiz::combat::deposit(config, actor, 0).map(|_| ())
        }
        crate::characters::RuntimeState::Iroi(_) => {
            crate::characters::iroi::resource::initialize(config, actor)
        }
        _ => Ok(()),
    }
}
pub(crate) fn initialize_team_effect(
    config: &GameActorConfig,
    actor_index: usize,
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    request: &crate::simulation::GameScenarioRequest,
    assumptions: &mut std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    crate::characters::iroi::awakenings::record_initial_hypotheses(config, request, assumptions)?;
    crate::characters::blackbird::malice::record_initial(config, request, assumptions)?;
    crate::characters::chaos::resources::record_initial(config, request, assumptions)?;
    if config.character_id == "character:1073" {
        crate::runtime_api::record_hypothesis_id(
            request,
            assumptions,
            crate::characters::chiz::resource::MODEL,
        )?;
        if crate::characters::chiz::finance::accepted(request) {
            crate::runtime_api::record_hypothesis_id(
                request,
                assumptions,
                crate::characters::chiz::finance::MODEL,
            )?;
        }
        if crate::characters::chiz::pressed::accepted(request) {
            crate::runtime_api::record_hypothesis_id(
                request,
                assumptions,
                crate::characters::chiz::pressed::MODEL,
            )?;
        }
    }
    crate::characters::zankou::passives::initialize_team(
        config,
        actor_index,
        buffs,
        request,
        assumptions,
    )?;
    if config.character_id == "character:1052" {
        crate::runtime_api::record_hypothesis_id(
            request,
            assumptions,
            crate::characters::hotori::timepiece::MODEL,
        )?;
    }
    crate::characters::zankou::skill::initialize_team_effect(config, actor_index, buffs)?;
    crate::characters::adler::righteous_heart::initialize_team(
        config,
        actor_index,
        buffs,
        request,
        assumptions,
    )?;
    crate::characters::iroi::awakenings::initialize_team_effect(config, actor_index, buffs)
}
pub(crate) fn observe_team_direct_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
    target: usize,
    metadata: &crate::simulation::metadata::ActionPacketMetadata,
) -> Result<(), String> {
    crate::characters::haniel::hootie::observe(ctx, actor, target, metadata)
}

pub(crate) fn observe_party_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    hit_index: usize,
    hits: &[crate::simulation::GameHitReceipt],
) -> Result<(), String> {
    for owner in 0..ctx.request.actors.len() {
        if let Some(kit) = lookup_kit(&ctx.request.actors[owner].character_id) {
            kit.observe_party_hit(ctx, owner, hit_index, hits)?;
        }
    }
    Ok(())
}

pub(crate) fn observe_cycle_hit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    hit_index: usize,
    hits: &[crate::simulation::GameHitReceipt],
) -> Result<(), String> {
    for owner in 0..ctx.request.actors.len() {
        if let Some(kit) = lookup_kit(&ctx.request.actors[owner].character_id) {
            kit.observe_cycle_hit(ctx, owner, hit_index, hits)?;
        }
    }
    Ok(())
}

pub(crate) fn blossom_emission_profile(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
) -> Option<crate::runtime_api::BlossomEmissionProfile> {
    request
        .actors
        .iter()
        .zip(&state.actors)
        .find_map(|(config, actor)| {
            lookup_kit(&config.character_id).and_then(|kit| kit.blossom_emission_profile(actor))
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlossomBudPolicy {
    pub bud_cap: usize,
    pub num_buds: usize,
}

pub(crate) fn blossom_bud_policy(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
) -> BlossomBudPolicy {
    let jiuyuan_b2 = request.actors.iter().enumerate().any(|(i, actor)| {
        actor.character_id == "character:1055"
            && state.actors[i].hp > 0
            && actor.awakening.has_effect(2)
    });
    if jiuyuan_b2 {
        BlossomBudPolicy {
            bud_cap: 6,
            num_buds: 2,
        }
    } else {
        BlossomBudPolicy {
            bud_cap: crate::simulation::reactions::BLOSSOM_PARTY_BUD_CAP,
            num_buds: 1,
        }
    }
}

/// Audit helper for Sakiri Scorch extension (DT_ReactionExtendDataTable row 1003, Buff_Reaction_ZhouAn8_1003).
#[allow(dead_code)]
pub(crate) fn sakiri_scorch_extension_eligible(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
) -> bool {
    request.actors.iter().enumerate().any(|(i, actor)| {
        actor.character_id == "character:1003"
            && state.actors[i].hp > 0
            && actor.awakening.has_effect(2)
    })
}

/// Audit helper for Shinku Reaction AoE (DT_ReactionExtendDataTable row 1076, Buff_Shinku_ReactionAOE).
#[allow(dead_code)]
pub(crate) fn shinku_reaction_aoe_eligible(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
) -> bool {
    request.actors.iter().enumerate().any(|(i, actor)| {
        actor.character_id == "character:1076"
            && state.actors[i].hp > 0
            && actor.awakening.has_effect(2)
    })
}

pub(crate) fn on_actor_defeated(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
) -> Result<(), String> {
    crate::simulation::pause::cancel_cinematic(ctx.state, ctx.queue, actor, None)?;
    if let Some(kit) = lookup_kit(&ctx.request.actors[actor].character_id) {
        kit.on_defeat(&mut ctx.state.actors[actor]);
        kit.on_field_exit(ctx, actor)?;
    }
    crate::characters::jiuyuan::pact::on_defeat(ctx, actor)?;
    crate::characters::hathor::power::defeat(ctx, actor)?;
    crate::characters::hathor::gaze::defeat(ctx, actor)?;
    crate::characters::hotori::timepiece::defeat(ctx, actor)?;
    crate::simulation::reactions::discard_unobserved_remora_deadlines(ctx);
    crate::characters::haniel::hootie::on_defeat(ctx, actor)
}

pub(crate) fn on_field_entry(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
    trigger: crate::simulation::GameCausalTrigger,
) -> Result<(), String> {
    crate::characters::on_field_entry(ctx, actor)?;
    for owner in 0..ctx.request.actors.len() {
        if let Some(kit) = lookup_kit(&ctx.request.actors[owner].character_id) {
            kit.on_party_entry(ctx, owner, actor, trigger)?;
        }
    }
    Ok(())
}

pub(crate) fn observes_vitals(request: &GameScenarioRequest) -> bool {
    request.actors.iter().any(|config| {
        lookup_kit(&config.character_id).is_some_and(|kit| kit.observes_vitals(request, config))
    })
}

pub(crate) fn on_vitals_changed(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    changes: &[crate::simulation::vitals::HpChange],
    trigger: crate::simulation::GameCausalTrigger,
) -> Result<(), String> {
    for owner in 0..ctx.request.actors.len() {
        if let Some(kit) = lookup_kit(&ctx.request.actors[owner].character_id) {
            kit.on_vitals_changed(ctx, owner, changes, trigger)?;
        }
    }
    Ok(())
}

pub(crate) fn on_connected_packet(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    key: usize,
    group: Option<&str>,
    target_index: usize,
    effect: &str,
) -> Result<(), String> {
    lookup_kit(&ctx.request.actors[owner].character_id).map_or(Ok(()), |kit| {
        kit.on_connected_packet(ctx, owner, key, group, target_index, effect)
    })
}

pub(crate) fn source_atk_percent(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &ActorRuntime,
    now: u64,
) -> Result<ExactFraction, String> {
    lookup_kit(&config.character_id).map_or(ExactFraction::new(0, 1), |kit| {
        kit.source_atk_percent(request, config, actor, now)
    })
}

pub(crate) fn on_field_exit(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
) -> Result<(), String> {
    crate::characters::on_field_exit(ctx, actor)?;
    lookup_kit(&ctx.request.actors[actor].character_id)
        .map_or(Ok(()), |kit| kit.on_field_exit(ctx, actor))
}

pub(crate) fn resource_summaries(
    actor: &crate::simulation::state::ActorRuntime,
) -> std::collections::BTreeMap<String, ntesim_core::kernel::state::Pool> {
    match &actor.character {
        crate::characters::RuntimeState::Adler(state) => std::collections::BTreeMap::from([(
            "adler.karma".into(),
            ntesim_core::kernel::state::Pool::new(state.karma_stacks.into(), 20)
                .expect("Adler Karma capped at twenty"),
        )]),
        crate::characters::RuntimeState::Chiz(state) => {
            state
                .pressed
                .as_ref()
                .map_or_else(Default::default, |pressed| {
                    std::collections::BTreeMap::from([(
                        "chiz.skill_stocks".into(),
                        pressed.stocks.clone(),
                    )])
                })
        }
        crate::characters::RuntimeState::Haniel(state) => std::collections::BTreeMap::from([
            (
                "haniel.harmony".into(),
                ntesim_core::kernel::state::Pool::new(u64::from(state.harmony), 3)
                    .expect("Harmony below four"),
            ),
            (
                "haniel.plot_armor".into(),
                ntesim_core::kernel::state::Pool::new(
                    u64::from(state.plot),
                    u64::from(state.plot_cap.max(4)),
                )
                .expect("Plot Armor capped"),
            ),
        ]),
        crate::characters::RuntimeState::Daffodil(state) => std::collections::BTreeMap::from([
            (
                "daffodil.resonance".into(),
                ntesim_core::kernel::state::Pool::new(u64::from(state.resonance_stacks), 2)
                    .expect("Daffodil Resonance is capped at two"),
            ),
            (
                "daffodil.phantom_step_charges".into(),
                ntesim_core::kernel::state::Pool::new(
                    u64::from(state.phantom.charges),
                    u64::from(crate::characters::daffodil::phantom::CHARGE_CAP),
                )
                .expect("Daffodil Phantom Step charges are capped at two"),
            ),
        ]),
        crate::characters::RuntimeState::Edgar(state) => std::collections::BTreeMap::from([(
            "edgar.keys_of_truth".into(),
            ntesim_core::kernel::state::Pool::new(state.keys_of_truth.into(), 3)
                .expect("Edgar keys capped at three"),
        )]),
        crate::characters::RuntimeState::Hathor(state) => {
            state.power.as_ref().map_or_else(Default::default, |power| {
                std::collections::BTreeMap::from([
                    (
                        "hathor.express_delivery_power".into(),
                        ntesim_core::kernel::state::Pool::new(
                            state.express_delivery_power.into(),
                            6,
                        )
                        .expect("Hathor Power bounded by six"),
                    ),
                    (
                        "hathor.a5_earned".into(),
                        ntesim_core::kernel::state::Pool::new(power.earned.into(), 15)
                            .expect("Hathor A5 earned bounded by fifteen"),
                    ),
                ])
            })
        }
        _ => Default::default(),
    }
}

pub(crate) fn fractional_resource_summaries(
    actor: &ActorRuntime,
    now_us: u64,
) -> Result<
    std::collections::BTreeMap<String, ntesim_core::kernel::state::ResourcePoolSummary>,
    String,
> {
    let summaries = match &actor.character {
        crate::characters::RuntimeState::Hotori(state) => {
            let summary = state.timepiece.summary(now_us)?;
            Ok(std::collections::BTreeMap::from([(
                "hotori.timepiece".into(),
                summary,
            )]))
        }
        crate::characters::RuntimeState::Hathor(state) => Ok(match &state.power {
            Some(power) => std::collections::BTreeMap::from([(
                "hathor.express_delivery_power".into(),
                power.pool.summary()?,
            )]),
            None => Default::default(),
        }),
        crate::characters::RuntimeState::Edgar(state) => {
            let mut summaries = std::collections::BTreeMap::new();
            if let Some(pool) = &state.pending_charge {
                summaries.insert("edgar.pending_charge".into(), pool.summary()?);
            }
            Ok(summaries)
        }
        crate::characters::RuntimeState::Chiz(state) => {
            return crate::characters::chiz::resource::summary(state);
        }
        crate::characters::RuntimeState::Blackbird(state) => {
            crate::characters::blackbird::malice::summary(state)
        }
        crate::characters::RuntimeState::Chaos(state) => {
            crate::characters::chaos::resources::summary(state)
        }
        _ => Ok(Default::default()),
    }?;
    Ok(summaries
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect())
}

pub fn primary_packet(character_id: &str, ability: &str) -> Option<&'static str> {
    lookup_kit(character_id).and_then(|kit| kit.primary_packet(ability))
}

pub(crate) fn incoming_damage_immune(
    config: &GameActorConfig,
    actor: &ActorRuntime,
    now_us: u64,
) -> bool {
    lookup_kit(&config.character_id).is_some_and(|kit| kit.incoming_damage_immune(actor, now_us))
}

pub(crate) fn settle_incoming_hp(
    mut ctx: crate::runtime_api::IncomingHpContext<'_>,
    vitals: &mut ntesim_core::state::VitalState,
) -> Result<crate::runtime_api::IncomingHpSettlement, String> {
    if let Some(kit) = lookup_kit(&ctx.request.actors[ctx.actor_index].character_id) {
        kit.validate_incoming_hp(&ctx)?;
    }
    if ctx.residual >= ctx.hp_before
        && let Some(kit) = lookup_kit(&ctx.request.actors[ctx.actor_index].character_id)
        && kit.intercept_fatal_hit(&mut ctx, vitals)?
    {
        return Ok(crate::runtime_api::IncomingHpSettlement {
            actual_hp_loss: ctx.hp_before.saturating_sub(1),
            overflow: 0,
            defeated: false,
        });
    }
    let (actual_hp_loss, overflow, defeated) = vitals
        .settle_hp_damage(ctx.residual)
        .map_err(|error| format!("incoming HP settlement failed: {error:?}"))?;
    Ok(crate::runtime_api::IncomingHpSettlement {
        actual_hp_loss,
        overflow,
        defeated,
    })
}

pub(crate) fn incoming_damage_plan(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    actor: usize,
    damage: u64,
) -> Result<crate::runtime_api::IncomingDamagePlan, String> {
    crate::characters::fadia::incoming::plan(ctx, actor, damage)
}

pub(crate) fn finishing_damage(
    ctx: &crate::runtime_api::FinishHitContext<'_>,
) -> Result<Option<crate::runtime_api::FinishDamage>, String> {
    for (owner, config) in ctx.request.actors.iter().enumerate() {
        if let Some(kit) = lookup_kit(&config.character_id)
            && let Some(finish) = kit.finishing_damage(ctx, owner)?
        {
            return Ok(Some(finish));
        }
    }
    Ok(None)
}

pub(crate) fn on_finish_target_defeated(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    if let Some(kit) = lookup_kit(&ctx.request.actors[owner].character_id) {
        kit.on_finish(ctx, owner, target)?;
    }
    on_party_target_defeated(ctx, owner, target)?;
    crate::characters::team::on_target_defeated(ctx.request, ctx.state, target, ctx.trace)?;
    for (config, actor) in ctx.request.actors.iter().zip(&mut ctx.state.actors) {
        on_target_defeated(config, actor, ctx.state.current_time_us)?;
    }
    crate::gear::on_target_defeated(ctx, owner, target)
}

pub(crate) fn on_party_target_defeated(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    target: usize,
) -> Result<(), String> {
    crate::simulation::reactions::cancel_target_remora(ctx, target)?;
    crate::characters::hathor::gaze::target_defeated(ctx, target)?;
    crate::characters::hathor::power::on_target_defeated(ctx, owner, target)?;
    crate::characters::chaos::warrant::on_party_defeat(ctx, owner, target)?;
    crate::characters::zankou::skill::on_scorch_target_defeated(ctx, target)
}

/// Initialize content-owned world timers after the shared event queue exists.
pub(crate) fn initialize_combat(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
) -> Result<(), String> {
    crate::characters::zankou::combat_passives::initialize(ctx)?;
    crate::characters::hathor::gaze::initialize(ctx)?;
    crate::characters::mint::clock::tenacity::initialize(ctx)?;
    for owner in 0..ctx.request.actors.len() {
        if ctx.request.actors[owner].character_id == "character:1073" {
            crate::characters::chiz::pressed::initialize_combat(ctx, owner)?;
        }
    }
    Ok(())
}
pub(crate) fn observe_dummy_combat(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    event_index: usize,
) -> Result<(), String> {
    crate::characters::hathor::gaze::observe(ctx, event_index)
}

pub(crate) fn observe_dummy_contact(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    target: usize,
) -> Result<(), String> {
    crate::characters::hathor::gaze::contact(ctx, target)
}

/// Sum live owner-managed carriers without coupling the hit pipeline to kits.
pub(crate) fn attached_dot_stacks(
    request: &GameScenarioRequest,
    state: &crate::simulation::state::GameSimulatorState,
    target: usize,
    now: u64,
) -> Result<u32, String> {
    request
        .actors
        .iter()
        .zip(&state.actors)
        .filter(|(_, actor)| actor.hp > 0)
        .try_fold(0u32, |count, (config, actor)| {
            let stacks = lookup_kit(&config.character_id)
                .map_or(0, |kit| kit.attached_dot_stacks(actor, target, now));
            count
                .checked_add(stacks)
                .ok_or_else(|| "attached DoT stack count overflow".into())
        })
}
