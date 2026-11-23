import ast
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import verification_workspace as workspace
import verification_engine as engine


class VerificationReceiptTests(unittest.TestCase):
    def test_parity_input_producer_recreates_cache_without_refreshing_legacy(self):
        with workspace.temporary_directory(prefix='parity-inputs-') as directory, \
             patch.object(workspace, 'ROOT', Path(directory)), \
             patch.object(engine, 'ROOT', Path(directory)), \
             patch.object(engine, 'write_review_receipt', workspace.write_review_receipt), \
             redirect_stdout(io.StringIO()):
            root = Path(directory)
            (root / 'examples').mkdir()
            (root / 'examples/research-crit.json').write_text('{"seed":42}')
            (root / 'examples/research-game-batch.json').write_text(
                '{"seeds":[42],"max_total_events":1,"max_run_receipt_bytes":1,"schema_version":1}')
            legacy = root / 'reviews/q10-research-batch/pilot-request.json'
            legacy.parent.mkdir(parents=True)
            legacy.write_bytes(b'{"legacy":true}')
            for _ in range(2):
                engine.prepare_batch_parity_inputs()
                cached = root / 'cache/reviews/q10-research-batch/pilot-request.json'
                value = json.loads(cached.read_bytes())
                self.assertEqual(value, dict(schema_version=1, seeds=list(range(1000)),
                                             max_total_events=300000, max_run_receipt_bytes=8 * 1024 * 1024))
                self.assertEqual(json.loads(cached.with_name('scenario-request.json').read_bytes()), {'seed':42})
                self.assertEqual(legacy.read_bytes(), b'{"legacy":true}')
                cached.unlink()

    def test_default_keeps_existing_bytes_and_does_not_create_missing_receipt(self):
        with workspace.temporary_directory(prefix="receipt-") as directory, \
             patch.object(workspace, "ROOT", Path(directory)), redirect_stdout(io.StringIO()):
            path = Path(directory) / "reviews/oracle.json"
            self.assertFalse(workspace.write_review_receipt(path, {"new": True}))
            self.assertFalse(path.parent.exists())
            path.parent.mkdir()
            path.write_bytes(b'{"historical":true}\r\n')
            self.assertFalse(workspace.write_review_receipt(path, {"new": True}))
            self.assertEqual(path.read_bytes(), b'{"historical":true}\r\n')
            self.assertTrue(workspace.write_review_receipt(path, {"new": True}, refresh=True))
            cached = Path(directory) / 'cache/reviews/oracle.json'
            self.assertEqual(json.loads(cached.read_text()), {"new": True})
            self.assertEqual(path.read_bytes(), b'{"historical":true}\r\n')
            self.assertTrue(workspace.write_review_receipt(cached, {"new": 2}, refresh=True))
            self.assertEqual(json.loads(cached.read_text()), {"new": 2})
            with self.assertRaises(ValueError):
                workspace.write_review_receipt(Path(directory) / "outside.json", {}, refresh=True)

    def test_receipt_paths_reject_traversal_siblings_and_redirected_ancestors(self):
        with workspace.temporary_directory(prefix='receipt-paths-') as directory, \
             patch.object(workspace, 'ROOT', Path(directory)):
            root = Path(directory)
            for path in ('reviews/../escape.json', 'cache/reviews/../escape.json', '.cache/reviews/../escape.json',
                         'reviews-sibling/out.json', 'cache/reviews-sibling/out.json', '.cache/reviews-sibling/out.json',
                         'reviews', root.parent / 'reviews/out.json'):
                with self.subTest(path=path), self.assertRaises(ValueError):
                    workspace.write_review_receipt(path, {}, refresh=True)
            original = Path.is_symlink
            for redirected in (root / 'cache', root / 'cache/reviews', root / 'reviews'):
                with self.subTest(redirected=redirected), \
                     patch.object(Path, 'is_symlink', lambda p: p == redirected or original(p)), \
                     self.assertRaises(ValueError):
                    workspace.review_receipt_path(root / 'reviews/out.json')
            with patch.object(Path, 'is_junction', lambda p: p == root / 'cache', create=True), \
                 self.assertRaises(ValueError):
                workspace.review_receipt_path('reviews/out.json')

    def test_all_kit_oracles_parse_and_route_review_writes_through_explicit_refresh(self):
        for path in (ROOT / "tools").glob("verify_*_kit.py"):
            with self.subTest(path=path.name):
                text = path.read_text(encoding="utf-8")
                tree = ast.parse(text)
                if "write_review_receipt" not in text:
                    continue  # Some oracles produce only temporary scenario files.
                calls = [node for node in ast.walk(tree) if isinstance(node, ast.Call)
                         and isinstance(node.func, ast.Name) and node.func.id == "write_review_receipt"]
                self.assertEqual(len(calls), 1)
                self.assertTrue(any(keyword.arg == "refresh" and isinstance(keyword.value, ast.Attribute)
                                    and keyword.value.attr == "refresh_receipt" for keyword in calls[0].keywords))
                self.assertIn('"--refresh-receipt"', text)


if __name__ == "__main__":
    unittest.main()
