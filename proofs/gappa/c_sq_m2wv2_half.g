# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at 4b642567
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
# 464.36: with zero factors m2, w, v2, each the double 0.
@rnd = float<ieee_64, ne>;

# The constructor: n is the nearest f64 to the exact unit vector m.
n0 = rnd(m0); n1 = rnd(m1); n2 = 0;
M2 = m0 * m0 + m1 * m1;
NN = n0 * n0 + n1 * n1 + n2 * n2;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = 0; v0 = rnd(v0_); v1 = rnd(v1_); v2 = 0;
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;

# nalgebra's turn of n without rounding, as vector.g's Ex writes x.
Ex0 = 2 * w * (v1 * n2 - v2 * n1)
    + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2)) + n0;
Ex1 = 2 * w * (v2 * n0 - v0 * n2)
    + v2 * (2 * (v1 * n2 - v2 * n1)) - v0 * (2 * (v0 * n1 - v1 * n0)) + n1;
Ex2 = 2 * w * (v0 * n1 - v1 * n0)
    + v0 * (2 * (v2 * n0 - v0 * n2)) - v1 * (2 * (v1 * n2 - v2 * n1)) + n2;
ExEx = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;

# The turned vector the f64 arithmetic gives: Ex and each rounding's slip.
o0 = Ex0 + d0; o1 = Ex1 + d1; o2 = Ex2 + d2;
OO = o0 * o0 + o1 * o1 + o2 * o2;
DD = d0 * d0 + d1 * d1 + d2 * d2;
F2 = 2 * (Ex0 * d0 + Ex1 * d1 + Ex2 * d2);

# v x n and v . n, without rounding, and v.v.
X0 = v1 * n2 - v2 * n1; X1 = v2 * n0 - v0 * n2; X2 = v0 * n1 - v1 * n0;
VN = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * n0 + v1 * n1 + v2 * n2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

# The constructor's shares: m_i^2, and what rounding m_i adds to its square.
g0 = m0 * m0; g1 = m1 * m1;
h0 = n0 * n0 - m0 * m0; h1 = n1 * n1 - m1 * m1;

{
  # m is a unit vector, so each component is at most one.
     M2 in [1, 1]
  /\ m0 in [-1, 1] /\ m1 in [-1, 1]
  # Above 2^-1012 each component is the nearest f64 to the exact ratio
  # (scaled_to_length_one_from_anywhere's doc), and none is zero but m2.
  /\ |m0| >= 1b-1012 /\ |m1| >= 1b-1012
  # Every quaternion component is a finite f64.
  /\ |v0| <= 1b1024 /\ |v1| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # What vector.g, vector1.g and vector2.g prove of each turned component.
  /\ d0 in [-20b-53, 20b-53] /\ d1 in [-20b-53, 20b-53]
  /\ d2 in [-20b-53, 20b-53]
->
  NN - 1 in ? /\ VN in ? /\ 4 * (N4 - 1) * VN in ? /\ F2 in ? /\ DD in ?
  /\ OO - 1 in [-1b-54, 1b-54]
}

# n.n off one is the constructor's, then the turn's own arithmetic, then the
# turn's rounding.
OO - 1 -> (NN - 1) + 4 * (N4 - 1) * VN + F2 + DD;
ExEx -> NN + 4 * (N4 - 1) * VN;

# The constructor: each n_i^2 is m_i^2 times a factor within 2u + u^2 of
# one, and the m_i^2 add to one; add_rr averages them.
NN - 1 -> ((NN - M2) / M2) * M2 + (M2 - 1);
(NN - M2) / M2 -> ((M2 + (NN - M2)) - M2) / M2;
M2 + (NN - M2) -> (g0 + h0) + (g1 + h1);
(g0 + h0) - g0 -> h0;
(g1 + h1) - g1 -> h1;
h0 / g0 -> (n0 / m0) * (n0 / m0) - 1;
h1 / g1 -> (n1 / m1) * (n1 / m1) - 1;
n0 / m0 -> 1 + (n0 - m0) / m0;
n1 / m1 -> 1 + (n1 - m1) / m1;

# |v x n|^2 is at least zero, a sum of squares, and at most v.v n.n, since
# what is between is (v . n)^2; v.v is q.q less w^2.
VN -> VV * NN - VD * VD;
VV -> N4 - w * w;

# 2 Ex . d, both ways round: 2ab is c a^2 + b^2 / c less (c a - b)^2 / c,
# and (c a + b)^2 / c less c a^2 + b^2 / c. c is near sqrt(3) * 20u, which
# makes c Ex.Ex + DD / c smallest for Ex.Ex near one and DD at most 3 (20u)^2.
F2 -> (34641b-53 / 1000 * ExEx + DD / (34641b-53 / 1000))
      - ((34641b-53 / 1000 * Ex0 - d0) * (34641b-53 / 1000 * Ex0 - d0)
       + (34641b-53 / 1000 * Ex1 - d1) * (34641b-53 / 1000 * Ex1 - d1)
       + (34641b-53 / 1000 * Ex2 - d2) * (34641b-53 / 1000 * Ex2 - d2))
        / (34641b-53 / 1000);
F2 -> ((34641b-53 / 1000 * Ex0 + d0) * (34641b-53 / 1000 * Ex0 + d0)
     + (34641b-53 / 1000 * Ex1 + d1) * (34641b-53 / 1000 * Ex1 + d1)
     + (34641b-53 / 1000 * Ex2 + d2) * (34641b-53 / 1000 * Ex2 + d2))
        / (34641b-53 / 1000)
      - (34641b-53 / 1000 * ExEx + DD / (34641b-53 / 1000));
