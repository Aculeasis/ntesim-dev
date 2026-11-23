# Design

## Context

Simulation aggregate reports categorize damage by `(actor_index, ability_id, effect_id)` tuples. In the web studio (`web/src/domain/reports/source-label.ts`), `sourceLabel` transforms this tuple into a user-facing string for the Source DPS and Source Damage Instances bar charts.

Currently, `source-label.ts` only defines 8 hardcoded effect overrides and 4 reaction cycle overrides. When an ability has a known title (e.g. Zankou's "Inferno Flamenco", "Sanguine Dash", or "Wildfire") but its `effect_id` is not in the hardcoded map, `sourceLabel` discards `effect_id` and returns only the ability title. Because characters often have dozens of damage packets under one ability ID (for example, Zankou has 32 packets under `GA_Zankou_Melee`, 10 under `GA_Zankou_Skill`, and 7 under `GA_Zankou_UltraSkill`), every delivered packet under that ability renders as an identical bar on the chart with the exact same name, duplicating up to 10 times in a single chart.

Furthermore, secondary actions (such as `GA_Zankou_PerfectEvade` or `GA_Lacrimosa_AirAttack`) are missing from `names.json`, causing them to fall back to raw engine identifiers like `GA_Zankou_PerfectEvade · GE_Player_Zankou_PerfectEvadeAttack_Damage`.

## Goals / Non-Goals

**Goals:**
- Provide clear, descriptive labels for all character damage effects, multi-hit combo hits, branches, stance attacks, and character DoTs.
- Eliminate identical duplicate rows on Source DPS and Source Damage Instances charts.
- Curate explicit human-readable names for known abilities, stances, DoTs, awakenings, and weapon procs.
- Provide a robust, systematic humanization fallback for any uncurated effect IDs so that all 592+ existing and future damage effects are cleanly readable without technical prefixes/suffixes.
- Ensure secondary action IDs (Air Attacks, Dodge Counters, special mode skills) display human-friendly names rather than raw `GA_...` strings.

**Non-Goals:**
- Modifying simulation mechanics, hit settlement, or Rust engine data tables.
- Merging distinct damage source rows into single bars in the engine aggregate. The engine records separate statistics per packet for granular analysis; this design preserves granular visibility while fixing presentation clarity.

## Decisions

### Decision 1: Two-Tier Label Resolution (Curated Dictionary + Algorithmic Fallback)
We use a two-tiered resolution strategy in `source-label.ts`:
1. **Tier 1 (Curated Dictionary)**: An explicit mapping of well-known skill mechanics, DoTs, awakenings, weapon procs, and stance transitions where game lore or kit documentation provides specific terminology:
   - Zankou:
     - `GE_Player_Zankou_DotDamage` $\rightarrow$ `Heartwrench (DoT)`
     - `GE_Player_Zankou_DotUltraDamage` $\rightarrow$ `Vile Ash (DoT)`
     - `GE_Player_Zankou_MagicUltraSkill1/2_Damage` $\rightarrow$ `Bloodfeast Reverie (Flurry / Finisher)`
     - `GE_Player_Zankou_ForceUltraSkill1/2_Damage` $\rightarrow$ `Enhanced Hit 1 / Enhanced Finisher`
     - `GE_Player_Zankou_Skill3/4_Damage` series $\rightarrow$ `Soulcross`
     - `GE_Player_Zankou_Skill1/2_Damage` series $\rightarrow$ `Sanguine Dash`
   - Lacrimosa:
     - `GE_Player_Lacrimosa_Blood_Damage` $\rightarrow$ `Nightmare (DoT)`
     - `GE_Player_Lacrimosa_B_Melee3_Explode_Damage` $\rightarrow$ `Shatter`
     - Bat stance combos, Pan attack, and AnHunZhou
   - Oneiroi:
     - Gun, Syringe, Trumpet lamb damage packets
   - Weapon procs:
     - `GE_Fork_serenity_Skill1_Damage1` $\rightarrow$ `Harsh Sound`
     - `GE_Fork_serenity_Skill2_Damage` $\rightarrow$ `Nagging Sound`
     - `GE_Fork_serenity_Skill2_Damage2` $\rightarrow$ `Resonance Finisher`
     - `GE_Fork_yaodao_Skill_Damage` $\rightarrow$ `Counter Slash`
     - `GE_Fork_BlackBook_damage` $\rightarrow$ `Chaos Thorn`
2. **Tier 2 (Algorithmic Humanization Fallback)**:
   Any uncurated `effect_id` is passed through a deterministic parser `humanizeEffect(effectId)`:
   - Strips leading prefixes (`GE_Player_[A-Za-z0-9]+_`, `GE_[A-Za-z0-9]+_`, `GE_`)
   - Strips trailing `_Damage`, `_Damage_New`, `Damage`
   - Replaces common tokens with readable terms (`Melee` $\rightarrow$ `Hit`, `Branch` $\rightarrow$ `Branch`, `AirAttack` $\rightarrow$ `Plunge`, `Dot` $\rightarrow$ `DoT`, `UltraSkill` $\rightarrow$ `Ultimate`, `PerfectEvadeAttack` $\rightarrow$ `Dodge Counter`)
   - Formats sub-indices cleanly (e.g. `Melee4_1` $\rightarrow$ `Hit 4-1`, `Branch2_1` $\rightarrow$ `Branch 2-1`, `MagicMelee3` $\rightarrow$ `Magic Hit 3`).

*Rationale*: Writing 592 static strings manually is error-prone and brittle across kit updates. A hybrid approach provides high-polish terminology for key mechanics while guaranteeing 100% deduplicated, readable labels for every packet.

### Decision 2: Secondary Ability Resolution in Localization
Extend `Localization.skill` (or `source-label.ts` ability lookup) to handle common secondary actions:
- `*_AirAttack` $\rightarrow$ `Plunge`
- `*_PerfectEvade` / `*_ExtremEvadeAtk` $\rightarrow$ `Dodge Counter`
- `*_SkillA` / `*_SkillB` $\rightarrow$ `Skill (Variant A / B)`
- `ntesim.break` $\rightarrow$ `Break`

*Rationale*: Prevents ugly `GA_...` raw strings from leaking to the UI when secondary actions deal damage.

### Decision 3: Formatting Composition
When assembling the final label in `sourceLabel(source)`:
- If `source.ability_id === 'ntesim.reaction'`: return the curated reaction label.
- If `source.ability_id === 'ntesim.break'`: return `"Break"`.
- Otherwise:
  - Obtain `abilityTitle` from localization.
  - Obtain `effectSublabel` from curated map or fallback humanizer.
  - If `effectSublabel` already contains or matches `abilityTitle`, avoid redundancy.
  - Return `${abilityTitle} · ${effectSublabel}`.

## Risks / Trade-offs

- **[Risk] Test assertions on exact label strings**: Existing tests in `web/tests/report_controls.test.mjs` assert specific outputs for `sourceLabel`.
  - *Mitigation*: Existing test cases (`Morning Tomato · Nightmare (DoT)`, `Tomato Banquet`, lamb labels, etc.) are strictly preserved and new test cases are added for Zankou, weapon procs, and fallback formatting.
- **[Risk] Label width in charts**: Longer composite labels could wrap or truncate in tight chart margins.
  - *Mitigation*: The Studio BarChart already has label ellipsis and hover/click inspection tooltips showing the full owner, ability, and effect details.
