# restates: crates/rcad-geom/src/torus.rs on_floats at 37bc60af
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 37bc60af
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs decide at 37bc60af
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
# The case where the y difference is zero: x has its left half zero; y does not cancel; z has its right half zero; hollow below the normals; P2 normal.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
# A difference is zero when the centre's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = py; oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = Fy; fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component with one half zero is the other half, or its negative,
# exactly.
hx = fy * dz; kx = rnd(fz * dy);
hy = rnd(fz * dx); ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = fy * dx;
Qy = hy - ky;
qy = rnd(Qy);
qx = -kx; qz = hz;

# squares and out_and_back, left to right as Rust adds them.
gx = rnd(fx * fx); gz = rnd(fz * fz);
# One square is zero, and adding it is exact wherever it falls, so
# out_and_back is the other two's sum rounded once.
O = gx + gz; oab = rnd(O);

RR = rnd(rg * rg); rr = rnd(tb * tb);
# hollow is below the normals, where it is exact (464.16's sumexact.g).
H = RR - rr; hollow = H;
Bu = oab + hollow; bulge = rnd(Bu);

# axis_squared.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; W = rnd(T);

# across.
sx = rnd(qx * qx); sy = rnd(qy * qy); sz = rnd(qz * qz);
S01 = sx + sy; s01 = rnd(S01);
S = s01 + sz; across = rnd(S);

F4 = rnd(4 * RR);
bb = rnd(bulge * bulge);
left = rnd(W * bb);
right = rnd(F4 * across);
y = left - right;
value = rnd(y);

# The permanent. spread is a sum() from zero, and adding the first square to
# zero is exact, as is adding the reach square of a component with both
# halves zero.
# A reach with one half zero is the other half's size, and its square
# is the same double as the component's square s.
nx = |kx|; ny = rnd(|hy| + |ky|); nz = |hz|;
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
spread = rnd(rnd(vx + vy) + vz);
br = rnd(rnd(oab + RR) + rr);
brr = rnd(br * br);
P1 = rnd(W * brr);
P2 = rnd(F4 * spread);
P = rnd(P1 + P2);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Wt = dx * dx + dy * dy + dz * dz;
G2 = Fx * Fx + Fy * Fy + Fz * Fz;
Bt = G2 + rg * rg - tb * tb;
Lt = Wt * (Bt * Bt);
Rt = 4 * (rg * rg) * (Cx * Cx + Cy * Cy + Cz * Cz);
Ex = Lt - Rt;

# The permanent with no rounding of its own, but with v in place of the
# reach square of a component that cancels and without one that rounds to
# zero, or with P2 in place of the 4 R^2 product where P2 is below the
# normals; Dx is what that adds to the two blocks below.
Rc = oab + RR + rr;
Phatx = W * (Rc * Rc) + F4 * (|kx| * |kx| + (|hy| + |ky|) * (|hy| + |ky|) + |hz| * |hz|);

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfz = fz / Fz;
rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy);
rqy = qy / Qy;
rgx = gx / (fx * fx); rgz = gz / (fz * fz);
ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rBu = bulge / Bu;
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rW = W / T;
rsx = sx / (qx * qx); rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs01 = s01 / S01; rS = across / S;
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb); rR = right / (F4 * across);

# The w.w bulge^2 block. W and oab as their pieces, one for each square
# that is not zero, which sum to them; with one, its ratio is direct.
Wx = rW * rt01 * tx; Wy = rW * rt01 * ty; Wz = rW * tz;
W3 = Wx + Wy + Wz;
jx = 1 / (rW * rt01 * rtx) - 1; jy = 1 / (rW * rt01 * rty) - 1; jz = 1 / (rW * rtz) - 1;
om = Wt / W;
Ox = ro * gx; Oz = ro * gz;
O2 = Ox + Oz;
zx = 1 / (ro * rgx * rfx * rfx) - 1;
zz = 1 / (ro * rgz * rfz * rfz) - 1;
Z = G2 / oab;

# bulge as y carries it is rBu (oab + rH (RR - rr)); the true one is
# Z oab + RR / rRR - rr / rrr. Squared and times W, that is six pieces, one
# for each pair of oab, RR and rr; W is each piece's error over the piece.
# hollow is exact here, so rH is 1 and is left out.
KL = rL * rbb * rBu * rBu;
WOO = KL - om * Z * Z;
WRR = KL - om / (rRR * rRR);
Wrr = KL - om / (rrr * rrr);
WOR = KL - om * Z / rRR;
WOr = KL - om * Z / rrr;
WRr = KL - om / (rRR * rrr);
cOO = W * oab * oab; cRR = W * RR * RR; crr = W * rr * rr;
cOR = 2 * W * oab * RR; cOr = 2 * W * oab * rr; cRr = 2 * W * RR * rr;
eOO = cOO * WOO; eRR = cRR * WRR; err = crr * Wrr;
eOR = cOR * WOR; eOr = -(cOr * WOr); eRr = -(cRr * WRr);
AW = cOO + cRR + crr + cOR + cOr + cRr;

# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's
# rounding, m undoes F4's and RR's.
Kx = rR * rS * rs01 * rsx;
Ky = rR * rS * rs01 * rsy * rqy * rqy;
Kz = rR * rS * rsz;
m = 1 / (rF4 * rRR);
bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz);
WKx = Kx - m * bx * bx;
WHy = Ky - m * ay * ay; WKy = Ky - m * by * by; WXy = Ky - m * ay * by;
WHz = Kz - m * az * az;
cKx = F4 * |kx| * |kx|;
cHy = F4 * |hy| * |hy|; cKy = F4 * |ky| * |ky|; cXy = 2 * F4 * |hy| * |ky|;
cHz = F4 * |hz| * |hz|;
eKx = -(F4 * kx * kx * WKx);
eHy = -(F4 * hy * hy * WHy); eKy = -(F4 * ky * ky * WKy); eXy = 2 * F4 * hy * ky * WXy;
eHz = -(F4 * hz * hz * WHz);
C3x = cKx; C3y = cHy + cKy + cXy; C3z = cHz;
CR = C3x + C3y + C3z;
Yx = F4 * Kx * qx * qx; Yy = F4 * Ky * Qy * Qy; Yz = F4 * Kz * qz * qz;
Zy = F4 * m * (Cy * Cy);

# The same permanent as the two blocks.
Phat2 = AW + CR;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  /\ |qy| >= 1b-1074
  # hollow is below the normals, where it is exact.
  /\ |hollow| <= 1b-1022
  # product_is_normal on the squares of from_centre, of direction and of
  # sideways, and on the halves, that are not zero; on R^2, r^2, bulge^2 and
  # axis_squared bulge^2; and on 4 R^2 across where across is not zero
  # (stays_accurate and the check after it).
  /\ |gx| >= 1b-1022 /\ |gz| >= 1b-1022 /\ |tx| >= 1b-1022
  /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022 /\ |sx| >= 1b-1022
  /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |hy| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kx| >= 1b-1022
  /\ |ky| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022 /\ |right| >= 1b-1022
  # The permanent's products, by monotone rounding: v >= s for a component
  # that does not cancel, and with one half zero v is s; for one that
  # cancels the case says which v is, and 464.14's tinysq.g and tinyzero.g
  # split them at 2^-1075. br >= bulge, as hollow <= RR, so brr >= bb and
  # P1 >= left. spread >= across, so P2 >= right.
  /\ |vx| >= 1b-1022 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ |P2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ gx -/ fx * fx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ right -/ F4 * across in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P2 -/ F4 * spread in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hy / |hy| in [-1, 1] /\ hz / |hz| in [-1, 1]
  /\ kx / |kx| in [-1, 1] /\ ky / |ky| in [-1, 1]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phatx in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fz / Fz -> 1 + (fz - Fz) / Fz;
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
qy / Qy -> 1 + (qy - Qy) / Qy;
gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);
gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);
oab / O -> 1 + (oab - O) / O;
RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);
rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);
bulge / Bu -> 1 + (bulge - Bu) / Bu;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
W / T -> 1 + (W - T) / T;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s01 / S01 -> 1 + (s01 - S01) / S01;
across / S -> 1 + (across - S) / S;
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);
right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);

# w.w over W, and |x|^2 over oab, each averaged over its pieces.
Wt / W -> 1 + ((W3 + (Wt - W)) - W3) / W3;
W3 + (Wt - W) -> (Wx + Wx * jx) + (Wy + Wy * jy) + (Wz + Wz * jz);
(Wx + Wx * jx) - Wx -> Wx * jx;
(Wy + Wy * jy) - Wy -> Wy * jy;
(Wz + Wz * jz) - Wz -> Wz * jz;
G2 / oab -> 1 + ((O2 + (G2 - oab)) - O2) / O2;
O2 + (G2 - oab) -> (Ox + Ox * zx) + (Oz + Oz * zz);
(Ox + Ox * zx) - Ox -> Ox * zx;
(Oz + Oz * zz) - Oz -> Oz * zz;

# The error is the sum of the two blocks' errors, each the sum of its
# pieces' errors, each a fraction of its own piece of the permanent; add_rr
# then averages them.
(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;
Phat2 + (y - Ex) -> (AW + (left - Lt)) + (CR + (Rt - right));
AW + (left - Lt) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + eOR) + (cOr + eOr) + (cRr + eRr);
(cOO + eOO) - cOO -> eOO;
(cRR + eRR) - cRR -> eRR;
(crr + err) - crr -> err;
(cOR + eOR) - cOR -> eOR;
(cOr + eOr) - cOr -> eOr;
(cRr + eRr) - cRr -> eRr;
eOr / cOr -> -WOr;
eRr / cRr -> -WRr;
CR + (Rt - right) -> (C3x + eKx) + (C3y + (Zy - Yy)) + (C3z + eHz);
C3y + (Zy - Yy) -> (cHy + eHy) + (cKy + eKy) + (cXy + eXy);
(C3x + eKx) - C3x -> eKx;
(cHy + eHy) - cHy -> eHy;
(cKy + eKy) - cKy -> eKy;
(cXy + eXy) - cXy -> eXy;
(C3z + eHz) - C3z -> eHz;
eKx / C3x -> -((kx / |kx|) * (kx / |kx|) * WKx);
eHy / cHy -> -((hy / |hy|) * (hy / |hy|) * WHy);
eKy / cKy -> -((ky / |ky|) * (ky / |ky|) * WKy);
eXy / cXy -> (hy / |hy|) * (ky / |ky|) * WXy;
eHz / C3z -> -((hz / |hz|) * (hz / |hz|) * WHz);

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phatx / P) * (P / eb);
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
