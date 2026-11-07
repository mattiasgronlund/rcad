"""Write tor.g's cases where a difference or a direction component is zero,
one script per case and a control at B = u / 2 beside each.

product_is_normal(a, b) is true when a or b is zero, so on_floats lets a zero
difference f and a zero direction component d through. A half of the cross
product, or a square, with a zero factor is then exactly 0, and its true
product is 0 too, since a rounded difference of two doubles is zero only when
the two are the same double. So each component of the cross product is one of:

h  its right half is zero: q = h exactly, its square s = rnd(h^2) is normal
   (stays_accurate), and its reach is |h|, so its reach square is s again;
k  its left half is zero: q = -k, and the same with k;
0  both halves are zero: q, s and the reach are 0 and its true square is 0,
   so it drops out of y, P and the error;
n, c, s, u  both halves live, as in torsplit.py.

A component keeps both halves only when none of its four factors is zero, so
with any factor zero at most one does, in the sets {fa}, {da} and {fa, da}.
The other two letters are torsplit.py's: hollow normal (n) or below the
normals (e), and P2 normal (n) or below the normals or zero (b), b also where
spread is zero and P is P1. Where any component's square is in across, P2 is
at least right and n is its only state. Left out: a zero set holding all three
direction components, since Torus's axis has no zero direction (torus.rs, the
doc on basis); and, with all three differences zero, which puts the point at
the centre, hollow below the normals: out_and_back is 0, so bulge is hollow
exactly, bulge^2 is not normal, and on_floats returns None. hollow is not
zero, since Torus::new keeps the tube narrower than the ring and the squares
of two normal doubles round apart. At the centre the
w.w bulge^2 block is the three pieces of (RR - rr)^2. The names are
torz_<zeros>_<state><hollow><P2>.g and c_torz_<zeros>_<state><hollow><P2>_half.g,
the state left out where no component keeps both halves.
Run as: python3 torzero.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
FACTORS = ("fx", "fy", "fz", "dx", "dy", "dz")
B_CODE = "B = 18b-53 + 7b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
SAYS = {"n": "does not cancel", "c": "cancels, its reach square normal",
        "s": "cancels, its reach square below the normals and not zero",
        "u": "cancels, its reach square rounding to zero",
        "h": "has its right half zero", "k": "has its left half zero", "0": "has both halves zero"}
HOLLOW = {"n": "hollow normal", "e": "hollow below the normals"}
SPREAD = {"n": "P2 normal", "b": "P2 below the normals or zero"}
NAMES = {"f": "difference", "d": "direction component"}


def kinds(zero):
    kind = {}
    for a in AXES:
        live = tuple(not set(p.split(" * ")) & zero for p in HALVES[a])
        kind[a] = {(True, True): "b", (True, False): "h", (False, True): "k", (False, False): "0"}[live]
    assert list(kind.values()).count("b") <= 1, zero
    return kind


def left_sum(terms):
    """Sum terms left to right as Rust adds them, each sum rounded."""
    acc = None
    for t in terms:
        acc = t if acc is None else f"rnd({acc} + {t})"
    return acc


def over_sum(v, a, live, r1, r2, total):
    """v_a / total as 1 over total / v_a, total being the live v summed as Rust
    adds them: r2 (r1 (vx + vy) + vz) for three, r2 (vp + vq) for two."""
    def part(b):
        return "1" if b == a else f"{v}{b} / {v}{a}"
    if len(live) == 3:
        return f"{v}{a} / {total} -> 1 / ({r2} * ({r1} * ({part('x')} + {part('y')}) + {part('z')}));"
    assert len(live) == 2, live
    return f"{v}{a} / {total} -> 1 / ({r2} * ({' + '.join(part(b) for b in live)}));"


def make(zero, state, hol_letter, p2, b=B_CODE):
    kind = kinds(zero)
    st = {a: state if kind[a] == "b" else kind[a] for a in AXES}
    hol = hol_letter == "n"
    fl = [a for a in AXES if f"f{a}" not in zero]
    dl = [a for a in AXES if f"d{a}" not in zero]
    hlive = [a for a in AXES if st[a] in "ncsuh"]
    klive = [a for a in AXES if st[a] in "ncsuk"]
    nq = [a for a in AXES if st[a] == "n"]
    one = [a for a in AXES if st[a] in "hk"]
    sq = [a for a in AXES if st[a] in "nhk"]
    dead = [a for a in AXES if st[a] in "csu"]
    small = [a for a in AXES if st[a] in "su"]
    there = [a for a in AXES if st[a] != "0"]
    kept = [a for a in there if st[a] != "u"]
    vdead = [a for a in dead if st[a] in "cs"]
    assert len(dead) <= 1 and (dead == [] or len(fl) >= 2 and len(dl) >= 2), zero
    assert fl or not there, zero
    assert p2 == "n" if sq else p2 in ("nb" if kept else "b"), (zero, state, p2)
    s01_rounds = "x" in sq and "y" in sq
    s_rounds = "z" in sq and ("x" in sq or "y" in sq)
    sxy = [a for a in "xy" if a in sq]
    names = [f"the {f[1]} {NAMES[f[0]]}" for f in FACTORS if f in zero]
    p2_says = SPREAD[p2] if kept else "spread and P2 zero, so P is P1"
    src = [
        "# restates: crates/rcad-geom/src/torus.rs on_floats at 37bc60af",
        "# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 37bc60af",
        "# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 37bc60af",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 37bc60af",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 37bc60af",
        "# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.",
        f"# The case where {', '.join(names[:-1]) + ' and ' if len(names) > 1 else ''}{names[-1]} "
        f"{'are' if len(names) > 1 else 'is'} zero: " + "; ".join(f"{a} {SAYS[st[a]]}" for a in AXES)
        + f"; {HOLLOW[hol_letter]}; {p2_says}.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre, the axis direction and the two radii, each an f64.",
        "# A difference is zero when the centre's coordinate is the same double as",
        "# the point's (464.11's diffzero.g), and a zero direction component is the",
        "# double 0.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        " ".join(f"o{a} = rnd(o{a}_);" if a in fl else f"o{a} = p{a};" for a in AXES),
        " ".join(f"d{a} = rnd(d{a}_);" if a in dl else f"d{a} = 0;" for a in AXES),
        "rg = rnd(rg_); tb = rnd(tb_);",
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
    src += ["", "# squares and out_and_back, left to right as Rust adds them."]
    if fl:
        src.append(" ".join(f"g{a} = rnd(f{a} * f{a});" for a in fl))
    if len(fl) == 3:
        src += ["O01 = gx + gy; o01 = rnd(O01);", "O = o01 + gz; oab = rnd(O);"]
    elif len(fl) == 2:
        src += ["# One square is zero, and adding it is exact wherever it falls, so",
                "# out_and_back is the other two's sum rounded once.",
                f"O = g{fl[0]} + g{fl[1]}; oab = rnd(O);"]
    elif fl:
        src += ["# Two squares are zero, so out_and_back is the third exactly.", f"oab = g{fl[0]};"]
    else:
        src += ["# The point is the centre: every difference is zero, and so is",
                "# out_and_back."]
    src += ["", "RR = rnd(rg * rg); rr = rnd(tb * tb);"]
    if hol:
        src.append("H = RR - rr; hollow = rnd(H);")
    else:
        src += ["# hollow is below the normals, where it is exact (464.16's sumexact.g).",
                "H = RR - rr; hollow = H;"]
    if fl:
        src.append("Bu = oab + hollow; bulge = rnd(Bu);")
    else:
        src += ["# out_and_back is zero, so bulge is hollow exactly.", "bulge = hollow;"]
    src += ["", "# axis_squared."]
    src.append(" ".join(f"t{a} = rnd(d{a} * d{a});" for a in dl))
    if len(dl) == 3:
        src += ["T01 = tx + ty; t01 = rnd(T01);", "T = t01 + tz; W = rnd(T);"]
    elif len(dl) == 2:
        src += ["# One square is zero, so axis_squared is the other two's sum rounded once.",
                f"T = t{dl[0]} + t{dl[1]}; W = rnd(T);"]
    else:
        src += ["# Two squares are zero, so axis_squared is the third exactly.", f"W = t{dl[0]};"]
    src += ["", "# across."]
    if dead or len(sq) < 3:
        src += ["# A zero square added to a double leaves that double, which then",
                "# rounds to itself, so a sum is written without its zero squares."]
    if sq:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in sq))
    if s01_rounds:
        src.append("S01 = sx + sy; s01 = rnd(S01);")
    elif sxy:
        src.append(f"s01 = s{sxy[0]};")
    if s_rounds:
        src.append("S = s01 + sz; across = rnd(S);")
    elif sxy:
        src.append("across = s01;")
    elif sq:
        src.append("across = sz;")
    else:
        src.append("# No component has a square, so across and right are zero, and y is left.")
    src += [
        "",
        "F4 = rnd(4 * RR);",
        "bb = rnd(bulge * bulge);",
        "left = rnd(W * bb);",
    ]
    if sq:
        src += ["right = rnd(F4 * across);", "y = left - right;"]
    else:
        src.append("y = left;")
    src += [
        "value = rnd(y);",
        "",
        "# The permanent. spread is a sum() from zero, and adding the first square to",
        "# zero is exact, as is adding the reach square of a component with both",
        "# halves zero.",
    ]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (464.15's dbl.g), so it is written unrounded."]
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
    if "u" in st.values():
        src += ["# A reach square that rounds to zero is written as the 0 it rounds to",
                "# (464.14's tinysq.g), and the sums leave it out."]
    vs = [f"v{a} = rnd(Xv{a});" if st[a] == "s" else f"v{a} = rnd(n{a} * n{a});" for a in kept]
    if vs:
        src.append(" ".join(vs))
    if kept:
        src.append(f"spread = {left_sum([f'v{a}' for a in kept])};")
    src.append("br = rnd(rnd(oab + RR) + rr);" if fl else "br = rnd(RR + rr);")
    src += ["brr = rnd(br * br);", "P1 = rnd(W * brr);"]
    if not kept:
        src += ["# spread and P2 are zero, and adding zero to P1 leaves it.", "P = P1;"]
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
        "# The permanent with no rounding of its own, but with v in place of the",
        "# reach square of a component that cancels and without one that rounds to",
        "# zero, or with P2 in place of the 4 R^2 product where P2 is below the",
        "# normals; Dx is what that adds to the two blocks below.",
        "Rc = oab + RR + rr;" if fl else "Rc = RR + rr;",
    ]

    def reach(a):
        if st[a] in "hk":
            return f"|{st[a]}{a}| * |{st[a]}{a}|"
        return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"

    dx_terms = None
    if p2 == "b" and kept:
        src.append("Phatx = W * (Rc * Rc) + P2;")
        dx_terms = "P2"
    elif kept:
        terms = [f"v{a}" if a in dead else reach(a) for a in kept]
        src.append(f"Phatx = W * (Rc * Rc) + F4 * ({' + '.join(terms)});")
        if vdead:
            dx_terms = f"F4 * ({' + '.join(f'v{a}' for a in vdead)})"
    else:
        src.append("Phatx = W * (Rc * Rc);")
    if dx_terms:
        src.append(f"Dx = {dx_terms};")
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
    if fl:
        src.append(" ".join(f"rg{a} = g{a} / (f{a} * f{a});" for a in fl))
    if len(fl) == 3:
        src.append("ro01 = o01 / O01; ro = oab / O;")
    elif len(fl) == 2:
        src.append("ro = oab / O;")
    src.append("rRR = RR / (rg * rg); rrr = rr / (tb * tb);")
    rb = (["rH = hollow / H;"] if hol else []) + (["rBu = bulge / Bu;"] if fl else [])
    src.append(" ".join(rb))
    src.append(" ".join(f"rt{a} = t{a} / (d{a} * d{a});" for a in dl))
    if len(dl) == 3:
        src.append("rt01 = t01 / T01; rW = W / T;")
    elif len(dl) == 2:
        src.append("rW = W / T;")
    if sq:
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in sq))
    rsum = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rS = across / S;"] if s_rounds else [])
    if rsum:
        src.append(" ".join(rsum))
    src.append("rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);")
    src.append("rL = left / (W * bb); rR = right / (F4 * across);" if sq else "rL = left / (W * bb);")
    src += ["", "# The w.w bulge^2 block. W and oab as their pieces, one for each square",
            "# that is not zero, which sum to them; with one, its ratio is direct."]
    if len(dl) == 3:
        src += ["Wx = rW * rt01 * tx; Wy = rW * rt01 * ty; Wz = rW * tz;",
                "W3 = Wx + Wy + Wz;",
                "jx = 1 / (rW * rt01 * rtx) - 1; jy = 1 / (rW * rt01 * rty) - 1; jz = 1 / (rW * rtz) - 1;"]
    elif len(dl) == 2:
        src += [" ".join(f"W{a} = rW * t{a};" for a in dl),
                f"W2 = W{dl[0]} + W{dl[1]};",
                " ".join(f"j{a} = 1 / (rW * rt{a}) - 1;" for a in dl)]
    src.append("om = Wt / W;")
    if len(fl) == 3:
        src += ["Ox = ro * ro01 * gx; Oy = ro * ro01 * gy; Oz = ro * gz;",
                "O3 = Ox + Oy + Oz;",
                "zx = 1 / (ro * ro01 * rgx * rfx * rfx) - 1;",
                "zy = 1 / (ro * ro01 * rgy * rfy * rfy) - 1;",
                "zz = 1 / (ro * rgz * rfz * rfz) - 1;"]
    elif len(fl) == 2:
        src += [" ".join(f"O{a} = ro * g{a};" for a in fl),
                f"O2 = O{fl[0]} + O{fl[1]};"]
        src += [f"z{a} = 1 / (ro * rg{a} * rf{a} * rf{a}) - 1;" for a in fl]
    if fl:
        src += [
            "Z = G2 / oab;",
            "",
            "# bulge as y carries it is rBu (oab + rH (RR - rr)); the true one is",
            "# Z oab + RR / rRR - rr / rrr. Squared and times W, that is six pieces, one",
            "# for each pair of oab, RR and rr; W is each piece's error over the piece.",
        ]
        six = [
            "KL = rL * rbb * rBu * rBu;",
            "WOO = KL - om * Z * Z;",
            "WRR = KL * rH * rH - om / (rRR * rRR);",
            "Wrr = KL * rH * rH - om / (rrr * rrr);",
            "WOR = KL * rH - om * Z / rRR;",
            "WOr = KL * rH - om * Z / rrr;",
            "WRr = KL * rH * rH - om / (rRR * rrr);",
        ]
        cs = [
            "cOO = W * oab * oab; cRR = W * RR * RR; crr = W * rr * rr;",
            "cOR = 2 * W * oab * RR; cOr = 2 * W * oab * rr; cRr = 2 * W * RR * rr;",
            "eOO = cOO * WOO; eRR = cRR * WRR; err = crr * Wrr;",
            "eOR = cOR * WOR; eOr = -(cOr * WOr); eRr = -(cRr * WRr);",
            "AW = cOO + cRR + crr + cOR + cOr + cRr;",
        ]
    else:
        src += [
            "",
            "# bulge as y carries it is hollow, rH (RR - rr); the true one is",
            "# RR / rRR - rr / rrr. Squared and times W, that is three pieces, one for",
            "# each pair of RR and rr; W is each piece's error over the piece.",
        ]
        six = [
            "KL = rL * rbb;",
            "WRR = KL * rH * rH - om / (rRR * rRR);",
            "Wrr = KL * rH * rH - om / (rrr * rrr);",
            "WRr = KL * rH * rH - om / (rRR * rrr);",
        ]
        cs = [
            "cRR = W * RR * RR; crr = W * rr * rr; cRr = 2 * W * RR * rr;",
            "eRR = cRR * WRR; err = crr * Wrr; eRr = -(cRr * WRr);",
            "AW = cRR + crr + cRr;",
        ]
    if not hol:
        src.append("# hollow is exact here, so rH is 1 and is left out.")
        six = [line.replace(" * rH * rH", "").replace(" * rH", "") for line in six]
    src += six + cs
    src += [
        "",
        "# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's",
        "# rounding, m undoes F4's and RR's.",
    ]
    if dead or len(sq) < 3:
        src.append("# A component with no square in y has no piece here.")
    for a in sq:
        carry = ["rR"] + (["rS"] if s_rounds else []) + (["rs01"] if a in "xy" and s01_rounds else [])
        carry += [f"rs{a}"] + ([f"rq{a}", f"rq{a}"] if a in nq else [])
        src.append(f"K{a} = " + " * ".join(carry) + ";")
    src.append("m = 1 / (rF4 * rRR);")
    for a in AXES:
        ab = ([f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a});"] if a in hlive else []) + \
             ([f"b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});"] if a in klive else [])
        if ab:
            src.append(" ".join(ab))
    for a in sq:
        if st[a] == "n":
            src.append(f"WH{a} = K{a} - m * a{a} * a{a}; WK{a} = K{a} - m * b{a} * b{a}; WX{a} = K{a} - m * a{a} * b{a};")
        elif st[a] == "h":
            src.append(f"WH{a} = K{a} - m * a{a} * a{a};")
        else:
            src.append(f"WK{a} = K{a} - m * b{a} * b{a};")
    for a in sq:
        if st[a] == "n":
            src.append(f"cH{a} = F4 * |h{a}| * |h{a}|; cK{a} = F4 * |k{a}| * |k{a}|; cX{a} = 2 * F4 * |h{a}| * |k{a}|;")
        else:
            p = st[a].upper()
            src.append(f"c{p}{a} = F4 * |{st[a]}{a}| * |{st[a]}{a}|;")
    for a in sq:
        if st[a] == "n":
            src.append(f"eH{a} = -(F4 * h{a} * h{a} * WH{a}); eK{a} = -(F4 * k{a} * k{a} * WK{a}); "
                       f"eX{a} = 2 * F4 * h{a} * k{a} * WX{a};")
        else:
            p = st[a].upper()
            src.append(f"e{p}{a} = -(F4 * {st[a]}{a} * {st[a]}{a} * W{p}{a});")
    if sq:
        src.append(" ".join(f"C3{a} = cH{a} + cK{a} + cX{a};" if st[a] == "n"
                            else f"C3{a} = c{st[a].upper()}{a};" for a in sq))
        src.append(f"CR = {' + '.join(f'C3{a}' for a in sq)};")
        src.append(" ".join(f"Y{a} = F4 * K{a} * Q{a} * Q{a};" if st[a] == "n"
                            else f"Y{a} = F4 * K{a} * q{a} * q{a};" for a in sq))
        if nq:
            src.append(" ".join(f"Z{a} = F4 * m * (C{a} * C{a});" for a in nq))
    ec = None
    if dead:
        src += ["# What y misses of a component that cancels is its whole true term."]
        ec = f"eC{dead[0]}"
        src.append(f"{ec} = F4 * m * (C{dead[0]} * C{dead[0]});")
    src += [
        "",
        "# The same permanent as the two blocks.",
        "Phat2 = AW + CR;" if sq else "Phat2 = AW;",
    ]

    def lines(rows):
        return "\n  /\\ ".join(rows)

    finite = ["|px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024"]
    finite += [" /\\ ".join(f"|o{a}| <= 1b1024" for a in fl)] if fl else []
    finite += [" /\\ ".join(f"|d{a}| <= 1b1024" for a in dl), "|rg| <= 1b1024 /\\ |tb| <= 1b1024"]
    nonzero = [" /\\ ".join(f"|f{a}| >= 1b-1074" for a in fl)] if fl else []
    nonzero += [" /\\ ".join(f"|d{a}| >= 1b-1074" for a in dl), "|rg| >= 1b-1074 /\\ |tb| >= 1b-1074"]
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
    ]
    if hol:
        src += ["  # hollow is a normal number; below the normals it is exact, and that is",
                "  # another case.",
                "  /\\ |hollow| >= 1b-1022"]
    else:
        src += ["  # hollow is below the normals, where it is exact.",
                "  /\\ |hollow| <= 1b-1022"]
    src += [
        "  # product_is_normal on the squares of from_centre, of direction and of",
        "  # sideways, and on the halves, that are not zero; on R^2, r^2, bulge^2 and",
        "  # axis_squared bulge^2; and on 4 R^2 across where across is not zero",
        "  # (stays_accurate and the check after it).",
    ]
    sqs = [f"|g{a}| >= 1b-1022" for a in fl] + [f"|t{a}| >= 1b-1022" for a in dl] + \
          [f"|s{a}| >= 1b-1022" for a in sq]
    src.append("  /\\ " + lines([" /\\ ".join(sqs[i:i + 3]) for i in range(0, len(sqs), 3)]))
    if halves:
        src.append("  /\\ " + lines([" /\\ ".join(halves[i:i + 3]) for i in range(0, len(halves), 3)]))
    src.append("  /\\ |RR| >= 1b-1022 /\\ |rr| >= 1b-1022")
    src.append("  /\\ |bb| >= 1b-1022 /\\ |left| >= 1b-1022" + (" /\\ |right| >= 1b-1022" if sq else ""))
    src += [
        "  # The permanent's products, by monotone rounding: v >= s for a component",
        "  # that does not cancel, and with one half zero v is s; for one that",
        "  # cancels the case says which v is, and 464.14's tinysq.g and tinyzero.g",
        "  # split them at 2^-1075. br >= bulge, as hollow <= RR, so brr >= bb and",
        "  # P1 >= left. "
        + ("spread >= across, so P2 >= right." if sq else "The case says what P2 is."),
    ]
    rows = []
    for a in there:
        if st[a] == "s":
            rows.append(f"Xv{a} <= 1b-1022 /\\ v{a} >= 1b-1074")
        elif st[a] == "u":
            rows.append(f"Xv{a} <= 1b-1075")
        else:
            rows.append(f"|v{a}| >= 1b-1022")
    if rows:
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
    for a in fl:
        src.append(f"  /\\ g{a} -/ f{a} * f{a} {LEMMA}")
    for a in dl:
        src.append(f"  /\\ t{a} -/ d{a} * d{a} {LEMMA}")
    for a in sq:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for a in AXES:
        left, right = HALVES[a]
        if a in hlive:
            src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        if a in klive:
            src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for p in ("RR -/ rg * rg", "rr -/ tb * tb", "bb -/ bulge * bulge", "left -/ W * bb"):
        src.append(f"  /\\ {p} {LEMMA}")
    if sq:
        src.append(f"  /\\ right -/ F4 * across {LEMMA}")
    for a in there:
        if st[a] not in "su":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    src.append(f"  /\\ brr -/ br * br {LEMMA}")
    src.append(f"  /\\ P1 -/ W * brr {LEMMA}")
    if p2 == "n":
        src.append(f"  /\\ P2 -/ F4 * spread {LEMMA}")
    src.append(f"  /\\ eb -/ B * P {LEMMA}")
    signs = [f"h{a} / |h{a}| in [-1, 1]" for a in hlive if a not in dead] + \
            [f"k{a} / |k{a}| in [-1, 1]" for a in klive if a not in dead]
    if signs:
        src.append("  # x / |x| is in [-1, 1] for any x that is not zero.")
        src.append("  /\\ " + lines([" /\\ ".join(signs[i:i + 2]) for i in range(0, len(signs), 2)]))
    src += [
        "->",
        "  (y - Ex) / Phat2 in ? /\\ P -/ Phatx in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
        "# How far each rounding is from the identity.",
    ]
    src += [f"f{a} / F{a} -> 1 + (f{a} - F{a}) / F{a};" for a in fl]
    for a in AXES:
        for h, prod, live in zip("hk", HALVES[a], (hlive, klive)):
            if a in live:
                src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in nq:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    src += [f"g{a} / (f{a} * f{a}) -> 1 + (g{a} - f{a} * f{a}) / (f{a} * f{a});" for a in fl]
    if len(fl) == 3:
        src.append("o01 / O01 -> 1 + (o01 - O01) / O01;")
    if len(fl) >= 2:
        src.append("oab / O -> 1 + (oab - O) / O;")
    src += [
        "RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);",
        "rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);",
    ]
    if hol:
        src.append("hollow / H -> 1 + (hollow - H) / H;")
    if fl:
        src.append("bulge / Bu -> 1 + (bulge - Bu) / Bu;")
    src += [f"t{a} / (d{a} * d{a}) -> 1 + (t{a} - d{a} * d{a}) / (d{a} * d{a});" for a in dl]
    if len(dl) == 3:
        src.append("t01 / T01 -> 1 + (t01 - T01) / T01;")
    if len(dl) >= 2:
        src.append("W / T -> 1 + (W - T) / T;")
    for a in sq:
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
    if sq:
        src.append("right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);")
    src += ["", "# w.w over W, and |x|^2 over oab, each averaged over its pieces."]
    if len(dl) == 1:
        src.append(f"Wt / W -> 1 / rt{dl[0]};")
    else:
        w = f"W{len(dl)}"
        src += [f"Wt / W -> 1 + (({w} + (Wt - W)) - {w}) / {w};",
                f"{w} + (Wt - W) -> " + " + ".join(f"(W{a} + W{a} * j{a})" for a in dl) + ";"]
        src += [f"(W{a} + W{a} * j{a}) - W{a} -> W{a} * j{a};" for a in dl]
    if len(fl) == 1:
        src.append(f"G2 / oab -> 1 / (rg{fl[0]} * rf{fl[0]} * rf{fl[0]});")
    elif fl:
        o = f"O{len(fl)}"
        src += [f"G2 / oab -> 1 + (({o} + (G2 - oab)) - {o}) / {o};",
                f"{o} + (G2 - oab) -> " + " + ".join(f"(O{a} + O{a} * z{a})" for a in fl) + ";"]
        src += [f"(O{a} + O{a} * z{a}) - O{a} -> O{a} * z{a};" for a in fl]
    src += [
        "",
        "# The error is the sum of the two blocks' errors, each the sum of its",
        "# pieces' errors, each a fraction of its own piece of the permanent; add_rr",
        "# then averages them.",
    ]
    if dead:
        src += ["# What y misses of a component that cancels goes with the piece cOR, and",
                "# is at most a u^2 of it."]
    src.append("(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;")
    if fl:
        oor = f"(cOR + (eOR + {ec}))" if dead else "(cOR + eOR)"
        aw_pieces = f"(cOO + eOO) + (cRR + eRR) + (crr + err) + {oor} + (cOr + eOr) + (cRr + eRr)"
    else:
        aw_pieces = "(cRR + eRR) + (crr + err) + (cRr + eRr)"
    aw = f"(AW + ((left - Lt) + {ec}))" if dead else "(AW + (left - Lt))"
    rest = (f"(CR + (Rt - right - {ec}))" if dead else "(CR + (Rt - right))") if sq else None
    src.append("Phat2 + (y - Ex) -> " + (f"{aw} + {rest};" if rest else f"{aw};"))
    src.append(f"{aw[1:-1]} -> {aw_pieces};")
    if fl:
        src += ["(cOO + eOO) - cOO -> eOO;"]
    src += ["(cRR + eRR) - cRR -> eRR;", "(crr + err) - crr -> err;"]
    if fl:
        src.append(f"(cOR + (eOR + {ec})) - cOR -> eOR + {ec};" if dead else "(cOR + eOR) - cOR -> eOR;")
        src.append("(cOr + eOr) - cOr -> eOr;")
    src.append("(cRr + eRr) - cRr -> eRr;")
    if fl:
        src.append("eOr / cOr -> -WOr;")
    src.append("eRr / cRr -> -WRr;")
    if dead:
        a = dead[0]
        f_, d_ = HALVES[a][0][1], HALVES[a][0][-1]
        src += [
            f"(eOR + {ec}) / cOR -> WOR + {ec} / cOR;",
            f"{ec} / cOR -> 2 * rF4 * m * (a{a} - b{a}) * (a{a} - b{a}) * rh{a} * rh{a}"
            f" * (f{f_} * f{f_} / oab) * (d{d_} * d{d_} / W);",
            f"f{f_} * f{f_} / oab -> (g{f_} / oab) / rg{f_};",
            over_sum("g", f_, fl, "ro01", "ro", "oab"),
            f"d{d_} * d{d_} / W -> (t{d_} / W) / rt{d_};",
            over_sum("t", d_, dl, "rt01", "rW", "W"),
        ]
    if sq:
        blocks = [f"(C3{a} + (Z{a} - Y{a}))" if st[a] == "n" else f"(C3{a} + e{st[a].upper()}{a})" for a in sq]
        src.append(f"{rest[1:-1]} -> " + " + ".join(blocks) + ";")
        for a in nq:
            src.append(f"C3{a} + (Z{a} - Y{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
        for a in sq:
            if st[a] == "n":
                for p in "HKX":
                    src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
            else:
                p = st[a].upper()
                src.append(f"(C3{a} + e{p}{a}) - C3{a} -> e{p}{a};")
        for a in sq:
            if st[a] == "n":
                src += [f"eH{a} / cH{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a});",
                        f"eK{a} / cK{a} -> -((k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a});",
                        f"eX{a} / cX{a} -> (h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a};"]
            else:
                p, x = st[a].upper(), f"{st[a]}{a}"
                src.append(f"e{p}{a} / C3{a} -> -(({x} / |{x}|) * ({x} / |{x}|) * W{p}{a});")
    src += ["", "# From the two blocks to the permanent on_floats rounds, and on to eb."]
    if dx_terms:
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
    for mask in itertools.product((False, True), repeat=6):
        zero = {f for f, z in zip(FACTORS, mask) if z}
        if not zero or {"dx", "dy", "dz"} <= zero:
            continue
        tag = "".join(f for f in FACTORS if f in zero)
        kind = kinds(zero)
        centre = {"fx", "fy", "fz"} <= zero
        for state in "ncsu" if "b" in kind.values() else [""]:
            st = {a: state if kind[a] == "b" else kind[a] for a in AXES}
            has_square = any(s in "nhk" for s in st.values())
            has_reach = any(s in "nhkcs" for s in st.values())
            for hollow in "n" if centre else "ne":
                for p2 in "n" if has_square else "nb" if has_reach else "b":
                    yield zero, tag, state, hollow, p2


count = 0
for zero, tag, state, hollow, p2 in cases():
    name = f"torz_{tag}_{state}{hollow}{p2}"
    write(f"{name}.g", make(zero, state, hollow, p2))
    write(f"c_{name}_half.g", make(zero, state, hollow, p2, B_HALF))
    count += 1
assert count == 169, count
print("cases written:", count)
