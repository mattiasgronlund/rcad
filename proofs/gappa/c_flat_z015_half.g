# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at 8c943dfd
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at 8c943dfd
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs decide at 8c943dfd
# SideOfAFlatSurfaceThroughAPoint::on_floats and its error bound: the sign
# of facing . point - facing . through.
# The case where products 0, 1, 5 are zero.
@rnd = float<ieee_64, ne>;

# The facing, the point and the through point, each an f64. A product that
# is zero has the double 0 for its point's or its through point's
# coordinate; where the facing's coordinate is the zero, both its products
# are zero, and every number below is the same as with both of those
# coordinates zero.
f0 = rnd(f0_); f1 = rnd(f1_); f2 = rnd(f2_);
p0 = 0; p1 = 0; p2 = rnd(p2_);
q0 = rnd(q0_); q1 = rnd(q1_); q2 = 0;

# What on_floats computes, line for line. The fold starts at 0.0, and
# adding 0.0 or a zero product to a double gives that double back
# (addzero.g), so the zero products and the first addition are left out.
t2 = rnd(f2 * p2); t3 = rnd(f0 * q0); t4 = rnd(f1 * q1);
S3 = t2 - t3;
s3 = rnd(S3);
y = s3 - t4;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(|t2| + |t3|) + |t4|);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = f0 * p0 + f1 * p1 + f2 * p2 - f0 * q0 - f1 * q1 - f2 * q2;

# The permanent with no rounding of its own.
Phat = |t2| + |t3| + |t4|;

# What each rounding multiplied its argument by.
rp2 = t2 / (f2 * p2); rp3 = t3 / (f0 * q0); rp4 = t4 / (f1 * q1);
rs3 = s3 / S3;

# Each product's term, as y carries it, less the term itself: e is
# that amount and W the same over the product as on_floats rounds it.
# The through point's terms are subtracted, so their e is the other way
# round.
W2 = rs3 - 1 / rp2;
W3 = rs3 - 1 / rp3;
W4 = 1 - 1 / rp4;
e2 = t2 * W2; e3 = -(t3 * W3); e4 = -(t4 * W4);
c2 = |t2|; c3 = |t3|; c4 = |t4|;

{
  # Every input is a finite f64.
     |f0| <= 1b1024 /\ |f1| <= 1b1024 /\ |f2| <= 1b1024
  /\ |p2| <= 1b1024 /\ |q0| <= 1b1024 /\ |q1| <= 1b1024
  # No factor of a product left is zero. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |f0| >= 1b-1074 /\ |f1| >= 1b-1074 /\ |f2| >= 1b-1074
  /\ |p2| >= 1b-1074 /\ |q0| >= 1b-1074 /\ |q1| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S3| >= 1b-1074
  # product_is_normal on the products left.
  /\ |t2| >= 1b-1022 /\ |t3| >= 1b-1022 /\ |t4| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t2 -/ f2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ f0 * q0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t4 -/ f1 * q1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t2 / |t2| in [-1, 1] /\ t3 / |t3| in [-1, 1] /\ t4 / |t4| in [-1, 1]
->
  (y - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t2 / (f2 * p2) -> 1 + (t2 - f2 * p2) / (f2 * p2);
t3 / (f0 * q0) -> 1 + (t3 - f0 * q0) / (f0 * q0);
t4 / (f1 * q1) -> 1 + (t4 - f1 * q1) / (f1 * q1);
s3 / S3 -> 1 + (s3 - S3) / S3;

# The error is the sum of the products' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / Phat -> ((Phat + (y - Ex)) - Phat) / Phat;
Phat + (y - Ex) -> (c2 + e2) + (c3 + e3) + (c4 + e4);
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(c4 + e4) - c4 -> e4;
e2 / c2 -> (t2 / |t2|) * W2;
e3 / c3 -> -((t3 / |t3|) * W3);
e4 / c4 -> -((t4 / |t4|) * W4);

# From the permanent to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
