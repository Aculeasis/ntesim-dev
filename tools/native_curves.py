"""Exact, bounded evaluation of serialized native curves, without unit inference."""
from fractions import Fraction


def fraction(value):
    value = Fraction(value)
    return {"numerator": value.numerator, "denominator": value.denominator}


def curve_at(curve, coordinate):
    mode = curve["InterpMode"].split("::")[-1]
    if mode not in ("RCIM_Linear", "RCIM_Constant"):
        raise ValueError(f"unsupported curve interpolation: {mode}")
    keys = [(Fraction(k["Time"]), Fraction(k["Value"])) for k in curve["Keys"]]
    if not keys or any(a[0] >= b[0] for a, b in zip(keys, keys[1:])):
        raise ValueError("curve keys must be nonempty and strictly increasing")
    x = Fraction(coordinate)
    if x < keys[0][0] or x > keys[-1][0]:
        side = "PreInfinityExtrap" if x < keys[0][0] else "PostInfinityExtrap"
        if curve.get(side, "").split("::")[-1] != "RCCE_Constant":
            raise ValueError("unsupported curve extrapolation")
        return keys[0][1] if x < keys[0][0] else keys[-1][1]
    for time, value in keys:
        if x == time:
            return value
    for (ta, va), (tb, vb) in zip(keys, keys[1:]):
        if ta < x < tb:
            return va if mode == "RCIM_Constant" else va + (vb - va) * (x - ta) / (tb - ta)
    raise ValueError("unresolved curve coordinate")
