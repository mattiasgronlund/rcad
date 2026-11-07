# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 zero.
# product 1 zero.
# product 2 normal.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p2 = rnd(p2_);
off = rnd(off_);

X2 = n2 * p2;
t2 = rnd(X2);
y = t2 - off;
value = rnd(y);

Ex = X2 - off;

rp2 = t2 / X2; rv = value / y;

W2 = rv * rp2 - 1;
W3 = rv - 1;
e2 = X2 * W2;
e3 = -(off * W3);

c2 = 2b-2 * (|n2| * L); c3 = 1b-2 * L;
D = c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t2| >= 1b-1022
  /\ t2 -/ X2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 1/4, and sqrt(1/4 (1 + 64 epsilon)) <= 513 / 1024.
  /\ 4b-3 * |n2| in [0, 513b-10]
  /\ n2 / |n2| in [-1, 1]
->
  (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

t2 / X2 -> 1 + (t2 - X2) / X2;
value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 4b-3 * |n2| + 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c2 + e2) + (c3 + e3);
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e2 / c2 -> 4 / 2 * ((n2 / |n2|) * (p2 / L) * W2);
e3 / c3 -> 4 / 1 * (-((off / L) * W3));
