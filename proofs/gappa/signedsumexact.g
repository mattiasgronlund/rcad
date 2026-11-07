# restates: crates/rcad-geom/src/plane.rs Plane::signed_distance_to at c3b2160e
# restates: crates/rcad-geom/src/point.rs Vector::dot at c3b2160e
# restates: Cargo.lock nalgebra at c3b2160e
# A sum of two doubles whose magnitude is at most 2^-1021 rounds to itself: it
# is a multiple of 2^-1074 below 2^53 of them. Two products below the normals
# (each at most 2^-1022 rounded) give such a sum, and so does a first sum that
# a _big case leaves out, one below 2^-1021.
@rnd = float<ieee_64, ne>;

a = rnd(a_); b = rnd(b_);
c = rnd(c_); d = rnd(d_);

{
     |a| <= 1b-1022 /\ |b| <= 1b-1022
  /\ |c + d| <= 1b-1021
->
     rnd(a + b) - (a + b) in [0, 0]
  /\ rnd(c + d) - (c + d) in [0, 0]
}
