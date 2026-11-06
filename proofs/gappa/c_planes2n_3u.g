# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at affe0f01
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at affe0f01
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs decide at affe0f01
# plane.g where n2 * p2 may be below the normals, which product_is_normal
# refuses: n2 * p2 is under 2^-1022 and negative, and the other two
# products are normal.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

X2 = n2 * p2;
t0 = rnd(n0 * p0); t1 = rnd(n1 * p1); t2 = rnd(X2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = s012 - off;
value = rnd(y);

Phat = |t0| + |t1| + -t2 + |off|;
P = rnd(rnd(rnd(|t0| + |t1|) + -t2) + |off|);

B = 3b-53;
eb = rnd(B * P);

Ex = n0 * p0 + n1 * p1 + X2 - off;

c0 = |t0|; c1 = |t1|; c2 = -X2;
P3 = |t0| + |t1| + -t2;
# The three terms' part of the permanent with X2 in place of t2.
P3x = c0 + c1 + c2;

rp0 = t0 / (n0 * p0); rp1 = t1 / (n1 * p1);
rs01 = s01 / S01; rs012 = s012 / S012;

# Each term's error as a fraction of the term, for X2 and not t2, and
# what rounding X2 cost, as an amount.
W0 = rs01 * rs012 - 1 / rp0;
W1 = rs01 * rs012 - 1 / rp1;
W2 = rs012 - 1;
e0 = t0 * W0; e1 = t1 * W1; e2 = X2 * W2;
d2 = (t2 - X2) * (rs012);

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024
  /\ |off| <= 1b1024
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074
  # The case: X2 is negative and below the normals, the other two
  # products are normal. A product of two doubles that are not zero is at
  # least 2^-2148.
  /\ X2 in [-1b-1022, -1b-2148]
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  /\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1]
->
  (e0 + e1 + e2) / P3x in ? /\ d2 in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The relative part is averaged over P3x as in plane.g; what rounding X2
# cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + d2 / eb;
(e0 + e1 + e2) / P3x -> ((P3x + (e0 + e1 + e2)) - P3x) / P3x;
P3x + (e0 + e1 + e2) -> (c0 + e0) + (c1 + e1) + (c2 + e2);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> -W2;
P3x / P -> (P3 / Phat) * (Phat / P) + (t2 - X2) / P;
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
