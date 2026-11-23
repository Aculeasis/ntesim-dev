# Design

## Context

See `proposal.md` for background.

Currently in `crates/content/src/characters/lacrimosa/skill.rs`, `on_hit` contains logic that checks:
```rust
let is_dot_or_shatter = matches!(
    ctx.damage_effect_id,
    "GE_Player_Lacrimosa_Blood_Damage"
        | "GE_Player_Lacrimosa_Blood_Damage_LV6"
        | "GE_Player_Lacrimosa_B_Melee3_Explode_Damage"
);
let has_discord = is_broken
    || ctx
        .buffs
        .active_tagged_stacks_for_target_index(
            ctx.target_index,
            "ntesim.daffodil.discord_cap",
            ctx.world_time_us,
        )
        .is_ok_and(|s| s > 0);

if is_dot_or_shatter && has_discord {
    schedule_an_hun_zhou_two(...);
}
```

This logic incorrectly treats any target in the `Broken` state (`is_broken == true`) as having a Discord reaction (`has_discord = is_broken`). Consequently, every single tick of Nightmare DoT during a target's Broken phase triggers a 400% ATK strike (`GE_Player_Lacrimosa_AnHunZhouTwo_Damage`).

In reality, the breakthrough 2 passive Tomato Banquet (蕃茄盛宴) states that the 400% ATK damage triggers **when a Discord reaction occurs**, provided the target is currently Broken. It is a reaction-driven proc, not a periodic DoT proc.

## Goals / Non-Goals

**Goals:**
- Eliminate the periodic DoT trigger in `on_hit` during target Break.
- Ensure `on_discord` callback triggers Tomato Banquet (`schedule_an_hun_zhou_two`) only when the target is Broken at the time of the Discord reaction.
- Assign human-readable name `"Tomato Banquet"` to `GA_lacrimosa_Passive_1` across `crates/data/src/ability_titles.rs` and web localization.
- Update tests to assert that periodic DoT on Broken targets does not trigger `AnHunZhouTwo_Damage`, while genuine Discord reactions on Broken targets do trigger it.

**Non-Goals:**
- Altering the stacking, duration, or base damage coefficient of Nightmare DoT (`GE_Player_Lacrimosa_Blood_Damage`), which correctly follows the `RemoveSingleStackAndRefreshDuration` decay model.
- Changing general Discord reaction mechanics or gauge deduction formulas.

## Decisions

### 1. Remove DoT Hook in `on_hit` and Guard `on_discord`
- **Choice**: Delete the block in `on_hit` that checks `is_dot_or_shatter && has_discord`. In `pub(crate) fn on_discord(...)`, check whether `ctx.state.targets[target_index]` is currently in the Broken state (`broken_until_us > ctx.state.current_time_us`). If Broken, schedule `schedule_an_hun_zhou_two`. If not Broken, do not schedule.
- **Alternative considered**: Keeping an opt-in model flag for DoT-triggered Discord. Rejected: game tooltips and mechanics dossiers confirm Tomato Banquet is strictly a Discord reaction extension, and no source indicates DoT ticks should proc 400% ATK hits.

### 2. Ability and Effect Naming
- **Choice**: Add `AbilityTitle { ability_id: "GA_lacrimosa_Passive_1", title: "Tomato Banquet" }` to `crates/data/src/ability_titles.rs` and update `web/src/generated/names.json`.
- In `web/src/domain/reports/source-label.ts`, if `source.ability_id === "GA_lacrimosa_Passive_1"`, the lookup will now cleanly resolve to `"Tomato Banquet"` instead of falling back to raw IDs.

### 3. Test Alignment
- **Choice**: Update `lacrimosa_broken_nightmare_passive_retains_its_actual_damage_parent` in `crates/content/src/simulation/tests/lacrimosa_public.rs` (which was explicitly testing the erroneous DoT proc) to instead verify that in `bohemian-rose-break-cycles.nte` (a team with no Discord reaction), `AnHunZhouTwo_Damage` is NOT scheduled.
- Retain and verify `daffodil_break_discord.rs` which exercises genuine Discord reactions with Daffodil and Lacrimosa on Broken targets.

## Risks / Trade-offs

- [Preset and baseline numbers change] → `blackbird-hathor.nte` and other presets featuring Lacrimosa will see reduced total damage and lower Lacrimosa DPS. This is an intentional bug fix bringing simulation into alignment with game mechanics.
- [Downstream tests expecting older receipts] → Run focused tests and verify all affected simulator tests are updated to the corrected behavior.
