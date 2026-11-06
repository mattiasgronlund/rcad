# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at affe0f01
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at affe0f01
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs decide at affe0f01
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

B = 4b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = n0 * p0 + n1 * p1 + n2 * p2 - off;

# The three terms' part of the permanent, with no rounding.
c0 = |t0|; c1 = |t1|; c2 = |t2|;
P3 = c0 + c1 + c2;

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * p0); rp1 = t1 / (n1 * p1); rp2 = t2 / (n2 * p2);
rs01 = s01 / S01; rs012 = s012 / S012;

# Each of the three terms, as y carries it, less the term itself. The
# offset reaches y with no rounding.
W0 = rs01 * rs012 - 1 / rp0;
W1 = rs01 * rs012 - 1 / rp1;
W2 = rs012 - 1 / rp2;
e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2;

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
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those four gives.
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1] /\ t2 / |t2| in [-1, 1]
->
  (y - Ex) / P3 in ? /\ P -/ Phat in ? /\ rp0 in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The error is the sum of the three terms' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / P3 -> ((P3 + (y - Ex)) - P3) / P3;
P3 + (y - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> (t2 / |t2|) * W2;

# From the three terms to the whole permanent, which the offset only
# makes larger, to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P3) * (P3 / Phat) * (Phat / P) * (P / eb);
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
