# restates: crates/rcad-geom/src/torus.rs on_floats at 37bc60af
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 37bc60af
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs decide at 37bc60af
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
# The case where the y difference and the y direction component are zero: x has both halves zero; y cancels, its reach square below the normals and not zero; z has both halves zero; hollow below the normals; P2 normal.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
# A difference is zero when the centre's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = py; oz = rnd(oz_);
dx = rnd(dx_); dy = 0; dz = rnd(dz_);
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = Fy; fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
hx = fy * dz; kx = fz * dy;
hy = rnd(fz * dx); ky = hy;
hz = fx * dy; kz = fy * dx;

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
tx = rnd(dx * dx); tz = rnd(dz * dz);
# One square is zero, so axis_squared is the other two's sum rounded once.
T = tx + tz; W = rnd(T);

# across.
# A zero square added to a double leaves that double, which then
# rounds to itself, so a sum is written without its zero squares.
# No component has a square, so across and right are zero, and y is left.

F4 = rnd(4 * RR);
bb = rnd(bulge * bulge);
left = rnd(W * bb);
y = left;
value = rnd(y);

# The permanent. spread is a sum() from zero, and adding the first square to
# zero is exact, as is adding the reach square of a component with both
# halves zero.
# A reach of two equal halves is twice one of them, which a double
# holds (464.15's dbl.g), so it is written unrounded.
ny = |hy| + |ky|;
Xvy = ny * ny;
vy = rnd(Xvy);
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
Phatx = W * (Rc * Rc) + F4 * (vy);
Dx = F4 * (vy);

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfz = fz / Fz;
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rgx = gx / (fx * fx); rgz = gz / (fz * fz);
ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rBu = bulge / Bu;
rtx = tx / (dx * dx); rtz = tz / (dz * dz);
rW = W / T;
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb);

# The w.w bulge^2 block. W and oab as their pieces, one for each square
# that is not zero, which sum to them; with one, its ratio is direct.
Wx = rW * tx; Wz = rW * tz;
W2 = Wx + Wz;
jx = 1 / (rW * rtx) - 1; jz = 1 / (rW * rtz) - 1;
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
# A component with no square in y has no piece here.
m = 1 / (rF4 * rRR);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
# What y misses of a component that cancels is its whole true term.
eCy = F4 * m * (Cy * Cy);

# The same permanent as the two blocks.
Phat2 = AW;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dz| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  # hollow is below the normals, where it is exact.
  /\ |hollow| <= 1b-1022
  # product_is_normal on the squares of from_centre, of direction and of
  # sideways, and on the halves, that are not zero; on R^2, r^2, bulge^2 and
  # axis_squared bulge^2; and on 4 R^2 across where across is not zero
  # (stays_accurate and the check after it).
  /\ |gx| >= 1b-1022 /\ |gz| >= 1b-1022 /\ |tx| >= 1b-1022
  /\ |tz| >= 1b-1022
  /\ |hy| >= 1b-1022 /\ |ky| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022
  # The permanent's products, by monotone rounding: v >= s for a component
  # that does not cancel, and with one half zero v is s; for one that
  # cancels the case says which v is, and 464.14's tinysq.g and tinyzero.g
  # split them at 2^-1075. br >= bulge, as hollow <= RR, so brr >= bb and
  # P1 >= left. The case says what P2 is.
  /\ Xvy <= 1b-1022 /\ vy >= 1b-1074
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ |P2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ gx -/ fx * fx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P2 -/ F4 * spread in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phatx in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fz / Fz -> 1 + (fz - Fz) / Fz;
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);
gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);
oab / O -> 1 + (oab - O) / O;
RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);
rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);
bulge / Bu -> 1 + (bulge - Bu) / Bu;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
W / T -> 1 + (W - T) / T;
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);

# w.w over W, and |x|^2 over oab, each averaged over its pieces.
Wt / W -> 1 + ((W2 + (Wt - W)) - W2) / W2;
W2 + (Wt - W) -> (Wx + Wx * jx) + (Wz + Wz * jz);
(Wx + Wx * jx) - Wx -> Wx * jx;
(Wz + Wz * jz) - Wz -> Wz * jz;
G2 / oab -> 1 + ((O2 + (G2 - oab)) - O2) / O2;
O2 + (G2 - oab) -> (Ox + Ox * zx) + (Oz + Oz * zz);
(Ox + Ox * zx) - Ox -> Ox * zx;
(Oz + Oz * zz) - Oz -> Oz * zz;

# The error is the sum of the two blocks' errors, each the sum of its
# pieces' errors, each a fraction of its own piece of the permanent; add_rr
# then averages them.
# What y misses of a component that cancels goes with the piece cOR, and
# is at most a u^2 of it.
(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;
Phat2 + (y - Ex) -> (AW + ((left - Lt) + eCy));
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
gz / oab -> 1 / (ro * (gx / gz + 1));
dx * dx / W -> (tx / W) / rtx;
tx / W -> 1 / (rW * (1 + tz / tx));

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat2 / P) * (P / eb);
Phat2 / P -> (Phat2 / Phatx) * (Phatx / P);
Phat2 / Phatx -> 1 / (1 + Dx / Phat2);
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
