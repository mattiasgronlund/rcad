"""Write block 1 with t1 + t2 cancelling to zero while neither t1 nor t2 is
zero, with one or more of the nine differences zero.

Run beside block1.g and the dz_<differences>.g that make_differences_zero.py
writes. Each argument is a set of differences such as adx_bdy, in the order
DIFFERENCES gives them, or "none" for block 1 with no difference zero, as
464.4 ran the branch. With no arguments, write "none" and every set of one or
more differences that leaves none of t1, t2 and t3 zero by its shape, and
print the names, one line each. With --sets, print those sets of
differences, one line each, and write nothing: make_differences_zero.py takes
them as its arguments.

Each is ds_<set>.g, with the controls c_ds_<set>_6u.g (not for the six pairs
in PROVES_AT_6U, which prove at 6u) and c_ds_<set>_5u.g,
and, where a minor rounds a double less a zero, dsq_<set>.g: the script with
its q hypotheses taken out and only its |q| goals left. In each, s12 is S, as
464.4 writes the branch: a sum that is zero rounds to itself, and nothing
there is rounded.
"""
import itertools
import os
import re
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))

DIFFERENCES = ["adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz"]
PRODUCT = {
    1: ("bdx", "cdy"),
    2: ("cdx", "bdy"),
    3: ("cdx", "ady"),
    4: ("adx", "cdy"),
    5: ("adx", "bdy"),
    6: ("bdx", "ady"),
}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
# These six prove at 6u, so their 6u script is no control (464.44).
PROVES_AT_6U = {"adx_bdy", "adx_cdy", "ady_bdx", "ady_cdx", "bdx_cdy", "bdy_cdx"}


def main():
    if sys.argv[1:] == ["--sets"]:
        for zero in every_branch():
            print("_".join(zero))
        return
    if sys.argv[1:]:
        sets = [[] if arg == "none" else arg.split("_") for arg in sys.argv[1:]]
    else:
        sets = [[]] + every_branch()
    for zero in sets:
        assert not shape_zero_terms(set(zero)), zero
        name = "_".join(zero) or "none"
        src = open(f"dz_{name}.g" if zero else "block1.g").read()
        src = cancel_first_two(src, zero)
        write(f"ds_{name}.g", src)
        if name not in PROVES_AT_6U:
            write(f"c_ds_{name}_6u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 6b-53;"))
        write(f"c_ds_{name}_5u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 5b-53;"))
        if "  /\\ q" in src:
            write(f"dsq_{name}.g", q_goals_alone(src))
        print(name)


def dead_products(zero):
    return {k for k, (f, g) in PRODUCT.items() if f in zero or g in zero}


def shape_zero_terms(zero):
    dead = dead_products(zero)
    return {m for m in (1, 2, 3) if Z_OF[m] in zero or {2 * m - 1, 2 * m} <= dead}


def every_branch():
    """Every set of one or more differences with none of the three terms zero
    by its shape."""
    return [
        list(zero)
        for n in range(1, 10)
        for zero in itertools.combinations(DIFFERENCES, n)
        if not shape_zero_terms(set(zero))
    ]


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def cancel_first_two(src, zero):
    """The script with s12 the exact sum t1 + t2: 464.4's branch, which
    takes rs out of W1 to W4 and drops rs, its hint and the hypothesis that
    t1 + t2 does not cancel."""
    if zero:
        differences = " and ".join(zero)
        src = sub(
            src,
            f"# Block 1 where {differences} are zero. A zero rounds to itself,\n"
            "# and so does a product with a zero factor.\n",
            f"# Block 1 where {differences} are zero and t1 + t2 cancels to zero,\n"
            "# so s12 rounds nothing. A zero rounds to itself, and so does a\n"
            "# product with a zero factor.\n",
        )
    else:
        src = sub(
            src,
            "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
            "# crates/rcad-predicates/src/orient3d.rs at d9f931e8, where t1 + t2\n"
            "# cancels to zero, so s12 rounds nothing.\n",
        )
    alive = 4 - len(dead_products(set(zero)) & {1, 2, 3, 4})
    src = sub(src, "s12 = rnd(S);", "s12 = S;")
    src = sub(src, "rs = s12 / S;\n", "")
    assert src.count(" * rs - 1") == alive, (zero, alive)
    src = src.replace(" * rs - 1", " - 1")
    src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
    src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
    assert not re.search(r"(?<![.\w])rs\b", src), zero
    return src


def q_goals_alone(src):
    """The script with its q hypotheses out and its |q| goals alone left."""
    lines = src.split("\n")
    lines = [l for l in lines if not l.startswith("  /\\ q")]
    start = lines.index("->")
    end = lines.index("}", start)
    goals = [l for l in lines[start + 1 : end] if l.startswith("  /\\ |q")]
    goals[0] = "  " + goals[0][len("  /\\ ") :]
    return "\n".join(lines[: start + 1] + goals + lines[end:])


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


main()
