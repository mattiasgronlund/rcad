# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at eb125ed0
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at eb125ed0
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0
# restates: crates/rcad-predicates/src/cascade.rs decide at eb125ed0
# SideOfAFlatSurfaceThroughAPoint::on_floats without its six product_is_normal
# checks: the sign of facing . point - facing . through, as six rounded
# products of two doubles folded left to right.
# The terms left, in the order the fold adds them: sn; 4 more round to zero.
@rnd = float<ieee_64, ne>;

# The two factors of each term left, each an f64.
a0 = rnd(a0_); b0 = rnd(b0_); a1 = rnd(a1_); b1 = rnd(b1_);
# The two factors of each term that rounds to zero.
g0 = rnd(g0_); h0 = rnd(h0_); g1 = rnd(g1_); h1 = rnd(h1_); g2 = rnd(g2_); h2 = rnd(h2_); g3 = rnd(g3_); h3 = rnd(h3_);

# What on_floats computes, line for line. The fold starts at 0.0, and
# adding 0.0 or a term that rounded to zero to a double gives that double
# back (464.13's addzero.g), so those and the first addition are left out.
X0 = a0 * b0; X1 = a1 * b1;
Z0 = g0 * h0; Z1 = g1 * h1; Z2 = g2 * h2; Z3 = g3 * h3;
t0 = rnd(X0); t1 = rnd(X1);
y = t0 + t1;

# The permanent folds the same terms' magnitudes in the same order.
P = rnd(|t0| + |t1|);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = X0 + X1 + Z0 + Z1 + Z2 + Z3;

# The permanent with no rounding of its own.
Phat = |t0| + |t1|;

# What each rounding multiplied its argument by.
rp1 = t1 / X1;

# Each term as y carries it less the term: e the part that is a fraction
# of the term as on_floats rounds it, with W that fraction, and for a term
# below the normals d what its own rounding cost, an amount.
W0 = 1 - 1;
W1 = 1 - 1 / rp1;
e0 = t0 * W0; e1 = t1 * W1;
c0 = |t0|; c1 = |t1|;
d0 = t0 - X0;
A = d0 - Z0 - Z1 - Z2 - Z3;

{
  # Every input is a finite f64.
     |a0| <= 1b1024 /\ |b0| <= 1b1024 /\ |a1| <= 1b1024 /\ |b1| <= 1b1024
  /\ |g0| <= 1b1024 /\ |h0| <= 1b1024 /\ |g1| <= 1b1024 /\ |h1| <= 1b1024 /\ |g2| <= 1b1024 /\ |h2| <= 1b1024 /\ |g3| <= 1b1024 /\ |h3| <= 1b1024
  # No factor of a normal term is zero.
  /\ |a1| >= 1b-1074 /\ |b1| >= 1b-1074
  # The terms below the normals that round to a double that is not zero.
  /\ |X0| <= 1b-1022 /\ |t0| >= 1b-1074
  # The terms that round to zero (prodzero.g).
  /\ |Z0| <= 1b-1075 /\ |Z1| <= 1b-1075 /\ |Z2| <= 1b-1075 /\ |Z3| <= 1b-1075
  # The normal terms, as product_is_normal has them.
  /\ |t1| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1]
->
  (e0 + e1) / Phat in ? /\ P -/ Phat in ? /\ A in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t1 / X1 -> 1 + (t1 - X1) / X1;

# The relative part is averaged over the permanent as in flat.g; what
# rounding a term below the normals cost, and each term that rounded to
# zero, is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0 + e1) / Phat) * (Phat / P) * (P / eb) + A / eb;
(e0 + e1) / Phat -> ((Phat + (e0 + e1)) - Phat) / Phat;
Phat + (e0 + e1) -> (c0 + e0) + (c1 + e1);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
