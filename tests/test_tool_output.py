"""Exercise quiet runners with real noisy, failing and nested subprocesses."""
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from uuid import uuid4

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from verification_workspace import temporary_directory


class ToolOutputTests(unittest.TestCase):
    def test_explicit_environment_file_is_literal_and_inherited_values_win(self):
        with temporary_directory(prefix="environment space-") as directory:
            folder = Path(directory)
            env_file = folder / ".env"
            output = folder / "observed.json"
            literal = "$(not-executed); ${OTHER} & `literal` # kept"
            env_file.write_text(f"# local data\nNTESIM_FIXTURE_LITERAL='{literal}'\n"
                                + ("ntesim_fixture_override" if os.name == "nt" else "NTESIM_FIXTURE_OVERRIDE")
                                + "=file\nNTESIM_FIXTURE_EMPTY=\n", encoding="utf-8")
            child = "import json,os,sys; from pathlib import Path; " \
                    "Path(sys.argv[1]).write_text(json.dumps([os.getenv(k) for k in sys.argv[2:]]))"
            env = {key: value for key, value in os.environ.items() if not key.startswith("NTESIM_FIXTURE_")}
            env["NTESIM_FIXTURE_OVERRIDE"] = "inherited"
            env["NTESIM_TOOL_VERBOSE"] = "0"
            command = [sys.executable, str(ROOT / "tools/check.py")]
            arguments = [sys.executable, "-c", child, str(output), "NTESIM_FIXTURE_LITERAL",
                         "NTESIM_FIXTURE_OVERRIDE", "NTESIM_FIXTURE_EMPTY"]
            result = subprocess.run([*command, "--env-file", str(env_file), "--", *arguments],
                                    env=env, cwd=folder, capture_output=True, text=True)
            self.assertEqual((result.returncode, result.stdout, result.stderr), (0, "", ""))
            self.assertEqual(json.loads(output.read_text()), [literal, "inherited", ""])
            result = subprocess.run([*command, "--", *arguments], env=env, cwd=folder,
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(output.read_text()), [None, "inherited", None])

            for invalid in ("export PRIVATE=do-not-print", "PRIVATE='do-not-print", "PRIVATE=one\nPRIVATE=do-not-print"):
                output.unlink(missing_ok=True)
                env_file.write_text(invalid, encoding="utf-8")
                result = subprocess.run([*command, "--env-file", str(env_file), "--", *arguments],
                                        env=env, capture_output=True, text=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("environment file line", result.stderr)
                self.assertNotIn("do-not-print", result.stderr)
                self.assertFalse(output.exists())

    def setUp(self):
        self.workspace = temporary_directory(prefix="tool-output-")
        self.directory = Path(self.workspace.__enter__())
        self.addCleanup(self.workspace.__exit__, None, None, None)
        self.label = "output-test-" + uuid4().hex
        self.addCleanup(self.remove_logs)
        self.child = self.directory / "noisy child.py"
        self.child.write_text(
            "import json, os, sys\n"
            "sys.stdout.reconfigure(encoding='utf-8'); sys.stderr.reconfigure(encoding='utf-8')\n"
            "mode = sys.argv[1]\n"
            "print(json.dumps(sys.argv[2:]), flush=True)\n"
            "print('early stderr', file=sys.stderr, flush=True)\n"
            "if mode == 'fail': print('AssertionError: independent sentinel', flush=True)\n"
            "print('Independent arithmetic passed', flush=True)\n"
            "print('ℹ pass 25', flush=True)\n"
            "for i in range(5000): print('PASS case', i)\n"
            "print('last output', flush=True)\n"
            "sys.exit(2 if mode == 'fail' else 0)\n", encoding="utf-8")
        python_launcher = self.directory / "launcher.py"
        python_launcher.write_text(
            f"import sys; sys.path.insert(0, {str(ROOT / 'tools')!r})\n"
            "from tool_output import run_logged\n"
            f"sys.exit(run_logged(sys.argv[2:], label={self.label!r}, verbose=sys.argv[1]=='verbose'))\n",
            encoding="utf-8")
        node_launcher = self.directory / "launcher.mjs"
        node_launcher.write_text(
            f"import {{ runLogged }} from {json.dumps((ROOT / 'tools/tool_output.mjs').as_uri())};\n"
            f"process.exit(runLogged(process.argv[3], process.argv.slice(4), {{label: {json.dumps(self.label)}, verbose: process.argv[2]==='verbose'}}));\n",
            encoding="utf-8")
        self.runners = [[sys.executable, str(python_launcher)], ["node", str(node_launcher)]]

    def remove_logs(self):
        for log in (ROOT / "cache/tool-logs").glob(self.label + "-*.log"):
            log.unlink()

    def run_fixture(self, runner, mode, verbose=False):
        return subprocess.run([*runner, "verbose" if verbose else "quiet",
                               sys.executable, str(self.child), mode, "spaces and кириллица", "$(literal)"],
                              capture_output=True, text=True, encoding="utf-8", cwd=ROOT)

    def test_success_is_silent_and_complete_log_keeps_both_streams_and_arguments(self):
        for runner in self.runners:
            with self.subTest(runner=runner[0]):
                result = self.run_fixture(runner, "pass")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout + result.stderr, "")
        logs = list((ROOT / "cache/tool-logs").glob(self.label + "-*.log"))
        self.assertEqual(len(logs), 2)
        for log in logs:
            text = log.read_text(encoding="utf-8")
            self.assertIn("early stderr", text)
            self.assertIn("PASS case 4999", text)
            self.assertIn("last output", text)
            self.assertIn(json.dumps(["spaces and кириллица", "$(literal)"]), text)

    def test_failure_keeps_exit_two_and_early_diagnostic_without_passing_noise(self):
        for runner in self.runners:
            with self.subTest(runner=runner[0]):
                result = self.run_fixture(runner, "fail")
                self.assertEqual(result.returncode, 2)
                self.assertEqual(result.stdout, "")
                self.assertIn("AssertionError: independent sentinel", result.stderr)
                self.assertIn("log:", result.stderr)
                self.assertNotIn("PASS case", result.stderr)
                self.assertNotIn("Independent arithmetic passed", result.stderr)
                self.assertNotIn("ℹ pass 25", result.stderr)
                self.assertLessEqual(len(result.stderr.splitlines()), 61)
                self.assertLess(len(result.stderr.encode("utf-8")), 12500)

    def test_verbose_streams_full_success_output(self):
        for runner in self.runners:
            with self.subTest(runner=runner[0]):
                result = self.run_fixture(runner, "pass", verbose=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("PASS case 4999", result.stdout)
                self.assertIn("early stderr", result.stderr)
                failed = self.run_fixture(runner, "fail", verbose=True)
                self.assertEqual(failed.returncode, 2)
                self.assertIn("AssertionError: independent sentinel", failed.stdout)
                self.assertIn("PASS case 4999", failed.stdout)

    def test_large_single_line_and_launch_error_are_bounded(self):
        huge = self.directory / "huge.py"
        huge.write_text("import sys; print('AssertionError: ' + 'X' * 2000000); sys.exit(1)", encoding="utf-8")
        for runner in self.runners:
            with self.subTest(runner=runner[0]):
                result = subprocess.run([*runner, "quiet", sys.executable, str(huge)],
                                        capture_output=True, cwd=ROOT)
                self.assertEqual(result.returncode, 1)
                self.assertIn(b"AssertionError", result.stderr)
                self.assertLess(len(result.stderr), 12500)
                missing = subprocess.run([*runner, "quiet", str(self.directory / "missing-command")],
                                         capture_output=True, cwd=ROOT)
                self.assertEqual(missing.returncode, 1)
                self.assertIn(b"log:", missing.stderr)

    def test_unrecognized_failure_uses_bounded_tail(self):
        fixture = self.directory / "unknown.py"
        fixture.write_text("import sys; print('opaque output\\n' * 1000); sys.exit(3)", encoding="utf-8")
        for runner in self.runners:
            with self.subTest(runner=runner[0]):
                result = subprocess.run([*runner, "quiet", sys.executable, str(fixture)],
                                        capture_output=True, text=True, cwd=ROOT)
                self.assertEqual(result.returncode, 3)
                self.assertIn("opaque output", result.stderr)
                self.assertLessEqual(len(result.stderr.splitlines()), 61)

    def test_check_cli_verbose_and_missing_command(self):
        check = [sys.executable, str(ROOT / "tools/check.py")]
        env = {**os.environ, "NTESIM_TOOL_VERBOSE": "0"}
        quiet = subprocess.run([*check, "--", sys.executable, "-c", "print('hidden')"], capture_output=True, env=env)
        self.assertEqual((quiet.returncode, quiet.stdout, quiet.stderr), (0, b"", b""))
        verbose = subprocess.run([*check, "--verbose", "--", "python", "-c", "print('visible')"], capture_output=True)
        self.assertEqual(verbose.returncode, 0)
        self.assertIn(b"visible", verbose.stdout)
        missing = subprocess.run(check, capture_output=True)
        self.assertEqual(missing.returncode, 2)
        self.assertIn(b"a check command is required", missing.stderr)

    def test_nested_check_keeps_success_details_in_outer_log(self):
        for runner in self.runners:
            with self.subTest(runner=runner[0]):
                result = subprocess.run([*runner, "quiet", sys.executable, str(ROOT / "tools/check.py"),
                                         "--", sys.executable, str(self.child), "pass"], capture_output=True)
                self.assertEqual((result.returncode, result.stdout, result.stderr), (0, b"", b""))
        logs = list((ROOT / "cache/tool-logs").glob(self.label + "-*.log"))
        self.assertEqual(len(logs), 2)
        for log in logs:
            self.assertIn("PASS case 4999", log.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
