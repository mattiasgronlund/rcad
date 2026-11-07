# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: Cargo.lock nalgebra at 4b642567
# restates: crates/rcad-geom/src/point.rs as_written at 2b2c90e3
# restates: crates/rcad-geom/src/point.rs transformed at 2b2c90e3
# restates: crates/rcad-geom/src/plane.rs has_length_one_to_within_a_turn at 2b2c90e3
# 464.39's vector.g, the turned vector's x, each component up to 1 + 64u, with zero factors w, v0, n0,
# and so zero P11, P12, P21, P22, C1, C2, K1, K2, KK, M, A, R, each rounded product on a live
# path normal, below the normals or 0, and each rounded sum on one free
# to cancel.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) and the vector n, each an f64, a zero factor the
# double 0.
w = 0; v0 = 0; v1 = rnd(v1_); v2 = rnd(v2_);
n0 = 0; n1 = rnd(n1_); n2 = rnd(n2_);

# What apply_to_vector computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g). A rounded product on a live
# path is written as what mixedn.g and mixeds.g prove of it, rnd(P) =
# P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],
# and a rounded sum on one as what sum.g proves of it, C * r.
P01 = v1 * n2; q01 = rnd(P01);
P02 = v2 * n1; q02 = rnd(P02);
P11 = v2 * n0; q11 = P11;
P12 = v0 * n2; q12 = P12;
P21 = v0 * n1; q21 = P21;
P22 = v1 * n0; q22 = P22;
C0 = q01 - q02; c0 = rnd(C0);
C1 = q11 - q12; c1 = C1;
C2 = q21 - q22; c2 = C2;
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; k01 = K1;
K2 = v2 * t1; k02 = K2;
KK = k01 - k02; k0 = KK;
M = t0 * w; m0 = M;
A = m0 + k0; a0 = A;
R = a0 + n0; out = R;

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * n2 - v2 * n1)
   + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2))
   + n0;


# q.q.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;

{
  # Every part of the turn is a finite f64.
     |v1| <= 1b1024 /\ |v2| <= 1b1024
  # No factor of a product is zero but w, v0, n0.
  /\ |v1| >= 1b-1074 /\ |v2| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  # Every component of the vector is at most 1 + 64u: Axis::as_written keeps a
  # direction whose a.a is within 64 eps of one, so each |a_i| is at most
  # sqrt(1 + 128u), and Axis::transformed turns it without scaling it.
  /\ n1 in [-9007199254741056b-53, 9007199254741056b-53] /\ n2 in [-9007199254741056b-53, 9007199254741056b-53]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ v1 / |v1| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  out - Ex in [-20b-53, 20b-53]
}

