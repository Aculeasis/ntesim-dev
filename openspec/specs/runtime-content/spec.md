# Runtime content and authored builds

## Purpose

Defines consumed runtime content, explicit dummy inputs and authored equipment and console contributions.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Consumed typed constants

Runtime constants SHALL retain only fields consumed by simulation with concrete numeric or causal checks; raw source acquisitions SHALL remain outside tracked runtime catalogs.

#### Scenario: Unused acquisition field
- **GIVEN** a source export contains an unused field alongside a required coefficient
- **WHEN** content is bound for runtime use
- **THEN** only the consumed coefficient is retained in typed runtime content

### Requirement: Authored console stats

Console stats SHALL enter through authored add stats declarations, while declared two-piece and four-piece set effects apply automatically without duplicating static bonuses.

#### Scenario: Declared set and additive stats
- **GIVEN** a script declares a qualifying set count and explicit console stats
- **WHEN** its build is compiled
- **THEN** the set effect and authored additions are each included once without automatic console specialization or geometry

### Requirement: Lacrimosa connected-packet Cycle accrual

Lacrimosa's reached connected packets SHALL accrue their exact frozen native HeterochromeAdd through the existing positive-HP-loss, living-owner and causal-group policy. Fractional carry SHALL persist across casts, and AoE recipients SHALL NOT multiply one packet's Cycle grant.

#### Scenario: Retain fractional Cycle independently of recipient count
- **WHEN** two Melee1 packets with frozen Cycle 2.376 connect against one or multiple eligible targets
- **THEN** the owner has four whole Cycle and 0.752 fractional carry
- **AND** the existing capacity, efficiency, reaction spending and zero-grant packet rules remain effective

### Requirement: Authored flat intensity contributions

Authored `ci` and `bi` additions SHALL contribute exact flat Cycle Intensity and Break Intensity points respectively. Repeated contributions SHALL sum without replacing innate or equipment bonuses, and shared build projections SHALL preserve these values through execution and inspection.

#### Scenario: Preserve independent intensity zones
- **WHEN** a script adds fractional Cycle and Break Intensity in multiple declarations
- **THEN** each intensity equals its baseline plus its exact authored sum
- **AND** neither addition changes ATK or general damage bonus or silently becomes zero
- **AND** native and WASM execution use the same projected intensity values

### Requirement: Zero Skill Cycle completion

An admitted Zero Skill SHALL fill the owner's Cycle pool to its existing 100-point capacity once, including semantic replay. It SHALL retain previous charge without overgranting and preserve continuing Cycle accounting through subsequent switches and casts.

#### Scenario: Fund continuing Blossom switches
- **GIVEN** Zero starts with zero Cycle and has authored Cycle Intensity
- **WHEN** two legally separated Skills each precede a switch to an Anima teammate
- **THEN** each switch spends 100 actually granted Cycle and creates a Blossom with the appropriate captured owner and Intensity
- **AND** a full pool cannot receive a duplicate fill, and failed Skill admission grants no additional Cycle

### Requirement: Sakiri A4 source-based ATK grant

Selected Sakiri A4 SHALL grant eligible living teammates, excluding Sakiri, flat ATK equal to thirty percent of her source base ATK for twenty unpaused world seconds after the sourced Burst activation event. Source base ATK SHALL include equipped weapon base ATK and exclude console flat ATK, ATK-percent buffs and recipient stats. Only actual node four SHALL enable this contribution.

#### Scenario: Independent flat-ATK arithmetic
- **GIVEN** Sakiri has source base ATK 1000 and exactly node four selected
- **WHEN** an admitted Burst reaches the buff activation event
- **THEN** each eligible teammate receives exactly 300 additional ATK from A4 and Sakiri receives none
- **AND** changing Sakiri's console ATK or a recipient's base ATK does not change that A4 amount
- **AND** a larger equipped weapon base ATK changes the grant by exactly thirty percent of that base difference

