# restates: crates/rcad-predicates/src/orient3d.rs on_floats at d9f931e8
# restates: crates/rcad-predicates/src/orient3d.rs exact_coordinates at d9f931e8
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at d9f931e8
# restates: crates/rcad-predicates/src/orient3d.rs ORIENT3D_ERROR_BOUND at d9f931e8
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at d9f931e8
# restates: crates/rcad-predicates/src/cascade.rs decide at d9f931e8
# Block 1 where adz and bdx and bdz and cdx are zero. A zero rounds to itself,
# and so does a product with a zero factor.
@rnd = float<ieee_64, ne>;

# The twelve input coordinates, each an f64.
ax = rnd(ax_); ay = rnd(ay_); az = rnd(az_);
bx = rnd(bx_); by = rnd(by_); bz = az;
cx = bx; cy = rnd(cy_); cz = rnd(cz_);
dx = bx; dy = rnd(dy_); dz = az;

# What on_floats computes, line for line.
Adx = ax - dx; Ady = ay - dy; Adz = az - dz;
Bdx = bx - dx; Bdy = by - dy; Bdz = bz - dz;
Cdx = cx - dx; Cdy = cy - dy; Cdz = cz - dz;
adx = rnd(Adx); ady = rnd(Ady); adz = Adz;
bdx = Bdx; bdy = rnd(Bdy); bdz = Bdz;
cdx = Cdx; cdy = rnd(Cdy); cdz = rnd(Cdz);

bdxcdy = bdx * cdy;
cdxbdy = cdx * bdy;
cdxady = cdx * ady;
adxcdy = rnd(adx * cdy);
adxbdy = rnd(adx * bdy);
bdxady = bdx * ady;

M1 = bdxcdy - cdxbdy; M2 = cdxady - adxcdy; M3 = adxbdy - bdxady;
first = M1; second = rnd(M2); third = rnd(M3);

U1 = adz * first; U2 = bdz * second; U3 = cdz * third;
t1 = U1; t2 = U2; t3 = rnd(U3);
S = t1 + t2;
s12 = S;
y = s12 + t3;
value = rnd(y);

q1 = |bdxcdy| + |cdxbdy|;
q2 = rnd(|cdxady| + |adxcdy|);
q3 = rnd(|adxbdy| + |bdxady|);
R1 = q1 * |adz|; R2 = q2 * |bdz|; R3 = q3 * |cdz|;
r1 = R1; r2 = R2; r3 = rnd(R3);
P = rnd((r1 + r2) + r3);

B = 7b-53 + 7b-103;
eb = rnd(B * P);

# The true determinant of the same inputs.
Ex = Adz * (Bdx * Cdy - Cdx * Bdy)
   + Bdz * (Cdx * Ady - Adx * Cdy)
   + Cdz * (Adx * Bdy - Bdx * Ady);

# The permanent with every rounding taken out, twice: as P groups it, and
# as six products.
Phat1 = (|bdxcdy| + |cdxbdy|) * |adz|;
Phat2 = (|cdxady| + |adxcdy|) * |bdz|;
Phat3 = (|adxbdy| + |bdxady|) * |cdz|;
Phat = Phat1 + Phat2 + Phat3;
c1 = |adz| * |bdxcdy|; c2 = |adz| * |cdxbdy|;
c3 = |bdz| * |cdxady|; c4 = |bdz| * |adxcdy|;
c5 = |cdz| * |adxbdy|; c6 = |cdz| * |bdxady|;
P6 = c1 + c2 + c3 + c4 + c5 + c6;

# What each rounding multiplied its argument by.
radx = adx / Adx; rady = ady / Ady;
rbdy = bdy / Bdy;
rcdy = cdy / Cdy; rcdz = cdz / Cdz;
rp4 = adxcdy / (adx * cdy);
rp5 = adxbdy / (adx * bdy);
rm3 = third / M3;
rt3 = t3 / U3;

