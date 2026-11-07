# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case where the x direction component is zero: x cancels, its reach square below the normals and not zero; y has its left half zero; z has its right half zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
# A difference is zero when the origin's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = 0; dy = rnd(dy_); dz = rnd(dz_);
r = rnd(r_);

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
hx = rnd(fy * dz); kx = hx;
hy = fz * dx; ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = fy * dx;
qy = -ky; qz = hz;

# direction_squared, left to right as Rust adds it.
ty = rnd(dy * dy); tz = rnd(dz * dz);
t01 = ty;
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
nx = |hx| + |kx|; ny = |ky|; nz = |hz|;
Xvx = nx * nx;
# A reach square that rounds to zero is written as the 0 it rounds to
# (464.14's tinysq.g), and the sums leave it out.
vx = rnd(Xvx); vy = rnd(ny * ny); vz = rnd(nz * nz);
P = rnd(rnd(rnd(vx + vy) + vz) + skin);

B = 10b-53 + 5b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hx| + |kx|) * (|hx| + |kx|) + |ky| * |ky| + |hz| * |hz| + skin;
# The same with v in place of each reach square below the normals that
# does not round to zero, and without each that does, added as P adds.
Phatx = ((vx + |ky| * |ky|) + |hz| * |hz|) + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rky = ky / (fx * dz);
rhz = hz / (fx * dy);
rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs012 = s012 / S012;
rty = ty / (dy * dy); rtz = tz / (dz * dz);
rD = D / T;
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One that cancels has no square in y, and its true one is (h a - h b)^2.
# One with a half zero is h^2 K against (h a)^2, or k^2 K against (k b)^2.
Ky = rsy * rs012;
Kz = rsz * rs012;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
by = 1 / (rfx * rky);
az = 1 / (rfx * rhz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One that cancels is off by its whole true square, eC.
# One with a half zero is one term, for the half that is not.
WKy = Ky - by * by;
WHz = Kz - az * az;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
cKy = |ky| * |ky|;
cHz = |hz| * |hz|;
eCx = -(Cx * Cx);
eKy = ky * ky * WKy;
eHz = hz * hz * WHz;

# The radius term is one piece for each direction component that is not
# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.
cGy = rsk * rr * rD * ty;
cGz = rsk * rr * rD * tz;
WGy = 1 / (rrr * rty * rsk * rD) - 1;
WGz = 1 / (rrr * rtz * rsk * rD) - 1;
eGy = cGy * WGy; eGz = cGz * WGz;

# The same permanent, as one block for each component that is not zero,
# its reach squared, and one for the radius term.
P3x = cHx + cKx + cXx; P3y = cKy; P3z = cHz;
SG = cGy + cGz;
P12 = P3x + P3y + P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Yy = Ky * qy * qy; Yz = Kz * qz * qz;
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the halves that are not zero
  # (cross_product_halves_if_normal), and on the components, the direction
  # components and the radius that are not zero, and the radius term
  # (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kx| >= 1b-1022
  /\ |ky| >= 1b-1022
  /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal, and with one half zero v is s.
  # For one that cancels the case says which it is. One below the normals
  # that does not round to zero is at least the smallest subnormal, and
  # tinysq.g and tinyzero.g split them at 2^-1075: at or below, they round
  # to zero.
  /\ Xvx <= 1b-1022 /\ vx >= 1b-1074 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ hz / |hz| in [-1, 1]
  /\ ky / |ky| in [-1, 1]
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
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s012 / S012 -> 1 + (s012 - S012) / S012;
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
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
P12 + (y - Ex) -> (P3x + eCx) + (P3y + eKy) + (P3z + eHz) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGy + eGy) + (cGz + eGz);
(P3x + eCx) - P3x -> eCx;
(P3y + eKy) - P3y -> eKy;
(P3z + eHz) - P3z -> eHz;
(cGy + eGy) - cGy -> eGy;
(cGz + eGz) - cGz -> eGz;
eCx / P3x -> -((hx / |hx|) * (hx / |hx|) * (ax - bx) * (ax - bx) / 4);
eKy / P3y -> (ky / |ky|) * (ky / |ky|) * WKy;
eHz / P3z -> (hz / |hz|) * (hz / |hz|) * WHz;

# From the pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
# What rounding a reach square below the normals cost is an amount, and
# eb being normal keeps P large enough that it is small against eb.
Phat / P -> Phatx / P + ((Xvx - vx)) / P;
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
