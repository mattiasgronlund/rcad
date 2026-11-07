# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: Cargo.lock nalgebra at ba0a654e
# Rounding a real P to an f64 is off by at most u (|P| + 2^-1022): here for
# |P| at least 2^-1022, where the slip is relative. mixeds.g is the rest.
@rnd = float<ieee_64, ne>;

q = rnd(P);

{
     |P| <= 1b1024 /\ |P| >= 1b-1022
  /\ P / |P| in [-1, 1]
->
  (q - P) / (|P| + 1b-1022) in [-1b-53, 1b-53]
}

(q - P) / (|P| + 1b-1022) -> ((q - P) / |P|) * (1 / (1 + 1b-1022 / |P|));
(q - P) / |P| -> ((q - P) / P) * (P / |P|);
