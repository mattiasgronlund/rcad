# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 normal.
# product 1 zero.
# product 2 normal.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p2 = rnd(p2_);
off = rnd(off_);

X0 = n0 * p0; X2 = n2 * p2;
t0 = rnd(X0); t2 = rnd(X2);
S02 = t0 + t2;
s02 = rnd(S02);
y = s02 - off;
value = rnd(y);

Ex = X0 + X2 - off;

rp0 = t0 / X0; rp2 = t2 / X2; rs02 = s02 / S02; rv = value / y;

W0 = rs02 * rv * rp0 - 1;
W2 = rs02 * rv * rp2 - 1;
W3 = rv - 1;
e0 = X0 * W0;
e2 = X2 * W2;
e3 = -(off * W3);

c0 = 3b-2 * (|n0| * L); c2 = 3b-2 * (|n2| * L); c3 = 1b-2 * L;
D = c0 + c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S02| >= 1b-1074 /\ |y| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t0| >= 1b-1022 /\ |t2| >= 1b-1022
  /\ t0 -/ X0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ X2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p0 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 9/8, and sqrt(9/8 (1 + 64 epsilon)) <= 1087 / 1024.
  /\ 6b-3 * |n0| + 6b-3 * |n2| in [0, 1087b-10]
  /\ n0 / |n0| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

t0 / X0 -> 1 + (t0 - X0) / X0;
t2 / X2 -> 1 + (t2 - X2) / X2;
s02 / S02 -> 1 + (s02 - S02) / S02;
value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 6b-3 * |n0| + 6b-3 * |n2| + 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c2 + e2) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e0 / c0 -> 4 / 3 * ((n0 / |n0|) * (p0 / L) * W0);
e2 / c2 -> 4 / 3 * ((n2 / |n2|) * (p2 / L) * W2);
e3 / c3 -> 4 / 1 * (-((off / L) * W3));
