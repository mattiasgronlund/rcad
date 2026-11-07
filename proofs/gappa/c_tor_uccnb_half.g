# restates: crates/rcad-geom/src/torus.rs on_floats at 4be293a5
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 4be293a5
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 4be293a5
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 4be293a5
# restates: crates/rcad-predicates/src/cascade.rs decide at 4be293a5
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
# The case uccnb: x cancels, its reach square rounding to zero; y cancels, its reach square normal; z cancels, its reach square normal; hollow normal; P2 below the normals or zero.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
hx = rnd(fy * dz); kx = hx;
hy = rnd(fz * dx); ky = hy;
hz = rnd(fx * dy); kz = hz;

# squares and out_and_back, left to right as Rust adds them.
gx = rnd(fx * fx); gy = rnd(fy * fy); gz = rnd(fz * fz);
O01 = gx + gy; o01 = rnd(O01);
O = o01 + gz; oab = rnd(O);

RR = rnd(rg * rg); rr = rnd(tb * tb);
H = RR - rr; hollow = rnd(H);
Bu = oab + hollow; bulge = rnd(Bu);

# axis_squared.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; W = rnd(T);

# across.
# A zero square added to a double leaves that double, which then
# rounds to itself, so a sum is written without its zero squares.
# All three cancel, so across and right are zero, and y is left.

F4 = rnd(4 * RR);
bb = rnd(bulge * bulge);
left = rnd(W * bb);
y = left;
value = rnd(y);

# The permanent. spread is a sum() from zero, and adding the first square to
# zero is exact.
# A reach of two equal halves is twice one of them, which a double
# holds (464.15's dbl.g), so it is written unrounded.
nx = |hx| + |kx|; ny = |hy| + |ky|; nz = |hz| + |kz|;
Xvx = nx * nx;
# A reach square that rounds to zero is written as the 0 it rounds to
# (464.14's tinysq.g), and the sums leave it out.
vy = rnd(ny * ny); vz = rnd(nz * nz);
spread = rnd(vy + vz);
br = rnd(rnd(oab + RR) + rr);
brr = rnd(br * br);
P1 = rnd(W * brr);
XP2 = F4 * spread; P2 = rnd(XP2);
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

# The permanent with no rounding of its own.
Rc = oab + RR + rr;
Phat = W * (Rc * Rc)
     + F4 * ((|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
             + (|hz| + |kz|) * (|hz| + |kz|));
# The same with P2 in place of the 4 R^2 product, as P adds them; Dx is
# what that adds to the two blocks below.
Phatx = W * (Rc * Rc) + P2;
Dx = P2;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rgx = gx / (fx * fx); rgy = gy / (fy * fy); rgz = gz / (fz * fz);
ro01 = o01 / O01; ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rH = hollow / H; rBu = bulge / Bu;
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rW = W / T;
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb);

# The w.w bulge^2 block. W and oab as their three pieces, which sum to them.
Wx = rW * rt01 * tx; Wy = rW * rt01 * ty; Wz = rW * tz;
W3 = Wx + Wy + Wz;
jx = 1 / (rW * rt01 * rtx) - 1; jy = 1 / (rW * rt01 * rty) - 1; jz = 1 / (rW * rtz) - 1;
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
A6 = cOO + cRR + crr + cOR + cOr + cRr;

# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's
# rounding, m undoes F4's and RR's.
# A component that cancels has no square in y, and is not in it.
m = 1 / (rF4 * rRR);
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);
# What y misses of a component that cancels is its whole true term, and
# EC is their sum.
eCx = F4 * m * (Cx * Cx); eCy = F4 * m * (Cy * Cy); eCz = F4 * m * (Cz * Cz);
EC = eCx + eCy + eCz;

# The same permanent as the two blocks.
Phat2 = A6;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero, and no component but those the case names cancels to
  # zero. For an f64 that is the same as being at least the smallest
  # subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  # hollow is a normal number; below the normals it is exact, and that is
  # another case.
  /\ |hollow| >= 1b-1022
  # product_is_normal on the three squares of from_centre, of direction and
  # of sideways, the six halves, R^2, r^2, bulge^2, axis_squared bulge^2 and
  # 4 R^2 across (stays_accurate and the check after it).
  /\ |gx| >= 1b-1022 /\ |gy| >= 1b-1022 /\ |gz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022
  # The permanent's products, by monotone rounding: v >= s for a component
  # that does not cancel; for one that does the case says which v is, and
  # 464.14's tinysq.g and tinyzero.g split them at 2^-1075. br >= bulge, as
  # hollow <= RR, so brr >= bb and P1 >= left. The case says what P2 is.
  /\ Xvx <= 1b-1075 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ XP2 <= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hollow -/ H in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gx -/ fx * fx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gy -/ fy * fy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phatx in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
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
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
W / T -> 1 + (W - T) / T;
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);

# w.w over W, and |x|^2 over oab, each averaged over three pieces.
Wt / W -> 1 + ((W3 + (Wt - W)) - W3) / W3;
W3 + (Wt - W) -> (Wx + Wx * jx) + (Wy + Wy * jy) + (Wz + Wz * jz);
(Wx + Wx * jx) - Wx -> Wx * jx;
(Wy + Wy * jy) - Wy -> Wy * jy;
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
Phat2 + (y - Ex) -> (A6 + ((left - Lt) + EC));
A6 + ((left - Lt) + EC) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + (eOR + EC)) + (cOr + eOr) + (cRr + eRr);
(cOO + eOO) - cOO -> eOO;
(cRR + eRR) - cRR -> eRR;
(crr + err) - crr -> err;
(cOR + (eOR + EC)) - cOR -> eOR + EC;
(cOr + eOr) - cOr -> eOr;
(cRr + eRr) - cRr -> eRr;
eOr / cOr -> -WOr;
eRr / cRr -> -WRr;
(eOR + EC) / cOR -> WOR + EC / cOR;
EC / cOR -> eCx / cOR + eCy / cOR + eCz / cOR;
eCx / cOR -> 2 * rF4 * m * (ax - bx) * (ax - bx) * rhx * rhx * (fy * fy / oab) * (dz * dz / W);
fy * fy / oab -> (gy / oab) / rgy;
gy / oab -> 1 / (ro * (ro01 * (gx / gy + 1) + gz / gy));
dz * dz / W -> (tz / W) / rtz;
tz / W -> 1 / (rW * (rt01 * (tx / tz + ty / tz) + 1));
eCy / cOR -> 2 * rF4 * m * (ay - by) * (ay - by) * rhy * rhy * (fz * fz / oab) * (dx * dx / W);
fz * fz / oab -> (gz / oab) / rgz;
gz / oab -> 1 / (ro * (ro01 * (gx / gz + gy / gz) + 1));
dx * dx / W -> (tx / W) / rtx;
tx / W -> 1 / (rW * (rt01 * (1 + ty / tx) + tz / tx));
eCz / cOR -> 2 * rF4 * m * (az - bz) * (az - bz) * rhz * rhz * (fx * fx / oab) * (dy * dy / W);
fx * fx / oab -> (gx / oab) / rgx;
gx / oab -> 1 / (ro * (ro01 * (1 + gy / gx) + gz / gx));
dy * dy / W -> (ty / W) / rty;
ty / W -> 1 / (rW * (rt01 * (tx / ty + 1) + tz / ty));

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat2 / P) * (P / eb);
Phat2 / P -> (Phat2 / Phatx) * (Phatx / P);
Phat2 / Phatx -> 1 / (1 + Dx / Phat2);
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
