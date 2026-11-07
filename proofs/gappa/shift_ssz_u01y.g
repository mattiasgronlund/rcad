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
# product 0 below the normals.
# product 1 below the normals.
# product 2 zero.
# Written unrounded, which covers each cancelling to zero: 01, y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_); d1 = rnd(d1_);
off = rnd(off_);

X0 = n0 * d0; X1 = n1 * d1;
t0 = rnd(X0); t1 = rnd(X1);
S01 = t0 + t1;
s01 = S01;
y = off + s01;
value = y;

Ex = off + X0 + X1;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);


a0 = (t0 - X0);
a1 = (t1 - X1);

c3 = 2b-2 * (N * L);
D = c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d0| <= 1b1024 /\ |d1| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |d0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |d1| >= 1b-1074
  # The products below the normals.
  /\ |X0| <= 1b-1022 /\ |X1| <= 1b-1022
  # |n| L is at least 2^-1022, L at least every coordinate of the shift
  # and the offset over |n|.
  /\ L in [1b-1074, 1b1024] /\ N * L in [1b-1022, 1b1030]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ off / (N * L) in [-1, 1]
  # Every product is below the normals and every operation exact: the
  # amounts are all D holds.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1]
->
  a0 in ? /\ a1 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-1b-49, 1b-49]
}


(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> 1/2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c3 + (a0 + a1));
(c3 + (a0 + a1)) - c3 -> a0 + a1;
(a0 + a1) / c3 -> 4 / 2 * (a0 / (N * L)) + 4 / 2 * (a1 / (N * L));
