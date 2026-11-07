"""Write 464.19's point.g in its zero branches, one script per case and a
control at u / 2 beside each.

point.g took every factor of its nine products to be nonzero and none of its
seven sums to cancel. Here each of those may be zero:

- a factor, one of w, v0, v1, v2, p0, p1, p2, written as the double 0. A turn
  about a coordinate axis stores two of v as 0, a half turn stores w as 0,
  and a point on a coordinate plane has a p_i of 0. s0 is no factor, and
  point.g already lets it be 0.
- a sum, one of C0, C1, C2, KK, A, R, O, whose two parts are not zero: it is
  either rounded, and then at least 2^-1074 as point.g has it, or it cancels
  to zero, which rounds to itself (464.11's diffzero.g). A sum with one part
  zero is the other part, or its negative, and is not zero; one with both
  parts zero is zero. The one part that can be zero without being a factor
  or a zero rounding is s0, so O = r0 + s0 with r0 zero is s0, and is split
  the same way: rounded, s0 not 0, or zero, s0 = 0, named as if it cancelled.

A product with a zero factor, a sum with both parts zero and a sum that
cancels are each zero and round to themselves (464.11's refl.g), so each is
written unrounded, and its ratio, its hypotheses and its ratio hint go; a
cancelling sum is put in [0, 0]. Every other rounding stays as point.g has
it, a sum with one part zero included: it rounds nothing, and charging it u
costs only room. Each rounded sum adds two doubles, so is a multiple of
2^-1074, and is said to be with @FIX: Gappa sees that itself where both parts
are rounded, and not where a part is a zero written unrounded. Each term of Ex
is carried through the roundings left on its path, so its W is their product
less one. A term with a zero factor is zero and drops out of the split, with
its share of D.

A rounding on no path of a term without a zero factor is multiplied by zero
before it reaches out. It stays rounded, with no hypothesis, and a sum there
is not split into rounded and cancelling, since the two scripts would be the
same.

D / L is hinted as point.g's quadratic less the share of each term a zero
factor took out. Gappa bounds |v| * |v| for no v that is the double 0, so a
zero v1 or v2 has its 1/545 square taken out with those shares, written
1/545 * |v| * |v|, which Gappa reads as (1/545 * |v|) * |v| and bounds as 0.

Where no term is live, p0 is zero and s0 is 0, out and Ex are both 0, so a
control at u / 2 holds. Its control asks for (out - Ex) / L in [2^-60, 1]
instead, which leaves out 0.

named_by (isometry.rs) takes a turn with one nonzero part, or two or four of
one size, as a turn of a cube, which is 464.19's 6 and not this arithmetic. So
a zero set holding three of w, v0, v1, v2 is left out, and all four cannot be
zero with q.q within 6 epsilon of one. A zero set holding two keeps the two
others unequal in size, which these scripts do not use. The moved point's y
and z are x with the indices turned (turn.py), and the zero sets left are
closed under that turn, so x in each of them is y and z in each too.

The names are pz_<zeros>[_x<sums>].g, <zeros> the zero factors or n for none
and <sums> the sums that cancel, and c_pz_<...>_half.g at u / 2.
Run as: python3 pointzero.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

FACTORS = ("w", "v0", "v1", "v2", "p0", "p1", "p2")
TURN = ("w", "v0", "v1", "v2")
GOAL = "(out - Ex) / L in [-36b-53, 36b-53]"
HALF = "(out - Ex) / L in [-1b-54, 1b-54]"
ZERO = "(out - Ex) / L in [1b-60, 1]"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"

# Each rounding of point.g: its result, its argument, the argument's
# definition, its ratio, and whether it is a product or a sum.
OPS = (
    ("q01", "P01", "v1 * p2", "rq01", "product"),
    ("q02", "P02", "v2 * p1", "rq02", "product"),
    ("q11", "P11", "v2 * p0", "rq11", "product"),
    ("q12", "P12", "v0 * p2", "rq12", "product"),
    ("q21", "P21", "v0 * p1", "rq21", "product"),
    ("q22", "P22", "v1 * p0", "rq22", "product"),
    ("c0", "C0", "q01 - q02", "rc0", "sum"),
    ("c1", "C1", "q11 - q12", "rc1", "sum"),
    ("c2", "C2", "q21 - q22", "rc2", "sum"),
    ("k01", "K1", "v1 * t2", "rk01", "product"),
    ("k02", "K2", "v2 * t1", "rk02", "product"),
    ("k0", "KK", "k01 - k02", "rk0", "sum"),
    ("m0", "M", "t0 * w", "rm", "product"),
    ("a0", "A", "m0 + k0", "ra", "sum"),
    ("r0", "R", "a0 + p0", "rr", "sum"),
    ("out", "O", "r0 + s0", "ro", "sum"),
)
# t_i is c_i * 2, which rounds nothing (464.19's doubling.g).
DOUBLED = {"t0": "c0", "t1": "c1", "t2": "c2"}

# The six terms of Ex with a product of the turn: the term, its factors, the
# roundings it passes through in order, its share of D, and e / d as point.g
# writes it.
TERMS = (
    ("2 * w * v1 * p2", ("w", "v1", "p2"), ("q01", "c0", "m0", "a0", "r0", "out"),
     "12 * |w| * |v1|", "(w / |w|) * (v1 / |v1|) * (p2 / L) * (W1 / 6)"),
    ("-(2 * w * v2 * p1)", ("w", "v2", "p1"), ("q02", "c0", "m0", "a0", "r0", "out"),
     "12 * |w| * |v2|", "-((w / |w|) * (v2 / |v2|) * (p1 / L) * (W2 / 6))"),
    ("2 * v1 * v0 * p1", ("v1", "v0", "p1"), ("q21", "c2", "k01", "k0", "a0", "r0", "out"),
     "14 * |v1| * |v0|", "(v1 / |v1|) * (v0 / |v0|) * (p1 / L) * (W3 / 7)"),
    ("-(2 * v1 * v1 * p0)", ("v1", "p0"), ("q22", "c2", "k01", "k0", "a0", "r0", "out"),
     "14 * |v1| * |v1|", "-((v1 / |v1|) * (v1 / |v1|) * (p0 / L) * (W4 / 7))"),
    ("-(2 * v2 * v2 * p0)", ("v2", "p0"), ("q11", "c1", "k02", "k0", "a0", "r0", "out"),
     "14 * |v2| * |v2|", "-((v2 / |v2|) * (v2 / |v2|) * (p0 / L) * (W5 / 7))"),
    ("2 * v2 * v0 * p2", ("v2", "v0", "p2"), ("q12", "c1", "k02", "k0", "a0", "r0", "out"),
     "14 * |v2| * |v0|", "(v2 / |v2|) * (v0 / |v0|) * (p2 / L) * (W6 / 7)"),
)
P0_PATH = ("r0", "out")
S0_PATH = ("out",)


def cases():
    """Every zero set and every choice of which sums on a live path cancel."""
    for n in range(len(FACTORS) + 1):
        for zero in itertools.combinations(FACTORS, n):
            if sum(f in zero for f in TURN) >= 3:
                continue
            yield from branches(set(zero), 0, {})


def used(zero):
    """The roundings on the path of a term that has no zero factor."""
    paths = [t[2] for t in TERMS if not set(t[1]) & zero] + [S0_PATH]
    if "p0" not in zero:
        paths.append(P0_PATH)
    return {r for path in paths for r in path}


def branches(zero, i, state):
    """state maps each rounding's result to rounded, zero or cancels."""
    if i == len(OPS):
        yield frozenset(zero), dict(state)
        return
    result, _, arg, _, kind = OPS[i]
    a, _, b = arg.split(" ")

    def is_zero(name):
        name = DOUBLED.get(name, name)
        if name in FACTORS:
            return name in zero
        if name == "s0":
            return False
        return state[name] != "rounded"

    za, zb = is_zero(a), is_zero(b)
    if kind == "product":
        choices = ("zero",) if za or zb else ("rounded",)
    elif za and zb:
        choices = ("zero",)
    elif result not in used(zero) or (za or zb) and b != "s0":
        choices = ("rounded",)
    else:
        choices = ("rounded", "cancels")
    for s in choices:
        state[result] = s
        yield from branches(zero, i + 1, state)
    del state[result]


