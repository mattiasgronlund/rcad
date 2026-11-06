# restates: crates/rcad-geom/src/isometry.rs Isometry::apply_to_point at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs turned_by at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::named_by at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::turned at c3b2160e
# restates: crates/rcad-geom/src/sphere.rs Sphere::transformed at c3b2160e
# restates: crates/rcad-geom/src/sphere.rs Sphere::holding at c3b2160e
# restates: crates/rcad-geom/src/point.rs Axis::transformed at c3b2160e
# restates: crates/rcad-geom/src/point.rs Axis::holding at c3b2160e
# restates: crates/rcad-geom/src/moved_corner.rs SURFACE_AFTER_SHIFTING_ROUNDOFF at c3b2160e
# Isometry::apply_to_point with no turn, as Sphere::transformed and
# Axis::transformed use it, and SURFACE_AFTER_SHIFTING_ROUNDOFF,
# crates/rcad-geom/src at c3b2160e: the shifted centre is within
# 8 * f64::EPSILON * L of c + t, L at least every |c_i| and |t_i|.
@rnd = float<ieee_64, ne>;

# The centre and the shift, each an f64.
c0 = rnd(c0_); c1 = rnd(c1_); c2 = rnd(c2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);

# turned_by hands the centre back bit for bit when there is no turn, and the
# translation is one addition per coordinate.
x0 = c0 + d0; x1 = c1 + d1; x2 = c2 + d2;
p0 = rnd(x0); p1 = rnd(x1); p2 = rnd(x2);

# Each coordinate's slip, per unit of L.
e0 = (p0 - x0) / L; e1 = (p1 - x1) / L; e2 = (p2 - x2) / L;

{
  # L is at least every |c_i| and |t_i|, and need not be a normal number.
     c0 / L in [-1, 1] /\ c1 / L in [-1, 1] /\ c2 / L in [-1, 1]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1]
  /\ L in [1b-1074, 1b1024]
->
  e0 in ? /\ e1 in ? /\ e2 in ?
  /\ sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 435b-60]
}

# A slip is relative to x_i, and x_i is at most 2L. Gappa needs no split at
# 2^-1022: a sum of two f64 that lands below it is exact.
(p0 - x0) / L -> ((p0 - x0) / x0) * (x0 / L);
(p1 - x1) / L -> ((p1 - x1) / x1) * (x1 / L);
(p2 - x2) / L -> ((p2 - x2) / x2) * (x2 / L);
x0 / L -> c0 / L + d0 / L;
x1 / L -> c1 / L + d1 / L;
x2 / L -> c2 / L + d2 / L;
