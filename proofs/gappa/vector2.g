# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: Cargo.lock nalgebra at 4b642567
# Isometry::apply_to_vector, crates/rcad-geom/src/isometry.rs at 4b642567,
# through nalgebra 0.35's UnitQuaternion * Vector3: point.g with no shift and
# a vector of components at most one. The turned vector's z is within
# 20u of what the same arithmetic gives without rounding. y and z are this
# with the indices turned 0 -> 1 -> 2 -> 0, as turn.py writes them.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) and the vector n, each an f64.
w = rnd(w_); v2 = rnd(v2_); v0 = rnd(v0_); v1 = rnd(v1_);
n2 = rnd(n2_); n0 = rnd(n0_); n1 = rnd(n1_);

# c = v x n, as nalgebra's cross writes each component: a*b - c*d.
P21 = v0 * n1; q21 = rnd(P21); P22 = v1 * n0; q22 = rnd(P22);
C2 = q21 - q22; c2 = rnd(C2);
P01 = v1 * n2; q01 = rnd(P01); P02 = v2 * n1; q02 = rnd(P02);
C0 = q01 - q02; c0 = rnd(C0);
P11 = v2 * n0; q11 = rnd(P11); P12 = v0 * n2; q12 = rnd(P12);
C1 = q11 - q12; c1 = rnd(C1);

# t = c * 2, written without its rounding, on the strength of 464.19's
# doubling.g.
t2 = c2 * 2; t0 = c0 * 2; t1 = c1 * 2;

# k = v x t, its x.
K1 = v0 * t1; k21 = rnd(K1); K2 = v1 * t0; k22 = rnd(K2);
KK = k21 - k22; k2 = rnd(KK);

# t * w + k + n, folded left to right. A vector takes no shift.
M = t2 * w; m0 = rnd(M);
A = m0 + k2; a0 = rnd(A);
R = a0 + n2; out = rnd(R);

# The same arithmetic without rounding.
Ex = 2 * w * (v0 * n1 - v1 * n0)
   + v0 * (2 * (v2 * n0 - v0 * n2)) - v1 * (2 * (v1 * n2 - v2 * n1))
   + n2;

# What each rounding multiplied its argument by.
rq21 = q21 / P21; rq22 = q22 / P22; rq01 = q01 / P01; rq02 = q02 / P02;
rq11 = q11 / P11; rq12 = q12 / P12;
rc2 = c2 / C2; rc0 = c0 / C0; rc1 = c1 / C1;
rk21 = k21 / K1; rk22 = k22 / K2; rk2 = k2 / KK;
rm = m0 / M; ra = a0 / A; rr = out / R;

# The seven terms of Ex, each as out carries it, less the term itself: five
# roundings for the two through w, six for the four through k, one for n2.
W1 = rr * ra * rm * rc2 * rq21 - 1;
W2 = rr * ra * rm * rc2 * rq22 - 1;
W3 = rr * ra * rk2 * rk21 * rc1 * rq11 - 1;
W4 = rr * ra * rk2 * rk21 * rc1 * rq12 - 1;
W5 = rr * ra * rk2 * rk22 * rc0 * rq01 - 1;
W6 = rr * ra * rk2 * rk22 * rc0 * rq02 - 1;
W7 = rr - 1;
e1 = 2 * w * v0 * n1 * W1; e2 = -(2 * w * v1 * n0 * W2);
e3 = 2 * v0 * v2 * n0 * W3; e4 = -(2 * v0 * v0 * n2 * W4);
e5 = -(2 * v1 * v1 * n2 * W5); e6 = 2 * v1 * v2 * n1 * W6;
e7 = n2 * W7;

# Each term's share of the bound: its size with 1 for the vector's
# component, times the roundings it meets.
d1 = 10 * |w| * |v0|; d2 = 10 * |w| * |v1|;
d3 = 12 * |v0| * |v2|; d4 = 12 * |v0| * |v0|;
d5 = 12 * |v1| * |v1|; d6 = 12 * |v1| * |v2|;
d7 = 1;
D = d1 + d2 + d3 + d4 + d5 + d6 + d7;

# q.q, each square written as |x| * |x|, and the four squares D is written
# with below.
N4 = |w| * |w| + |v2| * |v2| + |v0| * |v0| + |v1| * |v1|;
X = |w| - 25/93 * (|v0| + |v1|);
Y = |v2| - 10/31 * (|v0| + |v1|);
Z = |v0| - |v1|;

