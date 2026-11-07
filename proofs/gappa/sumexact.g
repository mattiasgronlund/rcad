# restates: crates/rcad-geom/src/cone.rs on_floats at e59fab75
# A sum of two doubles that is below the normals is a double: both are
# multiples of 2^-1074, so their sum is too, and below 2^-1022 that takes at
# most 52 bits. So rnd(a + b) is a + b, and conesplit.py writes u01 unrounded
# where it is below the normals.
@rnd = float<ieee_64, ne>;
a = rnd(a_); b = rnd(b_);
s = rnd(a + b);
{
     |a| <= 1b1024 /\ |b| <= 1b1024
  /\ |a + b| <= 1b-1022
->
  s - (a + b) in [0, 0]
}
