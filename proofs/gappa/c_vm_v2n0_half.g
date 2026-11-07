# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: Cargo.lock nalgebra at 4b642567
# 464.20's vector.g, the turned vector's x, with zero factors v2, n0,
# and so zero P02, P11, P22, K2, each rounded product on a live
# path normal, below the normals or 0, and each rounded sum on one free
# to cancel.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) and the vector n, each an f64, a zero factor the
# double 0.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = 0;
n0 = 0; n1 = rnd(n1_); n2 = rnd(n2_);

# What apply_to_vector computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g). A rounded product on a live
# path is written as what mixedn.g and mixeds.g prove of it, rnd(P) =
# P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],
# and a rounded sum on one as what sum.g proves of it, C * r.
P01 = v1 * n2; rq01 = 1 + mq01 * sq01; q01 = P01 * rq01 + mq01 * 1b-1022;
P02 = v2 * n1; q02 = P02;
P11 = v2 * n0; q11 = P11;
P12 = v0 * n2; q12 = rnd(P12);
P21 = v0 * n1; rq21 = 1 + mq21 * sq21; q21 = P21 * rq21 + mq21 * 1b-1022;
P22 = v1 * n0; q22 = P22;
C0 = q01 - q02; c0 = C0 * rc0;
C1 = q11 - q12; c1 = rnd(C1);
C2 = q21 - q22; c2 = C2 * rc2;
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; rk01 = 1 + mk01 * sk01; k01 = K1 * rk01 + mk01 * 1b-1022;
K2 = v2 * t1; k02 = K2;
KK = k01 - k02; k0 = KK * rk0;
M = t0 * w; rm = 1 + mm0 * sm0; m0 = M * rm + mm0 * 1b-1022;
A = m0 + k0; a0 = A * ra;
R = a0 + n0; out = R * rr;

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * n2 - v2 * n1)
   + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2))
   + n0;

# Each term of Ex as out carries it, less the term itself, through the
# roundings left on its path. A term with a zero factor is zero.
W1 = rr * ra * rm * rc0 * rq01 - 1;
W3 = rr * ra * rk0 * rk01 * rc2 * rq21 - 1;
e1 = 2 * w * v1 * n2 * W1;
e3 = 2 * v1 * v0 * n1 * W3;

# What each rounded product's m * 2^-1022 comes to in out, through the
# sums and products after it and the roundings left on its path.
Aq01 = mq01 * 1b-1022 * (2 * w) * (rr * ra * rm * rc0);
Aq21 = mq21 * 1b-1022 * (2 * v1) * (rr * ra * rk0 * rk01 * rc2);
Ak01 = mk01 * 1b-1022 * (rr * ra * rk0);
Am0 = mm0 * 1b-1022 * (rr * ra);

# Each live term's share of D, as vector.g's, with 1 for the vector's
# component.
d1 = 10 * |w| * |v1|;
d3 = 12 * |v1| * |v0|;
D = d1 + d3;

# Each amount's share of G, |f| Lo, written Lo itself where f is 1: Gappa
# 1.8.0 aborts on a name defined as another name alone, as in g = Lo.
# E is out - Ex less the amounts, the terms' errors alone.
gq01 = 2 * |w| * Lo;
gq21 = 2 * |v1| * Lo;
G = gq01 + gq21 + Lo + Lo;
Asum = Aq01 + Aq21 + Ak01 + Am0;
E = out - Ex - Asum;

# q.q and the four squares D is written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 25/93 * (|v1| + |v2|);
Y = |v0| - 10/31 * (|v1| + |v2|);
Z = |v1| - |v2|;

# G / Lo's linear part as squares, lam = 7/5.
Hw = |w| - 5/7;
H1 = |v1| - 5/7;

