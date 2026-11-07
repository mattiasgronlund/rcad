"""Write square.g's cases for a normal that does not come straight from a
constructor, from 464.20's square.g by replacing its lines, each asserted
found once, as 464.36's squarezero.py does.

square.g has n = rnd(m) for an exact unit vector m, and averages the
constructor's slip over the m_i^2 to put n.n within 2u of one. Here n is any
f64 vector whose n.n is held within K u of one, which is all that
from_point_and_normal_as_written asks of a normal it keeps (K = 128) and all
that square.g proves of a normal one turn has moved (K = 119.28): so m, its
shares and their hints go, and n.n - 1 is a hypothesis. Everything else is
square.g's: the turn's q.q within 6 epsilon of one, and each turned
component within vector.g's 20u of the turn's own arithmetic. vector.g proves
that 20u for components at most one; these reach 1 + K/2 u, and Q2831 asks
whether it holds there. So a case that proves rests on Q2831, and one that
fails does not: a larger d could only fail by more."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

square = open("square.g").read()


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


HEAD = """# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at 4b642567
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: crates/rcad-geom/src/point.rs scaled_to_length_one_from_anywhere at 4b642567
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# Plane::NORMAL_LENGTH_ROUNDOFF, crates/rcad-geom/src/plane.rs at 4b642567:
# a constructor's normal, turned once, has n.n within 64 * f64::EPSILON of
# one. The constructor rounds each component of an exact unit vector m to
# the nearest f64; the turn is nalgebra's arithmetic, and vector.g,
# vector1.g and vector2.g prove each turned component within 20u of what
# that arithmetic gives without rounding, which is the hypothesis on d0, d1
# and d2 here.
"""

NEW_HEAD = """# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at b1f1bf1a
# restates: crates/rcad-geom/src/plane.rs transformed at b1f1bf1a
# restates: crates/rcad-geom/src/plane.rs from_point_and_normal_as_written at b1f1bf1a
# restates: crates/rcad-geom/src/plane.rs has_length_one_to_within_a_turn at b1f1bf1a
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at b1f1bf1a
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# 464.37: Plane::NORMAL_LENGTH_ROUNDOFF for a normal that does not come
# straight from a constructor, turned once: n is any f64 vector with n.n
# within {k}u of one, {what}. This is 464.20's square.g with its
# constructor taken out. The hypothesis on d0, d1 and d2 is vector.g's 20u,
# which vector.g proves for components at most one; these reach 1 + {half}u
# (Q2831).
"""

CONSTRUCTOR = """# The constructor: n is the nearest f64 to the exact unit vector m.
n0 = rnd(m0); n1 = rnd(m1); n2 = rnd(m2);
M2 = m0 * m0 + m1 * m1 + m2 * m2;
NN = n0 * n0 + n1 * n1 + n2 * n2;
"""

NEW_CONSTRUCTOR = """# The normal: any f64 vector, held below to n.n near one.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
NN = n0 * n0 + n1 * n1 + n2 * n2;
"""

SHARES = """
# The constructor's shares: m_i^2, and what rounding m_i adds to its square.
g0 = m0 * m0; g1 = m1 * m1; g2 = m2 * m2;
h0 = n0 * n0 - m0 * m0; h1 = n1 * n1 - m1 * m1; h2 = n2 * n2 - m2 * m2;
"""

M = """  # m is a unit vector, so each component is at most one.
     M2 in [1, 1]
  /\\ m0 in [-1, 1] /\\ m1 in [-1, 1] /\\ m2 in [-1, 1]
  # Above 2^-1012 each component is the nearest f64 to the exact ratio
  # (scaled_to_length_one_from_anywhere's doc), and none is zero.
  /\\ |m0| >= 1b-1012 /\\ |m1| >= 1b-1012 /\\ |m2| >= 1b-1012
"""

NEW_M = """  # n.n within {k}u of one, so each component is at most 1 + {half}u.
     NN - 1 in [-{k2}b-54, {k2}b-54]
  /\\ n0 in [-{b}b-54, {b}b-54] /\\ n1 in [-{b}b-54, {b}b-54]
  /\\ n2 in [-{b}b-54, {b}b-54]
"""

AVERAGING = """
# The constructor: each n_i^2 is m_i^2 times a factor within 2u + u^2 of
# one, and the m_i^2 add to one; add_rr averages them.
NN - 1 -> ((NN - M2) / M2) * M2 + (M2 - 1);
(NN - M2) / M2 -> ((M2 + (NN - M2)) - M2) / M2;
M2 + (NN - M2) -> (g0 + h0) + (g1 + h1) + (g2 + h2);
(g0 + h0) - g0 -> h0;
(g1 + h1) - g1 -> h1;
(g2 + h2) - g2 -> h2;
h0 / g0 -> (n0 / m0) * (n0 / m0) - 1;
h1 / g1 -> (n1 / m1) * (n1 / m1) - 1;
h2 / g2 -> (n2 / m2) * (n2 / m2) - 1;
n0 / m0 -> 1 + (n0 - m0) / m0;
n1 / m1 -> 1 + (n1 - m1) / m1;
n2 / m2 -> 1 + (n2 - m2) / m2;
"""

GOAL = "/\\ OO - 1 in [-128b-53, 128b-53]"

# k is in units of u = 2^-53, written in halves so that 1 + k/2 u is a whole
# number of 2^-54.
CASES = (
    ("c_kept_10_k128.g", 256, "as much as from_point_and_normal_as_written keeps"),
    ("c_kept_10_k119.g", 238, "inside what square.g proves of a normal turned once"),
    ("c_kept_10_k11.g", 22, "the first whole u past what one turn leaves room for"),
    ("kept_10.g", 20, "the last whole u inside what one turn leaves room for"),
)

written = []
for name, k2, what in CASES:
    k = k2 / 2
    half = k / 2
    b = 2 ** 54 + k2 // 2
    src = sub(square, HEAD, NEW_HEAD.format(k=f"{k:g}", what=what, half=f"{half:g}"))
    src = sub(src, CONSTRUCTOR, NEW_CONSTRUCTOR)
    src = sub(src, SHARES, "")
    src = sub(src, M, NEW_M.format(k=f"{k:g}", half=f"{half:g}", k2=k2, b=b))
    src = sub(src, AVERAGING, "")
    open(name, "w").write(src)
    written.append(name)
    if name == "kept_10.g":
        # The goal at 127u, under the 10u + 48u + 69.28u the parts reach.
        control = sub(src, GOAL, "/\\ OO - 1 in [-127b-53, 127b-53]")
        open("c_kept_10_127.g", "w").write(control)
        written.append("c_kept_10_127.g")
print("written:", ", ".join(written))
