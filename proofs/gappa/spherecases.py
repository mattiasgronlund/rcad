"""Write sphere.g, HowFarOut::on_floats and its ERROR_BOUND line for line, and
its cases and controls beside it:

- spherev.g, the claim on value, the double on_floats returns, not on y;
- sphere_z<set>.g and spherev_z<set>.g for each set of the four squares that
  are zero, which product_is_normal lets through: <set> names which of the
  gaps x, y, z and the radius r are zero;
- c_<name>_half.g, each of those at B = u / 2, which must fail;
- c_sphere_nonormal.g, sphere.g without product_is_normal, and
  sphere_noeb.g, sphere.g without eb normal, which proves (464.12's 7).

Run as: python3 spherecases.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

AXES = ("x", "y", "z")
TERMS = AXES + ("r",)
B_CODE = "B = 8b-53 + 1b-100;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"


def make(zero="", on_value=False, b=B_CODE, normal=True, eb_normal=True):
    gaps = [a for a in AXES if a not in zero]
    radius = "r" not in zero
    live = gaps + (["r"] if radius else [])
    out = "value" if on_value else "y"
    sq = {a: f"s{a}" for a in AXES}
    sq["r"] = "rr"

    src = [
        "# restates: crates/rcad-predicates/src/surface.rs on_floats at 3d60b0de",
        "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 3d60b0de",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 3d60b0de",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 3d60b0de",
        "# HowFarOut::on_floats and its ERROR_BOUND, the sphere of surface.rs' tests.",
    ]
    if on_value:
        src.append("# The claim is on value, the double on_floats returns, not on y.")
    if zero:
        names = [f"the {a} gap" for a in AXES if a in zero] + (["the radius"] if not radius else [])
        src.append(f"# The case where {' and '.join(names)} {'is' if len(names) == 1 else 'are'} zero.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre and the radius, each an f64. A gap is zero when",
        "# the centre's coordinate is the same double as the point's (464.11's",
        "# diffzero.g), and a zero radius is the double 0.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        " ".join(f"c{a} = p{a};" if a in zero else f"c{a} = rnd(c{a}_);" for a in AXES),
        "r = rnd(r_);" if radius else "r = 0;",
        "",
        "# What on_floats computes, line for line. A zero, a square of one and a",
        "# sum of two round to themselves (464.11's refl.g), so they are left",
        "# unrounded.",
        " ".join(f"G{a} = p{a} - c{a};" for a in AXES),
        " ".join(f"g{a} = G{a};" if a in zero else f"g{a} = rnd(G{a});" for a in AXES),
        " ".join(f"s{a} = g{a} * g{a};" if a in zero else f"s{a} = rnd(g{a} * g{a});" for a in AXES),
        "rr = rnd(r * r);" if radius else "rr = r * r;",
        "S01 = sx + sy;",
        "s01 = S01;" if "x" in zero and "y" in zero else "s01 = rnd(S01);",
        "S012 = s01 + sz;",
        "s012 = S012;" if not gaps else "s012 = rnd(S012);",
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
    if live:
        src += ["", "# What each rounding multiplied its argument by."]
        if gaps:
            src.append(" ".join(f"rg{a} = g{a} / G{a};" for a in gaps))
            src.append(" ".join(f"rp{a} = s{a} / (g{a} * g{a});" for a in gaps))
        if radius:
            src.append("rpr = rr / (r * r);")
        rs = []
        if "x" in gaps or "y" in gaps:
            rs.append("rs01 = s01 / S01;")
        if gaps:
            rs.append("rs012 = s012 / S012;")
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
        for a in gaps:
            src.append(f"W{a} = {carry[a]} - 1 / (rg{a} * rg{a} * rp{a});")
        if radius:
            src.append(f"Wr = 1 / rpr - {carry['r']};")
        src.append(" ".join(f"e{a} = {sq[a]} * W{a};" for a in live))

    hyps = []
    finite = [f"|p{a}| <= 1b1024" for a in AXES] + [f"|c{a}| <= 1b1024" for a in gaps]
    if radius:
        finite.append("|r| <= 1b1024")
    hyps.append(("Every input is a finite f64.", [finite[:3], finite[3:]]))
    if live:
        nonzero = [f"|g{a}| >= 1b-1074" for a in gaps] + (["|r| >= 1b-1074"] if radius else [])
        hyps.append(("No factor of the squares left is zero. For an f64 that is the same\n"
                     "  # as being at least the smallest subnormal.", [nonzero]))
        if normal:
            hyps.append(("product_is_normal on the squares left.",
                         [[f"|{sq[a]}| >= 1b-1022" for a in live]]))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = []
    if normal:
        lemma += [[f"s{a} -/ g{a} * g{a} {LEMMA}"] for a in gaps]
        if radius:
            lemma.append([f"rr -/ r * r {LEMMA}"])
    if eb_normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if on_value and live:
        hyps.append(("y is not zero. Where it is, value is zero too, and the claim on y is\n"
                     "  # the claim on value.", [["|y| >= 1b-1074"]]))

    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            row = [h for h in row]
            if not row:
                continue
            src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
            first = False
    src.append("->")
    if live:
        src += [f"  ({out} - Ex) / Phat in ? /\\ P -/ Phat in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    else:
        src.append(f"  eb in ? /\\ ({out} - Ex) / eb in [-1, 1]")
    src += ["}", ""]

    if live:
        src.append("# How far each rounding is from the identity.")
        for a in gaps:
            src.append(f"g{a} / G{a} -> 1 + (g{a} - G{a}) / G{a};")
        for a in gaps:
            src.append(f"s{a} / (g{a} * g{a}) -> 1 + (s{a} - g{a} * g{a}) / (g{a} * g{a});")
        if radius:
            src.append("rr / (r * r) -> 1 + (rr - r * r) / (r * r);")
        if "x" in gaps or "y" in gaps:
            src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
        if gaps:
            src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
        if on_value:
            src.append("value / y -> 1 + (value - y) / y;")
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
    zh = []
    if "x" in zero and "y" not in zero:
        zh.append("sx + sy -> sy;")
    if "y" in zero and "x" not in zero:
        zh.append("sx + sy -> sx;")
    if "z" in zero:
        zh.append("s01 + sz -> s01;")
    elif "x" in zero and "y" in zero:
        zh.append("s01 + sz -> sz;")
    if not radius:
        zh.append("s012 + rr -> s012;")
        if on_value:
            zh.append("s012 - rr -> s012;")
    elif not gaps:
        zh.append("s012 + rr -> rr;")
        if on_value:
            zh.append("s012 - rr -> -rr;")
    if zh:
        src += ["", "# A zero square added to a double leaves that double, which then rounds",
                "# to itself."] + zh
    return "\n".join(src) + "\n"


def write(name, text):
    open(name, "w").write(text)


for n in range(len(TERMS)):
    for zero in itertools.combinations(TERMS, n):
        zero = "".join(zero)
        tag = f"_z{zero}" if zero else ""
        for on_value in (False, True):
            name = f"sphere{'v' if on_value else ''}{tag}"
            write(f"{name}.g", make(zero, on_value))
            write(f"c_{name}_half.g", make(zero, on_value, b=B_HALF))
# Every square zero: then eb is zero, so decide does not compare, and Gappa
# proves the goal from the contradiction. Its control drops eb normal.
write("sphere_zxyzr.g", make("xyzr"))
write("c_sphere_zxyzr_noeb.g", make("xyzr", eb_normal=False))
write("c_sphere_nonormal.g", make(normal=False))
write("sphere_noeb.g", make(eb_normal=False))
