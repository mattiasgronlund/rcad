"""Write the scripts of 464.16 from cone.g beside it: the controls
c_cone_half.g at B = u / 2, c_cone_nonormal.g without product_is_normal, and
c_cone_noeb.g without eb normal. Each change is a text replacement that must
match exactly once. Run as: python3 conecases.py. The section's bisection
scripts b_cone_<m>.g and b11_cone_<m>.g are not written here."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
CONE = open("cone.g").read()
B_CODE = "B = 12b-53 + 3b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
TITLE = "# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.\n"


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def at(text, b):
    return sub(text, B_CODE, b)


def without_normal(text):
    keep = []
    lemma_of = ("hx", "kx", "hy", "ky", "hz", "kz", "sx", "sy", "sz", "ux", "uy", "uz",
                "u01", "ss", "w", "along", "vx", "vy", "vz", "R2", "A")
    for line in text.split("\n"):
        if "1b-1022" in line and "|eb|" not in line:
            continue
        if line.startswith("  /\\ ") and line.split()[1] in lemma_of and LEMMA in line:
            continue
        keep.append(line)
    text = "\n".join(keep)
    text = sub(text, "  # The first sum of up is a normal number; where it is not, it is exact,\n"
               "  # and that is another case.\n", "")
    text = sub(text, "  # product_is_normal on the six halves (cross_product_halves_if_normal),\n"
               "  # and on the three components, the three up halves, the slope, up and the\n"
               "  # axial term (stays_accurate).\n", "")
    text = sub(text, "  # The permanent's three squares are normal too, since no component is\n"
               "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,\n"
               "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal. In the same way\n"
               "  # |up| <= R, so R2 >= w and A >= along, both normal.\n", "")
    return sub(text, TITLE, TITLE + "# A control: without product_is_normal and the lemma lines it gives.\n")


def without_eb(text):
    text = sub(text, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
    text = sub(text, f"  /\\ eb -/ B * P {LEMMA}\n", "")
    return sub(text, TITLE, TITLE + "# A control: without eb normal and the lemma line it gives.\n")


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


write("c_cone_half.g", at(CONE, B_HALF))
write("c_cone_nonormal.g", without_normal(CONE))
write("c_cone_noeb.g", without_eb(CONE))
