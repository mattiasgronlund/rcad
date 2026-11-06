# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at d9f931e8
# lemma.g: a double that rounds to at least MIN_POSITIVE in magnitude is
# off by at most u + 2u^2 relatively, u = 2^-53, even when the exact value
# is below MIN_POSITIVE.
@rnd = float<ieee_64, ne>;
p = rnd(x);
{ |x| <= 1b1024 /\ |p| >= 1b-1022
  -> (p - x) / x in [-1b-53, 1b-53] }
$ x in (-4503599627370497b-1074, 0, 4503599627370497b-1074);
