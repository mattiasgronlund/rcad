"""Write point.g's controls. Each changes one thing in it: a smaller bound,
to see whether it proved its goal from hypotheses that contradict each other,
or a hypothesis dropped, to see whether the proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


point = open("point.g").read()
GOAL = "/\\ (out - Ex) / L in [-36b-53, 36b-53]"

# 24.75u, below the 10 + sqrt(219) = 24.80u the averaging reaches.
write("c_point_2475.g", sub(point, GOAL, "/\\ (out - Ex) / L in [-99b-55, 99b-55]"))

# Without |q.q - 1| <= 6 epsilon.
write("c_point_non4.g",
      sub(point, "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]\n", ""))

# Without the nine products normal, and what lemma.g gives from them.
start = point.index("  # The nine products are normal")
end = point.index("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
end += len("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
block = point[start:end]
assert block.count(">= 1b-1022") == 9 and block.count(" -/ ") == 9, block
write("c_point_nonormal.g", point[:start] + point[end:])