#### Scenario: Refresh and actual expiry
- **GIVEN** a teammate carries Sakiri's active A4 grant
- **WHEN** another legally funded Burst refreshes it or its exact deadline is reached
- **THEN** the same source refreshes its twenty-second lifetime without stacking duplicate grants, or removes the expired contribution respectively
- **AND** field exit does not invent a second application or extend the timer
- **AND** source/recipient defeat follows the declared living-carrier lifetime

#### Scenario: Node identity and rejected casts
- **WHEN** Burst admission fails or Sakiri has only unrelated awakening nodes selected
- **THEN** no A4 grant is created or refreshed
- **AND** selecting four nodes without node four cannot substitute for that node

### Requirement: Sakiri ordinary and awakened Burst contributions

Sakiri's ordinary Burst SHALL grant other eligible living teammates thirty percent of her source base ATK for twenty unpaused world seconds. Selected A4 SHALL add a separate thirty-percent contribution with the same duration and source basis. Each carrier SHALL refresh without stacking itself. Damage and live buff observations SHALL use the actual effective grant, recipients and actual world-clock lifetime rather than a synthetic expected team-DPS percentage.

#### Scenario: Compare an identical Burst with and without A4
- **GIVEN** otherwise identical builds, resources, recipients and legal actions with A4 absent or selected
- **WHEN** Burst and ally damage settle inside and outside the buff window
- **THEN** the ordinary Burst contribution remains correct in both cases and the isolated A4 difference equals its sourced flat-ATK contribution
- **AND** non-ATK-scaling damage does not gain a fictional thirty-percent multiplier
- **AND** an observed small total-DPS difference is explained using actual buff uptime and affected damage rather than relabeled as a failed buff

#### Scenario: Sakiri grant survives stopped world time
- **GIVEN** the ordinary and selected A4 grants each have two world seconds remaining
- **WHEN** an Ultimate cinematic or Hotori Time Stop lasts ten absolute seconds without source removal
- **THEN** both grants retain exactly two seconds and their separate source-based ATK contributions throughout the pause
- **AND** neither grant stacks, refreshes or expires solely from that absolute elapsed time

### Requirement: Public character effect projection

Authored scenarios SHALL select supported owner effects with their canonical model provenance and native progression gates. Entry effects SHALL retain actual outgoing state, switch eligibility and qualified Support rules.

#### Scenario: Owner passives without an authored owner cast
- **GIVEN** a public scenario includes Adler with Righteous Heart unlocked or Aurelia with Harmonics unlocked
- **WHEN** Adler's starting DEF is evaluated or an eligible Nova expires with Aurelia in the party
- **THEN** the corresponding owner passive is available without a manual model declaration or prior owner cast

#### Scenario: Lacrimosa Evade into Baicang entry
- **GIVEN** Lacrimosa is in ordinary Evade recovery and the incoming Baicang qualifies for his ETD-4 entry effect
- **WHEN** `dash[window=swap]` yields at the existing Evade exit gate and the script admits the switch to Baicang before Evade recovery ends
- **THEN** the qualified entry uses the existing Evade exit and JumpZ models with their ordinary costs and deadlines

### Requirement: Character effect activation and thresholds

Reached character effects SHALL use their own cast markers, progression gates and exact target predicates across supported damage and action paths.

#### Scenario: Ultimate and Skill effect producers
- **WHEN** an authored Adler Burst reaches its second volley marker, Aurelia casts Burst, or Blackbird casts the evolved Finisher
- **THEN** Adler receives each selected Burst resource callback once at the marker
- **AND** Aurelia's Burst leaves Cadenza unchanged and Blackbird's Finisher leaves the transformation Burst's team ATK grant unchanged

#### Scenario: Target damage and resistance conditions
- **WHEN** Chaos hits a target marked by Warrant at Breakthrough 4, Jiuyuan binds a target at Resonance 6, or Mint with node 6 hits a target at exactly 40 percent HP
- **THEN** Chaos receives the 30 percent Warrant damage bonus, Pact reduces the bound target's Anima resistance by 10 percentage points, and Mint receives her low-HP damage bonus respectively
- **AND** selecting Jiuyuan's node 6 alone does not grant the resonance reduction

