# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 zero.
# product 1 zero.
# product 2 below the normals.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p2 = rnd(p2_);
off = rnd(off_);

X2 = n2 * p2;
t2 = rnd(X2);
y = t2 - off;
value = rnd(y);

Ex = X2 - off;

rv = value / y;

W2 = rv - 1;
a2 = (t2 - X2) * (rv);
W3 = rv - 1;
e2 = X2 * W2;
e3 = -(off * W3);

c2 = 1b-2 * (|n2| * L); c3 = 2b-2 * L;
D = c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The products below the normals.
  /\ |X2| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 1/16, and sqrt(1/16 (1 + 64 epsilon)) <= 257 / 1024.
  /\ 2b-3 * |n2| in [0, 257b-10]
  /\ n2 / |n2| in [-1, 1]
->
  a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-3063b-63, 3063b-63]
}

value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 2b-3 * |n2| + 1/2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c2 + e2) + (c3 + (e3 + a2));
(c2 + e2) - c2 -> e2;
(c3 + (e3 + a2)) - c3 -> e3 + a2;
e2 / c2 -> 4 / 1 * ((n2 / |n2|) * (p2 / L) * W2);
(e3 + a2) / c3 -> 4 / 2 * (-((off / L) * W3)) + 4 / 2 * (a2 / L);
