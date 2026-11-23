"""Prevent new verifier scripts while preserving independent boundaries.

Duplicate arithmetic repeats an owner's Rust assertions with the same inputs.
Unique checks cover source binding, CLI/package integration or transport parity
outside existing Rust unit tests. Remove duplicates during focused work on
that owner; preserve independent checks. No batch conversion or baseline reset.
"""
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class VerifierPolicyTests(unittest.TestCase):
    def test_frozen_verifier_paths(self):
        baseline = {line for line in (ROOT / 'tools/verifier-paths.txt').read_text().splitlines()
                    if line and not line.startswith('#')}
        actual = {path.relative_to(ROOT).as_posix() for path in (ROOT / 'tools').glob('verify_*.py')}
        self.assertLessEqual(actual, baseline, 'New verifiers need an explicit policy decision')
        self.assertEqual(actual, baseline, 'Prune deleted verifier paths in the same reviewed change')


if __name__ == '__main__':
    unittest.main()
