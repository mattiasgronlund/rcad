# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at eb125ed0
# pmin.g: how small the permanent can be with eb = rnd(B * P) normal, at the
# code's B. P is any non-negative real here, which covers every permanent.
@rnd = float<ieee_64, ne>;
B = 6b-53 + 1b-99;
eb = rnd(B * P);
{ P >= 0 /\ P <= 1b1000 /\ eb >= 1b-1022 /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  -> P in ? }
P -> (eb / B) / (1 + (eb - B * P) / (B * P));
