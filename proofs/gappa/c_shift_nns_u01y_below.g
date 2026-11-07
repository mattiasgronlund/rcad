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
# a plane's or a half-space's offset after a shift, offset + normal.dot(d), in the case:
# product 0 normal.
# product 1 normal.
# product 2 below the normals.
# Written unrounded, which covers each cancelling to zero: 01, y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

X0 = n0 * d0; X1 = n1 * d1; X2 = n2 * d2;
t0 = rnd(X0); t1 = rnd(X1); t2 = rnd(X2);
S01 = t0 + t1;
s01 = S01;
S012 = s01 + t2;
s012 = rnd(S012);
y = off + s012;
value = y;

Ex = off + X0 + X1 + X2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

rp0 = t0 / X0; rp1 = t1 / X1; rs012 = s012 / S012;

W0 = rs012 * rp0 - 1;
W1 = rs012 * rp1 - 1;
W2 = rs012 - 1;
a2 = (t2 - X2) * (rs012);
e0 = X0 * W0;
e1 = X1 * W1;
e2 = X2 * W2;

c0 = 3b-2 * (|n0| * L); c1 = 2b-2 * (|n1| * L); c2 = 1b-2 * (|n2| * L);
D = c0 + c1 + c2;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d0| <= 1b1024 /\ |d1| <= 1b1024 /\ |d2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |d0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |d1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |d2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S012| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022
  /\ t0 -/ X0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  # The products below the normals.
  /\ |X2| <= 1b-1022
  # |n| L is at least 2^-1022, L at least every coordinate of the shift
  # and the offset over |n|.
  /\ L in [1b-1074, 1b1024] /\ N * L in [1b-1022, 1b1030]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1] /\ off / (N * L) in [-1, 1]
  # Cauchy-Schwarz: the weights' squares add to 7/8, and sqrt(7/8) <= 958 / 1024.
  /\ (6b-3 * |n0| + 4b-3 * |n1| + 2b-3 * |n2|) / N in [0, 958b-10]
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-3817b-63, 3817b-63]
}

t0 / X0 -> 1 + (t0 - X0) / X0;
t1 / X1 -> 1 + (t1 - X1) / X1;
s012 / S012 -> 1 + (s012 - S012) / S012;

(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (6b-3 * |n0| + 4b-3 * |n1| + 2b-3 * |n2|) / N + 0;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + (e0 + a2)) + (c1 + e1) + (c2 + e2);
(c0 + (e0 + a2)) - c0 -> e0 + a2;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(e0 + a2) / c0 -> 4 / 3 * ((n0 / |n0|) * (d0 / L) * W0) + 4 / 3 * (a2 / (|n0| * L));
e1 / c1 -> 4 / 2 * ((n1 / |n1|) * (d1 / L) * W1);
e2 / c2 -> 4 / 1 * ((n2 / |n2|) * (d2 / L) * W2);
a2 / (|n0| * L) -> (a2 / t0) * rp0 * ((n0 / |n0|) * (d0 / L));
