"""Write Orient2d::on_floats with a difference zero, the branch 464.8 left on
paper. A difference of two doubles is zero only when the two are equal, and is
then exactly zero, and so is its product; so a zero difference makes forwards
or backwards the double 0.

- o2zf.g: forwards zero and acy * bcx at least 2^-1000, which must prove.
- o2zf_tiny.g: forwards zero and acy * bcx at most 2^-1000, where eb cannot be
  normal, which must print that the rest is pointless.
- o2zb.g and o2zb_tiny.g: the same with backwards zero.
- o2zfb.g: both zero, which must print pointless too.
- c_o2zf_half.g and c_o2zb_half.g: the two that prove again at B = u / 2,
  which must fail; c_<name>_noeb.g: the three pointless ones again without eb
  normal, which must fail.

Run as: python3 o2zero.py; it writes beside itself.
"""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

B_CODE = "B = 3b-53 + 1b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HEAD = """# restates: crates/rcad-predicates/src/orient2d.rs on_floats at 6a4a814c
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
"""

# Each one-zero case, by which product is zero: the differences the other
# product is made of, its name in on_floats, and its sign in y.
ONE = {
    "f": dict(zero="forwards", why="a and c share their x, so acx is zero; with b and c sharing\n# their y instead, bcy is zero",
              ins=("ay", "bx", "cx", "cy"), d1=("Acy", "ay - cy", "acy"), d2=("Bcx", "bx - cx", "bcx"),
              live="backwards", y="y = -backwards;", ex="Ex = -(Acy * Bcx);",
              y_why="0 less backwards is -backwards", e="e = -(backwards * W);",
              e_hint="e / P -> -((backwards / |backwards|) * W);"),
    "b": dict(zero="backwards", why="a and c share their y, so acy is zero; with b and c sharing\n# their x instead, bcx is zero",
              ins=("ax", "by", "cx", "cy"), d1=("Acx", "ax - cx", "acx"), d2=("Bcy", "by - cy", "bcy"),
              live="forwards", y="y = forwards;", ex="Ex = Acx * Bcy;",
              y_why="forwards less 0 is forwards", e="e = forwards * W;",
              e_hint="e / P -> (forwards / |forwards|) * W;"),
}


def one_zero(which, tiny=False, b=B_CODE, eb_normal=True):
    c = ONE[which]
    (D1, d1_def, d1), (D2, d2_def, d2) = c["d1"], c["d2"]
    live, zero = c["live"], c["zero"]
    lines = [
        HEAD.rstrip("\n"),
        f"# Orient2d::on_floats where {c['why']}, and every number below is the",
        f"# same. Either way {zero} is the double 0. {d1} * {d2} is at "
        + ("most" if tiny else "least") + " 2^-1000.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The four coordinates left, each an f64.",
        " ".join(f"{v} = rnd({v}_);" for v in c["ins"]),
        "",
        "# What on_floats computes, line for line. The zero difference is exactly",
        f"# 0, and so is {zero}; {c['y_why']}, and |forwards| plus",
        f"# |backwards| is |{live}| (464.13's addzero.g).",
        f"{D1} = {d1_def}; {D2} = {d2_def};",
        f"{d1} = rnd({D1}); {d2} = rnd({D2});",
        f"G = {d1} * {d2};",
        f"{live} = rnd(G);",
        c["y"],
        "value = rnd(y);",
        f"P = |{live}|;",
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true determinant of the same inputs, the zero difference being zero.",
        c["ex"],
        "",
        "# What each rounding multiplied its argument by.",
        f"r1 = {d1} / {D1}; r2 = {d2} / {D2}; rp = {live} / G;",
        "",
        f"# {live} less the product of the exact differences, as a fraction of it.",
        "W = 1 - 1 / (r1 * r2 * rp);",
        c["e"],
        "",
        "{",
        "  # Every coordinate is a finite f64.",
        "     " + " /\\ ".join(f"|{v}| <= 1b1024" for v in c["ins"]),
        "  # The other two differences are not zero.",
        f"  /\\ |{d1}| >= 1b-1074 /\\ |{d2}| >= 1b-1074",
    ]
    if tiny:
        lines += [f"  # The case: {d1} * {d2} is at most 2^-1000.", "  /\\ |G| <= 1b-1000"]
    else:
        lines += [f"  # The case: {d1} * {d2} is far above the normals.", "  /\\ |G| >= 1b-1000"]
    if eb_normal:
        lines += ["  # The bound is normal, which decide checks.", "  /\\ |eb| >= 1b-1022"]
        if not tiny:
            lines += ["  # What lemma.g proves that gives.", f"  /\\ eb -/ B * P {LEMMA}"]
    if not tiny:
        lines += ["  # x / |x| is in [-1, 1] for any x that is not zero.", f"  /\\ {live} / |{live}| in [-1, 1]"]
    lines += [
        "->",
        "  eb in ?" if tiny else "  e / P in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
        "# How far each rounding is from the identity.",
        f"{d1} / {D1} -> 1 + ({d1} - {D1}) / {D1};",
        f"{d2} / {D2} -> 1 + ({d2} - {D2}) / {D2};",
        f"{live} / G -> 1 + ({live} - G) / G;",
        "",
        f"# {zero} is zero, so y less the truth is {live}'s error alone.",
        "(y - Ex) / eb -> (e / P) * (P / eb);",
        c["e_hint"],
        "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
    ]
    return "\n".join(lines) + "\n"


def both_zero(eb_normal=True):
    lines = [
        HEAD.rstrip("\n"),
        "# Orient2d::on_floats where both products are the double 0: one of acx and",
        "# bcy is zero, and one of acy and bcx. Then forwards less backwards is 0,",
        "# and so are the permanent and the true determinant, a zero difference",
        "# being exactly zero.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "y = 0;",
        "value = rnd(y);",
        "P = 0;",
        "",
        B_CODE,
        "eb = rnd(B * P);",
        "",
        "Ex = 0;",
        "",
        "{",
    ]
    if eb_normal:
        lines += ["  # The bound is normal, which decide checks.", "     |eb| >= 1b-1022"]
    else:
        lines += ["  # Nothing: without eb normal there is no hypothesis left.", "     P in [0, 0]"]
    lines += ["->", "  eb in ?", "  /\\ (y - Ex) / eb in [-1, 1]", "}"]
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    for which in ONE:
        open(f"o2z{which}.g", "w").write(one_zero(which))
        open(f"c_o2z{which}_half.g", "w").write(one_zero(which, b=B_HALF))
        open(f"o2z{which}_tiny.g", "w").write(one_zero(which, tiny=True))
        open(f"c_o2z{which}_tiny_noeb.g", "w").write(one_zero(which, tiny=True, eb_normal=False))
    # o2zfb.g, both zero, has no hypothesis for coqc to print, which prove-bounds
    # refuses, so both_zero is kept here and not written (464.18 5).
    print("4 cases and 4 controls")
