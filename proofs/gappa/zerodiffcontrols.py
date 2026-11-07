"""Write the controls of §464.11's nine scripts: each at a smaller B, where it
must fail, to see that it did not prove its goal from hypotheses that
contradict each other. §464.11 runs them at 6u and at 5u.

Run as: python3 zerodiffcontrols.py, beside the zd_*.g that
make_one_difference_zero.py writes.
"""

B = "B = 7b-53 + 7b-103;"
NAMES = ("adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz")


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


for name in NAMES:
    src = open(f"zd_{name}.g").read()
    open(f"c_zd_{name}_6u.g", "w").write(sub(src, B, "B = 6b-53;"))
    open(f"c_zd_{name}_5u.g", "w").write(sub(src, B, "B = 5b-53;"))
