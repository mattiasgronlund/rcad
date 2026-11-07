# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# restates: Cargo.lock nalgebra at ba0a654e
# 464.19's point.g, the moved point's x, with zero factors v1, p0, p1,
# and so zero P01, P02, P11, P21, P22, C0, C2, K1, M, each rounded product on a live
# path normal, below the normals or 0, and each rounded sum on one free
# to cancel.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), the point and the shift's x, each an f64, a zero
# factor the double 0.
w = rnd(w_); v0 = rnd(v0_); v1 = 0; v2 = rnd(v2_);
p0 = 0; p1 = 0; p2 = rnd(p2_);
s0 = rnd(s0_);

# What apply_to_point computes, line for line, a zero written unrounded,
# since it rounds to itself (464.11's refl.g). A rounded product on a live
# path is written as what mixedn.g and mixeds.g prove of it, rnd(P) =
# P * (1 + m * s) + m * 2^-1022, m in [-u, u] and s, |P| / P, in [-1, 1],
# and a rounded sum on one as what sum.g proves of it, C * r.
P01 = v1 * p2; q01 = P01;
P02 = v2 * p1; q02 = P02;
P11 = v2 * p0; q11 = P11;
P12 = v0 * p2; rq12 = 1 + mq12 * sq12; q12 = P12 * rq12 + mq12 * 1b-1022;
P21 = v0 * p1; q21 = P21;
P22 = v1 * p0; q22 = P22;
C0 = q01 - q02; c0 = C0;
C1 = q11 - q12; c1 = C1 * rc1;
C2 = q21 - q22; c2 = C2;
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;
K1 = v1 * t2; k01 = K1;
K2 = v2 * t1; rk02 = 1 + mk02 * sk02; k02 = K2 * rk02 + mk02 * 1b-1022;
KK = k01 - k02; k0 = KK * rk0;
M = t0 * w; m0 = M;
A = m0 + k0; a0 = A * ra;
R = a0 + p0; r0 = R * rr;
O = r0 + s0; out = O * ro;

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * p2 - v2 * p1)
   + v1 * (2 * (v0 * p1 - v1 * p0)) - v2 * (2 * (v2 * p0 - v0 * p2))
   + p0 + s0;

# Each term of Ex as out carries it, less the term itself, through the
# roundings left on its path. A term with a zero factor is zero.
W6 = ro * rr * ra * rk0 * rk02 * rc1 * rq12 - 1;
W8 = ro - 1;
e6 = 2 * v2 * v0 * p2 * W6;
e7 = s0 * W8;

# What each rounded product's m * 2^-1022 comes to in out, through the
# sums and products after it and the roundings left on its path.
Aq12 = mq12 * 1b-1022 * (2 * v2) * (ro * rr * ra * rk0 * rk02 * rc1);
Ak02 = -(mk02 * 1b-1022 * (ro * rr * ra * rk0));

# Each live term's share of D, as point.g's, p0 and s0 sharing 3L.
d6 = 14 * |v2| * |v0| * L;
d7 = 3 * L;
D = d6 + d7;

# Each amount's share of G, |f| L, written L itself where f is 1: Gappa
# 1.8.0 aborts on a name defined as another name alone, as in g = L.
# E is out - Ex less the amounts, the terms' errors alone.
gq12 = 2 * |v2| * L;
G = gq12 + L;
Asum = Aq12 + Ak02;
E = out - Ex - Asum;

# q.q and the four squares D / L is written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 30/109 * (|v1| + |v2|);
Y = |v0| - 35/109 * (|v1| + |v2|);
Z = |v1| - |v2|;

# G / L's linear part as squares, lam = 1.
H2 = |v2| - 1;

{
  # Every input is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v2| <= 1b1024 /\ |p2| <= 1b1024
  /\ |s0| <= 1b1024
  # No factor of a product is zero but v1, p0, p1.
  /\ |w| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |v2| >= 1b-1074 /\ |p2| >= 1b-1074
  # What sum.g proves of each rounded sum on a live path, the sum of two
  # doubles, or 0 where it cancels, which any ratio meets.
  /\ rc1 in [9007199254740991b-53, 9007199254740993b-53] /\ rk0 in [9007199254740991b-53, 9007199254740993b-53]
  /\ ra in [9007199254740991b-53, 9007199254740993b-53] /\ rr in [9007199254740991b-53, 9007199254740993b-53]
  /\ ro in [9007199254740991b-53, 9007199254740993b-53]
  # What mixedn.g and mixeds.g prove of each rounded product on a live
  # path, normal or not, and |P| / P in [-1, 1] for any P not zero.
  /\ mq12 in [-1b-53, 1b-53] /\ mk02 in [-1b-53, 1b-53]
  /\ sq12 in [-1, 1] /\ sk02 in [-1, 1]
  # L is a normal number at least every coordinate of the point and the shift.
  /\ L in [1b-1022, 1b1024]
  /\ p2 / L in [-1, 1] /\ s0 / L in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v0 / |v0| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  E / D in ? /\ D / L in ? /\ Asum / G in ? /\ G / L in ?
  /\ (out - Ex) / L in [-1b-54, 1b-54]
}

# out - Ex is the terms' error and the amounts, each averaged on its own.
(out - Ex) / L -> E / L + Asum / L;
# The amounts, each a fraction of its own share of G; add_rr averages them.
Asum / L -> (Asum / G) * (G / L);
Asum / G -> ((G + Asum) - G) / G;
G + Asum -> (gq12 + Aq12) + (L + Ak02);
(gq12 + Aq12) - gq12 -> Aq12;
(L + Ak02) - L -> Ak02;
Aq12 / gq12 -> mq12 * (1b-1022 / L) * (v2 / |v2|) * (ro * rr * ra * rk0 * rk02 * rc1);
Ak02 / L -> -(mk02 * (1b-1022 / L) * (ro * rr * ra * rk0));
# The terms' error is the sum of the live shares' errors, each a fraction
# of its own share of D; add_rr then averages them.
E / L -> (E / D) * (D / L);
E / D -> ((D + E) - D) / D;
D + E -> (d6 + e6) + (d7 + e7);
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e6 / d6 -> (v2 / |v2|) * (v0 / |v0|) * (p2 / L) * (W6 / 7);
e7 / d7 -> ((s0 / L) * W8) / 3;

# D / L is point.g's 3 and quadratic, 109/5 q.q less four squares, less
# the share of each term a zero factor took out, and a zero v's square.
D / L -> 109/5 * N4 - (109/5 * (X * X) + 109/5 * (Y * Y) + 425/109 * (Z * Z)
                       + 1/545 * (|v2| * |v2|)) + 3
        - (12 * |w| * |v1| + 12 * |w| * |v2| + 14 * |v1| * |v0| + 14 * |v1| * |v1| + 14 * |v2| * |v2| + 1/545 * |v1| * |v1|);

# G / L is the amounts' shares, 2 n_w |w| + 2 n_1 |v1| + 2 n_2 |v2| + n_c,
# written as lam q.q, a constant and squares.
G / L -> 1 * N4 + 2
        - (1 * |v0| * |v0| + 1 * |w| * |w| + 1 * |v1| * |v1| + 1 * (H2 * H2));
