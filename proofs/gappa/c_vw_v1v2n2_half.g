# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: Cargo.lock nalgebra at 4b642567
# restates: crates/rcad-geom/src/point.rs as_written at 2b2c90e3
# restates: crates/rcad-geom/src/point.rs transformed at 2b2c90e3
# restates: crates/rcad-geom/src/plane.rs has_length_one_to_within_a_turn at 2b2c90e3
# 464.39's vector.g, the turned vector's x, each component up to 1 + 64u, with zero factors v1, v2, n2,
# and so zero P01, P02, P11, P12, P22, C0, C1, K1, K2, KK, M, A, each rounded product on a live
# path normal, below the normals or 0, and each rounded sum on one free
# to cancel.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) and the vector n, each an f64, a zero factor the
# double 0.
w = rnd(w_); v0 = rnd(v0_); v1 = 0; v2 = 0;
n0 = rnd(n0_); n1 = rnd(n1_); n2 = 0;

# What apply_to_vector computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g). A rounded product on a live
# path is written as what mixedn.g and mixeds.g prove of it, rnd(P) =
# P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],
# and a rounded sum on one as what sum.g proves of it, C * r.
P01 = v1 * n2; q01 = P01;
P02 = v2 * n1; q02 = P02;
P11 = v2 * n0; q11 = P11;
P12 = v0 * n2; q12 = P12;
P21 = v0 * n1; q21 = rnd(P21);
P22 = v1 * n0; q22 = P22;
C0 = q01 - q02; c0 = C0;
C1 = q11 - q12; c1 = C1;
C2 = q21 - q22; c2 = rnd(C2);
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; k01 = K1;
K2 = v2 * t1; k02 = K2;
KK = k01 - k02; k0 = KK;
M = t0 * w; m0 = M;
A = m0 + k0; a0 = A;
R = a0 + n0; out = R * rr;

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * n2 - v2 * n1)
   + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2))
   + n0;

# Each term of Ex as out carries it, less the term itself, through the
# roundings left on its path. A term with a zero factor is zero.
W7 = rr - 1;
e7 = n0 * W7;

# Each live term's share of D, as vector.g's, with 1 for the vector's
# component.
d7 = 1;
D = d7;

# q.q and the four squares D is written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 25/93 * (|v1| + |v2|);
Y = |v0| - 10/31 * (|v1| + |v2|);
Z = |v1| - |v2|;

{
  # Every part of the turn is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024
  # No factor of a product is zero but v1, v2, n2.
  /\ |w| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074
  # What sum.g proves of each rounded sum on a live path, the sum of two
  # doubles, or 0 where it cancels, which any ratio meets.
  /\ rr in [9007199254740991b-53, 9007199254740993b-53]
  # Every component of the vector is at most 1 + 64u: Axis::as_written keeps a
  # direction whose a.a is within 64 eps of one, so each |a_i| is at most
  # sqrt(1 + 128u), and Axis::transformed turns it without scaling it.
  /\ n0 in [-9007199254741056b-53, 9007199254741056b-53] /\ n1 in [-9007199254741056b-53, 9007199254741056b-53]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v0 / |v0| in [-1, 1]
->
  (out - Ex) / D in ? /\ D in ?
  /\ out - Ex in [-1b-54, 1b-54]
}

# The terms' error is the sum of the live shares' errors, each a fraction
# of its own share of D; add_rr then averages them.
out - Ex -> ((out - Ex) / D) * D;
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (d7 + e7);
(d7 + e7) - d7 -> e7;
e7 / d7 -> n0 * W7;

# D is vector.g's 93/5 q.q less four squares, plus 1 for n0, less the
# share of each term a zero factor took out, and a zero v's square.
D -> 93/5 * N4 - (93/5 * (X * X) + 93/5 * (Y * Y) + 305/93 * (Z * Z)) + 1
     - (10 * |w| * |v1| + 10 * |w| * |v2| + 12 * |v1| * |v0| + 12 * |v1| * |v1| + 12 * |v2| * |v2| + 12 * |v2| * |v0| + 19/465 * |v1| * |v1| + 19/465 * |v2| * |v2|);
