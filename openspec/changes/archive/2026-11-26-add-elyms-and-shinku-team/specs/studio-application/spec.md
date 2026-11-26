## MODIFIED Requirements

### Requirement: Curated complete team collection

Team list SHALL offer twenty-two distinct curated four-character compositions with equipped builds and executable continuing rotations, alongside the S5 control. It SHALL replace the previous selectable partial research entries. Different equipment alone SHALL NOT count as a distinct composition toward the collection target.

#### Scenario: Replace partial selectable presets
- **WHEN** the curated collection is delivered
- **THEN** Team list contains the `22` complete curated teams and S5, and no selectable S1, S2, S3, S4 or old Baicang domain example
- **AND** non-selectable language and regression fixtures can remain outside the collection
- **AND** every curated entry loads its exact complete script without starting a simulation

#### Scenario: Preserve the control
- **WHEN** the collection and its ranking are regenerated
- **THEN** S5 retains its identity, label, historical build profile, target conditions and default-preset role, with explicit `qte=8` and any timing changes required by the corrected Support Skill rules
- **AND** current-engine recomputation can update its displayed DPS without silently rewriting its source

### Requirement: Mandatory and excluded team content

The collection SHALL contain Nanally / Hotori / Zero / Jiuyuan, Zankou / Iroi / Sakiri / Daffodill, and Shinku / Akane Rin / Elyms / Iroi. Daffodill in the Zankou composition SHALL have exactly A3, A4 and A5 selected; Sakiri there SHALL have exactly A4 selected; and released Elyms, Akane Rin, and their signature weapons are permitted.

#### Scenario: Inspect mandatory compositions
- **WHEN** the mandatory entries are selected and inspected
- **THEN** the Nanally and Zankou entries match their required rosters, with Daffodill at `[3,4,5]` (without A1, A2, or A6) and Sakiri at `[4]`
- **AND** `shinku-elyms` declares Lv80/80 A0 (`awakening=[]`, `talent=8,8,8`, `qte=8`, `refine=1`, 15 useful lines) builds for Shinku (`lostradiance` + `fork_lunarphase`), Akane Rin (`streetboxer` + `fork_prism`), Elyms (`lostradiance` + `fork_elyms`), and Iroi (`speedyhedgehog` + `fork_door`, single `Suit11` carrier), completing 1,000 trials over 120 relative seconds

#### Scenario: Exclude unavailable content
- **WHEN** curated builds and Team list validation checks are inspected
- **THEN** none contains unreleased characters or unresolved placeholders, while released Elyms (`character:1090`), Akane Rin (`character:1057`), `weapon:fork_elyms`, and `weapon:fork_prism` are permitted in curated entries
- **AND** regenerated `web/src/generated/team-list.json` renders `ready` DPS status across all `22` curated teams plus S5
