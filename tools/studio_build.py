"""Resolve Studio's immutable build source and UTC build-start instant."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = "https://github.com/Aculeasis/ntesim-dev"
INSTANT = "NTESIM_BUILD_STARTED_AT"


def git(root, *args):
    result = subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True,
                            encoding="utf-8")
    if result.returncode:
        raise ValueError("Studio Git resolution failed: " + result.stderr.strip())
    return result.stdout.strip()


def build_instant(env):
    value = env.get(INSTANT) or datetime.now(timezone.utc).isoformat(timespec="milliseconds")
    instant = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if instant.tzinfo is None:
        raise ValueError("Studio build instant requires a UTC offset")
    return instant.astimezone(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def tag_commit(root, tag):
    ref = "refs/tags/" + tag
    git(root, "check-ref-format", ref)
    # Exact namespace and object existence precede peeling: never accept revisions.
    git(root, "show-ref", "--verify", ref)
    return git(root, "rev-parse", "--verify", ref + "^{commit}")


def require_main_history(root, commit, main):
    try:
        git(root, "merge-base", "--is-ancestor", commit, main)
    except ValueError as error:
        raise ValueError("Selected Studio commit must belong to fetched main history") from error


def resolve(root=ROOT, env=None):
    env = os.environ if env is None else env
    started = build_instant(env)
    commit = git(root, "rev-parse", "--verify", "HEAD^{commit}")
    short = git(root, "rev-parse", "--short=7", "HEAD")
    selected = env.get("NTESIM_DEPLOY_COMMIT")
    tag = env.get("NTESIM_DEPLOY_TAG", "")
    if tag and not selected:
        raise ValueError("Studio deployment tag requires an explicit commit")
    if selected:
        if not re.fullmatch(r"[0-9a-f]{40}", selected) or selected != commit:
            raise ValueError("Studio deployment commit must equal full checked-out HEAD")
        if tag and tag_commit(root, tag) != commit:
            raise ValueError("Studio deployment tag does not match HEAD")
        main = git(root, "rev-parse", "--verify", "refs/remotes/origin/main^{commit}")
        require_main_history(root, commit, main)
        branch = "main"
    else:
        branch = git(root, "rev-parse", "--symbolic-full-name", "HEAD").removeprefix("refs/heads/")
    label = tag or (short if branch in ("main", "HEAD") else branch + "/" + short)
    ref = "refs/tags/" + quote(tag, safe="") if tag else short
    identity = {"commit": commit, "label": label, "url": REPOSITORY + "/tree/" + ref,
                "repository_url": REPOSITORY, "build_started_at": started}
    supplied = env.get("NTESIM_STUDIO_BUILD")
    if supplied and json.loads(supplied) != identity:
        raise ValueError("Studio source or instant changed during build")
    return identity


if __name__ == "__main__":
    print(json.dumps(resolve()))
