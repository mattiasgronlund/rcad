"""Write signed.g's and shiftplane.g's cases with every product anywhere, as
464.18's planeall.py did for plane.g. A case is <kind>_<state>[_u<ops>][_big].g:

- kind: signed, for Plane::signed_distance_to, or shift, for a plane's or a
  half-space's offset after a shift with no turn;
- state: one letter a product, n its double at least 2^-1022, s the product at
  most 2^-1022 with neither factor zero, z a factor zero;
- _u<ops>: the rounded operations written unrounded, each named by the
  products its sum has added, y for value. Written unrounded it covers the
  sum's cancelling to zero, since zero rounds to itself, and any input where it
  rounds to itself; rounded, the case asks it to be at least 2^-1074, which a
  sum of two doubles that is not zero is;
- _big: the first sum rounded and at least 2^-1021. It replaces the plain
  rounded first sum where shares alone do not reach the code's bound; below
  2^-1021 that sum rounds to itself, which its _u case covers.

A zero product leaves the fold, since adding zero to a double gives the double
back (464.13's addzero.g). A sum of two products both s is unrounded only: it is
at most 2^-1021, a multiple of 2^-1074, so a double (sumexact.g).

The error is each term's error and what rounding each s product cost, its
amount a<k>, which is at most 2^-1075 times what rounds after it. 464.9's
averaging gives each term a share of the bound D: w/4 of |n_i| L for a product
that meets w roundings, w/4 of L for the offset, and each amount a quarter of
a payer's share, the payer being L itself (L at least 2^-1022), a normal
product (its |n_i| L at least 2^-1022, give or take a rounding), or, in a _big
case, (|n_a| + |n_b|) L for the first sum's two products, which that sum keeps
at least 2^-1021 (1 - 2u). The payers are chosen to make D smallest, and of
two choices with the same D, the one that takes less out of L.

Run as: python3 signedall.py; it writes beside itself and prints one line a
case: its name, its shares and the bound the averaging reaches in u."""
import itertools
import math
import os
from fractions import Fraction as F

os.chdir(os.path.dirname(os.path.abspath(__file__)))

LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
EPS = F(1, 2**52)
TRIANGLE = "4503599627370495b-1073"

RESTATES = {
    "signed": [
        "# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e",
        "# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e",
        "# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e",
        "# restates: Cargo.lock nalgebra at c3b2160e",
    ],
    "shift": [
        "# restates: crates/rcad-geom/src/plane.rs Plane::transformed at c3b2160e",
        "# restates: crates/rcad-geom/src/plane.rs Plane::holding at c3b2160e",
        "# restates: crates/rcad-geom/src/half_space.rs HalfSpace::shifted at c3b2160e",
        "# restates: crates/rcad-geom/src/moved_corner.rs SURFACE_AFTER_SHIFTING_ROUNDOFF at c3b2160e",
        "# restates: crates/rcad-geom/src/isometry.rs Isometry::apply_to_vector at c3b2160e",
        "# restates: crates/rcad-geom/src/isometry.rs Isometry::translation_part at c3b2160e",
        "# restates: crates/rcad-geom/src/isometry.rs turned_by at c3b2160e",
        "# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::named_by at c3b2160e",
        "# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::turned at c3b2160e",
        "# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e",
        "# restates: Cargo.lock nalgebra at c3b2160e",
    ],
}

KIND = {
    "signed": {
        "q": "p",
        "size": "L",
        "limit": 8,
        "about": "Plane::signed_distance_to, normal.dot(p) - offset",
        "goal": "(value - Ex) / L in [-1b-50, 1b-50]",
        "goal_lhs": "(value - Ex) / L",
    },
    "shift": {
        "q": "d",
        "size": "(N * L)",
        "limit": 16,
        "about": "a plane's or a half-space's offset after a shift, offset + normal.dot(d)",
        "goal": "(value - Ex) / (N * L) in [-1b-49, 1b-49]",
        "goal_lhs": "(value - Ex) / (N * L)",
    },
}


def ops(live):
    """The rounded operations of the fold, in order: each sum after the first
    live product, named by the products it has added, then value, y."""
    return ["".join(map(str, live[: j + 1])) for j in range(1, len(live))] + ["y"]


def after(live, pos, rounded):
    """The rounded operations product live[pos] meets after its own rounding."""
    return [o for o in ops(live)[max(pos, 1) - 1 :] if o in rounded]


def factor(o):
    return "rv" if o == "y" else f"rs{o}"


