"""Write SideOfAFlatSurface::on_floats in every case of where its three
products land, as 464.8's planecases.py did for one product at a time:

- each product in one of four states: n, normal, as product_is_normal lets
  through; z, of a zero factor, which it lets through too; p and m, below the
  normals and positive or negative, which it refuses;
- each rounded sum of the products left either rounded, and then at least
  2^-1074, or cancelling to zero, which rounds to itself, and is then written
  unrounded;

plane_<states>[_c<set>].g for each, <states> one letter per product, <set>
naming the product each cancelling sum adds; c_<name>_half.g, each case with
a normal product again at B = u / 2, which must fail; c_<name>_noeb.g, each
case with none again without eb normal, which must fail; c_<name>_tight.g,
each case with none again with every hypothesis but the last goal 2^-60 of eb,
or for plane_zzz off within 2^-900, which must fail too, since a real input
misses it; planev_off0.g, the claim on value with the offset zero; and
c_planev_off0_half.g.

Run as: python3 planeall.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

B_CODE = "B = 4b-53 + 1b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
STATES = "npmz"


TIGHT = "  /\\ (y - Ex) / eb in [-1b-60, 1b-60]"
TIGHT_ZZZ = "  /\\ off in [-1b-900, 1b-900]"


def make(states, cancel=(), b=B_CODE, eb_normal=True, tight=False):
    live = [k for k in range(3) if states[k] != "z"]
    small = [k for k in live if states[k] in "pm"]
    normal = [k for k in live if states[k] == "n"]

    def x(k):
        return f"X{k}" if k in small else f"n{k} * p{k}"

    def abs_t(k):
        return {"n": f"|t{k}|", "p": f"t{k}", "m": f"-t{k}"}[states[k]]

    src = [
        "# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at 6a4a814c",
        "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c",
        "# SideOfAFlatSurface::on_floats without its three product_is_normal checks:",
        "# the sign of n . p - off. Products: "
        + ", ".join(f"n{k} * p{k} {dict(n='normal', p='below the normals and positive', m='below the normals and negative', z='zero')[states[k]]}"
                    for k in range(3)) + ".",
    ]
    if cancel:
        src.append(f"# The sum{'s' if len(cancel) > 1 else ''} adding product{'s' if len(cancel) > 1 else ''} "
                   f"{', '.join(map(str, cancel))} cancel{'' if len(cancel) > 1 else 's'} to zero, which rounds to itself.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The normal, the point and the offset, each an f64. A zero product has",
        "# the double 0 for its point's coordinate; with the normal's zero instead,",
        "# every number below is the same.",
        "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);",
        " ".join(f"p{k} = 0;" if states[k] == "z" else f"p{k} = rnd(p{k}_);" for k in range(3)),
        "off = rnd(off_);",
        "",
        "# What on_floats computes, line for line. Adding a zero product to a double",
        "# gives that double back (464.13's addzero.g), so zero products are left",
        "# out of both folds.",
    ]
    if small:
        src.append(" ".join(f"X{k} = n{k} * p{k};" for k in small))
    if live:
        src.append(" ".join(f"t{k} = rnd({x(k)});" for k in live))
    rounded = []
    if live:
        running = f"t{live[0]}"
        for k in live[1:]:
            src.append(f"S{k} = {running} + t{k};")
            if k in cancel:
                src.append(f"s{k} = S{k};")
            else:
                src.append(f"s{k} = rnd(S{k});")
                rounded.append(k)
            running = f"s{k}"
        src.append(f"y = {running} - off;")
    else:
        src.append("y = -off;")
    src.append("value = rnd(y);")
    src += ["", "# The permanent folds the same products' magnitudes in the same order."]
    if live:
        perm = abs_t(live[0])
        for k in live[1:]:
            perm = f"rnd({perm} + {abs_t(k)})"
        src.append(f"P = rnd({perm} + |off|);")
        src.append("Phat = " + " + ".join(abs_t(k) for k in live) + " + |off|;")
    else:
        src.append("P = |off|;")
    src += ["", b, "eb = rnd(B * P);", "", "# The true value of the same inputs."]
    src.append("Ex = " + " + ".join(x(k) if k in small else f"n{k} * p{k}" for k in range(3)) + " - off;")

    carry = {}
    for i, k in enumerate(live):
        enters = live[max(i, 1):]
        carry[k] = [f"rs{j}" for j in rounded if j in enters]

    def r(k):
        return " * ".join(carry[k]) if carry[k] else "1"

    hint_lines = []
    goal = []
    if normal:
        src += ["", "# The products' part of the permanent, with no rounding of its own, and",
                "# the same with X in place of what each product below the normals rounds to."]
        src.append("P3 = " + " + ".join(abs_t(k) for k in live) + ";")
        cs = []
        for k in live:
            cs.append({"n": f"c{k} = |t{k}|;", "p": f"c{k} = X{k};", "m": f"c{k} = -X{k};"}[states[k]])
        src.append(" ".join(cs))
        src.append("P3x = " + " + ".join(f"c{k}" for k in live) + ";")
        src += ["", "# What each rounding multiplied its argument by."]
        src.append(" ".join(f"rp{k} = t{k} / (n{k} * p{k});" for k in normal))
        if rounded:
            src.append(" ".join(f"rs{j} = s{j} / S{j};" for j in rounded))
        src += ["", "# Each term as y carries it less the term: e as a fraction of the term,",
                "# with X for each product below the normals, and d what rounding X cost."]
        for k in live:
            src.append(f"W{k} = {r(k)} - 1 / rp{k};" if k in normal else f"W{k} = {r(k)} - 1;")
        src.append(" ".join(f"e{k} = t{k} * W{k};" if k in normal else f"e{k} = X{k} * W{k};" for k in live))
        if small:
            src.append(" ".join(f"d{k} = (t{k} - X{k}) * ({r(k)});" for k in small))
        esum = " + ".join(f"e{k}" for k in live)
        dsum = " + ".join(f"d{k}" for k in small)
        hint_lines.append("# How far each rounding is from the identity.")
        for k in normal:
            hint_lines.append(f"t{k} / (n{k} * p{k}) -> 1 + (t{k} - n{k} * p{k}) / (n{k} * p{k});")
        for j in rounded:
            hint_lines.append(f"s{j} / S{j} -> 1 + (s{j} - S{j}) / S{j};")
        hint_lines += [
            "",
            "# The relative part is averaged over P3x as in plane.g; what rounding each",
            "# X cost is an amount, and eb being normal keeps it small against eb.",
            f"(y - Ex) / eb -> (({esum}) / P3x) * (P3x / P) * (P / eb)" + (f" + ({dsum}) / eb;" if small else ";"),
            f"({esum}) / P3x -> ((P3x + ({esum})) - P3x) / P3x;",
            f"P3x + ({esum}) -> " + " + ".join(f"(c{k} + e{k})" for k in live) + ";",
        ]
        for k in live:
            hint_lines.append(f"(c{k} + e{k}) - c{k} -> e{k};")
        for k in live:
            hint_lines.append({"n": f"e{k} / c{k} -> (t{k} / |t{k}|) * W{k};",
                               "p": f"e{k} / c{k} -> W{k};", "m": f"e{k} / c{k} -> -W{k};"}[states[k]])
        moved = "".join(f" + (X{k} - t{k}) / P" if states[k] == "p" else f" + (t{k} - X{k}) / P" for k in small)
        hint_lines += [
            f"P3x / P -> (P3 / Phat) * (Phat / P){moved};",
            "P3 / Phat -> 1 / (1 + |off| / P3);",
            "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
            "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        ]
        if small:
            hint_lines.append("P -> eb / (B * (1 + (eb - B * P) / (B * P)));")
        goal.append(f"({esum}) / P3x in ?")
        if small:
            goal += [f"{dsum} in ?", "P in ?"]
        else:
            goal.append("P -/ Phat in ?")
    elif live:
        parts = [f"(t{k} - X{k})" for k in small] + [f"(s{j} - S{j})" for j in rounded]
        src += ["", "# No product is normal, so every error is an amount: what rounding each",
                "# X cost and what rounding each sum cost."]
        hint_lines += [
            "# y less the truth is what each rounding cost, as an amount.",
            "y - Ex -> " + " + ".join(parts) + ";",
        ]
        goal.append("y - Ex in ? /\\ P in ?")
    else:
        hint_lines += ["# Every product is zero, so y is the truth.", "y - Ex -> 0;"]
        goal.append("y - Ex in ?")

    hyps = [("Every input is a finite f64.",
             [[f"|n{k}| <= 1b1024" for k in range(3)],
              [f"|p{k}| <= 1b1024" for k in range(3) if states[k] != "z"] + ["|off| <= 1b1024"]])]
    if normal:
        hyps.append(("No factor of a normal product is zero.",
                     [[f"|n{k}| >= 1b-1074" for k in normal] + [f"|p{k}| >= 1b-1074" for k in normal]]))
    if rounded:
        hyps.append(("No rounded sum cancels to zero.", [[f"|S{j}| >= 1b-1074" for j in rounded]]))
    if small:
        hyps.append(("The products below the normals. A product of two doubles that are not\n"
                     "  # zero is at least 2^-2148.",
                     [[f"X{k} in [1b-2148, 1b-1022]" if states[k] == "p" else f"X{k} in [-1b-1022, -1b-2148]"
                       for k in small]]))
    if normal:
        hyps.append(("The normal products, as product_is_normal has them.",
                     [[f"|t{k}| >= 1b-1022" for k in normal]]))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = [[f"t{k} -/ n{k} * p{k} {LEMMA}"] for k in normal]
    if eb_normal and normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if normal:
        hyps.append(("x / |x| is in [-1, 1] for any x that is not zero.",
                     [[f"t{k} / |t{k}| in [-1, 1]" for k in normal]]))

    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
            first = False
    src.append("->")
    src.append("  " + " /\\ ".join(goal))
    if not tight:
        src.append("  /\\ (y - Ex) / eb in [-1, 1]")
    else:
        src.append(TIGHT if live else TIGHT_ZZZ)
    src += ["}", ""]
    src += hint_lines
    return "\n".join(src) + "\n"


def make_value_off0(b=B_CODE):
    """plane.g's claim taken on value, with the offset the double 0. Then y is
    s012 less zero, which is s012 (addzero.g), so value is y."""
    text = make("nnn", b=b)
    text = text.replace("# the sign of n . p - off. Products: n0 * p0 normal, n1 * p1 normal, n2 * p2 normal.",
                        "# the sign of n . p - off. Products: n0 * p0 normal, n1 * p1 normal, n2 * p2 normal.\n"
                        "# The offset is zero, and the claim is on value, the double on_floats returns.")
    for old, new in [
        ("off = rnd(off_);", "off = 0;"),
        ("y = s2 - off;\nvalue = rnd(y);", "y = s2 - off;\nvalue = rnd(y);"),
        ("  /\\ (y - Ex) / eb in [-1, 1]", "  /\\ value - y in ? /\\ (value - Ex) / eb in [-1, 1]"),
        ("P = rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |off|);", "P = rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |off|);"),
    ]:
        assert text.count(old) == 1, old
        text = text.replace(old, new)
    text = text.replace("     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024\n  /\\ |p0| <= 1b1024 /\\ |p1| <= 1b1024 /\\ |p2| <= 1b1024 /\\ |off| <= 1b1024",
                        "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024\n  /\\ |p0| <= 1b1024 /\\ |p1| <= 1b1024 /\\ |p2| <= 1b1024")
    old = "(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb);"
    assert text.count(old) == 1
    text = text.replace(old, "# value less zero is value, and so is its rounding.\n"
                        "y -> s2;\n"
                        "(value - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + (value - y) / eb;")
    return text


def name_of(states, cancel):
    return f"plane_{states}" + (f"_c{''.join(map(str, cancel))}" if cancel else "")


if __name__ == "__main__":
    proving, half, noeb = [], [], []
    for states in ("".join(s) for s in itertools.product(STATES, repeat=3)):
        live = [k for k in range(3) if states[k] != "z"]
        sums = live[1:]
        for n in range(len(sums) + 1):
            for cancel in itertools.combinations(sums, n):
                name = name_of(states, cancel)
                open(f"{name}.g", "w").write(make(states, cancel))
                proving.append(name)
                if "n" in states:
                    open(f"c_{name}_half.g", "w").write(make(states, cancel, b=B_HALF))
                    half.append(name)
                else:
                    open(f"c_{name}_noeb.g", "w").write(make(states, cancel, eb_normal=False))
                    open(f"c_{name}_tight.g", "w").write(make(states, cancel, tight=True))
                    noeb.append(name)
    open("planev_off0.g", "w").write(make_value_off0())
    open("c_planev_off0_half.g", "w").write(make_value_off0(b=B_HALF))
    print(f"{len(proving)} cases, {len(half)} controls at u / 2, {len(noeb)} controls without eb normal, "
          f"{len(noeb)} with the goal too tight, and planev_off0.g")