def name_of(zero, state):
    zeros = "".join(f for f in FACTORS if f in zero) or "n"
    cancels = "".join(op[1].lower() for op in OPS if state[op[0]] == "cancels")
    return "pz_" + zeros + ("_x" + cancels if cancels else "")


def make(zero, state, goal=GOAL):
    live_ops = used(zero)
    rounded = [op for op in OPS if state[op[0]] == "rounded" and op[0] in live_ops]
    live = [k for k, t in enumerate(TERMS) if not set(t[1]) & zero]
    gone = [k for k in range(len(TERMS)) if k not in live]
    p0_live = "p0" not in zero
    zeros = [f for f in FACTORS if f in zero]
    cancels = [op[1] for op in OPS if state[op[0]] == "cancels"]
    unrounded = [op[1] for op in OPS if state[op[0]] != "rounded"]

    def listed(names):
        return ", ".join(names) if names else "none"

    src = [
        "# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e",
        "# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e",
        "# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e",
        "# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee",
        "# restates: Cargo.lock nalgebra at ba0a654e",
        "# 464.19's point.g, the moved point's x, with zero factors " + listed(zeros) + ",",
        "# cancelling sums " + listed(cancels) + ", and so zero " + listed(unrounded) + ".",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The quaternion (w, v), the point and the shift's x, each an f64, a zero",
        "# factor the double 0.",
    ]

    def inputs(names):
        return " ".join(f"{n} = 0;" if n in zero else f"{n} = rnd({n}_);" for n in names)

    src += [inputs(TURN), inputs(("p0", "p1", "p2")), "s0 = rnd(s0_);", "",
            "# What apply_to_point computes, line for line, a zero written unrounded,",
            "# since it rounds to itself (464.11's refl.g)."]
    for result, argname, arg, _, _ in OPS:
        value = f"rnd({argname})" if state[result] == "rounded" else argname
        src.append(f"{argname} = {arg}; {result} = {value};")
        if result == "c2":
            src.append("t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;")
    src += ["", "# The same arithmetic without rounding.",
            "Ex = 2 * w * (v1 * p2 - v2 * p1)",
            "   + v1 * (2 * (v0 * p1 - v1 * p0)) - v2 * (2 * (v2 * p0 - v0 * p2))",
            "   + p0 + s0;", "",
            "# What each rounding on a live path multiplied its argument by."]
    for k in range(0, len(rounded), 4):
        src.append(" ".join(f"{op[3]} = {op[0]} / {op[1]};" for op in rounded[k:k + 4]))
    ratio = {op[0]: op[3] for op in rounded}

    def chain(path):
        rs = [ratio[r] for r in reversed(path) if r in ratio]
        return " * ".join(rs) + " - 1" if rs else "0"

    src += ["", "# Each term of Ex as out carries it, less the term itself, through the",
            "# roundings left on its path. A term with a zero factor is zero."]
    src += [f"W{k + 1} = {chain(TERMS[k][2])};" for k in live]
    if p0_live:
        src.append(f"W7 = {chain(P0_PATH)};")
    src.append(f"W8 = {chain(S0_PATH)};")
    for k in live:
        t = TERMS[k][0]
        src.append(f"e{k + 1} = -({t[2:-1]} * W{k + 1});" if t.startswith("-(") else f"e{k + 1} = {t} * W{k + 1};")
    src.append("e7 = p0 * W7 + s0 * W8;" if p0_live else "e7 = s0 * W8;")
    src += ["", "# Each live term's share of D, as point.g's, and p0 and s0 sharing 3L."]
    src += [f"d{k + 1} = {TERMS[k][3]} * L;" for k in live]
    src += ["d7 = 3 * L;", "D = " + " + ".join(f"d{k + 1}" for k in live + [6]) + ";", "",
            "# q.q and the four squares D / L is written with below.",
            "N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;",
            "X = |w| - 30/109 * (|v1| + |v2|);",
            "Y = |v0| - 35/109 * (|v1| + |v2|);",
            "Z = |v1| - |v2|;", ""]
    nz = [f for f in FACTORS if f not in zero]
    hyp = ["  # Every input is a finite f64.",
           "     " + " /\\ ".join(f"|{n}| <= 1b1024" for n in nz[:4]),
           "  /\\ " + " /\\ ".join(f"|{n}| <= 1b1024" for n in nz[4:] + ["s0"]),
           "  # No factor of a product is zero but " + listed(zeros) + ".",
           "  /\\ " + " /\\ ".join(f"|{n}| >= 1b-1074" for n in nz)]
    rs = [op for op in rounded if op[4] == "sum"]
    if rs:
        hyp += ["  # The rounded sums on a live path do not cancel, and each adds two doubles."]
        hyp += ["  /\\ " + " /\\ ".join(f"|{op[1]}| >= 1b-1074" for op in rs[k:k + 4]) for k in range(0, len(rs), 4)]
        hyp += ["  /\\ " + " /\\ ".join(f"@FIX({op[1]}, -1074)" for op in rs[k:k + 4]) for k in range(0, len(rs), 4)]
    if cancels:
        hyp += ["  # The sums that cancel.", "  /\\ " + " /\\ ".join(f"{c} in [0, 0]" for c in cancels)]
    rp = [op for op in rounded if op[4] == "product"]
    if rp:
        hyp += ["  # The products on a live path are normal, which apply_to_point does not",
                "  # check, and what lemma.g proves each of them gives."]
        hyp += ["  /\\ " + " /\\ ".join(f"|{op[0]}| >= 1b-1022" for op in rp[k:k + 3]) for k in range(0, len(rp), 3)]
        hyp += [f"  /\\ {op[0]} -/ {op[1]} {LEMMA}" for op in rp]
    hyp += ["  # L is at least every coordinate of the point and the shift.",
            "  /\\ L in [1b-1074, 1b1024]",
            "  /\\ " + " /\\ ".join(f"{n} / L in [-1, 1]" for n in ("p0", "p1", "p2", "s0") if n not in zero),
            "  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and",
            "  # 6 epsilon is 12b-53.",
            "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]",
            "  # x / |x| is in [-1, 1] for any x that is not zero.",
            "  /\\ " + " /\\ ".join(f"{n} / |{n}| in [-1, 1]" for n in TURN if n not in zero)]
    src += ["{"] + hyp + ["->", "  (out - Ex) / D in ? /\\ D / L in ?", f"  /\\ {goal}", "}", ""]
    if rounded:
        src.append("# How far each rounding on a live path is from the identity.")
        src += [f"{op[0]} / {op[1]} -> 1 + ({op[0]} - {op[1]}) / {op[1]};" for op in rounded]
        src.append("")
    src += ["# The error is the sum of the live shares' errors, each a fraction of its",
            "# own share of D; add_rr then averages them.",
            "(out - Ex) / L -> ((out - Ex) / D) * (D / L);",
            "(out - Ex) / D -> ((D + (out - Ex)) - D) / D;",
            "D + (out - Ex) -> " + " + ".join(f"(d{k + 1} + e{k + 1})" for k in live + [6]) + ";"]
    src += [f"(d{k + 1} + e{k + 1}) - d{k + 1} -> e{k + 1};" for k in live + [6]]
    src += [f"e{k + 1} / d{k + 1} -> {TERMS[k][4]};" for k in live]
    src.append("e7 / d7 -> ((p0 / L) * W7 + (s0 / L) * W8) / 3;" if p0_live else "e7 / d7 -> ((s0 / L) * W8) / 3;")
    kept = [v for v in ("v1", "v2") if v not in zero]
    took = [TERMS[k][3] for k in gone] + [f"1/545 * |{v}| * |{v}|" for v in ("v1", "v2") if v in zero]
    hint = ["D / L -> 109/5 * N4 - (109/5 * (X * X) + 109/5 * (Y * Y) + 425/109 * (Z * Z)"]
    if kept:
        hint.append("                       + 1/545 * (" + " + ".join(f"|{v}| * |{v}|" for v in kept) + ")) + 3")
    else:
        hint[0] += ") + 3"
    if took:
        hint.append("        - (" + " + ".join(took) + ")")
    src += ["", "# D / L is point.g's 3 and quadratic, 109/5 q.q less four squares, less",
            "# the share of each term a zero factor took out, and a zero v's square."]
    src += hint[:-1] + [hint[-1] + ";"]
    return "\n".join(src) + "\n"


def main():
    seen, exact = [], []
    for zero, state in cases():
        name = name_of(zero, state)
        assert name not in seen, name
        seen.append(name)
        open(name + ".g", "w").write(make(zero, state))
        if all(set(t[1]) & zero for t in TERMS) and "p0" in zero and state["out"] == "cancels":
            exact.append(name)
            open("c_" + name + "_half.g", "w").write(make(zero, state, ZERO))
        else:
            open("c_" + name + "_half.g", "w").write(make(zero, state, HALF))
    open("cases.txt", "w").write("\n".join(seen) + "\n")
    print(len(seen), "cases,", len(exact), "with out and Ex both 0")


main()
