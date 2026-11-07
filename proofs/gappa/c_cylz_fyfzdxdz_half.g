# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case where the y difference, the z difference, the x direction component and the z direction component are zero: x has both halves zero; y has both halves zero; z has its right half zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
# A difference is zero when the origin's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = py; oz = pz;
dx = 0; dy = rnd(dy_); dz = 0;
r = rnd(r_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = Fy; fz = Fz;

# cross_product_halves: h is each component's left half, k its right half.
# A component with one half zero is the other half, or its negative,
# exactly.
hx = fy * dz; kx = fz * dy;
hy = fz * dx; ky = fx * dz;
hz = rnd(fx * dy); kz = fy * dx;
qz = hz;

# direction_squared, left to right as Rust adds it.
ty = rnd(dy * dy);
t01 = ty;
D = t01;

rr = rnd(r * r);
skin = rnd(rr * D);
sz = rnd(qz * qz);
s012 = sz;
y = s012 - skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
# A reach with one half zero is the other half's size, and its square
# is the same double as the component's square s.
nz = |hz|;
vz = rnd(nz * nz);
P = rnd(vz + skin);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = |hz| * |hz| + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx;
rhz = hz / (fx * dy);
rsz = sz / (qz * qz);
rty = ty / (dy * dy);
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One with a half zero is h^2 K against (h a)^2, or k^2 K against (k b)^2.
Kz = rsz;
az = 1 / (rfx * rhz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One with a half zero is one term, for the half that is not.
WHz = Kz - az * az;
cHz = |hz| * |hz|;
eHz = hz * hz * WHz;

# The radius term is one piece for each direction component that is not
# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.
cGy = rsk * rr * ty;
WGy = 1 / (rrr * rty * rsk) - 1;
eGy = cGy * WGy;

# The same permanent, as one block for each component that is not zero,
# its reach squared, and one for the radius term.
P3z = cHz;
SG = cGy;
P12 = P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Yz = Kz * qz * qz;
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024
  /\ |dy| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074
  /\ |dy| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the halves that are not zero
  # (cross_product_halves_if_normal), and on the components, the direction
  # components and the radius that are not zero, and the radius term
  # (stays_accurate).
  /\ |hz| >= 1b-1022
  /\ |sz| >= 1b-1022
  /\ |ty| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal, and with one half zero v is s.
  /\ |vz| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hz / |hz| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of the blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. A component with both halves zero has no block: its
# square and its true square are both 0.
# A component with a half zero is one block of one piece.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3z + eHz) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGy + eGy);
(P3z + eHz) - P3z -> eHz;
(cGy + eGy) - cGy -> eGy;
eHz / P3z -> (hz / |hz|) * (hz / |hz|) * WHz;

# From the pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
