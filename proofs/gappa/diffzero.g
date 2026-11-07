# restates: crates/rcad-predicates/src/orient3d.rs on_floats at d9f931e8
# diffzero.g: a difference of two doubles that rounds to zero is zero.
@rnd = float<ieee_64, ne>;
x = rnd(x_); z = rnd(z_);
D = x - z;
{ |x| <= 1b1024 /\ |z| <= 1b1024 /\ rnd(D) in [0, 0] -> D in [0, 0] }
