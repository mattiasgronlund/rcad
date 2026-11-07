# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at 8c943dfd
# addzero.g: adding the double 0 to a double, on either side, or taking 0
# from it, gives that double back. That is the fold's first addition, 0.0
# plus the first product, and each zero product the fold adds.
@rnd = float<ieee_64, ne>;
x = rnd(x_);
z = 0;
L = z + x; R = x + z; D = x - z;
{ |x| <= 1b1024 -> rnd(L) - x in [0, 0] /\ rnd(R) - x in [0, 0] /\ rnd(D) - x in [0, 0] }
# Without these three, Gappa ran rnd(x + z) for 30 s and more and did not stop.
L -> x;
R -> x;
D -> x;
