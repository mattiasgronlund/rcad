# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0
# c_prodzero.g: prodzero.g asking for half its bound, which must fail: 2^-1075
# is a product of two doubles and rounds to zero, ties going to even.
@rnd = float<ieee_64, ne>;
a = rnd(a_); b = rnd(b_);
X = a * b;
t = rnd(X);
{ |a| <= 1b1024 /\ |b| <= 1b1024 /\ t in [0, 0]
  -> X in [-1b-1076, 1b-1076] }
