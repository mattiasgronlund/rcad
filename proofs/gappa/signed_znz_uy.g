# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to, normal.dot(p) - offset, in the case:
# product 0 zero.
# product 1 normal.
# product 2 zero.
# Written unrounded, which covers each cancelling to zero: y.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p1 = rnd(p1_);
off = rnd(off_);

X1 = n1 * p1;
t1 = rnd(X1);
y = t1 - off;
value = y;

Ex = X1 - off;

rp1 = t1 / X1;

W1 = rp1 - 1;
e1 = X1 * W1;

c1 = 1b-2 * (|n1| * L);
D = c1;

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p1| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a product left is zero.
  /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074
  # The normal products, and what lemma.g proves each gives.
  /\ |t1| >= 1b-1022
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least 2^-1022, and at least every coordinate and the offset.
  /\ L in [1b-1022, 1b1024]
  /\ p1 / L in [-1, 1] /\ off / L in [-1, 1]
  # Cauchy-Schwarz on a normal whose n.n is within 64 epsilon of one:
  # the weights' squares add to 1/16, and sqrt(1/16 (1 + 64 epsilon)) <= 257 / 1024.
  /\ 2b-3 * |n1| in [0, 257b-10]
  /\ n1 / |n1| in [-1, 1]
->
  (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

t1 / X1 -> 1 + (t1 - X1) / X1;

(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> 2b-3 * |n1| + 0;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c1 + e1);
(c1 + e1) - c1 -> e1;
e1 / c1 -> 4 / 1 * ((n1 / |n1|) * (p1 / L) * W1);
