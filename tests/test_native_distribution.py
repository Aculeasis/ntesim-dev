"""Native distribution boundaries: host closure, notices and exact payloads."""
import json
import os
from contextlib import contextmanager
from pathlib import Path
import stat
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import call, patch
import warnings
import zipfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import native_distribution as distribution
import browser_distribution as browser
import studio_build
import pages_deploy
from verification_workspace import temporary_directory


@contextmanager
def git_fixture():
    with temporary_directory(prefix="studio-git-") as temporary:
        root = Path(temporary)
        try:
            studio_build.git(root, "init", "-b", "main")
            studio_build.git(root, "config", "user.name", "Fixture")
            studio_build.git(root, "config", "user.email", "fixture@example.invalid")
            studio_build.git(root, "config", "core.hooksPath", str(root / ".no-hooks"))
            studio_build.git(root, "config", "commit.gpgSign", "false")
            studio_build.git(root, "config", "tag.gpgSign", "false")
            yield root
        finally:
            if os.name == "nt":
                for path in (root / ".git/objects").rglob("*"):
                    if path.is_file():
                        assert path.resolve().is_relative_to(root.resolve())
                        path.chmod(path.stat().st_mode | stat.S_IWRITE)


class BrowserDistributionTests(unittest.TestCase):
    def test_pages_selection_freezes_exact_main_history_tags(self):
        with git_fixture() as root:
            git = lambda *args: studio_build.git(root, *args)
            git("commit", "--allow-empty", "-m", "historical")
            historical = git("rev-parse", "HEAD")
            git("tag", "historical")
            git("tag", "-a", "release/annotated+#%", "-m", "version")
            git("commit", "--allow-empty", "-m", "tip")
            tip = git("rev-parse", "HEAD")
            git("tag", "tip")
            (root / "blob").write_text("not a commit")
            blob = git("hash-object", "-w", "blob")
            git("update-ref", "refs/tags/blob-tag", blob)
            git("switch", "-c", "outside", historical)
            git("commit", "--allow-empty", "-m", "not main")
            git("tag", "outside-tag")
            git("update-ref", "refs/remotes/origin/main", tip)
            for tag in ("historical", "release/annotated+#%"):
                for event, ref, requested in (("push", "refs/tags/" + tag, ""),
                                               ("workflow_dispatch", "refs/heads/main", tag)):
                    self.assertEqual(pages_deploy.selection(root, event, ref, requested),
                                     {"NTESIM_DEPLOY_COMMIT": historical, "NTESIM_DEPLOY_TAG": tag})
            for blank in ("", "   "):
                self.assertEqual(pages_deploy.selection(root, "workflow_dispatch", "refs/heads/main", blank),
                                 {"NTESIM_DEPLOY_COMMIT": tip, "NTESIM_DEPLOY_TAG": ""})
            for tag in ("missing", "outside-tag", "main", historical, "historical~0", "tip^{commit}", "blob-tag"):
                with self.subTest(tag=tag), self.assertRaises(ValueError):
                    pages_deploy.selection(root, "workflow_dispatch", "refs/heads/main", tag)
            with self.assertRaisesRegex(ValueError, "must belong to fetched main history"):
                pages_deploy.selection(root, "push", "refs/tags/outside-tag")
            with self.assertRaisesRegex(ValueError, "must run from main"):
                pages_deploy.selection(root, "workflow_dispatch", "refs/heads/outside", "historical")
            with self.assertRaisesRegex(ValueError, "only tag pushes"):
                pages_deploy.selection(root, "push", "refs/heads/main")
            # Bundle transport keeps the real fetch without spawning Git's shell for a local repository.
            origin = root / "origin.bundle"
            git("bundle", "create", str(origin), "refs/heads/main", "--tags")
            git("remote", "add", "origin", str(origin))
            git("update-ref", "refs/remotes/origin/main", historical)
            selected = pages_deploy.prepare(root, {"GITHUB_EVENT_NAME": "workflow_dispatch",
                "GITHUB_REF": "refs/heads/main", "INPUT_TAG": "release/annotated+#%"})
            self.assertEqual(git("rev-parse", "HEAD"), historical)
            self.assertEqual(git("rev-parse", "origin/main"), tip)
            self.assertEqual(git("rev-parse", "--abbrev-ref", "HEAD"), "HEAD")
            self.assertEqual(studio_build.resolve(root, selected)["label"], "release/annotated+#%")

    def test_pages_workflow_has_one_build_deploy_job_without_validation(self):
        # Reuse the YAML parser installed by the project's pinned OpenSpec setup.
        workflow = json.loads(subprocess.check_output([
            "node", "-e", "const fs=require('node:fs'), yaml=require(process.argv[1]); "
            "process.stdout.write(JSON.stringify(yaml.parse(fs.readFileSync(process.argv[2],'utf8'))));",
            str(ROOT / "cache/openspec-cli/node_modules/yaml"),
            str(ROOT / ".github/workflows/deploy-pages.yml")], text=True))
        self.assertEqual(workflow["on"]["push"], {"tags": ["**"]})
        self.assertEqual(set(workflow["on"]), {"push", "workflow_dispatch"})
        inputs = workflow["on"]["workflow_dispatch"]["inputs"]
        self.assertEqual(set(inputs), {"tag"})
        self.assertEqual((inputs["tag"]["type"], inputs["tag"]["required"], inputs["tag"]["default"]),
                         ("string", False, ""))
        self.assertEqual(workflow["permissions"], {"contents": "read", "pages": "write", "id-token": "write"})
        self.assertEqual(workflow["concurrency"], {"group": "pages", "cancel-in-progress": False})
        self.assertEqual(len(workflow["jobs"]), 1)
        job = workflow["jobs"]["deploy"]
        self.assertEqual(job["environment"]["name"], "github-pages")
        self.assertEqual(job["env"]["NTESIM_BUILD_CPUS"], "all")
        self.assertEqual(job["env"]["RUSTUP_HOME"], "${{ github.workspace }}/cache/rustup")
        steps = job["steps"]
        actions = {s.get("uses", "").split("@")[0]: i for i, s in enumerate(steps) if "uses" in s}
        self.assertEqual(steps[actions["actions/checkout"]]["with"]["fetch-depth"], 0)
        self.assertEqual(steps[actions["actions/upload-pages-artifact"]]["with"]["path"], "target/web")
        commands = "\n".join(s.get("run", "") for s in steps)
        self.assertIn("build --output target/ntesim-browser-pages.zip", commands)
        self.assertNotIn("--release", commands)
        self.assertNotRegex(commands, r"\b(test|verify|checkpoint|teams)\b")
        build_index = next(i for i, s in enumerate(steps) if "build --output" in s.get("run", ""))
        select_index = next(i for i, s in enumerate(steps) if "pages_deploy.py" in s.get("run", ""))
        self.assertLess(select_index, build_index)
        self.assertLess(build_index, actions["actions/upload-pages-artifact"])
        self.assertLess(actions["actions/upload-pages-artifact"], actions["actions/deploy-pages"])
        self.assertTrue(all("if" not in step and "continue-on-error" not in step for step in steps))

    def test_build_helper_changes_distribution_identity(self):
        before = browser.source_identity()
        read = Path.read_bytes
        with patch.object(Path, "read_bytes", lambda path: read(path) + (
                b"\n# changed" if path == ROOT / "tools/studio_build.py" else b"")):
            self.assertNotEqual(browser.source_identity()["inputs_sha256"], before["inputs_sha256"])

    def test_browser_build_prefetches_workspace_dependencies_only_online(self):
        vite = ["node", "tools/web.mjs", "build"]
        for offline in (False, True):
            with self.subTest(offline=offline), temporary_directory(prefix="browser-build-") as temporary, \
                 patch.object(browser, "os", SimpleNamespace(environ={})), \
                 patch.object(browser, "studio_build", return_value={}), \
                 patch.object(browser, "cargo") as cargo, \
                 patch.object(browser.subprocess, "run", side_effect=subprocess.CalledProcessError(1, vite)) as run:
                with self.assertRaises(subprocess.CalledProcessError):
                    browser.build(Path(temporary) / "studio.zip", offline)
                expected = [] if offline else [call("fetch", "--locked")]
                expected.append(call("build", "-p", "ntesim-wasm", "--release", "--target", browser.TARGET,
                                     "--locked", *(["--offline"] if offline else [])))
                self.assertEqual(cargo.call_args_list, expected)
                self.assertEqual(run.call_args.args, (vite,))

    def test_build_identity_uses_head_and_only_explicit_tags(self):
        with git_fixture() as root:
            git = lambda *args: studio_build.git(root, *args)
            git("commit", "--allow-empty", "-m", "initial")
            commit = git("rev-parse", "HEAD")
            short = git("rev-parse", "--short=7", "HEAD")
            instant = "2026-11-14T10:20:30Z"
            env = {studio_build.INSTANT: instant}
            main = studio_build.resolve(root, env)
            self.assertEqual(studio_build.resolve(root, {**env, "NTESIM_STUDIO_BUILD": json.dumps(main)}), main)
            with self.assertRaisesRegex(ValueError, "changed during build"):
                studio_build.resolve(root, {**env, "NTESIM_STUDIO_BUILD": json.dumps({**main, "commit": "0" * 40})})
            self.assertEqual(main["label"], short)
            self.assertEqual(main["url"], studio_build.REPOSITORY + "/tree/" + short)
            self.assertEqual(main["build_started_at"], "2026-11-14T10:20:30.000Z")
            (root / "dirty").write_text("uncommitted")
            self.assertEqual(studio_build.resolve(root, env), main)
            git("switch", "-c", "some/branch")
            self.assertEqual(studio_build.resolve(root, env)["label"], "some/branch/" + short)
            tag = "release/v1+#%"
            git("tag", "-a", tag, "-m", "annotated")
            git("switch", "--detach", commit)
            self.assertEqual(studio_build.resolve(root, env), main)
            git("update-ref", "refs/remotes/origin/main", commit)
            deployed = {**env, "NTESIM_DEPLOY_COMMIT": commit, "NTESIM_DEPLOY_TAG": tag}
            identity = studio_build.resolve(root, deployed)
            self.assertEqual(identity["label"], tag)
            self.assertEqual(identity["url"], studio_build.REPOSITORY + "/tree/refs/tags/release%2Fv1%2B%23%25")
            self.assertEqual(studio_build.resolve(root, {**env, "NTESIM_DEPLOY_COMMIT": commit}), main)
            for invalid in ({"NTESIM_DEPLOY_TAG": tag}, {"NTESIM_DEPLOY_COMMIT": "0" * 40},
                            {"NTESIM_DEPLOY_COMMIT": commit, "NTESIM_DEPLOY_TAG": "main"}):
                with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                    studio_build.resolve(root, {**env, **invalid})
            with self.assertRaisesRegex(ValueError, "UTC offset"):
                studio_build.resolve(root, {studio_build.INSTANT: "2026-11-14T10:20:30"})

    def test_staging_keeps_supplied_build_context(self):
        identity = studio_build.resolve(ROOT, {studio_build.INSTANT: "2026-11-14T10:20:30Z"})
        read = Path.read_bytes
        generated = {ROOT / "target/wasm-bindgen/ntesim_wasm_bg.wasm",
                     ROOT / "cache/web/node_modules/@bokuweb/zstd-wasm/dist/web/zstd.wasm"}
        with temporary_directory(prefix="studio-stage-") as temporary, \
             patch.dict(os.environ, {studio_build.INSTANT: identity["build_started_at"],
                                     "NTESIM_STUDIO_BUILD": json.dumps(identity)}), \
             patch.object(Path, "read_bytes", lambda path: b"fixture wasm" if path in generated else read(path)), \
             patch.object(browser, "cargo", return_value="{}"), \
             patch.object(browser, "resolve_toolchain", return_value=Path(temporary) / "bin"), \
             patch.object(browser, "runtime_dependencies", return_value=[]), \
             patch.object(browser, "dependency_notices", return_value=({}, [])), \
             patch.object(browser, "standard_library_notices", return_value={}), \
             patch.object(browser, "npm_notices", return_value=({}, [])):
            info = browser.stage(Path(temporary))
            self.assertEqual(info["studio_build"], identity)
            self.assertEqual(json.loads((Path(temporary) / "build-info.json").read_bytes())["studio_build"], identity)

    def test_npm_runtime_closure_binds_versions_integrities_and_license_texts(self):
        with temporary_directory(prefix="browser-npm-") as temporary:
            modules = Path(temporary)
            entries = {"": {"dependencies": {"react": "1"}}}
            for name, dependencies in (("react", {"scheduler": "1"}), ("scheduler", {}), ("vite", {})):
                directory = modules / name
                directory.mkdir()
                (directory / "package.json").write_text(json.dumps({"name": name, "version": "1"}))
                (directory / "LICENSE").write_bytes(b"full upstream license")
                entries["node_modules/" + name] = {"version": "1", "license": "MIT",
                    "resolved": "https://registry.example/" + name, "integrity": "sha512-pinned",
                    "dependencies": dependencies}
            files, inventory = browser.npm_notices({"packages": entries}, modules)
            self.assertEqual([p["name"] for p in inventory], ["react", "scheduler"])
            self.assertEqual(files["licenses/npm/react/LICENSE"], b"full upstream license")
            entries["node_modules/react"]["version"] = "2"
            with self.assertRaisesRegex(ValueError, "differs from lock"):
                browser.npm_notices({"packages": entries}, modules)
            entries["node_modules/react"]["version"] = "1"
            (modules / "scheduler/LICENSE").unlink()
            with self.assertRaisesRegex(ValueError, "missing npm license"):
                browser.npm_notices({"packages": entries}, modules)

    def test_browser_payload_rejects_unreviewed_material_and_missing_or_wrong_wasm(self):
        with temporary_directory(prefix="browser-payload-") as temporary:
            directory = Path(temporary)
            info = {"artifact": "ntesim-browser", "target": browser.TARGET,
                    "wasm_sha256": distribution.digest(b"wasm"),
                    "report_codec_wasm_sha256": distribution.digest(b"codec-wasm"),
                    "rust_dependencies": [{"license_files": ["licenses/dependency.txt"]}],
                    "npm_dependencies": []}
            files = {"index.html": b"html", "sw.js": b"worker", "NOTICE": b"notice",
                     "LICENSE": b"license", "licenses/runtime.txt": b"react",
                     "licenses/dependency.txt": b"upstream", "assets/engine.wasm": b"wasm",
                     "assets/zstd.wasm": b"codec-wasm",
                     "build-info.json": json.dumps(info).encode()}
            for name, raw in files.items():
                p = directory / name
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(raw)
            self.assertEqual(browser.validate(browser.payload(directory), False), info)
            artwork = next((ROOT / "web/public/assets/artwork").glob("*.webp"))
            image = directory / "assets/artwork" / artwork.name
            image.parent.mkdir()
            image.write_bytes(artwork.read_bytes())
            self.assertEqual(browser.payload(directory)["assets/artwork/" + artwork.name],
                             artwork.read_bytes())
            (directory / "assets/game.webp").write_bytes(b"game artwork")
            with self.assertRaisesRegex(ValueError, "unreviewed browser payload"):
                browser.payload(directory)
            with self.assertRaisesRegex(ValueError, "WASM identity"):
                browser.validate({**files, "assets/engine.wasm": b"changed"}, False)
            with self.assertRaisesRegex(ValueError, "codec WASM identity"):
                browser.validate({**files, "assets/zstd.wasm": b"changed"}, False)
            with self.assertRaisesRegex(ValueError, "WASM identity"):
                browser.validate({k: v for k, v in files.items() if k != 'assets/zstd.wasm'}, False)
            del files["licenses/dependency.txt"]
            with self.assertRaisesRegex(ValueError, "missing browser dependency notices"):
                browser.validate(files, False)
            for release in (False, True):
                output = directory / ("release.zip" if release else "local.zip")
                with patch.object(browser, "studio_build", return_value={"commit": "fixture"}), \
                     patch.object(browser, "cargo") as cargo, \
                     patch.object(browser.subprocess, "run") as run, \
                     patch.object(browser, "payload", return_value={}), \
                     patch.object(browser, "validate", return_value={}), \
                     patch.object(browser, "write_archive", side_effect=lambda path, *_: path.write_bytes(b"archive")):
                    browser.build(output, True, release)
                cargo.assert_called_once()
                commands = [call.args[0] for call in run.call_args_list]
                expected = [["node", "tools/web.mjs", "teams", "--fresh"]] if release else []
                self.assertEqual(commands, expected + [["node", "tools/web.mjs", "build"]])
                instants = {call.kwargs["env"][studio_build.INSTANT] for call in run.call_args_list}
                self.assertEqual(len(instants), 1)
                for call in run.call_args_list:
                    for key in ("GITHUB_ACTIONS", "NTESIM_BUILD_CPUS"):
                        self.assertEqual(call.kwargs["env"].get(key), os.environ.get(key))


