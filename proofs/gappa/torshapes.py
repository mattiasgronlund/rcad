"""Which case of torsplit.py a shape falls in: on_floats' doubles, worked in
Python's, which are IEEE doubles rounded to nearest even as Rust's are. The
direction is the nearest double to each component over the exact length,
which is what Axis::new gives above 2^-1012. Prints, for each shape, the
from_centre and direction components that are zero, each cross product
component's letter, hollow's and P2's, and whether on_floats' checks pass.
Run as: python3 shapes.py"""
from decimal import Decimal, getcontext

getcontext().prec = 60
NORMAL = 2.0 ** -1022


def unit(v):
    length = sum(Decimal(c) * Decimal(c) for c in v).sqrt()
    return [float(Decimal(c) / length) for c in v]


def normal(a, b):
    return a == 0.0 or b == 0.0 or abs(a * b) >= NORMAL


def case(name, origin, direction, ring, tube, point):
    d = unit(direction)
    f = [point[i] - origin[i] for i in range(3)]
    halves = [(f[1] * d[2], f[2] * d[1]), (f[2] * d[0], f[0] * d[2]), (f[0] * d[1], f[1] * d[0])]
    q = [h - k for h, k in halves]
    checks = all(normal(f[i], f[i]) and normal(d[i], d[i]) and normal(q[i], q[i]) for i in range(3))
    checks = checks and all(normal(f[(i + 1) % 3], d[(i + 2) % 3]) and normal(f[(i + 2) % 3], d[(i + 1) % 3])
                            for i in range(3))
    checks = checks and normal(ring, ring) and normal(tube, tube)
    oab = f[0] * f[0] + f[1] * f[1] + f[2] * f[2]
    hollow = ring * ring - tube * tube
    bulge = oab + hollow
    w = d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
    across = q[0] * q[0] + q[1] * q[1] + q[2] * q[2]
    f4 = 4.0 * (ring * ring)
    checks = checks and normal(bulge, bulge) and normal(w, bulge * bulge) and normal(f4, across)
    letters = ""
    spread = 0.0
    for (h, k), qa in zip(halves, q):
        v = (abs(h) + abs(k)) * (abs(h) + abs(k))
        spread += v
        letters += "n" if qa != 0.0 else "c" if v >= NORMAL else "s" if v > 0.0 else "u"
    letters += "n" if abs(hollow) >= NORMAL else "e"
    letters += "n" if f4 * spread >= NORMAL else "b"
    zeros = [f"f{'xyz'[i]}" for i in range(3) if f[i] == 0.0] + [f"d{'xyz'[i]}" for i in range(3) if d[i] == 0.0]
    print(f"{name}: direction {d}, from_centre {f}, halves {halves}")
    print(f"  zero factors {zeros or 'none'}; case {letters}; on_floats' checks pass: {checks}")


case("on the axis (1, 0, 0)", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0, 1.0, [2.0, 0.0, 0.0])
case("on the axis (1, 1, 1)", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2.0, 1.0, [1.0, 1.0, 1.0])
case("off the axis (1, 1, 1), one component cancelling", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2.0, 1.0,
     [1.0, 1.0, 2.0])
