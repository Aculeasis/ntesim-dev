export interface HighlightToken {
  text: string;
  type?: string;
}
const words = new Set(
  'options char add stats weapon cartridge target active let const if else while for repeat break continue true false null attack skill burst ultra swap special dash arc'.split(
    ' ',
  ),
);
const stats = new Set(
  'hp atk def cr cd dmg% heal% mental% shield% atk% hp% def% ce ci bi iteration duration seed start_fons max_hp mode crit awakening talent refine lvl count pack'.split(
    ' ',
  ),
);
export function highlightTokens(source: string): HighlightToken[] {
  const pattern =
    /(#[^\n]*|\/\/[^\n]*|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\b\d+(?:\.\d+)?(?:us|ms|s)?\b|[a-zA-Z_][a-zA-Z_0-9]*%?)/g;
  const result: HighlightToken[] = [];
  let cursor = 0;
  for (const m of source.matchAll(pattern)) {
    result.push({ text: source.slice(cursor, m.index) });
    const text = m[0];
    const type =
      text.startsWith('#') || text.startsWith('//')
        ? 'comment'
        : /^['"]/.test(text)
          ? 'string'
          : /^\d/.test(text)
            ? 'number'
            : words.has(text)
              ? 'keyword'
              : stats.has(text)
                ? 'stat'
                : /^\s*\(/.test(source.slice(m.index + text.length))
                  ? 'function'
                  : 'identifier';
    result.push({ text, type });
    cursor = m.index + text.length;
  }
  result.push({ text: source.slice(cursor) + '\n' });
  return result;
}
