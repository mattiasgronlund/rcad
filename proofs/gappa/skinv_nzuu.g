# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66
# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.
# The claim is on value, the double on_floats returns, not on y.
# A case product_is_normal refuses, which on_floats meets only without
# its check. Here:
#   the y gap is zero;
#   the z gap has a square below the normals that rounds to zero;
#   the radius has a square below the normals that rounds to zero.
@rnd = float<ieee_64, ne>;

# The point, the centre and the radius, each an f64. A gap is zero when
# the centre's coordinate is the same double as the point's (464.11's
# diffzero.g), and a zero radius is the double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
cx = rnd(cx_); cy = py; cz = rnd(cz_);
r = rnd(r_);

# What on_floats computes, line for line. A zero, a square of one and a
# sum of two round to themselves (464.11's refl.g), so they are left
# unrounded.
# A square below the normals is X. One that rounds to zero is written 0,
# which tinysq.g proves rnd(X) is.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = rnd(Gx); gy = Gy; gz = rnd(Gz);
Xz = gz * gz;
sx = rnd(gx * gx); sy = gy * gy; sz = 0;
Xr = r * r; rr = 0;
S01 = sx + sy;
s01 = rnd(S01);
S012 = s01 + sz;
s012 = rnd(S012);
y = s012 - rr;
value = rnd(y);

# The permanent sums the same three squares in the same order as value
# does, so its first two sums are s01 and s012.
P = rnd(s012 + rr);

B = 6b-53 + 3b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;

# The permanent with no rounding of its own.
Phat = sx + sy + sz + rr;
# The same with X in place of each square below the normals that does
# not round to zero, and without those that do.
Phatx = sx;

# What each rounding multiplied its argument by.
rgx = gx / Gx; rgz = gz / Gz;
rpx = sx / (gx * gx);
rs01 = s01 / S01; rs012 = s012 / S012;
rv = value / y;

# What each of the four terms, as value carries it, is off by: e is that
# amount and W the same over the term's square. The radius's term is
# subtracted, so its W is the other way round.
# For a square below the normals e and W are of X, and d is what
# rounding X cost, as an amount. One that rounds to zero is off by
# its whole true square, which is its d.
Wx = rs01 * rs012 * rv - 1 / (rgx * rgx * rpx);
ex = sx * Wx;
dz = -(Gz * Gz); dr = Xr;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cz| <= 1b1024 /\ |r| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gx| >= 1b-1074 /\ |gz| >= 1b-1074 /\ |r| >= 1b-1074
  # product_is_normal on the squares that pass it.
  /\ |sx| >= 1b-1022
  # Each square below the normals. Of two doubles that are not zero it is
  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and
  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and
  # above, to at least 2^-1074.
  /\ Xz in [1b-2148, 1b-1075] /\ |gz| <= 1b-511
  /\ Xr in [1b-2148, 1b-1075] /\ |r| <= 1b-511
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ sx -/ gx * gx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # y is not zero. Where it is, value is zero too, and the claim on y is
  # the claim on value.
  /\ |y| >= 1b-1074
->
  (ex) / Phatx in ? /\ (dz + dr) in ? /\ P in ?
  /\ (value - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
gx / Gx -> 1 + (gx - Gx) / Gx;
gz / Gz -> 1 + (gz - Gz) / Gz;
sx / (gx * gx) -> 1 + (sx - gx * gx) / (gx * gx);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;
Gz * Gz -> Xz / (rgz * rgz);

# The relative part is averaged over Phatx as in skin.g. What rounding a
# square below the normals cost is an amount, and eb being normal keeps
# P large enough that it is small against eb.
(value - Ex) / eb -> ((ex) / Phatx) * (Phatx / P) * (P / eb) + (dz + dr) / eb;
(ex) / Phatx -> ((Phatx + (ex)) - Phatx) / Phatx;
Phatx + (ex) -> (sx + ex);
(sx + ex) - sx -> ex;
Phatx / P -> Phat / P;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));

# A zero square added to a double leaves that double, which then rounds
# to itself.
sx + sy -> sx;
s01 + sz -> s01;
s012 + rr -> s012;
s012 - rr -> s012;
