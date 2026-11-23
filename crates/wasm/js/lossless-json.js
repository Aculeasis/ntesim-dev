function parseLosslessJson(text) {
  // Consume complete strings before numeric tokens, so quoted numbers and
  // escapes stay verbatim. Keep slices only where unsafe integers change;
  // safe-only documents go straight to the native parser without a second copy.
  // Scan runs of ordinary string characters together; escapes are disjoint.
  const tokens = /"[^"\\]*(?:\\.[^"\\]*)*"|(-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?)/g;
  let match, previous = 0, parts;
  while ((match = tokens.exec(text))) {
    const token = match[1];
    if (!token || /[.eE]/.test(token) || Number.isSafeInteger(Number(token))) continue;
    (parts ??= []).push(text.slice(previous, match.index), JSON.stringify(token));
    previous = match.index + token.length;
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
