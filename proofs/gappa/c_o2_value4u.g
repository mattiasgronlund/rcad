# restates: crates/rcad-predicates/src/orient2d.rs on_floats at affe0f01
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs decide at affe0f01
# o2.g with the claim taken on value rather than y: 464.8 1's o2v.g. It
# fails at the code's B, so it is a control of o2.g.
# Orient2d::on_floats and ORIENT2D_ERROR_BOUND,
# crates/rcad-predicates/src/orient2d.rs at affe0f01.
@rnd = float<ieee_64, ne>;

# The six input coordinates, each an f64.
ax = rnd(ax_); ay = rnd(ay_);
bx = rnd(bx_); by = rnd(by_);
cx = rnd(cx_); cy = rnd(cy_);

# What on_floats computes, line for line.
Acx = ax - cx; Acy = ay - cy;
Bcx = bx - cx; Bcy = by - cy;
acx = rnd(Acx); acy = rnd(Acy);
bcx = rnd(Bcx); bcy = rnd(Bcy);

forwards = rnd(acx * bcy);
backwards = rnd(acy * bcx);
y = forwards - backwards;
value = rnd(y);

Phat = |forwards| + |backwards|;
P = rnd(Phat);

B = 4b-53 + 1b-102;
eb = rnd(B * P);

# The true determinant of the same inputs.
Ex = Acx * Bcy - Acy * Bcx;

# What each rounding multiplied its argument by.
racx = acx / Acx; racy = acy / Acy;
rbcx = bcx / Bcx; rbcy = bcy / Bcy;
rp1 = forwards / (acx * bcy); rp2 = backwards / (acy * bcx);
rv = value / y;

# Each of the two products, as value carries it, less the product of the
# exact differences.
W1 = rv - 1 / (racx * rbcy * rp1);
W2 = rv - 1 / (racy * rbcx * rp2);
c1 = |forwards|; c2 = |backwards|;
e1 = forwards * W1; e2 = -(backwards * W2);

{
  # Every coordinate is a finite f64.
     |ax| <= 1b1024 /\ |ay| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  # No difference is zero.
  /\ |acx| >= 1b-1074 /\ |acy| >= 1b-1074
  /\ |bcx| >= 1b-1074 /\ |bcy| >= 1b-1074
  # y is not zero.
  /\ |y| >= 1b-1074
  # Both products are normal. on_floats does not check this.
  /\ |forwards| >= 1b-1022 /\ |backwards| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those three gives.
  /\ forwards -/ acx * bcy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ backwards -/ acy * bcx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ forwards / |forwards| in [-1, 1] /\ backwards / |backwards| in [-1, 1]
->
  (value - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (value - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
acx / Acx -> 1 + (acx - Acx) / Acx;
acy / Acy -> 1 + (acy - Acy) / Acy;
bcx / Bcx -> 1 + (bcx - Bcx) / Bcx;
bcy / Bcy -> 1 + (bcy - Bcy) / Bcy;
forwards / (acx * bcy) -> 1 + (forwards - acx * bcy) / (acx * bcy);
backwards / (acy * bcx) -> 1 + (backwards - acy * bcx) / (acy * bcx);
value / y -> 1 + (value - y) / y;

# The error is the sum of the two products' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(value - Ex) / Phat -> ((Phat + (value - Ex)) - Phat) / Phat;
Phat + (value - Ex) -> (c1 + e1) + (c2 + e2);
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e1 / c1 -> (forwards / |forwards|) * W1;
e2 / c2 -> -((backwards / |backwards|) * W2);

# From the permanent to the one on_floats rounds, and on to eb.
(value - Ex) / eb -> ((value - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
