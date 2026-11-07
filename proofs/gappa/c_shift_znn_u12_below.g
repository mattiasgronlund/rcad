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
# product 0 zero.
# product 1 normal.
# product 2 normal.
# Written unrounded, which covers each cancelling to zero: 12.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

X1 = n1 * d1; X2 = n2 * d2;
t1 = rnd(X1); t2 = rnd(X2);
S12 = t1 + t2;
s12 = S12;
y = off + s12;
value = rnd(y);

Ex = off + X1 + X2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

rp1 = t1 / X1; rp2 = t2 / X2; rv = value / y;

W1 = rv * rp1 - 1;
W2 = rv * rp2 - 1;
W3 = rv - 1;
e1 = X1 * W1;
e2 = X2 * W2;
e3 = off * W3;

c1 = 2b-2 * (|n1| * L); c2 = 2b-2 * (|n2| * L); c3 = 1b-2 * (N * L);
D = c1 + c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d1| <= 1b1024 /\ |d2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n1| >= 1b-1074 /\ |d1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |d2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ X2 in [-4503599627370497b-105, 4503599627370497b-105]
  # |n| L is at least 2^-1022, L at least every coordinate of the shift
  # and the offset over |n|.
  /\ L in [1b-1074, 1b1024] /\ N * L in [1b-1022, 1b1030]
  /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1] /\ off / (N * L) in [-1, 1]
  # Cauchy-Schwarz: the weights' squares add to 1/2, and sqrt(1/2) <= 725 / 1024.
  /\ (4b-3 * |n1| + 4b-3 * |n2|) / N in [0, 725b-10]
  /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-3908b-63, 3908b-63]
}

t1 / X1 -> 1 + (t1 - X1) / X1;
t2 / X2 -> 1 + (t2 - X2) / X2;
value / y -> 1 + (value - y) / y;

(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (4b-3 * |n1| + 4b-3 * |n2|) / N + 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c1 + e1) + (c2 + e2) + (c3 + e3);
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e1 / c1 -> 4 / 2 * ((n1 / |n1|) * (d1 / L) * W1);
e2 / c2 -> 4 / 2 * ((n2 / |n2|) * (d2 / L) * W2);
e3 / c3 -> 4 / 1 * ((off / (N * L)) * W3);
