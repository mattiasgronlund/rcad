# restates: crates/rcad-predicates/src/orient2d.rs on_floats at 6a4a814c
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# Orient2d::on_floats where a and c share their y, so acy is zero; with b and c sharing
# their x instead, bcx is zero, and every number below is the
# same. Either way backwards is the double 0. acx * bcy is at most 2^-1000.
@rnd = float<ieee_64, ne>;

# The four coordinates left, each an f64.
ax = rnd(ax_); by = rnd(by_); cx = rnd(cx_); cy = rnd(cy_);

# What on_floats computes, line for line. The zero difference is exactly
# 0, and so is backwards; forwards less 0 is forwards, and |forwards| plus
# |backwards| is |forwards| (464.13's addzero.g).
Acx = ax - cx; Bcy = by - cy;
acx = rnd(Acx); bcy = rnd(Bcy);
G = acx * bcy;
forwards = rnd(G);
y = forwards;
value = rnd(y);
P = |forwards|;

B = 3b-53 + 1b-102;
eb = rnd(B * P);

# The true determinant of the same inputs, the zero difference being zero.
Ex = Acx * Bcy;

# What each rounding multiplied its argument by.
r1 = acx / Acx; r2 = bcy / Bcy; rp = forwards / G;

# forwards less the product of the exact differences, as a fraction of it.
W = 1 - 1 / (r1 * r2 * rp);
e = forwards * W;

{
  # Every coordinate is a finite f64.
     |ax| <= 1b1024 /\ |by| <= 1b1024 /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  # The other two differences are not zero.
  /\ |acx| >= 1b-1074 /\ |bcy| >= 1b-1074
  # The case: acx * bcy is at most 2^-1000.
  /\ |G| <= 1b-1000
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
->
  eb in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
acx / Acx -> 1 + (acx - Acx) / Acx;
bcy / Bcy -> 1 + (bcy - Bcy) / Bcy;
forwards / G -> 1 + (forwards - G) / G;

# backwards is zero, so y less the truth is forwards's error alone.
(y - Ex) / eb -> (e / P) * (P / eb);
e / P -> (forwards / |forwards|) * W;
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
