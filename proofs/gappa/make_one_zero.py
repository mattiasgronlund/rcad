"""Write block 1 with second zero, as secondzero.g, and with third zero, as
thirdzero.g: the changes §464.7 4 lists for each, made to block1.g. first
zero is zero2.g, which §464.7 4 gives whole.

Run as: python3 make_one_zero.py, beside block1.g.
"""

HEADER = (
    "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
    "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n"
)
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def second_zero(src):
    src = sub(
        src,
        HEADER,
        "# Block 1 where second is zero: cdx * ady and adx * cdy round to the same\n"
        "# double, so second = rnd(0) = 0 and t2 = rnd(bdz * 0) = 0, and r2 is not\n"
        "# checked by any product_is_normal.\n",
    )
    src = sub(src, "adxcdy = rnd(adx * cdy);", "adxcdy = cdxady;")
    src = sub(src, "second = rnd(M2);", "second = M2;")
    src = sub(src, "t2 = rnd(U2);", "t2 = U2;")
    src = sub(src, "s12 = rnd(S);", "s12 = S;")
    src = sub(src, "rm2 = second / M2;", "rm2 = 1;")
    src = sub(src, "rt2 = t2 / U2;", "rt2 = 1;")
    src = sub(src, "rs = s12 / S;\n", "")
    assert src.count(" * rs - 1") == 4
    src = src.replace(" * rs - 1", " - 1")
    src = sub(src, " /\\ |second| >= 1b-1074", "")
    src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
    src = sub(src, " /\\ |t2| >= 1b-1022", "")
    src = sub(src, " /\\ |r2| >= 1b-1022", "")
    src = sub(src, f"  /\\ t2 -/ U2 {BOUND}", "")
    src = sub(src, f"  /\\ r2 -/ R2 {BOUND}", "")
    src = sub(src, "second / M2 -> 1 + (second - M2) / M2;\n", "")
    src = sub(src, "t2 / U2 -> 1 + (t2 - U2) / U2;\n", "")
    src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
    return src


def third_zero(src):
    src = sub(
        src,
        HEADER,
        "# Block 1 where third is zero: adx * bdy and bdx * ady round to the same\n"
        "# double, so third = rnd(0) = 0 and t3 = rnd(cdz * 0) = 0, and r3 is not\n"
        "# checked by any product_is_normal.\n",
    )
    src = sub(src, "bdxady = rnd(bdx * ady);", "bdxady = adxbdy;")
    src = sub(src, "third = rnd(M3);", "third = M3;")
    src = sub(src, "t3 = rnd(U3);", "t3 = U3;")
    src = sub(src, "rm3 = third / M3;", "rm3 = 1;")
    src = sub(src, "rt3 = t3 / U3;", "rt3 = 1;")
    src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P12 = rnd(r1 + r2);\nP = rnd(P12 + r3);")
    src = sub(
        src,
        "Phat = (|bdxcdy| + |cdxbdy|) * |adz|\n"
        "     + (|cdxady| + |adxcdy|) * |bdz|\n"
        "     + (|adxbdy| + |bdxady|) * |cdz|;\n",
        "Phat12 = (|bdxcdy| + |cdxbdy|) * |adz|\n"
        "       + (|cdxady| + |adxcdy|) * |bdz|;\n"
        "Phat3 = (|adxbdy| + |bdxady|) * |cdz|;\n"
        "Phat = Phat12 + Phat3;\n",
    )
    src = sub(src, " /\\ |third| >= 1b-1074", "")
    src = sub(src, " /\\ |t3| >= 1b-1022", "")
    src = sub(src, " /\\ |r3| >= 1b-1022", "")
    src = sub(src, f"  /\\ t3 -/ U3 {BOUND}", "")
    src = sub(src, f"  /\\ r3 -/ R3 {BOUND}", "")
    src = sub(src, "third / M3 -> 1 + (third - M3) / M3;\n", "")
    src = sub(src, "t3 / U3 -> 1 + (t3 - U3) / U3;\n", "")
    src += (
        "\n"
        "# r3 may be below the normals, so its error is an amount and not a\n"
        "# fraction. Carry it as an amount against Phat, which eb being normal keeps\n"
        "# far above it.\n"
        "(P - Phat) / Phat -> ((P - (P12 + r3)) / (P12 + r3)) * (1 + ((P12 + r3) - Phat) / Phat)"
        " + ((P12 + r3) - Phat) / Phat;\n"
        "((P12 + r3) - Phat) / Phat -> ((P12 - Phat12) / Phat12) * (Phat12 / Phat)"
        " + (r3 - R3) / Phat + ((q3 - (|adxbdy| + |bdxady|)) / (|adxbdy| + |bdxady|)) * (Phat3 / Phat);\n"
        "Phat12 / Phat -> 1 / (1 + Phat3 / Phat12);\n"
        "Phat3 / Phat -> 1 / (1 + Phat12 / Phat3);\n"
        "Phat12 -> P12 / (1 + (P12 - Phat12) / Phat12);\n"
        "P12 + r3 -> P / (1 + (P - (P12 + r3)) / (P12 + r3));\n"
    )
    return src


block1 = open("block1.g").read()
open("secondzero.g", "w").write(second_zero(block1))
open("thirdzero.g", "w").write(third_zero(block1))
