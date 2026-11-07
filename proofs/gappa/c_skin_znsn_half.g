# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66
# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.
# A case product_is_normal refuses, which on_floats meets only without
# its check. Here:
#   the x gap is zero;
#   the z gap has a square below the normals that rounds to a double, not to zero.
@rnd = float<ieee_64, ne>;

# The point, the centre and the radius, each an f64. A gap is zero when
# the centre's coordinate is the same double as the point's (464.11's
# diffzero.g), and a zero radius is the double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
cx = px; cy = rnd(cy_); cz = rnd(cz_);
r = rnd(r_);

# What on_floats computes, line for line. A zero, a square of one and a
# sum of two round to themselves (464.11's refl.g), so they are left
# unrounded.
# A square below the normals is X. One that rounds to zero is written 0,
# which tinysq.g proves rnd(X) is.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = Gx; gy = rnd(Gy); gz = rnd(Gz);
Xz = gz * gz;
sx = gx * gx; sy = rnd(gy * gy); sz = rnd(Xz);
rr = rnd(r * r);
S01 = sx + sy;
s01 = rnd(S01);
S012 = s01 + sz;
s012 = rnd(S012);
y = s012 - rr;
value = rnd(y);

# The permanent sums the same three squares in the same order as value
# does, so its first two sums are s01 and s012.
P = rnd(s012 + rr);

B = 1b-54;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;

# The permanent with no rounding of its own.
Phat = sx + sy + sz + rr;
# The same with X in place of each square below the normals that does
# not round to zero, and without those that do.
Phatx = sy + Xz + rr;

# What each rounding multiplied its argument by.
rgy = gy / Gy; rgz = gz / Gz;
rpy = sy / (gy * gy);
rpr = rr / (r * r);
rs01 = s01 / S01; rs012 = s012 / S012;

# What each of the four terms, as y carries it, is off by: e is that
# amount and W the same over the term's square. The radius's term is
# subtracted, so its W is the other way round.
# For a square below the normals e and W are of X, and d is what
# rounding X cost, as an amount. One that rounds to zero is off by
# its whole true square, which is its d.
Wy = rs01 * rs012 - 1 / (rgy * rgy * rpy);
Wz = rs012 - 1 / (rgz * rgz);
Wr = 1 / rpr - 1;
ey = sy * Wy; ez = Xz * Wz; er = rr * Wr;
dz = (sz - Xz) * rs012;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cy| <= 1b1024 /\ |cz| <= 1b1024 /\ |r| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gy| >= 1b-1074 /\ |gz| >= 1b-1074 /\ |r| >= 1b-1074
  # product_is_normal on the squares that pass it.
  /\ |sy| >= 1b-1022 /\ |rr| >= 1b-1022
  # Each square below the normals. Of two doubles that are not zero it is
  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and
  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and
  # above, to at least 2^-1074.
  /\ Xz in [1b-2148, 1b-1022] /\ sz >= 1b-1074 /\ |gz| <= 1b-511
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ sy -/ gy * gy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
->
  (ey + ez + er) / Phatx in ? /\ (dz) in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
gy / Gy -> 1 + (gy - Gy) / Gy;
gz / Gz -> 1 + (gz - Gz) / Gz;
sy / (gy * gy) -> 1 + (sy - gy * gy) / (gy * gy);
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The relative part is averaged over Phatx as in skin.g. What rounding a
# square below the normals cost is an amount, and eb being normal keeps
# P large enough that it is small against eb.
(y - Ex) / eb -> ((ey + ez + er) / Phatx) * (Phatx / P) * (P / eb) + (dz) / eb;
(ey + ez + er) / Phatx -> ((Phatx + (ey + ez + er)) - Phatx) / Phatx;
Phatx + (ey + ez + er) -> (sy + ey) + (Xz + ez) + (rr + er);
(sy + ey) - sy -> ey;
(Xz + ez) - Xz -> ez;
(rr + er) - rr -> er;
Phatx / P -> Phat / P + (Xz - sz) / P;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));

# A zero square added to a double leaves that double, which then rounds
# to itself.
sx + sy -> sy;
