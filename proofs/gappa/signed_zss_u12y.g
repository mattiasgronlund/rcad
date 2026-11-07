# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 zero.
# product 1 below the normals.
# product 2 below the normals.
# Written unrounded, which covers each cancelling to zero: 12, y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

X1 = n1 * p1; X2 = n2 * p2;
t1 = rnd(X1); t2 = rnd(X2);
S12 = t1 + t2;
s12 = S12;
y = s12 - off;
value = y;

Ex = X1 + X2 - off;


a1 = (t1 - X1);
a2 = (t2 - X2);

c3 = 2b-2 * L;
D = c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # The products below the normals.
  /\ |X1| <= 1b-1022 /\ |X2| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Every product is below the normals and every operation exact: the
  # amounts are all D holds.
  /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a1 in ? /\ a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}


(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 1/2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c3 + (a1 + a2));
(c3 + (a1 + a2)) - c3 -> a1 + a2;
(a1 + a2) / c3 -> 4 / 2 * (a1 / L) + 4 / 2 * (a2 / L);
