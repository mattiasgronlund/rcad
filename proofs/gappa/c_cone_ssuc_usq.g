# restates: crates/rcad-geom/src/cone.rs on_floats at e59fab75
# restates: crates/rcad-geom/src/cone.rs cross_product_halves_if_normal at e59fab75
# restates: crates/rcad-geom/src/cone.rs cross_product_halves at e59fab75
# restates: crates/rcad-geom/src/cone.rs SIDE_OF_CONE_ERROR_BOUND at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs decide at e59fab75
# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.
# The case ssuc: x cancels, its reach square below the normals and not zero; y cancels, its reach square below the normals and not zero; z cancels, its reach square rounding to zero; up cancels, R2 and A normal.
@rnd = float<ieee_64, ne>;

# The point, the apex, the axis direction and the slope, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
sl = rnd(sl_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
hx = rnd(fy * dz); kx = hx;
hy = rnd(fz * dx); ky = hy;
hz = rnd(fx * dy); kz = hz;

# up_halves, and up, summed left to right as Rust adds them.
ux = rnd(fx * dx); uy = rnd(fy * dy); uz = rnd(fz * dz);
U01 = ux + uy; u01 = rnd(U01);
# up cancels: u01 + uz is zero, so up, w and along are zero, and y is
# the squares alone.

# A zero square added to a double leaves that double, which then
# rounds to itself, so a sum is written without its zero squares.
ss = rnd(sl * sl);
y = 0;
value = rnd(y);

# The permanent: each half's reach squared, then the slope squared times the
# reach of up squared.
# A reach of two equal halves is twice one of them, which a double
# holds (dbl.g), so it is written unrounded.
nx = |hx| + |kx|; ny = |hy| + |ky|; nz = |hz| + |kz|;
Xvx = nx * nx; Xvy = ny * ny; Xvz = nz * nz;
# A reach square that rounds to zero is written as the 0 it rounds to
# (464.14's tinysq.g), and the sums leave it out.
vx = rnd(Xvx); vy = rnd(Xvy);
R01 = rnd(|ux| + |uy|); R = rnd(R01 + |uz|);
R2 = rnd(R * R);
XA = ss * R2; A = rnd(XA);
P = rnd(rnd(vx + vy) + A);

B = 1b-106;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Dt = Fx * dx + Fy * dy + Fz * dz;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - sl * sl * (Dt * Dt);

# The permanent with no rounding of its own.
G = |ux| + |uy| + |uz|;
# Where up cancels, its axial term is XA, which R2 need not keep near
# ss G^2.
Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
     + (|hz| + |kz|) * (|hz| + |kz|) + XA;
# The same with v in place of each reach square below the normals that
# does not round to zero, and without each that does, added as P adds.
Phatx = (vx + vy) + XA;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rux = ux / (fx * dx); ruy = uy / (fy * dy); ruz = uz / (fz * dz);
ru01 = u01 / U01;
rss = ss / (sl * sl);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One that cancels has no square in y, and its true one is (h a - h b)^2.
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One that cancels is off by its whole true square, eC.
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
cHy = |hy| * |hy|; cKy = |ky| * |ky|; cXy = 2 * |hy| * |ky|;
cHz = |hz| * |hz|; cKz = |kz| * |kz|; cXz = 2 * |hz| * |kz|;
eCx = -(Cx * Cx);
eCy = -(Cy * Cy);
eCz = -(Cz * Cz);

# Where up cancels, y has no axial term, and the true one, sl^2 Dt^2, is
# all error. Since u01 + uz is zero, Dt is what the roundings left: (e - g)
# u for each half, e undoing the roundings of f and of the product and g
# that of the first sum, so g is ru01, ru01 and 1. Dt is averaged over G,
# and the error over XA.
ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);
Dx = ex - ru01; Dy = ey - ru01; Dz = ez - 1;
D3 = Dx * ux + Dy * uy + Dz * uz;

# The same permanent, as four blocks: each component's reach squared, and
# the axial term.
P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;
P15 = P3x + P3y + P3z + XA;

