"""Measurement integrity and per-process resource accounting, not combat tests."""
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import native_benchmark as bench
from verification_workspace import temporary_directory


class NativeBenchmarkTests(unittest.TestCase):
    def test_missing_stale_failed_and_nonfinite_measurements_cannot_pass(self):
        profile = {"schema_version": 1, "engine_source_sha256": "engine", "operation_success": True,
                   "command_ms": 1, "serialization_ms": 2, "serialized_report_bytes": 42,
                   "events_processed": 3}
        usage = {"exit_code": 0, "wall_seconds": 0.1, "cpu_seconds": 0.05, "peak_resident_bytes": 4096}
        self.assertEqual(bench.validate_sample(profile, usage, 42, "engine", 0), [])
        for change in ({"engine_source_sha256": "old"}, {"operation_success": False},
                       {"serialization_ms": float("nan")}, {"events_processed": None},
                       {"serialized_report_bytes": 41}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                bench.validate_sample({**profile, **change}, usage, 42, "engine", 0)
        with self.assertRaises(ValueError):
            bench.validate_sample(profile, {**usage, "exit_code": 2}, 42, "engine", 0)
        slow = {**usage, "cpu_seconds": bench.BUDGET["cpu_seconds"] + 1}
        self.assertEqual(bench.validate_sample(profile, slow, 42, "engine", 0), ["cpu_seconds"])
        # A cancellation sample is explicitly incomplete, not a successful run.
        self.assertEqual(bench.validate_sample({**profile, "operation_success": False},
                         {**usage, "exit_code": 2}, 42, "engine", 2), [])

    def test_resource_measurement_uses_each_child_and_retains_output(self):
        with temporary_directory(prefix="native-benchmark-resource-") as directory:
            directory = Path(directory)
            output, errors = directory / "stdout", directory / "stderr"
            result = bench.collect_process([sys.executable, "-c", "print('measured')"], output, errors)
            self.assertEqual(result["exit_code"], 0)
            self.assertGreater(result["peak_resident_bytes"], 0)
            self.assertGreater(result["wall_seconds"], 0)
            self.assertGreaterEqual(result["cpu_seconds"], 0)
            self.assertEqual(output.read_text().strip(), "measured")

    def test_windows_timeout_kills_and_reaps_without_claiming_a_sample(self):
        process = unittest.mock.Mock()
        process.wait.side_effect = [subprocess.TimeoutExpired("native", 1), None]
        with patch.object(bench.os, "name", "nt"), self.assertRaises(RuntimeError):
            bench.wait_usage(process, 1)
        process.kill.assert_called_once()
        self.assertEqual(process.wait.call_count, 2)


if __name__ == "__main__":
    unittest.main()
