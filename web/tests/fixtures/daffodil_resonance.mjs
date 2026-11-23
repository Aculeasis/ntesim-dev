import { runSourceFixture } from '../native-suite.mjs';

// Mechanics and adversarial admission cases belong to the Rust Daffodil suites.
// Transport uses legal continuing source, never synthetic QTE/cooldown commands.
export function daffodilScenarios(engine) {
  const baseline = `options duration=30 crit=random seed=42;
    daffodil char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=120 start_resonance=2;
    daffodil add weapon=fork_appliance refine=1;
    zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
    target hp=100000000 def=1080 resist=0.2 break_gauge=1/1000 max_break_gauge=30 broken_duration=0.1;
    target hp=100000000 def=1080 resist=0.2 break_gauge=1/1000 max_break_gauge=30 broken_duration=0.1;
    active zero; daffodil swap; daffodil skill, attack:2, burst;
    wait_until(20);
    while .daffodil.skill.cd>0 { wait(.daffodil.skill.cd); }
    daffodil skill, attack;`;
  return [0, 1, '[4,5]', '[3,4,5]', 6, 'rose'].map((variant) => {
    let source = baseline.replace(
      'daffodil char lvl=80/80 awakening=0',
      `daffodil char lvl=80/80 awakening=${variant === 'rose' ? 6 : variant}`,
    );
    if (variant === 'rose') source = source.replace('fork_appliance', 'fork_Rose');
    return { source, request: runSourceFixture(engine, source).request };
  });
}
