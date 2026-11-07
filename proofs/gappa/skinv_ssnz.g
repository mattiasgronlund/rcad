# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66
# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.
# The claim is on value, the double on_floats returns, not on y.
# A case product_is_normal refuses, which on_floats meets only without
# its check. Here:
#   the x gap has a square below the normals that rounds to a double, not to zero;
#   the y gap has a square below the normals that rounds to a double, not to zero;
#   the radius is zero.
@rnd = float<ieee_64, ne>;

# The point, the centre and the radius, each an f64. A gap is zero when
# the centre's coordinate is the same double as the point's (464.11's
# diffzero.g), and a zero radius is the double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
cx = rnd(cx_); cy = rnd(cy_); cz = rnd(cz_);
r = 0;

# What on_floats computes, line for line. A zero, a square of one and a
# sum of two round to themselves (464.11's refl.g), so they are left
# unrounded.
# A square below the normals is X. One that rounds to zero is written 0,
# which tinysq.g proves rnd(X) is.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = rnd(Gx); gy = rnd(Gy); gz = rnd(Gz);
Xx = gx * gx; Xy = gy * gy;
sx = rnd(Xx); sy = rnd(Xy); sz = rnd(gz * gz);
rr = r * r;
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
Phatx = Xx + Xy + sz;

# What each rounding multiplied its argument by.
rgx = gx / Gx; rgy = gy / Gy; rgz = gz / Gz;
rpz = sz / (gz * gz);
rs01 = s01 / S01; rs012 = s012 / S012;
rv = value / y;

# What each of the four terms, as value carries it, is off by: e is that
# amount and W the same over the term's square. The radius's term is
# subtracted, so its W is the other way round.
# For a square below the normals e and W are of X, and d is what
# rounding X cost, as an amount. One that rounds to zero is off by
# its whole true square, which is its d.
Wx = rs01 * rs012 * rv - 1 / (rgx * rgx);
Wy = rs01 * rs012 * rv - 1 / (rgy * rgy);
Wz = rs012 * rv - 1 / (rgz * rgz * rpz);
ex = Xx * Wx; ey = Xy * Wy; ez = sz * Wz;
dx = (sx - Xx) * (rs01 * rs012 * rv); dy = (sy - Xy) * (rs01 * rs012 * rv);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024 /\ |cz| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gx| >= 1b-1074 /\ |gy| >= 1b-1074 /\ |gz| >= 1b-1074
  # product_is_normal on the squares that pass it.
  /\ |sz| >= 1b-1022
  # Each square below the normals. Of two doubles that are not zero it is
  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and
  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and
  # above, to at least 2^-1074.
  /\ Xx in [1b-2148, 1b-1022] /\ sx >= 1b-1074 /\ |gx| <= 1b-511
  /\ Xy in [1b-2148, 1b-1022] /\ sy >= 1b-1074 /\ |gy| <= 1b-511
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ sz -/ gz * gz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # y is not zero. Where it is, value is zero too, and the claim on y is
  # the claim on value.
  /\ |y| >= 1b-1074
->
  (ex + ey + ez) / Phatx in ? /\ (dx + dy) in ? /\ P in ?
  /\ (value - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
gx / Gx -> 1 + (gx - Gx) / Gx;
gy / Gy -> 1 + (gy - Gy) / Gy;
gz / Gz -> 1 + (gz - Gz) / Gz;
sz / (gz * gz) -> 1 + (sz - gz * gz) / (gz * gz);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The relative part is averaged over Phatx as in skin.g. What rounding a
# square below the normals cost is an amount, and eb being normal keeps
# P large enough that it is small against eb.
(value - Ex) / eb -> ((ex + ey + ez) / Phatx) * (Phatx / P) * (P / eb) + (dx + dy) / eb;
(ex + ey + ez) / Phatx -> ((Phatx + (ex + ey + ez)) - Phatx) / Phatx;
Phatx + (ex + ey + ez) -> (Xx + ex) + (Xy + ey) + (sz + ez);
(Xx + ex) - Xx -> ex;
(Xy + ey) - Xy -> ey;
(sz + ez) - sz -> ez;
Phatx / P -> Phat / P + ((Xx - sx) + (Xy - sy)) / P;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));

# A zero square added to a double leaves that double, which then rounds
# to itself.
s012 + rr -> s012;
s012 - rr -> s012;
