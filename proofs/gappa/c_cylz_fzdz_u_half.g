# restates: crates/rcad-geom/src/cylinder.rs on_floats at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at cc9adf3f
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at cc9adf3f
# restates: crates/rcad-predicates/src/cascade.rs decide at cc9adf3f
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The case where the z difference and the z direction component are zero: x has both halves zero; y has both halves zero; z cancels, its reach square rounding to zero.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
# A difference is zero when the origin's coordinate is the same double as
# the point's (464.11's diffzero.g), and a zero direction component is the
# double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = pz;
dx = rnd(dx_); dy = rnd(dy_); dz = 0;
r = rnd(r_);

# What on_floats computes, line for line. A zero, a product of a zero, and
# a double with a zero added round to themselves (464.11's refl.g), so they
# are left unrounded, and a zero added to a double is left out.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = Fz;

# cross_product_halves: h is each component's left half, k its right half.
# A component that cancels has the same double for both halves, so k
# is written h, and its q and s are zero and drop out.
hx = fy * dz; kx = fz * dy;
hy = fz * dx; ky = fx * dz;
hz = rnd(fx * dy); kz = hz;

# direction_squared, left to right as Rust adds it.
tx = rnd(dx * dx); ty = rnd(dy * dy);
T01 = tx + ty; t01 = rnd(T01);
D = t01;

rr = rnd(r * r);
skin = rnd(rr * D);
y = -skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
# A reach of two equal halves is twice one of them, which a double
# holds (dbl.g), so it is written unrounded.
nz = |hz| + |kz|;
Xvz = nz * nz;
# A reach square that rounds to zero is written as the 0 it rounds to
# (464.14's tinysq.g), and the sums leave it out.
P = skin;

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hz| + |kz|) * (|hz| + |kz|) + skin;
# The same with v in place of each reach square below the normals that
# does not round to zero, and without each that does, added as P adds.
Phatx = skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy;
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rtx = tx / (dx * dx); rty = ty / (dy * dy);
rt01 = t01 / T01;
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
# One that cancels has no square in y, and its true one is (h a - h b)^2.
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
# One that cancels is off by its whole true square, eC.
cHz = |hz| * |hz|; cKz = |kz| * |kz|; cXz = 2 * |hz| * |kz|;
eCz = -(Cz * Cz);

# The radius term is one piece for each direction component that is not
# zero, its square, and the pieces sum to skin. Each true piece is r^2 d^2.
cGx = rsk * rr * rt01 * tx;
cGy = rsk * rr * rt01 * ty;
WGx = 1 / (rrr * rtx * rsk * rt01) - 1;
WGy = 1 / (rrr * rty * rsk * rt01) - 1;
eGx = cGx * WGx; eGy = cGy * WGy;

# The same permanent, as one block for each component that is not zero,
# its reach squared, and one for the radius term.
P3z = cHz + cKz + cXz;
SG = cGx + cGy;
P12 = P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero but those the case names, and no component cancels
  # to zero but one the case says does. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074
  /\ |r| >= 1b-1074
  # product_is_normal on the halves that are not zero
  # (cross_product_halves_if_normal), and on the components, the direction
  # components and the radius that are not zero, and the radius term
  # (stays_accurate).
  /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The reach square of a component that does not cancel is normal: |q| <= n,
  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=
  # rnd(q * q) = s, which is normal, and with one half zero v is s.
  # For one that cancels the case says which it is. One below the normals
  # that does not round to zero is at least the smallest subnormal, and
  # tinysq.g and tinyzero.g split them at 2^-1075: at or below, they round
  # to zero.
  /\ Xvz <= 1b-1075
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  # k is h in a component that cancels, so its line is h's against k's product.
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hz / |hz| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phatx in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
t01 / T01 -> 1 + (t01 - T01) / T01;
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of the blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. A component with both halves zero has no block: its
# square and its true square are both 0.
# A component that cancels is one block, off by its true square, which is
# (a - b)^2 / 4 of its reach squared, 4 h^2.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3z + eCz) + (SG + (Gt - skin));
SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy);
(P3z + eCz) - P3z -> eCz;
(cGx + eGx) - cGx -> eGx;
(cGy + eGy) - cGy -> eGy;
eCz / P3z -> -((hz / |hz|) * (hz / |hz|) * (az - bz) * (az - bz) / 4);

# From the pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
# What rounding a reach square below the normals cost is an amount, and
# eb being normal keeps P large enough that it is small against eb.
Phat / P -> Phatx / P + (Xvz) / P;
Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
