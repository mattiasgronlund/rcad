# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: Cargo.lock nalgebra at ba0a654e
# Rounding a real P to an f64 is off by at most u (|P| + 2^-1022): here for
# |P| at most 2^-1022, below the normals, where the slip is at most 2^-1075.
# mixedn.g is the rest.
@rnd = float<ieee_64, ne>;

q = rnd(P);

{
  |P| <= 1b-1022
->
  (q - P) / (|P| + 1b-1022) in [-1b-53, 1b-53]
}
