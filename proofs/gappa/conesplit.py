"""Write cone.g's case split, one script per case and a control at B = u / 2
beside each. A case is four letters. The first three are the components x, y
and z of the cross product, as cylsplit.py (464.15) has them:

n  it does not cancel: q = rnd(h - k) is not zero, so its square s is
   normal (stays_accurate), and so is its reach square v, which is never
   below s;
c  it cancels to zero, h == k, and its reach square v = rnd(4 h^2) is
   normal;
s  it cancels, and v is below the normals and does not round to zero;
u  it cancels, and v rounds to zero.

The fourth is the axial term, up = rnd(rnd(ux + uy) + uz) and the
permanent's A = rnd(ss * rnd(R * R)):

n  up is not zero, so w and along are normal (stays_accurate), and
   u01 = rnd(ux + uy) is normal; |up| <= R, so R2 >= w and A >= along are
   normal too;
e  up is not zero, and u01 is below the normals, where it is exact
   (sumexact.g), so it is written unrounded;
c  up cancels to zero, u01 + uz == 0, and R2 and A are normal;
a  up cancels, R2 is normal and A is below the normals or zero;
s  up cancels, R2 is below the normals and not zero, and A is normal;
t  up cancels, R2 is below the normals and not zero, and A is below the
   normals or zero.

Only n, e and c are written here. a, s and t are states that on_floats
refuses since Q2779 (a96cd9d8), which made it check R2 and A normal, so a
script for one no longer restates the code and is not moved (464.10 5). The
six cover every axial term whose halves are normal, but one: up cancels and
R2 rounds to zero, which is where 464.16's probe found a wrong sign, so no
script is written for it. nnnn is cone.g,
byte for byte. Where up cancels, the axial term's error is the whole
of the true one, sl^2 Dt^2, and it is averaged over XA = ss R2 rather than
ss G^2, since R2 then need not be near G^2. The names are cone_<case>.g and
c_cone_<case>_half.g. Where no component is n and up cancels, y is 0 and its
error is a u^2 of the permanent, which B = u / 2 covers, so those 108 cases
have a second control, c_cone_<case>_usq.g, at B = u^2.
Run as: python3 conesplit.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
B_CODE = "B = 12b-53 + 3b-101;"
B_HALF = "B = 1b-54;"
B_USQ = "B = 1b-106;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
STATE = {"n": "does not cancel", "c": "cancels, its reach square normal",
         "s": "cancels, its reach square below the normals and not zero",
         "u": "cancels, its reach square rounding to zero"}
AXIAL = {"n": "up does not cancel and its first sum is normal",
         "e": "up does not cancel and its first sum is below the normals",
         "c": "up cancels, R2 and A normal",
         "a": "up cancels, R2 normal and A below the normals or zero",
         "s": "up cancels, R2 below the normals and not zero and A normal",
         "t": "up cancels, R2 below the normals and not zero and A below the normals or zero"}
UPS = "ne"


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
    st = dict(zip(AXES, case[:3]))
    ax = case[3]
    up = ax in UPS
    xr = ax in "st"
    xa_low = ax in "at"
    live = [a for a in AXES if st[a] == "n"]
    dead = [a for a in AXES if st[a] != "n"]
    small = [a for a in AXES if st[a] in "su"]
    shifted = bool(small) or xa_low
    live01 = [a for a in "xy" if a in live]
    s01_rounds = len(live01) == 2
    s012_rounds = bool(live01) and "z" in live
    src = [
        "# restates: crates/rcad-geom/src/cone.rs on_floats at e59fab75",
        "# restates: crates/rcad-geom/src/cone.rs cross_product_halves_if_normal at e59fab75",
        "# restates: crates/rcad-geom/src/cone.rs cross_product_halves at e59fab75",
        "# restates: crates/rcad-geom/src/cone.rs SIDE_OF_CONE_ERROR_BOUND at e59fab75",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at e59fab75",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at e59fab75",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.",
    ]
    if case != "nnnn":
        src.append(f"# The case {case}: " + "; ".join(f"{a} {STATE[st[a]]}" for a in AXES)
                   + f"; {AXIAL[ax]}.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the apex, the axis direction and the slope, each an f64.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        "ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);",
        "dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);",
        "sl = rnd(sl_);",
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
        "# up_halves, and up, summed left to right as Rust adds them.",
        "ux = rnd(fx * dx); uy = rnd(fy * dy); uz = rnd(fz * dz);",
    ]
    if ax == "e":
        src += ["# Its first sum is below the normals, where it is exact (sumexact.g).",
                "u01 = ux + uy;"]
    else:
        src.append("U01 = ux + uy; u01 = rnd(U01);")
    if up:
        src.append("U = u01 + uz; up = rnd(U);")
    else:
        src += ["# up cancels: u01 + uz is zero, so up, w and along are zero, and y is",
                "# the squares alone."]
    src.append("")
    if dead:
        src += ["# A zero square added to a double leaves that double, which then",
                "# rounds to itself, so a sum is written without its zero squares."]
    if live:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in live))
    src.append("ss = rnd(sl * sl);")
    if up:
        src += ["w = rnd(up * up);", "along = rnd(ss * w);"]
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
    if up:
        src.append("y = s012 - along;" if live else "y = -along;")
    else:
        src.append("y = s012;" if live else "y = 0;")
    src += ["value = rnd(y);", "",
            "# The permanent: each half's reach squared, then the slope squared times the",
            "# reach of up squared."]
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
    if vs:
        src.append(" ".join(vs))
    src.append("R01 = rnd(|ux| + |uy|); R = rnd(R01 + |uz|);")
    if xr:
        src.append("XR = R * R; R2 = rnd(XR);")
    else:
        src.append("R2 = rnd(R * R);")
    if up:
        src.append("A = rnd(ss * R2);")
    else:
        src.append("XA = ss * R2; A = rnd(XA);")
    kept = [f"v{a}" for a in AXES if st[a] != "u"]
    acc = left_sum(kept, True)
    src.append(f"P = rnd({acc} + A);" if acc else "P = A;")
    src += [
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;",
        "Dt = Fx * dx + Fy * dy + Fz * dz;",
        "Ex = Cx * Cx + Cy * Cy + Cz * Cz - sl * sl * (Dt * Dt);",
        "",
        "# The permanent with no rounding of its own.",
        "G = |ux| + |uy| + |uz|;",
    ]
    axial_ref = "ss * (G * G)" if up else "XA"
    if not up:
        src += ["# Where up cancels, its axial term is XA, which R2 need not keep near",
                "# ss G^2."]
    src += [
        "Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)",
        f"     + (|hz| + |kz|) * (|hz| + |kz|) + {axial_ref};",
    ]
    if shifted:
        terms = [f"v{a}" if st[a] == "s" else reach(a) for a in AXES if st[a] != "u"]
        acc = left_sum(terms, False)
        last = "A" if xa_low else axial_ref
        if small:
            src += ["# The same with v in place of each reach square below the normals that",
                    "# does not round to zero, and without each that does, "
                    + ("and A in place of XA,\n# " if xa_low else "")
                    + "added as P adds."]
        else:
            src.append("# The same with A in place of XA, added as P adds.")
        src.append(f"Phatx = {acc} + {last};" if acc else f"Phatx = {last};")
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
    src.append("rux = ux / (fx * dx); ruy = uy / (fy * dy); ruz = uz / (fz * dz);")
    if ax == "e":
        src.append("rU = up / U;")
    elif up:
        src.append("ru01 = u01 / U01; rU = up / U;")
    else:
        src.append("ru01 = u01 / U01;")
    src.append("rss = ss / (sl * sl); rw = w / (up * up); ral = along / (ss * w);" if up
               else "rss = ss / (sl * sl);")
    src += [
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
    src.append("")
    if up:
        src += [
            "# The axial term, as y carries it, is ss (gx ux + gy uy + gz uz)^2 times KA,",
            "# g undoing nothing and folding in the roundings of the two sums; the true",
            "# one is sl^2 (Dt)^2 = ss (ex ux + ey uy + ez uz)^2 / rss, e undoing the",
            "# roundings of f and of the product. So its error is six terms, one for each",
            "# piece of ss G^2: ss u^2 for each of the three, and 2 ss |u| |u'| for each",
            "# pair. W is each term's error over its piece.",
            "KA = rw * ral;",
            "gx = ru01 * rU; gy = ru01 * rU; gz = rU;" if ax == "n" else "gx = rU; gy = rU; gz = rU;",
            "ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);",
            "WAxx = KA * gx * gx - ex * ex / rss;",
            "WAyy = KA * gy * gy - ey * ey / rss;",
            "WAzz = KA * gz * gz - ez * ez / rss;",
            "WAxy = KA * gx * gy - ex * ey / rss;",
            "WAxz = KA * gx * gz - ex * ez / rss;",
            "WAyz = KA * gy * gz - ey * ez / rss;",
            "cAxx = ss * |ux| * |ux|; cAyy = ss * |uy| * |uy|; cAzz = ss * |uz| * |uz|;",
            "cAxy = 2 * ss * |ux| * |uy|; cAxz = 2 * ss * |ux| * |uz|; cAyz = 2 * ss * |uy| * |uz|;",
            "eAxx = -(ss * ux * ux * WAxx); eAyy = -(ss * uy * uy * WAyy); eAzz = -(ss * uz * uz * WAzz);",
            "eAxy = -(2 * ss * ux * uy * WAxy); eAxz = -(2 * ss * ux * uz * WAxz);",
            "eAyz = -(2 * ss * uy * uz * WAyz);",
            "",
            "# The same permanent, as four blocks of the fifteen pieces the error is",
            "# averaged over: each component's reach squared, and the axial term.",
            "P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;",
            "SA = cAxx + cAyy + cAzz + cAxy + cAxz + cAyz;",
            "P15 = P3x + P3y + P3z + SA;",
        ]
    else:
        src += [
            "# Where up cancels, y has no axial term, and the true one, sl^2 Dt^2, is",
            "# all error. Since u01 + uz is zero, Dt is what the roundings left: (e - g)",
            "# u for each half, e undoing the roundings of f and of the product and g",
            "# that of the first sum, so g is ru01, ru01 and 1. Dt is averaged over G,",
            "# and the error over XA.",
            "ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);",
            "Dx = ex - ru01; Dy = ey - ru01; Dz = ez - 1;",
            "D3 = Dx * ux + Dy * uy + Dz * uz;",
            "",
            "# The same permanent, as four blocks: each component's reach squared, and",
            "# the axial term.",
            "P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;",
            "P15 = P3x + P3y + P3z + XA;",
        ]
    src += ["", "# Each component's square as y carries it, and the axial term as Ex has it."]
    if live:
        src.append(" ".join(f"Y{a} = K{a} * Q{a} * Q{a};" for a in live))
    src.append("At = sl * sl * (Dt * Dt);")

    nonzero = "\n  /\\ ".join([
        "|fx| >= 1b-1074 /\\ |fy| >= 1b-1074 /\\ |fz| >= 1b-1074",
        "|dx| >= 1b-1074 /\\ |dy| >= 1b-1074 /\\ |dz| >= 1b-1074",
        "|sl| >= 1b-1074"] + ([" /\\ ".join(f"|q{a}| >= 1b-1074" for a in live)] if live else [])
        + (["|up| >= 1b-1074"] if up else []))
    if not dead and up:
        head = ["  # No factor is zero, no component cancels to zero and nor does up. For an",
                "  # f64 that is the same as being at least the smallest subnormal."]
    elif up:
        head = ["  # No factor is zero, no component but those the case names cancels to",
                "  # zero, and nor does up. For an f64 that is the same as being at least",
                "  # the smallest subnormal."]
    else:
        head = ["  # No factor is zero, and no component but those the case names cancels to",
                "  # zero. For an f64 that is the same as being at least the smallest",
                "  # subnormal."]
    src += [
        "",
        "{",
        "  # Every input is a finite f64.",
        "     |px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024",
        "  /\\ |ox| <= 1b1024 /\\ |oy| <= 1b1024 /\\ |oz| <= 1b1024",
        "  /\\ |dx| <= 1b1024 /\\ |dy| <= 1b1024 /\\ |dz| <= 1b1024",
        "  /\\ |sl| <= 1b1024",
    ] + head + ["  /\\ " + nonzero]
    if ax == "n":
        src += ["  # The first sum of up is a normal number; where it is not, it is exact,",
                "  # and that is another case.",
                "  /\\ |u01| >= 1b-1022"]
    elif ax == "e":
        src += ["  # The first sum of up is below the normals, where it is exact.",
                "  /\\ |u01| <= 1b-1022"]
    else:
        src += ["  # up cancels: u01 + uz is zero, so u01 is -uz, which is normal.",
                "  /\\ u01 + uz in [0, 0] /\\ |u01| >= 1b-1022"]
    src += [
        "  # product_is_normal on the six halves (cross_product_halves_if_normal),",
        "  # and on the three components, the three up halves, the slope, up and the",
        "  # axial term (stays_accurate).",
        "  /\\ |hx| >= 1b-1022 /\\ |kx| >= 1b-1022 /\\ |hy| >= 1b-1022",
        "  /\\ |ky| >= 1b-1022 /\\ |hz| >= 1b-1022 /\\ |kz| >= 1b-1022",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in live))
    src.append("  /\\ |ux| >= 1b-1022 /\\ |uy| >= 1b-1022 /\\ |uz| >= 1b-1022")
    src.append("  /\\ |ss| >= 1b-1022 /\\ |w| >= 1b-1022 /\\ |along| >= 1b-1022" if up
               else "  /\\ |ss| >= 1b-1022")
    if not dead:
        src += [
            "  # The permanent's three squares are normal too, since no component is",
            "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,",
            "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal."
            + (" In the same way" if up else ""),
        ]
        if up:
            src.append("  # |up| <= R, so R2 >= w and A >= along, both normal.")
        src.append("  /\\ |vx| >= 1b-1022 /\\ |vy| >= 1b-1022 /\\ |vz| >= 1b-1022")
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
        if up:
            src.append("  # |up| <= R, so R2 >= w and A >= along, both normal.")
    if up:
        src.append("  /\\ |R2| >= 1b-1022 /\\ |A| >= 1b-1022")
    else:
        src.append("  # Where up cancels, the case says what R2 and A are.")
        r2 = "XR <= 1b-1022 /\\ R2 >= 1b-1074" if xr else "|R2| >= 1b-1022"
        a_ ="XA <= 1b-1022" if xa_low else "|A| >= 1b-1022"
        src.append(f"  /\\ {r2} /\\ {a_}")
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
    for p in ("ux -/ fx * dx", "uy -/ fy * dy", "uz -/ fz * dz"):
        src.append(f"  /\\ {p} {LEMMA}")
    if ax != "e":
        src.append(f"  /\\ u01 -/ U01 {LEMMA}")
    src.append(f"  /\\ ss -/ sl * sl {LEMMA}")
    if up:
        src.append(f"  /\\ w -/ up * up {LEMMA}")
        src.append(f"  /\\ along -/ ss * w {LEMMA}")
    for a in AXES:
        if st[a] in "nc":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    if not xr:
        src.append(f"  /\\ R2 -/ R * R {LEMMA}")
    if not xa_low:
        src.append(f"  /\\ A -/ ss * R2 {LEMMA}" if up else f"  /\\ A -/ XA {LEMMA}")
    src += [
        f"  /\\ eb -/ B * P {LEMMA}",
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
    ]
    for a in AXES:
        src.append(f"  /\\ h{a} / |h{a}| in [-1, 1]" + ("" if a in dead else f" /\\ k{a} / |k{a}| in [-1, 1]"))
    src.append("  /\\ ux / |ux| in [-1, 1] /\\ uy / |uy| in [-1, 1] /\\ uz / |uz| in [-1, 1]")
    src.append("->")
    if shifted:
        src.append("  (y - Ex) / P15 in ? /\\ P -/ Phatx in ? /\\ P in ?")
    else:
        src.append("  (y - Ex) / P15 in ? /\\ P -/ Phat in ?")
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
        "ux / (fx * dx) -> 1 + (ux - fx * dx) / (fx * dx);",
        "uy / (fy * dy) -> 1 + (uy - fy * dy) / (fy * dy);",
        "uz / (fz * dz) -> 1 + (uz - fz * dz) / (fz * dz);",
    ]
    if ax != "e":
        src.append("u01 / U01 -> 1 + (u01 - U01) / U01;")
    if up:
        src.append("up / U -> 1 + (up - U) / U;")
    src.append("ss / (sl * sl) -> 1 + (ss - sl * sl) / (sl * sl);")
    if up:
        src += ["w / (up * up) -> 1 + (w - up * up) / (up * up);",
                "along / (ss * w) -> 1 + (along - ss * w) / (ss * w);"]
    src += [
        "",
        "# The error is the sum of four blocks' errors, each the sum of its pieces'",
        "# errors, each a fraction of its own piece of the permanent; add_rr then",
    ]
    if up:
        src += ["# averages them. In blocks so that no hint's identity spans all fifteen",
                "# pieces, as cyl.g learned."]
    else:
        src += ["# averages them. In blocks so that no hint's identity spans every piece,",
                "# as cyl.g learned."]
    if dead:
        src += ["# A component that cancels is one block, off by its true square, which is",
                "# (a - b)^2 / 4 of its reach squared, 4 h^2."]
    src.append("(y - Ex) / P15 -> ((P15 + (y - Ex)) - P15) / P15;")
    blocks = [f"(P3{a} + (Y{a} - C{a} * C{a}))" if a in live else f"(P3{a} + eC{a})" for a in AXES]
    src.append("P15 + (y - Ex) -> " + " + ".join(blocks)
               + (" + (SA + (At - along));" if up else " + (XA + At);"))
    for a in live:
        src.append(f"P3{a} + (Y{a} - C{a} * C{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
    if up:
        src.append("SA + (At - along) -> (cAxx + eAxx) + (cAyy + eAyy) + (cAzz + eAzz) + (cAxy + eAxy) + (cAxz + eAxz) + (cAyz + eAyz);")
    for a in AXES:
        if a in live:
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        else:
            src.append(f"(P3{a} + eC{a}) - P3{a} -> eC{a};")
    if up:
        for p in ("xx", "yy", "zz", "xy", "xz", "yz"):
            src.append(f"(cA{p} + eA{p}) - cA{p} -> eA{p};")
    else:
        src.append("(XA + At) - XA -> At;")
    for a in AXES:
        if a in live:
            src += [f"eH{a} / cH{a} -> (h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a};",
                    f"eK{a} / cK{a} -> (k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a};",
                    f"eX{a} / cX{a} -> -((h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a});"]
        else:
            src.append(f"eC{a} / P3{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * (a{a} - b{a}) * (a{a} - b{a}) / 4);")
    if up:
        src += [
            "eAxx / cAxx -> -((ux / |ux|) * (ux / |ux|) * WAxx);",
            "eAyy / cAyy -> -((uy / |uy|) * (uy / |uy|) * WAyy);",
            "eAzz / cAzz -> -((uz / |uz|) * (uz / |uz|) * WAzz);",
            "eAxy / cAxy -> -((ux / |ux|) * (uy / |uy|) * WAxy);",
            "eAxz / cAxz -> -((ux / |ux|) * (uz / |uz|) * WAxz);",
            "eAyz / cAyz -> -((uy / |uy|) * (uz / |uz|) * WAyz);",
        ]
    else:
        src += [
            "# The axial error over XA: Dt over G, twice, G^2 over R2, and the slope's",
            "# rounding. Dt over G is the average of what each half's roundings left.",
            "At / XA -> (Dt / G) * (Dt / G) * ((G * G) / R2) / rss;",
            "Dt / G -> D3 / G + (u01 + uz) / G;",
            "D3 / G -> ((G + D3) - G) / G;",
            "G + D3 -> (|ux| + Dx * ux) + (|uy| + Dy * uy) + (|uz| + Dz * uz);",
            "(|ux| + Dx * ux) - |ux| -> Dx * ux;",
            "(|uy| + Dy * uy) - |uy| -> Dy * uy;",
            "(|uz| + Dz * uz) - |uz| -> Dz * uz;",
            "(Dx * ux) / |ux| -> Dx * (ux / |ux|);",
            "(Dy * uy) / |uy| -> Dy * (uy / |uy|);",
            "(Dz * uz) / |uz| -> Dz * (uz / |uz|);",
        ]
        if xr:
            src += [
                "# R2 below the normals is off from R^2 by an amount, at most half of",
                "# R2's own least value.",
                "(G * G) / R2 -> ((G * G) / XR) * (XR / R2);",
                "(G * G) / XR -> 1 / (1 + (XR - G * G) / (G * G));",
                "XR / R2 -> 1 + (XR - R2) / R2;",
            ]
        else:
            src.append("(G * G) / R2 -> 1 / (1 + (R2 - G * G) / (G * G));")
    src += ["", "# From the " + ("fifteen pieces" if up else "blocks")
            + " to the permanent on_floats rounds, and on to eb.",
            "(y - Ex) / eb -> ((y - Ex) / P15) * (Phat / P) * (P / eb);"]
    if shifted:
        shift = [f"(Xv{a} - v{a})" if st[a] == "s" else f"Xv{a}" for a in small]
        if xa_low:
            shift.append("(XA - A)")
        src += ["# What rounding a square below the normals cost is an amount, and",
                "# eb being normal keeps P large enough that it is small against eb.",
                f"Phat / P -> Phatx / P + ({' + '.join(shift)}) / P;",
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


for cross in map("".join, itertools.product("ncsu", repeat=3)):
    for ax in "nec":
        case = cross + ax
        if case == "nnnn":
            assert make(case) == open("cone.g").read()
            continue
        write(f"cone_{case}.g", make(case))
        write(f"c_cone_{case}_half.g", make(case, B_HALF))
        if ax not in UPS and "n" not in cross:
            write(f"c_cone_{case}_usq.g", make(case, B_USQ))
