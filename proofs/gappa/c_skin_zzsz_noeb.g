# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66
# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.
# A case product_is_normal refuses, which on_floats meets only without
# its check. Here:
#   the x gap is zero;
#   the y gap is zero;
#   the z gap has a square below the normals that rounds to a double, not to zero;
#   the radius is zero.
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
# A square below the normals is X. One that rounds to zero is written 0,
# which tinysq.g proves rnd(X) is.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = Gx; gy = Gy; gz = rnd(Gz);
Xz = gz * gz;
sx = gx * gx; sy = gy * gy; sz = rnd(Xz);
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

B = 6b-53 + 3b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;

# The permanent with no rounding of its own.
Phat = sx + sy + sz + rr;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cz| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gz| >= 1b-1074
  # Each square below the normals. Of two doubles that are not zero it is
  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and
  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and
  # above, to at least 2^-1074.
  /\ Xz in [1b-2148, 1b-1022] /\ sz >= 1b-1074 /\ |gz| <= 1b-511
->
  eb in ? /\ (y - Ex) / eb in [-1, 1]
}


# A zero square added to a double leaves that double, which then rounds
# to itself.
s01 + sz -> sz;
s012 + rr -> s012;
