export interface SearchItem {
  name: string;
  id?: string;
  category?: string;
  description: string;
  code: string;
}
import { CATALOG } from '../../domain/catalog';

export const CODE_SNIPPETS: Record<string, [string, string][]> = {
  actions: [
    ['attack', 'mint attack:2;'],
    ['skill / burst', 'mint skill, burst;'],
    ['swap', 'sakiri swap;'],
    ['special / aim', 'nanally special, attack:4;'],
    ['dash / critical dodge', 'sakiri dash[critical=true], skill;'],
    ['Silent Garden Arc', 'baicang arc;'],
    ['Lacrimosa forms', 'lacrimosa special, attack:5, special, attack:5;'],
    ['wait', 'wait(0.5);'],
    ['repeat', 'repeat 2 {\n  mint attack:5;\n  wait(0.5);\n}'],
    ['condition', 'if .time < 10 {\n  mint attack;\n}'],
    ['conditional loop', 'while .time < 15 {\n  mint attack;\n  wait(0.5);\n}'],
  ],
  stats: [
    ['Equipment stats', 'mint add stats hp=5600 atk=1168.15 cr=0.52 cd=0.44;'],
    ['Additive stats', 'mint add stats atk=100 atk%=0.1 cr=0.0331*2;'],
    ['Mental bonus', 'fadia add stats mental%=0.1;'],
    ['Shield bonus', 'adler add stats shield%=0.1;'],
    ['Cycle and Break Intensity', 'zero add stats ci=168; daffodil add stats bi=168;'],
    ['Element bonus', 'mint add stats anima%=0.1; # adds matching Anima bonus, not other elements'],
    [
      'Starting energy',
      'mint char lvl=80/80 awakening=0 talent=8,8,8 qte=8 +params=[start_energy=50];',
    ],
    ['Simulation options', 'options iteration=1000 duration=20 seed=42;'],
    ['Fixed crit policy', 'options iteration=1000 duration=20 seed=42 crit=fixed;'],
    ['Party starting Fons', 'options iteration=1000 duration=20 start_fons=250000;'],
  ],
  targets: [
    ['Explicit NTE target', 'target lvl=80 hp=10000000 def=1080 resist=0.2;'],
    ['Boss assumption', 'target lvl=80 hp=10000000 def=1080 resist=0.2 boss=true;'],
  ],
};

export function buildSearchItems(category: string, query = ''): SearchItem[] {
  const q = query.toLowerCase();
  let items: SearchItem[] = [];

  if (category === 'characters') {
    items = CATALOG.characters.map((c) => ({
      name: c.name,
      description: `${c.element} · ${c.slug}`,
      id: c.slug,
      category,
      code: `${c.slug} char lvl=80/80 awakening=0 talent=8,8,8 qte=8;`,
    }));
  } else if (category === 'weapons' || category === 'cartridges') {
    const source = category === 'weapons' ? CATALOG.weapons : CATALOG.sets;
    items = Object.entries(source).map(([id, name]) => ({
      name,
      id,
      category,
      description: id,
      code: `mint add ${category === 'weapons' ? 'weapon' : 'cartridge'}=${name.toLowerCase().replace(/[^a-z0-9]/g, '')} ${category === 'weapons' ? 'refine=5' : 'count=4'};`,
    }));
  } else {
    items = (CODE_SNIPPETS[category] || []).map(([name, code]) => ({
      name,
      code,
      description: code,
    }));
  }

  return items.filter((x) =>
    (x.name + ' ' + x.description + ' ' + (x.id || '')).toLowerCase().includes(q),
  );
}
