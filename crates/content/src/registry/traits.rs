use crate::runtime_api::{
    ActorHitContext, BreakContext, BreakModifiers, CastContext, DamageContext, HitContext,
    HitModifiers, RecipientContext,
};
use crate::simulation::metadata::PacketMetadata;
use crate::simulation::state::{ActorRuntime, TargetRuntime};
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest};

pub(crate) trait CharacterKit: Sync {
    /// Live attached carriers outside the shared buff stack manager.
    fn attached_dot_stacks(&self, _actor: &ActorRuntime, _target: usize, _now: u64) -> u32 {
        0
    }

    fn select_source_command(
        &self,
        _request: &GameScenarioRequest,
        _actor: &ActorRuntime,
        _command: &mut crate::simulation::GameActionCommand,
    ) -> Result<(), String> {
        Ok(())
    }
    fn utility_invocation(
        &self,
        _actor: &ActorRuntime,
        _ability: &str,
        _now: u64,
    ) -> Option<crate::runtime_api::UtilityInvocation> {
        None
    }
    fn blossom_emission_profile(
        &self,
        _actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::BlossomEmissionProfile> {
        None
    }
    fn validate_hold_stock_selection(
        &self,
        input: &crate::simulation::GameHoldInput,
    ) -> Result<(), String> {
        if input.reserved_stocks.is_some() {
            Err("hold reserved_stocks has no supported owner binding".into())
        } else {
            Ok(())
        }
    }
    fn resource_admission_failure(
        &self,
        _request: &GameScenarioRequest,
        _command: usize,
        _actor: &ActorRuntime,
        _ability: &str,
        _now: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        None
    }
    fn skill_gear_trigger_on_finish(&self, _request: &GameScenarioRequest) -> bool {
        false
    }
    fn validate_player_input_state(
        &self,
        _request: &GameScenarioRequest,
        _actor: &ActorRuntime,
    ) -> Result<(), String> {
        Ok(())
    }
    fn validate_plunge(
        &self,
        _ctx: &crate::runtime_api::PlungeInputContext<'_>,
    ) -> Result<(), String> {
        Err("no supported owner plunge binding".into())
    }
    fn on_player_input(
        &self,
        _request: &GameScenarioRequest,
        _actor: &mut ActorRuntime,
        _input: crate::simulation::PlayerInput,
    ) -> Result<(), String> {
        Ok(())
    }
    fn observe_party_hit(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _owner: usize,
        _hit_index: usize,
        _hits: &[crate::simulation::GameHitReceipt],
    ) -> Result<(), String> {
        Ok(())
    }
    /// Settled positive Esper Cycle damage, separate from ordinary packet hits.
    fn observe_cycle_hit(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _owner: usize,
        _hit_index: usize,
        _hits: &[crate::simulation::GameHitReceipt],
    ) -> Result<(), String> {
        Ok(())
    }
    fn initialize(
        &self,
        _request: &GameScenarioRequest,
        _config: &GameActorConfig,
        _actor: &mut ActorRuntime,
    ) -> Result<(), String> {
        Ok(())
    }
    fn observes_vitals(&self, _request: &GameScenarioRequest, _config: &GameActorConfig) -> bool {
        false
    }
    fn on_vitals_changed(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _owner: usize,
        _changes: &[crate::simulation::vitals::HpChange],
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        Ok(())
    }
    fn on_party_entry(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _owner: usize,
        _entering: usize,
        _trigger: crate::simulation::GameCausalTrigger,
    ) -> Result<(), String> {
        Ok(())
    }
    fn on_connected_packet(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _owner: usize,
        _key: usize,
        _group: Option<&str>,
        _target_index: usize,
        _effect: &str,
    ) -> Result<(), String> {
        Ok(())
    }
    fn source_atk_percent(
        &self,
        _request: &GameScenarioRequest,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        _now: u64,
    ) -> Result<ExactFraction, String> {
        ExactFraction::new(0, 1)
    }
    fn on_field_exit(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _actor: usize,
    ) -> Result<(), String> {
        Ok(())
    }
    fn validate_incoming_hp(
        &self,
        _ctx: &crate::runtime_api::IncomingHpContext<'_>,
    ) -> Result<(), String> {
        Ok(())
    }
    fn entry_action(
        &self,
        _request: &GameScenarioRequest,
        _state: &crate::simulation::state::GameSimulatorState,
        _old: usize,
        _new: usize,
        qualified_support: bool,
    ) -> Result<Option<crate::runtime_api::OwnerEntryAction>, String> {
        Ok(self
            .single_packet_support()
            .filter(|_| qualified_support)
            .map(|profile| crate::runtime_api::OwnerEntryAction::Support(&profile.entry)))
    }
    fn single_packet_support(&self) -> Option<&'static crate::runtime_api::SinglePacketSupport> {
        None
    }
    fn on_single_support_contact(&self, _actor: &mut ActorRuntime) -> Result<(), String> {
        Ok(())
    }
    fn validate_interrupt(&self, _request: &GameScenarioRequest) -> Result<(), String> {
        Err("unsupported owner interruption model".into())
    }
    fn interrupt(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _actor_index: usize,
        _event_index: usize,
    ) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
        Err("unsupported owner interruption model".into())
    }
    fn recovery_lock_override(
        &self,
        _request: &GameScenarioRequest,
        _actor: &ActorRuntime,
        _ability_id: &str,
        _now_us: u64,
    ) -> Option<(u64, crate::HypothesisId)> {
        None
    }
    fn packet_cycle_grant(
        &self,
        _request: &GameScenarioRequest,
        _effect: &str,
    ) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
        Ok(None)
    }
    fn validate_player_input(
        &self,
        _request: &GameScenarioRequest,
        _input: crate::simulation::PlayerInput,
    ) -> Result<Option<crate::HypothesisId>, String> {
        Ok(None)
    }
    fn on_defeat(&self, _actor: &mut ActorRuntime) {}
    fn incoming_damage_immune(&self, _actor: &ActorRuntime, _now_us: u64) -> bool {
        false
    }
    fn intercept_fatal_hit(
        &self,
        _ctx: &mut crate::runtime_api::IncomingHpContext<'_>,
        _vitals: &mut ntesim_core::state::VitalState,
    ) -> Result<bool, String> {
        Ok(false)
    }
    fn finishing_damage(
        &self,
        _ctx: &crate::runtime_api::FinishHitContext<'_>,
        _owner: usize,
    ) -> Result<Option<crate::runtime_api::FinishDamage>, String> {
        Ok(None)
    }
    fn normal_action_override(
        &self,
        _actor: &ActorRuntime,
    ) -> Option<crate::runtime_api::NormalActionOverride> {
        None
    }
    fn critical_dodge_model(
        &self,
        _request: &crate::simulation::GameScenarioRequest,
    ) -> Option<crate::HypothesisId> {
        None
    }
    fn on_critical_dodge(&self, _ctx: &mut CastContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn validate_ability_admission(
        &self,
        _request: &GameScenarioRequest,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        _ability: &str,
    ) -> Result<(), String> {
        Ok(())
    }
    fn validate_packet_admission(
        &self,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        _ability: &str,
        _packet_effect: &str,
        _time_us: u64,
    ) -> Result<(), String> {
        Ok(())
    }
    fn on_cast(&self, _ctx: &mut CastContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn damage_bonus(
        &self,
        _ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        Ok(bonus)
    }
    fn crit_bonus(&self, _ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        ExactFraction::new(0, 1)
    }
    fn crit_rate_bonus(&self, _ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        ExactFraction::new(0, 1)
    }
    fn defense_ignore(&self, _ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
        ExactFraction::new(0, 1)
    }
    fn break_efficiency_bonus(
        &self,
        _config: &GameActorConfig,
        _actor: &ActorRuntime,
        _command_index: usize,
        _ability: &str,
    ) -> Result<ExactFraction, String> {
        ExactFraction::new(0, 1)
    }
    fn break_modifiers(&self, _ctx: &BreakContext<'_>) -> Result<BreakModifiers, String> {
        BreakModifiers::neutral()
    }
    fn cast_buff_duration(&self, _config: &GameActorConfig, _ability: &str) -> Option<u64> {
        None
    }
    fn on_hit(&self, _ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn on_finish(
        &self,
        _ctx: &mut crate::runtime_api::CallbackContext<'_>,
        _owner: usize,
        _target: usize,
    ) -> Result<(), String> {
        Ok(())
    }
    fn direct_packet_energy(
        &self,
        _ability: &str,
        _packet: &str,
    ) -> Option<crate::runtime_api::DirectPacketEnergyPolicy> {
        None
    }
    fn validate_packet_delivery(
        &self,
        _source_config: &GameActorConfig,
        _source_actor: &ActorRuntime,
        _packet: &PacketMetadata,
        _target_has_scorch: bool,
        _scorch_replacement_modeled: bool,
        _target_would_be_defeated: bool,
    ) -> Result<(), String> {
        Ok(())
    }
    fn on_hp_loss_hit(&self, _ctx: &mut CastContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn validate_hit_target(
        &self,
        _request: &GameScenarioRequest,
        _config: &GameActorConfig,
        _ability_id: &str,
        _packet: &PacketMetadata,
        _target: &TargetRuntime,
    ) -> Result<(), String> {
        Ok(())
    }
    fn has_before_hit(&self) -> bool {
        false
    }
    fn before_hit(&self, _ctx: &mut HitContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn recipient_modifiers(
        &self,
        _ctx: &RecipientContext<'_>,
        value: HitModifiers,
    ) -> Result<HitModifiers, String> {
        Ok(value)
    }
    fn primary_packet(&self, _ability: &str) -> Option<&'static str> {
        None
    }
}

pub(crate) trait WeaponPassive: Sync {
    fn weapon_id(&self) -> &'static str;
    fn on_cast(&self, _ctx: &mut CastContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn on_hit(&self, _ctx: &mut CastContext<'_>, _critical: bool) -> Result<(), String> {
        Ok(())
    }
    fn damage_bonus(
        &self,
        _ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        Ok(bonus)
    }
}

pub(crate) trait SetEffect: Sync {
    fn set_id(&self) -> &'static str;
    fn on_cast(&self, _ctx: &mut CastContext<'_>) -> Result<(), String> {
        Ok(())
    }
    fn on_hit(&self, _ctx: &mut CastContext<'_>, _critical: bool) -> Result<(), String> {
        Ok(())
    }
    fn damage_bonus(
        &self,
        _ctx: &DamageContext<'_>,
        bonus: HitModifiers,
    ) -> Result<HitModifiers, String> {
        Ok(bonus)
    }
}
