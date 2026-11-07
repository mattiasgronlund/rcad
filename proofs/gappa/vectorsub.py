"""Write 464.20's vector.g in 464.35's form: one script per zero set of its
seven factors, each rounded product on a live path free to fall below the
normals and each rounded sum on one free to cancel, and a control beside each.

vector.g is 464.19's point.g with no shift and every component of the vector
at most one, so this is 464.35's pointsub.py with n for its p, s0 gone and
the vector's own shares. A rounded product on a live path is written
q = P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],
which mixedn.g and mixeds.g prove of every real P, and a rounded sum on one
C * r, r free within u of 1, which sum.g proves of the sum of two doubles
that is not zero and which holds of any r where C is 0. A zero factor is the
double 0, and each product and sum it makes zero is written unrounded.

out has one rounding fewer on each path than point.g's, since no shift is
added, so the terms' shares of D are vector.g's: 10 |w| |v1| for the two
through w, 12 for the four through k, and 1 for n0, each with 1 for the
vector's component. D is vector.g's 93/5 q.q less four squares, plus 1,
less the share of each term a zero factor took out, a zero v's 19/465
square, and the 1 where n0 is zero.

Each amount m * 2^-1022 reaches out multiplied by the factor f the sums and
products after it multiply its product by, 2w for q01 as in pointsub.py, and
by the ratios after it. Its share of G is |f| * Lo, Lo the smallest normal
number 2^-1022 as a name, so each amount is within u of its share and G / Lo
is pointsub.py's G / L. The goal is absolute, as vector.g's: out - Ex within
20u, which the amounts, at most 9.93 u 2^-1022 together, barely touch.

Where no term of Ex keeps all its factors and n0 is zero, out and Ex are
both exactly 0: the script has no D and no control, since a control that
asks for out - Ex in [2^-60, 1] ends in Gappa's "no contradiction was found",
which prove-bounds does not take for a failed control (as pointzero.py's 21).
Every other control asks for u / 2.

The names are vm_<zeros>.g and c_vm_<zeros>_half.g.
Run as: python3 vectorsub.py
"""
import itertools
import math
import os
from fractions import Fraction

os.chdir(os.path.dirname(os.path.abspath(__file__)))

FACTORS = ("w", "v0", "v1", "v2", "n0", "n1", "n2")
TURN = ("w", "v0", "v1", "v2")
VECTOR = ("n0", "n1", "n2")
GOAL = "out - Ex in [-20b-53, 20b-53]"
HALF = "out - Ex in [-1b-54, 1b-54]"
RATIO = "9007199254740991b-53, 9007199254740993b-53"

