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
# product 1 zero.
# product 2 zero.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_);
off = rnd(off_);

X0 = n0 * d0;
t0 = rnd(X0);
y = off + t0;
value = rnd(y);

Ex = off + X0;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

rv = value / y;

W0 = rv - 1;
a0 = (t0 - X0) * (rv);
W3 = rv - 1;
e0 = X0 * W0;
e3 = off * W3;

c0 = 1b-2 * (|n0| * L); c3 = 2b-2 * (N * L);
D = c0 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d0| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |d0| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The products below the normals.
  /\ |X0| <= 1b-1022
  # |n| L is at least 2^-1022, L at least every coordinate of the shift
  # and the offset over |n|.
  /\ L in [1b-1074, 1b1024] /\ N * L in [1b-1022, 1b1030]
  /\ d0 / L in [-1, 1] /\ off / (N * L) in [-1, 1]
  # Cauchy-Schwarz: the weights' squares add to 1/16, and sqrt(1/16) <= 256 / 1024.
  /\ (2b-3 * |n0|) / N in [0, 256b-10]
  /\ n0 / |n0| in [-1, 1]
->
  a0 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-1b-49, 1b-49]
}

value / y -> 1 + (value - y) / y;

(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (2b-3 * |n0|) / N + 1/2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c3 + (e3 + a0));
(c0 + e0) - c0 -> e0;
(c3 + (e3 + a0)) - c3 -> e3 + a0;
e0 / c0 -> 4 / 1 * ((n0 / |n0|) * (d0 / L) * W0);
(e3 + a0) / c3 -> 4 / 2 * ((off / (N * L)) * W3) + 4 / 2 * (a0 / (N * L));
