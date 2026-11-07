# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 below the normals.
# product 1 below the normals.
# product 2 zero.
# Written unrounded, which covers each cancelling to zero: 01.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_);
off = rnd(off_);

X0 = n0 * p0; X1 = n1 * p1;
t0 = rnd(X0); t1 = rnd(X1);
S01 = t0 + t1;
s01 = S01;
y = s01 - off;
value = rnd(y);

Ex = X0 + X1 - off;

rv = value / y;

W0 = rv - 1;
a0 = (t0 - X0) * (rv);
W1 = rv - 1;
a1 = (t1 - X1) * (rv);
W3 = rv - 1;
e0 = X0 * W0;
e1 = X1 * W1;
e3 = -(off * W3);

c0 = 1b-2 * (|n0| * L); c1 = 1b-2 * (|n1| * L); c3 = 3b-2 * L;
D = c0 + c1 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The products below the normals.
  /\ |X0| <= 1b-1022 /\ |X1| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1074, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 1/8, and sqrt(1/8 (1 + 64 epsilon)) <= 363 / 1024.
  /\ 2b-3 * |n0| + 2b-3 * |n1| in [0, 363b-10]
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1]
->
  a0 in ? /\ a1 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 2b-3 * |n0| + 2b-3 * |n1| + 3/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c3 + (e3 + a0 + a1));
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c3 + (e3 + a0 + a1)) - c3 -> e3 + a0 + a1;
e0 / c0 -> 4 / 1 * ((n0 / |n0|) * (p0 / L) * W0);
e1 / c1 -> 4 / 1 * ((n1 / |n1|) * (p1 / L) * W1);
(e3 + a0 + a1) / c3 -> 4 / 3 * (-((off / L) * W3)) + 4 / 3 * (a0 / L) + 4 / 3 * (a1 / L);
