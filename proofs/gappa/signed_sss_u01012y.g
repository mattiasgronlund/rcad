# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 below the normals.
# product 1 below the normals.
# product 2 below the normals.
# Written unrounded, which covers each cancelling to zero: 01, 012, y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

X0 = n0 * p0; X1 = n1 * p1; X2 = n2 * p2;
t0 = rnd(X0); t1 = rnd(X1); t2 = rnd(X2);
S01 = t0 + t1;
s01 = S01;
S012 = s01 + t2;
s012 = S012;
y = s012 - off;
value = y;

Ex = X0 + X1 + X2 - off;


a0 = (t0 - X0);
a1 = (t1 - X1);
a2 = (t2 - X2);

c3 = 3b-2 * L;
D = c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # The products below the normals.
  /\ |X0| <= 1b-1022 /\ |X1| <= 1b-1022 /\ |X2| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Every product is below the normals and every operation exact: the
  # amounts are all D holds.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a0 in ? /\ a1 in ? /\ a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}


(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 3/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c3 + (a0 + a1 + a2));
(c3 + (a0 + a1 + a2)) - c3 -> a0 + a1 + a2;
(a0 + a1 + a2) / c3 -> 4 / 3 * (a0 / L) + 4 / 3 * (a1 / L) + 4 / 3 * (a2 / L);
