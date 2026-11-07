# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at 6a4a814c
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# SideOfAFlatSurface::on_floats without its three product_is_normal checks:
# the sign of n . p - off. Products: n0 * p0 below the normals and positive, n1 * p1 below the normals and negative, n2 * p2 zero.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64. A zero product has
# the double 0 for its point's coordinate; with the normal's zero instead,
# every number below is the same.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = 0;
off = rnd(off_);

# What on_floats computes, line for line. Adding a zero product to a double
# gives that double back (464.13's addzero.g), so zero products are left
# out of both folds.
X0 = n0 * p0; X1 = n1 * p1;
t0 = rnd(X0); t1 = rnd(X1);
S1 = t0 + t1;
s1 = rnd(S1);
y = s1 - off;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(t0 + -t1) + |off|);
Phat = t0 + -t1 + |off|;

B = 4b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = X0 + X1 + n2 * p2 - off;

# No product is normal, so every error is an amount: what rounding each
# X cost and what rounding each sum cost.

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |off| <= 1b1024
  # No rounded sum cancels to zero.
  /\ |S1| >= 1b-1074
  # The products below the normals. A product of two doubles that are not
  # zero is at least 2^-2148.
  /\ X0 in [1b-2148, 1b-1022] /\ X1 in [-1b-1022, -1b-2148]
->
  y - Ex in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# y less the truth is what each rounding cost, as an amount.
y - Ex -> (t0 - X0) + (t1 - X1) + (s1 - S1);
