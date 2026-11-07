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
# product 1 zero.
# product 2 below the normals.
# Written unrounded, which covers each cancelling to zero: y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d2 = rnd(d2_);
off = rnd(off_);

X2 = n2 * d2;
t2 = rnd(X2);
y = off + t2;
value = y;

Ex = off + X2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);


a2 = (t2 - X2);

c3 = 1b-2 * (N * L);
D = c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n2| >= 1b-1074 /\ |d2| >= 1b-1074
  # The products below the normals.
  /\ |X2| <= 1b-1022
  # |n| L is at least 2^-1022, L at least every coordinate of the shift
  # and the offset over |n|.
  /\ L in [1b-1074, 1b1024] /\ N * L in [1b-1022, 1b1030]
  /\ d2 / L in [-1, 1] /\ off / (N * L) in [-1, 1]
  # Every product is below the normals and every operation exact: the
  # amounts are all D holds.
  /\ n2 / |n2| in [-1, 1]
->
  a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-1b-49, 1b-49]
}


(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c3 + a2);
(c3 + a2) - c3 -> a2;
a2 / c3 -> 4 / 1 * (a2 / (N * L));