### Requirement: Adler Temperance on Scorch

Each successful Scorch application or refresh SHALL draw one Temperance outcome for its target when a living Breakthrough 2 Adler is in the party. Each outcome SHALL refresh one independent 15-world-second target status. The timed status SHALL survive source field exit or defeat and end on expiry or target defeat.

#### Scenario: Exact target changes from one seeded outcome
- **WHEN** a qualifying Scorch application draws ATK, resistance or Break acceptance
- **THEN** it reduces the target's authored ATK by 20% of original ATK, reduces Physical, Esper and Mental resistance by 10 percentage points, or adds 10 percentage points to incoming Break efficiency respectively
- **AND** subsequent ticks do not resample the outcome

#### Scenario: Refresh and replay
- **GIVEN** an existing Temperance status and another successful Scorch application on that target
- **WHEN** the same outcome is drawn again or a different outcome is drawn
- **THEN** the same outcome refreshes without stacking, or the distinct outcomes coexist with independent deadlines respectively
- **AND** repeating the scenario with the same seed reproduces the draws and outcomes

### Requirement: Zero Resonance 6 Cosmos Cycle ATK grant

When living Zero with six selected awakening nodes pays for a qualified Cosmos Esper Cycle, living party members SHALL receive a refreshable +12% base-ATK-scaled bonus for 20 world seconds.

#### Scenario: Trigger ownership and effect lifetime
- **WHEN** Zero funds a qualified Cosmos switch, including one whose damage is attributed to the higher-CI incoming actor
- **THEN** each living party member gains that carrier once and a later qualifying activation refreshes it without stacking
- **AND** an unpaid switch, a switch paid by another character, or fewer than six selected nodes does not grant the carrier

### Requirement: Far Side of the Moon Gravitide

Far Side of the Moon SHALL add its refinement Crit Rate to the wearer's base Crit Rate. Delivered Basic Attack packets with positive HP loss SHALL grant one Redirect Skill stack for five world seconds, up to four stacks with a 300ms acquisition gate. Qualified hits SHALL refresh the shared deadline. Admitted wearer Redirect Skills SHALL refresh a 15-world-second Basic Attack window before hit delivery. During overlap of both effects, the wearer SHALL gain the refinement Lakshana bonus.

#### Scenario: Accumulate bounded stacks from damage
- **GIVEN** a wearer with no active stacks
- **WHEN** Basic Attack packets damage one or several targets in the same instant and then land again at eligible times
- **THEN** the first instant grants one stack and each eligible later instant increases it up to four, and a qualified hit at cap renews the shared lifetime
- **AND** non-Basic packets, zero HP loss and another actor's hits do not grant stacks

#### Scenario: Combine independent effects on delivery
- **WHEN** a Redirect Skill is admitted while stacks are active and subsequent damage lands inside and outside either effect
- **THEN** Redirect Skill packets gain the current per-stack bonus and Basic Attack packets gain the active cast-window bonus
- **AND** Lakshana packets gain the element bonus only during overlap, additively with any matching category bonus
- **AND** expiry precedes equal-time delivery, switching preserves both effects, and rejected casts do not open or refresh the cast window

#### Scenario: Use the same refinement and timing in CLI and Studio
- **WHEN** a script equips the Arc on a compatible wearer and executes continuing actions in native or WASM
- **THEN** both paths report the same refinement values, stack and window transitions, and settled damage
- **AND** world-time pauses extend live effect and acquisition deadlines without granting extra stacks

### Requirement: Echoes on Hold equipment

The shared catalog SHALL expose Echoes on Hold as a Solid Arc with level 1–80 progression, six breakthroughs and five refinements. Compatible wearers SHALL receive its base ATK, CRIT Rate and refinement-dependent ATK bonus in native and Studio runs.

#### Scenario: Equip and inspect the Arc
- **WHEN** a compatible wearer equips Echoes on Hold at level 80 and breakthrough 6
- **THEN** the build contributes 570 base ATK and 24 percentage points of CRIT Rate and reports the selected passive refinement
- **AND** an incompatible wearer is rejected

