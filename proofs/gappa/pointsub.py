"""Write 464.34's zero sets of 464.19's point.g again with each product on a
live path free to fall below the normals, one script per zero set and a
control at u / 2 beside each.

pointzero.py (464.34) took each rounded product on a live path to be a normal
number, and gave its rounding lemma.g's relative bound. Here a product
q = rnd(P) may be normal, below the normals, or 0, and what mixedn.g and
mixeds.g prove of every real P is all the script knows of it:

  m = (q - P) / (|P| + 2^-1022) is in [-u, u].

So for P not zero, q = P * (1 + m * s) + m * 2^-1022 with s = |P| / P in
[-1, 1]: a rounding by the ratio 1 + m * s, within u of 1 as before, and an
amount m * 2^-1022 that no ratio carries. A product that rounds to 0 with
neither factor 0 is such a q too. The script writes each such product that
way, with m and s free in their intervals, which every rnd(P) meets, so it
proves the goal for more than the rounded products and so for them. Each
rounded sum on a live path is written C * r, r free within u of 1, which
sum.g proves of the sum of two doubles that is not zero, and which holds of
any r where C is 0.

Each amount reaches out multiplied by the factor the sums and products after
it multiply the product by, f, and by the ratios of the roundings after it:
f is 2w for q01, -2w for q02, 2v1 for q21, -2v1 for q22, -2v2 for q11, 2v2
for q12, 1 for k01, -1 for k02 and 1 for m0. So the amount is at most
|f| * 2^-1022 * u, and a little more for the ratios, and it gets a share of
its own, |f| * L. With L at least 2^-1022, the amount is at most u of its
share. With L below 2^-1022 nothing bounds 2^-1022 / L.

out - Ex is then E, the sum of the terms' errors as in point.g, plus Asum,
the sum of the amounts. E is averaged over D, point.g's shares, as before,
and Asum over G, the amounts' shares, on its own. G / L is
2 n_w |w| + 2 n_1 |v1| + 2 n_2 |v2| + n_c, the n the live rounded products
among q01 and q02, q21 and q22, q11 and q12, and k01, k02 and m0. Each
2 n |x| is written as lam |x|^2 + n^2 / lam - lam (|x| - n / lam)^2, and the
three lam |x|^2 as lam q.q less lam |v0|^2, so the hint is lam q.q, a
constant and squares taken away, which Gappa bounds by lam (1 + 12u) plus
the constant. lam is about sqrt(n_w^2 + n_1^2 + n_2^2), which makes
lam + (n_w^2 + n_1^2 + n_2^2) / lam smallest. An x whose n is 0 has its
square written lam * |x| * |x|, which Gappa bounds where x is the double 0
(464.34's 2).

Every other line is pointzero.py's. A sum that cancels is 0, and 0 times any
ratio is 0, so it needs no case of its own, and nor does a product that
rounds to 0: one case for each of the 88 zero sets.

The names are pm_<zeros>.g, as pointzero.py's pz_, and c_pm_<zeros>_half.g
at u / 2.
Run as: python3 pointsub.py
"""
import itertools
import math
import os
from fractions import Fraction

os.chdir(os.path.dirname(os.path.abspath(__file__)))

FACTORS = ("w", "v0", "v1", "v2", "p0", "p1", "p2")
TURN = ("w", "v0", "v1", "v2")
GOAL = "(out - Ex) / L in [-36b-53, 36b-53]"
HALF = "(out - Ex) / L in [-1b-54, 1b-54]"
RATIO = "9007199254740991b-53, 9007199254740993b-53"
# Where a product is rounded on a live path, out - Ex = E + Asum. These goals
# on each, which add up to 36u, are for a script of E or Asum alone.
PART_GOAL = {"e": "E / L in [-26b-53, 26b-53]", "a": "Asum / L in [-10b-53, 10b-53]"}
NORMAL_L = "1b-1022"

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

