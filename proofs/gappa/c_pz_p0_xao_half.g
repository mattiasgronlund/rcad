# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: Cargo.lock nalgebra at ba0a654e
# 464.19's point.g, the moved point's x, with zero factors p0,
# cancelling sums A, O, and so zero P11, P22, A, R, O.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), the point and the shift's x, each an f64, a zero
# factor the double 0.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
p0 = 0; p1 = rnd(p1_); p2 = rnd(p2_);
s0 = rnd(s0_);

# What apply_to_point computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g).
P01 = v1 * p2; q01 = rnd(P01);
P02 = v2 * p1; q02 = rnd(P02);
P11 = v2 * p0; q11 = P11;
P12 = v0 * p2; q12 = rnd(P12);
P21 = v0 * p1; q21 = rnd(P21);
P22 = v1 * p0; q22 = P22;
C0 = q01 - q02; c0 = rnd(C0);
C1 = q11 - q12; c1 = rnd(C1);
C2 = q21 - q22; c2 = rnd(C2);
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; k01 = rnd(K1);
K2 = v2 * t1; k02 = rnd(K2);
KK = k01 - k02; k0 = rnd(KK);
M = t0 * w; m0 = rnd(M);
A = m0 + k0; a0 = A;
R = a0 + p0; r0 = R;
O = r0 + s0; out = O;

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * p2 - v2 * p1)
   + v1 * (2 * (v0 * p1 - v1 * p0)) - v2 * (2 * (v2 * p0 - v0 * p2))
   + p0 + s0;

# What each rounding on a live path multiplied its argument by.
rq01 = q01 / P01; rq02 = q02 / P02; rq12 = q12 / P12; rq21 = q21 / P21;
rc0 = c0 / C0; rc1 = c1 / C1; rc2 = c2 / C2; rk01 = k01 / K1;
rk02 = k02 / K2; rk0 = k0 / KK; rm = m0 / M;

# Each term of Ex as out carries it, less the term itself, through the
# roundings left on its path. A term with a zero factor is zero.
W1 = rm * rc0 * rq01 - 1;
W2 = rm * rc0 * rq02 - 1;
W3 = rk0 * rk01 * rc2 * rq21 - 1;
W6 = rk0 * rk02 * rc1 * rq12 - 1;
W8 = 0;
e1 = 2 * w * v1 * p2 * W1;
e2 = -(2 * w * v2 * p1 * W2);
e3 = 2 * v1 * v0 * p1 * W3;
e6 = 2 * v2 * v0 * p2 * W6;
e7 = s0 * W8;

# Each live term's share of D, as point.g's, and p0 and s0 sharing 3L.
d1 = 12 * |w| * |v1| * L;
d2 = 12 * |w| * |v2| * L;
d3 = 14 * |v1| * |v0| * L;
d6 = 14 * |v2| * |v0| * L;
d7 = 3 * L;
D = d1 + d2 + d3 + d6 + d7;

# q.q and the four squares D / L is written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 30/109 * (|v1| + |v2|);
Y = |v0| - 35/109 * (|v1| + |v2|);
Z = |v1| - |v2|;

{
  # Every input is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |s0| <= 1b1024
  # No factor of a product is zero but p0.
  /\ |w| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |v1| >= 1b-1074 /\ |v2| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  # The rounded sums on a live path do not cancel, and each adds two doubles.
  /\ |C0| >= 1b-1074 /\ |C1| >= 1b-1074 /\ |C2| >= 1b-1074 /\ |KK| >= 1b-1074
  /\ @FIX(C0, -1074) /\ @FIX(C1, -1074) /\ @FIX(C2, -1074) /\ @FIX(KK, -1074)
  # The sums that cancel.
  /\ A in [0, 0] /\ O in [0, 0]
  # The products on a live path are normal, which apply_to_point does not
  # check, and what lemma.g proves each of them gives.
  /\ |q01| >= 1b-1022 /\ |q02| >= 1b-1022 /\ |q12| >= 1b-1022
  /\ |q21| >= 1b-1022 /\ |k01| >= 1b-1022 /\ |k02| >= 1b-1022
  /\ |m0| >= 1b-1022
  /\ q01 -/ P01 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q02 -/ P02 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q12 -/ P12 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q21 -/ P21 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k01 -/ K1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k02 -/ K2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the point and the shift.
  /\ L in [1b-1074, 1b1024]
  /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ s0 / L in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v0 / |v0| in [-1, 1] /\ v1 / |v1| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  (out - Ex) / D in ? /\ D / L in ?
  /\ (out - Ex) / L in [-1b-54, 1b-54]
}

# How far each rounding on a live path is from the identity.
q01 / P01 -> 1 + (q01 - P01) / P01;
q02 / P02 -> 1 + (q02 - P02) / P02;
q12 / P12 -> 1 + (q12 - P12) / P12;
q21 / P21 -> 1 + (q21 - P21) / P21;
c0 / C0 -> 1 + (c0 - C0) / C0;
c1 / C1 -> 1 + (c1 - C1) / C1;
c2 / C2 -> 1 + (c2 - C2) / C2;
k01 / K1 -> 1 + (k01 - K1) / K1;
k02 / K2 -> 1 + (k02 - K2) / K2;
k0 / KK -> 1 + (k0 - KK) / KK;
m0 / M -> 1 + (m0 - M) / M;

# The error is the sum of the live shares' errors, each a fraction of its
# own share of D; add_rr then averages them.
(out - Ex) / L -> ((out - Ex) / D) * (D / L);
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (d1 + e1) + (d2 + e2) + (d3 + e3) + (d6 + e6) + (d7 + e7);
(d1 + e1) - d1 -> e1;
(d2 + e2) - d2 -> e2;
(d3 + e3) - d3 -> e3;
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e1 / d1 -> (w / |w|) * (v1 / |v1|) * (p2 / L) * (W1 / 6);
e2 / d2 -> -((w / |w|) * (v2 / |v2|) * (p1 / L) * (W2 / 6));
e3 / d3 -> (v1 / |v1|) * (v0 / |v0|) * (p1 / L) * (W3 / 7);
e6 / d6 -> (v2 / |v2|) * (v0 / |v0|) * (p2 / L) * (W6 / 7);
e7 / d7 -> ((s0 / L) * W8) / 3;

# D / L is point.g's 3 and quadratic, 109/5 q.q less four squares, less
# the share of each term a zero factor took out, and a zero v's square.
D / L -> 109/5 * N4 - (109/5 * (X * X) + 109/5 * (Y * Y) + 425/109 * (Z * Z)
                       + 1/545 * (|v1| * |v1| + |v2| * |v2|)) + 3
        - (14 * |v1| * |v1| + 14 * |v2| * |v2|);
