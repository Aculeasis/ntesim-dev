from decimal import Decimal
from fractions import Fraction
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import native_curves as reviewer



class MintCurveReviewTests(unittest.TestCase):
    def test_decimal_interpolation_retains_exact_fraction(self):
        curve = {"InterpMode": "RCIM_Linear", "Keys": [
            {"Time": 1, "Value": Decimal("0.1")}, {"Time": 4, "Value": Decimal("0.2")} ]}
        self.assertEqual(reviewer.curve_at(curve, 2), Fraction(2, 15))

    def test_constant_endpoint_and_extrapolation(self):
        curve = {"InterpMode": "RCIM_Constant", "Keys": [
            {"Time": 1, "Value": 6}, {"Time": 4, "Value": 9}],
            "PreInfinityExtrap": "RCCE_Constant", "PostInfinityExtrap": "RCCE_Constant"}
        self.assertEqual(reviewer.curve_at(curve, 3), 6)
        self.assertEqual(reviewer.curve_at(curve, 4), 9)
        self.assertEqual(reviewer.curve_at(curve, 8), 9)
        self.assertEqual(reviewer.curve_at(curve, 0), 6)
        curve["PostInfinityExtrap"] = "RCCE_Linear"
        with self.assertRaisesRegex(ValueError, "extrapolation"):
            reviewer.curve_at(curve, 8)

    def test_invalid_curve_evidence_rejected(self):
        for mode, keys in (("RCIM_Cubic", [{"Time": 1, "Value": 0}]),
                           ("RCIM_Linear", []), ("RCIM_Linear", [
                               {"Time": 1, "Value": 0}, {"Time": 1, "Value": 1}])):
            with self.assertRaises(ValueError):
                reviewer.curve_at({"InterpMode": mode, "Keys": keys}, 1)


if __name__ == "__main__":
    unittest.main()
