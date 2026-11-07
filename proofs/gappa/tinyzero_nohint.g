# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# tinyzero.g without its one hint, to read what Gappa's warning on that hint means.
@rnd = float<ieee_64, ne>;
g = rnd(g_);
X = g * g;
s = rnd(X);
{ |g| <= 1b1024 /\ |g| >= 1b-1074 /\ X <= 1b-1022 /\ s in [0, 0] -> X in [0, 1b-1075] }
