"""Write vector.g's x, y and z, and the 88 vm_ cases of it with their controls,
with each component of the vector it turns up to 1 + 64u where they take it
at most one, by replacing lines, each asserted found as often as the script it
comes from has it (464.39).

A stored axis direction is what Axis::as_written keeps: a.a, worked out
exactly, within NORMAL_LENGTH_ROUNDOFF, 64 eps or 128u, of one. So each
|a_i| is at most sqrt(1 + 128u), under 1 + 64u, which is
9007199254741056b-53. Axis::transformed turns it without scaling it.

Writes wide.g, wide1.g and wide2.g, their control c_wide_195.g at 19.5u, and
two runs past what a direction can have, r_wide_6.g with each component up to
1 + 2^-6, which proves, and its control c_r_wide_6_five.g with up to 1 + 2^-5,
which fails. The 88 cases are vw_<zeros>.g with c_vw_<zeros>_half.g, from the
vm_ files vectorsub.py wrote; the 11 with every component of the vector zero
have nothing to widen and are not written.
Run as: python3 widen.py, after vectorturn.py and vectorsub.py"""
import os
import subprocess
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
subprocess.run([sys.executable, "vectorturn.py"], check=True)
subprocess.run([sys.executable, "vectorsub.py"], check=True)

AT_MOST_ONE = ("  # Every component of the vector is at most one: a constructor's normal is\n"
               "  # the nearest f64 to a unit vector's components.\n")
WHY = {
    "9007199254741056b-53": (
        "  # Every component of the vector is at most 1 + 64u: Axis::as_written keeps a\n"
        "  # direction whose a.a is within 64 eps of one, so each |a_i| is at most\n"
        "  # sqrt(1 + 128u), and Axis::transformed turns it without scaling it.\n",
        "1 + 64u"),
    "65b-6": (
        "  # Every component of the vector is at most 1 + 2^-6, past what a stored\n"
        "  # axis direction can have, to show how far the 20u reaches.\n",
        "1 + 2^-6"),
    "33b-5": (
        "  # Every component of the vector is at most 1 + 2^-5, past what a stored\n"
        "  # axis direction can have, to show how far the 20u reaches.\n",
        "1 + 2^-5"),
}
DIRECTION = "9007199254741056b-53"
VECTOR = ("n0", "n1", "n2")
LAST_RESTATES = "# restates: Cargo.lock nalgebra at 4b642567\n"
AXIS_RESTATES = ("# restates: crates/rcad-geom/src/point.rs as_written at 2b2c90e3\n"
                 "# restates: crates/rcad-geom/src/point.rs transformed at 2b2c90e3\n"
                 "# restates: crates/rcad-geom/src/plane.rs has_length_one_to_within_a_turn at 2b2c90e3\n")


def replace(src, old, new, times):
    assert src.count(old) == times, (old, src.count(old), times)
    return src.replace(old, new)


def widen(src, bound, kept):
    """src with each component of the vector that is not zero, those named in
    kept, up to bound, restating the axis's code beside the rest."""
    why, size = WHY[bound]
    src = replace(src, LAST_RESTATES, LAST_RESTATES + AXIS_RESTATES, 1)
    src = replace(src, AT_MOST_ONE, why, 1 if kept else 0)
    for f in VECTOR:
        src = replace(src, f" {f} in [-1, 1]", f" {f} in [-{bound}, {bound}]", 1 if f in kept else 0)
    return src, size


def main():
    written = []
    for name, base in (("wide.g", "vector.g"), ("wide1.g", "vector1.g"), ("wide2.g", "vector2.g")):
        src, size = widen(open(base).read(), DIRECTION, VECTOR)
        src = replace(src, "# a vector of components at most one.", f"# a vector of components at most {size}.", 1)
        open(name, "w").write(src)
        written.append(name)
    wide = open("wide.g").read()
    goal = "out - Ex in [-20b-53, 20b-53]"
    open("c_wide_195.g", "w").write(replace(wide, goal, "out - Ex in [-39b-54, 39b-54]", 1))
    written.append("c_wide_195.g")
    for name, bound in (("r_wide_6.g", "65b-6"), ("c_r_wide_6_five.g", "33b-5")):
        src, size = widen(open("vector.g").read(), bound, VECTOR)
        src = replace(src, "# a vector of components at most one.", f"# a vector of components at most {size}.", 1)
        open(name, "w").write(src)
        written.append(name)
    print("written:", ", ".join(written))

    cases = [n.strip() for n in open("casesv.txt") if n.strip()]
    count = 0
    for name in cases:
        assert name.startswith("vm_"), name
        for old, new in ((name + ".g", "vw_" + name[3:] + ".g"),
                         ("c_" + name + "_half.g", "c_vw_" + name[3:] + "_half.g")):
            if not os.path.exists(old):
                assert old.startswith("c_"), old
                continue
            text = open(old).read()
            kept = [f for f in VECTOR if f" {f} in [-1, 1]" in text]
            if not kept:
                continue
            src, size = widen(text, DIRECTION, kept)
            src = replace(src, "# 464.20's vector.g, the turned vector's x,",
                          f"# 464.39's vector.g, the turned vector's x, each component up to {size},", 1)
            open(new, "w").write(src)
            count += 1
    print(len(cases), "cases,", count, "scripts written as vw_")


main()