class NativeDistributionTests(unittest.TestCase):
    def test_default_runtime_payloads_reject_benchmark_and_profiling_code(self):
        for name, raw in (
            ("ntesim-throughput.exe", b"benchmark"),
            ("assets/worker.js", b"import 'node:inspector'"),
            ("assets/engine.wasm", b"\0source.from_configuration\0"),
            ("ntesim.exe", b"usage: ntesim-throughput SOURCE COUNT"),
            ("assets/worker.js", b"DEVELOPMENT_MEASURE"),
            ("trace.cpuprofile", b"{}"),
            ("ntesim.pdb", b"symbols"),
            ("training.profraw", b"instrumentation counters"),
            ("merged.profdata", b"merged profile"),
        ):
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "development measurement"):
                distribution.validate_runtime_artifacts({name: raw})
        # Documentation may describe the explicit developer commands.
        distribution.validate_runtime_artifacts({"README.md": b"ntesim-throughput", "ntesim.exe": b"portable engine"})

    def test_cli_dependency_closure_excludes_dev_and_unrelated_wasm(self):
        packages = [{"id": name, "name": name, "version": "1", "source": source}
                    for name, source in (("ntesim-cli", None), ("runtime", "registry"),
                                         ("macro", "registry"), ("test-only", "registry"),
                                         ("wasm-bindgen", "registry"))]
        metadata = {"packages": packages, "resolve": {"nodes": [
            {"id": "ntesim-cli", "deps": [
                {"pkg": "runtime", "dep_kinds": [{"kind": None}]},
                {"pkg": "macro", "dep_kinds": [{"kind": "build"}]},
                {"pkg": "test-only", "dep_kinds": [{"kind": "dev"}]}]},
            *({"id": p["id"], "deps": []} for p in packages[1:])]}}
        self.assertEqual([p["name"] for p in distribution.runtime_dependencies(metadata, "ntesim-cli")], ["macro", "runtime"])

    def test_license_bytes_and_registry_checksum_remain_attached_to_dependency(self):
        with temporary_directory(prefix="distribution-license-") as temporary:
            root = Path(temporary)
            (root / "LICENSE-MIT").write_bytes(b"upstream MIT text\n")
            (root / "LICENSE-UNICODE").write_bytes(b"upstream Unicode text\n")
            package = {"name": "unicode", "version": "1.2", "source": "registry",
                       "manifest_path": str(root / "Cargo.toml"), "license": "MIT AND Unicode-3.0"}
            lock = {"package": [{"name": "unicode", "version": "1.2", "source": "registry", "checksum": "abc"}]}
            files, inventory = distribution.dependency_notices([package], lock)
            self.assertEqual(files["licenses/unicode-1.2/LICENSE-UNICODE"], b"upstream Unicode text\n")
            self.assertEqual(inventory[0]["checksum"], "abc")
            self.assertEqual(set(inventory[0]["license_files"]), set(files))
            with self.assertRaisesRegex(ValueError, "pinned registry checksum"):
                distribution.dependency_notices([package], {"package": []})
            for path in root.glob("LICENSE*"):
                path.unlink()
            with self.assertRaisesRegex(ValueError, "missing dependency license"):
                distribution.dependency_notices([package], lock)
            with self.assertRaisesRegex(ValueError, "standard-library notice"):
                distribution.standard_library_notices(root)

    def test_archive_is_deterministic_and_rejects_changed_or_extra_payload(self):
        with temporary_directory(prefix="distribution-zip-") as temporary:
            root = Path(temporary)
            files = {"ntesim": b"executable", "LICENSE": b"original license"}
            first, second = root / "first.zip", root / "second.zip"
            distribution.write_archive(first, files, {"target": "fixture"})
            distribution.write_archive(second, dict(reversed(list(files.items()))), {"target": "fixture"})
            self.assertEqual(first.read_bytes(), second.read_bytes())
            manifest, restored = distribution.read_archive(first)
            self.assertEqual(restored, files)
            self.assertEqual(manifest["files"]["ntesim"]["sha256"], distribution.digest(b"executable"))
            first.with_suffix(".zip.sha256").write_text("wrong  first.zip\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "archive checksum mismatch"):
                distribution.smoke(first)
            stale = root / "stale.zip"
            distribution.write_archive(stale, files, {
                "engine_source_sha256": "stale", "cargo_lock_sha256": "stale", "packager_sha256": "stale"})
            stale.with_suffix(".zip.sha256").write_text(distribution.digest(stale.read_bytes()), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "archive is stale"):
                distribution.smoke(stale, current=True)
            with zipfile.ZipFile(root / "changed.zip", "w") as archive:
                archive.writestr("manifest.json", json.dumps(manifest))
                archive.writestr("ntesim", b"changed")
                archive.writestr("LICENSE", files["LICENSE"])
            with self.assertRaisesRegex(ValueError, "hash/size mismatch"):
                distribution.read_archive(root / "changed.zip")
            with zipfile.ZipFile(second, "a") as archive:
                archive.writestr("unexpected-data.json", b"{}")
            with self.assertRaisesRegex(ValueError, "inventory mismatch"):
                distribution.read_archive(second)

    def test_archive_rejects_traversal_and_duplicate_entries(self):
        with temporary_directory(prefix="distribution-invalid-") as temporary:
            for index, name in enumerate(("../outside", "/absolute", "C:/drive", "a/../b")):
                path = Path(temporary) / f"{index}.zip"
                with zipfile.ZipFile(path, "w") as archive:
                    archive.writestr("manifest.json", "{}")
                    archive.writestr(name, b"bad")
                with self.assertRaisesRegex(ValueError, "unsafe archive path"):
                    distribution.read_archive(path)
            path = Path(temporary) / "duplicate.zip"
            with zipfile.ZipFile(path, "w") as archive, warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                archive.writestr("manifest.json", "{}")
                archive.writestr("manifest.json", "{}")
            with self.assertRaisesRegex(ValueError, "duplicate entries"):
                distribution.read_archive(path)


if __name__ == "__main__":
    unittest.main()
