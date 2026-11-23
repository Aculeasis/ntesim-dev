import { getPresetSource } from '../../domain/presets';

export function LanguageReference() {
  return (
    <>
      <p>
        Declare options, builds, targets and the initial active character, then write the rotation.
      </p>
      <pre>{getPresetSource('language-reference')}</pre>
      <p>
        Actions: attack, skill, burst, swap, special (G), dash. Commas execute actions in order; :N
        repeats an input. Off-field actions swap automatically. Sequential attack inputs use the
        character model.
      </p>
      <p>
        special enters aiming for Adler, Nanally and Jiuyuan; attack then fires a shot. Dash, skill
        and burst exit aiming. Lacrimosa special deals damage and switches her persistent
        ranged/melee form; attack and burst follow that form. Other special models fail explicitly.
      </p>
      <p>
        Time uses seconds; s, ms and us suffixes are accepted. wait adds an interval; wait_until
        waits until an absolute time. Variables use let or const; conditions use if/else; loops use
        repeat, while or for let. Numeric built-ins: min, max, clamp, abs, floor, ceil.
      </p>
      <p>
        add stats rows sum equipment contributions. Ratios use fractions: cr=0.3 means 30%. Full HP
        and energy are defaults; +params=[start_hp=1000,start_energy=20] changes starting resources.
        Use start_hp%=0.5 for half of final projected Max HP; the exact decimal ratio is multiplied
        once and floored, and a result below 1 HP is rejected. start_hp and start_hp% cannot be
        combined. awakening=3 equals awakening=[1,2,3].
      </p>
      <p>
        ce adds to the native Charge Efficiency multiplier; ce=0.12 adds 12 percentage points. It
        scales character and equipment energy grants that support efficiency.
      </p>
      <p>
        Declare anonymous dummy hp, def and common resist directly in the script. Optional anima,
        chaos, cosmos, incantation, lakshana and psyche fields override that element’s resistance;
        omitted elements use common resist. Boss eligibility and Break gauges are explicit scenario
        settings.
      </p>
      <p>Ctrl+Enter runs; Ctrl+K searches; Tab indents.</p>
    </>
  );
}
