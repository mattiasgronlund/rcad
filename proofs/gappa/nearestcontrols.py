"""Write nearest.g's two controls, which 464.9 describes and no generator of it
wrote: L allowed down to 2^-1074, where a coordinate's slip below 2^-1022 is
no longer small against L, and a bound of 1.6875u in place of 2u, to see that
it did not prove its goal from hypotheses that contradict each other.

Run as: python3 nearestcontrols.py, beside nearest.g.
"""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


nearest = open("nearest.g").read()

write("c_nearest_subnormal.g", sub(nearest, "  /\\ L in [1b-1022, 1b1024]\n", "  /\\ L in [1b-1074, 1b1024]\n"))

write(
    "c_nearest_16875.g",
    sub(
        nearest,
        "  e0 in ? /\\ e1 in ? /\\ e2 in ?\n  /\\ sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 1b-52]\n",
        "  sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 27b-57]\n",
    ),
)
