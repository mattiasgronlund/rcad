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
# product 1 below the normals.
# product 2 normal.
# Written unrounded, which covers each cancelling to zero: y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

X1 = n1 * d1; X2 = n2 * d2;
t1 = rnd(X1); t2 = rnd(X2);
S12 = t1 + t2;
s12 = rnd(S12);
y = off + s12;
value = y;

Ex = off + X1 + X2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

rp2 = t2 / X2; rs12 = s12 / S12;

W1 = rs12 - 1;
a1 = (t1 - X1) * (rs12);
W2 = rs12 * rp2 - 1;
e1 = X1 * W1;
e2 = X2 * W2;

c1 = 1b-2 * (|n1| * L); c2 = 3b-2 * (|n2| * L);
D = c1 + c2;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d1| <= 1b1024 /\ |d2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n1| >= 1b-1074 /\ |d1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |d2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S12| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t2| >= 1b-1022
  /\ t2 -/ X2 in [-4503599627370497b-105, 4503599627370497b-105]
  # The products below the normals.
  /\ |X1| <= 1b-1022
  # |n| L is at least 2^-1022, L at least every coordinate of the shift
  # and the offset over |n|.
  /\ L in [1b-1074, 1b1024] /\ N * L in [1b-1022, 1b1030]
  /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1] /\ off / (N * L) in [-1, 1]
  # Cauchy-Schwarz: the weights' squares add to 5/8, and sqrt(5/8) <= 810 / 1024.
  /\ (2b-3 * |n1| + 6b-3 * |n2|) / N in [0, 810b-10]
  /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a1 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-3227b-63, 3227b-63]
}

t2 / X2 -> 1 + (t2 - X2) / X2;
s12 / S12 -> 1 + (s12 - S12) / S12;

(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (2b-3 * |n1| + 6b-3 * |n2|) / N + 0;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c1 + e1) + (c2 + (e2 + a1));
(c1 + e1) - c1 -> e1;
(c2 + (e2 + a1)) - c2 -> e2 + a1;
e1 / c1 -> 4 / 1 * ((n1 / |n1|) * (d1 / L) * W1);
(e2 + a1) / c2 -> 4 / 3 * ((n2 / |n2|) * (d2 / L) * W2) + 4 / 3 * (a1 / (|n2| * L));
a1 / (|n2| * L) -> (a1 / t2) * rp2 * ((n2 / |n2|) * (d2 / L));
