"""Write signed.g's and shiftplane.g's cases with one product below the
normals, as 464.8's planecases.py did for plane.g: signeds<k><sign>.g and
shifts<k><sign>.g for product k in 0, 1, 2 and its sign, "" positive or "n"
negative, the other two products normal, and what rounding product k cost
carried as an amount a<k>, not a ratio.

That amount is at most 2^-1075 times what rounds after it, and it is spent
out of another product's share of the bound, which that product being normal
keeps at least 2^-1022 / (1 + u): product 2's for k 0 or 1, since product 2
meets one rounding fewer than the bound gives it; for k 2, half each out of
products 0 and 1, whose shares are then 9/8 of |n_i| L, with product 2's,
which then meets two roundings, 1/2.

Run as: python3 signedcases.py; it writes beside itself."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
# What each product meets after its own rounding: two sums and value for
# products 0 and 1, one sum and value for product 2.
AFTER = {0: "rv * rs012 * rs01", 1: "rv * rs012 * rs01", 2: "rv * rs012"}

SIGNED_RESTATES = [
    "crates/rcad-geom/src/plane.rs Plane::signed_distance_to",
    "crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF",
    "crates/rcad-geom/src/point.rs Vector::dot",
    "Cargo.lock nalgebra",
]
SHIFT_RESTATES = [
    "crates/rcad-geom/src/plane.rs Plane::transformed",
    "crates/rcad-geom/src/plane.rs Plane::holding",
    "crates/rcad-geom/src/half_space.rs HalfSpace::shifted",
    "crates/rcad-geom/src/moved_corner.rs SURFACE_AFTER_SHIFTING_ROUNDOFF",
    "crates/rcad-geom/src/isometry.rs Isometry::apply_to_vector",
    "crates/rcad-geom/src/isometry.rs Isometry::translation_part",
    "crates/rcad-geom/src/isometry.rs turned_by",
    "crates/rcad-geom/src/isometry.rs TurnOfACube::named_by",
    "crates/rcad-geom/src/isometry.rs TurnOfACube::turned",
    "crates/rcad-geom/src/point.rs Vector::dot",
    "Cargo.lock nalgebra",
]

SIGNED = {
    "restates": SIGNED_RESTATES,
    "name": "signeds",
    "about": "signed.g",
    "q": "p",
    "check": "signed_distance_to does not check",
    "y": "y = s012 - off;",
    "e3": "e3 = -(off * W3);",
    "e3_hint": "e3 / c3 -> -(4 * (off / L) * W3);",
    "size": "L",
    "c3": "c3 = 1b-2 * L;",
    "off_bound": "off / L in [-1, 1]",
    "sum": "|n0| + |n1| + |n2| in [0, 1774b-10]",
    "sum_why": [
        "  # A stored normal has n.n within NORMAL_LENGTH_ROUNDOFF = 64 epsilon of",
        "  # one, so |n0| + |n1| + |n2| <= sqrt(3 (1 + 64 epsilon)) < 1774 / 1024.",
    ],
    "weighted": "9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2| in [0, 1708b-10]",
    "weighted_why": [
        "  # By Cauchy-Schwarz the same way, 9/8 |n0| + 9/8 |n1| + 1/2 |n2| <=",
        "  # sqrt(178) / 8 sqrt(1 + 64 epsilon) < 1708 / 1024.",
    ],
    "ratio": "D / L -> |n0| + |n1| + |n2| + 1b-2;",
    "ratio2": "D / L -> 9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2| + 1b-2;",
    "goal": "(value - Ex) / L in [-1b-50, 1b-50]",
}

SHIFT = {
    "restates": SHIFT_RESTATES,
    "name": "shifts",
    "about": "shiftplane.g",
    "q": "d",
    "check": "neither caller checks",
    "y": "y = off + s012;",
    "e3": "e3 = off * W3;",
    "e3_hint": "e3 / c3 -> 4 * (off / (N * L)) * W3;",
    "size": "(N * L)",
    "c3": "c3 = 1b-2 * (N * L);",
    "off_bound": "off / (N * L) in [-1, 1]",
    "sum": "(|n0| + |n1| + |n2|) / N in [0, 1774b-10]",
    "sum_why": [
        "  # |n0| + |n1| + |n2| <= sqrt(3) |n| < 1774 / 1024 |n|, by Cauchy-Schwarz.",
    ],
    "weighted": "(9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N in [0, 1708b-10]",
    "weighted_why": [
        "  # By Cauchy-Schwarz the same way, 9/8 |n0| + 9/8 |n1| + 1/2 |n2| <=",
        "  # sqrt(178) / 8 |n| < 1708 / 1024 |n|.",
    ],
    "ratio": "D / (N * L) -> (|n0| + |n1| + |n2|) / N + 1b-2;",
    "ratio2": "D / (N * L) -> (9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N + 1b-2;",
    "goal": "(value - Ex) / (N * L) in [-1b-49, 1b-49]",
}


def make(s, k, neg):
    q = s["q"]
    size = s["size"]
    others = [i for i in (0, 1, 2) if i != k]
    sign_word = "negative" if neg else "positive"
    prod = {i: f"X{k}" if i == k else f"n{i} * {q}{i}" for i in (0, 1, 2)}
    lines = [f"# restates: {item} at c3b2160e" for item in s["restates"]]
    lines += [
        f"# {s['about']} where n{k} * {q}{k} is below the normals, which",
        f"# {s['check']}: n{k} * {q}{k} is under 2^-1022 and {sign_word}, and the",
        "# other two products are normal.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);",
        f"{q}0 = rnd({q}0_); {q}1 = rnd({q}1_); {q}2 = rnd({q}2_);",
        "off = rnd(off_);",
        "",
        f"X{k} = n{k} * {q}{k};",
        "; ".join(f"t{i} = rnd({prod[i]})" for i in (0, 1, 2)) + ";",
        "S01 = t0 + t1;",
        "s01 = rnd(S01);",
        "S012 = s01 + t2;",
        "s012 = rnd(S012);",
        s["y"],
        "value = rnd(y);",
        "",
    ]
    if s is SIGNED:
        lines.append("Ex = " + " + ".join(prod[i] for i in (0, 1, 2)) + " - off;")
    else:
        lines.append("Ex = off + " + " + ".join(prod[i] for i in (0, 1, 2)) + ";")
        lines.append("N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);")
    lines += [
        "",
        "; ".join(f"rp{i} = t{i} / (n{i} * {q}{i})" for i in others) + ";",
        "rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;",
        "",
        f"# Each term as value carries it, less the term itself, with X{k} for t{k};",
        f"# and what rounding X{k} cost, as an amount.",
    ]
    for i in (0, 1, 2):
        lines.append(f"W{i} = {AFTER[i]} - 1;" if i == k else f"W{i} = {AFTER[i]} * rp{i} - 1;")
    lines.append("W3 = rv - 1;")
    lines.append("; ".join(f"e{i} = {prod[i]} * W{i}" for i in (0, 1, 2)) + "; " + s["e3"])
    lines.append(f"a{k} = (t{k} - X{k}) * ({AFTER[k]});")
    lines.append("")
    if k == 2:
        lines += [
            "# Each term's share of the bound. a2 is spent half out of product 0's",
            "# share and half out of product 1's, each 9/8 of |n_i| L; product 2",
            "# meets two roundings, so 1/2 of |n2| L is its share.",
            "c0 = 9b-3 * (|n0| * L); c1 = 9b-3 * (|n1| * L); c2 = 1b-1 * (|n2| * L); " + s["c3"],
        ]
    else:
        lines += [
            f"# Each term's share of the bound, as in {s['about']}. a{k} is spent out of",
            "# product 2's share, since product 2 meets one rounding fewer.",
            "c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; " + s["c3"],
        ]
    lines += [
        "D = c0 + c1 + c2 + c3;",
        "",
        "{",
        "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024",
        f"  /\\ |{q}0| <= 1b1024 /\\ |{q}1| <= 1b1024 /\\ |{q}2| <= 1b1024",
        "  /\\ |off| <= 1b1024",
        "  # No factor of the three products is zero.",
        "  /\\ |n0| >= 1b-1074 /\\ |n1| >= 1b-1074 /\\ |n2| >= 1b-1074",
        f"  /\\ |{q}0| >= 1b-1074 /\\ |{q}1| >= 1b-1074 /\\ |{q}2| >= 1b-1074",
        "  # No sum cancels to zero.",
        "  /\\ |S01| >= 1b-1074 /\\ |S012| >= 1b-1074 /\\ |y| >= 1b-1074",
        f"  # The case: X{k} is {sign_word} and below the normals, the other two",
        "  # products are normal. A product of two doubles that are not zero is at",
        "  # least 2^-2148.",
        f"  /\\ X{k} in " + ("[-1b-1022, -1b-2148]" if neg else "[1b-2148, 1b-1022]"),
        "  /\\ " + " /\\ ".join(f"|t{i}| >= 1b-1022" for i in others),
        "  # What lemma.g proves each of those two gives.",
    ]
    for i in others:
        lines.append(f"  /\\ t{i} -/ n{i} * {q}{i} {LEMMA}")
    lines += [
        "  # L is at least 2^-1022, and at least every coordinate and the offset.",
        "  /\\ L in [1b-1022, 1b1024]",
        f"  /\\ {q}0 / L in [-1, 1] /\\ {q}1 / L in [-1, 1] /\\ {q}2 / L in [-1, 1]",
        "  /\\ " + s["off_bound"],
    ]
    lines += s["sum_why"]
    lines.append("  /\\ " + s["sum"])
    if k == 2:
        lines += s["weighted_why"]
        lines.append("  /\\ " + s["weighted"])
    lines += [
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
        "  /\\ n0 / |n0| in [-1, 1] /\\ n1 / |n1| in [-1, 1] /\\ n2 / |n2| in [-1, 1]",
        "->",
        f"  a{k} in ? /\\ (value - Ex) / D in ?",
        "  /\\ " + s["goal"],
        "}",
        "",
        "# How far each rounding is from the identity.",
    ]
    for i in others:
        lines.append(f"t{i} / (n{i} * {q}{i}) -> 1 + (t{i} - n{i} * {q}{i}) / (n{i} * {q}{i});")
    lines += [
        "s01 / S01 -> 1 + (s01 - S01) / S01;",
        "s012 / S012 -> 1 + (s012 - S012) / S012;",
        "value / y -> 1 + (value - y) / y;",
        "",
        f"# The error is the sum of the four terms' errors and a{k}, each a fraction",
        "# of its share of D; add_rr then averages them.",
        f"(value - Ex) / {size} -> ((value - Ex) / D) * (D / {size});",
        s["ratio2"] if k == 2 else s["ratio"],
        "(value - Ex) / D -> ((D + (value - Ex)) - D) / D;",
    ]
    if k == 2:
        lines += [
            "D + (value - Ex) -> (c0 + (e0 + 1b-1 * a2)) + (c1 + (e1 + 1b-1 * a2)) + (c2 + e2) + (c3 + e3);",
            "(c0 + (e0 + 1b-1 * a2)) - c0 -> e0 + 1b-1 * a2;",
            "(c1 + (e1 + 1b-1 * a2)) - c1 -> e1 + 1b-1 * a2;",
            "(c2 + e2) - c2 -> e2;",
            "(c3 + e3) - c3 -> e3;",
            f"(e0 + 1b-1 * a2) / c0 -> 8 / 9 * ((n0 / |n0|) * ({q}0 / L) * W0) + 4 / 9 * (a2 / (|n0| * L));",
            f"(e1 + 1b-1 * a2) / c1 -> 8 / 9 * ((n1 / |n1|) * ({q}1 / L) * W1) + 4 / 9 * (a2 / (|n1| * L));",
            f"e2 / c2 -> 2 * ((n2 / |n2|) * ({q}2 / L) * W2);",
            s["e3_hint"],
            "# Product i being normal keeps |n_i| L at least |t_i| / (1 + u), and",
            "# |t_i| at least 2^-1022.",
            f"a2 / (|n0| * L) -> (a2 / t0) * rp0 * ((n0 / |n0|) * ({q}0 / L));",
            f"a2 / (|n1| * L) -> (a2 / t1) * rp1 * ((n1 / |n1|) * ({q}1 / L));",
        ]
    else:
        lines += [
            f"D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + (e2 + a{k})) + (c3 + e3);",
            "(c0 + e0) - c0 -> e0;",
            "(c1 + e1) - c1 -> e1;",
            f"(c2 + (e2 + a{k})) - c2 -> e2 + a{k};",
            "(c3 + e3) - c3 -> e3;",
            f"e{others[0]} / c{others[0]} -> (n{others[0]} / |n{others[0]}|) * ({q}{others[0]} / L) * W{others[0]};",
            f"e{k} / c{k} -> (n{k} / |n{k}|) * ({q}{k} / L) * W{k};",
            f"(e2 + a{k}) / c2 -> (n2 / |n2|) * ({q}2 / L) * W2 + a{k} / (|n2| * L);",
            s["e3_hint"],
            "# Product 2 being normal keeps |n2| L at least |t2| / (1 + u), and |t2|",
            "# at least 2^-1022.",
            f"a{k} / (|n2| * L) -> (a{k} / t2) * rp2 * ((n2 / |n2|) * ({q}2 / L));",
        ]
    return "\n".join(lines) + "\n"


for s in (SIGNED, SHIFT):
    for k in (0, 1, 2):
        for neg in (False, True):
            name = f"{s['name']}{k}{'n' if neg else ''}"
            open(f"{name}.g", "w").write(make(s, k, neg))