def cap(squares, signed):
    """The least k with (k / 1024)^2 at least squares, times 1 + 64 epsilon
    for a stored normal's n.n."""
    target = squares * (1 + 64 * EPS) if signed else squares
    k = math.isqrt(int(target * 1024 * 1024))
    while F(k * k, 1024 * 1024) < target:
        k += 1
    return k


def plan(kind, st, live, rounded, big):
    """Each term's weight and the payer of each amount that makes D least."""
    w = {}
    for pos, i in enumerate(live):
        w[i] = len(after(live, pos, rounded)) + (1 if st[i] == "n" else 0)
    v = 1 if "y" in rounded else 0
    subs = [i for i in live if st[i] == "s"]
    payers = ["L"] + [i for i in live if st[i] == "n"] + (["F"] if big else [])
    best = None
    for choice in itertools.product(payers, repeat=len(subs)):
        pay = dict(zip(subs, choice))
        m = {p: sum(1 for k in subs if pay[k] == p) for p in payers}
        omega = {i: F(w[i] + m.get(i, 0), 4) for i in live}
        if big:
            for i in live[:2]:
                omega[i] += F(m["F"], 8)
        k = cap(sum(o * o for o in omega.values()), kind == "signed")
        c3 = F(v + m["L"], 4)
        bound = 4 * (F(k, 1024) + c3)
        # Of two plans with the same D, the one that takes less out of L, so
        # that fewer cases need L at least 2^-1022.
        if best is None or (bound, m["L"]) < (best[0], best[2]["L"]):
            best = (bound, pay, m, omega, k, c3)
    return w, v, best


