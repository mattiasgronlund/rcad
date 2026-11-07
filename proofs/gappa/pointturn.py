"""Write point1.g and point2.g, the moved point's y and z: point.g with every
coordinate index turned 0 -> 1 -> 2 -> 0, once and twice. nalgebra's cross
writes component i as v[i+1] * p[i+2] - v[i+2] * p[i+1], indices mod 3, and
the last line works each coordinate on its own, so y is x with the indices
turned once and z with them turned twice.

A name whose first digit is a coordinate index is turned there and keeps any
second digit, which counts products. W, e, d, K and N number terms, shares,
products and the four squares, and are left alone."""

import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))

TURNED = {"v", "p", "s", "P", "q", "C", "c", "rq", "rc", "t", "k", "rk"}
NAME = re.compile(r"\b([A-Za-z]+)(\d)(\d?)(_?)\b")


def turn(src):
    def one(m):
        prefix, index, rest, under = m.groups()
        if prefix not in TURNED:
            return m.group(0)
        return prefix + str((int(index) + 1) % 3) + rest + under

    return NAME.sub(one, src)


point = open("point.g").read()
once = turn(point)
twice = turn(once)
assert turn(twice) == point
X = "# moved point's x is within"
assert point.count(X) == 1
for name, src, axis in (("point1.g", once, "y"), ("point2.g", twice, "z")):
    open(name, "w").write(src.replace(X, "# moved point's " + axis + " is within"))
    print(name, "written,", len(NAME.findall(src)), "names")
