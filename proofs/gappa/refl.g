# restates: crates/rcad-predicates/src/orient3d.rs on_floats at d9f931e8
# refl.g: x - x is zero, and so are its rounding and a product with it.
@rnd = float<ieee_64, ne>;
x = rnd(x_); y = rnd(y_);
a = x - x;
p = a * y;
{ |x| <= 1b1024 /\ |y| <= 1b1024 -> a in [0, 0] /\ rnd(a) in [0, 0] /\ rnd(p) in [0, 0] }