def make(kind, st, unrounded, big, goal=None, anyl=False):
    s = KIND[kind]
    q = s["q"]
    size = s["size"]
    live = [i for i in (0, 1, 2) if st[i] != "z"]
    rounded = [o for o in ops(live) if o not in unrounded]
    w, v, (bound, pay, m, omega, k, c3) = plan(kind, st, live, rounded, big)
    subs = [i for i in live if st[i] == "s"]
    normals = [i for i in live if st[i] == "n"]
    first = ops(live)[0] if len(live) >= 2 else None
    word = {"n": "normal", "s": "below the normals", "z": "zero"}
    lines = RESTATES[kind] + [f"# {s['about']}, in the case:"]
    for i in (0, 1, 2):
        lines.append(f"# product {i} {word[st[i]]}.")
    if unrounded:
        lines.append(f"# Written unrounded, which covers each cancelling to zero: {', '.join(unrounded)}.")
    if big:
        lines.append(f"# The first sum, S{first}, at least 2^-1021.")
    lines += ["@rnd = float<ieee_64, ne>;", "", "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);"]
    lines.append(" ".join(f"{q}{i} = rnd({q}{i}_);" for i in live))
    lines.append("off = rnd(off_);")
    lines.append("")
    lines.append(" ".join(f"X{i} = n{i} * {q}{i};" for i in live))
    lines.append(" ".join(f"t{i} = rnd(X{i});" for i in live))
    run = f"t{live[0]}"
    for j, o in enumerate(ops(live)[:-1]):
        nxt = live[j + 1]
        lines.append(f"S{o} = {run} + t{nxt};")
        lines.append(f"s{o} = rnd(S{o});" if o in rounded else f"s{o} = S{o};")
        run = f"s{o}"
    lines.append(f"y = {run} - off;" if kind == "signed" else f"y = off + {run};")
    lines.append("value = rnd(y);" if "y" in rounded else "value = y;")
    lines.append("")
    xs = " + ".join(f"X{i}" for i in live)
    lines.append(f"Ex = {xs} - off;" if kind == "signed" else f"Ex = off + {xs};")
    if kind == "shift":
        lines.append("N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);")
    lines.append("")
    defs = [f"rp{i} = t{i} / X{i};" for i in normals]
    defs += [f"{factor(o)} = s{o} / S{o};" for o in rounded if o != "y"]
    if "y" in rounded:
        defs.append("rv = value / y;")
    if defs:
        lines.append(" ".join(defs))
    lines.append("")
    # Terms and amounts.
    e = {}
    a = {}
    for pos, i in enumerate(live):
        fs = [factor(o) for o in after(live, pos, rounded)]
        prod = " * ".join(fs)
        if st[i] == "n":
            lines.append(f"W{i} = {prod + ' * ' if prod else ''}rp{i} - 1;")
            e[i] = f"e{i}"
        else:
            if fs:
                lines.append(f"W{i} = {prod} - 1;")
                e[i] = f"e{i}"
            lines.append(f"a{i} = (t{i} - X{i})" + (f" * ({prod});" if prod else ";"))
            a[i] = f"a{i}"
    if v:
        lines.append("W3 = rv - 1;")
    for i in live:
        if i in e:
            lines.append(f"e{i} = X{i} * W{i};")
    if v:
        lines.append("e3 = -(off * W3);" if kind == "signed" else "e3 = off * W3;")
    lines.append("")
    # Shares, in the order D adds them.
    shares = []
    for i in live:
        weight = w[i] + m.get(i, 0)
        if weight == 0:
            continue
        terms = ([e[i]] if i in e else []) + [a[k] for k in subs if pay[k] == i]
        shares.append((f"c{i}", f"{weight}b-2 * (|n{i}| * L)", terms, ("p", i, weight)))
    if v + m["L"]:
        terms = (["e3"] if v else []) + [a[k] for k in subs if pay[k] == "L"]
        shares.append(("c3", f"{v + m['L']}b-2 * {size}", terms, ("L", None, v + m["L"])))
    if big and m["F"]:
        ia, ib = live[0], live[1]
        terms = [a[k] for k in subs if pay[k] == "F"]
        shares.append(("c4", f"{m['F']}b-3 * ((|n{ia}| + |n{ib}|) * L)", terms, ("F", None, m["F"])))
    lines.append(" ".join(f"{n} = {d};" for n, d, _, _ in shares))
    lines.append("D = " + " + ".join(n for n, _, _, _ in shares) + ";")
    lines.append("")
    weighted = " + ".join(f"{int(omega[i] * 8)}b-3 * |n{i}|" for i in live if omega[i])
    hyp = ["{",
           "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024",
           "  /\\ " + " /\\ ".join(f"|{q}{i}| <= 1b1024" for i in live) + " /\\ |off| <= 1b1024",
           "  # No factor of a product left is zero.",
           "  /\\ " + " /\\ ".join(f"|n{i}| >= 1b-1074 /\\ |{q}{i}| >= 1b-1074" for i in live)]
    sums = [o for o in rounded if o != "y"]
    if sums or v:
        hyp.append("  # No rounded sum cancels to zero.")
        hyp.append("  /\\ " + " /\\ ".join(
            [f"|S{o}| >= {'1b-1021' if big and o == first else '1b-1074'}" for o in sums]
            + (["|y| >= 1b-1074"] if v else [])))
    if normals:
        hyp.append("  # The normal products, and what lemma.g proves each gives.")
        hyp.append("  /\\ " + " /\\ ".join(f"|t{i}| >= 1b-1022" for i in normals))
        for i in normals:
            hyp.append(f"  /\\ t{i} -/ X{i} {LEMMA}")
    if subs:
        hyp.append("  # The products below the normals.")
        hyp.append("  /\\ " + " /\\ ".join(f"|X{i}| <= 1b-1022" for i in subs))
    if kind == "signed":
        hyp.append("  # L is at least 2^-1022, and at least every coordinate and the offset.")
        hyp.append("  /\\ L in [" + ("1b-1074" if anyl else "1b-1022") + ", 1b1024]")
    else:
        hyp.append("  # |n| L is at least 2^-1022, L at least every coordinate of the shift")
        hyp.append("  # and the offset over |n|.")
        hyp.append("  /\\ L in [1b-1074, 1b1024] /\\ N * L in [" + ("1b-1100" if anyl else "1b-1022") + ", 1b1030]")
    hyp.append("  /\\ " + " /\\ ".join(f"{q}{i} / L in [-1, 1]" for i in live)
               + (" /\\ off / L in [-1, 1]" if kind == "signed" else " /\\ off / (N * L) in [-1, 1]"))
    total = sum(o * o for o in omega.values())
    if not weighted:
        hyp.append("  # Every product is below the normals and every operation exact: the")
        hyp.append("  # amounts are all D holds.")
    elif kind == "signed":
        hyp.append(f"  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:")
        hyp.append(f"  # the weights' squares add to {total}, and sqrt({total} (1 + 64 epsilon)) <= {k} / 1024.")
        hyp.append(f"  /\\ {weighted} in [0, {k}b-10]")
    else:
        hyp.append(f"  # Cauchy-Schwarz: the weights' squares add to {total}, and sqrt({total}) <= {k} / 1024.")
        hyp.append(f"  /\\ ({weighted}) / N in [0, {k}b-10]")
    if big:
        ia, ib = live[0], live[1]
        hyp.append(f"  # |S{first}| <= (1 + u) (|n{ia}| + |n{ib}|) L + 2^-1075, the products being")
        hyp.append("  # normal and below the normals, and |S| at least 2^-1021.")
        hyp.append(f"  /\\ (|n{ia}| + |n{ib}|) * L >= {TRIANGLE}")
    hyp.append("  /\\ " + " /\\ ".join(f"n{i} / |n{i}| in [-1, 1]" for i in live))
    hyp.append("->")
    hyp.append("  " + " /\\ ".join([f"a{i} in ?" for i in subs] + ["(value - Ex) / D in ?"]))
    hyp.append("  /\\ " + (goal or s["goal"]))
    hyp.append("}")
    lines += hyp
    lines.append("")
    for i in normals:
        lines.append(f"t{i} / X{i} -> 1 + (t{i} - X{i}) / X{i};")
    for o in sums:
        lines.append(f"s{o} / S{o} -> 1 + (s{o} - S{o}) / S{o};")
    if v:
        lines.append("value / y -> 1 + (value - y) / y;")
    lines.append("")
    lines.append(f"{s['goal_lhs']} -> ((value - Ex) / D) * (D / {size});")
    if not weighted:
        lines.append(f"D / {size} -> {c3};")
    elif kind == "signed":
        lines.append(f"D / L -> {weighted} + {c3};")
    else:
        lines.append(f"D / (N * L) -> ({weighted}) / N + {c3};")
    lines.append("(value - Ex) / D -> ((D + (value - Ex)) - D) / D;")

    def grp(terms):
        return terms[0] if len(terms) == 1 else "(" + " + ".join(terms) + ")"

    lines.append("D + (value - Ex) -> " + " + ".join(f"({n} + {grp(t)})" for n, _, t, _ in shares) + ";")
    for n, _, t, _ in shares:
        lines.append(f"({n} + {grp(t)}) - {n} -> {' + '.join(t)};")
    for n, _, t, (who, i, weight) in shares:
        parts = []
        if who == "p":
            for term in t:
                if term.startswith("e"):
                    parts.append(f"4 / {weight} * ((n{i} / |n{i}|) * ({q}{i} / L) * W{i})")
                else:
                    parts.append(f"4 / {weight} * ({term} / (|n{i}| * L))")
        elif who == "L":
            for term in t:
                if term == "e3":
                    parts.append(f"4 / {weight} * (-((off / L) * W3))" if kind == "signed"
                                 else f"4 / {weight} * ((off / (N * L)) * W3)")
                else:
                    parts.append(f"4 / {weight} * ({term} / {size})")
        else:
            ia, ib = live[0], live[1]
            for term in t:
                parts.append(f"8 / {weight} * ({term} / ((|n{ia}| + |n{ib}|) * L))")
        lines.append(f"{grp(t)} / {n} -> {' + '.join(parts)};")
    for k2 in subs:
        p = pay[k2]
        if p not in ("L", "F"):
            lines.append(f"a{k2} / (|n{p}| * L) -> (a{k2} / t{p}) * rp{p} * ((n{p} / |n{p}|) * ({q}{p} / L));")
    return "\n".join(lines) + "\n", bound, shares, pay


