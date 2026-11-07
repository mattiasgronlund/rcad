# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at 6a4a814c
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# SideOfAFlatSurface::on_floats without its three product_is_normal checks:
# the sign of n . p - off. Products: n0 * p0 normal, n1 * p1 zero, n2 * p2 zero.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64. A zero product has
# the double 0 for its point's coordinate; with the normal's zero instead,
# every number below is the same.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = 0; p2 = 0;
off = rnd(off_);

# What on_floats computes, line for line. Adding a zero product to a double
# gives that double back (464.13's addzero.g), so zero products are left
# out of both folds.
t0 = rnd(n0 * p0);
y = t0 - off;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(|t0| + |off|);
Phat = |t0| + |off|;

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = n0 * p0 + n1 * p1 + n2 * p2 - off;

# The products' part of the permanent, with no rounding of its own, and
# the same with X in place of what each product below the normals rounds to.
P3 = |t0|;
c0 = |t0|;
P3x = c0;

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * p0);

# Each term as y carries it less the term: e as a fraction of the term,
# with X for each product below the normals, and d what rounding X cost.
W0 = 1 - 1 / rp0;
e0 = t0 * W0;

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a normal product is zero.
  /\ |n0| >= 1b-1074 /\ |p0| >= 1b-1074
  # The normal products, as product_is_normal has them.
  /\ |t0| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1]
->
  (e0) / P3x in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);

# The relative part is averaged over P3x as in plane.g; what rounding each
# X cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0) / P3x) * (P3x / P) * (P / eb);
(e0) / P3x -> ((P3x + (e0)) - P3x) / P3x;
P3x + (e0) -> (c0 + e0);
(c0 + e0) - c0 -> e0;
e0 / c0 -> (t0 / |t0|) * W0;
P3x / P -> (P3 / Phat) * (Phat / P);
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
