# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at eb125ed0
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at eb125ed0
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0
# restates: crates/rcad-predicates/src/cascade.rs decide at eb125ed0
# SideOfAFlatSurfaceThroughAPoint::on_floats without its six product_is_normal
# checks: the sign of facing . point - facing . through, as six rounded
# products of two doubles folded left to right.
# No term is normal: each product is at most 2^-1022, and may be zero.
@rnd = float<ieee_64, ne>;

a0 = rnd(a0_); b0 = rnd(b0_); a1 = rnd(a1_); b1 = rnd(b1_); a2 = rnd(a2_); b2 = rnd(b2_); a3 = rnd(a3_); b3 = rnd(b3_); a4 = rnd(a4_); b4 = rnd(b4_); a5 = rnd(a5_); b5 = rnd(b5_);

# What on_floats computes, every term in the fold.
X0 = a0 * b0; X1 = a1 * b1; X2 = a2 * b2; X3 = a3 * b3; X4 = a4 * b4; X5 = a5 * b5;
t0 = rnd(X0); t1 = rnd(X1); t2 = rnd(X2); t3 = rnd(X3); t4 = rnd(X4); t5 = rnd(X5);
S1 = t0 + t1;
s1 = rnd(S1);
S2 = s1 + t2;
s2 = rnd(S2);
S3 = s2 + t3;
s3 = rnd(S3);
S4 = s3 + t4;
s4 = rnd(S4);
y = s4 + t5;

P = rnd(rnd(rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |t3|) + |t4|) + |t5|);

B = 6b-53 + 1b-99;
eb = rnd(B * P);

Ex = X0 + X1 + X2 + X3 + X4 + X5;

{
  # Every input is a finite f64.
     |a0| <= 1b1024 /\ |b0| <= 1b1024 /\ |a1| <= 1b1024 /\ |b1| <= 1b1024 /\ |a2| <= 1b1024 /\ |b2| <= 1b1024 /\ |a3| <= 1b1024 /\ |b3| <= 1b1024 /\ |a4| <= 1b1024 /\ |b4| <= 1b1024 /\ |a5| <= 1b1024 /\ |b5| <= 1b1024
  # No term is normal.
  /\ |X0| <= 1b-1022 /\ |X1| <= 1b-1022 /\ |X2| <= 1b-1022 /\ |X3| <= 1b-1022 /\ |X4| <= 1b-1022 /\ |X5| <= 1b-1022
->
  eb in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}