# Each of the six terms of the determinant, as y carries it, less the
# term itself.
W5 = rm3 * rt3 - 1 / (rcdz * radx * rbdy * rp5);
e5 = cdz * adxbdy * W5;

{
  # Every coordinate is a finite f64.
     |ax| <= 1b1024 /\ |ay| <= 1b1024 /\ |az| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024 /\ |bz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024 /\ |cz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  # No factor of the nine products is zero but adz and bdx and bdz and cdx. For an
  # f64 that is the same as being at least the smallest subnormal.
  /\ |adx| >= 1b-1074 /\ |ady| >= 1b-1074
  /\ |bdy| >= 1b-1074
  /\ |cdy| >= 1b-1074 /\ |cdz| >= 1b-1074
  /\ |third| >= 1b-1074
  # §12.2's first condition: product_is_normal on the nine products. Those
  # on a zero pass, and say nothing.
  /\ |adxbdy| >= 1b-1022
  /\ |t3| >= 1b-1022
  # Each of r3 is normal, because its t is and its minor is
  # at most its q. Each of r1, r2 is zero.
  /\ |r3| >= 1b-1022
  # §12.2's second condition: the bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those thirteen gives: the rounding is off by
  # at most u + 2u^2. Gappa's own theorem needs the value strictly above
  # 2^-1022, so it cannot take the step itself.
  /\ adxcdy -/ adx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxbdy -/ adx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ U3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r3 -/ R3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # q2 and q3 each round a double plus a zero, so each is at least 2^-1022
  # as that double is, and lemma.g gives the same of it. The goals
  # check the first part.
  /\ q2 -/ (|cdxady| + |adxcdy|) in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q3 -/ (|adxbdy| + |bdxady|) in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ cdz / |cdz| in [-1, 1]
  /\ adxcdy / |adxcdy| in [-1, 1]
  /\ adxbdy / |adxbdy| in [-1, 1]
->
  |second| >= 1b-1022
}

# How far each rounding is from the identity.
adx / Adx -> 1 + (adx - Adx) / Adx;
ady / Ady -> 1 + (ady - Ady) / Ady;
bdy / Bdy -> 1 + (bdy - Bdy) / Bdy;
cdy / Cdy -> 1 + (cdy - Cdy) / Cdy;
cdz / Cdz -> 1 + (cdz - Cdz) / Cdz;
adxcdy / (adx * cdy) -> 1 + (adxcdy - adx * cdy) / (adx * cdy);
adxbdy / (adx * bdy) -> 1 + (adxbdy - adx * bdy) / (adx * bdy);
third / M3 -> 1 + (third - M3) / M3;
t3 / U3 -> 1 + (t3 - U3) / U3;

# The error is the sum of the six terms' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / P6 -> ((P6 + (y - Ex)) - P6) / P6;
P6 + (y - Ex) -> c1 + c2 + c3 + c4 + (c5 + e5) + c6;
(c5 + e5) - c5 -> e5;
e5 / c5 -> (cdz / |cdz|) * (adxbdy / |adxbdy|) * W5;

# From the six terms' permanent to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P6) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));

# third each round a double less a zero, or a zero less a double,
# and so round to themselves.
adxbdy - bdxady -> adxbdy;

# r1 and r2 are zero, and so are Phat1 and Phat2. Carry each as an
# amount against Phat, which Phat3 keeps away from zero.
(P - Phat) / Phat -> ((P - ((r1 + r2) + r3)) / ((r1 + r2) + r3)) * (1 + (((r1 + r2) + r3) - Phat) / Phat) + (((r1 + r2) + r3) - Phat) / Phat;
(((r1 + r2) + r3) - Phat) / Phat -> (r1 - Phat1) / Phat + (r2 - Phat2) / Phat + ((r3 - Phat3) / Phat3) * (Phat3 / Phat);
Phat3 / Phat -> 1 / (1 + (Phat1 + Phat2) / Phat3);

# second rounds a double less a zero, or a zero less a double,
# and so rounds to itself.
cdxady - adxcdy -> -adxcdy;
