"""Write the control runs. Each changes one thing in a script that proves: a
smaller B, to see whether it proved its goal from hypotheses that contradict
each other, or a hypothesis dropped, to see whether the proof needs it.

464.8 6 quotes this as controls.py, a name 464.9's generator has too, and
prove-bounds holds every c_*.g to failing. So what 464.8 found proves is not
written as a control here: the four o2 cases at 3u + 14u^2 are o2s_14.g and
the like, and plane.g without eb normal is plane_noeb.g, with the run that
fails on why, c_plane_noeb_why.g, as its control. o2v.g, the claim on value
that fails, is c_o2_value.g, which make_value.py writes. o2.g's and plane.g's
own runs below the code's B, which 464.8 1 and 3 ran by hand, are written
here as c_o2_14.g, c_o2_3u.g and c_plane_14.g.

Run as: python3 o2planecontrols.py, beside the scripts, once o2cases.py and
make_value.py have run.
"""


def sub(src, old, new):
    assert src.count(old) == 1, (old, src[:80])
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


B_O2 = "B = 3b-53 + 1b-102;"
for name in ("o2s", "o2sn", "o2f", "o2fn"):
    write(f"c_{name}_3u.g", sub(open(f"{name}.g").read(), B_O2, "B = 3b-53;"))
    write(f"{name}_14.g", sub(open(f"{name}.g").read(), B_O2, "B = 3b-53 + 14b-106;"))

# o2.g at 3u + 14u^2, one u^2 under the smallest B it proves, and at 3u.
write("c_o2_14.g", sub(open("o2.g").read(), B_O2, "B = 3b-53 + 14b-106;"))
write("c_o2_3u.g", sub(open("o2.g").read(), B_O2, "B = 3b-53;"))

# o2tiny without eb normal.
write("c_o2tiny_noeb.g", sub(open("o2tiny.g").read(), "  /\\ |eb| >= 1b-1022\n", ""))

# The claim on value for orient2d at 4u + 16u^2.
write("c_o2_value4u.g", sub(open("c_o2_value.g").read(), B_O2, "B = 4b-53 + 1b-102;"))

# planev below the code's B.
B_PL = "B = 4b-53 + 1b-101;"
write("c_planev_4u.g", sub(open("planev.g").read(), B_PL, "B = 4b-53;"))
write("c_planev_3u.g", sub(open("planev.g").read(), B_PL, "B = 3b-53 + 1b-101;"))

# plane.g at 3u + 14u^2, one u^2 under the smallest B it proves.
write("c_plane_14.g", sub(open("plane.g").read(), B_PL, "B = 3b-53 + 14b-106;"))

# plane.g without product_is_normal, and what lemma.g gives from it.
src = open("plane.g").read()
src = sub(src, "  # product_is_normal on the three products.\n  /\\ |t0| >= 1b-1022 /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022\n", "")
src = sub(src, "  /\\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
src = sub(src, "  (y - Ex) / P3 in ? /\\ P -/ Phat in ?\n", "  (y - Ex) / P3 in ? /\\ P -/ Phat in ? /\\ rp0 in ?\n")
write("c_plane_nonormal.g", src)

# plane.g without eb normal, which proves; asked how far eb is from B * P, it
# finds no bound.
src = open("plane.g").read()
src = sub(src, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
src = sub(src, "  /\\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
write("plane_noeb.g", src)
write("c_plane_noeb_why.g", sub(src, "  (y - Ex) / P3 in ?", "  P in ? /\\ eb -/ B * P in ? /\\ (y - Ex) / P3 in ?"))
