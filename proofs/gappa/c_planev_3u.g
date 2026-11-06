# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at affe0f01
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at affe0f01
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs decide at affe0f01
# plane.g with the claim taken on value rather than y: 464.8 3's planev.g.
# SideOfAFlatSurface::on_floats and SIDE_OF_PLANE_ERROR_BOUND,
# crates/rcad-geom/src/plane.rs at affe0f01.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

# What on_floats computes, line for line.
t0 = rnd(n0 * p0); t1 = rnd(n1 * p1); t2 = rnd(n2 * p2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = s012 - off;
value = rnd(y);

Phat = |t0| + |t1| + |t2| + |off|;
P = rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |off|);

B = 3b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = n0 * p0 + n1 * p1 + n2 * p2 - off;

# Each term's part of the permanent, with no rounding.
c0 = |t0|; c1 = |t1|; c2 = |t2|;
c3 = |off|;

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * p0); rp1 = t1 / (n1 * p1); rp2 = t2 / (n2 * p2);
rs01 = s01 / S01; rs012 = s012 / S012;
rv = value / y;

# Each of the four terms, as value carries it, less the term itself. The
# offset reaches value through its last rounding alone.
W0 = rv * rs01 * rs012 - 1 / rp0;
W1 = rv * rs01 * rs012 - 1 / rp1;
W2 = rv * rs012 - 1 / rp2;
W3 = rv - 1;
e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2; e3 = -(off * W3);

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  # Neither sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074
  # Neither the offset nor y is zero.
  /\ |off| >= 1b-1074 /\ |y| >= 1b-1074
  # product_is_normal on the three products.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those four gives.
  /\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1] /\ t2 / |t2| in [-1, 1]
  /\ off / |off| in [-1, 1]
->
  (value - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (value - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(value - Ex) / Phat -> ((Phat + (value - Ex)) - Phat) / Phat;
Phat + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> (t2 / |t2|) * W2;
e3 / c3 -> -((off / |off|) * W3);

# From the permanent to the one on_floats rounds, and on to eb.
(value - Ex) / eb -> ((value - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
