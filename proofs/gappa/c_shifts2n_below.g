# restates: crates/rcad-geom/src/plane.rs Plane::transformed at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::holding at c3b2160e
# restates: crates/rcad-geom/src/half_space.rs HalfSpace::shifted at c3b2160e
# restates: crates/rcad-geom/src/moved_corner.rs SURFACE_AFTER_SHIFTING_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs Isometry::apply_to_vector at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs Isometry::translation_part at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs turned_by at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::named_by at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::turned at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# shiftplane.g where n2 * d2 is below the normals, which
# neither caller checks: n2 * d2 is under 2^-1022 and negative, and the
# other two products are normal.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

X2 = n2 * d2;
t0 = rnd(n0 * d0); t1 = rnd(n1 * d1); t2 = rnd(X2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = off + s012;
value = rnd(y);

Ex = off + n0 * d0 + n1 * d1 + X2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

rp0 = t0 / (n0 * d0); rp1 = t1 / (n1 * d1);
rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;

# Each term as value carries it, less the term itself, with X2 for t2;
# and what rounding X2 cost, as an amount.
W0 = rv * rs012 * rs01 * rp0 - 1;
W1 = rv * rs012 * rs01 * rp1 - 1;
W2 = rv * rs012 - 1;
W3 = rv - 1;
e0 = n0 * d0 * W0; e1 = n1 * d1 * W1; e2 = X2 * W2; e3 = off * W3;
a2 = (t2 - X2) * (rv * rs012);

# Each term's share of the bound. a2 is spent half out of product 0's
# share and half out of product 1's, each 9/8 of |n_i| L; product 2
# meets two roundings, so 1/2 of |n2| L is its share.
c0 = 9b-3 * (|n0| * L); c1 = 9b-3 * (|n1| * L); c2 = 1b-1 * (|n2| * L); c3 = 1b-2 * (N * L);
D = c0 + c1 + c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d0| <= 1b1024 /\ |d1| <= 1b1024 /\ |d2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |d0| >= 1b-1074 /\ |d1| >= 1b-1074 /\ |d2| >= 1b-1074
  # No sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074 /\ |y| >= 1b-1074
  # The case: X2 is negative and below the normals, the other two
  # products are normal. A product of two doubles that are not zero is at
  # least 2^-2148.
  /\ X2 in [-1b-1022, -1b-2148]
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022
  # What lemma.g proves each of those two gives.
  /\ t0 -/ n0 * d0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * d1 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1]
  /\ off / (N * L) in [-1, 1]
  # |n0| + |n1| + |n2| <= sqrt(3) |n| < 1774 / 1024 |n|, by Cauchy-Schwarz.
  /\ (|n0| + |n1| + |n2|) / N in [0, 1774b-10]
  # By Cauchy-Schwarz the same way, 9/8 |n0| + 9/8 |n1| + 1/2 |n2| <=
  # sqrt(178) / 8 |n| < 1708 / 1024 |n|.
  /\ (9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N in [0, 1708b-10]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-61b-56, 61b-56]
}

# How far each rounding is from the identity.
t0 / (n0 * d0) -> 1 + (t0 - n0 * d0) / (n0 * d0);
t1 / (n1 * d1) -> 1 + (t1 - n1 * d1) / (n1 * d1);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors and a2, each a fraction
# of its share of D; add_rr then averages them.
(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N + 1b-2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + (e0 + 1b-1 * a2)) + (c1 + (e1 + 1b-1 * a2)) + (c2 + e2) + (c3 + e3);
(c0 + (e0 + 1b-1 * a2)) - c0 -> e0 + 1b-1 * a2;
(c1 + (e1 + 1b-1 * a2)) - c1 -> e1 + 1b-1 * a2;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(e0 + 1b-1 * a2) / c0 -> 8 / 9 * ((n0 / |n0|) * (d0 / L) * W0) + 4 / 9 * (a2 / (|n0| * L));
(e1 + 1b-1 * a2) / c1 -> 8 / 9 * ((n1 / |n1|) * (d1 / L) * W1) + 4 / 9 * (a2 / (|n1| * L));
e2 / c2 -> 2 * ((n2 / |n2|) * (d2 / L) * W2);
e3 / c3 -> 4 * (off / (N * L)) * W3;
# Product i being normal keeps |n_i| L at least |t_i| / (1 + u), and
# |t_i| at least 2^-1022.
a2 / (|n0| * L) -> (a2 / t0) * rp0 * ((n0 / |n0|) * (d0 / L));
a2 / (|n1| * L) -> (a2 / t1) * rp1 * ((n1 / |n1|) * (d1 / L));
