# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: Cargo.lock nalgebra at ba0a654e
# 464.19's point.g, the moved point's x, with zero factors w, p0, p1,
# cancelling sums none, and so zero P02, P11, P21, P22, C2, K1, M.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), the point and the shift's x, each an f64, a zero
# factor the double 0.
w = 0; v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
p0 = 0; p1 = 0; p2 = rnd(p2_);
s0 = rnd(s0_);

# What apply_to_point computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g).
P01 = v1 * p2; q01 = rnd(P01);
P02 = v2 * p1; q02 = P02;
P11 = v2 * p0; q11 = P11;
P12 = v0 * p2; q12 = rnd(P12);
P21 = v0 * p1; q21 = P21;
P22 = v1 * p0; q22 = P22;
C0 = q01 - q02; c0 = rnd(C0);
C1 = q11 - q12; c1 = rnd(C1);
C2 = q21 - q22; c2 = C2;
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; k01 = K1;
K2 = v2 * t1; k02 = rnd(K2);
KK = k01 - k02; k0 = rnd(KK);
M = t0 * w; m0 = M;
A = m0 + k0; a0 = rnd(A);
R = a0 + p0; r0 = rnd(R);
O = r0 + s0; out = rnd(O);

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * p2 - v2 * p1)
   + v1 * (2 * (v0 * p1 - v1 * p0)) - v2 * (2 * (v2 * p0 - v0 * p2))
   + p0 + s0;

# What each rounding on a live path multiplied its argument by.
rq12 = q12 / P12; rc1 = c1 / C1; rk02 = k02 / K2; rk0 = k0 / KK;
ra = a0 / A; rr = r0 / R; ro = out / O;

# Each term of Ex as out carries it, less the term itself, through the
# roundings left on its path. A term with a zero factor is zero.
W6 = ro * rr * ra * rk0 * rk02 * rc1 * rq12 - 1;
W8 = ro - 1;
e6 = 2 * v2 * v0 * p2 * W6;
e7 = s0 * W8;

# Each live term's share of D, as point.g's, and p0 and s0 sharing 3L.
d6 = 14 * |v2| * |v0| * L;
d7 = 3 * L;
D = d6 + d7;

# q.q and the four squares D / L is written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 30/109 * (|v1| + |v2|);
Y = |v0| - 35/109 * (|v1| + |v2|);
Z = |v1| - |v2|;

{
  # Every input is a finite f64.
     |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024 /\ |p2| <= 1b1024
  /\ |s0| <= 1b1024
  # No factor of a product is zero but w, p0, p1.
  /\ |v0| >= 1b-1074 /\ |v1| >= 1b-1074 /\ |v2| >= 1b-1074 /\ |p2| >= 1b-1074
  # The rounded sums on a live path do not cancel, and each adds two doubles.
  /\ |C1| >= 1b-1074 /\ |KK| >= 1b-1074 /\ |A| >= 1b-1074 /\ |R| >= 1b-1074
  /\ |O| >= 1b-1074
  /\ @FIX(C1, -1074) /\ @FIX(KK, -1074) /\ @FIX(A, -1074) /\ @FIX(R, -1074)
  /\ @FIX(O, -1074)
  # The products on a live path are normal, which apply_to_point does not
  # check, and what lemma.g proves each of them gives.
  /\ |q12| >= 1b-1022 /\ |k02| >= 1b-1022
  /\ q12 -/ P12 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k02 -/ K2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the point and the shift.
  /\ L in [1b-1074, 1b1024]
  /\ p2 / L in [-1, 1] /\ s0 / L in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ v0 / |v0| in [-1, 1] /\ v1 / |v1| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  (out - Ex) / D in ? /\ D / L in ?
  /\ (out - Ex) / L in [-1b-54, 1b-54]
}

# How far each rounding on a live path is from the identity.
q12 / P12 -> 1 + (q12 - P12) / P12;
c1 / C1 -> 1 + (c1 - C1) / C1;
k02 / K2 -> 1 + (k02 - K2) / K2;
k0 / KK -> 1 + (k0 - KK) / KK;
a0 / A -> 1 + (a0 - A) / A;
r0 / R -> 1 + (r0 - R) / R;
out / O -> 1 + (out - O) / O;

# The error is the sum of the live shares' errors, each a fraction of its
# own share of D; add_rr then averages them.
(out - Ex) / L -> ((out - Ex) / D) * (D / L);
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (d6 + e6) + (d7 + e7);
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e6 / d6 -> (v2 / |v2|) * (v0 / |v0|) * (p2 / L) * (W6 / 7);
e7 / d7 -> ((s0 / L) * W8) / 3;

# D / L is point.g's 3 and quadratic, 109/5 q.q less four squares, less
# the share of each term a zero factor took out, and a zero v's square.
D / L -> 109/5 * N4 - (109/5 * (X * X) + 109/5 * (Y * Y) + 425/109 * (Z * Z)
                       + 1/545 * (|v1| * |v1| + |v2| * |v2|)) + 3
        - (12 * |w| * |v1| + 12 * |w| * |v2| + 14 * |v1| * |v0| + 14 * |v1| * |v1| + 14 * |v2| * |v2|);
