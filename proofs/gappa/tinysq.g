# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# tinysq.g: a square of two doubles at or below 2^-1075 rounds to zero, and
# one above it rounds to at least 2^-1074, the smallest subnormal.
@rnd = float<ieee_64, ne>;
g = rnd(g_);
X = g * g;
{ |g| <= 1b1024 /\ |g| >= 1b-1074
  -> (X <= 1b-1075 -> rnd(X) in [0, 0]) /\ (X >= 1b-1075 /\ X <= 1b-1022 -> rnd(X) in [0, 1b-1022]) }
