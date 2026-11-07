# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: Cargo.lock nalgebra at 4b642567
# 464.20's vector.g, the turned vector's x, with zero factors w, v1,
# and so zero P01, P22, K1, M, each rounded product on a live
# path normal, below the normals or 0, and each rounded sum on one free
# to cancel.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) and the vector n, each an f64, a zero factor the
# double 0.
w = 0; v0 = rnd(v0_); v1 = 0; v2 = rnd(v2_);
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);

# What apply_to_vector computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g). A rounded product on a live
# path is written as what mixedn.g and mixeds.g prove of it, rnd(P) =
# P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],
# and a rounded sum on one as what sum.g proves of it, C * r.
P01 = v1 * n2; q01 = P01;
P02 = v2 * n1; q02 = rnd(P02);
P11 = v2 * n0; rq11 = 1 + mq11 * sq11; q11 = P11 * rq11 + mq11 * 1b-1022;
P12 = v0 * n2; rq12 = 1 + mq12 * sq12; q12 = P12 * rq12 + mq12 * 1b-1022;
P21 = v0 * n1; q21 = rnd(P21);
P22 = v1 * n0; q22 = P22;
C0 = q01 - q02; c0 = rnd(C0);
C1 = q11 - q12; c1 = C1 * rc1;
C2 = q21 - q22; c2 = rnd(C2);
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; k01 = K1;
K2 = v2 * t1; rk02 = 1 + mk02 * sk02; k02 = K2 * rk02 + mk02 * 1b-1022;
KK = k01 - k02; k0 = KK * rk0;
M = t0 * w; m0 = M;
A = m0 + k0; a0 = A * ra;
R = a0 + n0; out = R * rr;

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * n2 - v2 * n1)
   + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2))
   + n0;

# Each term of Ex as out carries it, less the term itself, through the
# roundings left on its path. A term with a zero factor is zero.
W5 = rr * ra * rk0 * rk02 * rc1 * rq11 - 1;
W6 = rr * ra * rk0 * rk02 * rc1 * rq12 - 1;
W7 = rr - 1;
e5 = -(2 * v2 * v2 * n0 * W5);
e6 = 2 * v2 * v0 * n2 * W6;
e7 = n0 * W7;

# What each rounded product's m * 2^-1022 comes to in out, through the
# sums and products after it and the roundings left on its path.
Aq11 = -(mq11 * 1b-1022 * (2 * v2) * (rr * ra * rk0 * rk02 * rc1));
Aq12 = mq12 * 1b-1022 * (2 * v2) * (rr * ra * rk0 * rk02 * rc1);
Ak02 = -(mk02 * 1b-1022 * (rr * ra * rk0));

# Each live term's share of D, as vector.g's, with 1 for the vector's
# component.
d5 = 12 * |v2| * |v2|;
d6 = 12 * |v2| * |v0|;
d7 = 1;
D = d5 + d6 + d7;

# Each amount's share of G, |f| Lo, written Lo itself where f is 1: Gappa
# 1.8.0 aborts on a name defined as another name alone, as in g = Lo.
# E is out - Ex less the amounts, the terms' errors alone.
gq11 = 2 * |v2| * Lo;
gq12 = 2 * |v2| * Lo;
G = gq11 + gq12 + Lo;
Asum = Aq11 + Aq12 + Ak02;
E = out - Ex - Asum;

# q.q and the four squares D is written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 25/93 * (|v1| + |v2|);
Y = |v0| - 10/31 * (|v1| + |v2|);
Z = |v1| - |v2|;

# G / Lo's linear part as squares, lam = 2.
H2 = |v2| - 1;

{
  # Every part of the turn is a finite f64.
     |v0| <= 1b1024 /\ |v2| <= 1b1024
  # No factor of a product is zero but w, v1.
  /\ |v0| >= 1b-1074 /\ |v2| >= 1b-1074 /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  # What sum.g proves of each rounded sum on a live path, the sum of two
  # doubles, or 0 where it cancels, which any ratio meets.
  /\ rc1 in [9007199254740991b-53, 9007199254740993b-53] /\ rk0 in [9007199254740991b-53, 9007199254740993b-53]
  /\ ra in [9007199254740991b-53, 9007199254740993b-53] /\ rr in [9007199254740991b-53, 9007199254740993b-53]
  # What mixedn.g and mixeds.g prove of each rounded product on a live
  # path, normal or not, and |P| / P in [-1, 1] for any P not zero.
  /\ mq11 in [-1b-53, 1b-53] /\ mq12 in [-1b-53, 1b-53] /\ mk02 in [-1b-53, 1b-53]
  /\ sq11 in [-1, 1] /\ sq12 in [-1, 1] /\ sk02 in [-1, 1]
  # Lo is the smallest normal number.
  /\ Lo in [1b-1022, 1b-1022]
  # Every component of the vector is at most one: a constructor's normal is
  # the nearest f64 to a unit vector's components.
  /\ n0 in [-1, 1] /\ n1 in [-1, 1] /\ n2 in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ v0 / |v0| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  E / D in ? /\ D in ? /\ Asum / G in ? /\ G / Lo in ?
  /\ out - Ex in [-1b-54, 1b-54]
}

# out - Ex is the terms' error and the amounts, each averaged on its own.
out - Ex -> E + Asum;
# The amounts, each a fraction of its own share of G; add_rr averages them.
Asum -> (Asum / G) * (G / Lo) * Lo;
Asum / G -> ((G + Asum) - G) / G;
G + Asum -> (gq11 + Aq11) + (gq12 + Aq12) + (Lo + Ak02);
(gq11 + Aq11) - gq11 -> Aq11;
(gq12 + Aq12) - gq12 -> Aq12;
(Lo + Ak02) - Lo -> Ak02;
Aq11 / gq11 -> -(mq11 * (1b-1022 / Lo) * (v2 / |v2|) * (rr * ra * rk0 * rk02 * rc1));
Aq12 / gq12 -> mq12 * (1b-1022 / Lo) * (v2 / |v2|) * (rr * ra * rk0 * rk02 * rc1);
Ak02 / Lo -> -(mk02 * (1b-1022 / Lo) * (rr * ra * rk0));
# The terms' error is the sum of the live shares' errors, each a fraction
# of its own share of D; add_rr then averages them.
E -> (E / D) * D;
E / D -> ((D + E) - D) / D;
D + E -> (d5 + e5) + (d6 + e6) + (d7 + e7);
(d5 + e5) - d5 -> e5;
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e5 / d5 -> -((v2 / |v2|) * (v2 / |v2|) * n0 * (W5 / 6));
e6 / d6 -> (v2 / |v2|) * (v0 / |v0|) * n2 * (W6 / 6);
e7 / d7 -> n0 * W7;

# D is vector.g's 93/5 q.q less four squares, plus 1 for n0, less the
# share of each term a zero factor took out, and a zero v's square.
D -> 93/5 * N4 - (93/5 * (X * X) + 93/5 * (Y * Y) + 305/93 * (Z * Z)
                 + 19/465 * (|v2| * |v2|)) + 1
     - (10 * |w| * |v1| + 10 * |w| * |v2| + 12 * |v1| * |v0| + 12 * |v1| * |v1| + 19/465 * |v1| * |v1|);

# G / Lo is the amounts' shares, 2 n_w |w| + 2 n_1 |v1| + 2 n_2 |v2| + n_c,
# written as lam q.q, a constant and squares.
G / Lo -> 2 * N4 + 3
        - (2 * |v0| * |v0| + 2 * |w| * |w| + 2 * |v1| * |v1| + 2 * (H2 * H2));
