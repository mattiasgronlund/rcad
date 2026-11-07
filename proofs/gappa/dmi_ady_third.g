# restates: crates/rcad-predicates/src/orient3d.rs on_floats at d9f931e8
# restates: crates/rcad-predicates/src/orient3d.rs exact_coordinates at d9f931e8
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at d9f931e8
# restates: crates/rcad-predicates/src/orient3d.rs ORIENT3D_ERROR_BOUND at d9f931e8
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at d9f931e8
# restates: crates/rcad-predicates/src/cascade.rs decide at d9f931e8
# Block 1 where ady are zero. A zero rounds to itself,
# and so does a product with a zero factor.
@rnd = float<ieee_64, ne>;

# The twelve input coordinates, each an f64.
ax = rnd(ax_); ay = rnd(ay_); az = rnd(az_);
bx = rnd(bx_); by = rnd(by_); bz = rnd(bz_);
cx = rnd(cx_); cy = rnd(cy_); cz = rnd(cz_);
dx = rnd(dx_); dy = ay; dz = rnd(dz_);

# What on_floats computes, line for line.
Adx = ax - dx; Ady = ay - dy; Adz = az - dz;
Bdx = bx - dx; Bdy = by - dy; Bdz = bz - dz;
Cdx = cx - dx; Cdy = cy - dy; Cdz = cz - dz;
adx = rnd(Adx); ady = Ady; adz = rnd(Adz);
bdx = rnd(Bdx); bdy = rnd(Bdy); bdz = rnd(Bdz);
cdx = rnd(Cdx); cdy = rnd(Cdy); cdz = rnd(Cdz);

bdxcdy = rnd(bdx * cdy);
cdxbdy = rnd(cdx * bdy);
cdxady = cdx * ady;
adxcdy = rnd(adx * cdy);
adxbdy = rnd(adx * bdy);
bdxady = bdx * ady;

M1 = bdxcdy - cdxbdy; M2 = cdxady - adxcdy; M3 = adxbdy - bdxady;
first = rnd(M1); second = rnd(M2); third = rnd(M3);

U1 = adz * first; U2 = bdz * second; U3 = cdz * third;
t1 = rnd(U1); t2 = rnd(U2); t3 = rnd(U3);
S = t1 + t2;
s12 = rnd(S);
y = s12 + t3;
value = rnd(y);

q1 = rnd(|bdxcdy| + |cdxbdy|);
q2 = rnd(|cdxady| + |adxcdy|);
q3 = rnd(|adxbdy| + |bdxady|);
R1 = q1 * |adz|; R2 = q2 * |bdz|; R3 = q3 * |cdz|;
r1 = rnd(R1); r2 = rnd(R2); r3 = rnd(R3);
P = rnd(rnd(r1 + r2) + r3);

B = 7b-53 + 7b-103;
eb = rnd(B * P);

# The true determinant of the same inputs.
Ex = Adz * (Bdx * Cdy - Cdx * Bdy)
   + Bdz * (Cdx * Ady - Adx * Cdy)
   + Cdz * (Adx * Bdy - Bdx * Ady);

# The permanent with every rounding taken out, twice: as P groups it, and
# as six products.
Phat = (|bdxcdy| + |cdxbdy|) * |adz|
     + (|cdxady| + |adxcdy|) * |bdz|
     + (|adxbdy| + |bdxady|) * |cdz|;
c1 = |adz| * |bdxcdy|; c2 = |adz| * |cdxbdy|;
c3 = |bdz| * |cdxady|; c4 = |bdz| * |adxcdy|;
c5 = |cdz| * |adxbdy|; c6 = |cdz| * |bdxady|;
P6 = c1 + c2 + c3 + c4 + c5 + c6;

# What each rounding multiplied its argument by.
radx = adx / Adx; radz = adz / Adz;
rbdx = bdx / Bdx; rbdy = bdy / Bdy; rbdz = bdz / Bdz;
rcdx = cdx / Cdx; rcdy = cdy / Cdy; rcdz = cdz / Cdz;
rp1 = bdxcdy / (bdx * cdy); rp2 = cdxbdy / (cdx * bdy);
rp4 = adxcdy / (adx * cdy);
rp5 = adxbdy / (adx * bdy);
rm1 = first / M1; rm2 = second / M2; rm3 = third / M3;
rt1 = t1 / U1; rt2 = t2 / U2; rt3 = t3 / U3;
rs = s12 / S;

# Each of the six terms of the determinant, as y carries it, less the
# term itself.
W1 = rm1 * rt1 * rs - 1 / (radz * rbdx * rcdy * rp1);
W2 = rm1 * rt1 * rs - 1 / (radz * rcdx * rbdy * rp2);
W4 = rm2 * rt2 * rs - 1 / (rbdz * radx * rcdy * rp4);
W5 = rm3 * rt3 - 1 / (rcdz * radx * rbdy * rp5);
e1 = adz * bdxcdy * W1; e2 = -(adz * cdxbdy * W2);
e4 = -(bdz * adxcdy * W4);
e5 = cdz * adxbdy * W5;