# Each component's square as y carries it, and the axial term as Ex has it.
At = sl * sl * (Dt * Dt);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |sl| <= 1b1024
  # No factor is zero, and no component but those the case names cancels to
  # zero. For an f64 that is the same as being at least the smallest
  # subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |sl| >= 1b-1074
  # up cancels: u01 + uz is zero, so u01 is -uz, which is normal.
  /\ u01 + uz in [0, 0] /\ |u01| >= 1b-1022
  # product_is_normal on the six halves (cross_product_halves_if_normal),
  # and on the three components, the three up halves, the slope, up and the
  # axial term (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |ux| >= 1b-1022 /\ |uy| >= 1b-1022 /\ |uz| >= 1b-1022
  /\ |ss| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal. For one that cancels the case says
  # which it is. One below the normals that does not round to zero is at
  # least the smallest subnormal, and tinysq.g and tinyzero.g split them
  # at 2^-1075: at or below, they round to zero.
  /\ Xvx <= 1b-1022 /\ vx >= 1b-1074 /\ Xvy <= 1b-1022 /\ vy >= 1b-1074 /\ Xvz <= 1b-1075
  # Where up cancels, the case says what R2 and A are.
  /\ |R2| >= 1b-1022 /\ |A| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ux -/ fx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ uy -/ fy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ uz -/ fz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ u01 -/ U01 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ss -/ sl * sl in [-4503599627370497b-105, 4503599627370497b-105]
  /\ R2 -/ R * R in [-4503599627370497b-105, 4503599627370497b-105]
  /\ A -/ XA in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1]
  /\ hy / |hy| in [-1, 1]
  /\ hz / |hz| in [-1, 1]
  /\ ux / |ux| in [-1, 1] /\ uy / |uy| in [-1, 1] /\ uz / |uz| in [-1, 1]
->
  (y - Ex) / P15 in ? /\ P -/ Phatx in ? /\ P in ?
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
ux / (fx * dx) -> 1 + (ux - fx * dx) / (fx * dx);
uy / (fy * dy) -> 1 + (uy - fy * dy) / (fy * dy);
uz / (fz * dz) -> 1 + (uz - fz * dz) / (fz * dz);
u01 / U01 -> 1 + (u01 - U01) / U01;
ss / (sl * sl) -> 1 + (ss - sl * sl) / (sl * sl);

# The error is the sum of four blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. In blocks so that no hint's identity spans every piece,
# as cyl.g learned.
# A component that cancels is one block, off by its true square, which is
# (a - b)^2 / 4 of its reach squared, 4 h^2.
(y - Ex) / P15 -> ((P15 + (y - Ex)) - P15) / P15;
P15 + (y - Ex) -> (P3x + eCx) + (P3y + eCy) + (P3z + eCz) + (XA + At);
(P3x + eCx) - P3x -> eCx;
(P3y + eCy) - P3y -> eCy;
(P3z + eCz) - P3z -> eCz;
(XA + At) - XA -> At;
eCx / P3x -> -((hx / |hx|) * (hx / |hx|) * (ax - bx) * (ax - bx) / 4);
eCy / P3y -> -((hy / |hy|) * (hy / |hy|) * (ay - by) * (ay - by) / 4);
eCz / P3z -> -((hz / |hz|) * (hz / |hz|) * (az - bz) * (az - bz) / 4);
# The axial error over XA: Dt over G, twice, G^2 over R2, and the slope's
# rounding. Dt over G is the average of what each half's roundings left.
At / XA -> (Dt / G) * (Dt / G) * ((G * G) / R2) / rss;
Dt / G -> D3 / G + (u01 + uz) / G;
D3 / G -> ((G + D3) - G) / G;
G + D3 -> (|ux| + Dx * ux) + (|uy| + Dy * uy) + (|uz| + Dz * uz);
(|ux| + Dx * ux) - |ux| -> Dx * ux;
(|uy| + Dy * uy) - |uy| -> Dy * uy;
(|uz| + Dz * uz) - |uz| -> Dz * uz;
(Dx * ux) / |ux| -> Dx * (ux / |ux|);
(Dy * uy) / |uy| -> Dy * (uy / |uy|);
(Dz * uz) / |uz| -> Dz * (uz / |uz|);
(G * G) / R2 -> 1 / (1 + (R2 - G * G) / (G * G));

# From the blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P15) * (Phat / P) * (P / eb);
# What rounding a square below the normals cost is an amount, and
# eb being normal keeps P large enough that it is small against eb.
Phat / P -> Phatx / P + ((Xvx - vx) + (Xvy - vy) + Xvz) / P;
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
