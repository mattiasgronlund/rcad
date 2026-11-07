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
# shiftplane.g where n1 * d1 is below the normals, which
# neither caller checks: n1 * d1 is under 2^-1022 and positive, and the
# other two products are normal.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

X1 = n1 * d1;
t0 = rnd(n0 * d0); t1 = rnd(X1); t2 = rnd(n2 * d2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = off + s012;
value = rnd(y);

Ex = off + n0 * d0 + X1 + n2 * d2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

rp0 = t0 / (n0 * d0); rp2 = t2 / (n2 * d2);
rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;

# Each term as value carries it, less the term itself, with X1 for t1;
# and what rounding X1 cost, as an amount.
W0 = rv * rs012 * rs01 * rp0 - 1;
W1 = rv * rs012 * rs01 - 1;
W2 = rv * rs012 * rp2 - 1;
W3 = rv - 1;
e0 = n0 * d0 * W0; e1 = X1 * W1; e2 = n2 * d2 * W2; e3 = off * W3;
a1 = (t1 - X1) * (rv * rs012 * rs01);

# Each term's share of the bound, as in shiftplane.g. a1 is spent out of
# product 2's share, since product 2 meets one rounding fewer.
c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; c3 = 1b-2 * (N * L);
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
  # The case: X1 is positive and below the normals, the other two
  # products are normal. A product of two doubles that are not zero is at
  # least 2^-2148.
  /\ X1 in [1b-2148, 1b-1022]
  /\ |t0| >= 1b-1022 /\ |t2| >= 1b-1022
  # What lemma.g proves each of those two gives.
  /\ t0 -/ n0 * d0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * d2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1]
  /\ off / (N * L) in [-1, 1]
  # |n0| + |n1| + |n2| <= sqrt(3) |n| < 1774 / 1024 |n|, by Cauchy-Schwarz.
  /\ (|n0| + |n1| + |n2|) / N in [0, 1774b-10]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a1 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-1b-49, 1b-49]
}

# How far each rounding is from the identity.
t0 / (n0 * d0) -> 1 + (t0 - n0 * d0) / (n0 * d0);
t2 / (n2 * d2) -> 1 + (t2 - n2 * d2) / (n2 * d2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors and a1, each a fraction
# of its share of D; add_rr then averages them.
(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (|n0| + |n1| + |n2|) / N + 1b-2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + (e2 + a1)) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + (e2 + a1)) - c2 -> e2 + a1;
(c3 + e3) - c3 -> e3;
e0 / c0 -> (n0 / |n0|) * (d0 / L) * W0;
e1 / c1 -> (n1 / |n1|) * (d1 / L) * W1;
(e2 + a1) / c2 -> (n2 / |n2|) * (d2 / L) * W2 + a1 / (|n2| * L);
e3 / c3 -> 4 * (off / (N * L)) * W3;
# Product 2 being normal keeps |n2| L at least |t2| / (1 + u), and |t2|
# at least 2^-1022.
a1 / (|n2| * L) -> (a1 / t2) * rp2 * ((n2 / |n2|) * (d2 / L));
