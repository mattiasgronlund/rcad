# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case where the x difference is zero: x cancels, its reach square normal; y has its right half zero; z has its left half zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
# A difference is zero when the origin's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = px; oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
r = rnd(r_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = Fx; fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
# A component with one half zero is the other half, or its negative,
# exactly.
hx = rnd(fy * dz); kx = hx;
hy = rnd(fz * dx); ky = fx * dz;
hz = fx * dy; kz = rnd(fy * dx);
qy = hy; qz = -kz;

# direction_squared, left to right as Rust adds it.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; D = rnd(T);

rr = rnd(r * r);
skin = rnd(rr * D);
sy = rnd(qy * qy); sz = rnd(qz * qz);
s01 = sy;
S012 = s01 + sz; s012 = rnd(S012);
y = s012 - skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
# A reach of two equal halves is twice one of them, which a double
# holds (dbl.g), so it is written unrounded.
# A reach with one half zero is the other half's size, and its square
# is the same double as the component's square s.
nx = |hx| + |kx|; ny = |hy|; nz = |kz|;
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
P = rnd(rnd(rnd(vx + vy) + vz) + skin);

B = 10b-53 + 5b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hx| + |kx|) * (|hx| + |kx|) + |hy| * |hy| + |kz| * |kz| + skin;

# What each rounding multiplied its argument by.
rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx);
rkz = kz / (fy * dx);
rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs012 = s012 / S012;
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rD = D / T;
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One that cancels has no square in y, and its true one is (h a - h b)^2.
# One with a half zero is h^2 K against (h a)^2, or k^2 K against (k b)^2.
Ky = rsy * rs012;
Kz = rsz * rs012;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy);
bz = 1 / (rfy * rkz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One that cancels is off by its whole true square, eC.
# One with a half zero is one term, for the half that is not.
WHy = Ky - ay * ay;
WKz = Kz - bz * bz;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
cHy = |hy| * |hy|;
cKz = |kz| * |kz|;
eCx = -(Cx * Cx);
eHy = hy * hy * WHy;
eKz = kz * kz * WKz;

# The radius term is one piece for each direction component that is not
# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.
cGx = rsk * rr * rD * rt01 * tx;
cGy = rsk * rr * rD * rt01 * ty;
cGz = rsk * rr * rD * tz;
WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;
WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;
WGz = 1 / (rrr * rtz * rsk * rD) - 1;
eGx = cGx * WGx; eGy = cGy * WGy; eGz = cGz * WGz;

# The same permanent, as one block for each component that is not zero,
# its reach squared, and one for the radius term.
P3x = cHx + cKx + cXx; P3y = cHy; P3z = cKz;
SG = cGx + cGy + cGz;
P12 = P3x + P3y + P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Yy = Ky * qy * qy; Yz = Kz * qz * qz;
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the halves that are not zero
  # (cross_product_halves_if_normal), and on the components, the direction
  # components and the radius that are not zero, and the radius term
  # (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |hy| >= 1b-1022 /\ |kx| >= 1b-1022
  /\ |kz| >= 1b-1022
  /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal, and with one half zero v is s.
  # For one that cancels the case says which it is. One below the normals
  # that does not round to zero is at least the smallest subnormal, and
  # tinysq.g and tinyzero.g split them at 2^-1075: at or below, they round
  # to zero.
  /\ |vx| >= 1b-1022 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ hy / |hy| in [-1, 1]
  /\ kz / |kz| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s012 / S012 -> 1 + (s012 - S012) / S012;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
D / T -> 1 + (D - T) / T;
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of the blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. A component with both halves zero has no block: its
# square and its true square are both 0.
# A component that cancels is one block, off by its true square, which is
# (a - b)^2 / 4 of its reach squared, 4 h^2.
# A component with a half zero is one block of one piece.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3x + eCx) + (P3y + eHy) + (P3z + eKz) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy) + (cGz + eGz);
(P3x + eCx) - P3x -> eCx;
(P3y + eHy) - P3y -> eHy;
(P3z + eKz) - P3z -> eKz;
(cGx + eGx) - cGx -> eGx;
(cGy + eGy) - cGy -> eGy;
(cGz + eGz) - cGz -> eGz;
eCx / P3x -> -((hx / |hx|) * (hx / |hx|) * (ax - bx) * (ax - bx) / 4);
eHy / P3y -> (hy / |hy|) * (hy / |hy|) * WHy;
eKz / P3z -> (kz / |kz|) * (kz / |kz|) * WKz;

# From the pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