def cases():
    for kind in ("signed", "shift"):
        for st in itertools.product("nsz", repeat=3):
            st = "".join(st)
            live = [i for i in (0, 1, 2) if st[i] != "z"]
            if not live:
                continue
            names = ops(live)
            must = [names[0]] if len(live) >= 2 and st[live[0]] == "s" and st[live[1]] == "s" else []
            for r in range(len(names) + 1):
                for unrounded in itertools.combinations(names, r):
                    if any(o not in unrounded for o in must):
                        continue
                    rounded = [o for o in names if o not in unrounded]
                    _, _, best = plan(kind, st, live, rounded, False)
                    big = False
                    if best[0] >= KIND[kind]["limit"] - F(1, 16):
                        first = names[0]
                        mixed = len(live) >= 2 and {st[live[0]], st[live[1]]} == {"n", "s"}
                        assert first in rounded and mixed, (kind, st, unrounded, best[0])
                        big = True
                    yield kind, st, list(unrounded), big


def name_of(kind, st, unrounded, big):
    return f"{kind}_{st}" + (f"_u{''.join(unrounded)}" if unrounded else "") + ("_big" if big else "")


if __name__ == "__main__":
    for kind, st, unrounded, big in cases():
        name = name_of(kind, st, unrounded, big)
        src, bound, shares, pay = make(kind, st, unrounded, big)
        open(f"{name}.g", "w").write(src)
        print(name, " ".join(f"{n}={d}" for n, d, _, _ in shares), f"bound {float(bound):.3f}u")
