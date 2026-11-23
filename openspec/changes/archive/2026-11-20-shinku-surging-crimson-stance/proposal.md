# Proposal: Shinku Surging Crimson Stance & Rotation Overhaul

## Why

Shinku's implementation in `ntesim` currently only supports a baseline partial model (`shinku.normal_chain_v1`, `shinku.skill_packets_v1`, `shinku.burst_packets_v1`), lacking her core character mechanics: the Surging Crimson stance transformation, Tidal Assault attacks generating Fading Reason, Scarlet Descent empowered skill casts, Crimson Judgment dash stacks, and the Dragonflame Verdict finisher nuke. Furthermore, `openspec/specs/characters/shinku/spec.md` contains an obsolete placeholder requirement describing a generic 100-rage meter, and the curated comparison scenario `presets/studio/shinku-charge.nte` only executes ordinary basic attacks and skills on cooldown without engaging her true burst windows. Aligning Shinku's runtime mechanics, specification, and curated team rotation with verified game data (`DT_GameplayAbilityTipsData`, `DT_SkillDamageData`, `DT_ShinkuEffectFigure`) is required for accurate combat simulation and DPS evaluation.

## What Changes

- **Update Shinku Specification**: Replace the obsolete 100-rage meter requirements in `characters/shinku` with the authentic kit behavior:
  - Base stance accumulates Defiant Spirit (cap 60) through normal attacks and skills to cast Ultimate (Crimson Fury).
  - Crimson Fury enters the 13-second Surging Crimson stance, replacing normal attacks with Tidal Assault and the skill with Scarlet Descent.
  - Tidal Assault attacks generate Fading Reason (cap 12).
  - Scarlet Descent consumes 12 Fading Reason with no standard cooldown, castable up to 5 times per Surging Crimson window, granting 1 stack of Crimson Judgment per cast (up to 3 stacks max).
  - During the final window of Surging Crimson (starting at 9.0s), the Ultimate input becomes Crimson Judgment, enabling rapid dash attacks that consume Crimson Judgment stacks.
  - After consuming Crimson Judgment stacks or upon state completion, the Ultimate input executes the Dragonflame Verdict finisher (10 flurry hits plus the 400% ATK nuke).
- **Register Native Rage Damage Packets**: Add `GE_Player_Shinku_Skill1_Rage_Damage`, `GE_Player_Shinku_Skill2_Rage_Damage`, and `GE_Player_Shinku_UltraSkill2_Rage_Damage` to `crates/data/src/native_damage/shinku.rs` matching frozen native rows.
- **Implement Runtime Stance State Machine**:
  - Extend `Shinku` runtime state with Fading Reason tracking, Scarlet Descent cast counts, and Crimson Judgment stacks.
  - Implement dynamic action resolution in `ShinkuKit::select_source_command` and `on_cast` to route `skill` and `burst` commands to their respective stance variants during Surging Crimson.
- **Refine Curated Scenario Rotation**: Update `presets/studio/shinku-charge.nte` to utilize the full Surging Crimson rotation (Burst -> Tidal Assault / Scarlet Descent loops -> Crimson Judgment dashes -> Dragonflame Verdict).
- **Verification Tests**: Add comprehensive mechanics and causality tests in `crates/content/src/simulation/tests/shinku_actions.rs` verifying the full stance loop.

## Capabilities

### Modified Capabilities
- `characters/shinku`: Replace obsolete rage meter requirements with authentic Surging Crimson stance, Fading Reason resource accounting, Scarlet Descent casts, Crimson Judgment dashes, and Dragonflame Verdict finisher.

## Impact

- `openspec/specs/characters/shinku/spec.md`: Updated specification requirements and scenarios.
- `crates/data/src/native_damage/shinku.rs`: Added native damage packet definitions for Rage skill and finisher.
- `crates/content/src/characters/shinku/`: Updated runtime state, ability dispatch, packet scheduling, and kit hooks.
- `crates/content/src/registry/kits.rs`: Updated `ShinkuKit` hooks.
- `crates/content/src/simulation/tests/shinku_actions.rs`: New unit and integration tests for stance transitions.
- `presets/studio/shinku-charge.nte`: Updated curated script rotation.
