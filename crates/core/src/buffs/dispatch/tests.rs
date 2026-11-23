use super::*;

fn synthetic_buff(scope: BuffStackingScope, policy: RefreshPolicy) -> BuffDefinition {
    BuffDefinition {
        stacking_scope: scope,
        semantic_tags: Default::default(),
        source_lifetime: Default::default(),
        id: "synthetic-carrier".into(),
        source: "first".into(),
        recipient: BuffRecipient::TeamAll,
        duration_us: Some(10),
        refresh_policy: policy,
        max_stacks: 1,
        decay_policy: DecayPolicy::AllAtOnce,
        stat_modifiers: BTreeMap::from([(
            "atk".into(),
            Ratio {
                numerator: 7,
                denominator: 2,
            },
        )]),
    }
}

mod arbitrary_carriers_share_real_trigger_basis_and_hit_count_lifecycle;
mod arbitrary_icd_ids_do_not_alias_and_unreachable_timer_deadlines_never_fire;
mod buff_recipients_and_active_actor_switch;
mod buff_source_identity_independent_caps_target_recipients_and_sparse_decay;
mod buff_stacking_and_decay_policies;
mod dispatch_nested_rules_reject_unknown_fields;
mod dispatch_recursion_guard_and_child_procs;
mod dispatch_scenario_rejects_source_lifetime_without_actor_liveness_input;
mod evaluate_dispatch_scenario_receipt_matches;
mod extend_refreshes_snapshot_but_disallow_retains_original_snapshot;
mod failed_dispatch_rolls_back_every_proc_and_does_not_consume_icd_or_resources;
mod icd_invalid_rule_and_reversed_time_preserve_state;
mod icd_per_target_and_global_scopes;
mod icd_read_only_eligibility_predicts_the_same_next_event_for_every_rule;
mod icd_standard_3hit_2_5s_rule;
mod incompatible_or_overflowing_reapplication_is_atomic_even_after_expiry;
mod owned_removal_keeps_other_sources_owners_and_effect_ids;
mod recipient_queries_apply_source_lifetime_without_changing_snapshots;
mod sequential_stack_decay_requires_representable_final_expiry;
mod shared_and_per_source_applications_refresh_exact_snapshots_and_expire;
mod switch_commands_and_events_share_field_state_and_callback_execution;
mod tagged_target_stack_query_is_target_scoped_and_excludes_other_semantics;
mod target_death_resolves_independent_target_icd;
