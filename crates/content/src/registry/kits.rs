use super::traits::CharacterKit;
use crate::runtime_api::{
    ActorHitContext, BreakContext, BreakModifiers, CastContext, DamageContext, HitContext,
    HitModifiers, RecipientContext,
};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::state::{ActorRuntime, TargetRuntime};
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest};
pub(super) struct AdlerKit;
impl CharacterKit for AdlerKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::adler::SUPPORT)
    }
    fn attached_dot_stacks(&self, actor: &ActorRuntime, target: usize, now: u64) -> u32 {
        crate::characters::adler::skill::attached_dot_stacks(actor, target, now)
    }

    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::adler::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::adler::skill::damage_bonus(ctx, bonus)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::adler::skill::on_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::adler::primary_packet(ability)
    }
    fn recipient_modifiers(
        &self,
        ctx: &RecipientContext<'_>,
        value: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::adler::skill::recipient_modifiers(ctx, value)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::adler::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        crate::characters::adler::riposte::accepted(request)
            .then_some(crate::characters::adler::riposte::HYPOTHESIS_ID)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::adler::riposte::accepted(ctx.request) {
            ctx.record_hypothesis(crate::characters::adler::riposte::HYPOTHESIS_ID)?;
            ctx.actor.character.adler_mut()?.riposte_ready = true;
        }
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability_id: &str,
        packet_id: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        crate::characters::adler::riposte::validate(actor, ability_id, packet_id)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::adler::riposte::direct_packet_energy(ability, packet)
            .or_else(|| crate::characters::adler::plunge::direct_packet_energy(ability, packet))
    }
}

