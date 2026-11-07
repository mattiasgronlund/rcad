# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case where the y difference and the y direction component are zero: x has both halves zero; y cancels, its reach square normal; z has both halves zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
# A difference is zero when the origin's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = py; oz = rnd(oz_);
dx = rnd(dx_); dy = 0; dz = rnd(dz_);
r = rnd(r_);

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

# direction_squared, left to right as Rust adds it.
tx = rnd(dx * dx); tz = rnd(dz * dz);
t01 = tx;
T = t01 + tz; D = rnd(T);

rr = rnd(r * r);
skin = rnd(rr * D);
y = -skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
# A reach of two equal halves is twice one of them, which a double
# holds (dbl.g), so it is written unrounded.
ny = |hy| + |ky|;
vy = rnd(ny * ny);
P = rnd(vy + skin);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hy| + |ky|) * (|hy| + |ky|) + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfz = fz / Fz;
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rtx = tx / (dx * dx); rtz = tz / (dz * dz);
rD = D / T;
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One that cancels has no square in y, and its true one is (h a - h b)^2.
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One that cancels is off by its whole true square, eC.
cHy = |hy| * |hy|; cKy = |ky| * |ky|; cXy = 2 * |hy| * |ky|;
eCy = -(Cy * Cy);

# The radius term is one piece for each direction component that is not
# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.
cGx = rsk * rr * rD * tx;
cGz = rsk * rr * rD * tz;
WGx = 1 / (rrr * rtx * rsk * rD) - 1;
WGz = 1 / (rrr * rtz * rsk * rD) - 1;
eGx = cGx * WGx; eGz = cGz * WGz;

# The same permanent, as one block for each component that is not zero,
# its reach squared, and one for the radius term.
P3y = cHy + cKy + cXy;
SG = cGx + cGz;
P12 = P3y + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the halves that are not zero
  # (cross_product_halves_if_normal), and on the components, the direction
  # components and the radius that are not zero, and the radius term
  # (stays_accurate).
  /\ |hy| >= 1b-1022 /\ |ky| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal, and with one half zero v is s.
  # For one that cancels the case says which it is. One below the normals
  # that does not round to zero is at least the smallest subnormal, and
  # tinysq.g and tinyzero.g split them at 2^-1075: at or below, they round
  # to zero.
  /\ |vy| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hy / |hy| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fz / Fz -> 1 + (fz - Fz) / Fz;
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
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
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3y + eCy) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGx + eGx) + (cGz + eGz);
(P3y + eCy) - P3y -> eCy;
(cGx + eGx) - cGx -> eGx;
(cGz + eGz) - cGz -> eGz;
eCy / P3y -> -((hy / |hy|) * (hy / |hy|) * (ay - by) * (ay - by) / 4);

# From the pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
