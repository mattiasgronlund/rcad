# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 normal.
# product 1 normal.
# product 2 below the normals.
# Written unrounded, which covers each cancelling to zero: 01, 012.
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
value = rnd(y);

Ex = X0 + X1 + X2 - off;

rp0 = t0 / X0; rp1 = t1 / X1; rv = value / y;

W0 = rv * rp0 - 1;
W1 = rv * rp1 - 1;
W2 = rv - 1;
a2 = (t2 - X2) * (rv);
W3 = rv - 1;
e0 = X0 * W0;
e1 = X1 * W1;
e2 = X2 * W2;
e3 = -(off * W3);

c0 = 3b-2 * (|n0| * L); c1 = 2b-2 * (|n1| * L); c2 = 1b-2 * (|n2| * L); c3 = 1b-2 * L;
D = c0 + c1 + c2 + c3;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |y| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022
  /\ t0 -/ X0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  # The products below the normals.
  /\ |X2| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 7/8, and sqrt(7/8 (1 + 64 epsilon)) <= 958 / 1024.
  /\ 6b-3 * |n0| + 4b-3 * |n1| + 2b-3 * |n2| in [0, 958b-10]
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a2 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

t0 / X0 -> 1 + (t0 - X0) / X0;
t1 / X1 -> 1 + (t1 - X1) / X1;
value / y -> 1 + (value - y) / y;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 6b-3 * |n0| + 4b-3 * |n1| + 2b-3 * |n2| + 1/4;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + (e0 + a2)) + (c1 + e1) + (c2 + e2) + (c3 + e3);
(c0 + (e0 + a2)) - c0 -> e0 + a2;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(e0 + a2) / c0 -> 4 / 3 * ((n0 / |n0|) * (p0 / L) * W0) + 4 / 3 * (a2 / (|n0| * L));
e1 / c1 -> 4 / 2 * ((n1 / |n1|) * (p1 / L) * W1);
e2 / c2 -> 4 / 1 * ((n2 / |n2|) * (p2 / L) * W2);
e3 / c3 -> 4 / 1 * (-((off / L) * W3));
a2 / (|n0| * L) -> (a2 / t0) * rp0 * ((n0 / |n0|) * (p0 / L));
