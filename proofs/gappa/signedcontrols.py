"""Write signed.g's controls. Each changes one thing in it: a smaller bound,
to see whether it proved its goal from hypotheses that contradict each other,
or a hypothesis dropped, to see whether the proof needs it.

464.9 quotes this as controls.py, a name 464.8's generator has too."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


signed = open("signed.g").read()
GOAL = "/\\ (value - Ex) / L in [-1b-50, 1b-50]"

# 7.875u, below the (4 sqrt(3) + 1)u = 7.93u the doc comment counts.
write("c_signed_7875.g", sub(signed, GOAL, "/\\ (value - Ex) / L in [-63b-56, 63b-56]"))

# Without |n0| + |n1| + |n2| bounded.
write("c_signed_nosum.g", sub(signed, "  /\\ |n0| + |n1| + |n2| in [0, 1774b-10]\n", ""))

# Without the three products normal, and what lemma.g gives from them.
src = sub(signed, "  /\\ |t0| >= 1b-1022 /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022\n", "")
src = sub(src, "  /\\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
write("c_signed_nonormal.g", src)
