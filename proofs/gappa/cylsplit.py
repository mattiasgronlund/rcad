"""Write cyl.g's case split, one script per case and a control at B = u / 2
beside each. A case is three letters, one for each of the components x, y
and z of the cross product, saying what it is:

n  it does not cancel: q = rnd(h - k) is not zero, so its square s is
   normal (stays_accurate), and so is its reach square v, which is never
   below s;
c  it cancels to zero, h == k, and its reach square v = rnd(4 h^2) is
   normal;
s  it cancels, and v is below the normals and does not round to zero;
u  it cancels, and v rounds to zero.

Those four cover every component whose halves are normal, which
cross_product_halves_if_normal checks: q is zero exactly when h == k. nnn
is cyl.g, byte for byte. A component that cancels has no square in y, and
its true square, (h a - h b)^2, is the whole of its block's error. The
names are cyl_<case>.g and c_cyl_<case>_half.g. Run as: python3 cylsplit.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
B_CODE = "B = 10b-53 + 5b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
STATE = {"n": "does not cancel", "c": "cancels, its reach square normal",
         "s": "cancels, its reach square below the normals and not zero",
         "u": "cancels, its reach square rounding to zero"}


def reach(a):
    return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"


def left_sum(terms, rounded):
    """Sum terms left to right as Rust adds them, each sum rounded or not."""
    acc = None
    for t in terms:
        if acc is None:
            acc = t
        else:
            acc = f"rnd({acc} + {t})" if rounded else f"({acc} + {t})"
    return acc


def make(case, b=B_CODE):
    st = dict(zip(AXES, case))
    live = [a for a in AXES if st[a] == "n"]
    dead = [a for a in AXES if st[a] != "n"]
    small = [a for a in AXES if st[a] in "su"]
    live01 = [a for a in "xy" if a in live]
    s01_rounds = len(live01) == 2
    s012_rounds = bool(live01) and "z" in live
    src = [
        "# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75",
        "# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at e59fab75",
        "# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at e59fab75",
        "# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at e59fab75",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at e59fab75",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at e59fab75",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.",
    ]
    if dead:
        src.append(f"# The case {case}: " + "; ".join(f"{a} {STATE[st[a]]}" for a in AXES) + ".")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the axis origin, the axis direction and the radius, each an f64.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        "ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);",
        "dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);",
        "r = rnd(r_);",
        "",
        "# What on_floats computes, line for line.",
        "Fx = px - ox; Fy = py - oy; Fz = pz - oz;",
        "fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);",
        "",
        "# cross_product_halves: h is each component's left half, k its right half.",
    ]
    if dead:
        src += ["# A component that cancels has the same double for both halves, so k",
                "# is written h, and its q and s are zero and drop out."]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"h{a} = rnd({left}); k{a} = " + (f"h{a};" if a in dead else f"rnd({right});"))
    if live:
        src.append(" ".join(f"Q{a} = h{a} - k{a};" for a in live))
        src.append(" ".join(f"q{a} = rnd(Q{a});" for a in live))
    src += [
        "",
        "# direction_squared, left to right as Rust adds it.",
        "tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);",
        "T01 = tx + ty; t01 = rnd(T01);",
        "T = t01 + tz; D = rnd(T);",
        "",
        "rr = rnd(r * r);",
        "skin = rnd(rr * D);",
    ]
    if dead:
        src += ["# A zero square added to a double leaves that double, which then",
                "# rounds to itself, so a sum is written without its zero squares."]
    if live:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in live))
    if s01_rounds:
        src.append("S01 = sx + sy; s01 = rnd(S01);")
    elif live01:
        src.append(f"s01 = s{live01[0]};")
    if s012_rounds:
        src.append("S012 = s01 + sz; s012 = rnd(S012);")
    elif live01:
        src.append("s012 = s01;")
    elif live:
        src.append("s012 = sz;")
    src.append("y = s012 - skin;" if live else "y = -skin;")
    src += ["value = rnd(y);", "", "# The permanent: each half's reach squared, then the radius term."]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (dbl.g), so it is written unrounded."]
    src.append(" ".join(f"n{a} = rnd(|h{a}| + |k{a}|);" if a in live else f"n{a} = |h{a}| + |k{a}|;"
                        for a in AXES))
    if small:
        src.append(" ".join(f"Xv{a} = n{a} * n{a};" for a in small))
        src += ["# A reach square that rounds to zero is written as the 0 it rounds to",
                "# (464.14's tinysq.g), and the sums leave it out."]
    vs = []
    for a in AXES:
        if st[a] in "nc":
            vs.append(f"v{a} = rnd(n{a} * n{a});")
        elif st[a] == "s":
            vs.append(f"v{a} = rnd(Xv{a});")
    src.append(" ".join(vs))
    kept = [f"v{a}" for a in AXES if st[a] != "u"]
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
        "Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)",
        "     + (|hz| + |kz|) * (|hz| + |kz|) + skin;",
    ]
    if small:
        terms = [f"v{a}" if st[a] == "s" else reach(a) for a in AXES if st[a] != "u"]
        acc = left_sum(terms, False)
        src += ["# The same with v in place of each reach square below the normals that",
                "# does not round to zero, and without each that does, added as P adds.",
                f"Phatx = {acc} + skin;" if acc else "Phatx = skin;"]
    src += [
        "",
        "# What each rounding multiplied its argument by.",
        "rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;",
    ]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"rh{a} = h{a} / ({left}); rk{a} = k{a} / ({right});")
    if live:
        src.append(" ".join(f"rq{a} = q{a} / Q{a};" for a in live))
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in live))
    rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rs012 = s012 / S012;"] if s012_rounds else [])
    if rs:
        src.append(" ".join(rs))
    src += [
        "rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);",
        "rt01 = t01 / T01; rD = D / T;",
        "rrr = rr / (r * r); rsk = skin / (rr * D);",
        "",
        "# Each component's square, as y carries it, is (h - k)^2 times K; the true",
        "# one is (h a - k b)^2, a and b undoing the roundings of h and k.",
    ]
    if dead:
        src.append("# One that cancels has no square in y, and its true one is (h a - h b)^2.")
    for a in live:
        carry = [f"rq{a}", f"rq{a}", f"rs{a}"]
        if a in "xy" and s01_rounds:
            carry.append("rs01")
        if s012_rounds:
            carry.append("rs012")
        src.append(f"K{a} = " + " * ".join(carry) + ";")
    for a in AXES:
        src.append(f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a}); b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});")
    src += [
        "",
        "# So each component's error is three terms, one for each piece of its",
        "# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its",
        "# piece.",
    ]
    if dead:
        src.append("# One that cancels is off by its whole true square, eC.")
    for a in live:
        src.append(f"WH{a} = K{a} - a{a} * a{a}; WK{a} = K{a} - b{a} * b{a}; WX{a} = K{a} - a{a} * b{a};")
    for a in AXES:
        src.append(f"cH{a} = |h{a}| * |h{a}|; cK{a} = |k{a}| * |k{a}|; cX{a} = 2 * |h{a}| * |k{a}|;")
    for a in AXES:
        if a in live:
            src.append(f"eH{a} = h{a} * h{a} * WH{a}; eK{a} = k{a} * k{a} * WK{a}; eX{a} = -(2 * h{a} * k{a} * WX{a});")
        else:
            src.append(f"eC{a} = -(C{a} * C{a});")
    src += [
        "",
        "# The radius term is three pieces, one for each direction component's",
        "# square, which sum to skin. Each true piece is r^2 d^2.",
        "cGx = rsk * rr * rD * rt01 * tx;",
        "cGy = rsk * rr * rD * rt01 * ty;",
        "cGz = rsk * rr * rD * tz;",
        "WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;",
        "WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;",
        "WGz = 1 / (rrr * rtz * rsk * rD) - 1;",
        "eGx = cGx * WGx; eGy = cGy * WGy; eGz = cGz * WGz;",
        "",
        "# The same permanent, as four blocks of the twelve pieces the error is",
        "# averaged over: each component's reach squared, and the radius term.",
        "P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;",
        "SG = cGx + cGy + cGz;",
        "P12 = P3x + P3y + P3z + SG;",
        "",
        "# Each component's square as y carries it, and the radius term as Ex has it.",
    ]
    if live:
        src.append(" ".join(f"Y{a} = K{a} * Q{a} * Q{a};" for a in live))
    src.append("Gt = r * r * (dx * dx + dy * dy + dz * dz);")

    nonzero = "\n  /\\ ".join([
        "|fx| >= 1b-1074 /\\ |fy| >= 1b-1074 /\\ |fz| >= 1b-1074",
        "|dx| >= 1b-1074 /\\ |dy| >= 1b-1074 /\\ |dz| >= 1b-1074",
        "|r| >= 1b-1074"] + ([" /\\ ".join(f"|q{a}| >= 1b-1074" for a in live)] if live else []))
    src += [
        "",
        "{",
        "  # Every input is a finite f64.",
        "     |px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024",
        "  /\\ |ox| <= 1b1024 /\\ |oy| <= 1b1024 /\\ |oz| <= 1b1024",
        "  /\\ |dx| <= 1b1024 /\\ |dy| <= 1b1024 /\\ |dz| <= 1b1024",
        "  /\\ |r| <= 1b1024",
        "  # No factor is zero, and no component cancels to zero. For an f64 that is"
        if not dead else
        "  # No factor is zero, and no component but those the case names cancels to\n"
        "  # zero. For an f64 that is",
        "  # the same as being at least the smallest subnormal.",
        "  /\\ " + nonzero,
        "  # product_is_normal on the six halves (cross_product_halves_if_normal),",
        "  # and on the three components, the three direction components, the",
        "  # radius and the radius term (stays_accurate).",
        "  /\\ |hx| >= 1b-1022 /\\ |kx| >= 1b-1022 /\\ |hy| >= 1b-1022",
        "  /\\ |ky| >= 1b-1022 /\\ |hz| >= 1b-1022 /\\ |kz| >= 1b-1022",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in live))
    src += [
        "  /\\ |tx| >= 1b-1022 /\\ |ty| >= 1b-1022 /\\ |tz| >= 1b-1022",
        "  /\\ |rr| >= 1b-1022 /\\ |skin| >= 1b-1022",
    ]
    if not dead:
        src += [
            "  # The permanent's three squares are normal too, since no component is",
            "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,",
            "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal.",
            "  /\\ |vx| >= 1b-1022 /\\ |vy| >= 1b-1022 /\\ |vz| >= 1b-1022",
        ]
    else:
        src += [
            "  # The reach square of a component that does not cancel is normal: |q| <= n,",
            "  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=",
            "  # rnd(q * q) = s, which is normal. For one that cancels the case says",
            "  # which it is. One below the normals that does not round to zero is at",
            "  # least the smallest subnormal, and tinysq.g and tinyzero.g split them",
            "  # at 2^-1075: at or below, they round to zero.",
        ]
        rows = []
        for a in AXES:
            if st[a] in "nc":
                rows.append(f"|v{a}| >= 1b-1022")
            elif st[a] == "s":
                rows.append(f"Xv{a} <= 1b-1022 /\\ v{a} >= 1b-1074")
            else:
                rows.append(f"Xv{a} <= 1b-1075")
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
        src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for a in live:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for p in ("tx -/ dx * dx", "ty -/ dy * dy", "tz -/ dz * dz", "rr -/ r * r", "skin -/ rr * D"):
        src.append(f"  /\\ {p} {LEMMA}")
    for a in AXES:
        if st[a] in "nc":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    src += [
        f"  /\\ eb -/ B * P {LEMMA}",
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
    ]
    for a in AXES:
        src.append(f"  /\\ h{a} / |h{a}| in [-1, 1]" + ("" if a in dead else f" /\\ k{a} / |k{a}| in [-1, 1]"))
    src.append("->")
    if small:
        src.append("  (y - Ex) / P12 in ? /\\ P -/ Phatx in ? /\\ P in ?")
    else:
        src.append("  (y - Ex) / P12 in ? /\\ P -/ Phat in ?")
    src += ["  /\\ (y - Ex) / eb in [-1, 1]", "}", "", "# How far each rounding is from the identity."]
    src += ["fx / Fx -> 1 + (fx - Fx) / Fx;", "fy / Fy -> 1 + (fy - Fy) / Fy;", "fz / Fz -> 1 + (fz - Fz) / Fz;"]
    for a in AXES:
        for h, prod in zip("hk", HALVES[a]):
            src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in live:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    for a in live:
        src.append(f"s{a} / (q{a} * q{a}) -> 1 + (s{a} - q{a} * q{a}) / (q{a} * q{a});")
    if s01_rounds:
        src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
    if s012_rounds:
        src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
    src += [
        "tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);",
        "ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);",
        "tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);",
        "t01 / T01 -> 1 + (t01 - T01) / T01;",
        "D / T -> 1 + (D - T) / T;",
        "rr / (r * r) -> 1 + (rr - r * r) / (r * r);",
        "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);",
        "",
        "# The error is the sum of four blocks' errors, each the sum of its pieces'",
        "# errors, each a fraction of its own piece of the permanent; add_rr then",
        "# averages them. In blocks so that no hint's identity spans all twelve",
        "# pieces: one hint over all twelve did not finish in 600 s.",
    ]
    if dead:
        src += ["# A component that cancels is one block, off by its true square, which is",
                "# (a - b)^2 / 4 of its reach squared, 4 h^2."]
    src.append("(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;")
    blocks = [f"(P3{a} + (Y{a} - C{a} * C{a}))" if a in live else f"(P3{a} + eC{a})" for a in AXES]
    src.append("P12 + (y - Ex) -> " + " + ".join(blocks) + " + (SG + (Gt - skin));")
    for a in live:
        src.append(f"P3{a} + (Y{a} - C{a} * C{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
    src.append("SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy) + (cGz + eGz);")
    for a in AXES:
        if a in live:
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        else:
            src.append(f"(P3{a} + eC{a}) - P3{a} -> eC{a};")
    for g in AXES:
        src.append(f"(cG{g} + eG{g}) - cG{g} -> eG{g};")
    for a in AXES:
        if a in live:
            src += [f"eH{a} / cH{a} -> (h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a};",
                    f"eK{a} / cK{a} -> (k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a};",
                    f"eX{a} / cX{a} -> -((h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a});"]
        else:
            src.append(f"eC{a} / P3{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * (a{a} - b{a}) * (a{a} - b{a}) / 4);")
    src += ["", "# From the twelve pieces to the permanent on_floats rounds, and on to eb.",
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


for case in map("".join, itertools.product("ncsu", repeat=3)):
    if case == "nnn":
        assert make(case) == open("cyl.g").read()
        continue
    name = f"cyl_{case}"
    write(f"{name}.g", make(case))
    write(f"c_{name}_half.g", make(case, B_HALF))
