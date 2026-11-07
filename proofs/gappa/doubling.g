# restates: Cargo.lock nalgebra at ba0a654e
# nalgebra's t = c * 2 rounds nothing: for any f64 c that does not overflow
# when doubled, rnd(c * 2) is c * 2. point.g writes t without the rounding on
# the strength of this.
@rnd = float<ieee_64, ne>;

c = rnd(c_);
t = rnd(c * 2);

{
  |c| <= 1b1023
->
  t - c * 2 in [0, 0]
}
