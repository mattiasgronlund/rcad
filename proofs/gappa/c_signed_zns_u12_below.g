# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 zero.
# product 1 normal.
# product 2 below the normals.
# Written unrounded, which covers each cancelling to zero: 12.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

X1 = n1 * p1; X2 = n2 * p2;
t1 = rnd(X1); t2 = rnd(X2);
S12 = t1 + t2;
s12 = S12;
y = s12 - off;
value = rnd(y);

Ex = X1 + X2 - off;

rp1 = t1 / X1; rv = value / y;

W1 = rv * rp1 - 1;
W2 = rv - 1;
a2 = (t2 - X2) * (rv);
W3 = rv - 1;
e1 = X1 * W1;
e2 = X2 * W2;
e3 = -(off * W3);

c1 = 3b-2 * (|n1| * L); c2 = 1b-2 * (|n2| * L); c3 = 1b-2 * L;
D = c1 + c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t1| >= 1b-1022
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  # The products below the normals.
  /\ |X2| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 5/8, and sqrt(5/8 (1 + 64 epsilon)) <= 810 / 1024.
  /\ 6b-3 * |n1| + 2b-3 * |n2| in [0, 810b-10]
  /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-4247b-63, 4247b-63]
}

t1 / X1 -> 1 + (t1 - X1) / X1;
value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 6b-3 * |n1| + 2b-3 * |n2| + 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c1 + (e1 + a2)) + (c2 + e2) + (c3 + e3);
(c1 + (e1 + a2)) - c1 -> e1 + a2;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(e1 + a2) / c1 -> 4 / 3 * ((n1 / |n1|) * (p1 / L) * W1) + 4 / 3 * (a2 / (|n1| * L));
e2 / c2 -> 4 / 1 * ((n2 / |n2|) * (p2 / L) * W2);
e3 / c3 -> 4 / 1 * (-((off / L) * W3));
a2 / (|n1| * L) -> (a2 / t1) * rp1 * ((n1 / |n1|) * (p1 / L));
