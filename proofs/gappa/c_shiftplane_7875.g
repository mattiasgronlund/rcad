# restates: crates/rcad-geom/src/plane.rs Plane::transformed at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::holding at c3b2160e
# restates: crates/rcad-geom/src/half_space.rs HalfSpace::shifted at c3b2160e
# restates: crates/rcad-geom/src/moved_corner.rs SURFACE_AFTER_SHIFTING_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs Isometry::apply_to_vector at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs Isometry::translation_part at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs turned_by at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::named_by at c3b2160e
# restates: crates/rcad-geom/src/isometry.rs TurnOfACube::turned at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::transformed by a shift with no turn, HalfSpace::shifted, and
# SURFACE_AFTER_SHIFTING_ROUNDOFF, crates/rcad-geom/src at c3b2160e: the new
# offset is within 8 * f64::EPSILON * |n| * L of offset + n.t, L at least
# every |t_i| and |offset| / |n|.
@rnd = float<ieee_64, ne>;

# The normal, the shift and the offset, each an f64.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

# self.offset + normal.dot(shift). A movement with no turn hands the normal
# back bit for bit, and nalgebra 0.35's dot on a Vector3 is (a + b) + c.
t0 = rnd(n0 * d0); t1 = rnd(n1 * d1); t2 = rnd(n2 * d2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = off + s012;
value = rnd(y);

# The true new offset of the same inputs, and the normal's true length.
Ex = off + n0 * d0 + n1 * d1 + n2 * d2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * d0); rp1 = t1 / (n1 * d1); rp2 = t2 / (n2 * d2);
rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;

# Each term as value carries it, less the term itself.
W0 = rv * rs012 * rs01 * rp0 - 1;
W1 = rv * rs012 * rs01 * rp1 - 1;
W2 = rv * rs012 * rp2 - 1;
W3 = rv - 1;
e0 = n0 * d0 * W0; e1 = n1 * d1 * W1; e2 = n2 * d2 * W2; e3 = off * W3;

# Each term's share of the bound: |n_i| L for a product, and |n| L / 4 for
# the offset, which meets one rounding where a product meets four.
c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; c3 = 1b-2 * (N * L);
D = c0 + c1 + c2 + c3;

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d0| <= 1b1024 /\ |d1| <= 1b1024 /\ |d2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |d0| >= 1b-1074 /\ |d1| >= 1b-1074 /\ |d2| >= 1b-1074
  # No sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074 /\ |y| >= 1b-1074
  # The three products are normal, which neither caller checks.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # What lemma.g proves each of those three gives.
  /\ t0 -/ n0 * d0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * d1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * d2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the shift and the offset over |n|.
  /\ L in [1b-1074, 1b1024]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1]
  /\ off / (N * L) in [-1, 1]
  # |n0| + |n1| + |n2| <= sqrt(3) |n| < 1774 / 1024 |n|, by Cauchy-Schwarz.
  /\ (|n0| + |n1| + |n2|) / N in [0, 1774b-10]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  W0 in ? /\ W2 in ? /\ W3 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-63b-56, 63b-56]
}

# How far each rounding is from the identity.
t0 / (n0 * d0) -> 1 + (t0 - n0 * d0) / (n0 * d0);
t1 / (n1 * d1) -> 1 + (t1 - n1 * d1) / (n1 * d1);
t2 / (n2 * d2) -> 1 + (t2 - n2 * d2) / (n2 * d2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors, each a fraction of its own
# share of D; add_rr then averages them.
(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (|n0| + |n1| + |n2|) / N + 1b-2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e0 / c0 -> (n0 / |n0|) * (d0 / L) * W0;
e1 / c1 -> (n1 / |n1|) * (d1 / L) * W1;
e2 / c2 -> (n2 / |n2|) * (d2 / L) * W2;
e3 / c3 -> 4 * (off / (N * L)) * W3;
