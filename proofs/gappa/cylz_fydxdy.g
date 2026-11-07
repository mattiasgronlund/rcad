# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case where the y difference, the x direction component and the y direction component are zero: x has both halves zero; y has its left half zero; z has both halves zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
# A difference is zero when the origin's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = py; oz = rnd(oz_);
dx = 0; dy = 0; dz = rnd(dz_);
r = rnd(r_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = Fy; fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
# A component with one half zero is the other half, or its negative,
# exactly.
hx = fy * dz; kx = fz * dy;
hy = fz * dx; ky = rnd(fx * dz);
hz = fx * dy; kz = fy * dx;
qy = -ky;

# direction_squared, left to right as Rust adds it.
tz = rnd(dz * dz);
D = tz;

rr = rnd(r * r);
skin = rnd(rr * D);
sy = rnd(qy * qy);
s01 = sy;
s012 = s01;
y = s012 - skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
# A reach with one half zero is the other half's size, and its square
# is the same double as the component's square s.
ny = |ky|;
vy = rnd(ny * ny);
P = rnd(vy + skin);

B = 10b-53 + 5b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = |ky| * |ky| + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfz = fz / Fz;
rky = ky / (fx * dz);
rsy = sy / (qy * qy);
rtz = tz / (dz * dz);
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One with a half zero is h^2 K against (h a)^2, or k^2 K against (k b)^2.
Ky = rsy;
by = 1 / (rfx * rky);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One with a half zero is one term, for the half that is not.
WKy = Ky - by * by;
cKy = |ky| * |ky|;
eKy = ky * ky * WKy;

# The radius term is one piece for each direction component that is not
# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.
cGz = rsk * rr * tz;
WGz = 1 / (rrr * rtz * rsk) - 1;
eGz = cGz * WGz;

# The same permanent, as one block for each component that is not zero,
# its reach squared, and one for the radius term.
P3y = cKy;
SG = cGz;
P12 = P3y + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Yy = Ky * qy * qy;
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the halves that are not zero
  # (cross_product_halves_if_normal), and on the components, the direction
  # components and the radius that are not zero, and the radius term
  # (stays_accurate).
  /\ |ky| >= 1b-1022
  /\ |sy| >= 1b-1022
  /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal, and with one half zero v is s.
  /\ |vy| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ ky / |ky| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fz / Fz -> 1 + (fz - Fz) / Fz;
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of the blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. A component with both halves zero has no block: its
# square and its true square are both 0.
# A component with a half zero is one block of one piece.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3y + eKy) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGz + eGz);
(P3y + eKy) - P3y -> eKy;
(cGz + eGz) - cGz -> eGz;
eKy / P3y -> (ky / |ky|) * (ky / |ky|) * WKy;

# From the pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
