from contextlib import contextmanager
import json
from pathlib import Path
import shutil
import uuid

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / 'cache'


@contextmanager
def temporary_directory(*, dir=None, prefix='verification-'):
    """Yield a unique path under cache; remove only that checked directory.

    Python 3.13's Windows tempfile mode 0700 strips sandbox group inheritance.
    Ordinary mkdir preserves the parent ACL without changing existing rights.
    """
    cache = CACHE.resolve()
    parent = (Path(dir) if dir is not None else CACHE).resolve()
    if not cache.is_relative_to(ROOT) or not parent.is_relative_to(cache):
        raise ValueError('verification temporary directory must stay under workspace cache')
    if not prefix or '/' in prefix or '\\' in prefix or ':' in prefix:
        raise ValueError('verification temporary prefix must be a filename component')
    parent.mkdir(parents=True, exist_ok=True)
    path = parent / (prefix + uuid.uuid4().hex)
    path.mkdir()  # Deliberately normal inherited permissions, never mode 0700.
    try:
        yield str(path)
    finally:
        resolved = path.resolve()
        if (path.is_symlink() or getattr(path, 'is_junction', lambda: False)()
                or resolved.parent != parent or not resolved.is_relative_to(cache)):
            raise ValueError('refusing cleanup outside the allocated verification workspace')
        shutil.rmtree(path)


def output_directory(path):
    target = Path(path)
    target = (target if target.is_absolute() else ROOT / target).resolve()
    if not target.is_relative_to(ROOT.resolve()):
        raise ValueError('verification output directory must stay inside the workspace')
    target.mkdir(parents=True, exist_ok=True)
    return target


def review_receipt_path(path):
    """Resolve a legacy receipt name to cache, rejecting redirected ancestors."""
    root = ROOT.resolve()
    supplied = Path(path)
    if '..' in supplied.parts:
        raise ValueError('review receipt traversal is forbidden')
    target = supplied if supplied.is_absolute() else root / supplied
    try:
        relative = target.relative_to(root)
    except ValueError as error:
        raise ValueError('review receipt must stay inside the workspace') from error
    if relative.parts[:1] == ('reviews',):
        suffix = Path(*relative.parts[1:])
    elif relative.parts[:2] == ('cache', 'reviews') or relative.parts[:2] == ('.cache', 'reviews'):
        suffix = Path(*relative.parts[2:])
    else:
        raise ValueError('review receipt must name reviews/ or cache/reviews/')
    if not suffix.parts:
        raise ValueError('review receipt needs a relative destination')
    destination = root / 'cache/reviews' / suffix
    for candidate in (target, destination):
        for ancestor in (candidate, *candidate.parents):
            if ancestor == root:
                break
            if ancestor.is_symlink() or getattr(ancestor, 'is_junction', lambda: False)():
                raise ValueError('review receipt symlinks/junctions are forbidden')
        if not candidate.resolve().is_relative_to(root):
            raise ValueError('review receipt escapes the workspace')
    if not destination.resolve().is_relative_to(root / 'cache/reviews'):
        raise ValueError('review receipt escapes the cache reviews directory')
    return destination


def write_review_receipt(path, value, *, refresh=False):
    """Write only an explicitly refreshed cache receipt; preserve legacy bytes."""
    target = review_receipt_path(path)
    if not refresh:
        print('No review receipt written (pass --refresh-receipt to write a cache copy).')
        return False
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8', newline='\n')
    print(f'Review receipt refreshed: {target}')
    return True