pub(super) struct AkaneKit;
impl CharacterKit for AkaneKit {
    fn skill_gear_trigger_on_finish(&self, request: &GameScenarioRequest) -> bool {
        crate::characters::akane::clock::accepted(request)
    }
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::akane::SUPPORT)
    }
    fn select_source_command(
        &self,
        actor: &ActorRuntime,
        command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        crate::characters::akane::skill::select_source_command(actor, command)
    }
    fn resource_admission_failure(
        &self,
        request: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::akane::skill::resource_admission_failure(
            request, command, actor, ability, now,
        )
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::akane::skill::initialize(request, config, actor)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::akane::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::akane::secondary::accepted(request)
            .then_some(crate::characters::akane::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::akane::secondary::critical_dodge(ctx)
    }
    fn on_player_input(
        &self,
        request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        crate::characters::akane::secondary::on_input(request, actor, input)
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if input == crate::simulation::PlayerInput::Jump
            && crate::characters::akane::clock::accepted(request)
        {
            return Ok(Some(crate::characters::akane::clock::MODEL));
        }
        crate::characters::akane::secondary::validate_input(request, input)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::akane::secondary::validate_plunge(ctx)
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        crate::characters::akane::secondary::validate_packet(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::akane::secondary::energy(ability, packet)
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::akane::primary_packet(ability)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::akane::skill::on_cast(ctx)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::akane::skill::on_hit(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor: usize,
    ) -> Result<(), String> {
        crate::characters::akane::clock::exit(ctx, actor)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::akane::awakenings::damage_bonus(ctx, bonus)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::akane::buff_duration(ability)
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::akane::clock::cancel::recovery(request, actor, ability, now)
            .ok()
            .flatten()
    }
}

pub(super) struct ElymsKit;
impl CharacterKit for ElymsKit {
    fn skill_gear_trigger_on_finish(&self, request: &GameScenarioRequest) -> bool {
        crate::characters::elyms::clock::accepted(request)
    }
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::elyms::SUPPORT)
    }
    fn select_source_command(
        &self,
        actor: &ActorRuntime,
        command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        crate::characters::elyms::skill::select_source_command(actor, command)
    }
    fn resource_admission_failure(
        &self,
        request: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::elyms::skill::resource_admission_failure(
            request, command, actor, ability, now,
        )
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::elyms::skill::initialize_actor(request, config, actor)
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        crate::characters::elyms::skill::on_defeat(actor);
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::elyms::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::elyms::secondary::accepted(request)
            .then_some(crate::characters::elyms::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::elyms::secondary::critical_dodge(ctx)
    }
    fn on_player_input(
        &self,
        request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        crate::characters::elyms::secondary::on_input(request, actor, input)
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if input == crate::simulation::PlayerInput::Jump
            && crate::characters::elyms::clock::accepted(request)
        {
            return Ok(Some(crate::characters::elyms::clock::MODEL));
        }
        crate::characters::elyms::secondary::validate_input(request, input)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::elyms::secondary::validate_plunge(ctx)
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        crate::characters::elyms::secondary::validate_packet(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::elyms::secondary::energy(ability, packet)
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::elyms::primary_packet(ability)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::elyms::skill::on_cast(ctx)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::elyms::skill::on_hit(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor: usize,
    ) -> Result<(), String> {
        crate::characters::elyms::clock::exit(ctx, actor)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::elyms::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::elyms::awakenings::crit_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::elyms::awakenings::defense_ignore(ctx)
    }
    fn recipient_modifiers(
        &self,
        ctx: &RecipientContext<'_>,
        value: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::elyms::awakenings::recipient_modifiers(ctx, value)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::elyms::buff_duration(ability)
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::elyms::clock::cancel::recovery(request, actor, ability, now)
            .ok()
            .flatten()
    }
}

pub(super) struct AureliaKit;
impl CharacterKit for AureliaKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::aurelia::SUPPORT)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::aurelia::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        if crate::characters::aurelia::riposte::accepted(request) {
            Some(crate::characters::aurelia::riposte::HYPOTHESIS_ID)
        } else if request.model_assumptions.contains_key(
            crate::characters::aurelia::staccato::PERFECT_DODGE_HYPOTHESIS_ID.as_str(),
        ) {
            Some(crate::characters::aurelia::staccato::PERFECT_DODGE_HYPOTHESIS_ID)
        } else {
            None
        }
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        let in_staccato = {
            let state = ctx.actor.character.aurelia()?;
            state.is_in_staccato(ctx.time_us)
        };
        if in_staccato {
            crate::characters::aurelia::staccato::handle_staccato_critical_dodge(ctx)?;
        } else {
            ctx.actor.character.aurelia_mut()?.riposte_ready = true;
        }
        crate::characters::aurelia::cadenza::handle_a2_trigger(ctx)?;
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        crate::characters::aurelia::riposte::validate(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::aurelia::riposte::direct_packet_energy(ability, packet)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::aurelia::riposte::on_cast(ctx)? {
            return Ok(());
        }
        if crate::characters::aurelia::plunge::on_cast(ctx)? {
            return Ok(());
        }
        if crate::characters::aurelia::staccato::on_cast(ctx)? {
            return Ok(());
        }
        crate::characters::aurelia::skill::on_cast(ctx)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::aurelia::crescendo::on_hit(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::aurelia::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::aurelia::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::aurelia::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::aurelia::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::aurelia::buff_duration(ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::aurelia::primary_packet(ability)
    }
}

pub(super) struct BaicangKit;
impl CharacterKit for BaicangKit {
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::baicang::plunge::validate_input(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor: usize,
    ) -> Result<(), String> {
        ctx.state.actors[actor]
            .character
            .baicang_mut()?
            .entry_jump_pending = false;
        Ok(())
    }
    fn observes_vitals(&self, request: &GameScenarioRequest, config: &GameActorConfig) -> bool {
        crate::characters::baicang::fadia_assist::observes(request, config)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        changes: &[crate::simulation::vitals::HpChange],
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::baicang::fadia_assist::changed(ctx, owner, changes, trigger)
    }
    fn on_party_entry(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _entering: usize,
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::baicang::fadia_assist::request_check(ctx, owner, trigger)
    }
    fn on_connected_packet(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        key: usize,
        _group: Option<&str>,
        _target_index: usize,
        effect: &str,
    ) -> Result<(), String> {
        crate::characters::baicang::fadia_assist::connected(ctx, owner, key, _group, effect)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        effect: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        (ability == crate::characters::baicang::fadia_assist::ABILITY
            && effect == crate::characters::baicang::fadia_assist::PACKET)
            .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                hypothesis_id: crate::characters::baicang::fadia_assist::HYPOTHESIS_ID,
                event_kind: "baicang_fadia_energy_grant",
            })
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        actor.character.baicang_mut()?.power_words_mode =
            crate::characters::baicang::words::accepted(request);
        Ok(())
    }
    fn source_atk_percent(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &ActorRuntime,
        now: u64,
    ) -> Result<ExactFraction, String> {
        crate::characters::baicang::awakenings::source_atk_percent(
            config,
            actor,
            now,
            crate::characters::baicang::words::accepted(request),
        )
    }
    fn entry_action(
        &self,
        request: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        crate::characters::baicang::etd::entry_action(request, state, old, new, qualified_support)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::baicang::clock::on_hit(ctx)
    }
    fn on_finish(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        target: usize,
    ) -> Result<(), String> {
        crate::characters::baicang::clock::execute(ctx, owner, target)
    }
    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        crate::characters::baicang::interrupt::validate(request)
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor_index: usize,
        event_index: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::baicang::interrupt::apply(ctx, actor_index, event_index)
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability_id: &str,
        now_us: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::baicang::cancel::recovery_override(request, actor, ability_id, now_us)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !crate::characters::baicang::words::accepted(request) {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if input == crate::simulation::PlayerInput::Jump {
            if !crate::characters::baicang::cancel::accepted(request) {
                return Err("Baicang jump requires baicang.cancel_windows_v1".into());
            }
            return Ok(Some(crate::characters::baicang::cancel::HYPOTHESIS_ID));
        }
        if input == crate::simulation::PlayerInput::Dash
            && (crate::characters::baicang::words::accepted(request)
                || crate::characters::baicang::riposte::accepted(request)
                || !request.hold_inputs.is_empty())
        {
            if request.combat_stamina.is_none() {
                return Err("Baicang dodge requires an explicit combat stamina profile".into());
            }
            return Ok(Some(crate::characters::baicang::branches::HYPOTHESIS_ID));
        }
        Ok(None)
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        if let crate::characters::RuntimeState::Baicang(state) = &mut actor.character {
            *state = Default::default();
        }
    }
    fn incoming_damage_immune(&self, actor: &ActorRuntime, now_us: u64) -> bool {
        crate::characters::baicang::survival::is_actor_fatal_hit_immune(actor, now_us)
    }
    fn intercept_fatal_hit(
        &self,
        ctx: &mut crate::runtime_api::IncomingHpContext<'_>,
        vitals: &mut ntesim_core::state::VitalState,
    ) -> Result<bool, String> {
        crate::characters::baicang::survival::intercept_fatal_hit(ctx, vitals)
    }
    fn finishing_damage(
        &self,
        ctx: &crate::runtime_api::FinishHitContext<'_>,
        owner: usize,
    ) -> Result<Option<crate::runtime_api::FinishDamage>, String> {
        crate::characters::baicang::survival::finishing_damage(ctx, owner)
    }
    fn on_hp_loss_hit(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::baicang::words::on_connected_hit(ctx)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::baicang::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        if crate::characters::baicang::riposte::accepted(request) {
            Some(crate::characters::baicang::riposte::HYPOTHESIS_ID)
        } else {
            None
        }
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        let state = ctx.actor.character.baicang_mut()?;
        state.riposte_ready = true;
        state.derived_skill_stage = 3;
        state.derived_skill_stage_expires_at_us =
            ctx.actor.locked_until_us.saturating_add(2_000_000);
        Ok(())
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::baicang::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::baicang::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::baicang::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::baicang::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::baicang::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::baicang::buff_duration(&config.awakening, ability)
    }
    fn validate_hit_target(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        ability_id: &str,
        packet: &PacketMetadata,
        target: &TargetRuntime,
    ) -> Result<(), String> {
        crate::characters::baicang::skill::validate_hit_target(request, ability_id, packet, target)
    }
    fn recipient_modifiers(
        &self,
        ctx: &RecipientContext<'_>,
        value: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::baicang::awakenings::recipient_modifiers(ctx, value)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::baicang::primary_packet(ability)
    }
}

pub(super) struct BlackbirdKit;
impl CharacterKit for BlackbirdKit {
    fn resource_admission_failure(
        &self,
        _request: &GameScenarioRequest,
        _command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::blackbird::skill::permission_admission_failure(actor, ability, now)
    }
    fn select_source_command(
        &self,
        actor: &ActorRuntime,
        command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        crate::characters::blackbird::source::select(actor, command)
    }
    fn validate_ability_admission(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
    ) -> Result<(), String> {
        crate::characters::blackbird::skill::validate_ability(request, actor, ability)
    }
    fn validate_player_input_state(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::blackbird::skill::validate_player_input(request, actor)
    }
    fn on_connected_packet(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        key: usize,
        group: Option<&str>,
        target_index: usize,
        effect: &str,
    ) -> Result<(), String> {
        crate::characters::blackbird::malice::connected(
            ctx,
            owner,
            key,
            group,
            target_index,
            effect,
        )
    }
    fn on_party_entry(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        entering: usize,
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if owner == entering {
            crate::characters::blackbird::form::enter(ctx, owner)
        } else {
            Ok(())
        }
    }
    fn entry_action(
        &self,
        request: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            crate::characters::blackbird::support::select(request, state, new, qualified_support)?
                .map(crate::runtime_api::OwnerEntryAction::Support),
        )
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::blackbird::support::switch_lock(request, actor, ability, now)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::blackbird::secondary::validate_plunge(ctx)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::blackbird::secondary::replacement(actor)
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        crate::characters::blackbird::secondary::validate_packet(actor, ability, packet)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::blackbird::secondary::accepted(request)
            .then_some(crate::characters::blackbird::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::blackbird::secondary::critical_dodge(ctx)
    }
    fn on_player_input(
        &self,
        request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        crate::characters::blackbird::secondary::on_input(request, actor, input)
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        crate::characters::blackbird::secondary::validate_input(request, input)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::blackbird::secondary::energy(ability, packet)
            .or_else(|| crate::characters::blackbird::support::energy(ability, packet))
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::blackbird::secondary::on_exit(&mut ctx.state.actors[owner])?;
        crate::characters::blackbird::clock::on_exit(ctx, owner)?;
        crate::characters::blackbird::form::exit(ctx, owner)
    }
    fn observe_party_hit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        hit_index: usize,
        hits: &[crate::simulation::GameHitReceipt],
    ) -> Result<(), String> {
        crate::characters::blackbird::bird::observe(ctx, owner, hit_index, hits)
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::blackbird::malice::initialize(request, config, actor)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !effect.starts_with("GE_Player_BlackBird_") {
            return Ok(None);
        }
        if effect.starts_with("GE_Player_BlackBird_Evolve_")
            && !crate::characters::blackbird::skill::evolve_accepted(request)
        {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::blackbird::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::blackbird::awakenings::damage_bonus(ctx, bonus)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::blackbird::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::blackbird::buff_duration(&config.awakening, ability)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::blackbird::skill::on_hit(ctx)
    }
    fn recipient_modifiers(
        &self,
        ctx: &RecipientContext<'_>,
        value: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::blackbird::awakenings::recipient_modifiers(ctx, value)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::blackbird::primary_packet(ability)
    }
}

pub(super) struct ChaosKit;
impl CharacterKit for ChaosKit {
    fn skill_gear_trigger_on_finish(&self, request: &GameScenarioRequest) -> bool {
        crate::characters::chaos::clock::accepted(request)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::chaos::secondary::replacement(actor)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::chaos::secondary::validate_plunge(ctx)
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        crate::characters::chaos::secondary::validate_packet(actor, ability, packet)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::chaos::secondary::accepted(request)
            .then_some(crate::characters::chaos::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::chaos::secondary::critical_dodge(ctx)
    }
    fn on_player_input(
        &self,
        request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        crate::characters::chaos::secondary::on_input(request, actor, input)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::chaos::secondary::energy(ability, packet)
    }
    fn entry_action(
        &self,
        request: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            crate::characters::chaos::support::select(request, state, new, qualified_support)?
                .map(crate::runtime_api::OwnerEntryAction::Support),
        )
    }
    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        crate::characters::chaos::clock::validate_interrupt(request)
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor_index: usize,
        event_index: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::chaos::clock::interrupt(ctx, actor_index, event_index)
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if crate::characters::chaos::clock::accepted(request)
            && matches!(
                input,
                crate::simulation::PlayerInput::Dash | crate::simulation::PlayerInput::Jump
            )
        {
            if input == crate::simulation::PlayerInput::Jump
                && crate::characters::chaos::cancel::accepted(
                    request,
                    crate::characters::chaos::cancel::JUMP,
                )
            {
                return Ok(Some(crate::characters::chaos::cancel::JUMP));
            }
            if let Some(model) =
                crate::characters::chaos::secondary::validate_input(request, input)?
            {
                return Ok(Some(model));
            }
            return Err("UNSUPPORTED_CHAOS_SECONDARY: ordinary clock dodge/jump requires its owner secondary/cancel binding".into());
        }
        Ok(None)
    }
    fn validate_ability_admission(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        ability: &str,
    ) -> Result<(), String> {
        if crate::characters::chaos::form::accepted(request)
            && ability == "GA_Chaos071_QTE"
            && !crate::characters::chaos::support::accepted(request)
        {
            return Err(
                "UNSUPPORTED_CHAOS_SUPPORT: primary form requires the qualified QTE binding".into(),
            );
        }
        Ok(())
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        crate::characters::chaos::form::defeat(actor);
    }
    fn incoming_damage_immune(&self, actor: &ActorRuntime, now_us: u64) -> bool {
        crate::characters::chaos::form::immune(actor, now_us)
    }
    fn recovery_lock_override(
        &self,
        _request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::chaos::form::switch_lock(actor, ability, now)
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::chaos::resources::initialize(request, config, actor)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::chaos::clock::on_hit(ctx)?;
        crate::characters::chaos::resources::on_hit(ctx)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        crate::characters::chaos::resources::cycle(request, effect)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::chaos::clock::exit(ctx, owner)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::chaos::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::chaos::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::chaos::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::chaos::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::chaos::buff_duration(ability)
    }
    fn has_before_hit(&self) -> bool {
        true
    }
    fn before_hit(&self, ctx: &mut HitContext<'_>) -> Result<(), String> {
        crate::characters::chaos::skill::before_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::chaos::primary_packet(ability)
    }
}

