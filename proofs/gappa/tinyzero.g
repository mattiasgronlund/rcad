# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# tinyzero.g: a square of two doubles below the normals that rounds to zero
# is at most 2^-1075. With tinysq.g that splits the squares below the normals
# in two at 2^-1075: at or below it they round to zero, above it they do not.
@rnd = float<ieee_64, ne>;
g = rnd(g_);
X = g * g;
s = rnd(X);
{ |g| <= 1b1024 /\ |g| >= 1b-1074 /\ X <= 1b-1022 /\ s in [0, 0] -> X in [0, 1b-1075] }
X -> s - (s - X);
