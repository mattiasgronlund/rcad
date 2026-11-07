# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at 6a4a814c
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# SideOfAFlatSurface::on_floats without its three product_is_normal checks:
# the sign of n . p - off. Products: n0 * p0 below the normals and negative, n1 * p1 normal, n2 * p2 zero.
# The sum adding product 1 cancels to zero, which rounds to itself.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64. A zero product has
# the double 0 for its point's coordinate; with the normal's zero instead,
# every number below is the same.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = 0;
off = rnd(off_);

# What on_floats computes, line for line. Adding a zero product to a double
# gives that double back (464.13's addzero.g), so zero products are left
# out of both folds.
X0 = n0 * p0;
t0 = rnd(X0); t1 = rnd(n1 * p1);
S1 = t0 + t1;
s1 = S1;
y = s1 - off;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(-t0 + |t1|) + |off|);
Phat = -t0 + |t1| + |off|;

B = 4b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = X0 + n1 * p1 + n2 * p2 - off;

# The products' part of the permanent, with no rounding of its own, and
# the same with X in place of what each product below the normals rounds to.
P3 = -t0 + |t1|;
c0 = -X0; c1 = |t1|;
P3x = c0 + c1;

# What each rounding multiplied its argument by.
rp1 = t1 / (n1 * p1);

# Each term as y carries it less the term: e as a fraction of the term,
# with X for each product below the normals, and d what rounding X cost.
W0 = 1 - 1;
W1 = 1 - 1 / rp1;
e0 = X0 * W0; e1 = t1 * W1;
d0 = (t0 - X0) * (1);

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a normal product is zero.
  /\ |n1| >= 1b-1074 /\ |p1| >= 1b-1074
  # The products below the normals. A product of two doubles that are not
  # zero is at least 2^-2148.
  /\ X0 in [-1b-1022, -1b-2148]
  # The normal products, as product_is_normal has them.
  /\ |t1| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t1 / |t1| in [-1, 1]
->
  (e0 + e1) / P3x in ? /\ d0 in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);

# The relative part is averaged over P3x as in plane.g; what rounding each
# X cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0 + e1) / P3x) * (P3x / P) * (P / eb) + (d0) / eb;
(e0 + e1) / P3x -> ((P3x + (e0 + e1)) - P3x) / P3x;
P3x + (e0 + e1) -> (c0 + e0) + (c1 + e1);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
e0 / c0 -> -W0;
e1 / c1 -> (t1 / |t1|) * W1;
P3x / P -> (P3 / Phat) * (Phat / P) + (t0 - X0) / P;
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