pub(super) struct ChizKit;
impl CharacterKit for ChizKit {
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::chiz::secondary::validate_plunge(ctx)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::chiz::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::chiz::secondary::accepted(request)
            .then_some(crate::characters::chiz::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::chiz::secondary::critical_dodge(ctx)
    }
    fn entry_action(
        &self,
        request: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            crate::characters::chiz::secondary::select(request, state, new, qualified_support)?
                .map(crate::runtime_api::OwnerEntryAction::Support),
        )
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        _ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        if packet == crate::characters::chiz::secondary::RIPOSTE
            && crate::characters::chiz::secondary::replacement(actor).is_none()
        {
            return Err("Chiz Critical Riposte requires one paid successful Critical Dodge".into());
        }
        Ok(())
    }
    fn validate_hold_stock_selection(
        &self,
        input: &crate::simulation::GameHoldInput,
    ) -> Result<(), String> {
        crate::characters::chiz::pressed::held::validate_stock_selection(input)
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::chiz::pressed::held::finish::recovery(request, actor, ability, now)
            .or_else(|| {
                crate::characters::chiz::pressed::held::switch_recovery(request, actor, ability)
            })
            .or_else(|| crate::characters::chiz::cancel::recovery(request, actor, ability, now))
    }
    fn resource_admission_failure(
        &self,
        request: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        _now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::chiz::barren::failure(request, command, actor, ability).or_else(|| {
            crate::characters::chiz::pressed::admission_failure(request, command, actor, ability)
        })
    }
    fn has_before_hit(&self) -> bool {
        true
    }
    fn before_hit(&self, ctx: &mut HitContext<'_>) -> Result<(), String> {
        crate::characters::chiz::skill::before_hit(ctx)
    }
    fn on_player_input(
        &self,
        request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        crate::characters::chiz::secondary::on_input(request, actor, input)?;
        crate::characters::chiz::cancel::player_input(request, actor, input)?;
        crate::characters::chiz::pressed::reset(actor)
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if let Some(model) = crate::characters::chiz::secondary::validate_input(request, input)? {
            return Ok(Some(model));
        }
        Ok((input == crate::simulation::PlayerInput::Jump
            && crate::characters::chiz::cancel::accepted(request))
        .then_some(crate::characters::chiz::cancel::MODEL))
    }
    fn observes_vitals(&self, request: &GameScenarioRequest, config: &GameActorConfig) -> bool {
        crate::characters::chiz::finance::observes(request, config)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if ctx.state.actors[owner].hp == 0 {
            crate::characters::chiz::clock::exit(ctx, owner)?;
            crate::characters::chiz::pressed::exit(ctx, owner)?;
            ctx.state.actors[owner].character.chiz_mut()?.riposte_ready = false;
            crate::characters::chiz::market::exit(ctx, owner, true)?;
        }
        crate::characters::chiz::finance::refresh(ctx, owner)
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::chiz::clock::initialize(request, actor)?;
        crate::characters::chiz::pressed::initialize(request, config, actor)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !crate::characters::chiz::clock::accepted(request) {
            return Ok(None);
        }
        if effect.starts_with("GE_Player_Chiichan073_Skill")
            && !crate::characters::chiz::pressed::accepted(request)
        {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::chiz::secondary::energy(ability, packet)
            .or_else(|| crate::characters::chiz::passives::energy(ability, packet))
    }
    fn on_party_entry(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        entering: usize,
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::chiz::finance::refresh(ctx, owner)?;
        if owner == entering {
            crate::characters::chiz::passives::field(ctx, owner, true)
        } else {
            Ok(())
        }
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor: usize,
    ) -> Result<(), String> {
        ctx.state.actors[actor].character.chiz_mut()?.riposte_ready = false;
        crate::characters::chiz::passives::field(ctx, actor, false)?;
        crate::characters::chiz::clock::exit(ctx, actor)?;
        crate::characters::chiz::pressed::exit(ctx, actor)?;
        crate::characters::chiz::market::exit(ctx, actor, ctx.state.actors[actor].hp == 0)?;
        crate::characters::chiz::finance::refresh(ctx, actor)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::chiz::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::chiz::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::chiz::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::chiz::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::chiz::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::chiz::buff_duration(ability)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::chiz::skill::on_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::chiz::primary_packet(ability)
    }
}

