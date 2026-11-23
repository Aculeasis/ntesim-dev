function endOfJsonString(text, start) {
  for (let quote = text.indexOf('"', start); quote !== -1; quote = text.indexOf('"', start)) {
    // An odd run of backslashes escapes the quote.
    let before = quote;
    while (before > start && text[before - 1] === '\\') before--;
    if ((quote - before) % 2 === 0) return quote + 1;
    start = quote + 1;
  }
  throw new SyntaxError('Unterminated JSON string');
}

function parseLosslessJson(text) {
  // Match opening quotes separately so a missing closing quote cannot restart the scan.
  const tokens = /"|(-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?)/g;
  const keyEnd = /[ \t\n\r]*:/y;
  let match, previous = 0, parts;
  while ((match = tokens.exec(text))) {
    const token = match[1];
    if (token === undefined) {
      tokens.lastIndex = endOfJsonString(text, tokens.lastIndex);
      continue;
    }
    if (/[.eE]/.test(token) || Number.isSafeInteger(Number(token))) continue;
    // Leave unquoted object keys invalid for JSON.parse.
    keyEnd.lastIndex = tokens.lastIndex;
    if (keyEnd.test(text)) continue;
    (parts ??= []).push(text.slice(previous, match.index), JSON.stringify(token));
    previous = tokens.lastIndex;
  }
  if (parts) {
    parts.push(text.slice(previous));
    text = parts.join('');
  }
  return JSON.parse(text);
}

const api = { parseLosslessJson };
if (typeof module !== 'undefined' && module.exports) module.exports = api;
if (typeof globalThis !== 'undefined') globalThis.NtesimLosslessJson = api;