# Each product's amount as it reaches out: its sign, and the part of the turn
# its factor f is twice, or None where f is 1.
AMOUNTS = {
    "q01": ("", "w"), "q02": ("-", "w"),
    "q21": ("", "v1"), "q22": ("-", "v1"),
    "q11": ("-", "v2"), "q12": ("", "v2"),
    "k01": ("", None), "k02": ("-", None), "m0": ("", None),
}
# The names the squares of G / L are written with.
SQUARES = {"w": "Hw", "v1": "H1", "v2": "H2"}

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
    """Every zero set."""
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
    """state maps each rounding's result to rounded or zero: zero where a
    factor of a product is zero, or both parts of a sum."""
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
        state[result] = "zero" if za or zb else "rounded"
    else:
        state[result] = "zero" if za and zb else "rounded"
    yield from branches(zero, i + 1, state)
    del state[result]


def name_of(zero, state):
    return "pm_" + ("".join(f for f in FACTORS if f in zero) or "n")


def products(zero, state):
    """The rounded products on a live path."""
    live_ops = used(zero)
    return [op for op in OPS if state[op[0]] == "rounded" and op[0] in live_ops and op[4] == "product"]


def make(zero, state, goal=GOAL, lmin=NORMAL_L, part=None):
    """part is None where no product is rounded on a live path. Otherwise it
    is "both", which bounds E and Asum and adds them, or "e" or "a" for a
    script that bounds E or Asum alone."""
    live_ops = used(zero)
    rounded = [op for op in OPS if state[op[0]] == "rounded" and op[0] in live_ops]
    assert (part is None) == (not products(zero, state)), part
    live = [k for k, t in enumerate(TERMS) if not set(t[1]) & zero]
    gone = [k for k in range(len(TERMS)) if k not in live]
    p0_live = "p0" not in zero
    zeros = [f for f in FACTORS if f in zero]
    unrounded = [op[1] for op in OPS if state[op[0]] != "rounded"]
    rp = [op for op in rounded if op[4] == "product"]
    rs = [op for op in rounded if op[4] == "sum"]

    def listed(names):
        return ", ".join(names) if names else "none"

    src = [
        "# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e",
        "# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e",
        "# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e",
        "# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee",
        "# restates: Cargo.lock nalgebra at ba0a654e",
        "# 464.19's point.g, the moved point's x, with zero factors " + listed(zeros) + ",",
        "# and so zero " + listed(unrounded) + ", each rounded product on a live",
        "# path normal, below the normals or 0, and each rounded sum on one free",
        "# to cancel.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The quaternion (w, v), the point and the shift's x, each an f64, a zero",
        "# factor the double 0.",
    ]

    def inputs(names):
        return " ".join(f"{n} = 0;" if n in zero else f"{n} = rnd({n}_);" for n in names)

    src += [inputs(TURN), inputs(("p0", "p1", "p2")), "s0 = rnd(s0_);", "",
            "# What apply_to_point computes, line for line, a zero written unrounded,",
            "# since it rounds to itself (464.11's refl.g). A rounded product on a live",
            "# path is written as what mixedn.g and mixeds.g prove of it, rnd(P) =",
            "# P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],",
            "# and a rounded sum on one as what sum.g proves of it, C * r."]
    for op in OPS:
        result, argname, arg, r, _ = op
        if op in rp:
            src.append(f"{argname} = {arg}; {r} = 1 + m{result} * s{result};"
                       f" {result} = {argname} * {r} + m{result} * 1b-1022;")
        elif op in rs:
            src.append(f"{argname} = {arg}; {result} = {argname} * {r};")
        else:
            value = f"rnd({argname})" if state[result] == "rounded" else argname
            src.append(f"{argname} = {arg}; {result} = {value};")
        if result == "c2":
            src.append("t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;")
    src += ["", "# The same arithmetic without rounding.",
            "Ex = 2 * w * (v1 * p2 - v2 * p1)",
            "   + v1 * (2 * (v0 * p1 - v1 * p0)) - v2 * (2 * (v2 * p0 - v0 * p2))",
            "   + p0 + s0;", ""]
    ratio = {op[0]: op[3] for op in rounded}

    def share(q):
        return f"g{q}" if AMOUNTS[q][1] else "L"

    def ratios_on(path):
        return [ratio[r] for r in reversed(path) if r in ratio]

    def chain(path):
        rs_ = ratios_on(path)
        return " * ".join(rs_) + " - 1" if rs_ else "0"

    def after(result):
        path = next(t[2] for k, t in enumerate(TERMS) if k in live and result in t[2])
        return ratios_on(path[path.index(result) + 1:])

    src += ["# Each term of Ex as out carries it, less the term itself, through the",
            "# roundings left on its path. A term with a zero factor is zero."]
    src += [f"W{k + 1} = {chain(TERMS[k][2])};" for k in live]
    if p0_live:
        src.append(f"W7 = {chain(P0_PATH)};")
    src.append(f"W8 = {chain(S0_PATH)};")
    for k in live:
        t = TERMS[k][0]
        src.append(f"e{k + 1} = -({t[2:-1]} * W{k + 1});" if t.startswith("-(") else f"e{k + 1} = {t} * W{k + 1};")
    src.append("e7 = p0 * W7 + s0 * W8;" if p0_live else "e7 = s0 * W8;")

    amounts, hints = [], []
    for op in rp:
        q = op[0]
        sign, x = AMOUNTS[q]
        rest = after(q)
        tail = " * (" + " * ".join(rest) + ")" if rest else ""
        factor = f" * (2 * {x})" if x else ""
        body = f"m{q} * 1b-1022{factor}{tail}"
        amounts.append(f"A{q} = {sign}({body});" if sign else f"A{q} = {body};")
        unit = f" * ({x} / |{x}|)" if x else ""
        hbody = f"m{q} * (1b-1022 / L){unit}{tail}"
        hints.append(f"A{q} / {share(q)} -> {sign}({hbody});" if sign else f"A{q} / {share(q)} -> {hbody};")
    if amounts:
        src += ["", "# What each rounded product's m * 2^-1022 comes to in out, through the",
                "# sums and products after it and the roundings left on its path."]
        src += amounts

    src += ["", "# Each live term's share of D, as point.g's, p0 and s0 sharing 3L."]
    src += [f"d{k + 1} = {TERMS[k][3]} * L;" for k in live]
    src.append("d7 = 3 * L;")
    src += ["D = " + " + ".join(f"d{k + 1}" for k in live + [6]) + ";"]
    if rp:
        src += ["", "# Each amount's share of G, |f| L, written L itself where f is 1: Gappa",
                "# 1.8.0 aborts on a name defined as another name alone, as in g = L.",
                "# E is out - Ex less the amounts, the terms' errors alone."]
        src += [f"g{op[0]} = 2 * |{AMOUNTS[op[0]][1]}| * L;" for op in rp if AMOUNTS[op[0]][1]]
        src += ["G = " + " + ".join(share(op[0]) for op in rp) + ";",
                "Asum = " + " + ".join(f"A{op[0]}" for op in rp) + ";",
                "E = out - Ex - Asum;"]
    src += ["",
            "# q.q and the four squares D / L is written with below.",
            "N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;",
            "X = |w| - 30/109 * (|v1| + |v2|);",
            "Y = |v0| - 35/109 * (|v1| + |v2|);",
            "Z = |v1| - |v2|;"]

    n = {x: sum(AMOUNTS[op[0]][1] == x for op in rp) for x in ("w", "v1", "v2")}
    n_c = sum(AMOUNTS[op[0]][1] is None for op in rp)
    s = sum(c * c for c in n.values())
    lam = Fraction(math.sqrt(s)).limit_denominator(8) if s else None
    if lam is not None:
        src += ["", f"# G / L's linear part as squares, lam = {lam}."]
        src += [f"{SQUARES[x]} = |{x}| - {Fraction(c) / lam};" for x, c in n.items() if c]
    src.append("")

    nz = [f for f in FACTORS if f not in zero]
    hyp = ["  # Every input is a finite f64.",
           "     " + " /\\ ".join(f"|{f}| <= 1b1024" for f in nz[:4]),
           "  /\\ " + " /\\ ".join(f"|{f}| <= 1b1024" for f in nz[4:] + ["s0"]),
           "  # No factor of a product is zero but " + listed(zeros) + ".",
           "  /\\ " + " /\\ ".join(f"|{f}| >= 1b-1074" for f in nz)]
    if rs:
        hyp += ["  # What sum.g proves of each rounded sum on a live path, the sum of two",
                "  # doubles, or 0 where it cancels, which any ratio meets."]
        hyp += ["  /\\ " + " /\\ ".join(f"{op[3]} in [{RATIO}]" for op in rs[k:k + 2]) for k in range(0, len(rs), 2)]
    if rp:
        hyp += ["  # What mixedn.g and mixeds.g prove of each rounded product on a live",
                "  # path, normal or not, and |P| / P in [-1, 1] for any P not zero."]
        hyp += ["  /\\ " + " /\\ ".join(f"m{op[0]} in [-1b-53, 1b-53]" for op in rp[k:k + 3]) for k in range(0, len(rp), 3)]
        hyp += ["  /\\ " + " /\\ ".join(f"s{op[0]} in [-1, 1]" for op in rp[k:k + 3]) for k in range(0, len(rp), 3)]
    hyp += ["  # L is a normal number at least every coordinate of the point and the shift.",
            f"  /\\ L in [{lmin}, 1b1024]",
            "  /\\ " + " /\\ ".join(f"{f} / L in [-1, 1]" for f in ("p0", "p1", "p2", "s0") if f not in zero),
            "  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and",
            "  # 6 epsilon is 12b-53.",
            "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]",
            "  # x / |x| is in [-1, 1] for any x that is not zero.",
            "  /\\ " + " /\\ ".join(f"{f} / |{f}| in [-1, 1]" for f in TURN if f not in zero)]
    err = "E" if rp else "(out - Ex)"
    do_e, do_a = part != "a", part in ("a", "both")
    queries = []
    if do_e:
        queries.append(f"{err} / D in ? /\\ D / L in ?")
    if do_a:
        queries.append("Asum / G in ? /\\ G / L in ?")
    src += ["{"] + hyp + ["->", "  " + " /\\ ".join(queries), f"  /\\ {goal}", "}", ""]
    if part == "both":
        src += ["# out - Ex is the terms' error and the amounts, each averaged on its own.",
                "(out - Ex) / L -> E / L + Asum / L;"]
    if do_a:
        src += ["# The amounts, each a fraction of its own share of G; add_rr averages them.",
                "Asum / L -> (Asum / G) * (G / L);",
                "Asum / G -> ((G + Asum) - G) / G;",
                "G + Asum -> " + " + ".join(f"({share(op[0])} + A{op[0]})" for op in rp) + ";"]
        src += [f"({share(op[0])} + A{op[0]}) - {share(op[0])} -> A{op[0]};" for op in rp]
        src += hints
    if do_e:
        src += ["# The terms' error is the sum of the live shares' errors, each a fraction",
                "# of its own share of D; add_rr then averages them.",
                f"{err} / L -> ({err} / D) * (D / L);",
                f"{err} / D -> ((D + {err}) - D) / D;",
                f"D + {err} -> " + " + ".join(f"(d{k + 1} + e{k + 1})" for k in live + [6]) + ";"]
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
    if do_e:
        src += ["", "# D / L is point.g's 3 and quadratic, 109/5 q.q less four squares, less",
                "# the share of each term a zero factor took out, and a zero v's square."]
        src += hint[:-1] + [hint[-1] + ";"]
    if do_a and lam is not None:
        constant = sum(Fraction(c * c) / lam for c in n.values()) + n_c
        squares = [f"{lam} * |v0| * |v0|"] + [
            f"{lam} * ({SQUARES[x]} * {SQUARES[x]})" if c else f"{lam} * |{x}| * |{x}|" for x, c in n.items()]
        src += ["", "# G / L is the amounts' shares, 2 n_w |w| + 2 n_1 |v1| + 2 n_2 |v2| + n_c,",
                "# written as lam q.q, a constant and squares.",
                f"G / L -> {lam} * N4 + {constant}",
                "        - (" + " + ".join(squares) + ");"]
    elif do_a:
        src += ["", "# G / L is the amounts' shares, n_c L.", f"G / L -> {n_c};"]
    return "\n".join(src) + "\n"


def main():
    seen = []
    for zero, state in cases():
        name = name_of(zero, state)
        assert name not in seen, name
        seen.append(name)
        part = "both" if products(zero, state) else None
        open(name + ".g", "w").write(make(zero, state, part=part))
        open("c_" + name + "_half.g", "w").write(make(zero, state, HALF, part=part))
    open("casesm.txt", "w").write("\n".join(seen) + "\n")
    print(len(seen), "cases,", sum(not products(z, s) for z, s in cases()), "with no rounded product on a live path")


if __name__ == "__main__":
    main()
