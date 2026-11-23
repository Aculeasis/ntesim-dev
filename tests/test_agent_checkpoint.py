import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import checkpoint
from verification_workspace import temporary_directory


class CheckpointTests(unittest.TestCase):
    def setUp(self):
        self.workspace = temporary_directory(prefix="checkpoint space-")
        self.root = Path(self.workspace.__enter__())
        self.addCleanup(self.workspace.__exit__, None, None, None)
        self.addCleanup(self.make_fixture_objects_writable)
        (self.root / "tools").mkdir()
        for name in ("checkpoint.py", "tool_output.py", "cpu_limit.py"):
            shutil.copyfile(ROOT / "tools" / name, self.root / "tools" / name)
        (self.root / ".githooks").mkdir()
        shutil.copyfile(ROOT / ".githooks/pre-commit", self.root / ".githooks/pre-commit")
        (self.root / ".githooks/pre-commit").chmod(0o755)
        checkpoint.prepare_hook(self.root)
        (self.root / ".gitignore").write_text("cache/\n.cache/\ntarget/\n__pycache__/\n", encoding="utf-8")
        (self.root / ".gitattributes").write_text("*.md text eol=lf\n", encoding="utf-8")
        (self.root / "README.md").write_text("# Baseline\n", encoding="utf-8")
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "core.hooksPath=cache/no-hooks", "commit", "-qm", "Create fixture")

    def make_fixture_objects_writable(self):
        if os.name == "nt":
            for path in (self.root / ".git/objects").rglob("*"):
                if path.is_file():
                    self.assertTrue(path.resolve().is_relative_to(self.root.resolve()))
                    path.chmod(path.stat().st_mode | stat.S_IWRITE)

    def git(self, *args, success=True):
        result = subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Checkpoint Test",
                                 "-c", "user.email=checkpoint@example.invalid", "-c", "core.hooksPath=cache/checkpoint/hooks",
                                 *args], capture_output=True, text=True, encoding="utf-8",
                                env=os.environ.copy())
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        return result

    def change_readme(self, text="# Changed\n"):
        (self.root / "README.md").write_text(text, encoding="utf-8")

    def test_privacy_boundary_rejects_host_paths_proxies_and_tokens_without_echoing_values(self):
        values = [str(self.root / "private-input.json"),
                  "/".join(("", "home", "example-person", "private-input")),
                  "sk-" + "x" * 32,
                  'API_KEY="' + 'x' * 32 + '"',
                  "HTTPS_PROXY=http://" + "127.0.0.1" + ":9999",
                  "HTTP_PROXY=https://" + "proxy.example.test" + ":9999"]
        for value in values:
            with self.subTest(category=values.index(value)):
                self.change_readme(value)
                result = self.run_cli("privacy")
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("README.md:1:", result.stderr)
                self.assertNotIn(value, result.stderr)
        self.change_readme("# Portable\nPublic attribution: Example Contributor\n")
        (self.root / 'tests').mkdir()
        fixture = self.root / 'tests/paths.py'
        fixture.write_text('path = ' + repr('C:' + '/secret'), encoding='utf-8')
        self.assertEqual(self.run_cli('privacy').returncode, 0)

    def test_privacy_boundary_rejects_forced_tracked_ignored_configuration(self):
        ignore = self.root / '.gitignore'
        ignore.write_text(ignore.read_text() + '.local/\n.env*\n.venv/\n')
        for name in ('.local/.env', '.env.private', '.venv/config'):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('PRIVATE=do-not-print', encoding='utf-8')
            self.assertEqual(self.run_cli('privacy').returncode, 0)
            self.git('add', '-f', name)
            result = self.run_cli('privacy')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('private configuration', result.stderr)
            self.assertNotIn('do-not-print', result.stderr)
            self.git('rm', '--cached', '-f', name)

    def test_privacy_boundary_is_always_selected_and_never_cached(self):
        for changed in ([], ['README.md'], ['tools/checkpoint.py'], ['crates/core/src/lib.rs']):
            plan = checkpoint.check_plan(ROOT, changed)
            item = next(item for item in plan if item['name'] == 'repository-privacy')
            self.assertEqual(item['command'], ['python', 'tools/checkpoint.py', 'privacy'])
            self.assertIsNone(checkpoint.check_inputs(item, {'README.md': {}}))

    def run_cli(self, *args):
        return subprocess.run([sys.executable, str(self.root / "tools/checkpoint.py"), *args],
                              cwd=self.root, capture_output=True, text=True, encoding="utf-8",
                              env=os.environ.copy())

    def test_git_preserves_empty_config_environment_value(self):
        index = int(os.environ.get("GIT_CONFIG_COUNT", "0"))
        with patch.dict(os.environ, {
            "GIT_CONFIG_COUNT": str(index + 1),
            f"GIT_CONFIG_KEY_{index}": "checkpoint.testValue",
            f"GIT_CONFIG_VALUE_{index}": "",
        }):
            self.assertEqual(checkpoint.git(self.root, "config", "--get", "checkpoint.testValue"), b"\n")

    def test_real_precommit_rejects_missing_receipt_then_accepts_checked_tree(self):
        self.change_readme()
        self.git("add", "README.md")
        rejected = self.git("commit", "-qm", "Unchecked change", success=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("no successful checkpoint receipt", rejected.stderr)
        checked = self.run_cli("run")
        self.assertEqual((checked.returncode, checked.stdout, checked.stderr), (0, "", ""))
        self.git("commit", "-qm", "Checked change")

    def test_cached_batch_artifact_is_separate_from_tracked_curated_inputs(self):
        curated = self.root / 'reviews/curated.csv'
        curated.parent.mkdir()
        curated.write_bytes(b'consumed,fixture\n')
        self.git('add', 'reviews/curated.csv')
        before = checkpoint.worktree_files(self.root)
        item = checkpoint.check('native-wasm-parity', ['python'])
        artifacts = checkpoint.artifact_inputs(self.root, item)
        cached = self.root / 'cache/reviews/q10-research-batch/pilot-request.json'
        cached.parent.mkdir(parents=True)
        cached.write_bytes(b'{"current":true}')
        self.assertEqual(checkpoint.worktree_files(self.root), before)
        self.assertIn('reviews/curated.csv', checkpoint.check_inputs(item, before))
        self.assertNotEqual(checkpoint.artifact_inputs(self.root, item), artifacts)

    def test_installation_is_persistent_idempotent_and_preserves_custom_hooks(self):
        checkpoint.install_hook(self.root)
        hook = self.root / ".git/hooks/pre-commit"
        first = hook.read_bytes()
        self.assertTrue(first.startswith(f"#!{Path(sys.executable).as_posix()}\n".encode()))
        checkpoint.install_hook(self.root)
        self.assertEqual(hook.read_bytes(), first)
        hook.write_text("custom hook", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "existing pre-commit"):
            checkpoint.install_hook(self.root)
        self.assertEqual(hook.read_text(encoding="utf-8"), "custom hook")
        self.git("config", "--local", "core.hooksPath", "custom-hooks")
        with self.assertRaisesRegex(ValueError, "existing core.hooksPath"):
            checkpoint.install_hook(self.root)

    def test_run_checkpoint_automatically_installs_missing_hook(self):
        hook = self.root / ".git/hooks/pre-commit"
        self.assertFalse(hook.exists())
        self.change_readme()
        self.assertEqual(self.run_cli("run").returncode, 0)
        self.assertTrue(hook.is_file())
        self.assertTrue(hook.read_bytes().startswith(f"#!{Path(sys.executable).as_posix()}\n".encode()))

    def test_ensure_hook_skips_when_core_hooks_path_configured(self):
        self.git("config", "--local", "core.hooksPath", "custom-hooks")
        checkpoint.ensure_hook(self.root)
        hook = self.root / ".git/hooks/pre-commit"
        self.assertFalse(hook.exists())

    def test_stale_worktree_and_new_head_cannot_reuse_receipt(self):
        self.change_readme()
        self.assertEqual(self.run_cli("run").returncode, 0)
        self.git("add", "README.md")
        self.change_readme("# Changed again\n")
        self.git("add", "README.md")
        with self.assertRaisesRegex(ValueError, "inputs changed"):
            checkpoint.verify_checkpoint(self.root)
        self.assertEqual(self.run_cli("run").returncode, 0)
        self.git("commit", "-qm", "Checked change")
        self.change_readme("# Next change\n")
        self.git("add", "README.md")
        with self.assertRaisesRegex(ValueError, "another revision"):
            checkpoint.verify_checkpoint(self.root)

    def test_partial_staging_and_untracked_dependency_are_rejected_without_touching_edits(self):
        self.change_readme()
        other = self.root / "notes space é.md"
        other.write_text("# Extra dependency\n", encoding="utf-8")
        self.git("add", "README.md")
        self.assertEqual(self.run_cli("run").returncode, 0)
        with self.assertRaisesRegex(ValueError, "staged tree differs"):
            checkpoint.verify_checkpoint(self.root)
        self.assertEqual(other.read_text(encoding="utf-8"), "# Extra dependency\n")
        self.git("add", other.name)
        checkpoint.verify_checkpoint(self.root)
        self.change_readme("# Unstaged edit\n")
        self.assertEqual(self.run_cli("run").returncode, 0)
        with self.assertRaisesRegex(ValueError, "staged tree differs"):
            checkpoint.verify_checkpoint(self.root)

    def test_failed_rerun_removes_success_and_preserves_exit_two(self):
        self.change_readme()
        self.git("add", "README.md")
        self.assertEqual(self.run_cli("run").returncode, 0)
        extras = self.root / "cache/extra-checks.json"
        extras.write_text(json.dumps([[sys.executable, "-c", "import sys; sys.exit(2)"]]), encoding="utf-8")
        failed = self.run_cli("run", "--extra-checks", str(extras))
        self.assertEqual(failed.returncode, 2, failed.stderr)
        self.assertFalse(checkpoint.receipt_path(self.root).exists())
        with self.assertRaisesRegex(ValueError, "no successful"):
            checkpoint.verify_checkpoint(self.root)
        self.assertEqual(self.run_cli("run").returncode, 0)
        extras.write_text("invalid JSON", encoding="utf-8")
        self.assertNotEqual(self.run_cli("run", "--extra-checks", str(extras)).returncode, 0)
        self.assertFalse(checkpoint.receipt_path(self.root).exists())

    def test_timings_record_every_check_and_failure_without_changing_receipt_contract(self):
        self.change_readme()
        checks = [checkpoint.check("first", ["python", "first.py"]),
                  checkpoint.check("second", ["python", "second.py"])]
        for status in (0, 2):
            with self.subTest(status=status), \
                 patch.object(checkpoint, "check_plan", return_value=checks), \
                 patch.object(checkpoint.time, "perf_counter", side_effect=[0, 1, 3, 4, 7, 8]):
                executed = []
                def execute(command, **kwargs):
                    executed.append(command)
                    return status if len(executed) == 2 else 0
                self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute, fresh=True), status)
                timing = json.loads(checkpoint.receipt_path(self.root).with_name("timings.json").read_text())
                self.assertEqual(timing["seconds"], 8)
                self.assertEqual(timing["checks"], [
                    {"name": "first", "seconds": 2, "exit_code": 0},
                    {"name": "second", "seconds": 3, "exit_code": status},
                ])
                self.assertEqual(checkpoint.receipt_path(self.root).exists(), status == 0)
                if status == 0:
                    self.git("add", "README.md")
                    checkpoint.verify_checkpoint(self.root)

    def test_edits_during_checks_and_missing_required_checks_are_rejected(self):
        self.change_readme()
        def mutate(command, **kwargs):
            self.change_readme("# Concurrent edit\n")
            return 0
        with self.assertRaisesRegex(ValueError, "inputs changed during"):
            checkpoint.run_checkpoint(self.root, execute=mutate)
        self.assertFalse(checkpoint.receipt_path(self.root).exists())
        self.assertEqual(self.run_cli("run").returncode, 0)
        self.git("add", "README.md")
        receipt = checkpoint.receipt_path(self.root)
        saved = json.loads(receipt.read_text())
        saved["checks"] = []
        receipt.write_text(json.dumps(saved), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "required checks are missing"):
            checkpoint.verify_checkpoint(self.root)

    def test_deleted_file_and_crlf_staging_use_git_content_identity(self):
        note = self.root / "old.md"
        note.write_text("# Old\n", encoding="utf-8")
        self.git("add", "old.md")
        self.assertEqual(self.run_cli("run").returncode, 0)
        self.git("commit", "-qm", "Add note")
        note.unlink()
        (self.root / "README.md").write_bytes(b"# Windows text\r\n")
        self.assertEqual(self.run_cli("run").returncode, 0)
        self.git("add", "-A")
        checkpoint.verify_checkpoint(self.root)

    def test_links_anchors_and_structured_files_fail_closed(self):
        cases = ["[broken](missing.md)", "[broken](README.md#missing)",
                 "[broken](#fake)\n\n```md\n# Fake\n```\n"]
        for text in cases:
            with self.subTest(text=text):
                self.change_readme(text)
                with self.assertRaisesRegex(ValueError, "missing local"):
                    checkpoint.static_checks(self.root, ["README.md"])
        self.change_readme('# Heading\n\n[ok](#heading) [external](https://example.invalid/page)\n')
        checkpoint.static_checks(self.root, ["README.md"])
        (self.root / "bad.json").write_text("invalid", encoding="utf-8")
        with self.assertRaises(ValueError):
            checkpoint.static_checks(self.root, ["bad.json"])
        with self.assertRaisesRegex(ValueError, "classify new"):
            checkpoint.check_plan(self.root, ["unknown.config"])
        new_config = self.root / "configs/new.json"
        new_config.parent.mkdir()
        new_config.write_text("{}", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "classify new"):
            checkpoint.check_plan(self.root, ["configs/new.json"])

    def test_deleted_markdown_checks_inbound_links(self):
        self.change_readme("[gone](docs/removed.md)\n")
        with self.assertRaisesRegex(ValueError, "missing local link target"):
            checkpoint.static_checks(self.root, ["docs/removed.md"])

    def test_syntax_checks_and_check_links_subcommands(self):
        valid_json = self.root / "valid.json"
        valid_json.write_text('{"key": "value"}', encoding="utf-8")
        valid_toml = self.root / "valid.toml"
        valid_toml.write_text('key = "value"\n', encoding="utf-8")
        checkpoint.syntax_checks(self.root, ["valid.json", "valid.toml"])

        bad_json = self.root / "broken.json"
        bad_json.write_text("{broken", encoding="utf-8")
        with self.assertRaises(ValueError):
            checkpoint.syntax_checks(self.root, ["broken.json"])

        self.change_readme("# Valid Header\n\n[self](#valid-header)\n")
        checkpoint.check_links(self.root, ["README.md"])

        self.change_readme("[broken](missing_file.md)\n")
        with self.assertRaisesRegex(ValueError, "missing local link target"):
            checkpoint.check_links(self.root, ["README.md"])

        # Structured-syntax check selection
        cfg = self.root / "tools/quality/test.json"
        cfg.parent.mkdir(parents=True, exist_ok=True)
        cfg.write_text('{"key": "value"}', encoding="utf-8")
        plan = checkpoint.check_plan(self.root, ["tools/quality/test.json"])
        names = {item["name"] for item in plan}
        self.assertIn("structured-syntax", names)
        self.assertNotIn("local-links-and-formats", names)

    def test_rerun_reuses_checks_across_docs_and_commit_but_invalidates_changed_dependencies(self):
        source = self.root / "crates/core/src/lib.rs"
        source.parent.mkdir(parents=True)
        source.write_text("original", encoding="utf-8")
        checked = checkpoint.check("rust-tests", ["python", "sentinel.py"])
        calls = []
        def execute(command, **kwargs):
            calls.append(command)
            return 0
        with patch.object(checkpoint, "check_plan", return_value=[checked]):
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.change_readme()
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 1)
            timing = json.loads(checkpoint.receipt_path(self.root).with_name("timings.json").read_text())
            self.assertTrue(timing["checks"][0]["reused"])
            self.git("add", "-A")
            checkpoint.verify_checkpoint(self.root)
            self.git("-c", "core.hooksPath=cache/no-hooks", "commit", "-qm", "Checked fixture")
            self.change_readme("# Another doc\n")
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 1)
            for change in ("changed", None, "new"):
                if change is None:
                    source.unlink()
                else:
                    source.write_text(change, encoding="utf-8")
                self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 4)
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute, fresh=True), 0)
            self.assertEqual(len(calls), 5)

    def test_failed_check_reruns_while_prior_success_survives(self):
        self.change_readme()
        checks = [checkpoint.check("python-tests", ["python", "passing.py"]),
                  checkpoint.check("web-types", ["python", "failing.py"])]
        calls = []
        def execute(command, **kwargs):
            calls.append(command[-1])
            return 2 if command[-1] == "failing.py" else 0
        with patch.object(checkpoint, "check_plan", return_value=checks):
            for _ in range(2):
                self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 2)
                self.assertFalse(checkpoint.receipt_path(self.root).exists())
            self.assertEqual(calls, ["passing.py", "failing.py", "failing.py"])
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=lambda *a, **kw: 0), 0)
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute, fresh=True), 2)
            self.assertFalse(checkpoint.receipt_path(self.root).exists())
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 2)
            self.assertEqual(calls[-3:], ["passing.py", "failing.py", "failing.py"])

    def test_extra_checks_run_even_when_ignored_external_dependencies_change(self):
        self.change_readme()
        counter = self.root / "cache/counter.txt"
        script = self.root / "cache/extra.py"
        script.write_text("from pathlib import Path\np=Path('cache/counter.txt')\n"
                          "p.write_text(p.read_text()+'x' if p.exists() else 'x')\n", encoding="utf-8")
        extras = self.root / "cache/extra-checks.json"
        extras.write_text(json.dumps([["python", str(script)]]), encoding="utf-8")
        for _ in range(2):
            result = self.run_cli("run", "--extra-checks", str(extras))
            self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(counter.read_text(), "xx")
        script.write_text("import sys\nsys.exit(2)\n", encoding="utf-8")
        self.assertEqual(self.run_cli("run", "--extra-checks", str(extras)).returncode, 2)
        self.assertFalse(checkpoint.receipt_path(self.root).exists())

    def test_cache_changes_command_environment_artifact_and_corrupt_or_missing_cache(self):
        self.change_readme()
        checked = checkpoint.check("native-wasm-parity", ["python", "parity.py"])
        native = self.root / ("target/debug/ntesim.exe" if os.name == "nt" else "target/debug/ntesim")
        native.parent.mkdir(parents=True)
        native.write_bytes(b"first")
        calls = []
        def execute(command, **kwargs):
            calls.append(command)
            return 0
        with patch.object(checkpoint, "check_plan", return_value=[checked]):
            checkpoint.run_checkpoint(self.root, execute=execute)
            checkpoint.run_checkpoint(self.root, execute=execute)
            self.assertEqual(len(calls), 1)
            with patch.dict(os.environ, {"NTESIM_TOOL_VERBOSE": "1"}):
                checkpoint.run_checkpoint(self.root, execute=execute)
            self.assertEqual(len(calls), 1)
            native.write_bytes(b"second")
            checkpoint.run_checkpoint(self.root, execute=execute)
            native.unlink()
            checkpoint.run_checkpoint(self.root, execute=execute)
            checked["command"].append("--new")
            checkpoint.run_checkpoint(self.root, execute=execute)
            with patch.dict(os.environ, {"NTESIM_TEST_STRESS": "1"}):
                checkpoint.run_checkpoint(self.root, execute=execute)
            checked["env"]["MODE"] = "new"
            checkpoint.run_checkpoint(self.root, execute=execute)
            cache = checkpoint.receipt_path(self.root).with_name("successes.json")
            cache.write_text("not JSON", encoding="utf-8")
            checkpoint.run_checkpoint(self.root, execute=execute)
            cache.unlink()
            checkpoint.run_checkpoint(self.root, execute=execute)
            self.assertEqual(len(calls), 8)

    def test_mutation_during_checks_cannot_populate_reusable_success(self):
        self.change_readme()
        checked = checkpoint.check("python-tests", ["python", "sentinel.py"])
        def mutate(command, **kwargs):
            self.change_readme("# Mutated\n")
            return 0
        with patch.object(checkpoint, "check_plan", return_value=[checked]):
            with self.assertRaisesRegex(ValueError, "inputs changed during"):
                checkpoint.run_checkpoint(self.root, execute=mutate)
            self.assertEqual(json.loads(checkpoint.receipt_path(self.root).with_name("successes.json").read_text()), {})

    def test_mutation_keeps_independent_success_but_never_authorizes_commit(self):
        self.change_readme()
        source = self.root / "web/src/App.tsx"
        source.parent.mkdir(parents=True)
        source.write_text("initial", encoding="utf-8")
        checks = [checkpoint.check("rust-format", ["python", "format.py"]),
                  checkpoint.check("web-types", ["python", "types.py"])]
        calls = []
        def execute(command, **kwargs):
            calls.append(command[-1])
            if command[-1] == "types.py":
                source.write_text("changed", encoding="utf-8")
            return 0
        with patch.object(checkpoint, "check_plan", return_value=checks):
            with self.assertRaisesRegex(ValueError, "web/src/App.tsx"):
                checkpoint.run_checkpoint(self.root, execute=execute)
            self.assertFalse(checkpoint.receipt_path(self.root).exists())
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=lambda command, **kw: calls.append(command[-1]) or 0), 0)
        self.assertEqual(calls, ["format.py", "types.py", "types.py"])

    def test_command_sandbox_temp_paths_use_stable_execution_environment_and_reuse(self):
        self.change_readme()
        checked = checkpoint.check("python-tests", ["python", "sentinel.py"])
        calls = []
        def execute(command, **kwargs):
            calls.append(kwargs["env"])
            return 0
        with patch.object(checkpoint, "check_plan", return_value=[checked]):
            for value in ("first-sandbox", "second-sandbox"):
                with patch.dict(os.environ, {"TEMP": value, "TMP": value,
                                             **({"LOCALAPPDATA": value} if os.name == "nt" else {})}):
                    self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
        self.assertEqual(len(calls), 1)
        for key in ("TEMP", "TMP", "LOCALAPPDATA") if os.name == "nt" else ("TEMP", "TMP"):
            path = Path(calls[0][key])
            self.assertTrue(path.is_relative_to(self.root))
            self.assertTrue(path.is_dir())

    def test_granular_package_caching_skips_unchanged_upstream_packages(self):
        self.change_readme()
        source_content = self.root / "crates/content/src/lib.rs"
        source_content.parent.mkdir(parents=True, exist_ok=True)
        source_content.write_text("fn content() {}", encoding="utf-8")
        source_research = self.root / "crates/research/src/lib.rs"
        source_research.parent.mkdir(parents=True, exist_ok=True)
        source_research.write_text("fn research() {}", encoding="utf-8")

        checked = checkpoint.check("rust-tests", ["python", "tools/cargo.py", "test",
                                                  "--locked", "--offline",
                                                  "-p", "ntesim-content",
                                                  "-p", "ntesim-research"])
        calls = []
        def execute(command, **kwargs):
            calls.append(command)
            return 0

        with patch.object(checkpoint, "check_plan", return_value=[checked]):
            # First run: both packages execute sequentially
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 2)
            self.assertIn("-p", calls[0])
            self.assertIn("ntesim-content", calls[0])
            self.assertIn("ntesim-research", calls[1])

            # Immediate rerun: fully reused
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 2)

            # Change only research: content is skipped, only research runs
            source_research.write_text("fn research_v2() {}", encoding="utf-8")
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 3)
            self.assertIn("ntesim-research", calls[2])
            self.assertNotIn("ntesim-content", calls[2])

    def test_native_build_caching_and_artifact_invalidation(self):
        self.change_readme()
        native = self.root / ("target/debug/ntesim.exe" if os.name == "nt" else "target/debug/ntesim")
        native.parent.mkdir(parents=True, exist_ok=True)
        native.write_bytes(b"binary-v1")
        checked = checkpoint.check("native-build", ["python", "tools/cargo.py", "build",
                                                    "-p", "ntesim-cli", "--locked", "--offline"])
        calls = []
        def execute(command, **kwargs):
            calls.append(command)
            return 0

        with patch.object(checkpoint, "check_plan", return_value=[checked]):
            # First run executes build
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 1)

            # Second run with unchanged binary and sources reuses cache
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 1)

            # Deleting the artifact invalidates cache and triggers rebuild
            native.unlink()
            self.assertEqual(checkpoint.run_checkpoint(self.root, execute=execute), 0)
            self.assertEqual(len(calls), 2)