pub(super) struct DaffodilKit;
impl CharacterKit for DaffodilKit {
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::daffodil::secondary::validate_plunge(ctx)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::daffodil::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::daffodil::secondary::accepted(request)
            .then_some(crate::characters::daffodil::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::daffodil::secondary::critical(ctx)
    }
    fn incoming_damage_immune(&self, actor: &ActorRuntime, now: u64) -> bool {
        crate::characters::daffodil::clock::dodge_immune(actor, now)
    }
    fn entry_action(
        &self,
        request: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        if let Some(support) =
            crate::characters::daffodil::secondary::select(request, state, new, qualified_support)?
        {
            return Ok(Some(crate::runtime_api::OwnerEntryAction::Support(support)));
        }
        Ok(
            crate::characters::daffodil::phantom::select(request, state, old, new)?
                .map(crate::runtime_api::OwnerEntryAction::Attack),
        )
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::daffodil::secondary::energy(ability, packet)
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        _ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        if packet == crate::characters::daffodil::secondary::RIPOSTE
            && crate::characters::daffodil::secondary::replacement(actor).is_none()
        {
            return Err("Daffodil Riposte requires a paid successful Critical Dodge".into());
        }
        Ok(())
    }
    fn on_player_input(
        &self,
        request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        _input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        if crate::characters::daffodil::secondary::accepted(request) {
            actor.character.daffodil_mut()?.riposte_ready = false;
        }
        Ok(())
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::daffodil::clock::cancel::recovery(request, actor, ability, now)
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if let Some(model) = crate::characters::daffodil::secondary::validate_input(request, input)?
        {
            return Ok(Some(model));
        }
        Ok((input == crate::simulation::PlayerInput::Jump
            && crate::characters::daffodil::clock::cancel::accepted(request))
        .then_some(crate::characters::daffodil::clock::cancel::MODEL))
    }
    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::daffodil::clock::cancel::accepted(request) {
            Ok(())
        } else {
            Err("Daffodil interruption requires action_cancels_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::daffodil::clock::cancel::interrupt(ctx, owner, event)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !crate::characters::daffodil::clock::accepted(request) {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn observes_vitals(&self, request: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        crate::characters::daffodil::clock::accepted(request)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if ctx.state.actors[owner].hp == 0 {
            crate::characters::daffodil::clock::stop(
                &mut ctx.state.actors[owner],
                ctx.queue,
                ctx.state.current_time_us,
            )?;
        }
        Ok(())
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        if crate::characters::daffodil::clock::accepted(request) {
            actor.character.daffodil_mut()?.clock = Some(Box::default());
        }
        crate::characters::daffodil::resonance::initialize(config, actor)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::daffodil::clock::contact(ctx)?;
        crate::characters::daffodil::phantom::on_hit(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::daffodil::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )
    }
    fn break_efficiency_bonus(
        &self,
        config: &GameActorConfig,
        actor: &ActorRuntime,
        command_index: usize,
        ability: &str,
    ) -> Result<ExactFraction, String> {
        if ability != "GA_Daffodill_Skill" {
            return ExactFraction::new(0, 1);
        }
        crate::characters::daffodil::resonance::break_efficiency(config, actor, command_index)
    }
    fn break_modifiers(&self, ctx: &BreakContext<'_>) -> Result<BreakModifiers, String> {
        crate::characters::daffodil::break_damage::modifiers(ctx)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::daffodil::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::daffodil::awakenings::damage_bonus(ctx, bonus)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::daffodil::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::daffodil::buff_duration(&config.awakening, ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::daffodil::primary_packet(ability)
    }
}

pub(super) struct EdgarKit;
impl CharacterKit for EdgarKit {
    fn incoming_damage_immune(&self, actor: &ActorRuntime, now: u64) -> bool {
        crate::characters::edgar::clock::immune(actor, now)
    }
    fn validate_player_input_state(
        &self,
        _request: &GameScenarioRequest,
        actor: &ActorRuntime,
    ) -> Result<(), String> {
        if matches!(&actor.character,crate::characters::RuntimeState::Edgar(s) if s.out_of_combat) {
            return Err("Edgar action requires being in combat".into());
        }
        Ok(())
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        if crate::characters::edgar::clock::accepted(request) {
            actor.character.edgar_mut()?.clock = Some(Box::default());
            actor.character.edgar_mut()?.pending_charge =
                Some(ntesim_core::kernel::state::ExactPool::new(
                    ExactFraction::new(20, 1)?,
                    ExactFraction::new(0, 1)?,
                )?);
        }
        Ok(())
    }
    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        input: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::edgar::clock::recovery(request, actor, input, now)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::edgar::clock::contact(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::edgar::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )
    }
    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::edgar::clock::accepted(request) {
            Ok(())
        } else {
            Err("Edgar interruption requires full_kit_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::edgar::clock::interrupt(ctx, owner, event)
    }
    fn observes_vitals(&self, request: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        crate::characters::edgar::clock::accepted(request)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::edgar::effects::on_vitals(ctx, owner, changes)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !crate::characters::edgar::clock::accepted(request) {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        ((ability == "GA_Edgar_Melee"
            && matches!(
                packet,
                crate::characters::edgar::secondary::PLUNGE
                    | crate::characters::edgar::secondary::RIPOSTE
            ))
            || (ability == "GA_Edgar_QTE" && packet == crate::characters::edgar::secondary::QTE))
            .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                hypothesis_id: crate::characters::edgar::clock::MODEL,
                event_kind: "edgar_secondary_energy",
            })
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        if !crate::characters::edgar::clock::accepted(ctx.request)
            || ctx.metadata.packet.damage_effect_id != crate::characters::edgar::secondary::PLUNGE
            || ctx.input.height_bonus.numerator > ctx.input.height_bonus.denominator
        {
            return Err("Edgar plunge requires full_kit_v1 and normalized height in0..1".into());
        }
        ctx.timestamp_us
            .checked_add(ctx.input.landing_delay_us)
            .and_then(|t| t.checked_add(198_548))
            .ok_or("Edgar plunge deadline overflow")?;
        Ok(())
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::edgar::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, request: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::edgar::clock::accepted(request)
            .then_some(crate::characters::edgar::clock::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.actor.character.edgar_mut()?.riposte_ready = true;
        Ok(())
    }
    fn on_player_input(
        &self,
        _request: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        _input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        actor.character.edgar_mut()?.riposte_ready = false;
        Ok(())
    }
    fn validate_player_input(
        &self,
        request: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        if crate::characters::edgar::clock::accepted(request)
            && matches!(
                input,
                crate::simulation::PlayerInput::Dash | crate::simulation::PlayerInput::Jump
            )
        {
            if request.combat_stamina.is_none() {
                return Err("Edgar Dodge/Jump requires explicit stamina".into());
            }
            return Ok(Some(crate::characters::edgar::clock::MODEL));
        }
        Ok(None)
    }
    fn entry_action(
        &self,
        request: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        if !crate::characters::edgar::clock::accepted(request)
            || state.actors[new].locked_until_us > state.current_time_us
            || matches!(&state.actors[new].character,crate::characters::RuntimeState::Edgar(s) if s.out_of_combat)
        {
            return Ok(None);
        }
        if qualified_support {
            return Ok(Some(crate::runtime_api::OwnerEntryAction::Support(
                &crate::characters::edgar::secondary::ENTRY,
            )));
        }
        Ok(None)
    }
    fn validate_ability_admission(
        &self,
        _request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
    ) -> Result<(), String> {
        if matches!(&actor.character,crate::characters::RuntimeState::Edgar(s) if s.out_of_combat) {
            return Err("Edgar action requires being in combat".into());
        }
        crate::characters::edgar::awakenings::validate_burst_admission(config, actor, ability)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::edgar::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::edgar::awakenings::damage_bonus(ctx, bonus)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::edgar::buff_duration(&config.awakening, ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::edgar::primary_packet(ability)
    }
}

pub(super) struct FadiaKit;
impl CharacterKit for FadiaKit {
    fn recovery_lock_override(
        &self,
        req: &GameScenarioRequest,
        actor: &ActorRuntime,
        input: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::fadia::clock::controls::recovery(req, actor, input, now)
    }
    fn validate_player_input_state(
        &self,
        _req: &GameScenarioRequest,
        actor: &ActorRuntime,
    ) -> Result<(), String> {
        if matches!(&actor.character,crate::characters::RuntimeState::Fadia(s) if s.out_of_combat) {
            return Err("Fadia action requires being in combat".into());
        }
        Ok(())
    }
    fn validate_player_input(
        &self,
        req: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        crate::characters::fadia::secondary::validate_input(req, input)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        if !crate::characters::fadia::effects::accepted(ctx.request)
            || ctx.metadata.packet.damage_effect_id != crate::characters::fadia::secondary::PLUNGE
        {
            return Err("Fadia plunge requires full_kit_v1 and native landing packet".into());
        }
        ctx.timestamp_us
            .checked_add(ctx.input.landing_delay_us)
            .and_then(|t| t.checked_add(294920))
            .ok_or("Fadia plunge deadline overflow")?;
        Ok(())
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::fadia::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, req: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::fadia::effects::accepted(req)
            .then_some(crate::characters::fadia::effects::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.actor.character.fadia_mut()?.riposte_ready = true;
        Ok(())
    }
    fn on_player_input(
        &self,
        _req: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        _input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        actor.character.fadia_mut()?.riposte_ready = false;
        Ok(())
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        if let crate::characters::RuntimeState::Fadia(s) = &mut actor.character {
            s.riposte_ready = false;
            s.lilith_until_us = None;
            s.high_q = false;
            s.marks.clear();
        }
    }
    fn entry_action(
        &self,
        req: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        if !crate::characters::fadia::effects::accepted(req)
            || state.actors[new].locked_until_us > state.current_time_us
            || matches!(&state.actors[new].character,crate::characters::RuntimeState::Fadia(s) if s.out_of_combat)
        {
            return Ok(None);
        }
        Ok(
            qualified_support.then_some(crate::runtime_api::OwnerEntryAction::Support(
                &crate::characters::fadia::secondary::ENTRY,
            )),
        )
    }
    fn on_party_entry(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _entering: usize,
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if trigger == crate::simulation::GameCausalTrigger::InitialState {
            crate::characters::fadia::stats::enter(ctx, owner)?;
        }
        Ok(())
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        ExactFraction::new(
            u128::from(crate::characters::fadia::clock::flags(ctx.actor, ctx.command_index).1),
            2,
        )
    }
    fn direct_packet_energy(
        &self,
        _ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        packet.starts_with("GE_Player_Fadia_").then_some(
            crate::runtime_api::DirectPacketEnergyPolicy {
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                hypothesis_id: crate::characters::fadia::effects::MODEL,
                event_kind: "fadia_hit_energy",
            },
        )
    }
    fn observes_vitals(&self, request: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        crate::characters::fadia::effects::accepted(request)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        changes: &[crate::simulation::vitals::HpChange],
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::fadia::effects::changed(ctx, owner, changes, trigger)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        modifiers: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::fadia::effects::damage(ctx, modifiers)
    }
    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::fadia::clock::accepted(request) {
            Ok(())
        } else {
            Err("Fadia interruption requires ordinary_clock_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::fadia::clock::interrupt(ctx, owner, event)
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        if crate::characters::fadia::clock::accepted(request) {
            actor.character.fadia_mut()?.clock = Some(Box::default());
            actor.character.fadia_mut()?.full_kit =
                crate::characters::fadia::effects::accepted(request);
        }
        Ok(())
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::fadia::clock::contact(ctx)?;
        crate::characters::fadia::effects::connected(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::fadia::clock::stop(&mut ctx.state.actors[owner], ctx.queue)?;
        let state = ctx.state.actors[owner].character.fadia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.lilith_until_us = None;
        state.high_q = false;
        state.riposte_ready = false;
        Ok(())
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !crate::characters::fadia::clock::accepted(request) {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn validate_packet_admission(
        &self,
        config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet_effect: &str,
        time_us: u64,
    ) -> Result<(), String> {
        crate::characters::fadia::skill::validate_admission(
            config,
            actor,
            ability,
            packet_effect,
            time_us,
        )
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::fadia::skill::on_cast(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::fadia::primary_packet(ability)
    }
}

pub(super) struct HanielKit;
impl CharacterKit for HanielKit {
    fn recovery_lock_override(
        &self,
        req: &GameScenarioRequest,
        actor: &ActorRuntime,
        input: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::haniel::clock::recovery(req, actor, input, now)
    }
    fn validate_player_input_state(
        &self,
        _req: &GameScenarioRequest,
        actor: &ActorRuntime,
    ) -> Result<(), String> {
        if matches!(&actor.character,crate::characters::RuntimeState::Haniel(s) if s.out_of_combat)
        {
            return Err("Haniel cannot act outside combat".into());
        }
        Ok(())
    }
    fn validate_player_input(
        &self,
        req: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        crate::characters::haniel::secondary::validate_input(req, input)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        if !crate::characters::haniel::secondary::accepted(ctx.request)
            || ctx.metadata.packet.damage_effect_id != crate::characters::haniel::secondary::PLUNGE
            || ctx.input.height_bonus.numerator > ctx.input.height_bonus.denominator
        {
            return Err("Haniel plunge requires full_kit_v1 and normalized height in0..1".into());
        }
        ctx.timestamp_us
            .checked_add(ctx.input.landing_delay_us)
            .and_then(|t| t.checked_add(175_616))
            .ok_or("Haniel plunge deadline overflow")?;
        Ok(())
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::haniel::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, req: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::haniel::secondary::accepted(req)
            .then_some(crate::characters::haniel::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.actor.character.haniel_mut()?.riposte_ready_us = Some(ctx.time_us);
        Ok(())
    }
    fn on_player_input(
        &self,
        _req: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        _input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        actor.character.haniel_mut()?.riposte_ready_us = None;
        Ok(())
    }
    fn entry_action(
        &self,
        req: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        crate::characters::haniel::secondary::entry(req, state, new, qualified_support)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        ((ability == "GA_Haniel_Melee"
            && matches!(
                packet,
                crate::characters::haniel::secondary::PLUNGE
                    | crate::characters::haniel::secondary::RIPOSTE
            ))
            || (ability == "GA_Haniel_QTE" && packet == crate::characters::haniel::secondary::QTE))
            .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                hypothesis_id: crate::characters::haniel::secondary::MODEL,
                event_kind: "haniel_secondary_energy",
            })
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::haniel::clock::contact(ctx)
    }
    fn initialize(
        &self,
        request: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        if crate::characters::haniel::clock::accepted(request) {
            actor.character.haniel_mut()?.clock = Some(Box::default());
        }
        Ok(())
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::haniel::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )?;
        let state = ctx.state.actors[owner].character.haniel_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.riposte_ready_us = None;
        Ok(())
    }
    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::haniel::clock::accepted(request) {
            Ok(())
        } else {
            Err("Haniel interruption requires ordinary_clock_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::haniel::clock::interrupt(ctx, owner, event)
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        crate::characters::haniel::clock::cycle(request, effect)
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &crate::simulation::state::ActorRuntime,
        _ability: &str,
        packet: &str,
        time: u64,
    ) -> Result<(), String> {
        if matches!(&actor.character, crate::characters::RuntimeState::Haniel(s) if s.out_of_combat)
        {
            return Err("Haniel cannot act outside combat".into());
        }
        if packet == crate::characters::haniel::secondary::RIPOSTE
            && matches!(&actor.character,crate::characters::RuntimeState::Haniel(s) if s.riposte_ready_us.is_none_or(|ready|ready>time))
        {
            return Err("Haniel Riposte response is not ready".into());
        }
        Ok(())
    }
    fn on_party_entry(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _entering: usize,
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if trigger == crate::simulation::GameCausalTrigger::InitialState {
            crate::characters::haniel::stats::enter(ctx, owner)?;
        }
        Ok(())
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::haniel::skill::on_cast(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::haniel::buff_duration(&config.awakening, ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::haniel::primary_packet(ability)
    }
}

pub(super) struct HathorKit;
impl CharacterKit for HathorKit {
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::hathor::air::validate_plunge(ctx)
    }
    fn validate_player_input(
        &self,
        req: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        crate::characters::hathor::secondary::validate_input(req, input)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::hathor::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, req: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        crate::characters::hathor::secondary::accepted(req)
            .then_some(crate::characters::hathor::secondary::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.actor.character.hathor_mut()?.riposte_ready_us = Some(ctx.time_us);
        Ok(())
    }
    fn on_player_input(
        &self,
        _req: &GameScenarioRequest,
        actor: &mut ActorRuntime,
        _input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        actor.character.hathor_mut()?.riposte_ready_us = None;
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        _ability: &str,
        packet: &str,
        now: u64,
    ) -> Result<(), String> {
        if packet == crate::characters::hathor::secondary::RIPOSTE
            && matches!(&actor.character,crate::characters::RuntimeState::Hathor(s) if s.riposte_ready_us.is_none_or(|t|t>now))
        {
            return Err("Hathor Riposte response is not ready".into());
        }
        Ok(())
    }
    fn entry_action(
        &self,
        req: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        crate::characters::hathor::secondary::entry(req, state, new, qualified_support)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        if ability == "GA_Hathor_Melee" && packet == crate::characters::hathor::air::PACKET {
            return Some(crate::runtime_api::DirectPacketEnergyPolicy {
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                hypothesis_id: crate::characters::hathor::air::MODEL,
                event_kind: "hathor_airattack_energy",
            });
        }
        ((ability == "GA_Hathor_Melee"
            && matches!(
                packet,
                crate::characters::hathor::secondary::RIPOSTE
                    | crate::characters::hathor::secondary::RIPOSTE2
            ))
            || (ability == "GA_Hathor_QTE"
                && matches!(
                    packet,
                    crate::characters::hathor::secondary::QTE
                        | crate::characters::hathor::secondary::QTE2
                )))
        .then_some(crate::runtime_api::DirectPacketEnergyPolicy {
            trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            hypothesis_id: crate::characters::hathor::secondary::MODEL,
            event_kind: "hathor_secondary_energy",
        })
    }
    fn validate_ability_admission(
        &self,
        _req: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        _ability: &str,
    ) -> Result<(), String> {
        crate::characters::hathor::combat::admission(actor)
    }
    fn validate_player_input_state(
        &self,
        _req: &GameScenarioRequest,
        actor: &ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::hathor::combat::admission(actor)
    }
    fn recovery_lock_override(
        &self,
        req: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::hathor::clock::controls::recovery(req, actor, ability, now).or_else(
            || {
                crate::characters::hathor::cyclone::accepted(req)
                    .then(|| crate::characters::hathor::cyclone::recovery(actor, ability, now))
                    .flatten()
            },
        )
    }
    fn initialize(
        &self,
        req: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        if crate::characters::hathor::clock::accepted(req) {
            actor.character.hathor_mut()?.clock = Some(Box::default());
        }
        crate::characters::hathor::power::initialize(req, config, actor)?;
        if crate::characters::hathor::burst::accepted(req) {
            actor.character.hathor_mut()?.burst = Some(Box::default());
        }
        Ok(())
    }
    fn resource_admission_failure(
        &self,
        request: &GameScenarioRequest,
        _command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        if ability == "GA_Hathor_Skill" && crate::characters::hathor::burst::accepted(request) {
            if crate::characters::hathor::cyclone::accepted(request) {
                crate::characters::hathor::cyclone::failure(actor, now)
            } else {
                crate::characters::hathor::burst::skill_failure(actor, now)
            }
        } else {
            None
        }
    }
    fn source_atk_percent(
        &self,
        _request: &GameScenarioRequest,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        now: u64,
    ) -> Result<ExactFraction, String> {
        crate::characters::hathor::burst::atk_percent(actor, now)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::hathor::clock::contact(ctx)?;
        crate::characters::hathor::secondary::contact(ctx)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::hathor::burst::clear(ctx, owner)?;
        ctx.state.actors[owner]
            .character
            .hathor_mut()?
            .riposte_ready_us = None;
        ctx.state
            .buffs
            .remove_owned_buff("hathor.cyclone_def", "character:1025", owner);
        crate::characters::hathor::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )
    }
    fn validate_interrupt(&self, req: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::hathor::clock::accepted(req) {
            Ok(())
        } else {
            Err("Hathor interruption requires ordinary_clock_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::hathor::clock::interrupt(ctx, owner, event)
    }
    fn packet_cycle_grant(
        &self,
        req: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        crate::characters::hathor::clock::cycle(req, effect)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::hathor::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::hathor::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::hathor::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::hathor::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::hathor::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::hathor::buff_duration(ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::hathor::primary_packet(ability)
    }
}

pub(super) struct HotoriKit;
impl CharacterKit for HotoriKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::hotori::SUPPORT)
    }
    fn validate_interrupt(&self, _request: &GameScenarioRequest) -> Result<(), String> {
        Ok(())
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event_index: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::hotori::timepiece::interrupt(ctx, owner, event_index)
    }
    fn observes_vitals(&self, _request: &GameScenarioRequest, config: &GameActorConfig) -> bool {
        config.character_id == "character:1052"
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::hotori::timepiece::on_vitals_changed(ctx, owner, changes)
    }
    fn resource_admission_failure(
        &self,
        _request: &GameScenarioRequest,
        _command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::hotori::timepiece::resource_admission_failure(actor, ability, now)
    }
    fn recovery_lock_override(
        &self,
        _request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability_id: &str,
        now_us: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::hotori::timepiece::switch_lock(actor, ability_id, now_us)
            .or_else(|| crate::characters::hotori::katana::recovery_lock(actor, ability_id, now_us))
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::hotori::katana::replacement(actor)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::hotori::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::hotori::damage_bonus(ctx, bonus)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::hotori::awakenings::defense_ignore(ctx)
    }
    fn recipient_modifiers(
        &self,
        ctx: &RecipientContext<'_>,
        value: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::hotori::awakenings::recipient_modifiers(ctx, value)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::hotori::primary_packet(ability)
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
}

pub(super) struct IroiKit;
impl CharacterKit for IroiKit {
    fn entry_action(
        &self,
        _request: &GameScenarioRequest,
        _state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        _new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            qualified_support.then_some(crate::runtime_api::OwnerEntryAction::Support(
                &crate::characters::iroi::clock::SUPPORT,
            )),
        )
    }
    fn resource_admission_failure(
        &self,
        req: &GameScenarioRequest,
        command: usize,
        _actor: &ActorRuntime,
        ability: &str,
        _now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::iroi::clock::admission(req, command, ability)
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::iroi::plunge::validate_plunge(ctx)
    }
    fn validate_interrupt(&self, req: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::iroi::clock::accepted(req) {
            Ok(())
        } else {
            Err("Iroi interruption requires action_clock_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::iroi::clock::interrupt(ctx, owner, event)
    }
    fn recovery_lock_override(
        &self,
        req: &GameScenarioRequest,
        actor: &ActorRuntime,
        input: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::iroi::clock::input_lock(req, actor, input, now)
    }
    fn on_hit(&self, ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::iroi::clock::contact(ctx)
    }
    fn observes_vitals(&self, req: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        crate::characters::iroi::clock::accepted(req)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if ctx.state.actors[owner].hp == 0 {
            crate::characters::iroi::clock::stop(
                &mut ctx.state.actors[owner],
                ctx.queue,
                ctx.state.current_time_us,
            )?;
        }
        for change in changes {
            if change.actor_index != owner && change.after == 0 && change.before > 0 {
                crate::characters::iroi::regression::on_ally_fallen(
                    ctx,
                    owner,
                    change.actor_index,
                )?;
            }
        }
        Ok(())
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::iroi::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
        )
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        crate::characters::iroi::dream::defeat(actor);
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::iroi::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        crate::characters::iroi::riposte::accepted(request)
            .then_some(crate::characters::iroi::riposte::HYPOTHESIS_ID)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::iroi::riposte::accepted(ctx.request) {
            ctx.record_hypothesis(crate::characters::iroi::riposte::HYPOTHESIS_ID)?;
            ctx.actor.character.iroi_mut()?.riposte_ready = true;
        }
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        crate::characters::iroi::riposte::validate(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        if ability == "GA_Oneiroi_Skill" && packet == "GE_Player_Oneiroi_Skill_Damage" {
            return Some(crate::runtime_api::DirectPacketEnergyPolicy {
                hypothesis_id: crate::characters::iroi::clock::MODEL,
                event_kind: "iroi_skill_carrier_energy",
                trigger: crate::runtime_api::PacketEnergyTrigger::EffectApplication,
            });
        }
        if ability == crate::characters::iroi::clock::QTE
            && packet == crate::characters::iroi::clock::QTE_PACKET
        {
            return Some(crate::runtime_api::DirectPacketEnergyPolicy {
                hypothesis_id: crate::characters::iroi::clock::MODEL,
                event_kind: "iroi_support_hit_energy_grant",
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
            });
        }
        crate::characters::iroi::riposte::direct_packet_energy(ability, packet)
            .or_else(|| crate::characters::iroi::plunge::direct_packet_energy(ability, packet))
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::iroi::clock::cast(ctx)? {
            return Ok(());
        }
        crate::characters::iroi::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::iroi::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::iroi::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::iroi::awakenings::crit_rate_bonus(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::iroi::buff_duration(ability)
    }
    fn on_hp_loss_hit(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::iroi::skill::on_hp_loss_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::iroi::primary_packet(ability)
    }
}

pub(super) struct JiuyuanKit;
impl CharacterKit for JiuyuanKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::jiuyuan::SUPPORT)
    }
    fn on_single_support_contact(&self, actor: &mut ActorRuntime) -> Result<(), String> {
        let state = actor.character.jiuyuan_mut()?;
        state.bullets = state
            .bullets
            .saturating_add(2)
            .min(crate::characters::jiuyuan::resource::CAPACITY);
        Ok(())
    }
    fn initialize(
        &self,
        _request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        crate::characters::jiuyuan::resource::initialize(config, actor)
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        crate::characters::jiuyuan::resource::defeat(actor);
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::jiuyuan::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::jiuyuan::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::jiuyuan::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::jiuyuan::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::jiuyuan::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::jiuyuan::buff_duration(ability)
    }
    fn has_before_hit(&self) -> bool {
        true
    }
    fn before_hit(&self, ctx: &mut HitContext<'_>) -> Result<(), String> {
        crate::characters::jiuyuan::skill::before_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::jiuyuan::primary_packet(ability)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        let state = ctx.actor.character.jiuyuan_mut()?;
        state.bullets = state
            .bullets
            .saturating_add(2)
            .min(crate::characters::jiuyuan::resource::CAPACITY);
        Ok(())
    }
    fn on_connected_packet(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _key: usize,
        _group: Option<&str>,
        target_index: usize,
        effect: &str,
    ) -> Result<(), String> {
        if !ctx
            .request
            .model_assumptions
            .contains_key(crate::characters::jiuyuan::skill::PACKETS_HYPOTHESIS_ID.as_str())
        {
            return Ok(());
        }
        match effect {
            "GE_Player_Kuhara_UltraSkill4_Damage" => {
                crate::characters::jiuyuan::pact::on_ultra4_hit(ctx, owner, target_index)?;
            }
            "GE_Player_Kuhara_BranchPower_Damage" => {
                crate::characters::jiuyuan::pact::on_branch_power_hit(ctx, owner, target_index)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn packet_cycle_grant(
        &self,
        request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if !crate::characters::jiuyuan::pact::accepted(request) {
            return Ok(None);
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
}

pub(super) struct LacrimosaKit;
impl CharacterKit for LacrimosaKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::lacrimosa::SUPPORT)
    }
    fn attached_dot_stacks(&self, actor: &ActorRuntime, target: usize, now: u64) -> u32 {
        crate::characters::lacrimosa::dot::attached_dot_stacks(actor, target, now)
    }

    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }

    fn recovery_lock_override(
        &self,
        request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::lacrimosa::bat_exit::recovery_override(request, actor, ability, now)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor: usize,
    ) -> Result<(), String> {
        crate::characters::lacrimosa::bat_exit::on_exit(ctx, actor)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.actor.character.lacrimosa_mut()?.bat_evade_until_us = 0;
        Ok(())
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::lacrimosa::skill::on_cast(ctx)
    }
    fn on_hit(&self, ctx: &mut crate::runtime_api::ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::lacrimosa::skill::on_hit(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::lacrimosa::awakenings::damage_bonus(ctx, bonus)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::lacrimosa::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::lacrimosa::buff_duration(&config.awakening, ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::lacrimosa::primary_packet(ability)
    }
}

pub(super) struct LinkoKit;
impl CharacterKit for LinkoKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::linko::SUPPORT)
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        crate::characters::linko::skill::on_defeat(actor);
    }
    fn resource_admission_failure(
        &self,
        request: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::linko::skill::resource_admission_failure(
            request, command, actor, ability, now,
        )
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::linko::skill::on_cast(ctx)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::linko::skill::on_hit(ctx)
    }
    fn observe_party_hit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        hit_index: usize,
        hits: &[crate::simulation::GameHitReceipt],
    ) -> Result<(), String> {
        crate::characters::linko::skill::observe_party_hit(ctx, owner, hit_index, hits)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::linko::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::linko::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::linko::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::linko::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::linko::buff_duration(&config.awakening, ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::linko::primary_packet(ability)
    }
}

pub(super) struct MintKit;
impl CharacterKit for MintKit {
    fn select_source_command(
        &self,
        _actor: &ActorRuntime,
        command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        crate::characters::mint::clock::source::select(command);
        Ok(())
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::mint::clock::secondary::validate_plunge(ctx)
    }
    fn validate_player_input(
        &self,
        req: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        crate::characters::mint::clock::secondary::validate_input(req, input)
    }
    fn resource_admission_failure(
        &self,
        req: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        _now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::mint::clock::secondary::admission(req, command, actor, ability)
    }
    fn entry_action(
        &self,
        req: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(crate::characters::mint::clock::secondary::select_support(
            req,
            state,
            new,
            qualified_support,
        )?
        .map(crate::runtime_api::OwnerEntryAction::Support))
    }
    fn on_party_entry(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        actor: usize,
        trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if owner == actor
            && crate::characters::mint::clock::accepted(ctx.request)
            && matches!(
                trigger,
                crate::simulation::GameCausalTrigger::FieldEntry { .. }
            )
        {
            ctx.state.actors[owner]
                .character
                .mint_mut()?
                .entry_normal_pending = true;
        }
        Ok(())
    }
    fn recovery_lock_override(
        &self,
        req: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::mint::clock::input_lock(req, actor, ability, now)
    }
    fn validate_interrupt(&self, req: &GameScenarioRequest) -> Result<(), String> {
        if crate::characters::mint::clock::accepted(req) {
            Ok(())
        } else {
            Err("Mint interruption requires ordinary_clock_v1".into())
        }
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::mint::clock::interrupt(ctx, owner, event)
    }
    fn observes_vitals(&self, request: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        crate::characters::mint::clock::accepted(request)
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        if ctx.state.actors[owner].hp == 0 {
            crate::characters::mint::clock::stop(
                &mut ctx.state.actors[owner],
                ctx.queue,
                ctx.state.current_time_us,
                ctx.state.combat_stamina.as_mut(),
            )?;
            let state = ctx.state.actors[owner].character.mint_mut()?;
            state.airborne = false;
            state.entry_normal_pending = false;
            state.riposte_ready = false;
        }
        Ok(())
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::mint::clock::stop(
            &mut ctx.state.actors[owner],
            ctx.queue,
            ctx.state.current_time_us,
            ctx.state.combat_stamina.as_mut(),
        )
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::mint::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        _request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        Some(crate::characters::mint::riposte::HYPOTHESIS_ID)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.record_hypothesis(crate::characters::mint::riposte::HYPOTHESIS_ID)?;
        ctx.actor.character.mint_mut()?.riposte_ready = true;
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        crate::characters::mint::riposte::validate(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        if ability == "GA_Mint019_QTE"
            && packet == crate::characters::mint::clock::secondary::SUPPORT_PACKET
        {
            return Some(crate::runtime_api::DirectPacketEnergyPolicy {
                trigger: crate::runtime_api::PacketEnergyTrigger::AppliedHpLoss,
                hypothesis_id: crate::characters::mint::clock::MODEL,
                event_kind: "mint_support_energy",
            });
        }
        crate::characters::mint::riposte::direct_packet_energy(ability, packet)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::mint::clock::accepted(ctx.request) {
            return crate::characters::mint::clock::cast(ctx);
        }
        if crate::characters::mint::riposte::on_cast(ctx)? {
            return Ok(());
        }
        ctx.actor.character.mint_mut()?.riposte_ready = false;
        crate::characters::mint::skill::on_cast(ctx)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::mint::clock::contact(ctx)?;
        crate::characters::mint::skill::on_hit(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::mint::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::mint::awakenings::crit_rate_bonus(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::mint::buff_duration(&config.awakening, ability)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::mint::primary_packet(ability)
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        if effect == "GE_Player_Mint_Skill1_Damage_New" {
            return Ok(Some(crate::runtime_api::PacketCycleGrant {
                raw: ExactFraction::new(41, 10)?,
                hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
            }));
        }
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
}

pub(super) struct NanallyKit;
impl CharacterKit for NanallyKit {
    fn entry_action(
        &self,
        _request: &GameScenarioRequest,
        _state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        _new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            qualified_support.then_some(crate::runtime_api::OwnerEntryAction::Support(
                &crate::characters::nanally::support::ENTRY,
            )),
        )
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::nanally::plunge::validate_input(ctx)
    }
    fn utility_invocation(
        &self,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::runtime_api::UtilityInvocation> {
        crate::characters::nanally::removal::select(actor, ability, now)
    }
    fn initialize(
        &self,
        _request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        let breakthrough = ntesim_data::skill_levels::progression(
            &config.character_id,
            config.character_level,
            config.character_max_level,
        )?["breakthrough"]
            .as_u64()
            .ok_or("Nanally progression lacks breakthrough")?;
        let state = actor.character.nanally_mut()?;
        state.fair_duel_unlocked = breakthrough >= 4;
        state.blossom_enhancement_unlocked = breakthrough >= 2;
        Ok(())
    }
    fn blossom_emission_profile(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::BlossomEmissionProfile> {
        crate::characters::nanally::blossom::profile(actor)
    }
    fn observe_cycle_hit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        hit_index: usize,
        hits: &[crate::simulation::GameHitReceipt],
    ) -> Result<(), String> {
        crate::characters::nanally::fair_duel::observe_cycle(ctx, owner, hit_index, hits)
    }
    fn observe_party_hit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        hit_index: usize,
        hits: &[crate::simulation::GameHitReceipt],
    ) -> Result<(), String> {
        crate::characters::nanally::fair_duel::observe_direct(ctx, owner, hit_index, hits)
    }

    fn validate_interrupt(&self, request: &GameScenarioRequest) -> Result<(), String> {
        crate::characters::nanally::clock::validate_interrupt(request)
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::nanally::clock::interrupt(ctx, owner, event)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::nanally::clock::contact(ctx)
    }
    fn recovery_lock_override(
        &self,
        _request: &GameScenarioRequest,
        actor: &ActorRuntime,
        ability: &str,
        _now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::nanally::clock::recovery(actor, ability)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::nanally::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        _request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        Some(crate::characters::nanally::riposte::HYPOTHESIS_ID)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        ctx.record_hypothesis(crate::characters::nanally::riposte::HYPOTHESIS_ID)?;
        ctx.actor.character.nanally_mut()?.riposte_ready = true;
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        crate::characters::nanally::riposte::validate(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::nanally::riposte::direct_packet_energy(ability, packet)
            .or_else(|| crate::characters::nanally::support::direct_packet_energy(ability, packet))
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::nanally::riposte::on_cast(ctx)? {
            return Ok(());
        }
        if crate::characters::nanally::clock::held::resume_cast(ctx)? {
            return Ok(());
        }
        ctx.actor.character.nanally_mut()?.riposte_ready = false;
        ctx.actor.character.nanally_mut()?.heavy_resume = None;
        crate::characters::nanally::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::nanally::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::nanally::awakenings::crit_bonus(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::nanally::buff_duration(&config.awakening, ability)
    }
    fn on_hp_loss_hit(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::nanally::authority::on_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::nanally::primary_packet(ability)
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        if let Ok(state) = actor.character.nanally_mut() {
            state.clock = Default::default();
            state.underboss_until_us = 0;
            state.underboss_coordinate = None;
            state.authority_until_us = 0;
            state.authority_command = None;
            state.authority_started_us = 0;
            state.authority_recast_unlock_us = 0;
            state.normal_stage = 0;
            state.normal_started_us = None;
            state.riposte_ready = false;
            state.heavy_resume = None;
            state.a5_stacks = 0;
            state.a5_until_us = 0;
        }
    }
}

pub(super) struct SakiriKit;
impl CharacterKit for SakiriKit {
    fn entry_action(
        &self,
        _request: &GameScenarioRequest,
        _state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        _new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            qualified_support.then_some(crate::runtime_api::OwnerEntryAction::Support(
                &crate::characters::sakiri::support::ENTRY,
            )),
        )
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::sakiri::riposte::replacement(actor)
    }
    fn critical_dodge_model(
        &self,
        request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        crate::characters::sakiri::riposte::accepted(request)
            .then_some(crate::characters::sakiri::riposte::HYPOTHESIS_ID)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::sakiri::riposte::accepted(ctx.request) {
            ctx.record_hypothesis(crate::characters::sakiri::riposte::HYPOTHESIS_ID)?;
            ctx.actor.character.sakiri_mut()?.riposte_ready = true;
        }
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        crate::characters::sakiri::riposte::validate(actor, ability, packet)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::sakiri::riposte::direct_packet_energy(ability, packet)
            .or_else(|| crate::characters::sakiri::support::direct_packet_energy(ability, packet))
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        if crate::characters::sakiri::riposte::on_cast(ctx)? {
            return Ok(());
        }
        ctx.actor.character.sakiri_mut()?.riposte_ready = false;
        crate::characters::sakiri::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::sakiri::awakenings::damage_bonus(ctx, bonus)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::sakiri::buff_duration(&config.awakening, ability)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::sakiri::skill::on_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::sakiri::primary_packet(ability)
    }
}

pub(super) struct ShinkuKit;
impl CharacterKit for ShinkuKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::shinku::SUPPORT)
    }
    fn select_source_command(
        &self,
        actor: &ActorRuntime,
        command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        if let crate::simulation::GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            energy_cost,
            cooldown_us,
            recovery_us,
            timestamp_us,
            ..
        } = command
            && let Ok(state) = actor.character.shinku()
            && state.surging_crimson_until_us > *timestamp_us
        {
            if ability_id == "GA_Shinku_Skill" {
                if state.fading_reason >= 12 && state.scarlet_descent_casts < 5 {
                    *damage_effect_id = "GE_Player_Shinku_Skill1_Rage_Damage".into();
                    *cooldown_us = 0;
                    *energy_cost = 0;
                    *recovery_us = 800_000;
                }
            } else if ability_id == "GA_Shinku_UltraSkill"
                && state.surging_crimson_until_us.saturating_sub(*timestamp_us) <= 4_000_000
            {
                if state.crimson_judgment_stacks > 0 {
                    *damage_effect_id = "GE_Player_Shinku_UltraSkillPre_Rage_Damage".into();
                    *cooldown_us = 0;
                    *energy_cost = 0;
                    *recovery_us = 400_000;
                } else {
                    *damage_effect_id = "GE_Player_Shinku_UltraSkill_Rage_Damage".into();
                    *cooldown_us = state.paid_burst_until_us.saturating_sub(*timestamp_us);
                    *energy_cost = 0;
                    *recovery_us = 1_200_000;
                }
            }
        }
        Ok(())
    }
    fn resource_admission_failure(
        &self,
        request: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        let state = actor.character.shinku().ok()?;
        let in_rage = state.surging_crimson_until_us > now;
        if in_rage {
            if let Some(crate::simulation::GameActionCommand::InvokeAbility {
                damage_effect_id,
                ..
            }) = request.actions.get(command)
                && (damage_effect_id == "GE_Player_Shinku_Skill1_1_Damage"
                    || damage_effect_id == "GE_Player_Shinku_UltraSkill1_Damage"
                    || damage_effect_id == "GE_Player_Shinku_UltraSkillPre_Rage_Damage"
                    || damage_effect_id == "GE_Player_Shinku_UltraSkill_Rage_Damage")
            {
                return None;
            }
            if ability == "GA_Shinku_Skill" {
                if state.fading_reason < 12 {
                    return Some(crate::simulation::rejection::AdmissionFailure {
                        reason: crate::simulation::GameAdmissionReason::Other {
                            message: "Shinku Scarlet Descent requires 12 Fading Reason".into(),
                        },
                        message: "Shinku Scarlet Descent requires 12 Fading Reason".into(),
                    });
                }
                if state.scarlet_descent_casts >= 5 {
                    return Some(crate::simulation::rejection::AdmissionFailure {
                        reason: crate::simulation::GameAdmissionReason::Other {
                            message: "Shinku Scarlet Descent maximum 5 casts per stance reached"
                                .into(),
                        },
                        message: "Shinku Scarlet Descent maximum 5 casts per stance reached".into(),
                    });
                }
            } else if ability == "GA_Shinku_UltraSkill" {
                let remaining = state.surging_crimson_until_us.saturating_sub(now);
                if remaining > 4_000_000 {
                    let ready_at = state.surging_crimson_until_us - 4_000_000;
                    return Some(crate::simulation::rejection::AdmissionFailure {
                        reason: crate::simulation::GameAdmissionReason::Cooldown {
                            until_us: ready_at,
                        },
                        message: format!("Shinku Crimson Judgment is locked until {ready_at}us"),
                    });
                }
            }
        }
        None
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::shinku::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::shinku::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::shinku::awakenings::crit_bonus(ctx)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::shinku::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::shinku::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::shinku::buff_duration(ability)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::shinku::skill::on_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::shinku::primary_packet(ability)
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        if let Ok(state) = actor.character.shinku_mut() {
            state.surging_crimson_until_us = 0;
            state.normal_stage = 0;
            state.normal_started_us = None;
            state.fading_reason = 0;
            state.scarlet_descent_casts = 0;
            state.crimson_judgment_stacks = 0;
        }
    }
}

pub(super) struct SkiaKit;
impl CharacterKit for SkiaKit {
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        Some(&crate::characters::skia::SUPPORT)
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        actor: usize,
    ) -> Result<(), String> {
        crate::characters::skia::entry_state::leave(ctx, actor);
        Ok(())
    }
    fn validate_incoming_hp(
        &self,
        _ctx: &crate::runtime_api::IncomingHpContext<'_>,
    ) -> Result<(), String> {
        Ok(())
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::skia::entry_state::on_hit(ctx)?;
        crate::characters::skia::skill::on_hit(ctx)?;
        Ok(())
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        if let crate::characters::RuntimeState::Skia(state) = &mut actor.character {
            *state = Default::default();
        }
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::skia::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::skia::awakenings::damage_bonus(ctx, bonus)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::skia::buff_duration(&config.awakening, ability)
    }
    fn has_before_hit(&self) -> bool {
        true
    }
    fn before_hit(&self, ctx: &mut HitContext<'_>) -> Result<(), String> {
        crate::characters::skia::skill::before_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::skia::primary_packet(ability)
    }
}

pub(super) struct ZankouKit;
impl CharacterKit for ZankouKit {
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        ability: &str,
        packet: &str,
        _now: u64,
    ) -> Result<(), String> {
        if ability == "GA_Zankou_Skill" && packet != "GE_Player_Zankou_Skill2_Damage" {
            return Err("unsupported Zankou Skill variant: submit the canonical Skill2 input; the owner derives the earned variant".into());
        }
        Ok(())
    }
    fn observes_vitals(&self, _req: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        true
    }
    fn on_vitals_changed(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        _changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        crate::characters::zankou::vile_ash::owner_defeated(ctx, owner)?;
        crate::characters::zankou::modifiers::vitals(ctx, owner)
    }
    fn initialize(
        &self,
        _request: &GameScenarioRequest,
        config: &GameActorConfig,
        actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        actor.character.zankou_mut()?.a4 = config.awakening.has_effect(4);
        actor.character.zankou_mut()?.a1 = config.awakening.has_effect(1);
        Ok(())
    }

    fn entry_action(
        &self,
        req: &GameScenarioRequest,
        state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            crate::characters::zankou::secondary::select(req, state, new, qualified_support)?
                .map(crate::runtime_api::OwnerEntryAction::Support),
        )
    }
    fn validate_plunge(
        &self,
        ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        crate::characters::zankou::secondary::validate_plunge(ctx)
    }
    fn normal_action_override(
        &self,
        actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        crate::characters::zankou::secondary::replacement(actor)
    }
    fn critical_dodge_model(&self, _req: &GameScenarioRequest) -> Option<crate::HypothesisId> {
        Some(crate::characters::zankou::forms::MODEL)
    }
    fn on_critical_dodge(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::zankou::secondary::critical(ctx)
    }
    fn validate_player_input(
        &self,
        req: &GameScenarioRequest,
        input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        crate::characters::zankou::secondary::validate_input(req, input)
    }
    fn resource_admission_failure(
        &self,
        req: &GameScenarioRequest,
        command: usize,
        actor: &ActorRuntime,
        ability: &str,
        now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        crate::characters::zankou::forms::admission(req, command, ability)
            .or_else(|| {
                crate::characters::zankou::secondary::admission(req, command, actor, ability)
            })
            .or_else(|| {
                crate::characters::zankou::burst::admission(req, command, actor, ability, now)
            })
    }
    fn on_defeat(&self, actor: &mut ActorRuntime) {
        if let Ok(state) = actor.character.zankou_mut() {
            state.form = crate::characters::zankou::state::Form::Reality;
            state.form_until_us = 0;
            state.pending_skill = None;
            state.clock = None;
            state.bloodfeast_ready = false;
            state.free_inferno_until_us = 0;
            state.kindling_active_for_ultra = false;
            state.kindling_stacks = 0;
            state.healing_next_us = None;
            state.r6_enhancement_until_us = 0;
            state.hunt_until_us = 0;
            state.hunt_retained_until_us = 0;
            state.delusion_until_us = 0;
            state.delusion_retained_until_us = 0;
            state.normal_stage = 0;
            state.normal_started_us = None;
        }
    }
    fn recovery_lock_override(
        &self,
        _request: &GameScenarioRequest,
        actor: &ActorRuntime,
        input: &str,
        now: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        crate::characters::zankou::clock::recovery(actor, input, now)
    }
    fn select_source_command(
        &self,
        actor: &ActorRuntime,
        command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        crate::characters::zankou::forms::select(actor, command);
        crate::characters::zankou::burst::select(actor, command);
        Ok(())
    }
    fn on_field_exit(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
    ) -> Result<(), String> {
        crate::characters::zankou::forms::field_exit(ctx, owner)
    }
    fn validate_interrupt(&self, _request: &GameScenarioRequest) -> Result<(), String> {
        Ok(())
    }
    fn interrupt(
        &self,
        ctx: &mut crate::runtime_api::CallbackContext<'_>,
        owner: usize,
        event: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        crate::characters::zankou::forms::interrupt(ctx, owner, event)
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
            .map(|raw| {
                Ok(crate::runtime_api::PacketCycleGrant {
                    raw: ExactFraction::new(raw.numerator(), raw.denominator())?,
                    hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
                })
            })
            .transpose()
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::zankou::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::zankou::awakenings::damage_bonus(ctx, bonus)
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::zankou::buff_duration(ability)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::zankou::skill::on_hit(ctx)
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::zankou::skill::direct_packet_energy(ability, packet)
    }
    fn validate_packet_delivery(
        &self,
        source_config: &GameActorConfig,
        source_actor: &ActorRuntime,
        packet: &PacketMetadata,
        target_has_scorch: bool,
        scorch_replacement_modeled: bool,
        target_would_be_defeated: bool,
    ) -> Result<(), String> {
        crate::characters::zankou::skill::validate_packet_delivery(
            source_config,
            source_actor,
            packet,
            target_has_scorch,
            scorch_replacement_modeled,
            target_would_be_defeated,
        )
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::zankou::primary_packet(ability)
    }
}

pub(super) struct ZeroKit;
impl CharacterKit for ZeroKit {
    fn entry_action(
        &self,
        _request: &GameScenarioRequest,
        _state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        _new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(
            qualified_support.then_some(crate::runtime_api::OwnerEntryAction::Support(
                &crate::characters::zero::support::ENTRY,
            )),
        )
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        ability: &str,
        packet_effect: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        match (ability, packet_effect) {
            ("GA_Female046_Melee", p)
                if !matches!(
                    p,
                    "GE_Player_Female046_Melee1_Damage"
                        | "GE_Player_Female046_Melee2_Damage"
                        | "GE_Player_Female046_Melee2_1_Damage"
                        | "GE_Player_Female046_Melee3_Damage"
                        | "GE_Player_Female046_Melee4_Damage"
                        | "GE_Player_Female046_Melee5_Damage"
                        | "GE_Player_Female046_Melee6_Damage"
                        | "GE_Player_Female046_Melee7_Damage"
                        | "GE_Player_Female046_Melee8_Damage"
                ) =>
            {
                Err(format!(
                    "packet '{p}' is not admitted for Zero Normal chain"
                ))
            }
            ("GA_Female046_Skill", p)
                if !matches!(
                    p,
                    "GE_Player_Female046_Skill1_Damage"
                        | "GE_Player_Female046_Skill2_Damage"
                        | "GE_Player_Female046_Skill3_Damage"
                        | "GE_Player_Female046_Skill4_Damage"
                        | "GE_Player_Female046_Skill_Kill_Damage_lv1"
                        | "GE_Player_Female046_Skill_Kill_Damage_lv2"
                ) =>
            {
                Err(format!("packet '{p}' is not admitted for Zero Skill"))
            }
            ("GA_Female046_UltraSkill", p)
                if !matches!(
                    p,
                    "GE_Player_Female046_UltraSkill1_Damage"
                        | "GE_Player_Female046_UltraSkill2_Damage"
                        | "GE_Player_Female046_UltraSkill3_Damage"
                        | "GE_Player_Female046_UltraSkill4_Damage"
                ) =>
            {
                Err(format!("packet '{p}' is not admitted for Zero Burst"))
            }
            ("GA_Female046_QTE", p)
                if !matches!(
                    p,
                    "GE_Player_Female046_QTE1_Damage" | "GE_Player_Female046_QTE2_Damage"
                ) =>
            {
                Err(format!("packet '{p}' is not admitted for Zero Support"))
            }
            _ => Ok(()),
        }
    }
    fn direct_packet_energy(
        &self,
        ability: &str,
        packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        crate::characters::zero::support::direct_packet_energy(ability, packet)
    }
    fn on_cast(&self, ctx: &mut CastContext<'_>) -> Result<(), String> {
        crate::characters::zero::skill::on_cast(ctx)
    }
    fn damage_bonus(
        &self,
        ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        crate::characters::zero::awakenings::damage_bonus(ctx, bonus)
    }
    fn crit_rate_bonus(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::zero::awakenings::crit_rate_bonus(ctx)
    }
    fn defense_ignore(&self, ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        crate::characters::zero::awakenings::defense_ignore(ctx)
    }
    fn cast_buff_duration(&self, config: &GameActorConfig, ability: &str) -> Option<u64> {
        crate::characters::zero::buff_duration(&config.awakening, ability)
    }
    fn on_hit(&self, ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        crate::characters::zero::skill::on_hit(ctx)
    }
    fn primary_packet(&self, ability: &str) -> Option<&'static str> {
        crate::characters::zero::primary_packet(ability)
    }
}
