"""Write the claims on value, the double on_floats returns, rather than y, the
sum before its last rounding: planev.g from plane.g, which proves, and
c_o2_value.g from o2.g, which fails at the code's B and so is a control of
o2.g. 464.8 3 and 1 describe both and quote neither; this makes the changes
they name, as 464.5 makes them to block 1. 464.8 1 calls the second o2v.g.

Run as: python3 make_value.py, beside o2.g and plane.g.
"""

PLANE_HEADER = (
    "# SideOfAFlatSurface::on_floats and SIDE_OF_PLANE_ERROR_BOUND,\n"
    "# crates/rcad-geom/src/plane.rs at affe0f01.\n"
)
O2_HEADER = (
    "# Orient2d::on_floats and ORIENT2D_ERROR_BOUND,\n"
    "# crates/rcad-predicates/src/orient2d.rs at affe0f01.\n"
)


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def planev(src):
    src = sub(src, PLANE_HEADER, "# plane.g with the claim taken on value rather than y: 464.8 3's planev.g.\n" + PLANE_HEADER)
    src = sub(
        src,
        "# The three terms' part of the permanent, with no rounding.\n"
        "c0 = |t0|; c1 = |t1|; c2 = |t2|;\n"
        "P3 = c0 + c1 + c2;\n",
        "# Each term's part of the permanent, with no rounding.\n"
        "c0 = |t0|; c1 = |t1|; c2 = |t2|;\n"
        "c3 = |off|;\n",
    )
    src = sub(src, "rs01 = s01 / S01; rs012 = s012 / S012;\n", "rs01 = s01 / S01; rs012 = s012 / S012;\nrv = value / y;\n")
    src = sub(
        src,
        "# Each of the three terms, as y carries it, less the term itself. The\n"
        "# offset reaches y with no rounding.\n"
        "W0 = rs01 * rs012 - 1 / rp0;\n"
        "W1 = rs01 * rs012 - 1 / rp1;\n"
        "W2 = rs012 - 1 / rp2;\n"
        "e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2;\n",
        "# Each of the four terms, as value carries it, less the term itself. The\n"
        "# offset reaches value through its last rounding alone.\n"
        "W0 = rv * rs01 * rs012 - 1 / rp0;\n"
        "W1 = rv * rs01 * rs012 - 1 / rp1;\n"
        "W2 = rv * rs012 - 1 / rp2;\n"
        "W3 = rv - 1;\n"
        "e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2; e3 = -(off * W3);\n",
    )
    src = sub(
        src,
        "  /\\ |S01| >= 1b-1074 /\\ |S012| >= 1b-1074\n",
        "  /\\ |S01| >= 1b-1074 /\\ |S012| >= 1b-1074\n"
        "  # Neither the offset nor y is zero.\n"
        "  /\\ |off| >= 1b-1074 /\\ |y| >= 1b-1074\n",
    )
    src = sub(
        src,
        "  /\\ t0 / |t0| in [-1, 1] /\\ t1 / |t1| in [-1, 1] /\\ t2 / |t2| in [-1, 1]\n",
        "  /\\ t0 / |t0| in [-1, 1] /\\ t1 / |t1| in [-1, 1] /\\ t2 / |t2| in [-1, 1]\n"
        "  /\\ off / |off| in [-1, 1]\n",
    )
    src = sub(
        src,
        "  (y - Ex) / P3 in ? /\\ P -/ Phat in ?\n  /\\ (y - Ex) / eb in [-1, 1]\n",
        "  (value - Ex) / Phat in ? /\\ P -/ Phat in ?\n  /\\ (value - Ex) / eb in [-1, 1]\n",
    )
    src = sub(
        src,
        "s012 / S012 -> 1 + (s012 - S012) / S012;\n",
        "s012 / S012 -> 1 + (s012 - S012) / S012;\nvalue / y -> 1 + (value - y) / y;\n",
    )
    src = sub(
        src,
        "# The error is the sum of the three terms' errors, each a fraction of its\n"
        "# own piece of the permanent; add_rr then averages them.\n"
        "(y - Ex) / P3 -> ((P3 + (y - Ex)) - P3) / P3;\n"
        "P3 + (y - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2);\n",
        "# The error is the sum of the four terms' errors, each a fraction of its\n"
        "# own piece of the permanent; add_rr then averages them.\n"
        "(value - Ex) / Phat -> ((Phat + (value - Ex)) - Phat) / Phat;\n"
        "Phat + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3);\n",
    )
    src = sub(src, "(c2 + e2) - c2 -> e2;\n", "(c2 + e2) - c2 -> e2;\n(c3 + e3) - c3 -> e3;\n")
    src = sub(src, "e2 / c2 -> (t2 / |t2|) * W2;\n", "e2 / c2 -> (t2 / |t2|) * W2;\ne3 / c3 -> -((off / |off|) * W3);\n")
    src = sub(
        src,
        "# From the three terms to the whole permanent, which the offset only\n"
        "# makes larger, to the one on_floats rounds, and on to eb.\n"
        "(y - Ex) / eb -> ((y - Ex) / P3) * (P3 / Phat) * (Phat / P) * (P / eb);\n"
        "P3 / Phat -> 1 / (1 + |off| / P3);\n",
        "# From the permanent to the one on_floats rounds, and on to eb.\n"
        "(value - Ex) / eb -> ((value - Ex) / Phat) * (Phat / P) * (P / eb);\n",
    )
    assert "(y - Ex)" not in src and "P3" not in src
    return src


def o2_value(src):
    src = sub(
        src,
        O2_HEADER,
        "# o2.g with the claim taken on value rather than y: 464.8 1's o2v.g. It\n"
        "# fails at the code's B, so it is a control of o2.g.\n" + O2_HEADER,
    )
    src = sub(
        src,
        "rp1 = forwards / (acx * bcy); rp2 = backwards / (acy * bcx);\n",
        "rp1 = forwards / (acx * bcy); rp2 = backwards / (acy * bcx);\nrv = value / y;\n",
    )
    src = sub(
        src,
        "# Each of the two products, as y carries it, less the product of the\n"
        "# exact differences.\n"
        "W1 = 1 - 1 / (racx * rbcy * rp1);\n"
        "W2 = 1 - 1 / (racy * rbcx * rp2);\n",
        "# Each of the two products, as value carries it, less the product of the\n"
        "# exact differences.\n"
        "W1 = rv - 1 / (racx * rbcy * rp1);\n"
        "W2 = rv - 1 / (racy * rbcx * rp2);\n",
    )
    src = sub(
        src,
        "  /\\ |bcx| >= 1b-1074 /\\ |bcy| >= 1b-1074\n",
        "  /\\ |bcx| >= 1b-1074 /\\ |bcy| >= 1b-1074\n  # y is not zero.\n  /\\ |y| >= 1b-1074\n",
    )
    src = sub(
        src,
        "backwards / (acy * bcx) -> 1 + (backwards - acy * bcx) / (acy * bcx);\n",
        "backwards / (acy * bcx) -> 1 + (backwards - acy * bcx) / (acy * bcx);\nvalue / y -> 1 + (value - y) / y;\n",
    )
    src = src.replace("(y - Ex)", "(value - Ex)")
    return src


open("planev.g", "w").write(planev(open("plane.g").read()))
open("c_o2_value.g", "w").write(o2_value(open("o2.g").read()))