### Requirement: Voicemail Redial

Echoes on Hold SHALL grant independent 20-world-second team DEF Ignore and Cosmos Resistance Penetration windows at combat start. The DEF Ignore window SHALL also grant additional DEF Ignore to its activating wearer. A successful wearer Redirect Skill SHALL refresh Cosmos penetration. Three qualifying wearer Follow-up Attack packets after an admitted Ultimate SHALL refresh DEF Ignore once per Ultimate.

#### Scenario: Count continuing Follow-up Attack damage
- **WHEN** the wearer delivers tagged Follow-up Attack packets that cause positive HP loss after an admitted Ultimate, including while off-field
- **THEN** each distinct packet counts once across its recipients and the third packet refreshes DEF Ignore after damage settlement
- **AND** an admitted replacement Ultimate starts a fresh count, and untagged damage, another owner's damage and rejected casts cannot advance it

#### Scenario: Refresh team effects independently
- **WHEN** either window is triggered again from the same or another copy
- **THEN** its latest application replaces the previous one without stacking and leaves the other window's deadline unchanged
- **AND** Cosmos penetration applies only to Cosmos damage and each window expires before damage at its world-time deadline

### Requirement: Released Character Registry Includes Akane Rin
`ntesim-data`, `ntesim-content`, `ntesim-research`, and Studio contracts SHALL include Akane Rin (`character:1057`, slug `akane`, script alias `Akane Rin`) as the 24th released character (`10626` four-member combinations), supporting `start_string_energy`, live `weapon:fork_prism` pairing, `'akane'` portrait mapping, `akane_hit` energy attribution, and regenerated catalog/name assets while preserving the 21-team comparison collection and `recommended-gear.csv` unchanged.

#### Scenario: Expanded released roster, research combinations, and DSL catalog
- **WHEN** callers query released characters, four-character team combinations, resonance parity fixtures, or script catalogs
- **THEN** `24` released characters and `10626` team combinations are exposed including `character:1057` (`akane` / `Akane Rin`)
- **AND** the curated 21-team Studio comparison list and `presets/research/recommended-gear.csv` remain unchanged

#### Scenario: Far Side of the Moon character pairing and Studio bundle parity
- **WHEN** `Akane Rin` equips `weapon:fork_prism` (*Far Side of the Moon*) or Studio loads localization and portrait assets
- **THEN** `Synced Resonance` (`RedirectSkill`) and `Perfect Pitch` (`Category E`) interoperate with `weapon:fork_prism` triggers
- **AND** `'akane'` resolves to `characters_akane-rin.webp` and `akane_hit` resolves in energy attribution reports alongside regenerated Studio catalog and name bundles

### Requirement: Shared Remora Reapplication and Pre-Overwrite Duration Hooks
Shared `Remora` (`Cosmos + Lakshana`) reaction handling SHALL provide an owner-sourced periodic `Remora` application path honoring active team `Remora` enhancements, and SHALL sample the target's existing active `Remora` remaining duration (`existing_remaining_us`) BEFORE overwriting or extending its `Remora` timer when notifying reaction listeners alongside existing `Hathor` B2 and `Chaos` B2 hooks.

#### Scenario: Existing Remora remainder delivered before timer refresh
- **WHEN** a character applies or periodically ticks `Remora` onto a target that currently has an active `Remora` status with `existing_remaining_us > 0`
- **THEN** reaction listeners receive `existing_remaining_us` captured strictly before the target's `Remora` expiration is refreshed
- **AND** initial application onto a target without active `Remora` reports no pre-existing remainder

#### Scenario: Owner-sourced periodic Remora ticks preserve Cycle reaction accounting
- **WHEN** Akane Rin's `Overtone` or `Reverb Field` applies a `3.0s` periodic `Remora` tick onto a target
- **THEN** the tick honors active `Hathor` B2 duration enhancement and notifies `Chaos` B2 and `Perfect Pitch` listeners
- **AND** Esper Cycle switch reaction counters continue distinguishing Cycle-triggered reactions from passive periodic applications
