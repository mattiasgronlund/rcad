# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 below the normals.
# product 1 zero.
# product 2 zero.
# Written unrounded, which covers each cancelling to zero: y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_);
off = rnd(off_);

X0 = n0 * p0;
t0 = rnd(X0);
y = t0 - off;
value = y;

Ex = X0 - off;


a0 = (t0 - X0);

c3 = 1b-2 * L;
D = c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074
  # The products below the normals.
  /\ |X0| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p0 / L in [-1, 1] /\ off / L in [-1, 1]
  # Every product is below the normals and every operation exact: the
  # amounts are all D holds.
  /\ n0 / |n0| in [-1, 1]
->
  a0 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}


(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c3 + a0);
(c3 + a0) - c3 -> a0;
a0 / c3 -> 4 / 1 * (a0 / L);
