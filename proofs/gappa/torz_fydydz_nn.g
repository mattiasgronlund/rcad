# restates: crates/rcad-geom/src/torus.rs on_floats at 37bc60af
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 37bc60af
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs decide at 37bc60af
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
# The case where the y difference, the y direction component and the z direction component are zero: x has both halves zero; y has its right half zero; z has both halves zero; hollow normal; P2 normal.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
# A difference is zero when the centre's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = py; oz = rnd(oz_);
dx = rnd(dx_); dy = 0; dz = 0;
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = Fy; fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component with one half zero is the other half, or its negative,
# exactly.
hx = fy * dz; kx = fz * dy;
hy = rnd(fz * dx); ky = fx * dz;
hz = fx * dy; kz = fy * dx;
qy = hy;

# squares and out_and_back, left to right as Rust adds them.
gx = rnd(fx * fx); gz = rnd(fz * fz);
# One square is zero, and adding it is exact wherever it falls, so
# out_and_back is the other two's sum rounded once.
O = gx + gz; oab = rnd(O);

RR = rnd(rg * rg); rr = rnd(tb * tb);
H = RR - rr; hollow = rnd(H);
Bu = oab + hollow; bulge = rnd(Bu);

# axis_squared.
tx = rnd(dx * dx);
# Two squares are zero, so axis_squared is the third exactly.
W = tx;

# across.
# A zero square added to a double leaves that double, which then
# rounds to itself, so a sum is written without its zero squares.
sy = rnd(qy * qy);
s01 = sy;
across = s01;

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
ny = |hy|;
vy = rnd(ny * ny);
spread = vy;
br = rnd(rnd(oab + RR) + rr);
brr = rnd(br * br);
P1 = rnd(W * brr);
P2 = rnd(F4 * spread);
P = rnd(P1 + P2);

B = 18b-53 + 7b-101;
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
Phatx = W * (Rc * Rc) + F4 * (|hy| * |hy|);

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfz = fz / Fz;
rhy = hy / (fz * dx);
rgx = gx / (fx * fx); rgz = gz / (fz * fz);
ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rH = hollow / H; rBu = bulge / Bu;
rtx = tx / (dx * dx);
rsy = sy / (qy * qy);
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb); rR = right / (F4 * across);

# The w.w bulge^2 block. W and oab as their pieces, one for each square
# that is not zero, which sum to them; with one, its ratio is direct.
om = Wt / W;
Ox = ro * gx; Oz = ro * gz;
O2 = Ox + Oz;
zx = 1 / (ro * rgx * rfx * rfx) - 1;
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
Ky = rR * rsy;
m = 1 / (rF4 * rRR);
ay = 1 / (rfz * rhy);
WHy = Ky - m * ay * ay;
cHy = F4 * |hy| * |hy|;
eHy = -(F4 * hy * hy * WHy);
C3y = cHy;
CR = C3y;
Yy = F4 * Ky * qy * qy;

# The same permanent as the two blocks.
Phat2 = AW + CR;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  # hollow is a normal number; below the normals it is exact, and that is
  # another case.
  /\ |hollow| >= 1b-1022
  # product_is_normal on the squares of from_centre, of direction and of
  # sideways, and on the halves, that are not zero; on R^2, r^2, bulge^2 and
  # axis_squared bulge^2; and on 4 R^2 across where across is not zero
  # (stays_accurate and the check after it).
  /\ |gx| >= 1b-1022 /\ |gz| >= 1b-1022 /\ |tx| >= 1b-1022
  /\ |sy| >= 1b-1022
  /\ |hy| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022 /\ |right| >= 1b-1022
  # The permanent's products, by monotone rounding: v >= s for a component
  # that does not cancel, and with one half zero v is s; for one that
  # cancels the case says which v is, and 464.14's tinysq.g and tinyzero.g
  # split them at 2^-1075. br >= bulge, as hollow <= RR, so brr >= bb and
  # P1 >= left. spread >= across, so P2 >= right.
  /\ |vy| >= 1b-1022
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ |P2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ hollow -/ H in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gx -/ fx * fx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ right -/ F4 * across in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P2 -/ F4 * spread in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hy / |hy| in [-1, 1]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phatx in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fz / Fz -> 1 + (fz - Fz) / Fz;
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);
gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);
oab / O -> 1 + (oab - O) / O;
RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);
rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);
hollow / H -> 1 + (hollow - H) / H;
bulge / Bu -> 1 + (bulge - Bu) / Bu;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);
right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);

# w.w over W, and |x|^2 over oab, each averaged over its pieces.
Wt / W -> 1 / rtx;
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
CR + (Rt - right) -> (C3y + eHy);
(C3y + eHy) - C3y -> eHy;
eHy / C3y -> -((hy / |hy|) * (hy / |hy|) * WHy);

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phatx / P) * (P / eb);
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
