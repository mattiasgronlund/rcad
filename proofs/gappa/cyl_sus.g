# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at e59fab75
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at e59fab75
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs decide at e59fab75
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case sus: x cancels, its reach square below the normals and not zero; y cancels, its reach square rounding to zero; z cancels, its reach square below the normals and not zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
r = rnd(r_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
hx = rnd(fy * dz); kx = hx;
hy = rnd(fz * dx); ky = hy;
hz = rnd(fx * dy); kz = hz;

# direction_squared, left to right as Rust adds it.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; D = rnd(T);

rr = rnd(r * r);
skin = rnd(rr * D);
# A zero square added to a double leaves that double, which then
# rounds to itself, so a sum is written without its zero squares.
y = -skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
# A reach of two equal halves is twice one of them, which a double
# holds (dbl.g), so it is written unrounded.
nx = |hx| + |kx|; ny = |hy| + |ky|; nz = |hz| + |kz|;
Xvx = nx * nx; Xvy = ny * ny; Xvz = nz * nz;
# A reach square that rounds to zero is written as the 0 it rounds to
# (464.14's tinysq.g), and the sums leave it out.
vx = rnd(Xvx); vz = rnd(Xvz);
P = rnd(rnd(vx + vz) + skin);

B = 10b-53 + 5b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
     + (|hz| + |kz|) * (|hz| + |kz|) + skin;
# The same with v in place of each reach square below the normals that
# does not round to zero, and without each that does, added as P adds.
Phatx = (vx + vz) + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rD = D / T;
rrr = rr / (r * r); rsk = skin / (rr * D);

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

# The radius term is three pieces, one for each direction component's
# square, which sum to skin. Each true piece is r^2 d^2.
cGx = rsk * rr * rD * rt01 * tx;
cGy = rsk * rr * rD * rt01 * ty;
cGz = rsk * rr * rD * tz;
WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;
WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;
WGz = 1 / (rrr * rtz * rsk * rD) - 1;
eGx = cGx * WGx; eGy = cGy * WGy; eGz = cGz * WGz;

# The same permanent, as four blocks of the twelve pieces the error is
# averaged over: each component's reach squared, and the radius term.
P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;
SG = cGx + cGy + cGz;
P12 = P3x + P3y + P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero, and no component but those the case names cancels to
  # zero. For an f64 that is
  # the same as being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the six halves (cross_product_halves_if_normal),
  # and on the three components, the three direction components, the
  # radius and the radius term (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal. For one that cancels the case says
  # which it is. One below the normals that does not round to zero is at
  # least the smallest subnormal, and tinysq.g and tinyzero.g split them
  # at 2^-1075: at or below, they round to zero.
  /\ Xvx <= 1b-1022 /\ vx >= 1b-1074 /\ Xvy <= 1b-1075 /\ Xvz <= 1b-1022 /\ vz >= 1b-1074
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
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1]
  /\ hy / |hy| in [-1, 1]
  /\ hz / |hz| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phatx in ? /\ P in ?
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
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
D / T -> 1 + (D - T) / T;
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of four blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. In blocks so that no hint's identity spans all twelve
# pieces: one hint over all twelve did not finish in 600 s.
# A component that cancels is one block, off by its true square, which is
# (a - b)^2 / 4 of its reach squared, 4 h^2.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3x + eCx) + (P3y + eCy) + (P3z + eCz) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy) + (cGz + eGz);
(P3x + eCx) - P3x -> eCx;
(P3y + eCy) - P3y -> eCy;
(P3z + eCz) - P3z -> eCz;
(cGx + eGx) - cGx -> eGx;
(cGy + eGy) - cGy -> eGy;
(cGz + eGz) - cGz -> eGz;
eCx / P3x -> -((hx / |hx|) * (hx / |hx|) * (ax - bx) * (ax - bx) / 4);
eCy / P3y -> -((hy / |hy|) * (hy / |hy|) * (ay - by) * (ay - by) / 4);
eCz / P3z -> -((hz / |hz|) * (hz / |hz|) * (az - bz) * (az - bz) / 4);

# From the twelve pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
# What rounding a reach square below the normals cost is an amount, and
# eb being normal keeps P large enough that it is small against eb.
Phat / P -> Phatx / P + ((Xvx - vx) + Xvy + (Xvz - vz)) / P;
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
