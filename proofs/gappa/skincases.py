"""Write skin.g, HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND
line for line, and its cases and controls beside it. A case is four letters,
one for each of the gaps x, y, z and the radius r, saying what its square is:
n normal; z of a zero factor, which product_is_normal lets through; s below
the normals and rounding to a double that is not zero, and u below the
normals and rounding to zero, both of which product_is_normal refuses. skin.g
is the case nnnn. It writes:

- skinv.g, the claim on value, the double on_floats returns, not on y;
- skin_<case>.g for every other case, and skinv_<case>.g for each with an n;
- c_<name>_half.g, each script with an n at B = u / 2, which must fail;
- c_skin_<case>_noeb.g, each case with no n without eb normal. With no normal
  square P is too small for eb to be normal, so such a case proves from that
  contradiction, as 464.12's sphere_zxyzr.g does, and without it must fail;
- c_skin_nonormal.g, skin.g without product_is_normal, and skin_noeb.g,
  skin.g without eb normal, which proves (464.14's 7).

A case of n and z alone is 464.12's script for the same zero set, but for its
first lines and B. A case with an s or a u carries what rounding that square
cost as an amount, as 464.8's planecases.py does.

Run as: python3 skincases.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

AXES = ("x", "y", "z")
TERMS = AXES + ("r",)
B_CODE = "B = 6b-53 + 3b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
SAYS = {"z": "is zero",
        "s": "has a square below the normals that rounds to a double, not to zero",
        "u": "has a square below the normals that rounds to zero"}


def make(case="nnnn", on_value=False, b=B_CODE, normal=True, eb_normal=True):
    st = dict(zip(TERMS, case))
    gaps = [a for a in AXES if st[a] != "z"]
    radius = st["r"] != "z"
    flat = [a for a in TERMS if st[a] in "zu"]
    live = [a for a in TERMS if st[a] in "ns"]
    small = [a for a in TERMS if st[a] in "su"]
    norm = [a for a in TERMS if st[a] == "n"]
    out = "value" if on_value else "y"
    sq = {a: f"s{a}" for a in AXES}
    sq["r"] = "rr"
    part = {a: f"X{a}" if st[a] == "s" else sq[a] for a in TERMS}

    src = [
        "# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66",
        "# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66",
        "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.",
    ]
    if on_value:
        src.append("# The claim is on value, the double on_floats returns, not on y.")
    if small:
        src.append("# A case product_is_normal refuses, which on_floats meets only without")
        src.append("# its check. Here:")
        for a in TERMS:
            if st[a] != "n":
                who = "the radius" if a == "r" else f"the {a} gap"
                src.append(f"#   {who} {SAYS[st[a]]};")
        src[-1] = src[-1][:-1] + "."
    elif case != "nnnn":
        names = [f"the {a} gap" for a in AXES if st[a] == "z"] + (["the radius"] if not radius else [])
        src.append(f"# The case where {' and '.join(names)} {'is' if len(names) == 1 else 'are'} zero.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre and the radius, each an f64. A gap is zero when",
        "# the centre's coordinate is the same double as the point's (464.11's",
        "# diffzero.g), and a zero radius is the double 0.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        " ".join(f"c{a} = rnd(c{a}_);" if a in gaps else f"c{a} = p{a};" for a in AXES),
        "r = rnd(r_);" if radius else "r = 0;",
        "",
        "# What on_floats computes, line for line. A zero, a square of one and a",
        "# sum of two round to themselves (464.11's refl.g), so they are left",
        "# unrounded.",
    ]
    if small:
        src += ["# A square below the normals is X. One that rounds to zero is written 0,",
                "# which tinysq.g proves rnd(X) is."]
    src.append(" ".join(f"G{a} = p{a} - c{a};" for a in AXES))
    src.append(" ".join(f"g{a} = rnd(G{a});" if a in gaps else f"g{a} = G{a};" for a in AXES))
    xs = [f"X{a} = g{a} * g{a};" for a in AXES if st[a] in "su"]
    if xs:
        src.append(" ".join(xs))
    square = {"n": "rnd(g{a} * g{a})", "z": "g{a} * g{a}", "s": "rnd(X{a})", "u": "0"}
    src.append(" ".join(f"s{a} = {square[st[a]].format(a=a)};" for a in AXES))
    src.append({"n": "rr = rnd(r * r);", "z": "rr = r * r;",
                "s": "Xr = r * r; rr = rnd(Xr);", "u": "Xr = r * r; rr = 0;"}[st["r"]])
    s01_rounds = not ("x" in flat and "y" in flat)
    s012_rounds = not all(a in flat for a in AXES)
    src += [
        "S01 = sx + sy;",
        "s01 = rnd(S01);" if s01_rounds else "s01 = S01;",
        "S012 = s01 + sz;",
        "s012 = rnd(S012);" if s012_rounds else "s012 = S012;",
        "y = s012 - rr;",
        "value = rnd(y);",
        "",
        "# The permanent sums the same three squares in the same order as value",
        "# does, so its first two sums are s01 and s012.",
        "P = rnd(s012 + rr);",
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;",
        "",
        "# The permanent with no rounding of its own.",
        "Phat = sx + sy + sz + rr;",
    ]
    if small and norm:
        src += ["# The same with X in place of each square below the normals that does",
                "# not round to zero, and without those that do.",
                "Phatx = " + " + ".join(part[a] for a in live) + ";"]
    if norm:
        src += ["", "# What each rounding multiplied its argument by."]
        if gaps:
            src.append(" ".join(f"rg{a} = g{a} / G{a};" for a in gaps))
        if [a for a in AXES if st[a] == "n"]:
            src.append(" ".join(f"rp{a} = s{a} / (g{a} * g{a});" for a in AXES if st[a] == "n"))
        if st["r"] == "n":
            src.append("rpr = rr / (r * r);")
        rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rs012 = s012 / S012;"] if s012_rounds else [])
        if rs:
            src.append(" ".join(rs))
        if on_value:
            src.append("rv = value / y;")
        v = " * rv" if on_value else ""
        carry = {"x": "rs01 * rs012" + v, "y": "rs01 * rs012" + v, "z": "rs012" + v,
                 "r": "rv" if on_value else "1"}
        src += ["", f"# What each of the four terms, as {out} carries it, is off by: e is that",
                "# amount and W the same over the term's square. The radius's term is",
                "# subtracted, so its W is the other way round."]
        if small:
            src += ["# For a square below the normals e and W are of X, and d is what",
                    "# rounding X cost, as an amount. One that rounds to zero is off by",
                    "# its whole true square, which is its d."]
        for a in AXES:
            if st[a] == "n":
                src.append(f"W{a} = {carry[a]} - 1 / (rg{a} * rg{a} * rp{a});")
            elif st[a] == "s":
                src.append(f"W{a} = {carry[a]} - 1 / (rg{a} * rg{a});")
        if st["r"] == "n":
            src.append(f"Wr = 1 / rpr - {carry['r']};")
        elif st["r"] == "s":
            src.append(f"Wr = 1 - {carry['r']};")
        src.append(" ".join(f"e{a} = {part[a]} * W{a};" for a in live))
        ds = []
        for a in AXES:
            if st[a] == "s":
                ds.append(f"d{a} = (s{a} - X{a}) * " + (f"({carry[a]});" if " " in carry[a] else f"{carry[a]};"))
            elif st[a] == "u":
                ds.append(f"d{a} = -(G{a} * G{a});")
        if st["r"] == "s":
            ds.append("dr = Xr - rr;" if carry["r"] == "1" else f"dr = (Xr - rr) * {carry['r']};")
        elif st["r"] == "u":
            ds.append("dr = Xr;")
        if ds:
            src.append(" ".join(ds))

    hyps = []
    finite = [f"|p{a}| <= 1b1024" for a in AXES] + [f"|c{a}| <= 1b1024" for a in gaps]
    if radius:
        finite.append("|r| <= 1b1024")
    hyps.append(("Every input is a finite f64.", [finite[:3], finite[3:]]))
    if gaps or radius:
        nonzero = [f"|g{a}| >= 1b-1074" for a in gaps] + (["|r| >= 1b-1074"] if radius else [])
        hyps.append(("No factor of the squares left is zero. For an f64 that is the same\n"
                     "  # as being at least the smallest subnormal.", [nonzero]))
        if normal and norm:
            hyps.append(("product_is_normal on the squares that pass it." if small else
                         "product_is_normal on the squares left.",
                         [[f"|{sq[a]}| >= 1b-1022" for a in norm]]))
    if small:
        rows = []
        for a in small:
            factor = "r" if a == "r" else f"g{a}"
            if st[a] == "s":
                rows.append([f"X{a} in [1b-2148, 1b-1022]", f"{sq[a]} >= 1b-1074", f"|{factor}| <= 1b-511"])
            else:
                rows.append([f"X{a} in [1b-2148, 1b-1075]", f"|{factor}| <= 1b-511"])
        hyps.append(("Each square below the normals. Of two doubles that are not zero it is\n"
                     "  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and\n"
                     "  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and\n"
                     "  # above, to at least 2^-1074.", rows))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = []
    if normal:
        lemma += [[f"s{a} -/ g{a} * g{a} {LEMMA}"] for a in AXES if st[a] == "n"]
        if st["r"] == "n":
            lemma.append([f"rr -/ r * r {LEMMA}"])
    if eb_normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if on_value and norm:
        hyps.append(("y is not zero. Where it is, value is zero too, and the claim on y is\n"
                     "  # the claim on value.", [["|y| >= 1b-1074"]]))

    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            if not row:
                continue
            src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
            first = False
    src.append("->")
    es = " + ".join(f"e{a}" for a in live)
    dsum = " + ".join(f"d{a}" for a in small)
    if not norm:
        src.append(f"  eb in ? /\\ ({out} - Ex) / eb in [-1, 1]")
    elif not small:
        src += [f"  ({out} - Ex) / Phat in ? /\\ P -/ Phat in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    else:
        src += [f"  ({es}) / Phatx in ? /\\ ({dsum}) in ? /\\ P in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    src += ["}", ""]

    if norm:
        src.append("# How far each rounding is from the identity.")
        for a in gaps:
            src.append(f"g{a} / G{a} -> 1 + (g{a} - G{a}) / G{a};")
        for a in AXES:
            if st[a] == "n":
                src.append(f"s{a} / (g{a} * g{a}) -> 1 + (s{a} - g{a} * g{a}) / (g{a} * g{a});")
        if st["r"] == "n":
            src.append("rr / (r * r) -> 1 + (rr - r * r) / (r * r);")
        if s01_rounds:
            src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
        if s012_rounds:
            src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
        if on_value:
            src.append("value / y -> 1 + (value - y) / y;")
        if not small:
            split = " + ".join(f"({sq[a]} + e{a})" if a in live else sq[a] for a in TERMS)
            src += [
                "",
                "# The error is the sum of the terms' errors, each a fraction of its own",
                "# piece of the permanent; add_rr then averages them.",
                f"({out} - Ex) / Phat -> ((Phat + ({out} - Ex)) - Phat) / Phat;",
                f"Phat + ({out} - Ex) -> {split};",
            ]
            for a in live:
                src.append(f"({sq[a]} + e{a}) - {sq[a]} -> e{a};")
            src += [
                "",
                "# From the permanent to the one on_floats rounds, and on to eb.",
                f"({out} - Ex) / eb -> (({out} - Ex) / Phat) * (Phat / P) * (P / eb);",
                "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
            ]
        else:
            for a in AXES:
                if st[a] == "u":
                    src.append(f"G{a} * G{a} -> X{a} / (rg{a} * rg{a});")
            shifts = [f"(X{a} - {sq[a]})" for a in small if st[a] == "s"]
            shift = shifts[0] if len(shifts) == 1 else f"({' + '.join(shifts)})"
            src += [
                "",
                "# The relative part is averaged over Phatx as in skin.g. What rounding a",
                "# square below the normals cost is an amount, and eb being normal keeps",
                "# P large enough that it is small against eb.",
                f"({out} - Ex) / eb -> (({es}) / Phatx) * (Phatx / P) * (P / eb) + ({dsum}) / eb;",
                f"({es}) / Phatx -> ((Phatx + ({es})) - Phatx) / Phatx;",
                f"Phatx + ({es}) -> " + " + ".join(f"({part[a]} + e{a})" for a in live) + ";",
            ]
            for a in live:
                src.append(f"({part[a]} + e{a}) - {part[a]} -> e{a};")
            src += [
                f"Phatx / P -> Phat / P + {shift} / P;" if shifts else "Phatx / P -> Phat / P;",
                "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
                "P -> eb / (B * (1 + (eb - B * P) / (B * P)));",
            ]
    zh = []
    if "x" in flat and "y" not in flat:
        zh.append("sx + sy -> sy;")
    if "y" in flat and "x" not in flat:
        zh.append("sx + sy -> sx;")
    if "z" in flat:
        zh.append("s01 + sz -> s01;")
    elif "x" in flat and "y" in flat:
        zh.append("s01 + sz -> sz;")
    if "r" in flat:
        zh.append("s012 + rr -> s012;")
        if on_value:
            zh.append("s012 - rr -> s012;")
    elif not s012_rounds:
        zh.append("s012 + rr -> rr;")
        if on_value:
            zh.append("s012 - rr -> -rr;")
    if zh:
        src += ["", "# A zero square added to a double leaves that double, which then rounds",
                "# to itself."] + zh
    return "\n".join(src) + "\n"


def write(name, text):
    open(name, "w").write(text)


for case in map("".join, itertools.product("nzsu", repeat=4)):
    tag = "" if case == "nnnn" else f"_{case}"
    if "n" in case:
        for on_value in (False, True):
            name = f"skin{'v' if on_value else ''}{tag}"
            write(f"{name}.g", make(case, on_value))
            write(f"c_{name}_half.g", make(case, on_value, b=B_HALF))
    else:
        write(f"skin{tag}.g", make(case))
        write(f"c_skin{tag}_noeb.g", make(case, eb_normal=False))
write("c_skin_nonormal.g", make(normal=False))
write("skin_noeb.g", make(eb_normal=False))
