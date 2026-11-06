# restates: crates/rcad-geom/src/lib.rs NEAREST_F64_POINT_ROUNDOFF at c3b2160e
# NEAREST_F64_POINT_ROUNDOFF, crates/rcad-geom/src/lib.rs at c3b2160e:
# the nearest f64 triple to a point x lies within f64::EPSILON * L of it,
# L at least the largest |x_i|.
@rnd = float<ieee_64, ne>;

p0 = rnd(x0); p1 = rnd(x1); p2 = rnd(x2);

# Each coordinate's slip, per unit of L.
e0 = (p0 - x0) / L; e1 = (p1 - x1) / L; e2 = (p2 - x2) / L;

{
  # L is at least every |x_i|, and is a normal number no larger than the
  # largest f64.
     x0 / L in [-1, 1] /\ x1 / L in [-1, 1] /\ x2 / L in [-1, 1]
  /\ L in [1b-1074, 1b1024]
->
  e0 in ? /\ e1 in ? /\ e2 in ?
  /\ sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 1b-52]
}

# Above 2^-1022 a slip is relative to x_i, and x_i is at most L. Below it the
# slip is at most 2^-1075, and L is at least 2^-1022.
(p0 - x0) / L -> ((p0 - x0) / x0) * (x0 / L);
(p1 - x1) / L -> ((p1 - x1) / x1) * (x1 / L);
(p2 - x2) / L -> ((p2 - x2) / x2) * (x2 / L);
$ x0 in (-1b-1022, 1b-1022);
$ x1 in (-1b-1022, 1b-1022);
$ x2 in (-1b-1022, 1b-1022);
