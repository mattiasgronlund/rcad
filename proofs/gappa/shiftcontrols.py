"""Write shiftplane.g's and shiftball.g's controls. Each changes one thing in
its script: a smaller bound, to see whether it proved its goal from
hypotheses that contradict each other, or a hypothesis dropped, to see
whether the proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


plane = open("shiftplane.g").read()
ball = open("shiftball.g").read()

# 7.875u, below the (4 sqrt(3) + 1)u = 7.93u the averaging reaches.
write("c_shiftplane_7875.g", sub(plane, "/\\ (value - Ex) / (N * L) in [-1b-49, 1b-49]",
                                 "/\\ (value - Ex) / (N * L) in [-63b-56, 63b-56]"))

# Without the three products normal, and what lemma.g gives from them.
src = sub(plane, "  /\\ |t0| >= 1b-1022 /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022\n", "")
src = sub(src, "  /\\ t0 -/ n0 * d0 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t1 -/ n1 * d1 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t2 -/ n2 * d2 in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
write("c_shiftplane_nonormal.g", src)

# 3.4u, below the 2 sqrt(3) u = 3.46u the doc comment counts.
write("c_shiftball_34.g", sub(ball, "sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 1b-49]",
                              "sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 435b-60]"))

# The centre on a grid of 2^-1100 rather than an f64: a sum below 2^-1022 is
# then no longer exact, and L may be below 2^-1022. (Written as a plain real,
# c0 = c0_, Gappa 1.8.0 aborts in ast.cpp printing a warning.)
write("c_shiftball_finer.g", sub(ball, "c0 = rnd(c0_); c1 = rnd(c1_); c2 = rnd(c2_);",
                                 "c0 = fixed<-1100, ne>(c0_); c1 = fixed<-1100, ne>(c1_);"
                                 " c2 = fixed<-1100, ne>(c2_);"))
