"""Write tor.g's case split, one script per case and a control at B = u / 2
beside each. A case is five letters. The first three are the components x, y
and z of the cross product, as cylsplit.py (464.15) has them:

n  it does not cancel: q = rnd(h - k) is not zero, so its square s is
   normal (stays_accurate), and so is its reach square v, which is never
   below s;
c  it cancels to zero, h == k, and its reach square v = rnd(4 h^2) is
   normal;
s  it cancels, and v is below the normals and does not round to zero;
u  it cancels, and v rounds to zero.

The fourth is hollow = rnd(RR - rr):

n  it is normal;
e  it is below the normals, where it is exact (464.16's sumexact.g), so it
   is written unrounded.

The fifth is the permanent's P2 = rnd(F4 * spread):

n  it is normal. Where a component does not cancel, spread >= across, so
   P2 >= right, which is normal (stays_accurate), and n is the only state;
b  all three components cancel, so right is zero, and P2 is below the
   normals or zero. Where all three reach squares round to zero, uuu, spread
   and P2 are zero and P is P1, and b is the only state.

That is 37 * 2 + 26 * 2 * 2 + 2 = 180 cases, and tor_nnnnn.g is tor.g, byte
for byte. A component that cancels has no square in y, so its error is the
whole of its true term, eC = 4 R^2 C^2. It is averaged over the piece
cOR = 2 W oab RR of the w.w bulge^2 block rather than over its own reach
square, so that what v or P2 rounds to does not matter to it. The names are
tor_<case>.g and c_tor_<case>_half.g.
Run as: python3 torsplit.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
B_CODE = "B = 18b-53 + 7b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
STATE = {"n": "does not cancel", "c": "cancels, its reach square normal",
         "s": "cancels, its reach square below the normals and not zero",
         "u": "cancels, its reach square rounding to zero"}
HOLLOW = {"n": "hollow normal", "e": "hollow below the normals"}
SPREAD = {"n": "P2 normal", "b": "P2 below the normals or zero"}


def reach(a):
    return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"


def left_sum(terms):
    """Sum terms left to right as Rust adds them, each sum rounded."""
    acc = None
    for t in terms:
        acc = t if acc is None else f"rnd({acc} + {t})"
    return acc


def over_sum(v, a, r1, r2, total):
    """v_a / total as 1 over total / v_a, total being r2 (r1 (vx + vy) + vz)."""
    xy = " + ".join("1" if b == a else f"{v}{b} / {v}{a}" for b in "xy")
    z = "1" if a == "z" else f"{v}z / {v}{a}"
    return f"{v}{a} / {total} -> 1 / ({r2} * ({r1} * ({xy}) + {z}));"


def make(case, b=B_CODE):
    st = dict(zip(AXES, case[:3]))
    hol = case[3] == "n"
    p2 = case[4]
    live = [a for a in AXES if st[a] == "n"]
    dead = [a for a in AXES if st[a] != "n"]
    small = [a for a in AXES if st[a] in "su"]
    kept = [a for a in AXES if st[a] != "u"]
    vdead = [a for a in dead if st[a] in "cs"]
    live01 = [a for a in "xy" if a in live]
    s01_rounds = len(live01) == 2
    s_rounds = bool(live01) and "z" in live
    split = case != "nnnnn"
    src = [
        "# restates: crates/rcad-geom/src/torus.rs on_floats at 4be293a5",
        "# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 4be293a5",
        "# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 4be293a5",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 4be293a5",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 4be293a5",
        "# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.",
    ]
    if split:
        src.append(f"# The case {case}: " + "; ".join(f"{a} {STATE[st[a]]}" for a in AXES)
                   + f"; {HOLLOW[case[3]]}; {SPREAD[p2]}.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre, the axis direction and the two radii, each an f64.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        "ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);",
        "dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);",
        "rg = rnd(rg_); tb = rnd(tb_);",
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
        "# squares and out_and_back, left to right as Rust adds them.",
        "gx = rnd(fx * fx); gy = rnd(fy * fy); gz = rnd(fz * fz);",
        "O01 = gx + gy; o01 = rnd(O01);",
        "O = o01 + gz; oab = rnd(O);",
        "",
        "RR = rnd(rg * rg); rr = rnd(tb * tb);",
    ]
    if hol:
        src.append("H = RR - rr; hollow = rnd(H);")
    else:
        src += ["# hollow is below the normals, where it is exact (464.16's sumexact.g).",
                "H = RR - rr; hollow = H;"]
    src += [
        "Bu = oab + hollow; bulge = rnd(Bu);",
        "",
        "# axis_squared.",
        "tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);",
        "T01 = tx + ty; t01 = rnd(T01);",
        "T = t01 + tz; W = rnd(T);",
        "",
        "# across.",
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
    if s_rounds:
        src.append("S = s01 + sz; across = rnd(S);")
    elif live01:
        src.append("across = s01;")
    elif live:
        src.append("across = sz;")
    else:
        src.append("# All three cancel, so across and right are zero, and y is left.")
    src += [
        "",
        "F4 = rnd(4 * RR);",
        "bb = rnd(bulge * bulge);",
        "left = rnd(W * bb);",
    ]
    if live:
        src += ["right = rnd(F4 * across);", "y = left - right;"]
    else:
        src.append("y = left;")
    src += [
        "value = rnd(y);",
        "",
        "# The permanent. spread is a sum() from zero, and adding the first square to",
        "# zero is exact.",
    ]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (464.15's dbl.g), so it is written unrounded."]
    src.append(" ".join(f"n{a} = rnd(|h{a}| + |k{a}|);" if a in live else f"n{a} = |h{a}| + |k{a}|;"
                        for a in AXES))
    if small:
        src.append(" ".join(f"Xv{a} = n{a} * n{a};" for a in small))
    if "u" in case[:3]:
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
    if kept:
        src.append(f"spread = {left_sum([f'v{a}' for a in kept])};")
    src += [
        "br = rnd(rnd(oab + RR) + rr);",
        "brr = rnd(br * br);",
        "P1 = rnd(W * brr);",
    ]
    if not kept:
        src.append("# spread and P2 are zero, and adding zero to P1 leaves it.")
        src.append("P = P1;")
    else:
        src.append("P2 = rnd(F4 * spread);" if p2 == "n" else "XP2 = F4 * spread; P2 = rnd(XP2);")
        src.append("P = rnd(P1 + P2);")
    src += [
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;",
        "Wt = dx * dx + dy * dy + dz * dz;",
        "G2 = Fx * Fx + Fy * Fy + Fz * Fz;",
        "Bt = G2 + rg * rg - tb * tb;",
        "Lt = Wt * (Bt * Bt);",
        "Rt = 4 * (rg * rg) * (Cx * Cx + Cy * Cy + Cz * Cz);",
        "Ex = Lt - Rt;",
        "",
        "# The permanent with no rounding of its own.",
        "Rc = oab + RR + rr;",
        "Phat = W * (Rc * Rc)",
        "     + F4 * ((|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)",
        "             + (|hz| + |kz|) * (|hz| + |kz|));",
    ]
    dx_terms = None
    if split:
        if p2 == "b" and kept:
            src += ["# The same with P2 in place of the 4 R^2 product, as P adds them; Dx is",
                    "# what that adds to the two blocks below."]
            src.append("Phatx = W * (Rc * Rc) + P2;")
            dx_terms = "P2"
        elif p2 == "b":
            src.append("# The same with no 4 R^2 product, as P is P1.")
            src.append("Phatx = W * (Rc * Rc);")
        else:
            src += ["# The same with v in place of the reach square of each component that",
                    "# cancels, and without each that rounds to zero; Dx is what that adds",
                    "# to the two blocks below."]
            terms = [reach(a) if a in live else f"v{a}" for a in AXES if st[a] != "u"]
            src.append(f"Phatx = W * (Rc * Rc) + F4 * ({' + '.join(terms)});")
            if vdead:
                dx_terms = f"F4 * ({' + '.join(f'v{a}' for a in vdead)})"
        if dx_terms:
            src.append(f"Dx = {dx_terms};")
    src += [
        "",
        "# What each rounding multiplied its argument by.",
        "rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;",
        "rhx = hx / (fy * dz); rkx = kx / (fz * dy);",
        "rhy = hy / (fz * dx); rky = ky / (fx * dz);",
        "rhz = hz / (fx * dy); rkz = kz / (fy * dx);",
    ]
    if live:
        src.append(" ".join(f"rq{a} = q{a} / Q{a};" for a in live))
    src += [
        "rgx = gx / (fx * fx); rgy = gy / (fy * fy); rgz = gz / (fz * fz);",
        "ro01 = o01 / O01; ro = oab / O;",
        "rRR = RR / (rg * rg); rrr = rr / (tb * tb);",
        "rH = hollow / H; rBu = bulge / Bu;" if hol else "rBu = bulge / Bu;",
        "rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);",
        "rt01 = t01 / T01; rW = W / T;",
    ]
    if live:
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in live))
    rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rS = across / S;"] if s_rounds else [])
    if rs:
        src.append(" ".join(rs))
    src.append("rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);")
    src.append("rL = left / (W * bb); rR = right / (F4 * across);" if live else "rL = left / (W * bb);")
    src += [
        "",
        "# The w.w bulge^2 block. W and oab as their three pieces, which sum to them.",
        "Wx = rW * rt01 * tx; Wy = rW * rt01 * ty; Wz = rW * tz;",
        "W3 = Wx + Wy + Wz;",
        "jx = 1 / (rW * rt01 * rtx) - 1; jy = 1 / (rW * rt01 * rty) - 1; jz = 1 / (rW * rtz) - 1;",
        "om = Wt / W;",
        "Ox = ro * ro01 * gx; Oy = ro * ro01 * gy; Oz = ro * gz;",
        "O3 = Ox + Oy + Oz;",
        "zx = 1 / (ro * ro01 * rgx * rfx * rfx) - 1;",
        "zy = 1 / (ro * ro01 * rgy * rfy * rfy) - 1;",
        "zz = 1 / (ro * rgz * rfz * rfz) - 1;",
        "Z = G2 / oab;",
        "",
        "# bulge as y carries it is rBu (oab + rH (RR - rr)); the true one is",
        "# Z oab + RR / rRR - rr / rrr. Squared and times W, that is six pieces, one",
        "# for each pair of oab, RR and rr; W is each piece's error over the piece.",
    ]
    if not hol:
        src.append("# hollow is exact here, so rH is 1 and is left out.")
    six = [
        "KL = rL * rbb * rBu * rBu;",
        "WOO = KL - om * Z * Z;",
        "WRR = KL * rH * rH - om / (rRR * rRR);",
        "Wrr = KL * rH * rH - om / (rrr * rrr);",
        "WOR = KL * rH - om * Z / rRR;",
        "WOr = KL * rH - om * Z / rrr;",
        "WRr = KL * rH * rH - om / (rRR * rrr);",
    ]
    if not hol:
        six = [line.replace(" * rH * rH", "").replace(" * rH", "") for line in six]
    src += six
    src += [
        "cOO = W * oab * oab; cRR = W * RR * RR; crr = W * rr * rr;",
        "cOR = 2 * W * oab * RR; cOr = 2 * W * oab * rr; cRr = 2 * W * RR * rr;",
        "eOO = cOO * WOO; eRR = cRR * WRR; err = crr * Wrr;",
        "eOR = cOR * WOR; eOr = -(cOr * WOr); eRr = -(cRr * WRr);",
        "A6 = cOO + cRR + crr + cOR + cOr + cRr;",
        "",
        "# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's",
        "# rounding, m undoes F4's and RR's.",
    ]
    if dead:
        src.append("# A component that cancels has no square in y, and is not in it.")
    for a in live:
        carry = ["rR"] + (["rS"] if s_rounds else []) + (["rs01"] if a in "xy" and s01_rounds else [])
        src.append(f"K{a} = " + " * ".join(carry + [f"rs{a}", f"rq{a}", f"rq{a}"]) + ";")
    src.append("m = 1 / (rF4 * rRR);")
    for a in AXES:
        src.append(f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a}); b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});")
    for a in live:
        src.append(f"WH{a} = K{a} - m * a{a} * a{a}; WK{a} = K{a} - m * b{a} * b{a}; WX{a} = K{a} - m * a{a} * b{a};")
    for a in live:
        src.append(f"cH{a} = F4 * |h{a}| * |h{a}|; cK{a} = F4 * |k{a}| * |k{a}|; cX{a} = 2 * F4 * |h{a}| * |k{a}|;")
    for a in live:
        src.append(f"eH{a} = -(F4 * h{a} * h{a} * WH{a}); eK{a} = -(F4 * k{a} * k{a} * WK{a}); "
                   f"eX{a} = 2 * F4 * h{a} * k{a} * WX{a};")
    if live:
        src.append(" ".join(f"C3{a} = cH{a} + cK{a} + cX{a};" for a in live))
        src.append(f"C9 = {' + '.join(f'C3{a}' for a in live)};")
        src.append(" ".join(f"Y{a} = F4 * K{a} * Q{a} * Q{a};" for a in live))
        src.append(" ".join(f"Z{a} = F4 * m * (C{a} * C{a});" for a in live))
    if dead:
        src += ["# What y misses of a component that cancels is its whole true term, and",
                "# EC is their sum."]
        src.append(" ".join(f"eC{a} = F4 * m * (C{a} * C{a});" for a in dead))
        ec = f"eC{dead[0]}"
        if len(dead) > 1:
            src.append(f"EC = {' + '.join(f'eC{a}' for a in dead)};")
            ec = "EC"
    src += [
        "",
        "# The same permanent as the two blocks.",
        "Phat2 = A6 + C9;" if live else "Phat2 = A6;",
        "",
        "{",
        "  # Every input is a finite f64.",
        "     |px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024",
        "  /\\ |ox| <= 1b1024 /\\ |oy| <= 1b1024 /\\ |oz| <= 1b1024",
        "  /\\ |dx| <= 1b1024 /\\ |dy| <= 1b1024 /\\ |dz| <= 1b1024",
        "  /\\ |rg| <= 1b1024 /\\ |tb| <= 1b1024",
    ]
    if dead:
        src += ["  # No factor is zero, and no component but those the case names cancels to",
                "  # zero. For an f64 that is the same as being at least the smallest",
                "  # subnormal."]
    else:
        src += ["  # No factor is zero, and no component cancels to zero. For an f64 that is",
                "  # the same as being at least the smallest subnormal."]
    src += [
        "  /\\ |fx| >= 1b-1074 /\\ |fy| >= 1b-1074 /\\ |fz| >= 1b-1074",
        "  /\\ |dx| >= 1b-1074 /\\ |dy| >= 1b-1074 /\\ |dz| >= 1b-1074",
        "  /\\ |rg| >= 1b-1074 /\\ |tb| >= 1b-1074",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|q{a}| >= 1b-1074" for a in live))
    if hol:
        src += ["  # hollow is a normal number; below the normals it is exact, and that is",
                "  # another case.",
                "  /\\ |hollow| >= 1b-1022"]
    else:
        src += ["  # hollow is below the normals, where it is exact.",
                "  /\\ |hollow| <= 1b-1022"]
    src += [
        "  # product_is_normal on the three squares of from_centre, of direction and",
        "  # of sideways, the six halves, R^2, r^2, bulge^2, axis_squared bulge^2 and",
        "  # 4 R^2 across (stays_accurate and the check after it).",
        "  /\\ |gx| >= 1b-1022 /\\ |gy| >= 1b-1022 /\\ |gz| >= 1b-1022",
        "  /\\ |tx| >= 1b-1022 /\\ |ty| >= 1b-1022 /\\ |tz| >= 1b-1022",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in live))
    src += [
        "  /\\ |hx| >= 1b-1022 /\\ |kx| >= 1b-1022 /\\ |hy| >= 1b-1022",
        "  /\\ |ky| >= 1b-1022 /\\ |hz| >= 1b-1022 /\\ |kz| >= 1b-1022",
        "  /\\ |RR| >= 1b-1022 /\\ |rr| >= 1b-1022",
        "  /\\ |bb| >= 1b-1022 /\\ |left| >= 1b-1022 /\\ |right| >= 1b-1022" if live
        else "  /\\ |bb| >= 1b-1022 /\\ |left| >= 1b-1022",
    ]
    if not dead:
        src += [
            "  # The permanent's products are normal too, since nothing cancels here, by",
            "  # monotone rounding: v >= s, as no component is zero; br >= bulge, as",
            "  # hollow <= RR, so brr >= bb and P1 >= left; spread >= across, so",
            "  # P2 >= right.",
            "  /\\ |vx| >= 1b-1022 /\\ |vy| >= 1b-1022 /\\ |vz| >= 1b-1022",
            "  /\\ |brr| >= 1b-1022 /\\ |P1| >= 1b-1022 /\\ |P2| >= 1b-1022",
        ]
    else:
        src += [
            "  # The permanent's products, by monotone rounding: v >= s for a component",
            "  # that does not cancel; for one that does the case says which v is, and",
            "  # 464.14's tinysq.g and tinyzero.g split them at 2^-1075. br >= bulge, as",
            "  # hollow <= RR, so brr >= bb and P1 >= left. "
            + ("spread >= across, so P2 >= right." if live else "The case says what P2 is."),
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
        tail = {"n": " /\\ |P2| >= 1b-1022", "b": " /\\ XP2 <= 1b-1022" if kept else ""}[p2]
        src.append("  /\\ |brr| >= 1b-1022 /\\ |P1| >= 1b-1022" + tail)
    src += [
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
        "  # What lemma.g proves each of those gives.",
    ]
    if dead:
        src.append("  # k is h in a component that cancels, so its line is h's against k's product.")
    if hol:
        src.append(f"  /\\ hollow -/ H {LEMMA}")
    for p in ("gx -/ fx * fx", "gy -/ fy * fy", "gz -/ fz * fz",
              "tx -/ dx * dx", "ty -/ dy * dy", "tz -/ dz * dz"):
        src.append(f"  /\\ {p} {LEMMA}")
    for a in live:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for p in ("RR -/ rg * rg", "rr -/ tb * tb", "bb -/ bulge * bulge", "left -/ W * bb"):
        src.append(f"  /\\ {p} {LEMMA}")
    if live:
        src.append(f"  /\\ right -/ F4 * across {LEMMA}")
    for a in AXES:
        if st[a] in "nc":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    src.append(f"  /\\ brr -/ br * br {LEMMA}")
    src.append(f"  /\\ P1 -/ W * brr {LEMMA}")
    if p2 == "n":
        src.append(f"  /\\ P2 -/ F4 * spread {LEMMA}")
    src.append(f"  /\\ eb -/ B * P {LEMMA}")
    if live:
        src.append("  # x / |x| is in [-1, 1] for any x that is not zero.")
        for a in live:
            src.append(f"  /\\ h{a} / |h{a}| in [-1, 1] /\\ k{a} / |k{a}| in [-1, 1]")
    src += [
        "->",
        "  (y - Ex) / Phat2 in ? /\\ P -/ " + ("Phatx" if split else "Phat") + " in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
        "# How far each rounding is from the identity.",
        "fx / Fx -> 1 + (fx - Fx) / Fx;",
        "fy / Fy -> 1 + (fy - Fy) / Fy;",
        "fz / Fz -> 1 + (fz - Fz) / Fz;",
    ]
    for a in AXES:
        for h, prod in zip("hk", HALVES[a]):
            src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in live:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    src += [
        "gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);",
        "gy / (fy * fy) -> 1 + (gy - fy * fy) / (fy * fy);",
        "gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);",
        "o01 / O01 -> 1 + (o01 - O01) / O01;",
        "oab / O -> 1 + (oab - O) / O;",
        "RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);",
        "rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);",
    ]
    if hol:
        src.append("hollow / H -> 1 + (hollow - H) / H;")
    src += [
        "bulge / Bu -> 1 + (bulge - Bu) / Bu;",
        "tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);",
        "ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);",
        "tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);",
        "t01 / T01 -> 1 + (t01 - T01) / T01;",
        "W / T -> 1 + (W - T) / T;",
    ]
    for a in live:
        src.append(f"s{a} / (q{a} * q{a}) -> 1 + (s{a} - q{a} * q{a}) / (q{a} * q{a});")
    if s01_rounds:
        src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
    if s_rounds:
        src.append("across / S -> 1 + (across - S) / S;")
    src += [
        "F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);",
        "bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);",
        "left / (W * bb) -> 1 + (left - W * bb) / (W * bb);",
    ]
    if live:
        src.append("right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);")
    src += [
        "",
        "# w.w over W, and |x|^2 over oab, each averaged over three pieces.",
        "Wt / W -> 1 + ((W3 + (Wt - W)) - W3) / W3;",
        "W3 + (Wt - W) -> (Wx + Wx * jx) + (Wy + Wy * jy) + (Wz + Wz * jz);",
        "(Wx + Wx * jx) - Wx -> Wx * jx;",
        "(Wy + Wy * jy) - Wy -> Wy * jy;",
        "(Wz + Wz * jz) - Wz -> Wz * jz;",
        "G2 / oab -> 1 + ((O3 + (G2 - oab)) - O3) / O3;",
        "O3 + (G2 - oab) -> (Ox + Ox * zx) + (Oy + Oy * zy) + (Oz + Oz * zz);",
        "(Ox + Ox * zx) - Ox -> Ox * zx;",
        "(Oy + Oy * zy) - Oy -> Oy * zy;",
        "(Oz + Oz * zz) - Oz -> Oz * zz;",
        "",
        "# The error is the sum of the two blocks' errors, each the sum of its",
        "# pieces' errors, each a fraction of its own piece of the permanent; add_rr",
        "# then averages them.",
    ]
    if dead:
        src += ["# What y misses of a component that cancels goes with the piece cOR, and",
                "# is at most a u^2 of it."]
    src.append("(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;")
    oor = "(cOR + eOR)"
    if dead:
        oor = f"(cOR + (eOR + {ec}))"
        a6 = f"(A6 + ((left - Lt) + {ec}))"
        src.append("Phat2 + (y - Ex) -> " + (f"{a6} + (C9 + (Rt - right - {ec}));" if live else f"{a6};"))
        src.append(f"A6 + ((left - Lt) + {ec}) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + {oor} "
                   "+ (cOr + eOr) + (cRr + eRr);")
    else:
        src.append("Phat2 + (y - Ex) -> (A6 + (left - Lt)) + (C9 + (Rt - right));")
        src.append("A6 + (left - Lt) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + eOR) "
                   "+ (cOr + eOr) + (cRr + eRr);")
    src += [
        "(cOO + eOO) - cOO -> eOO;",
        "(cRR + eRR) - cRR -> eRR;",
        "(crr + err) - crr -> err;",
        f"(cOR + (eOR + {ec})) - cOR -> eOR + {ec};" if dead else "(cOR + eOR) - cOR -> eOR;",
        "(cOr + eOr) - cOr -> eOr;",
        "(cRr + eRr) - cRr -> eRr;",
        "eOr / cOr -> -WOr;",
        "eRr / cRr -> -WRr;",
    ]
    if dead:
        src.append(f"(eOR + {ec}) / cOR -> WOR + {ec} / cOR;")
        if len(dead) > 1:
            src.append(f"EC / cOR -> {' + '.join(f'eC{a} / cOR' for a in dead)};")
        for a in dead:
            f_, d_ = HALVES[a][0][1], HALVES[a][0][-1]
            src += [
                f"eC{a} / cOR -> 2 * rF4 * m * (a{a} - b{a}) * (a{a} - b{a}) * rh{a} * rh{a}"
                f" * (f{f_} * f{f_} / oab) * (d{d_} * d{d_} / W);",
                f"f{f_} * f{f_} / oab -> (g{f_} / oab) / rg{f_};",
                over_sum("g", f_, "ro01", "ro", "oab"),
                f"d{d_} * d{d_} / W -> (t{d_} / W) / rt{d_};",
                over_sum("t", d_, "rt01", "rW", "W"),
            ]
    if live:
        rest = f"C9 + (Rt - right - {ec})" if dead else "C9 + (Rt - right)"
        src.append(f"{rest} -> " + " + ".join(f"(C3{a} + (Z{a} - Y{a}))" for a in live) + ";")
        for a in live:
            src.append(f"C3{a} + (Z{a} - Y{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
        for a in live:
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        for a in live:
            src += [f"eH{a} / cH{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a});",
                    f"eK{a} / cK{a} -> -((k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a});",
                    f"eX{a} / cX{a} -> (h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a};"]
    src += ["", "# From the two blocks to the permanent on_floats rounds, and on to eb."]
    if not split:
        src += ["(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat / P) * (P / eb);",
                "Phat / P -> 1 / (1 + (P - Phat) / Phat);"]
    elif dx_terms:
        src += ["(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat2 / P) * (P / eb);",
                "Phat2 / P -> (Phat2 / Phatx) * (Phatx / P);",
                "Phat2 / Phatx -> 1 / (1 + Dx / Phat2);",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);"]
    else:
        src += ["(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phatx / P) * (P / eb);",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);"]
    src.append("P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));")
    return "\n".join(src) + "\n"


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


def cases():
    for cross in map("".join, itertools.product("ncsu", repeat=3)):
        for hollow in "ne":
            if "n" in cross:
                yield cross + hollow + "n"
            elif cross == "uuu":
                yield cross + hollow + "b"
            else:
                yield cross + hollow + "n"
                yield cross + hollow + "b"


assert make("nnnnn") == open("tor.g").read(), "tor_nnnnn.g is tor.g"
for case in cases():
    if case != "nnnnn":
        write(f"tor_{case}.g", make(case))
        write(f"c_tor_{case}_half.g", make(case, B_HALF))
