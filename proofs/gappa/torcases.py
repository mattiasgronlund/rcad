"""Write the scripts of 464.17 from tor.g beside it: the controls c_tor_half.g
at B = u / 2, c_tor_nonormal.g without product_is_normal, and c_tor_noeb.g
without eb normal; and c_tor_b17_231.g at B = 17u + 231u^2, the largest of
the section's b17_tor_<m> that fail (0 to 231), so every smaller m fails too.
The section's b_tor_<m> all prove and are not controls. Each change is a text
replacement that must match exactly once. Run as: python3 torcases.py."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
TOR = open("tor.g").read()
B_CODE = "B = 18b-53 + 7b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
TITLE = "# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.\n"


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def at(text, b):
    return sub(text, B_CODE, b)


def without_normal(text):
    keep = []
    for line in text.split("\n"):
        if "1b-1022" in line and "|eb|" not in line:
            continue
        if line.startswith("  /\\ ") and LEMMA in line and not line.startswith("  /\\ eb -/"):
            continue
        keep.append(line)
    text = "\n".join(keep)
    text = sub(text, "  # hollow is a normal number; below the normals it is exact, and that is\n"
               "  # another case.\n", "")
    text = sub(text, "  # product_is_normal on the three squares of from_centre, of direction and\n"
               "  # of sideways, the six halves, R^2, r^2, bulge^2, axis_squared bulge^2 and\n"
               "  # 4 R^2 across (stays_accurate and the check after it).\n", "")
    text = sub(text, "  # The permanent's products are normal too, since nothing cancels here, by\n"
               "  # monotone rounding: v >= s, as no component is zero; br >= bulge, as\n"
               "  # hollow <= RR, so brr >= bb and P1 >= left; spread >= across, so\n"
               "  # P2 >= right.\n", "")
    return sub(text, TITLE, TITLE + "# A control: without product_is_normal and the lemma lines it gives.\n")


def without_eb(text):
    text = sub(text, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
    text = sub(text, f"  /\\ eb -/ B * P {LEMMA}\n", "")
    return sub(text, TITLE, TITLE + "# A control: without eb normal and the lemma line it gives.\n")


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


write("c_tor_half.g", at(TOR, B_HALF))
write("c_tor_nonormal.g", without_normal(TOR))
write("c_tor_noeb.g", without_eb(TOR))
write("c_tor_b17_231.g", at(TOR, "B = 17b-53 + 231b-106;"))
