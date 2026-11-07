"""Write the controls of §464.7's scripts with a minor at zero: each script at
a smaller B, where it must fail, to see that it did not prove its goal from
hypotheses that contradict each other.

Run as: python3 zerocontrols.py, beside zero2.g, secondzero.g, thirdzero.g
and the zero*d.g that make_two_zero.py writes.
"""

B = "B = 7b-53 + 7b-103;"


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


# §464.7 3: first zero and second zero fail at 5u, best ±1.2.
write("c_zero2_5u.g", sub(open("zero2.g").read(), B, "B = 5b-53;"))
write("c_secondzero_5u.g", sub(open("secondzero.g").read(), B, "B = 5b-53;"))

# §464.7 3: third zero fails at 6u, best ±1.16667, and at 7u + 46u^2, best ±1.
third = open("thirdzero.g").read()
write("c_thirdzero_6u.g", sub(third, B, "B = 6b-53;"))
write("c_thirdzero_46.g", sub(third, B, "B = 7b-53 + 23b-105;"))

# §464.7 3: two at zero prove within 6u + 13u^2. The script for first and
# second fails at 5u, best ±1.2; the other two are held to the same 5u.
for pair in ("12", "13", "23"):
    write(f"c_zero{pair}d_5u.g", sub(open(f"zero{pair}d.g").read(), B, "B = 5b-53;"))
