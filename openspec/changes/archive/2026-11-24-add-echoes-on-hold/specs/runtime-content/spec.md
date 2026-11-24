## ADDED Requirements

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
