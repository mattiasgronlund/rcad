"""The claim on value of 464.15 4, written from cyl.g beside it: cylv2.g is
cyl.g with value's own rounding taken as a fraction of y, and y as a fraction
of the permanent, at B = 10u + 82u^2, the smallest m that proves; its controls
are c_cylv2_half.g at u / 2 and c_cylv2_81.g at B = 10u + 81u^2, which fails.
The section's t_vtriv.g and t_v2triv.g (every hint kept, goal px in ?), its
other b_cylv2_<m>.g and cylv.g are not written here: the probes prove nothing
about a bound, the larger m prove what 82 does and cylv.g does not finish.
Each change is a text replacement that must match exactly once. Run as:
python3 vgen.py"""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


GOAL = "  (y - Ex) / P12 in ? /\\ P -/ Phat in ?\n  /\\ (y - Ex) / eb in [-1, 1]\n"
B_CODE = "B = 10b-53 + 5b-102;"
c = open("cyl.g").read()
c = sub(c, "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n"
        "# The claim is on value, the double on_floats returns, not on y.\n")
c = sub(c, "Gt = r * r * (dx * dx + dy * dy + dz * dz);\n",
        "Gt = r * r * (dx * dx + dy * dy + dz * dz);\n\n"
        "# y over the permanent: each component's square as y carries it is K\n"
        "# times (h - k)^2, which is K times the three pieces of its reach\n"
        "# squared, the third with a sign; the radius term is taken off whole.\n"
        "fHx = Kx * hx * hx; fKx = Kx * kx * kx; fXx = -(2 * Kx * hx * kx);\n"
        "fHy = Ky * hy * hy; fKy = Ky * ky * ky; fXy = -(2 * Ky * hy * ky);\n"
        "fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);\n")
c = sub(c, "  # The bound is normal, which decide checks.\n",
        "  # y is not zero. Where it is, value is zero too, and the claim on y is\n"
        "  # the claim on value.\n"
        "  /\\ |y| >= 1b-1074\n"
        "  # The bound is normal, which decide checks.\n")
c = sub(c, GOAL,
        "  (y - Ex) / P12 in ? /\\ y / P12 in ? /\\ (value - y) / y in ? /\\ P -/ Phat in ?\n"
        "  /\\ (value - Ex) / eb in [-1, 1]\n")
c = sub(c, "(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);\n",
        "(value - Ex) / eb -> (y - Ex) / eb + (value - y) / eb;\n"
        "(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);\n"
        "(value - y) / eb -> ((value - y) / y) * (y / P12) * (Phat / P) * (P / eb);\n")
c = sub(c, "fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);\n",
        "fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);\n"
        "# The same pieces doubled, so that each f is at most about half its\n"
        "# piece: Gappa averages a sum's relative errors only while each is\n"
        "# above -1, and f against the piece itself reaches -1.\n"
        "dHx = 2 * cHx; dKx = 2 * cKx; dXx = 2 * cXx;\n"
        "dHy = 2 * cHy; dKy = 2 * cKy; dXy = 2 * cXy;\n"
        "dHz = 2 * cHz; dKz = 2 * cKz; dXz = 2 * cXz;\n"
        "P3dx = dHx + dKx + dXx; P3dy = dHy + dKy + dXy; P3dz = dHz + dKz + dXz;\n"
        "SGd = 2 * SG;\n"
        "P12d = P3dx + P3dy + P3dz + SGd;\n")
c += ("\n# value's own rounding is a fraction of y, and y a fraction of the\n"
      "# permanent, averaged over the same four blocks, doubled.\n"
      "y / P12 -> 2 * (((P12d + y) - P12d) / P12d);\n"
      "P12d + y -> (P3dx + Yx) + (P3dy + Yy) + (P3dz + Yz) + (SGd - skin);\n")
for a in "xyz":
    c += f"P3d{a} + Y{a} -> (dH{a} + fH{a}) + (dK{a} + fK{a}) + (dX{a} + fX{a});\n"
for a in "xyz":
    for p in "HKX":
        c += f"(d{p}{a} + f{p}{a}) - d{p}{a} -> f{p}{a};\n"
for a in "xyz":
    c += (f"fH{a} / dH{a} -> K{a} * (h{a} / |h{a}|) * (h{a} / |h{a}|) / 2;\n"
          f"fK{a} / dK{a} -> K{a} * (k{a} / |k{a}|) * (k{a} / |k{a}|) / 2;\n"
          f"fX{a} / dX{a} -> -(K{a} * (h{a} / |h{a}|) * (k{a} / |k{a}|)) / 2;\n")
c += "((SGd - skin) - SGd) / SGd -> -(skin / SGd);\nskin / SGd -> 1 / 2;\n"
open("cylv2.g", "w").write(sub(c, B_CODE, "B = 10b-53 + 82b-106;"))
open("c_cylv2_half.g", "w").write(sub(c, B_CODE, "B = 1b-54;"))
open("c_cylv2_81.g", "w").write(sub(c, B_CODE, "B = 10b-53 + 81b-106;"))