# Each rounding of vector.g: its result, its argument, the argument's
# definition, its ratio, and whether it is a product or a sum.
OPS = (
    ("q01", "P01", "v1 * n2", "rq01", "product"),
    ("q02", "P02", "v2 * n1", "rq02", "product"),
    ("q11", "P11", "v2 * n0", "rq11", "product"),
    ("q12", "P12", "v0 * n2", "rq12", "product"),
    ("q21", "P21", "v0 * n1", "rq21", "product"),
    ("q22", "P22", "v1 * n0", "rq22", "product"),
    ("c0", "C0", "q01 - q02", "rc0", "sum"),
    ("c1", "C1", "q11 - q12", "rc1", "sum"),
    ("c2", "C2", "q21 - q22", "rc2", "sum"),
    ("k01", "K1", "v1 * t2", "rk01", "product"),
    ("k02", "K2", "v2 * t1", "rk02", "product"),
    ("k0", "KK", "k01 - k02", "rk0", "sum"),
    ("m0", "M", "t0 * w", "rm", "product"),
    ("a0", "A", "m0 + k0", "ra", "sum"),
    ("out", "R", "a0 + n0", "rr", "sum"),
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
SQUARES = {"w": "Hw", "v1": "H1", "v2": "H2"}

# The six terms of Ex with a product of the turn: the term, its factors, the
# roundings it passes through in order, its share of D, and e / d as
# vector.g writes it.
TERMS = (
    ("2 * w * v1 * n2", ("w", "v1", "n2"), ("q01", "c0", "m0", "a0", "out"),
     "10 * |w| * |v1|", "(w / |w|) * (v1 / |v1|) * n2 * (W1 / 5)"),
    ("-(2 * w * v2 * n1)", ("w", "v2", "n1"), ("q02", "c0", "m0", "a0", "out"),
     "10 * |w| * |v2|", "-((w / |w|) * (v2 / |v2|) * n1 * (W2 / 5))"),
    ("2 * v1 * v0 * n1", ("v1", "v0", "n1"), ("q21", "c2", "k01", "k0", "a0", "out"),
     "12 * |v1| * |v0|", "(v1 / |v1|) * (v0 / |v0|) * n1 * (W3 / 6)"),
    ("-(2 * v1 * v1 * n0)", ("v1", "n0"), ("q22", "c2", "k01", "k0", "a0", "out"),
     "12 * |v1| * |v1|", "-((v1 / |v1|) * (v1 / |v1|) * n0 * (W4 / 6))"),
    ("-(2 * v2 * v2 * n0)", ("v2", "n0"), ("q11", "c1", "k02", "k0", "a0", "out"),
     "12 * |v2| * |v2|", "-((v2 / |v2|) * (v2 / |v2|) * n0 * (W5 / 6))"),
    ("2 * v2 * v0 * n2", ("v2", "v0", "n2"), ("q12", "c1", "k02", "k0", "a0", "out"),
     "12 * |v2| * |v0|", "(v2 / |v2|) * (v0 / |v0|) * n2 * (W6 / 6)"),
)
N0_PATH = ("out",)


def cases():
    """Every zero set but those with three or four parts of the turn zero,
    which never reach Rodrigues' form or cannot keep q.q near one (464.34's 1)."""
    for n in range(len(FACTORS) + 1):
        for zero in itertools.combinations(FACTORS, n):
            if sum(f in zero for f in TURN) >= 3:
                continue
            yield frozenset(zero), state_of(set(zero))


def state_of(zero):
    """Each rounding's result, rounded or zero: zero where a factor of a
    product is zero, or both parts of a sum."""
    state = {}

    def is_zero(name):
        name = DOUBLED.get(name, name)
        if name in FACTORS:
            return name in zero
        return state[name] != "rounded"

    for result, _, arg, _, kind in OPS:
        a, _, b = arg.split(" ")
        za, zb = is_zero(a), is_zero(b)
        if kind == "product":
            state[result] = "zero" if za or zb else "rounded"
        else:
            state[result] = "zero" if za and zb else "rounded"
    return state


def used(zero):
    """The roundings on the path of a term that has no zero factor."""
    paths = [t[2] for t in TERMS if not set(t[1]) & zero]
    if "n0" not in zero:
        paths.append(N0_PATH)
    return {r for path in paths for r in path}


def name_of(zero):
    return "vm_" + ("".join(f for f in FACTORS if f in zero) or "n")


def products(zero, state):
    """The rounded products on a live path."""
    live_ops = used(zero)
    return [op for op in OPS if state[op[0]] == "rounded" and op[0] in live_ops and op[4] == "product"]


def make(zero, state, goal=GOAL):
    live_ops = used(zero)
    rounded = [op for op in OPS if state[op[0]] == "rounded" and op[0] in live_ops]
    live = [k for k, t in enumerate(TERMS) if not set(t[1]) & zero]
    gone = [k for k in range(len(TERMS)) if k not in live]
    n0_live = "n0" not in zero
    nothing = not live and not n0_live
    zeros = [f for f in FACTORS if f in zero]
    unrounded = [op[1] for op in OPS if state[op[0]] != "rounded"]
    rp = [op for op in rounded if op[4] == "product"]
    rs = [op for op in rounded if op[4] == "sum"]
    assert not live or rp, zero

    def listed(names):
        return ", ".join(names) if names else "none"

    src = [
        "# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567",
        "# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567",
        "# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee",
        "# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567",
        "# restates: Cargo.lock nalgebra at 4b642567",
        "# 464.20's vector.g, the turned vector's x, with zero factors " + listed(zeros) + ",",
        "# and so zero " + listed(unrounded) + ", each rounded product on a live",
        "# path normal, below the normals or 0, and each rounded sum on one free",
        "# to cancel.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The quaternion (w, v) and the vector n, each an f64, a zero factor the",
        "# double 0.",
    ]

    def inputs(names):
        return " ".join(f"{n} = 0;" if n in zero else f"{n} = rnd({n}_);" for n in names)

    src += [inputs(TURN), inputs(VECTOR), "",
            "# What apply_to_vector computes, line for line, a zero written unrounded,",
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
            "Ex = 2 * w * (v1 * n2 - v2 * n1)",
            "   + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2))",
            "   + n0;", ""]
    ratio = {op[0]: op[3] for op in rounded}

    def share(q):
        return f"g{q}" if AMOUNTS[q][1] else "Lo"

    def ratios_on(path):
        return [ratio[r] for r in reversed(path) if r in ratio]

    def chain(path):
        rs_ = ratios_on(path)
        return " * ".join(rs_) + " - 1" if rs_ else "0"

    def after(result):
        path = next(t[2] for k, t in enumerate(TERMS) if k in live and result in t[2])
        return ratios_on(path[path.index(result) + 1:])

    shares = [k + 1 for k in live] + ([7] if n0_live else [])
    if not nothing:
        src += ["# Each term of Ex as out carries it, less the term itself, through the",
                "# roundings left on its path. A term with a zero factor is zero."]
        src += [f"W{k + 1} = {chain(TERMS[k][2])};" for k in live]
        if n0_live:
            src.append(f"W7 = {chain(N0_PATH)};")
        for k in live:
            t = TERMS[k][0]
            src.append(f"e{k + 1} = -({t[2:-1]} * W{k + 1});" if t.startswith("-(") else f"e{k + 1} = {t} * W{k + 1};")
        if n0_live:
            src.append("e7 = n0 * W7;")

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
        hbody = f"m{q} * (1b-1022 / Lo){unit}{tail}"
        hints.append(f"A{q} / {share(q)} -> {sign}({hbody});" if sign else f"A{q} / {share(q)} -> {hbody};")
    if amounts:
        src += ["", "# What each rounded product's m * 2^-1022 comes to in out, through the",
                "# sums and products after it and the roundings left on its path."]
        src += amounts

    if not nothing:
        src += ["", "# Each live term's share of D, as vector.g's, with 1 for the vector's",
                "# component."]
        src += [f"d{k + 1} = {TERMS[k][3]};" for k in live]
        if n0_live:
            src.append("d7 = 1;")
        src += ["D = " + " + ".join(f"d{k}" for k in shares) + ";"]
    if rp:
        src += ["", "# Each amount's share of G, |f| Lo, written Lo itself where f is 1: Gappa",
                "# 1.8.0 aborts on a name defined as another name alone, as in g = Lo.",
                "# E is out - Ex less the amounts, the terms' errors alone."]
        src += [f"g{op[0]} = 2 * |{AMOUNTS[op[0]][1]}| * Lo;" for op in rp if AMOUNTS[op[0]][1]]
        src += ["G = " + " + ".join(share(op[0]) for op in rp) + ";",
                "Asum = " + " + ".join(f"A{op[0]}" for op in rp) + ";",
                "E = out - Ex - Asum;"]
    if not nothing:
        src += ["",
                "# q.q and the four squares D is written with below.",
                "N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;",
                "X = |w| - 25/93 * (|v1| + |v2|);",
                "Y = |v0| - 10/31 * (|v1| + |v2|);",
                "Z = |v1| - |v2|;"]
    else:
        src += ["", "# q.q.", "N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;"]

    n = {x: sum(AMOUNTS[op[0]][1] == x for op in rp) for x in ("w", "v1", "v2")}
    n_c = sum(AMOUNTS[op[0]][1] is None for op in rp)
    s = sum(c * c for c in n.values())
    lam = Fraction(math.sqrt(s)).limit_denominator(8) if s else None
    if lam is not None:
        src += ["", f"# G / Lo's linear part as squares, lam = {lam}."]
        src += [f"{SQUARES[x]} = |{x}| - {Fraction(c) / lam};" for x, c in n.items() if c]
    src.append("")

    nzt = [f for f in TURN if f not in zero]
    nzv = [f for f in VECTOR if f not in zero]
    hyp = ["  # Every part of the turn is a finite f64.",
           "     " + " /\\ ".join(f"|{f}| <= 1b1024" for f in nzt),
           "  # No factor of a product is zero but " + listed(zeros) + ".",
           "  /\\ " + " /\\ ".join(f"|{f}| >= 1b-1074" for f in nzt + nzv)]
    if rs:
        hyp += ["  # What sum.g proves of each rounded sum on a live path, the sum of two",
                "  # doubles, or 0 where it cancels, which any ratio meets."]
        hyp += ["  /\\ " + " /\\ ".join(f"{op[3]} in [{RATIO}]" for op in rs[k:k + 2]) for k in range(0, len(rs), 2)]
    if rp:
        hyp += ["  # What mixedn.g and mixeds.g prove of each rounded product on a live",
                "  # path, normal or not, and |P| / P in [-1, 1] for any P not zero."]
        hyp += ["  /\\ " + " /\\ ".join(f"m{op[0]} in [-1b-53, 1b-53]" for op in rp[k:k + 3]) for k in range(0, len(rp), 3)]
        hyp += ["  /\\ " + " /\\ ".join(f"s{op[0]} in [-1, 1]" for op in rp[k:k + 3]) for k in range(0, len(rp), 3)]
        hyp += ["  # Lo is the smallest normal number.", "  /\\ Lo in [1b-1022, 1b-1022]"]
    if nzv:
        hyp += ["  # Every component of the vector is at most one: a constructor's normal is",
                "  # the nearest f64 to a unit vector's components.",
                "  /\\ " + " /\\ ".join(f"{f} in [-1, 1]" for f in nzv)]
    hyp += ["  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and",
            "  # 6 epsilon is 12b-53.",
            "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]",
            "  # x / |x| is in [-1, 1] for any x that is not zero.",
            "  /\\ " + " /\\ ".join(f"{f} / |{f}| in [-1, 1]" for f in nzt)]
    err = "E" if rp else "(out - Ex)"
    queries = []
    if not nothing:
        queries.append(f"{err} / D in ? /\\ D in ?")
    if rp:
        queries.append("Asum / G in ? /\\ G / Lo in ?")
    concl = ["  " + " /\\ ".join(queries), f"  /\\ {goal}"] if queries else [f"  {goal}"]
    src += ["{"] + hyp + ["->"] + concl + ["}", ""]
    if rp:
        src += ["# out - Ex is the terms' error and the amounts, each averaged on its own.",
                "out - Ex -> E + Asum;",
                "# The amounts, each a fraction of its own share of G; add_rr averages them.",
                "Asum -> (Asum / G) * (G / Lo) * Lo;",
                "Asum / G -> ((G + Asum) - G) / G;",
                "G + Asum -> " + " + ".join(f"({share(op[0])} + A{op[0]})" for op in rp) + ";"]
        src += [f"({share(op[0])} + A{op[0]}) - {share(op[0])} -> A{op[0]};" for op in rp]
        src += hints
    if not nothing:
        src += ["# The terms' error is the sum of the live shares' errors, each a fraction",
                "# of its own share of D; add_rr then averages them.",
                f"{err.strip('()')} -> ({err} / D) * D;",
                f"{err} / D -> ((D + {err}) - D) / D;",
                f"D + {err} -> " + " + ".join(f"(d{k} + e{k})" for k in shares) + ";"]
        src += [f"(d{k} + e{k}) - d{k} -> e{k};" for k in shares]
        src += [f"e{k + 1} / d{k + 1} -> {TERMS[k][4]};" for k in live]
        if n0_live:
            src.append("e7 / d7 -> n0 * W7;")

        kept = [v for v in ("v1", "v2") if v not in zero]
        took = [TERMS[k][3] for k in gone] + [f"19/465 * |{v}| * |{v}|" for v in ("v1", "v2") if v in zero]
        one = " + 1" if n0_live else ""
        hint = ["D -> 93/5 * N4 - (93/5 * (X * X) + 93/5 * (Y * Y) + 305/93 * (Z * Z)"]
        if kept:
            hint.append("                 + 19/465 * (" + " + ".join(f"|{v}| * |{v}|" for v in kept) + "))" + one)
        else:
            hint[0] += ")" + one
        if took:
            hint.append("     - (" + " + ".join(took) + ")")
        src += ["", "# D is vector.g's 93/5 q.q less four squares, plus 1 for n0, less the",
                "# share of each term a zero factor took out, and a zero v's square."]
        src += hint[:-1] + [hint[-1] + ";"]
    if rp and lam is not None:
        constant = sum(Fraction(c * c) / lam for c in n.values()) + n_c
        squares = [f"{lam} * |v0| * |v0|"] + [
            f"{lam} * ({SQUARES[x]} * {SQUARES[x]})" if c else f"{lam} * |{x}| * |{x}|" for x, c in n.items()]
        src += ["", "# G / Lo is the amounts' shares, 2 n_w |w| + 2 n_1 |v1| + 2 n_2 |v2| + n_c,",
                "# written as lam q.q, a constant and squares.",
                f"G / Lo -> {lam} * N4 + {constant}",
                "        - (" + " + ".join(squares) + ");"]
    elif rp:
        src += ["", "# G / Lo is the amounts' shares, n_c Lo.", f"G / Lo -> {n_c};"]
    return "\n".join(src) + "\n"


def main():
    seen = []
    for zero, state in cases():
        name = name_of(zero)
        assert name not in seen, name
        seen.append(name)
        open(name + ".g", "w").write(make(zero, state))
        if used(zero):
            open("c_" + name + "_half.g", "w").write(make(zero, state, HALF))
    open("casesv.txt", "w").write("\n".join(seen) + "\n")
    print(len(seen), "cases,", sum(not products(z, s) for z, s in cases()), "with no rounded product on a live path,",
          sum(not used(z) for z, s in cases()), "with nothing live")


if __name__ == "__main__":
    main()
