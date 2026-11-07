# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# dbl.g: a double added to itself rounds to itself, so the reach |h| + |k| of
# a component whose halves are the same double h is 2 |h| exactly, a double.
# Gappa's float has no top to its exponents, so this says nothing of overflow.
@rnd = float<ieee_64, ne>;
h = rnd(h_);
{ |h| <= 1b1024 /\ |h| >= 1b-1074 -> rnd(|h| + |h|) - (|h| + |h|) in [0, 0] }
|h| + |h| -> 2 * |h|;
