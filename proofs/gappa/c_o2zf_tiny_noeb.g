# restates: crates/rcad-predicates/src/orient2d.rs on_floats at 6a4a814c
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# Orient2d::on_floats where a and c share their x, so acx is zero; with b and c sharing
# their y instead, bcy is zero, and every number below is the
# same. Either way forwards is the double 0. acy * bcx is at most 2^-1000.
@rnd = float<ieee_64, ne>;

# The four coordinates left, each an f64.
ay = rnd(ay_); bx = rnd(bx_); cx = rnd(cx_); cy = rnd(cy_);

# What on_floats computes, line for line. The zero difference is exactly
# 0, and so is forwards; 0 less backwards is -backwards, and |forwards| plus
# |backwards| is |backwards| (464.13's addzero.g).
Acy = ay - cy; Bcx = bx - cx;
acy = rnd(Acy); bcx = rnd(Bcx);
G = acy * bcx;
backwards = rnd(G);
y = -backwards;
value = rnd(y);
P = |backwards|;

B = 3b-53 + 1b-102;
eb = rnd(B * P);

# The true determinant of the same inputs, the zero difference being zero.
Ex = -(Acy * Bcx);

# What each rounding multiplied its argument by.
r1 = acy / Acy; r2 = bcx / Bcx; rp = backwards / G;

# backwards less the product of the exact differences, as a fraction of it.
W = 1 - 1 / (r1 * r2 * rp);
e = -(backwards * W);

{
  # Every coordinate is a finite f64.
     |ay| <= 1b1024 /\ |bx| <= 1b1024 /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  # The other two differences are not zero.
  /\ |acy| >= 1b-1074 /\ |bcx| >= 1b-1074
  # The case: acy * bcx is at most 2^-1000.
  /\ |G| <= 1b-1000
->
  eb in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
acy / Acy -> 1 + (acy - Acy) / Acy;
bcx / Bcx -> 1 + (bcx - Bcx) / Bcx;
backwards / G -> 1 + (backwards - G) / G;

# forwards is zero, so y less the truth is backwards's error alone.
(y - Ex) / eb -> (e / P) * (P / eb);
e / P -> -((backwards / |backwards|) * W);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
