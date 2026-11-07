# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 normal.
# product 1 below the normals.
# product 2 normal.
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

rp0 = t0 / X0; rp2 = t2 / X2;

W0 = rp0 - 1;
a1 = (t1 - X1);
W2 = rp2 - 1;
e0 = X0 * W0;
e2 = X2 * W2;

c0 = 2b-2 * (|n0| * L); c2 = 1b-2 * (|n2| * L);
D = c0 + c2;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t0| >= 1b-1022 /\ |t2| >= 1b-1022
  /\ t0 -/ X0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ X2 in [-4503599627370497b-105, 4503599627370497b-105]
  # The products below the normals.
  /\ |X1| <= 1b-1022
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 5/16, and sqrt(5/16 (1 + 64 epsilon)) <= 573 / 1024.
  /\ 4b-3 * |n0| + 2b-3 * |n2| in [0, 573b-10]
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  a1 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

t0 / X0 -> 1 + (t0 - X0) / X0;
t2 / X2 -> 1 + (t2 - X2) / X2;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 4b-3 * |n0| + 2b-3 * |n2| + 0;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + (e0 + a1)) + (c2 + e2);
(c0 + (e0 + a1)) - c0 -> e0 + a1;
(c2 + e2) - c2 -> e2;
(e0 + a1) / c0 -> 4 / 2 * ((n0 / |n0|) * (p0 / L) * W0) + 4 / 2 * (a1 / (|n0| * L));
e2 / c2 -> 4 / 1 * ((n2 / |n2|) * (p2 / L) * W2);
a1 / (|n0| * L) -> (a1 / t0) * rp0 * ((n0 / |n0|) * (p0 / L));
