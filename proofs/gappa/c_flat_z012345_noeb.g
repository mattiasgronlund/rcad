# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at 8c943dfd
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at 8c943dfd
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs decide at 8c943dfd
# SideOfAFlatSurfaceThroughAPoint::on_floats and its error bound: the sign
# of facing . point - facing . through.
# The case where products 0, 1, 2, 3, 4, 5 are zero.
@rnd = float<ieee_64, ne>;

# The facing, the point and the through point, each an f64. A product that
# is zero has the double 0 for its point's or its through point's
# coordinate; where the facing's coordinate is the zero, both its products
# are zero, and every number below is the same as with both of those
# coordinates zero.
f0 = rnd(f0_); f1 = rnd(f1_); f2 = rnd(f2_);
p0 = 0; p1 = 0; p2 = 0;
q0 = 0; q1 = 0; q2 = 0;

# What on_floats computes, line for line. The fold starts at 0.0, and
# adding 0.0 or a zero product to a double gives that double back
# (addzero.g), so the zero products and the first addition are left out.
y = 0;
value = y;

# The permanent folds the same products' magnitudes in the same order.
P = 0;

B = 6b-53 + 1b-99;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = f0 * p0 + f1 * p1 + f2 * p2 - f0 * q0 - f1 * q1 - f2 * q2;

# The permanent with no rounding of its own.
Phat = 0;

{
  # Every input is a finite f64.
     |f0| <= 1b1024 /\ |f1| <= 1b1024 /\ |f2| <= 1b1024
->
  eb in ? /\ (y - Ex) / eb in [-1, 1]
}

