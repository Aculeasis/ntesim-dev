"""Lexical ownership ratchet, not Rust name resolution or a crate split proof.

Recognizes ordinary paths, use trees, aliases and explicit re-exports. Macros,
glob-introduced names, dynamic dispatch and complex cfg expressions still need
review. Registry/callbacks/gear are allowed composition interfaces, not owners
hidden behind a scanner workaround; review changes to those interfaces.
"""
from collections import Counter
import re

OWNERS = {'characters', 'weapons', 'sets'}
DISPATCH = ('crate::registry', 'crate::callbacks', 'crate::gear')


def reference_growth(actual, baseline):
    return {name: {symbol: count for symbol, count in references.items()
                   if count > baseline.get(name, {}).get('references', {}).get(symbol, 0)}
            for name, references in actual.items()
            if any(count > baseline.get(name, {}).get('references', {}).get(symbol, 0)
                   for symbol, count in references.items())}


def rust_tokens(source):
    tokens = []
    i = 0
    while i < len(source):
        if source.startswith('//', i):
            end = source.find('\n', i)
            i = len(source) if end < 0 else end + 1
        elif source.startswith('/*', i):
            depth = 1
            i += 2
            while i < len(source) and depth:
                if source.startswith('/*', i):
                    depth += 1
                    i += 2
                elif source.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
        elif match := re.match(r'(?:br|cr|r)(#*)"', source[i:]):
            end = source.find('"' + match[1], i + len(match[0]))
            i = len(source) if end < 0 else end + 1 + len(match[1])
        elif source[i] == '"':
            i += 1
            while i < len(source):
                if source[i] == '\\':
                    i += 2
                elif source[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
        elif match := re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", source[i:]):
            i += len(match[0])
        elif match := re.match(r'[A-Za-z_][A-Za-z_0-9]*|::|[^\s]', source[i:]):
            tokens.append(match[0])
            i += len(match[0])
        else:
            i += 1
    return tokens


def closing(tokens, start):
    pairs = {'[': ']', '(': ')', '{': '}'}
    end = pairs[tokens[start]]
    i = start + 1
    while i < len(tokens):
        if tokens[i] in pairs:
            i = closing(tokens, i) + 1
        elif tokens[i] == end:
            return i
        else:
            i += 1
    raise ValueError('unbalanced Rust delimiter in ownership guard')


def production_tokens(source):
    tokens = rust_tokens(source)
    result = []
    i = 0
    while i < len(tokens):
        if tokens[i:i+2] == ['#', '[']:
            end = closing(tokens, i + 1)
            attr = tokens[i+2:end]
            if attr in (['cfg', '(', 'test', ')'], ['test']):
                i = end + 1
                while tokens[i:i+2] == ['#', '[']:
                    i = closing(tokens, i + 1) + 1
                while i < len(tokens) and tokens[i] not in ('{', ';'):
                    i += 1
                i = closing(tokens, i) + 1 if i < len(tokens) and tokens[i] == '{' else i + 1
                continue
        result.append(tokens[i])
        i += 1
    return result


def use_leaves(tokens, prefix=()):
    """Expand a use tree into (path, local name), including grouped aliases."""
    i = 0
    path = list(prefix)
    while i < len(tokens):
        token = tokens[i]
        if token == '{':
            end = closing(tokens, i)
            group = tokens[i+1:end]
            start = 0
            j = 0
            while j <= len(group):
                if j == len(group) or group[j] == ',':
                    if j > start:
                        yield from use_leaves(group[start:j], tuple(path))
                    start = j + 1
                elif group[j] == '{':
                    j = closing(group, j)
                j += 1
            return
        if token == 'as':
            yield tuple(path), tokens[i+1]
            return
        if token != '::':
            if token != 'self' or not path:
                path.append(token)
        i += 1
    if path:
        yield tuple(path), path[-1]


def resolve(path, aliases, module):
    path = tuple(path)
    seen = set()
    while path and path[0] in aliases and path[0] not in seen:
        seen.add(path[0])
        path = aliases[path[0]] + path[1:]
    if path[:1] == ('self',):
        path = module + path[1:]
    elif path[:1] == ('super',):
        parent = module
        while path[:1] == ('super',):
            parent = parent[:-1]
            path = path[1:]
        path = parent + path
    return path


def imports(tokens, module, inherited=None):
    aliases = dict(inherited or {})
    uses = []
    ranges = set()
    for i, token in enumerate(tokens):
        if token != 'use':
            continue
        end = tokens.index(';', i)
        ranges.update(range(i, end + 1))
        for path, alias in use_leaves(tokens[i+1:end]):
            path = resolve(path, aliases, module)
            if path and path[0] not in ('crate', 'std', 'core', 'alloc', 'serde', 'serde_json') and not path[0].startswith('ntesim_'):
                path = module + path
            uses.append((i, path, alias))
            if alias != '*':
                aliases[alias] = path
    return aliases, uses, ranges


def module_path(path, root):
    parts = list(path.relative_to(root).with_suffix('').parts)
    if parts[-1] in ('mod', 'lib'):
        parts.pop()
    return ('crate', *parts)


def reexports(root):
    exports = {}
    for path in root.rglob('*.rs'):
        if path.name == 'tests.rs' or 'tests' in path.parts:
            continue
        source = path.read_text(encoding='utf-8')
        if not re.search(r'pub(?:\s*\([^)]*\))?\s+use\b', source):
            continue
        tokens = production_tokens(source)
        module = module_path(path, root)
        _, uses, _ = imports(tokens, module)
        for i, target, alias in uses:
            # Includes restricted visibility; visibility itself is compiler-owned.
            start = max((j for j in range(i) if tokens[j] in (';', '{', '}')), default=-1)
            if 'pub' in tokens[start+1:i] and alias != '*':
                exports[module + (alias,)] = target
    return exports


def canonical(path, exports):
    seen = set()
    while path not in seen:
        seen.add(path)
        replacement = next(((key, value) for key, value in exports.items()
                            if path[:len(key)] == key), None)
        if replacement is None:
            break
        key, value = replacement
        path = value + path[len(key):]
    return path


def owner_references(source, *, module=('crate', 'simulation'), exports=None):
    tokens = production_tokens(source)
    aliases, uses, ranges = imports(tokens, module)
    paths = [path for _, path, _ in uses]
    i = 0
    while i < len(tokens):
        if i in ranges or not re.fullmatch(r'[A-Za-z_]\w*', tokens[i]):
            i += 1
            continue
        path = [tokens[i]]
        end = i + 1
        while end + 1 < len(tokens) and tokens[end] == '::' and re.fullmatch(r'[A-Za-z_]\w*', tokens[end+1]):
            path.append(tokens[end+1])
            end += 2
        if len(path) > 1 or path[0] in aliases:
            paths.append(resolve(path, aliases, module))
        i = end
    counts = Counter()
    for path in paths:
        path = canonical(path, exports or {})
        if len(path) > 1 and path[0] == 'crate' and path[1] in OWNERS:
            counts['::'.join(path)] += 1
    return dict(sorted(counts.items()))