{
  # Every input is a finite f64.
     |w| <= 1b1024 /\ |v2| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024
  # No factor of a product is zero.
  /\ |w| >= 1b-1074 /\ |v2| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |v1| >= 1b-1074
  /\ |n2| >= 1b-1074 /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074
  # No sum cancels to zero.
  /\ |C2| >= 1b-1074 /\ |C0| >= 1b-1074 /\ |C1| >= 1b-1074 /\ |KK| >= 1b-1074
  /\ |A| >= 1b-1074 /\ |R| >= 1b-1074
  # The nine products are normal, which apply_to_vector does not check.
  /\ |q21| >= 1b-1022 /\ |q22| >= 1b-1022 /\ |q01| >= 1b-1022
  /\ |q02| >= 1b-1022 /\ |q11| >= 1b-1022 /\ |q12| >= 1b-1022
  /\ |k21| >= 1b-1022 /\ |k22| >= 1b-1022 /\ |m0| >= 1b-1022
  # What lemma.g proves each of those nine gives.
  /\ q21 -/ P21 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q22 -/ P22 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q01 -/ P01 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q02 -/ P02 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q11 -/ P11 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q12 -/ P12 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k21 -/ K1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k22 -/ K2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]
  # Every component of the vector is at most one: a constructor's normal is
  # the nearest f64 to a unit vector's components.
  /\ n2 in [-1, 1] /\ n0 in [-1, 1] /\ n1 in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v2 / |v2| in [-1, 1]
  /\ v0 / |v0| in [-1, 1] /\ v1 / |v1| in [-1, 1]
->
  W1 in ? /\ W3 in ? /\ W7 in ? /\ D in ?
  /\ (out - Ex) / D in ?
  /\ out - Ex in [-20b-53, 20b-53]
}

# How far each rounding is from the identity.
q21 / P21 -> 1 + (q21 - P21) / P21;
q22 / P22 -> 1 + (q22 - P22) / P22;
q01 / P01 -> 1 + (q01 - P01) / P01;
q02 / P02 -> 1 + (q02 - P02) / P02;
q11 / P11 -> 1 + (q11 - P11) / P11;
q12 / P12 -> 1 + (q12 - P12) / P12;
c2 / C2 -> 1 + (c2 - C2) / C2;
c0 / C0 -> 1 + (c0 - C0) / C0;
c1 / C1 -> 1 + (c1 - C1) / C1;
k21 / K1 -> 1 + (k21 - K1) / K1;
k22 / K2 -> 1 + (k22 - K2) / K2;
k2 / KK -> 1 + (k2 - KK) / KK;
m0 / M -> 1 + (m0 - M) / M;
a0 / A -> 1 + (a0 - A) / A;
out / R -> 1 + (out - R) / R;

# The error is the sum of the seven shares' errors, each a fraction of its
# own share of D; add_rr then averages them.
out - Ex -> ((out - Ex) / D) * D;
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (d1 + e1) + (d2 + e2) + (d3 + e3) + (d4 + e4)
                + (d5 + e5) + (d6 + e6) + (d7 + e7);
(d1 + e1) - d1 -> e1;
(d2 + e2) - d2 -> e2;
(d3 + e3) - d3 -> e3;
(d4 + e4) - d4 -> e4;
(d5 + e5) - d5 -> e5;
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e1 / d1 -> (w / |w|) * (v0 / |v0|) * n1 * (W1 / 5);
e2 / d2 -> -((w / |w|) * (v1 / |v1|) * n0 * (W2 / 5));
e3 / d3 -> (v0 / |v0|) * (v2 / |v2|) * n0 * (W3 / 6);
e4 / d4 -> -((v0 / |v0|) * (v0 / |v0|) * n2 * (W4 / 6));
e5 / d5 -> -((v1 / |v1|) * (v1 / |v1|) * n2 * (W5 / 6));
e6 / d6 -> (v1 / |v1|) * (v2 / |v2|) * n1 * (W6 / 6);
e7 / d7 -> n2 * W7;

# D is 1 and a quadratic in |w|, |v2|, |v0|, |v1|, and that quadratic is
# 93/5 q.q less four squares: Gappa bounds each square below by zero.
D -> 93/5 * N4 - (93/5 * (X * X) + 93/5 * (Y * Y) + 305/93 * (Z * Z)
                  + 19/465 * (|v0| * |v0| + |v1| * |v1|)) + 1;
