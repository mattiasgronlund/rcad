# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: Cargo.lock nalgebra at ba0a654e
# Rounding the sum or the difference of two doubles that is not zero
# multiplies it by a ratio within u of 1: a sum below the normals is a
# double already. pointsub.py writes each rounded sum that way.
@rnd = float<ieee_64, ne>;

a = rnd(a_); b = rnd(b_);
S = a + b; s = rnd(S);
T = a - b; t = rnd(T);

{
     |a| <= 1b1024 /\ |b| <= 1b1024
  /\ |S| >= 1b-1074 /\ |T| >= 1b-1074
->
  (s - S) / S in [-1b-53, 1b-53] /\ (t - T) / T in [-1b-53, 1b-53]
}
