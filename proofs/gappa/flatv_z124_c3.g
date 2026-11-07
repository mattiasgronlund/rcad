# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at 8c943dfd
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at 8c943dfd
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs decide at 8c943dfd
# SideOfAFlatSurfaceThroughAPoint::on_floats and its error bound: the sign
# of facing . point - facing . through.
# The claim is on value, the double on_floats returns, not on y.
# The case where products 1, 2, 4 are zero.
# The sum adding product 3 may cancel to zero, which rounds to itself.
@rnd = float<ieee_64, ne>;

# The facing, the point and the through point, each an f64. A product that
# is zero has the double 0 for its point's or its through point's
# coordinate; where the facing's coordinate is the zero, both its products
# are zero, and every number below is the same as with both of those
# coordinates zero.
f0 = rnd(f0_); f1 = rnd(f1_); f2 = rnd(f2_);
p0 = rnd(p0_); p1 = 0; p2 = 0;
q0 = rnd(q0_); q1 = 0; q2 = rnd(q2_);

# What on_floats computes, line for line. The fold starts at 0.0, and
# adding 0.0 or a zero product to a double gives that double back
# (addzero.g), so the zero products and the first addition are left out.
t0 = rnd(f0 * p0); t3 = rnd(f0 * q0); t5 = rnd(f2 * q2);
S3 = t0 - t3;
s3 = S3;
y = s3 - t5;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(|t0| + |t3|) + |t5|);

B = 6b-53 + 1b-99;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = f0 * p0 + f1 * p1 + f2 * p2 - f0 * q0 - f1 * q1 - f2 * q2;

# The permanent with no rounding of its own.
Phat = |t0| + |t3| + |t5|;

# What each rounding multiplied its argument by.
rp0 = t0 / (f0 * p0); rp3 = t3 / (f0 * q0); rp5 = t5 / (f2 * q2);
rv = value / y;

# Each product's term, as value carries it, less the term itself: e is
# that amount and W the same over the product as on_floats rounds it.
# The through point's terms are subtracted, so their e is the other way
# round.
W0 = rv - 1 / rp0;
W3 = rv - 1 / rp3;
W5 = rv - 1 / rp5;
e0 = t0 * W0; e3 = -(t3 * W3); e5 = -(t5 * W5);
c0 = |t0|; c3 = |t3|; c5 = |t5|;

{
  # Every input is a finite f64.
     |f0| <= 1b1024 /\ |f1| <= 1b1024 /\ |f2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |q0| <= 1b1024 /\ |q2| <= 1b1024
  # No factor of a product left is zero. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |f0| >= 1b-1074 /\ |f2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |q0| >= 1b-1074 /\ |q2| >= 1b-1074
  # product_is_normal on the products left.
  /\ |t0| >= 1b-1022 /\ |t3| >= 1b-1022 /\ |t5| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t0 -/ f0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ f0 * q0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t5 -/ f2 * q2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t3 / |t3| in [-1, 1] /\ t5 / |t5| in [-1, 1]
  # y is not zero. Where it is, value is zero too, and the claim on y is
  # the claim on value.
  /\ |y| >= 1b-1074
->
  (value - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (value - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / (f0 * p0) -> 1 + (t0 - f0 * p0) / (f0 * p0);
t3 / (f0 * q0) -> 1 + (t3 - f0 * q0) / (f0 * q0);
t5 / (f2 * q2) -> 1 + (t5 - f2 * q2) / (f2 * q2);
value / y -> 1 + (value - y) / y;

# The error is the sum of the products' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(value - Ex) / Phat -> ((Phat + (value - Ex)) - Phat) / Phat;
Phat + (value - Ex) -> (c0 + e0) + (c3 + e3) + (c5 + e5);
(c0 + e0) - c0 -> e0;
(c3 + e3) - c3 -> e3;
(c5 + e5) - c5 -> e5;
e0 / c0 -> (t0 / |t0|) * W0;
e3 / c3 -> -((t3 / |t3|) * W3);
e5 / c5 -> -((t5 / |t5|) * W5);

# From the permanent to the one on_floats rounds, and on to eb.
(value - Ex) / eb -> ((value - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
