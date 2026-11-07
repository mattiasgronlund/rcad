# restates: crates/rcad-geom/src/torus.rs on_floats at 37bc60af
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 37bc60af
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs decide at 37bc60af
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
# The case where the y direction component is zero: x has its right half zero; y cancels, its reach square below the normals and not zero; z has its left half zero; hollow normal; P2 normal.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
# A difference is zero when the centre's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = 0; dz = rnd(dz_);
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
# A component with one half zero is the other half, or its negative,
# exactly.
hx = rnd(fy * dz); kx = fz * dy;
hy = rnd(fz * dx); ky = hy;
hz = fx * dy; kz = rnd(fy * dx);
qx = hx; qz = -kz;

# squares and out_and_back, left to right as Rust adds them.
gx = rnd(fx * fx); gy = rnd(fy * fy); gz = rnd(fz * fz);
O01 = gx + gy; o01 = rnd(O01);
O = o01 + gz; oab = rnd(O);

RR = rnd(rg * rg); rr = rnd(tb * tb);
H = RR - rr; hollow = rnd(H);
Bu = oab + hollow; bulge = rnd(Bu);

# axis_squared.
tx = rnd(dx * dx); tz = rnd(dz * dz);
# One square is zero, so axis_squared is the other two's sum rounded once.
T = tx + tz; W = rnd(T);

# across.
# A zero square added to a double leaves that double, which then
# rounds to itself, so a sum is written without its zero squares.
sx = rnd(qx * qx); sz = rnd(qz * qz);
s01 = sx;
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
# A reach of two equal halves is twice one of them, which a double
# holds (464.15's dbl.g), so it is written unrounded.
# A reach with one half zero is the other half's size, and its square
# is the same double as the component's square s.
nx = |hx|; ny = |hy| + |ky|; nz = |kz|;
Xvy = ny * ny;
vx = rnd(nx * nx); vy = rnd(Xvy); vz = rnd(nz * nz);
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
Phatx = W * (Rc * Rc) + F4 * (|hx| * |hx| + vy + |kz| * |kz|);
Dx = F4 * (vy);

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rkz = kz / (fy * dx);
rgx = gx / (fx * fx); rgy = gy / (fy * fy); rgz = gz / (fz * fz);
ro01 = o01 / O01; ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rH = hollow / H; rBu = bulge / Bu;
rtx = tx / (dx * dx); rtz = tz / (dz * dz);
rW = W / T;
rsx = sx / (qx * qx); rsz = sz / (qz * qz);
rS = across / S;
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb); rR = right / (F4 * across);

# The w.w bulge^2 block. W and oab as their pieces, one for each square
# that is not zero, which sum to them; with one, its ratio is direct.
Wx = rW * tx; Wz = rW * tz;
W2 = Wx + Wz;
jx = 1 / (rW * rtx) - 1; jz = 1 / (rW * rtz) - 1;
om = Wt / W;
Ox = ro * ro01 * gx; Oy = ro * ro01 * gy; Oz = ro * gz;
O3 = Ox + Oy + Oz;
zx = 1 / (ro * ro01 * rgx * rfx * rfx) - 1;
zy = 1 / (ro * ro01 * rgy * rfy * rfy) - 1;
zz = 1 / (ro * rgz * rfz * rfz) - 1;
Z = G2 / oab;

# bulge as y carries it is rBu (oab + rH (RR - rr)); the true one is
# Z oab + RR / rRR - rr / rrr. Squared and times W, that is six pieces, one
# for each pair of oab, RR and rr; W is each piece's error over the piece.
KL = rL * rbb * rBu * rBu;
WOO = KL - om * Z * Z;
WRR = KL * rH * rH - om / (rRR * rRR);
Wrr = KL * rH * rH - om / (rrr * rrr);
WOR = KL * rH - om * Z / rRR;
WOr = KL * rH - om * Z / rrr;
WRr = KL * rH * rH - om / (rRR * rrr);
cOO = W * oab * oab; cRR = W * RR * RR; crr = W * rr * rr;
cOR = 2 * W * oab * RR; cOr = 2 * W * oab * rr; cRr = 2 * W * RR * rr;
eOO = cOO * WOO; eRR = cRR * WRR; err = crr * Wrr;
eOR = cOR * WOR; eOr = -(cOr * WOr); eRr = -(cRr * WRr);
AW = cOO + cRR + crr + cOR + cOr + cRr;

# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's
# rounding, m undoes F4's and RR's.
# A component with no square in y has no piece here.
Kx = rR * rS * rsx;
Kz = rR * rS * rsz;
m = 1 / (rF4 * rRR);
ax = 1 / (rfy * rhx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
bz = 1 / (rfy * rkz);
WHx = Kx - m * ax * ax;
WKz = Kz - m * bz * bz;
cHx = F4 * |hx| * |hx|;
cKz = F4 * |kz| * |kz|;
eHx = -(F4 * hx * hx * WHx);
eKz = -(F4 * kz * kz * WKz);
C3x = cHx; C3z = cKz;
CR = C3x + C3z;
Yx = F4 * Kx * qx * qx; Yz = F4 * Kz * qz * qz;
# What y misses of a component that cancels is its whole true term.
eCy = F4 * m * (Cy * Cy);

# The same permanent as the two blocks.
Phat2 = AW + CR;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dz| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  # hollow is a normal number; below the normals it is exact, and that is
  # another case.
  /\ |hollow| >= 1b-1022
  # product_is_normal on the squares of from_centre, of direction and of
  # sideways, and on the halves, that are not zero; on R^2, r^2, bulge^2 and
  # axis_squared bulge^2; and on 4 R^2 across where across is not zero
  # (stays_accurate and the check after it).
  /\ |gx| >= 1b-1022 /\ |gy| >= 1b-1022 /\ |gz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |tz| >= 1b-1022 /\ |sx| >= 1b-1022
  /\ |sz| >= 1b-1022
  /\ |hx| >= 1b-1022 /\ |hy| >= 1b-1022 /\ |ky| >= 1b-1022
  /\ |kz| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022 /\ |right| >= 1b-1022
  # The permanent's products, by monotone rounding: v >= s for a component
  # that does not cancel, and with one half zero v is s; for one that
  # cancels the case says which v is, and 464.14's tinysq.g and tinyzero.g
  # split them at 2^-1075. br >= bulge, as hollow <= RR, so brr >= bb and
  # P1 >= left. spread >= across, so P2 >= right.
  /\ |vx| >= 1b-1022 /\ Xvy <= 1b-1022 /\ vy >= 1b-1074 /\ |vz| >= 1b-1022
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ |P2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hollow -/ H in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gx -/ fx * fx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gy -/ fy * fy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ right -/ F4 * across in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P2 -/ F4 * spread in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ kz / |kz| in [-1, 1]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phatx in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);
gy / (fy * fy) -> 1 + (gy - fy * fy) / (fy * fy);
gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);
o01 / O01 -> 1 + (o01 - O01) / O01;
oab / O -> 1 + (oab - O) / O;
RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);
rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);
hollow / H -> 1 + (hollow - H) / H;
bulge / Bu -> 1 + (bulge - Bu) / Bu;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
W / T -> 1 + (W - T) / T;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
across / S -> 1 + (across - S) / S;
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);
right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);

# w.w over W, and |x|^2 over oab, each averaged over its pieces.
Wt / W -> 1 + ((W2 + (Wt - W)) - W2) / W2;
W2 + (Wt - W) -> (Wx + Wx * jx) + (Wz + Wz * jz);
(Wx + Wx * jx) - Wx -> Wx * jx;
(Wz + Wz * jz) - Wz -> Wz * jz;
G2 / oab -> 1 + ((O3 + (G2 - oab)) - O3) / O3;
O3 + (G2 - oab) -> (Ox + Ox * zx) + (Oy + Oy * zy) + (Oz + Oz * zz);
(Ox + Ox * zx) - Ox -> Ox * zx;
(Oy + Oy * zy) - Oy -> Oy * zy;
(Oz + Oz * zz) - Oz -> Oz * zz;

# The error is the sum of the two blocks' errors, each the sum of its
# pieces' errors, each a fraction of its own piece of the permanent; add_rr
# then averages them.
# What y misses of a component that cancels goes with the piece cOR, and
# is at most a u^2 of it.
(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;
Phat2 + (y - Ex) -> (AW + ((left - Lt) + eCy)) + (CR + (Rt - right - eCy));
AW + ((left - Lt) + eCy) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + (eOR + eCy)) + (cOr + eOr) + (cRr + eRr);
(cOO + eOO) - cOO -> eOO;
(cRR + eRR) - cRR -> eRR;
(crr + err) - crr -> err;
(cOR + (eOR + eCy)) - cOR -> eOR + eCy;
(cOr + eOr) - cOr -> eOr;
(cRr + eRr) - cRr -> eRr;
eOr / cOr -> -WOr;
eRr / cRr -> -WRr;
(eOR + eCy) / cOR -> WOR + eCy / cOR;
eCy / cOR -> 2 * rF4 * m * (ay - by) * (ay - by) * rhy * rhy * (fz * fz / oab) * (dx * dx / W);
fz * fz / oab -> (gz / oab) / rgz;
gz / oab -> 1 / (ro * (ro01 * (gx / gz + gy / gz) + 1));
dx * dx / W -> (tx / W) / rtx;
tx / W -> 1 / (rW * (1 + tz / tx));
CR + (Rt - right - eCy) -> (C3x + eHx) + (C3z + eKz);
(C3x + eHx) - C3x -> eHx;
(C3z + eKz) - C3z -> eKz;
eHx / C3x -> -((hx / |hx|) * (hx / |hx|) * WHx);
eKz / C3z -> -((kz / |kz|) * (kz / |kz|) * WKz);

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat2 / P) * (P / eb);
Phat2 / P -> (Phat2 / Phatx) * (Phatx / P);
Phat2 / Phatx -> 1 / (1 + Dx / Phat2);
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
