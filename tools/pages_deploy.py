"""Freeze an eligible fetched main/tag source for the Pages build; never publish."""
import os
from pathlib import Path
import sys

from studio_build import ROOT, git, require_main_history, tag_commit


def selection(root, event, event_ref, requested_tag=""):
    if event == "workflow_dispatch":
        if event_ref != "refs/heads/main":
            raise ValueError("Pages manual dispatch must run from main")
        tag = requested_tag if requested_tag.strip() else ""
    elif event == "push" and event_ref.startswith("refs/tags/"):
        tag = event_ref.removeprefix("refs/tags/")
        if not tag:
            raise ValueError("Pages tag event requires an exact tag")
    else:
        raise ValueError("Pages accepts only tag pushes or manual dispatch from main")
    main = git(root, "rev-parse", "--verify", "refs/remotes/origin/main^{commit}")
    commit = tag_commit(root, tag) if tag else main
    require_main_history(root, commit, main)
    return {"NTESIM_DEPLOY_COMMIT": commit, "NTESIM_DEPLOY_TAG": tag}


def prepare(root, env):
    git(root, "fetch", "--force", "--prune", "origin",
        "+refs/heads/main:refs/remotes/origin/main", "+refs/tags/*:refs/tags/*")
    selected = selection(root, env["GITHUB_EVENT_NAME"], env["GITHUB_REF"], env.get("INPUT_TAG", ""))
    git(root, "checkout", "--detach", selected["NTESIM_DEPLOY_COMMIT"])
    if git(root, "rev-parse", "HEAD") != selected["NTESIM_DEPLOY_COMMIT"]:
        raise ValueError("Pages checkout differs from resolved commit")
    return selected


def main():
    try:
        selected = prepare(ROOT, os.environ)
        with Path(os.environ["GITHUB_ENV"]).open("a", encoding="utf-8", newline="\n") as output:
            for name, value in selected.items():
                output.write(f"{name}={value}\n")
    except (ValueError, OSError, KeyError) as error:
        print(f"Pages source selection failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
