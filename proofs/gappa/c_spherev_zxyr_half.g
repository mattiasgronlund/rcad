# restates: crates/rcad-predicates/src/surface.rs on_floats at 3d60b0de
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 3d60b0de
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 3d60b0de
# restates: crates/rcad-predicates/src/cascade.rs decide at 3d60b0de
# HowFarOut::on_floats and its ERROR_BOUND, the sphere of surface.rs' tests.
# The claim is on value, the double on_floats returns, not on y.
# The case where the x gap and the y gap and the radius are zero.
@rnd = float<ieee_64, ne>;

# The point, the centre and the radius, each an f64. A gap is zero when
# the centre's coordinate is the same double as the point's (464.11's
# diffzero.g), and a zero radius is the double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
cx = px; cy = py; cz = rnd(cz_);
r = 0;

# What on_floats computes, line for line. A zero, a square of one and a
# sum of two round to themselves (464.11's refl.g), so they are left
# unrounded.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = Gx; gy = Gy; gz = rnd(Gz);
sx = gx * gx; sy = gy * gy; sz = rnd(gz * gz);
rr = r * r;
S01 = sx + sy;
s01 = S01;
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

# What each rounding multiplied its argument by.
rgz = gz / Gz;
rpz = sz / (gz * gz);
rs012 = s012 / S012;
rv = value / y;

# What each of the four terms, as value carries it, is off by: e is that
# amount and W the same over the term's square. The radius's term is
# subtracted, so its W is the other way round.
Wz = rs012 * rv - 1 / (rgz * rgz * rpz);
ez = sz * Wz;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cz| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gz| >= 1b-1074
  # product_is_normal on the squares left.
  /\ |sz| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ sz -/ gz * gz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # y is not zero. Where it is, value is zero too, and the claim on y is
  # the claim on value.
  /\ |y| >= 1b-1074
->
  (value - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (value - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
gz / Gz -> 1 + (gz - Gz) / Gz;
sz / (gz * gz) -> 1 + (sz - gz * gz) / (gz * gz);
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the terms' errors, each a fraction of its own
# piece of the permanent; add_rr then averages them.
(value - Ex) / Phat -> ((Phat + (value - Ex)) - Phat) / Phat;
Phat + (value - Ex) -> sx + sy + (sz + ez) + rr;
(sz + ez) - sz -> ez;

# From the permanent to the one on_floats rounds, and on to eb.
(value - Ex) / eb -> ((value - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));

# A zero square added to a double leaves that double, which then rounds
# to itself.
s01 + sz -> sz;
s012 + rr -> s012;
s012 - rr -> s012;
