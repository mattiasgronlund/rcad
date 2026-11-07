"""Write dk_<set>.g for every set of one or more differences that leaves
exactly one of t1 and t2 zero by its shape: dz_<set>.g, which
make_differences_zero.py writes beside it, with block 1's hypothesis that
t1 + t2 does not cancel taken out. With no arguments, print the sets, one
line each, and write nothing; with --write, write the scripts and print
their names."""
import itertools
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))

DIFFERENCES = ["adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz"]
PRODUCT = {1: ("bdx", "cdy"), 2: ("cdx", "bdy"), 3: ("cdx", "ady"), 4: ("adx", "cdy"), 5: ("adx", "bdy"), 6: ("bdx", "ady")}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
HYPOTHESIS = "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n"


def shape_zero_terms(zero):
    dead = {k for k, (f, g) in PRODUCT.items() if f in zero or g in zero}
    return {m for m in (1, 2, 3) if Z_OF[m] in zero or {2 * m - 1, 2 * m} <= dead}


sets = [
    "_".join(zero)
    for n in range(1, 10)
    for zero in itertools.combinations(DIFFERENCES, n)
    if len(shape_zero_terms(set(zero)) & {1, 2}) == 1
]
if sys.argv[1:] == ["--write"]:
    for name in sets:
        src = open(f"dz_{name}.g").read()
        assert src.count(HYPOTHESIS) == 1, name
        assert "t1 + t2 -> t" in src, name
        with open(f"dk_{name}.g", "w") as f:
            f.write(src.replace(HYPOTHESIS, ""))
        print(f"dk_{name}")
else:
    print("\n".join(sets))
