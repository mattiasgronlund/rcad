"""Write SideOfAFlatSurfaceThroughAPoint::on_floats without its six
product_is_normal checks, in every case of where its six products land, as
464.18's planeall.py does for SideOfAFlatSurface's three.

Each of the six terms the fold adds is a rounded product of two doubles: a
through product enters negated, and -rnd(f * q) is rnd(f * -q). A term that
rounds to zero is left out of both folds (464.13's addzero.g), so a case is
the states of the terms left, in the order the fold adds them, and how many
rounded to zero. Each term left is in one of two states:

- n, normal: it rounds to at least 2^-1022, as product_is_normal lets through;
- s, below the normals and rounding to a double that is not zero, which
  product_is_normal refuses;

and each term that rounds to zero is z: a zero factor, which product_is_normal
lets through, or a product below the normals at most 2^-1075 (prodzero.g).
Each rounded sum either rounds, and is then at least 2^-1074, or cancels to
zero and is written unrounded; two cancelling sums are never next to each
other, since the sum after one is a term left alone, which is not zero.

fa_<states>[_c<set>].g for each case with an n term, <states> one letter per
term left, <set> naming each term whose sum cancels by its place among those
left; c_<name>_half.g, each again at B = u / 2, which must fail;
fa_none.g, every case with no n term at once, where eb cannot be normal, and
c_fa_none_noeb.g, that again without eb normal, which must fail.

Run as: python3 flatall.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

B_CODE = "B = 6b-53 + 1b-99;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HEAD = [
    "# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at eb125ed0",
    "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at eb125ed0",
    "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0",
    "# restates: crates/rcad-predicates/src/cascade.rs decide at eb125ed0",
    "# SideOfAFlatSurfaceThroughAPoint::on_floats without its six product_is_normal",
    "# checks: the sign of facing . point - facing . through, as six rounded",
    "# products of two doubles folded left to right.",
]


def cancellable(left):
    sums = list(range(1, left - 1))
    for n in range(len(sums) + 1):
        for c in itertools.combinations(sums, n):
            if all(b - a > 1 for a, b in zip(c, c[1:])):
                yield c


def fold(src, left, cancel):
    """Write the fold of t0 .. t<left-1> into y; return the sums that round."""
    rounded = []
    running = "t0"
    for k in range(1, left - 1):
        src.append(f"S{k} = {running} + t{k};")
        if k in cancel:
            src.append(f"s{k} = S{k};")
        else:
            src.append(f"s{k} = rnd(S{k});")
            rounded.append(k)
        running = f"s{k}"
    src.append(f"y = {running} + t{left - 1};" if left >= 2 else "y = t0;")
    return rounded


def permanent(left):
    perm = "|t0|"
    for k in range(1, left):
        perm = f"rnd({perm} + |t{k}|)"
    return perm


def hypotheses(src, hyps):
    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            if row:
                src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
                first = False
    src.append("->")


def make(states, cancel=(), b=B_CODE):
    left = len(states)
    zeros = 6 - left
    small = [k for k in range(left) if states[k] == "s"]
    normal = [k for k in range(left) if states[k] == "n"]
    src = list(HEAD)
    src.append(f"# The terms left, in the order the fold adds them: {states}"
               + (f"; {zeros} more round{'s' if zeros == 1 else ''} to zero." if zeros else "."))
    if cancel:
        src.append(f"# The sum{'s' if len(cancel) > 1 else ''} adding term{'s' if len(cancel) > 1 else ''} "
                   f"{', '.join(map(str, cancel))} cancel{'' if len(cancel) > 1 else 's'} to zero, which rounds to itself.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The two factors of each term left, each an f64.",
        " ".join(f"a{k} = rnd(a{k}_); b{k} = rnd(b{k}_);" for k in range(left)),
    ]
    if zeros:
        src += ["# The two factors of each term that rounds to zero.",
                " ".join(f"g{j} = rnd(g{j}_); h{j} = rnd(h{j}_);" for j in range(zeros))]
    src += [
        "",
        "# What on_floats computes, line for line. The fold starts at 0.0, and",
        "# adding 0.0 or a term that rounded to zero to a double gives that double",
        "# back (464.13's addzero.g), so those and the first addition are left out.",
        " ".join(f"X{k} = a{k} * b{k};" for k in range(left)),
    ]
    if zeros:
        src.append(" ".join(f"Z{j} = g{j} * h{j};" for j in range(zeros)))
    src.append(" ".join(f"t{k} = rnd(X{k});" for k in range(left)))
    rounded = fold(src, left, cancel)
    src += ["", "# The permanent folds the same terms' magnitudes in the same order.",
            f"P = {permanent(left)};", "", b, "eb = rnd(B * P);", "",
            "# The true value of the same inputs."]
    src.append("Ex = " + " + ".join([f"X{k}" for k in range(left)] + [f"Z{j}" for j in range(zeros)]) + ";")
    src += ["", "# The permanent with no rounding of its own.",
            "Phat = " + " + ".join(f"|t{k}|" for k in range(left)) + ";"]
    src += ["", "# What each rounding multiplied its argument by."]
    if normal:
        src.append(" ".join(f"rp{k} = t{k} / X{k};" for k in normal))
    if rounded:
        src.append(" ".join(f"rs{k} = s{k} / S{k};" for k in rounded))
    src += ["", "# Each term as y carries it less the term: e the part that is a fraction",
            "# of the term as on_floats rounds it, with W that fraction, and for a term",
            "# below the normals d what its own rounding cost, an amount."]
    for k in range(left):
        carry = [f"rs{j}" for j in rounded if j >= k]
        carried = " * ".join(carry) if carry else "1"
        src.append(f"W{k} = {carried} - 1 / rp{k};" if k in normal else f"W{k} = {carried} - 1;")
    src.append(" ".join(f"e{k} = t{k} * W{k};" for k in range(left)))
    src.append(" ".join(f"c{k} = |t{k}|;" for k in range(left)))
    if small:
        src.append(" ".join(f"d{k} = t{k} - X{k};" for k in small))
    amount = " + ".join([f"d{k}" for k in small]) + "".join(f" - Z{j}" for j in range(zeros))
    if not small and zeros:
        amount = "-" + " - ".join(f"Z{j}" for j in range(zeros))
    if amount:
        src.append(f"A = {amount};")

    hyps = [("Every input is a finite f64.",
             [[f"|a{k}| <= 1b1024 /\\ |b{k}| <= 1b1024" for k in range(left)],
              [f"|g{j}| <= 1b1024 /\\ |h{j}| <= 1b1024" for j in range(zeros)]]),
            ("No factor of a normal term is zero.",
             [[f"|a{k}| >= 1b-1074 /\\ |b{k}| >= 1b-1074" for k in normal]])]
    if rounded:
        hyps.append(("No rounded sum cancels to zero.", [[f"|S{k}| >= 1b-1074" for k in rounded]]))
    if small:
        hyps.append(("The terms below the normals that round to a double that is not zero.",
                     [[f"|X{k}| <= 1b-1022 /\\ |t{k}| >= 1b-1074" for k in small]]))
    if zeros:
        hyps.append(("The terms that round to zero (prodzero.g).",
                     [[f"|Z{j}| <= 1b-1075" for j in range(zeros)]]))
    hyps += [("The normal terms, as product_is_normal has them.", [[f"|t{k}| >= 1b-1022" for k in normal]]),
             ("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]),
             ("What lemma.g proves each of those gives.",
              [[f"t{k} -/ X{k} {LEMMA}"] for k in normal] + [[f"eb -/ B * P {LEMMA}"]]),
             ("x / |x| is in [-1, 1] for any x that is not zero.",
              [[f"t{k} / |t{k}| in [-1, 1]" for k in range(left)]])]
    hypotheses(src, hyps)
    esum = " + ".join(f"e{k}" for k in range(left))
    src.append(f"  ({esum}) / Phat in ? /\\ P -/ Phat in ?" + (" /\\ A in ?" if amount else ""))
    src += ["  /\\ (y - Ex) / eb in [-1, 1]", "}", "", "# How far each rounding is from the identity."]
    for k in normal:
        src.append(f"t{k} / X{k} -> 1 + (t{k} - X{k}) / X{k};")
    for k in rounded:
        src.append(f"s{k} / S{k} -> 1 + (s{k} - S{k}) / S{k};")
    src += [
        "",
        "# The relative part is averaged over the permanent as in flat.g; what",
        "# rounding a term below the normals cost, and each term that rounded to",
        "# zero, is an amount, and eb being normal keeps it small against eb.",
        f"(y - Ex) / eb -> (({esum}) / Phat) * (Phat / P) * (P / eb)" + (" + A / eb;" if amount else ";"),
        f"({esum}) / Phat -> ((Phat + ({esum})) - Phat) / Phat;",
        f"Phat + ({esum}) -> " + " + ".join(f"(c{k} + e{k})" for k in range(left)) + ";",
    ]
    for k in range(left):
        src.append(f"(c{k} + e{k}) - c{k} -> e{k};")
    for k in range(left):
        src.append(f"e{k} / c{k} -> (t{k} / |t{k}|) * W{k};")
    src += [
        "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
        "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
    ]
    return "\n".join(src) + "\n"


def make_none(eb_normal=True):
    """Every case with no normal term: each of the six rounds below 2^-1022 or
    to zero, so each product is at most 2^-1022, the permanent at most about
    six times that, and eb far below the normals."""
    src = list(HEAD)
    src += ["# No term is normal: each product is at most 2^-1022, and may be zero.",
            "@rnd = float<ieee_64, ne>;", "",
            " ".join(f"a{k} = rnd(a{k}_); b{k} = rnd(b{k}_);" for k in range(6)), "",
            "# What on_floats computes, every term in the fold.",
            " ".join(f"X{k} = a{k} * b{k};" for k in range(6)),
            " ".join(f"t{k} = rnd(X{k});" for k in range(6))]
    fold(src, 6, ())
    src += ["", f"P = {permanent(6)};", "", B_CODE, "eb = rnd(B * P);", "",
            "Ex = " + " + ".join(f"X{k}" for k in range(6)) + ";"]
    hyps = [("Every input is a finite f64.", [[f"|a{k}| <= 1b1024 /\\ |b{k}| <= 1b1024" for k in range(6)]]),
            ("No term is normal.", [[f"|X{k}| <= 1b-1022" for k in range(6)]])]
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    hypotheses(src, hyps)
    src += ["  eb in ? /\\ P in ?", "  /\\ (y - Ex) / eb in [-1, 1]", "}"]
    return "\n".join(src) + "\n"


def name_of(states, cancel):
    return f"fa_{states}" + (f"_c{''.join(map(str, cancel))}" if cancel else "")


if __name__ == "__main__":
    proving = []
    for left in range(1, 7):
        for states in ("".join(s) for s in itertools.product("ns", repeat=left)):
            if "n" not in states:
                continue
            for cancel in cancellable(left):
                name = name_of(states, cancel)
                open(f"{name}.g", "w").write(make(states, cancel))
                open(f"c_{name}_half.g", "w").write(make(states, cancel, b=B_HALF))
                proving.append(name)
    open("fa_none.g", "w").write(make_none())
    open("c_fa_none_noeb.g", "w").write(make_none(eb_normal=False))
    print(f"{len(proving)} cases with a normal term, each with a control at u / 2, and fa_none.g")