class CheckSelectionTests(unittest.TestCase):
    def names(self, *paths):
        return {item["name"] for item in checkpoint.check_plan(ROOT, list(paths))}

    def test_boundary_matrix_separates_native_api_smoke_and_full_browser(self):
        cases = [
            (("docs/DEVELOPMENT.md",), {"local-links-and-formats"}, {"rust-tests", "web-tests"}),
            (("tools/checkpoint.py",), {"python-tests"}, {"rust-quality", "web-tests"}),
            ((".github/workflows/deploy-pages.yml",), {"python-tests"}, {"rust-tests", "web-tests"}),
            (("tools/pages_deploy.py",), {"python-tests"}, {"rust-tests", "web-tests"}),
            (("tools/studio_build.py",), {"python-tests", "web-tests", "wasm-build"}, {"rust-tests"}),
            (("configs/verifier-paths.txt",), {"python-tests"}, {"rust-tests", "wasm-build", "web-tests"}),
            (("crates/research/tests/fixtures/profile-variants.csv",), {"rust-tests"}, {"rust-format", "rust-quality", "wasm-build", "web-tests"}),
            (("presets/research/variants.csv",), {"rust-tests", "native-build", "wasm-build", "wasm-smoke"}, {"web-tests", "native-wasm-parity"}),
            (("crates/content/src/simulation/tests/ordinary_hit_energy.rs",),
             {"rust-tests", "rust-quality", "rust-format"}, {"wasm-build"}),
            (("crates/content/src/characters/mint/kit.rs",),
             {"rust-tests", "rust-quality", "native-build"},
             {"wasm-build", "native-wasm-parity", "web-tests", "web-types", "web-format", "wasm-smoke"}),
            (("crates/wasm/src/source_api.rs",), {"wasm-build", "wasm-smoke", "native-build", "rust-tests"},
             {"native-wasm-parity", "web-tests", "web-types", "web-format"}),
            (("crates/content/src/dsl/inspection.rs",), {"wasm-build", "wasm-smoke", "native-build"}, {"web-tests"}),
            (("crates/content/src/simulation/model/request.rs",), {"wasm-build", "wasm-smoke"}, {"web-tests"}),
            (("crates/content/src/simulation/report/game.rs",), {"wasm-build", "wasm-smoke"}, {"web-tests"}),
            (("web/src/App.tsx",), {"web-types", "web-format", "web-tests", "wasm-build"}, {"rust-quality"}),
            (("tools/studio_teams.mts",), {"web-types", "web-format", "web-tests", "wasm-build"}, {"rust-quality"}),
            (("tools/browser_distribution.py",), {"web-tests", "wasm-build", "python-tests"}, {"rust-quality"}),
            (("tools/native_distribution.py",), {"web-tests", "wasm-build", "python-tests"}, {"rust-quality"}),
            (("licenses/pcg32-NOTICE.txt",), {"web-tests", "wasm-build"}, {"rust-quality"}),
            (("presets/studio/s1.nte",), {"native-build"}, {"wasm-build", "web-tests"}),
            (("crates/wasm/js/wire.js",), {"web-tests", "wasm-build", "native-wasm-parity"}, set()),
            (("Cargo.lock",), {"wasm-build", "wasm-smoke"}, {"web-tests"}),
        ]
        for paths, required, excluded in cases:
            with self.subTest(paths=paths):
                selected = self.names(*paths)
                self.assertLessEqual(required, selected)
                self.assertFalse(excluded & selected)
        plan = checkpoint.check_plan(ROOT, ["crates/core/src/lib.rs"])
        packages = next(item for item in plan if item["name"] == "rust-tests")["command"]
        for package in ("ntesim-core", "ntesim-content", "ntesim-research", "ntesim-cli"):
            self.assertIn(package, packages)
        self.assertNotIn("ntesim-wasm", packages)
        self.assertNotIn("wasm-build", {item["name"] for item in plan})

    def test_model_fixture_and_receipt_changes_select_their_consumers(self):
        required = {"rust-tests", "native-build"}
        for path in ("fixtures/energy.json", "research/energy-case.json", "reviews/energy-receipt.json"):
            with self.subTest(path=path):
                selected = self.names(path)
                self.assertLessEqual(required, selected)
                self.assertNotIn("rust-quality", selected)
                self.assertNotIn("rust-format", selected)
                self.assertFalse({"wasm-build", "web-tests", "native-wasm-parity"} & selected)

    def test_current_batch_cache_producer_precedes_its_parity_reader(self):
        plan = checkpoint.check_plan(ROOT, ['web/src/App.tsx'])
        names = [item['name'] for item in plan]
        self.assertLess(names.index('native-build'), names.index('parity-inputs'))
        self.assertLess(names.index('parity-inputs'), names.index('native-wasm-parity'))
        self.assertIn('cache/reviews/q10-research-batch/pilot-request.json',
                      checkpoint.artifact_inputs(ROOT, checkpoint.check('native-wasm-parity', ['python'])))

    def test_milestone_smoke_is_bounded_and_full_browser_receipt_covers_it(self):
        path = "crates/content/src/characters/mint/kit.rs"
        plan = checkpoint.check_plan(ROOT, [path], wasm_smoke=True)
        names = {item["name"] for item in plan}
        self.assertLessEqual({"native-build", "wasm-build", "wasm-smoke"}, names)
        self.assertFalse({"web-types", "web-format", "web-tests", "native-wasm-parity"} & names)
        smoke = next(item for item in plan if item["name"] == "wasm-smoke")
        self.assertEqual(smoke["command"], ["node", "tools/web.mjs", "test",
                                          "tests/wasm_contract.test.mjs", "tests/wasm_loading.test.mjs"])
        packages = next(item for item in plan if item["name"] == "rust-tests")["command"]
        self.assertIn("ntesim-wasm", packages)
        full = [{**item, "exit_code": 0} for item in checkpoint.full_check_plan(ROOT, [path])]
        self.assertTrue(all(any(checkpoint.covers_check(done, item) for done in full) for item in plan))
        failed = next(item for item in full if item["name"] == "web-tests") | {"exit_code": 1}
        self.assertFalse(checkpoint.covers_check(failed, smoke))
        self.assertIn("target/wasm-bindgen/ntesim_wasm_bg.wasm", checkpoint.artifact_inputs(ROOT, smoke))
        native = checkpoint.check("oracle:tools/verify_game_runner_cli.py", ["python"])
        self.assertNotIn("target/wasm-bindgen/ntesim_wasm_bg.wasm", checkpoint.artifact_inputs(ROOT, native))

    def test_smoke_flags_are_available_without_running_a_full_browser_suite(self):
        result = subprocess.run([sys.executable, str(ROOT / "tools/checkpoint.py"), "plan", "--help"],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--wasm-smoke", result.stdout)

    def test_removed_markdown_still_selects_static_validation(self):
        selected = checkpoint.check_plan(ROOT, ["docs/__checkpoint_deleted__.md"])
        static = next(item for item in selected if item["name"] == "local-links-and-formats")
        self.assertEqual(static["command"][-1], "docs/__checkpoint_deleted__.md")

    def test_openspec_metadata_skills_and_wrapper_select_document_checks_only(self):
        for path in ("openspec/config.yaml", "openspec/changes/fix/.openspec.yaml",
                     "openspec/specs/script-inspection/spec.md", "tools/openspec.py",
                     ".agents/skills/openspec-apply-change/SKILL.md"):
            with self.subTest(path=path):
                plan = checkpoint.check_plan(ROOT, [path])
                names = {item["name"] for item in plan}
                self.assertLessEqual({"openspec-validation", "openspec-archive-tasks"}, names)
                self.assertFalse({"rust-tests", "native-build", "wasm-build", "web-tests"} & names)
                for item in plan:
                    if item["name"].startswith("openspec-"):
                        self.assertEqual(item["command"][-2:], ["--concurrency", "4"])

    def test_rust_does_not_run_unrelated_tooling_tests_and_full_plan_covers_scoped_plan(self):
        path = "crates/content/src/characters/mint/kit.rs"
        scoped = checkpoint.check_plan(ROOT, [path])
        self.assertNotIn("python-tests", {item["name"] for item in scoped})
        full = checkpoint.full_check_plan(ROOT, [path])
        completed = [{**item, "exit_code": 0} for item in full]
        self.assertIn("python-tests", {item["name"] for item in full})
        self.assertTrue(all(any(checkpoint.covers_check(value, item) for value in completed) for item in scoped))
        rust = next(item for item in scoped if item["name"] == "rust-tests")
        incomplete = {**rust, "command": rust["command"][:-2], "exit_code": 0}
        self.assertFalse(checkpoint.covers_check(incomplete, rust))

    def test_dependency_scopes_keep_unknown_checks_uncached_and_include_new_paths(self):
        files = {name: {"sha256": name} for name in ("crates/core/src/lib.rs", "web/src/App.tsx",
                 "data/package.json", "crates/research/tests/fixtures/profile-variants.csv", "presets/research/variants.csv", "README.md", "tools/checkpoint.py", "tools/browser_distribution.py", "tools/studio_teams.mts",
                 "tools/native_distribution.py", "tools/studio_build.py", "tools/pages_deploy.py",
                 ".github/workflows/deploy-pages.yml", "LICENSE", "licenses/pcg32-NOTICE.txt")}
        runtime = checkpoint.check_inputs(checkpoint.check("rust-tests", ["python"]), files)
        self.assertIn("crates/core/src/lib.rs", runtime)
        self.assertIn("data/package.json", runtime)
        self.assertIn("crates/research/tests/fixtures/profile-variants.csv", runtime)
        self.assertIn("presets/research/variants.csv", runtime)
        self.assertNotIn("web/src/App.tsx", runtime)
        self.assertNotIn("README.md", runtime)
        browser = checkpoint.check_inputs(checkpoint.check("web-types", ["node"]), files)
        self.assertIn("tools/studio_teams.mts", browser)
        self.assertIn("tools/studio_teams.mts", checkpoint.check_inputs(checkpoint.check("web-tests", ["node"]), files))
        self.assertIn("web/src/App.tsx", browser)
        self.assertIn("tools/studio_build.py", browser)
        self.assertNotIn("crates/core/src/lib.rs", browser)
        execution = checkpoint.check_inputs(checkpoint.check("web-tests", ["node"]), files)
        for path in ("tools/browser_distribution.py", "tools/native_distribution.py", "tools/studio_build.py", "LICENSE",
                     "licenses/pcg32-NOTICE.txt"):
            self.assertIn(path, execution)
        tooling = checkpoint.check_inputs(checkpoint.check("python-tests", ["python"]), files)
        for path in (".github/workflows/deploy-pages.yml", "tools/pages_deploy.py", "tools/studio_build.py"):
            before = checkpoint.check_key(ROOT, checkpoint.check("python-tests", ["python"]), files)
            changed = {**files, path: {"sha256": "changed"}}
            self.assertIn(path, tooling)
            self.assertNotEqual(checkpoint.check_key(ROOT, checkpoint.check("python-tests", ["python"]), changed), before)
        for name in ("extra-0", "custom", "local-links-and-formats"):
            self.assertIsNone(checkpoint.check_inputs(checkpoint.check(name, ["python"]), files))
        native = checkpoint.check_inputs(checkpoint.check("native-build", ["python"]), files)
        self.assertIn("crates/core/src/lib.rs", native)
        self.assertNotIn("web/src/App.tsx", native)
        wasm = checkpoint.check_inputs(checkpoint.check("wasm-build", ["python"]), files)
        self.assertIn("crates/core/src/lib.rs", wasm)
        self.assertNotIn("README.md", wasm)

    def test_granular_package_upstream_closures_and_inputs(self):
        core_closure = checkpoint.crate_upstream_closure(ROOT, "ntesim-core")
        self.assertEqual(core_closure, {"ntesim-contracts", "ntesim-core"})
        content_closure = checkpoint.crate_upstream_closure(ROOT, "ntesim-content")
        self.assertEqual(content_closure, {"ntesim-contracts", "ntesim-core", "ntesim-data", "ntesim-content"})
        self.assertNotIn("ntesim-research", content_closure)
        self.assertNotIn("ntesim-cli", content_closure)
        self.assertNotIn("ntesim-wasm", content_closure)

        files = {
            "crates/contracts/src/lib.rs": {"sha256": "1"},
            "crates/core/src/lib.rs": {"sha256": "2"},
            "crates/data/src/lib.rs": {"sha256": "3"},
            "crates/content/src/lib.rs": {"sha256": "4"},
            "crates/research/src/lib.rs": {"sha256": "5"},
            "crates/cli/src/main.rs": {"sha256": "6"},
            "Cargo.toml": {"sha256": "7"},
        }
        content_inputs = checkpoint.package_test_inputs(ROOT, "ntesim-content", files)
        self.assertIn("crates/content/src/lib.rs", content_inputs)
        self.assertIn("crates/core/src/lib.rs", content_inputs)
        self.assertIn("crates/contracts/src/lib.rs", content_inputs)
        self.assertNotIn("crates/research/src/lib.rs", content_inputs)
        self.assertNotIn("crates/cli/src/main.rs", content_inputs)

    def test_oracle_edits_do_not_invalidate_rust_checks_but_do_invalidate_oracles(self):
        files = {path: {} for path in ("crates/core/src/lib.rs", "tools/verify_zero_kit.py", "tools/cargo.py",
                                      "tools/rust_quality_reports.py", "tools/quality/rust-quality-baseline.json")}
        for name in ("rust-tests", "rust-quality"):
            inputs = checkpoint.check_inputs(checkpoint.check(name, ["python"]), files)
            self.assertNotIn("tools/verify_zero_kit.py", inputs)
            self.assertIn("tools/cargo.py", inputs)
        inputs = checkpoint.check_inputs(checkpoint.check("oracle:tools/verify_zero_kit.py", ["python"]), files)
        self.assertIn("tools/verify_zero_kit.py", inputs)
        browser = checkpoint.check_inputs(checkpoint.check("web-tests", ["node"]), files)
        self.assertNotIn("tools/verify_zero_kit.py", browser)
        self.assertIn("crates/core/src/lib.rs", browser)


if __name__ == "__main__":
    unittest.main()
