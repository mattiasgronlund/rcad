"""Write block 1 with two of first, second and third zero, as zeroJK.g."""
import sys

PAIR = {
    1: ("cdxbdy = rnd(cdx * bdy);", "cdxbdy = bdxcdy;"),
    2: ("adxcdy = rnd(adx * cdy);", "adxcdy = cdxady;"),
    3: ("bdxady = rnd(bdx * ady);", "bdxady = adxbdy;"),
}
MINOR = {1: "first", 2: "second", 3: "third"}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def make(zero):
    (normal,) = {1, 2, 3} - zero
    src = open("block1.g").read()
    names = " and ".join(MINOR[k] for k in sorted(zero))
    src = sub(
        src,
        "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
        "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
        f"# Block 1 where {names} are both zero. The permanent's products\n"
        f"# for those two are checked by no product_is_normal.\n",
    )
    for k in zero:
        src = sub(src, *PAIR[k])
        src = sub(src, f"{MINOR[k]} = rnd(M{k});", f"{MINOR[k]} = M{k};")
        src = sub(src, f"t{k} = rnd(U{k});", f"t{k} = U{k};")
        src = sub(src, f"rm{k} = {MINOR[k]} / M{k};", f"rm{k} = 1;")
        src = sub(src, f"rt{k} = t{k} / U{k};", f"rt{k} = 1;")
        src = sub(src, f"{MINOR[k]} / M{k} -> 1 + ({MINOR[k]} - M{k}) / M{k};\n", "")
        src = sub(src, f"t{k} / U{k} -> 1 + (t{k} - U{k}) / U{k};\n", "")
        bound = "in [-4503599627370497b-105, 4503599627370497b-105]\n"
        src = sub(src, f"  /\\ t{k} -/ U{k} {bound}", "")
        src = sub(src, f"  /\\ r{k} -/ R{k} {bound}", "")
    # One of t1 and t2 is zero, so their sum is the other and rounds to
    # itself.
    src = sub(src, "s12 = rnd(S);", "s12 = S;")
    src = sub(src, "rs = s12 / S;\n", "")
    src = src.replace(" * rs - 1", " - 1")
    src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
    src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
    m = MINOR[normal]
    src = sub(
        src,
        "  # No factor of the nine products is zero. For an f64 that is the same as\n"
        "  # being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {names}. For an f64\n"
        "  # that is the same as being at least the smallest subnormal.\n",
    )
    src = sub(
        src,
        "  # The permanent's own three products are normal too. The comment above\n"
        "  # stays_accurate argues it from the nine.\n",
        f"  # r{normal} is normal because |{m}| <= q{normal}. Nothing says the other\n"
        "  # two are.\n",
    )
    src = sub(
        src,
        "  /\\ |first| >= 1b-1074 /\\ |second| >= 1b-1074 /\\ |third| >= 1b-1074\n",
        f"  /\\ |{m}| >= 1b-1074\n",
    )
    src = sub(
        src,
        "  /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022 /\\ |t3| >= 1b-1022\n",
        f"  /\\ |t{normal}| >= 1b-1022\n",
    )
    src = sub(
        src,
        "  /\\ |r1| >= 1b-1022 /\\ |r2| >= 1b-1022 /\\ |r3| >= 1b-1022\n",
        f"  /\\ |r{normal}| >= 1b-1022\n",
    )
    src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P12 = rnd(r1 + r2);\nP = rnd(P12 + r3);")
    src = sub(
        src,
        "Phat = (|bdxcdy| + |cdxbdy|) * |adz|\n"
        "     + (|cdxady| + |adxcdy|) * |bdz|\n"
        "     + (|adxbdy| + |bdxady|) * |cdz|;\n",
        "Phat1 = (|bdxcdy| + |cdxbdy|) * |adz|;\n"
        "Phat2 = (|cdxady| + |adxcdy|) * |bdz|;\n"
        "Phat3 = (|adxbdy| + |bdxady|) * |cdz|;\n"
        "Phat12 = Phat1 + Phat2;\n"
        "Phat = Phat12 + Phat3;\n",
    )
    pieces = ["(P12 - (r1 + r2)) / Phat"]
    for k in (1, 2, 3):
        if k in zero:
            pieces.append(f"(r{k} - R{k}) / Phat")
            pieces.append(f"((q{k} - ({NOTE[k]})) / ({NOTE[k]})) * (Phat{k} / Phat)")
        else:
            pieces.append(f"((r{k} - Phat{k}) / Phat{k}) * (Phat{k} / Phat)")
    others = {1: "Phat2 + Phat3", 2: "Phat1 + Phat3", 3: "Phat12"}
    src += (
        "\n"
        "# Two of r1, r2 and r3 may be below the normals, so their errors are\n"
        "# amounts and not fractions. Carry each as an amount against Phat,\n"
        f"# which r{normal} being normal keeps away from zero.\n"
        "(P - Phat) / Phat -> ((P - (P12 + r3)) / (P12 + r3)) * (1 + ((P12 + r3) - Phat) / Phat)"
        " + ((P12 + r3) - Phat) / Phat;\n"
        f"((P12 + r3) - Phat) / Phat -> {' + '.join(pieces)};\n"
        + "".join(
            f"Phat{k} / Phat -> 1 / (1 + ({others[k]}) / Phat{k});\n" for k in (1, 2, 3)
        )
        + f"Phat{normal} -> r{normal} / (1 + (r{normal} - Phat{normal}) / Phat{normal});\n"
        "P12 + r3 -> P / (1 + (P - (P12 + r3)) / (P12 + r3));\n"
    )
    return src


for arg in sys.argv[1:]:
    zero = {int(c) for c in arg}
    open(f"zero{arg}d.g", "w").write(make(zero))
