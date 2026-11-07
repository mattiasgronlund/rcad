"""Write the controls of 464.15 from cyl.g beside it: c_cyl_value.g, the claim
on value, which does not prove at the code's B (the section's cylv.g);
c_cyl_half.g at B = u / 2; and c_cyl_nonormal.g without product_is_normal.
Each change is a text replacement that must match exactly once. Run as:
python3 cylcases.py. The section's c_cyl_noeb.g, which exited 124 at 600 s,
c_cylv_half.g and the bisection scripts are not written here: a control must
exit 1 (464.10 5).
"""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
CYL = open("cyl.g").read()
B_CODE = "B = 10b-53 + 5b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def on_value(text):
    text = sub(text, "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n",
               "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n"
               "# The claim is on value, the double on_floats returns, not on y.\n")
    text = sub(text, "rrr = rr / (r * r); rsk = skin / (rr * D);\n",
               "rrr = rr / (r * r); rsk = skin / (rr * D);\nrv = value / y;\n")
    text = sub(text, "# Each component's square, as y carries it,", "# Each component's square, as value carries it,")
    text = sub(text, "Kx = rqx * rqx * rsx * rs01 * rs012;", "Kx = rqx * rqx * rsx * rs01 * rs012 * rv;")
    text = sub(text, "Ky = rqy * rqy * rsy * rs01 * rs012;", "Ky = rqy * rqy * rsy * rs01 * rs012 * rv;")
    text = sub(text, "Kz = rqz * rqz * rsz * rs012;", "Kz = rqz * rqz * rsz * rs012 * rv;")
    text = sub(text, "WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;", "WGx = 1 / (rrr * rtx * rsk * rD * rt01) - rv;")
    text = sub(text, "WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;", "WGy = 1 / (rrr * rty * rsk * rD * rt01) - rv;")
    text = sub(text, "WGz = 1 / (rrr * rtz * rsk * rD) - 1;", "WGz = 1 / (rrr * rtz * rsk * rD) - rv;")
    text = sub(text, "# The radius term is three pieces,", "# The radius term, as value carries it times 1 / rv, is three pieces,")
    text = sub(text, "# Each component's square as y carries it,", "# Each component's square as value carries it,")
    text = sub(text, "  # The bound is normal, which decide checks.\n",
               "  # y is not zero. Where it is, value is zero too, and the claim on y is\n"
               "  # the claim on value.\n"
               "  /\\ |y| >= 1b-1074\n"
               "  # The bound is normal, which decide checks.\n")
    text = sub(text, "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);\n",
               "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);\nvalue / y -> 1 + (value - y) / y;\n")
    text = sub(text, "SG + (Gt - skin) ->", "SG + (Gt - rv * skin) ->")
    text = sub(text, "(SG + (Gt - skin));", "(SG + (Gt - rv * skin));")
    return text.replace("(y - Ex)", "(value - Ex)")


def at(text, b):
    return sub(text, B_CODE, b)


def without_normal(text):
    keep = []
    lemma_of = ("hx", "kx", "hy", "ky", "hz", "kz", "sx", "sy", "sz", "tx", "ty", "tz",
                "rr", "skin", "vx", "vy", "vz")
    for line in text.split("\n"):
        if "1b-1022" in line and "|eb|" not in line:
            continue
        if line.startswith("  /\\ ") and line.split()[1] in lemma_of and LEMMA in line:
            continue
        keep.append(line)
    text = "\n".join(keep)
    text = sub(text, "  # product_is_normal on the six halves (cross_product_halves_if_normal),\n"
               "  # and on the three components, the three direction components, the\n"
               "  # radius and the radius term (stays_accurate).\n", "")
    text = sub(text, "  # The permanent's three squares are normal too, since no component is\n"
               "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,\n"
               "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal.\n", "")
    return sub(text, ", rcad-geom's cylinder.\n", ", rcad-geom's cylinder.\n# A control: without product_is_normal and the lemma lines it gives.\n")


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


CYLV = on_value(CYL)
write("c_cyl_value.g", CYLV)
write("c_cyl_half.g", at(CYL, B_HALF))
write("c_cyl_nonormal.g", without_normal(CYL))
