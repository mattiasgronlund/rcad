# restates: crates/rcad-predicates/src/orient2d.rs on_floats at affe0f01
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at affe0f01
# restates: crates/rcad-predicates/src/cascade.rs decide at affe0f01
# o2.g where backwards may be below the normals, which on_floats does not
# check: acy * bcx is under 2^-1022 and negative, and acx * bcy is at
# least 2^-1000.
@rnd = float<ieee_64, ne>;

ax = rnd(ax_); ay = rnd(ay_);
bx = rnd(bx_); by = rnd(by_);
cx = rnd(cx_); cy = rnd(cy_);

Acx = ax - cx; Acy = ay - cy;
Bcx = bx - cx; Bcy = by - cy;
acx = rnd(Acx); acy = rnd(Acy);
bcx = rnd(Bcx); bcy = rnd(Bcy);

F = acx * bcy;
G = acy * bcx;
forwards = rnd(F);
backwards = rnd(G);
y = forwards - backwards;
value = rnd(y);

# backwards is not positive, as G is not, so |backwards| is -backwards.
Phat = |forwards| - backwards;
P = rnd(Phat);

B = 3b-53;
eb = rnd(B * P);

Ex = Acx * Bcy - Acy * Bcx;

racx = acx / Acx; racy = acy / Acy;
rbcx = bcx / Bcx; rbcy = bcy / Bcy;
rp1 = forwards / F;

# The permanent with G in place of what it rounds to.
c1 = |forwards|; c2 = -G;
Phat2 = c1 + c2;

# Each product less the product of the exact differences, as relative
# errors for forwards and for G, and what rounding G cost, as an amount.
W1 = 1 - 1 / (racx * rbcy * rp1);
W2 = 1 - 1 / (racy * rbcx);
e1 = forwards * W1; e2 = -(G * W2);
d2 = -(backwards - G);

{
     |ax| <= 1b1024 /\ |ay| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  /\ |acx| >= 1b-1074 /\ |acy| >= 1b-1074
  /\ |bcx| >= 1b-1074 /\ |bcy| >= 1b-1074
  # The case: G is negative and above the normals, F is far above them.
  # A product of two doubles that are not zero is at least 2^-2148.
  /\ G in [-1b-1022, -1b-2148] /\ |F| >= 1b-1000
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  /\ forwards / |forwards| in [-1, 1]
->
  (e1 + e2) / Phat2 in ? /\ d2 in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

acx / Acx -> 1 + (acx - Acx) / Acx;
acy / Acy -> 1 + (acy - Acy) / Acy;
bcx / Bcx -> 1 + (bcx - Bcx) / Bcx;
bcy / Bcy -> 1 + (bcy - Bcy) / Bcy;
forwards / F -> 1 + (forwards - F) / F;

# The relative part is averaged over Phat2 as in o2.g; what rounding G
# cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e1 + e2) / Phat2) * (Phat2 / P) * (P / eb) + d2 / eb;
(e1 + e2) / Phat2 -> ((Phat2 + (e1 + e2)) - Phat2) / Phat2;
Phat2 + (e1 + e2) -> (c1 + e1) + (c2 + e2);
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e1 / c1 -> (forwards / |forwards|) * W1;
e2 / c2 -> W2;
Phat2 / P -> Phat / P + (backwards - G) / P;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
