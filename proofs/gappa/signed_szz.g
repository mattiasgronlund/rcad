# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 below the normals.
# product 1 zero.
# product 2 zero.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_);
off = rnd(off_);

X0 = n0 * p0;
t0 = rnd(X0);
y = t0 - off;
value = rnd(y);

Ex = X0 - off;

rv = value / y;

W0 = rv - 1;
a0 = (t0 - X0) * (rv);
W3 = rv - 1;
e0 = X0 * W0;
e3 = -(off * W3);

c0 = 1b-2 * (|n0| * L); c3 = 2b-2 * L;
D = c0 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The products below the normals.
  /\ |X0| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p0 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 1/16, and sqrt(1/16 (1 + 64 epsilon)) <= 257 / 1024.
  /\ 2b-3 * |n0| in [0, 257b-10]
  /\ n0 / |n0| in [-1, 1]
->
  a0 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 2b-3 * |n0| + 1/2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c3 + (e3 + a0));
(c0 + e0) - c0 -> e0;
(c3 + (e3 + a0)) - c3 -> e3 + a0;
e0 / c0 -> 4 / 1 * ((n0 / |n0|) * (p0 / L) * W0);
(e3 + a0) / c3 -> 4 / 2 * (-((off / L) * W3)) + 4 / 2 * (a0 / L);
