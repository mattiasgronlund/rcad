# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at eb125ed0
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at eb125ed0
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0
# restates: crates/rcad-predicates/src/cascade.rs decide at eb125ed0
# SideOfAFlatSurfaceThroughAPoint::on_floats without its six product_is_normal
# checks: the sign of facing . point - facing . through, as six rounded
# products of two doubles folded left to right.
# The terms left, in the order the fold adds them: nnssns.
# The sums adding terms 1, 4 cancel to zero, which rounds to itself.
@rnd = float<ieee_64, ne>;

# The two factors of each term left, each an f64.
a0 = rnd(a0_); b0 = rnd(b0_); a1 = rnd(a1_); b1 = rnd(b1_); a2 = rnd(a2_); b2 = rnd(b2_); a3 = rnd(a3_); b3 = rnd(b3_); a4 = rnd(a4_); b4 = rnd(b4_); a5 = rnd(a5_); b5 = rnd(b5_);

# What on_floats computes, line for line. The fold starts at 0.0, and
# adding 0.0 or a term that rounded to zero to a double gives that double
# back (464.13's addzero.g), so those and the first addition are left out.
X0 = a0 * b0; X1 = a1 * b1; X2 = a2 * b2; X3 = a3 * b3; X4 = a4 * b4; X5 = a5 * b5;
t0 = rnd(X0); t1 = rnd(X1); t2 = rnd(X2); t3 = rnd(X3); t4 = rnd(X4); t5 = rnd(X5);
S1 = t0 + t1;
s1 = S1;
S2 = s1 + t2;
s2 = rnd(S2);
S3 = s2 + t3;
s3 = rnd(S3);
S4 = s3 + t4;
s4 = S4;
y = s4 + t5;

# The permanent folds the same terms' magnitudes in the same order.
P = rnd(rnd(rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |t3|) + |t4|) + |t5|);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = X0 + X1 + X2 + X3 + X4 + X5;

# The permanent with no rounding of its own.
Phat = |t0| + |t1| + |t2| + |t3| + |t4| + |t5|;

# What each rounding multiplied its argument by.
rp0 = t0 / X0; rp1 = t1 / X1; rp4 = t4 / X4;
rs2 = s2 / S2; rs3 = s3 / S3;

# Each term as y carries it less the term: e the part that is a fraction
# of the term as on_floats rounds it, with W that fraction, and for a term
# below the normals d what its own rounding cost, an amount.
W0 = rs2 * rs3 - 1 / rp0;
W1 = rs2 * rs3 - 1 / rp1;
W2 = rs2 * rs3 - 1;
W3 = rs3 - 1;
W4 = 1 - 1 / rp4;
W5 = 1 - 1;
e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2; e3 = t3 * W3; e4 = t4 * W4; e5 = t5 * W5;
c0 = |t0|; c1 = |t1|; c2 = |t2|; c3 = |t3|; c4 = |t4|; c5 = |t5|;
d2 = t2 - X2; d3 = t3 - X3; d5 = t5 - X5;
A = d2 + d3 + d5;

{
  # Every input is a finite f64.
     |a0| <= 1b1024 /\ |b0| <= 1b1024 /\ |a1| <= 1b1024 /\ |b1| <= 1b1024 /\ |a2| <= 1b1024 /\ |b2| <= 1b1024 /\ |a3| <= 1b1024 /\ |b3| <= 1b1024 /\ |a4| <= 1b1024 /\ |b4| <= 1b1024 /\ |a5| <= 1b1024 /\ |b5| <= 1b1024
  # No factor of a normal term is zero.
  /\ |a0| >= 1b-1074 /\ |b0| >= 1b-1074 /\ |a1| >= 1b-1074 /\ |b1| >= 1b-1074 /\ |a4| >= 1b-1074 /\ |b4| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S2| >= 1b-1074 /\ |S3| >= 1b-1074
  # The terms below the normals that round to a double that is not zero.
  /\ |X2| <= 1b-1022 /\ |t2| >= 1b-1074 /\ |X3| <= 1b-1022 /\ |t3| >= 1b-1074 /\ |X5| <= 1b-1022 /\ |t5| >= 1b-1074
  # The normal terms, as product_is_normal has them.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t4| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t0 -/ X0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ X1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t4 -/ X4 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1] /\ t2 / |t2| in [-1, 1] /\ t3 / |t3| in [-1, 1] /\ t4 / |t4| in [-1, 1] /\ t5 / |t5| in [-1, 1]
->
  (e0 + e1 + e2 + e3 + e4 + e5) / Phat in ? /\ P -/ Phat in ? /\ A in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / X0 -> 1 + (t0 - X0) / X0;
t1 / X1 -> 1 + (t1 - X1) / X1;
t4 / X4 -> 1 + (t4 - X4) / X4;
s2 / S2 -> 1 + (s2 - S2) / S2;
s3 / S3 -> 1 + (s3 - S3) / S3;

# The relative part is averaged over the permanent as in flat.g; what
# rounding a term below the normals cost, and each term that rounded to
# zero, is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0 + e1 + e2 + e3 + e4 + e5) / Phat) * (Phat / P) * (P / eb) + A / eb;
(e0 + e1 + e2 + e3 + e4 + e5) / Phat -> ((Phat + (e0 + e1 + e2 + e3 + e4 + e5)) - Phat) / Phat;
Phat + (e0 + e1 + e2 + e3 + e4 + e5) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3) + (c4 + e4) + (c5 + e5);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(c4 + e4) - c4 -> e4;
(c5 + e5) - c5 -> e5;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> (t2 / |t2|) * W2;
e3 / c3 -> (t3 / |t3|) * W3;
e4 / c4 -> (t4 / |t4|) * W4;
e5 / c5 -> (t5 / |t5|) * W5;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
