import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import cpu_limit


class CpuLimitTests(unittest.TestCase):
    def test_only_opted_in_github_build_preserves_all_runner_cpus(self):
        for env, enabled in (({"GITHUB_ACTIONS": "true", "NTESIM_BUILD_CPUS": "all"}, True),
                             ({"GITHUB_ACTIONS": "true"}, False),
                             ({"NTESIM_BUILD_CPUS": "all"}, False), ({}, False)):
            with self.subTest(env=env), patch.dict(os.environ, {**env, "CARGO_BUILD_JOBS": "2"}, clear=True), \
                 patch.object(cpu_limit.sys, "platform", "linux"), \
                 patch.object(cpu_limit.os, "sched_getaffinity", return_value=set(range(12)), create=True), \
                 patch.object(cpu_limit.os, "sched_setaffinity", create=True) as affinity:
                self.assertEqual(cpu_limit.limit_cpus(), list(range(12 if enabled else 4)))
                self.assertEqual(os.environ["CARGO_BUILD_JOBS"], "2")
                if enabled:
                    affinity.assert_not_called()
                    self.assertNotIn("RAYON_NUM_THREADS", os.environ)
                    self.assertNotIn("OMP_NUM_THREADS", os.environ)
                    self.assertNotIn("RUST_TEST_THREADS", os.environ)
                else:
                    affinity.assert_called_once_with(0, list(range(4)))
                    self.assertEqual(os.environ["RAYON_NUM_THREADS"], "4")

    def test_environment_caps_preserve_smaller_limits(self):
        for value, expected in (("1", "1"), ("4", "4"), ("64", "4"), ("-1", "4"), ("invalid", "4")):
            with self.subTest(value=value), patch.dict(os.environ, {"RUST_TEST_THREADS": value}, clear=True):
                cpu_limit.limit_cpus()
                self.assertEqual(os.environ["RUST_TEST_THREADS"], expected)
                for key in ("CARGO_BUILD_JOBS", "RAYON_NUM_THREADS", "OMP_NUM_THREADS"):
                    self.assertEqual(os.environ[key], "4")

    def test_linux_affinity_uses_only_cpus_already_allowed(self):
        for available in ({2, 5}, {2, 5, 8, 9, 10, 15}):
            with self.subTest(available=available), \
                 patch.object(cpu_limit.sys, "platform", "linux"), \
                 patch.object(cpu_limit.os, "sched_getaffinity", return_value=available, create=True), \
                 patch.object(cpu_limit.os, "sched_setaffinity", create=True) as set_affinity:
                cpu_limit.limit_cpus()
                set_affinity.assert_called_once_with(0, sorted(available)[:4])

    @unittest.skipUnless(sys.platform == "win32" or hasattr(os, "sched_getaffinity"), "CPU affinity unavailable")
    def test_real_child_and_grandchild_inherit_affinity_and_exit_status(self):
        source = """
import ctypes, json, os, subprocess, sys
if sys.platform == 'win32':
    from ctypes import wintypes
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    kernel.GetProcessAffinityMask.argtypes = [wintypes.HANDLE, ctypes.POINTER(ctypes.c_size_t), ctypes.POINTER(ctypes.c_size_t)]
    mask, system = ctypes.c_size_t(), ctypes.c_size_t()
    assert kernel.GetProcessAffinityMask(kernel.GetCurrentProcess(), ctypes.byref(mask), ctypes.byref(system))
    cpus = [bit for bit in range(mask.value.bit_length()) if mask.value & (1 << bit)]
else:
    cpus = sorted(os.sched_getaffinity(0))
print(json.dumps({'cpus': cpus, 'threads': os.environ['RUST_TEST_THREADS']}), flush=True)
if len(sys.argv) == 2:
    subprocess.run([sys.executable, '-c', sys.argv[1], sys.argv[1], 'child'], check=True)
sys.exit(2 if len(sys.argv) == 2 else 0)
"""
        result = subprocess.run(
            [sys.executable, str(ROOT / "tools/cpu_limit.py"), "--", sys.executable, "-c", source, source],
            env={**os.environ, "RUST_TEST_THREADS": "64"}, capture_output=True, text=True)
        self.assertEqual(result.returncode, 2, result.stderr)
        parent, child = map(json.loads, result.stdout.splitlines())
        self.assertEqual(parent, child)
        self.assertGreater(len(parent["cpus"]), 0)
        self.assertLessEqual(len(parent["cpus"]), 4)
        self.assertEqual(parent["threads"], "4")


if __name__ == "__main__":
    unittest.main()
