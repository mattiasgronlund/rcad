"""Write vector1.g and vector2.g, the turned vector's y and z: vector.g with
every coordinate index turned 0 -> 1 -> 2 -> 0, once and twice, as 464.19's
turn.py writes point1.g and point2.g from point.g, with n for its p and no
shift.

A name whose first digit is a coordinate index is turned there and keeps any
second digit, which counts products. W, e, d, K and N number terms, shares,
products and q.q, and are left alone."""

import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))

TURNED = {"v", "n", "P", "q", "C", "c", "rq", "rc", "t", "k", "rk"}
NAME = re.compile(r"\b([A-Za-z]+)(\d)(\d?)(_?)\b")


def turn(src):
    def one(m):
        prefix, index, rest, under = m.groups()
        if prefix not in TURNED:
            return m.group(0)
        return prefix + str((int(index) + 1) % 3) + rest + under

    return NAME.sub(one, src)


vector = open("vector.g").read()
once = turn(vector)
twice = turn(once)
assert turn(twice) == vector
X = "# a vector of components at most one. The turned vector's x is within"
assert vector.count(X) == 1
for name, src, axis in (("vector1.g", once, "y"), ("vector2.g", twice, "z")):
    open(name, "w").write(src.replace(X, X[:-len("x is within")] + axis + " is within"))
    print(name, "written,", len(NAME.findall(src)), "names")