{
  # Every part of the turn is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024
  # No factor of a product is zero but v2, n0.
  /\ |w| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |v1| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  # What sum.g proves of each rounded sum on a live path, the sum of two
  # doubles, or 0 where it cancels, which any ratio meets.
  /\ rc0 in [9007199254740991b-53, 9007199254740993b-53] /\ rc2 in [9007199254740991b-53, 9007199254740993b-53]
  /\ rk0 in [9007199254740991b-53, 9007199254740993b-53] /\ ra in [9007199254740991b-53, 9007199254740993b-53]
  /\ rr in [9007199254740991b-53, 9007199254740993b-53]
  # What mixedn.g and mixeds.g prove of each rounded product on a live
  # path, normal or not, and |P| / P in [-1, 1] for any P not zero.
  /\ mq01 in [-1b-53, 1b-53] /\ mq21 in [-1b-53, 1b-53] /\ mk01 in [-1b-53, 1b-53]
  /\ mm0 in [-1b-53, 1b-53]
  /\ sq01 in [-1, 1] /\ sq21 in [-1, 1] /\ sk01 in [-1, 1]
  /\ sm0 in [-1, 1]
  # Lo is the smallest normal number.
  /\ Lo in [1b-1022, 1b-1022]
  # Every component of the vector is at most one: a constructor's normal is
  # the nearest f64 to a unit vector's components.
  /\ n1 in [-1, 1] /\ n2 in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v0 / |v0| in [-1, 1] /\ v1 / |v1| in [-1, 1]
->
  E / D in ? /\ D in ? /\ Asum / G in ? /\ G / Lo in ?
  /\ out - Ex in [-1b-54, 1b-54]
}

# out - Ex is the terms' error and the amounts, each averaged on its own.
out - Ex -> E + Asum;
# The amounts, each a fraction of its own share of G; add_rr averages them.
Asum -> (Asum / G) * (G / Lo) * Lo;
Asum / G -> ((G + Asum) - G) / G;
G + Asum -> (gq01 + Aq01) + (gq21 + Aq21) + (Lo + Ak01) + (Lo + Am0);
(gq01 + Aq01) - gq01 -> Aq01;
(gq21 + Aq21) - gq21 -> Aq21;
(Lo + Ak01) - Lo -> Ak01;
(Lo + Am0) - Lo -> Am0;
Aq01 / gq01 -> mq01 * (1b-1022 / Lo) * (w / |w|) * (rr * ra * rm * rc0);
Aq21 / gq21 -> mq21 * (1b-1022 / Lo) * (v1 / |v1|) * (rr * ra * rk0 * rk01 * rc2);
Ak01 / Lo -> mk01 * (1b-1022 / Lo) * (rr * ra * rk0);
Am0 / Lo -> mm0 * (1b-1022 / Lo) * (rr * ra);
# The terms' error is the sum of the live shares' errors, each a fraction
# of its own share of D; add_rr then averages them.
E -> (E / D) * D;
E / D -> ((D + E) - D) / D;
D + E -> (d1 + e1) + (d3 + e3);
(d1 + e1) - d1 -> e1;
(d3 + e3) - d3 -> e3;
e1 / d1 -> (w / |w|) * (v1 / |v1|) * n2 * (W1 / 5);
e3 / d3 -> (v1 / |v1|) * (v0 / |v0|) * n1 * (W3 / 6);

# D is vector.g's 93/5 q.q less four squares, plus 1 for n0, less the
# share of each term a zero factor took out, and a zero v's square.
D -> 93/5 * N4 - (93/5 * (X * X) + 93/5 * (Y * Y) + 305/93 * (Z * Z)
                 + 19/465 * (|v1| * |v1|))
     - (10 * |w| * |v2| + 12 * |v1| * |v1| + 12 * |v2| * |v2| + 12 * |v2| * |v0| + 19/465 * |v2| * |v2|);

# G / Lo is the amounts' shares, 2 n_w |w| + 2 n_1 |v1| + 2 n_2 |v2| + n_c,
# written as lam q.q, a constant and squares.
G / Lo -> 7/5 * N4 + 24/7
        - (7/5 * |v0| * |v0| + 7/5 * (Hw * Hw) + 7/5 * (H1 * H1) + 7/5 * |v2| * |v2|);
