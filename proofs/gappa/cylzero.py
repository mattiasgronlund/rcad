"""Write cyl.g's cases where a difference or a direction component is zero,
one script per case and a control at B = u / 2 beside each.

product_is_normal(a, b) is true when a or b is zero, so on_floats lets a zero
difference f and a zero direction component d through. A half of the cross
product with a zero factor is then exactly 0, and its true product is 0 too,
since a rounded difference of two doubles is zero only when the two are the
same double. So each component is one of:

h  its right half is zero: q = h exactly, its square s = rnd(h^2) is normal
   (stays_accurate), and its reach is |h|, so its reach square is s again;
k  its left half is zero: q = -k, and the same with k;
0  both halves are zero: q, s and the reach are 0 and its true square is 0,
   so it drops out of y, P and the error;
n, c, s, u  both halves live, as in cylsplit.py.

x has both halves live only when the zeros are among fx and dx, y only among
fy and dy, and z only among fz and dz, so with any factor zero at most one
component does, and its four states make four cases. A zero set holding all
three direction components is left out: Cylinder's axis has no zero
direction (cylinder.rs, the doc on basis). A zero radius is left out too:
Cylinder::new refuses it. The names are cylz_<zeros>[_<state>].g and
c_cylz_<zeros>[_<state>]_half.g. Run as: python3 cylzero.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
FACTORS = ("fx", "fy", "fz", "dx", "dy", "dz")
B_CODE = "B = 10b-53 + 5b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
SAYS = {"n": "does not cancel", "c": "cancels, its reach square normal",
        "s": "cancels, its reach square below the normals and not zero",
        "u": "cancels, its reach square rounding to zero",
        "h": "has its right half zero", "k": "has its left half zero", "0": "has both halves zero"}
NAMES = {"f": "difference", "d": "direction component"}


def kinds(zero):
    kind = {}
    for a in AXES:
        live = tuple(not set(p.split(" * ")) & zero for p in HALVES[a])
        kind[a] = {(True, True): "b", (True, False): "h", (False, True): "k", (False, False): "0"}[live]
    assert list(kind.values()).count("b") <= 1, zero
    return kind


def left_sum(terms, rounded):
    """Sum terms left to right as Rust adds them, each sum rounded or not."""
    acc = None
    for t in terms:
        if acc is None:
            acc = t
        else:
            acc = f"rnd({acc} + {t})" if rounded else f"({acc} + {t})"
    return acc


def make(zero, state, b=B_CODE):
    kind = kinds(zero)
    st = {a: state if kind[a] == "b" else kind[a] for a in AXES}
    hlive = [a for a in AXES if st[a] in "ncsuh"]
    klive = [a for a in AXES if st[a] in "ncsuk"]
    sq = [a for a in AXES if st[a] in "nhk"]
    nq = [a for a in AXES if st[a] == "n"]
    one = [a for a in AXES if st[a] in "hk"]
    dead = [a for a in AXES if st[a] in "csu"]
    small = [a for a in AXES if st[a] in "su"]
    there = [a for a in AXES if st[a] != "0"]
    fl = [a for a in AXES if f"f{a}" not in zero]
    dl = [a for a in AXES if f"d{a}" not in zero]
    s01_rounds = "x" in sq and "y" in sq
    s012_rounds = "z" in sq and ("x" in sq or "y" in sq)
    t01_rounds = "x" in dl and "y" in dl
    d_rounds = "z" in dl and ("x" in dl or "y" in dl)
    names = [f"the {f[1]} {NAMES[f[0]]}" for f in FACTORS if f in zero]
    src = [
        "# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f",
        "# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f",
        "# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f",
        "# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.",
        f"# The case where {', '.join(names[:-1]) + ' and ' if len(names) > 1 else ''}{names[-1]} "
        f"{'are' if len(names) > 1 else 'is'} zero: " + "; ".join(f"{a} {SAYS[st[a]]}" for a in AXES) + ".",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the axis origin, the axis direction and the radius, each an f64.",
        "# A difference is zero when the origin's coordinate is the same double as",
        "# the point's (464.11's diffzero.g), and a zero direction component is the",
        "# double 0.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        " ".join(f"o{a} = rnd(o{a}_);" if a in fl else f"o{a} = p{a};" for a in AXES),
        " ".join(f"d{a} = rnd(d{a}_);" if a in dl else f"d{a} = 0;" for a in AXES),
        "r = rnd(r_);",
        "",
        "# What on_floats computes, line for line. A zero, a product of a zero, and",
        "# a double with a zero added round to themselves (464.11's refl.g), so they",
        "# are left unrounded, and a zero added to a double is left out.",
        "Fx = px - ox; Fy = py - oy; Fz = pz - oz;",
        " ".join(f"f{a} = rnd(F{a});" if a in fl else f"f{a} = F{a};" for a in AXES),
        "",
        "# cross_product_halves: h is each component's left half, k its right half.",
    ]
    if dead:
        src += ["# A component that cancels has the same double for both halves, so k",
                "# is written h, and its q and s are zero and drop out."]
    if one:
        src += ["# A component with one half zero is the other half, or its negative,",
                "# exactly."]
    for a in AXES:
        left, right = HALVES[a]
        h = f"rnd({left})" if a in hlive else left
        k = f"h{a}" if a in dead else f"rnd({right})" if a in klive else right
        src.append(f"h{a} = {h}; k{a} = {k};")
    if nq:
        src.append(" ".join(f"Q{a} = h{a} - k{a};" for a in nq))
        src.append(" ".join(f"q{a} = rnd(Q{a});" for a in nq))
    if one:
        src.append(" ".join(f"q{a} = h{a};" if st[a] == "h" else f"q{a} = -k{a};" for a in one))
    src += ["", "# direction_squared, left to right as Rust adds it."]
    src.append(" ".join(f"t{a} = rnd(d{a} * d{a});" for a in dl))
    xy = [a for a in "xy" if a in dl]
    if t01_rounds:
        src.append("T01 = tx + ty; t01 = rnd(T01);")
    elif xy:
        src.append(f"t01 = t{xy[0]};")
    if d_rounds:
        src.append("T = t01 + tz; D = rnd(T);")
    elif xy:
        src.append("D = t01;")
    else:
        src.append("D = tz;")
    src += ["", "rr = rnd(r * r);", "skin = rnd(rr * D);"]
    if sq:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in sq))
    sxy = [a for a in "xy" if a in sq]
    if s01_rounds:
        src.append("S01 = sx + sy; s01 = rnd(S01);")
    elif sxy:
        src.append(f"s01 = s{sxy[0]};")
    if s012_rounds:
        src.append("S012 = s01 + sz; s012 = rnd(S012);")
    elif sxy:
        src.append("s012 = s01;")
    elif sq:
        src.append("s012 = sz;")
    src.append("y = s012 - skin;" if sq else "y = -skin;")
    src += ["value = rnd(y);", "", "# The permanent: each half's reach squared, then the radius term."]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (dbl.g), so it is written unrounded."]
    if one:
        src += ["# A reach with one half zero is the other half's size, and its square",
                "# is the same double as the component's square s."]
    ns = []
    for a in there:
        if st[a] == "n":
            ns.append(f"n{a} = rnd(|h{a}| + |k{a}|);")
        elif st[a] in "csu":
            ns.append(f"n{a} = |h{a}| + |k{a}|;")
        else:
            ns.append(f"n{a} = |{st[a]}{a}|;")
    if ns:
        src.append(" ".join(ns))
    if small:
        src.append(" ".join(f"Xv{a} = n{a} * n{a};" for a in small))
        src += ["# A reach square that rounds to zero is written as the 0 it rounds to",
                "# (464.14's tinysq.g), and the sums leave it out."]
    vs = [f"v{a} = rnd(Xv{a});" if st[a] == "s" else f"v{a} = rnd(n{a} * n{a});"
          for a in there if st[a] != "u"]
    if vs:
        src.append(" ".join(vs))
    kept = [f"v{a}" for a in there if st[a] != "u"]
    acc = left_sum(kept, True)
    src.append(f"P = rnd({acc} + skin);" if acc else "P = skin;")
    src += [
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;",
        "Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);",
        "",
        "# The permanent with no rounding of its own.",
    ]

    def reach(a):
        if st[a] in "hk":
            return f"|{st[a]}{a}| * |{st[a]}{a}|"
        return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"

    src.append("Phat = " + " + ".join([reach(a) for a in there] + ["skin"]) + ";")
    if small:
        terms = [f"v{a}" if st[a] == "s" else reach(a) for a in there if st[a] != "u"]
        acc = left_sum(terms, False)
        src += ["# The same with v in place of each reach square below the normals that",
                "# does not round to zero, and without each that does, added as P adds.",
                f"Phatx = {acc} + skin;" if acc else "Phatx = skin;"]
    src += ["", "# What each rounding multiplied its argument by."]
    if fl:
        src.append(" ".join(f"rf{a} = f{a} / F{a};" for a in fl))
    for a in AXES:
        left, right = HALVES[a]
        rs = ([f"rh{a} = h{a} / ({left});"] if a in hlive else []) + \
             ([f"rk{a} = k{a} / ({right});"] if a in klive else [])
        if rs:
            src.append(" ".join(rs))
    if nq:
        src.append(" ".join(f"rq{a} = q{a} / Q{a};" for a in nq))
    if sq:
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in sq))
    rsum = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rs012 = s012 / S012;"] if s012_rounds else [])
    if rsum:
        src.append(" ".join(rsum))
    src.append(" ".join(f"rt{a} = t{a} / (d{a} * d{a});" for a in dl))
    rsum = (["rt01 = t01 / T01;"] if t01_rounds else []) + (["rD = D / T;"] if d_rounds else [])
    if rsum:
        src.append(" ".join(rsum))
    src += [
        "rrr = rr / (r * r); rsk = skin / (rr * D);",
        "",
        "# Each component's square, as y carries it, is (h - k)^2 times K; the true",
        "# one is (h a - k b)^2, a and b undoing the roundings of h and k.",
    ]
    if dead:
        src.append("# One that cancels has no square in y, and its true one is (h a - h b)^2.")
    if one:
        src.append("# One with a half zero is h^2 K against (h a)^2, or k^2 K against (k b)^2.")
    for a in sq:
        carry = ([f"rq{a}", f"rq{a}"] if a in nq else []) + [f"rs{a}"]
        if a in "xy" and s01_rounds:
            carry.append("rs01")
        if s012_rounds:
            carry.append("rs012")
        src.append(f"K{a} = " + " * ".join(carry) + ";")
    for a in AXES:
        ab = ([f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a});"] if a in hlive else []) + \
             ([f"b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});"] if a in klive else [])
        if ab:
            src.append(" ".join(ab))
    src += [
        "",
        "# So each component's error is three terms, one for each piece of its",
        "# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its",
        "# piece.",
    ]
    if dead:
        src.append("# One that cancels is off by its whole true square, eC.")
    if one:
        src.append("# One with a half zero is one term, for the half that is not.")
    for a in AXES:
        if st[a] == "n":
            src.append(f"WH{a} = K{a} - a{a} * a{a}; WK{a} = K{a} - b{a} * b{a}; WX{a} = K{a} - a{a} * b{a};")
        elif st[a] == "h":
            src.append(f"WH{a} = K{a} - a{a} * a{a};")
        elif st[a] == "k":
            src.append(f"WK{a} = K{a} - b{a} * b{a};")
    for a in AXES:
        if st[a] in "ncsu":
            src.append(f"cH{a} = |h{a}| * |h{a}|; cK{a} = |k{a}| * |k{a}|; cX{a} = 2 * |h{a}| * |k{a}|;")
        elif st[a] in "hk":
            p = st[a].upper()
            src.append(f"c{p}{a} = |{st[a]}{a}| * |{st[a]}{a}|;")
    for a in AXES:
        if st[a] == "n":
            src.append(f"eH{a} = h{a} * h{a} * WH{a}; eK{a} = k{a} * k{a} * WK{a}; eX{a} = -(2 * h{a} * k{a} * WX{a});")
        elif st[a] in "csu":
            src.append(f"eC{a} = -(C{a} * C{a});")
        elif st[a] in "hk":
            p = st[a].upper()
            src.append(f"e{p}{a} = {st[a]}{a} * {st[a]}{a} * W{p}{a};")
    src += [
        "",
        "# The radius term is one piece for each direction component that is not",
        "# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.",
    ]
    for a in dl:
        carry = (["rD"] if d_rounds else []) + (["rt01"] if t01_rounds and a in "xy" else [])
        src.append(f"cG{a} = " + " * ".join(["rsk", "rr"] + carry + [f"t{a}"]) + ";")
    for a in dl:
        carry = (["rD"] if d_rounds else []) + (["rt01"] if t01_rounds and a in "xy" else [])
        src.append(f"WG{a} = 1 / (" + " * ".join(["rrr", f"rt{a}", "rsk"] + carry) + ") - 1;")
    src.append(" ".join(f"eG{a} = cG{a} * WG{a};" for a in dl))
    src += [
        "",
        "# The same permanent, as one block for each component that is not zero,",
        "# its reach squared, and one for the radius term.",
    ]
    p3 = []
    for a in there:
        if st[a] in "ncsu":
            p3.append(f"P3{a} = cH{a} + cK{a} + cX{a};")
        else:
            p3.append(f"P3{a} = c{st[a].upper()}{a};")
    if p3:
        src.append(" ".join(p3))
    src.append("SG = " + " + ".join(f"cG{a}" for a in dl) + ";")
    src.append("P12 = " + " + ".join([f"P3{a}" for a in there] + ["SG"]) + ";")
    src += ["", "# Each component's square as y carries it, and the radius term as Ex has it."]
    ys = [f"Y{a} = K{a} * Q{a} * Q{a};" if a in nq else f"Y{a} = K{a} * q{a} * q{a};" for a in sq]
    if ys:
        src.append(" ".join(ys))
    src.append("Gt = r * r * (dx * dx + dy * dy + dz * dz);")

    def lines(rows):
        return "\n  /\\ ".join(rows)

    finite = ["|px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024"]
    finite += [" /\\ ".join(f"|o{a}| <= 1b1024" for a in fl)] if fl else []
    finite += [" /\\ ".join(f"|d{a}| <= 1b1024" for a in dl), "|r| <= 1b1024"]
    nonzero = [" /\\ ".join(f"|f{a}| >= 1b-1074" for a in fl)] if fl else []
    nonzero += [" /\\ ".join(f"|d{a}| >= 1b-1074" for a in dl), "|r| >= 1b-1074"]
    if nq:
        nonzero.append(" /\\ ".join(f"|q{a}| >= 1b-1074" for a in nq))
    halves = [f"|h{a}| >= 1b-1022" for a in hlive] + [f"|k{a}| >= 1b-1022" for a in klive]
    src += [
        "",
        "{",
        "  # Every input is a finite f64.",
        "     " + lines(finite),
        "  # No factor is zero but those the case names, and no component cancels",
        "  # to zero but one the case says does. For an f64 that is the same as",
        "  # being at least the smallest subnormal.",
        "  /\\ " + lines(nonzero),
        "  # product_is_normal on the halves that are not zero",
        "  # (cross_product_halves_if_normal), and on the components, the direction",
        "  # components and the radius that are not zero, and the radius term",
        "  # (stays_accurate).",
    ]
    if halves:
        src.append("  /\\ " + lines([" /\\ ".join(halves[i:i + 3]) for i in range(0, len(halves), 3)]))
    if sq:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in sq))
    src += [
        "  /\\ " + " /\\ ".join(f"|t{a}| >= 1b-1022" for a in dl),
        "  /\\ |rr| >= 1b-1022 /\\ |skin| >= 1b-1022",
    ]
    if there:
        src.append("  # The reach square of a component that does not cancel is normal: |q| <= n,")
        src.append("  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=")
        src.append("  # rnd(q * q) = s, which is normal, and with one half zero v is s.")
        if dead:
            src += ["  # For one that cancels the case says which it is. One below the normals",
                    "  # that does not round to zero is at least the smallest subnormal, and",
                    "  # tinysq.g and tinyzero.g split them at 2^-1075: at or below, they round",
                    "  # to zero."]
        rows = []
        for a in there:
            if st[a] == "s":
                rows.append(f"Xv{a} <= 1b-1022 /\\ v{a} >= 1b-1074")
            elif st[a] == "u":
                rows.append(f"Xv{a} <= 1b-1075")
            else:
                rows.append(f"|v{a}| >= 1b-1022")
        src.append("  /\\ " + " /\\ ".join(rows))
    src += [
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
        "  # What lemma.g proves each of those gives.",
    ]
    if dead:
        src.append("  # k is h in a component that cancels, so its line is h's against k's product.")
    for a in AXES:
        left, right = HALVES[a]
        if a in hlive:
            src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        if a in klive:
            src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for a in sq:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for a in dl:
        src.append(f"  /\\ t{a} -/ d{a} * d{a} {LEMMA}")
    src.append(f"  /\\ rr -/ r * r {LEMMA}")
    src.append(f"  /\\ skin -/ rr * D {LEMMA}")
    for a in there:
        if st[a] not in "su":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    src += [
        f"  /\\ eb -/ B * P {LEMMA}",
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
    ]
    signs = [f"h{a} / |h{a}| in [-1, 1]" for a in hlive] + \
            [f"k{a} / |k{a}| in [-1, 1]" for a in klive if a not in dead]
    if signs:
        src.append("  /\\ " + lines([" /\\ ".join(signs[i:i + 2]) for i in range(0, len(signs), 2)]))
    src.append("->")
    if small:
        src.append("  (y - Ex) / P12 in ? /\\ P -/ Phatx in ? /\\ P in ?")
    else:
        src.append("  (y - Ex) / P12 in ? /\\ P -/ Phat in ?")
    src += ["  /\\ (y - Ex) / eb in [-1, 1]", "}", "", "# How far each rounding is from the identity."]
    src += [f"f{a} / F{a} -> 1 + (f{a} - F{a}) / F{a};" for a in fl]
    for a in AXES:
        for h, prod, live in zip("hk", HALVES[a], (hlive, klive)):
            if a in live:
                src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in nq:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    for a in sq:
        src.append(f"s{a} / (q{a} * q{a}) -> 1 + (s{a} - q{a} * q{a}) / (q{a} * q{a});")
    if s01_rounds:
        src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
    if s012_rounds:
        src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
    for a in dl:
        src.append(f"t{a} / (d{a} * d{a}) -> 1 + (t{a} - d{a} * d{a}) / (d{a} * d{a});")
    if t01_rounds:
        src.append("t01 / T01 -> 1 + (t01 - T01) / T01;")
    if d_rounds:
        src.append("D / T -> 1 + (D - T) / T;")
    src += [
        "rr / (r * r) -> 1 + (rr - r * r) / (r * r);",
        "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);",
        "",
        "# The error is the sum of the blocks' errors, each the sum of its pieces'",
        "# errors, each a fraction of its own piece of the permanent; add_rr then",
        "# averages them. A component with both halves zero has no block: its",
        "# square and its true square are both 0.",
    ]
    if dead:
        src += ["# A component that cancels is one block, off by its true square, which is",
                "# (a - b)^2 / 4 of its reach squared, 4 h^2."]
    if one:
        src.append("# A component with a half zero is one block of one piece.")
    src.append("(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;")
    blocks = []
    for a in there:
        if st[a] == "n":
            blocks.append(f"(P3{a} + (Y{a} - C{a} * C{a}))")
        elif st[a] in "csu":
            blocks.append(f"(P3{a} + eC{a})")
        else:
            blocks.append(f"(P3{a} + e{st[a].upper()}{a})")
    src.append("P12 + (y - Ex) -> " + " + ".join(blocks + ["(SG + (Gt - skin))"]) + ";")
    for a in nq:
        src.append(f"P3{a} + (Y{a} - C{a} * C{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
    src.append("SG + (Gt - skin) -> " + " + ".join(f"(cG{a} + eG{a})" for a in dl) + ";")
    for a in there:
        if st[a] == "n":
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        elif st[a] in "csu":
            src.append(f"(P3{a} + eC{a}) - P3{a} -> eC{a};")
        else:
            p = st[a].upper()
            src.append(f"(P3{a} + e{p}{a}) - P3{a} -> e{p}{a};")
    for g in dl:
        src.append(f"(cG{g} + eG{g}) - cG{g} -> eG{g};")
    for a in there:
        if st[a] == "n":
            src += [f"eH{a} / cH{a} -> (h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a};",
                    f"eK{a} / cK{a} -> (k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a};",
                    f"eX{a} / cX{a} -> -((h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a});"]
        elif st[a] in "csu":
            src.append(f"eC{a} / P3{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * (a{a} - b{a}) * (a{a} - b{a}) / 4);")
        else:
            p, x = st[a].upper(), f"{st[a]}{a}"
            src.append(f"e{p}{a} / P3{a} -> ({x} / |{x}|) * ({x} / |{x}|) * W{p}{a};")
    src += ["", "# From the pieces to the permanent on_floats rounds, and on to eb.",
            "(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);"]
    if small:
        shift = " + ".join(f"(Xv{a} - v{a})" if st[a] == "s" else f"Xv{a}" for a in small)
        src += ["# What rounding a reach square below the normals cost is an amount, and",
                "# eb being normal keeps P large enough that it is small against eb.",
                f"Phat / P -> Phatx / P + ({shift}) / P;",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
                "P -> eb / (B * (1 + (eb - B * P) / (B * P)));"]
    else:
        src += ["Phat / P -> 1 / (1 + (P - Phat) / Phat);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));"]
    return "\n".join(src) + "\n"


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


count = 0
for mask in itertools.product((False, True), repeat=6):
    zero = {f for f, z in zip(FACTORS, mask) if z}
    if not zero or {"dx", "dy", "dz"} <= zero:
        continue
    tag = "".join(f for f in FACTORS if f in zero)
    for state in ("ncsu" if "b" in kinds(zero).values() else [None]):
        name = f"cylz_{tag}" + (f"_{state}" if state else "")
        write(f"{name}.g", make(zero, state))
        write(f"c_{name}_half.g", make(zero, state, B_HALF))
        count += 1
print("cases written:", count)
