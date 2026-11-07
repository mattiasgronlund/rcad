# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0
# prodzero.g: a product of two doubles, of either sign, that rounds to zero
# is at most 2^-1075 in magnitude.
@rnd = float<ieee_64, ne>;
a = rnd(a_); b = rnd(b_);
X = a * b;
t = rnd(X);
{ |a| <= 1b1024 /\ |b| <= 1b1024 /\ t in [0, 0]
  -> X in [-1b-1075, 1b-1075] }
