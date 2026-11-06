# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/plane.rs Plane::SIGNED_DISTANCE_ROUNDOFF at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# Plane::signed_distance_to and Plane::SIGNED_DISTANCE_ROUNDOFF,
# crates/rcad-geom/src/plane.rs at c3b2160e: what it returns is within
# 4 * f64::EPSILON * L of the true value, L at least every |p_i| and |offset|.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

# normal.dot(point - origin) - offset. The origin is zero, so subtracting it
# rounds nothing, and nalgebra 0.35's dot on a Vector3 is (a + b) + c.
t0 = rnd(n0 * p0); t1 = rnd(n1 * p1); t2 = rnd(n2 * p2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = s012 - off;
value = rnd(y);

# The true value of the same inputs.
Ex = n0 * p0 + n1 * p1 + n2 * p2 - off;

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * p0); rp1 = t1 / (n1 * p1); rp2 = t2 / (n2 * p2);
rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;

# Each term as value carries it, less the term itself.
W0 = rv * rs012 * rs01 * rp0 - 1;
W1 = rv * rs012 * rs01 * rp1 - 1;
W2 = rv * rs012 * rp2 - 1;
W3 = rv - 1;
e0 = n0 * p0 * W0; e1 = n1 * p1 * W1; e2 = n2 * p2 * W2; e3 = -(off * W3);

# Each term's share of the bound: |n_i| L for a product, and L / 4 for the
# offset, which meets one rounding where a product meets four.
c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; c3 = 1b-2 * L;
D = c0 + c1 + c2 + c3;

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  # No sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074 /\ |y| >= 1b-1074
  # The three products are normal, which signed_distance_to does not check.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # What lemma.g proves each of those three gives.
  /\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the point and the offset.
  /\ L in [1b-1074, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1]
  /\ off / L in [-1, 1]
  # A stored normal has n.n within NORMAL_LENGTH_ROUNDOFF = 64 epsilon of
  # one, so |n0| + |n1| + |n2| <= sqrt(3 (1 + 64 epsilon)) < 1774 / 1024.
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  W0 in ? /\ W2 in ? /\ W3 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

# How far each rounding is from the identity.
t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors, each a fraction of its own
# share of D; add_rr then averages them.
(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> |n0| + |n1| + |n2| + 1b-2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e0 / c0 -> (n0 / |n0|) * (p0 / L) * W0;
e1 / c1 -> (n1 / |n1|) * (p1 / L) * W1;
e2 / c2 -> (n2 / |n2|) * (p2 / L) * W2;
e3 / c3 -> -(4 * (off / L) * W3);
