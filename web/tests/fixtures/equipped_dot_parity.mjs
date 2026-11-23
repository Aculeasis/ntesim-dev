import { readFileSync } from 'node:fs';

export function equippedDotSources(tier) {
  const nestBird = readFileSync(
    new URL('../../../examples/nestbird-gaze-cycles.nte', import.meta.url),
    'utf8',
  ).replace('refine=1', `refine=${tier}`);
  return [
    nestBird,
    ...(tier === 1
      ? ['[4]', '[1,2,3,4,5,6]'].map(
          (nodes) => `options iteration=1 duration=120 crit=random;
      zankou char lvl=80/80 awakening=${nodes} talent=1,1,1 qte=1;
      zankou add stats ce=1;
      daffodil char lvl=80/80 awakening=[] talent=1,1,1 qte=1;
      target hp=1000000000 def=0 resist=0 break_gauge=30 max_break_gauge=30 broken_duration=10;
      target hp=1000000000 def=0 resist=0 break_gauge=30 max_break_gauge=30 broken_duration=10;
      active zankou; daffodil swap; zankou swap;
      repeat 2 {
        zankou attack; zankou attack[hold=200ms]; zankou skill;
        while !.zankou.enhanced_soulcross.active { zankou attack; }
        zankou skill; zankou burst; zankou burst;
        zankou attack:20;
      }`,
        )
      : []),
    `options iteration=1 duration=70 crit=fixed;
      zankou char lvl=80/80 awakening=0 talent=8,8,1 qte=8 start_energy=0;
      zankou add weapon=fork_NestBird refine=${tier};
      lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      target hp=100000000 def=1080 resist=0; target hp=100000000 def=1080 resist=0;
      active zankou; zankou skill; wait(1); lacrimosa swap;
      wait(3); zankou swap; wait(9); zankou skill; lacrimosa swap;`,
    `options iteration=1 duration=140 crit=fixed;
      lacrimosa char lvl=80/80 awakening=0 talent=8,1,1 qte=1 start_energy=120;
      lacrimosa add weapon=fork_Rose refine=${tier};
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      target hp=100000000 def=0 resist=0;
      target hp=100000000 def=0 resist=0;
      active lacrimosa; lacrimosa burst, attack:40; wait(20); lacrimosa burst; mint swap;`,
  ];
}