{
  # Every coordinate is a finite f64.
     |ax| <= 1b1024 /\ |ay| <= 1b1024 /\ |az| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024 /\ |bz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024 /\ |cz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  # No factor of the nine products is zero but ady. For an
  # f64 that is the same as being at least the smallest subnormal.
  /\ |adx| >= 1b-1074 /\ |adz| >= 1b-1074
  /\ |bdx| >= 1b-1074 /\ |bdy| >= 1b-1074 /\ |bdz| >= 1b-1074
  /\ |cdx| >= 1b-1074 /\ |cdy| >= 1b-1074 /\ |cdz| >= 1b-1074
  /\ |first| >= 1b-1074 /\ |second| >= 1b-1074
  # The first two terms do not cancel to zero.
  /\ |t1 + t2| >= 1b-1074
  # §12.2's first condition: product_is_normal on the nine products. Those
  # on a zero pass, and say nothing.
  /\ |bdxcdy| >= 1b-1022 /\ |cdxbdy| >= 1b-1022
  /\ |adxcdy| >= 1b-1022 /\ |adxbdy| >= 1b-1022
  /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022 /\ |t3| >= 1b-1022
  # Each of r1, r2, r3 is normal, because its t is and its minor is
  # at most its q. Each of  is zero.
  /\ |r1| >= 1b-1022 /\ |r2| >= 1b-1022 /\ |r3| >= 1b-1022
  # §12.2's second condition: the bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those thirteen gives: the rounding is off by
  # at most u + 2u^2. Gappa's own theorem needs the value strictly above
  # 2^-1022, so it cannot take the step itself.
  /\ bdxcdy -/ bdx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ cdxbdy -/ cdx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxcdy -/ adx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxbdy -/ adx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ U1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ U2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ U3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r1 -/ R1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r2 -/ R2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r3 -/ R3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # q2 and q3 each round a double plus a zero, so each is at least 2^-1022
  # as that double is, and lemma.g gives the same of it. The goals
  # check the first part.
  /\ q2 -/ (|cdxady| + |adxcdy|) in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q3 -/ (|adxbdy| + |bdxady|) in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ adz / |adz| in [-1, 1] /\ bdz / |bdz| in [-1, 1] /\ cdz / |cdz| in [-1, 1]
  /\ bdxcdy / |bdxcdy| in [-1, 1] /\ cdxbdy / |cdxbdy| in [-1, 1]
  /\ adxcdy / |adxcdy| in [-1, 1]
  /\ adxbdy / |adxbdy| in [-1, 1]
->
  |third| >= 1b-1022
}

# How far each rounding is from the identity.
adx / Adx -> 1 + (adx - Adx) / Adx;
adz / Adz -> 1 + (adz - Adz) / Adz;
bdx / Bdx -> 1 + (bdx - Bdx) / Bdx;
bdy / Bdy -> 1 + (bdy - Bdy) / Bdy;
bdz / Bdz -> 1 + (bdz - Bdz) / Bdz;
cdx / Cdx -> 1 + (cdx - Cdx) / Cdx;
cdy / Cdy -> 1 + (cdy - Cdy) / Cdy;
cdz / Cdz -> 1 + (cdz - Cdz) / Cdz;
bdxcdy / (bdx * cdy) -> 1 + (bdxcdy - bdx * cdy) / (bdx * cdy);
cdxbdy / (cdx * bdy) -> 1 + (cdxbdy - cdx * bdy) / (cdx * bdy);
adxcdy / (adx * cdy) -> 1 + (adxcdy - adx * cdy) / (adx * cdy);
adxbdy / (adx * bdy) -> 1 + (adxbdy - adx * bdy) / (adx * bdy);
first / M1 -> 1 + (first - M1) / M1;
second / M2 -> 1 + (second - M2) / M2;
third / M3 -> 1 + (third - M3) / M3;
t1 / U1 -> 1 + (t1 - U1) / U1;
t2 / U2 -> 1 + (t2 - U2) / U2;
t3 / U3 -> 1 + (t3 - U3) / U3;
s12 / S -> 1 + (s12 - S) / S;

# The error is the sum of the six terms' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / P6 -> ((P6 + (y - Ex)) - P6) / P6;
P6 + (y - Ex) -> (c1 + e1) + (c2 + e2) + c3 + (c4 + e4) + (c5 + e5) + c6;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c4 + e4) - c4 -> e4;
(c5 + e5) - c5 -> e5;
e1 / c1 -> (adz / |adz|) * (bdxcdy / |bdxcdy|) * W1;
e2 / c2 -> -((adz / |adz|) * (cdxbdy / |cdxbdy|) * W2);
e4 / c4 -> -((bdz / |bdz|) * (adxcdy / |adxcdy|) * W4);
e5 / c5 -> (cdz / |cdz|) * (adxbdy / |adxbdy|) * W5;

# From the six terms' permanent to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P6) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));

# second and third each round a double less a zero, or a zero less a double,
# and so round to themselves.
cdxady - adxcdy -> -adxcdy;
adxbdy - bdxady -> adxbdy;
