# restates: crates/rcad-geom/src/torus.rs on_floats at 37bc60af
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 37bc60af
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 37bc60af
# restates: crates/rcad-predicates/src/cascade.rs decide at 37bc60af
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
# The case where the x difference and the x direction component are zero: x cancels, its reach square normal; y has both halves zero; z has both halves zero; hollow normal; P2 normal.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
# A difference is zero when the centre's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = px; oy = rnd(oy_); oz = rnd(oz_);
dx = 0; dy = rnd(dy_); dz = rnd(dz_);
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = Fx; fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
hx = rnd(fy * dz); kx = hx;
hy = fz * dx; ky = fx * dz;
hz = fx * dy; kz = fy * dx;

# squares and out_and_back, left to right as Rust adds them.
gy = rnd(fy * fy); gz = rnd(fz * fz);
# One square is zero, and adding it is exact wherever it falls, so
# out_and_back is the other two's sum rounded once.
O = gy + gz; oab = rnd(O);

RR = rnd(rg * rg); rr = rnd(tb * tb);
H = RR - rr; hollow = rnd(H);
Bu = oab + hollow; bulge = rnd(Bu);

# axis_squared.
ty = rnd(dy * dy); tz = rnd(dz * dz);
# One square is zero, so axis_squared is the other two's sum rounded once.
T = ty + tz; W = rnd(T);

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
nx = |hx| + |kx|;
vx = rnd(nx * nx);
spread = vx;
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
Phatx = W * (Rc * Rc) + F4 * (vx);
Dx = F4 * (vx);

# What each rounding multiplied its argument by.
rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rgy = gy / (fy * fy); rgz = gz / (fz * fz);
ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rH = hollow / H; rBu = bulge / Bu;
rty = ty / (dy * dy); rtz = tz / (dz * dz);
rW = W / T;
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb);

# The w.w bulge^2 block. W and oab as their pieces, one for each square
# that is not zero, which sum to them; with one, its ratio is direct.
Wy = rW * ty; Wz = rW * tz;
W2 = Wy + Wz;
jy = 1 / (rW * rty) - 1; jz = 1 / (rW * rtz) - 1;
om = Wt / W;
Oy = ro * gy; Oz = ro * gz;
O2 = Oy + Oz;
zy = 1 / (ro * rgy * rfy * rfy) - 1;
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
m = 1 / (rF4 * rRR);
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
# What y misses of a component that cancels is its whole true term.
eCx = F4 * m * (Cx * Cx);

# The same permanent as the two blocks.
Phat2 = AW;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  # hollow is a normal number; below the normals it is exact, and that is
  # another case.
  /\ |hollow| >= 1b-1022
  # product_is_normal on the squares of from_centre, of direction and of
  # sideways, and on the halves, that are not zero; on R^2, r^2, bulge^2 and
  # axis_squared bulge^2; and on 4 R^2 across where across is not zero
  # (stays_accurate and the check after it).
  /\ |gy| >= 1b-1022 /\ |gz| >= 1b-1022 /\ |ty| >= 1b-1022
  /\ |tz| >= 1b-1022
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022
  # The permanent's products, by monotone rounding: v >= s for a component
  # that does not cancel, and with one half zero v is s; for one that
  # cancels the case says which v is, and 464.14's tinysq.g and tinyzero.g
  # split them at 2^-1075. br >= bulge, as hollow <= RR, so brr >= bb and
  # P1 >= left. The case says what P2 is.
  /\ |vx| >= 1b-1022
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ |P2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hollow -/ H in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gy -/ fy * fy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P2 -/ F4 * spread in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phatx in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
gy / (fy * fy) -> 1 + (gy - fy * fy) / (fy * fy);
gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);
oab / O -> 1 + (oab - O) / O;
RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);
rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);
hollow / H -> 1 + (hollow - H) / H;
bulge / Bu -> 1 + (bulge - Bu) / Bu;
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
W / T -> 1 + (W - T) / T;
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);

# w.w over W, and |x|^2 over oab, each averaged over its pieces.
Wt / W -> 1 + ((W2 + (Wt - W)) - W2) / W2;
W2 + (Wt - W) -> (Wy + Wy * jy) + (Wz + Wz * jz);
(Wy + Wy * jy) - Wy -> Wy * jy;
(Wz + Wz * jz) - Wz -> Wz * jz;
G2 / oab -> 1 + ((O2 + (G2 - oab)) - O2) / O2;
O2 + (G2 - oab) -> (Oy + Oy * zy) + (Oz + Oz * zz);
(Oy + Oy * zy) - Oy -> Oy * zy;
(Oz + Oz * zz) - Oz -> Oz * zz;

# The error is the sum of the two blocks' errors, each the sum of its
# pieces' errors, each a fraction of its own piece of the permanent; add_rr
# then averages them.
# What y misses of a component that cancels goes with the piece cOR, and
# is at most a u^2 of it.
(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;
Phat2 + (y - Ex) -> (AW + ((left - Lt) + eCx));
AW + ((left - Lt) + eCx) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + (eOR + eCx)) + (cOr + eOr) + (cRr + eRr);
(cOO + eOO) - cOO -> eOO;
(cRR + eRR) - cRR -> eRR;
(crr + err) - crr -> err;
(cOR + (eOR + eCx)) - cOR -> eOR + eCx;
(cOr + eOr) - cOr -> eOr;
(cRr + eRr) - cRr -> eRr;
eOr / cOr -> -WOr;
eRr / cRr -> -WRr;
(eOR + eCx) / cOR -> WOR + eCx / cOR;
eCx / cOR -> 2 * rF4 * m * (ax - bx) * (ax - bx) * rhx * rhx * (fy * fy / oab) * (dz * dz / W);
fy * fy / oab -> (gy / oab) / rgy;
gy / oab -> 1 / (ro * (1 + gz / gy));
dz * dz / W -> (tz / W) / rtz;
tz / W -> 1 / (rW * (ty / tz + 1));

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat2 / P) * (P / eb);
Phat2 / P -> (Phat2 / Phatx) * (Phatx / P);
Phat2 / Phatx -> 1 / (1 + Dx / Phat2);
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
