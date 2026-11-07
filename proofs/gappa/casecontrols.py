"""Write the controls of signedcases.py's twelve cases, run beside them. Each
changes one thing in a case: a smaller bound, to see whether it proved its
goal from hypotheses that contradict each other; the weighted sum of k = 2
bounded only as far as the plain sum bounds it, to see whether the proof needs
the unit normal. It also writes each case with L allowed below 2^-1022 as a
proof named <case>_anyl: it still proves, so L needs no lower bound there."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


GOAL = {"signeds": "(value - Ex) / L in [-1b-50, 1b-50]",
        "shifts": "(value - Ex) / (N * L) in [-1b-49, 1b-49]"}
# Just under what the averaging reaches: (1774/1024 + 1/4) 4u = 7.93u for k 0
# or 1, (1708/1024 + 1/4) 4u = 7.67u for k 2.
BELOW = {0: "63b-56", 1: "63b-56", 2: "61b-56"}
WEIGHTED = {"signeds": "9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2| in [0, 1708b-10]",
            "shifts": "(9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N in [0, 1708b-10]"}

for name in ("signeds", "shifts"):
    for k in (0, 1, 2):
        for sign in ("", "n"):
            case = f"{name}{k}{sign}"
            src = open(f"{case}.g").read()
            lhs = GOAL[name].split(" in ")[0]
            b = BELOW[k]
            write(f"c_{case}_below.g", sub(src, GOAL[name], f"{lhs} in [-{b}, {b}]"))
            write(f"{case}_anyl.g", sub(src, "L in [1b-1022, 1b1024]", "L in [1b-1074, 1b1024]"))
            if k == 2:
                # 9/8 of the plain sum's 1774/1024 is all that bound gives.
                # The shift's goal is 16u, so it proves with the plain sum
                # and the file is a proof there, not a control.
                prefix = "c_" if name == "signeds" else ""
                write(f"{prefix}{case}_sumonly.g",
                      sub(src, WEIGHTED[name], WEIGHTED[name].replace("1708b-10", "15966b-13")))
