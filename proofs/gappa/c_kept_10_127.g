# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at b1f1bf1a
# restates: crates/rcad-geom/src/plane.rs transformed at b1f1bf1a
# restates: crates/rcad-geom/src/plane.rs from_point_and_normal_as_written at b1f1bf1a
# restates: crates/rcad-geom/src/plane.rs has_length_one_to_within_a_turn at b1f1bf1a
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at b1f1bf1a
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 17306d5ee
# 464.37: Plane::NORMAL_LENGTH_ROUNDOFF for a normal that does not come
# straight from a constructor, turned once: n is any f64 vector with n.n
# within 10u of one, the last whole u inside what one turn leaves room for. This is 464.20's square.g with its
# constructor taken out. The hypothesis on d0, d1 and d2 is vector.g's 20u,
# which vector.g proves for components at most one; these reach 1 + 5u
# (Q2831).
@rnd = float<ieee_64, ne>;

# The normal: any f64 vector, held below to n.n near one.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
NN = n0 * n0 + n1 * n1 + n2 * n2;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;

# nalgebra's turn of n without rounding, as vector.g's Ex writes x.
Ex0 = 2 * w * (v1 * n2 - v2 * n1)
    + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2)) + n0;
Ex1 = 2 * w * (v2 * n0 - v0 * n2)
    + v2 * (2 * (v1 * n2 - v2 * n1)) - v0 * (2 * (v0 * n1 - v1 * n0)) + n1;
Ex2 = 2 * w * (v0 * n1 - v1 * n0)
    + v0 * (2 * (v2 * n0 - v0 * n2)) - v1 * (2 * (v1 * n2 - v2 * n1)) + n2;
ExEx = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;

# The turned vector the f64 arithmetic gives: Ex and each rounding's slip.
o0 = Ex0 + d0; o1 = Ex1 + d1; o2 = Ex2 + d2;
OO = o0 * o0 + o1 * o1 + o2 * o2;
DD = d0 * d0 + d1 * d1 + d2 * d2;
F2 = 2 * (Ex0 * d0 + Ex1 * d1 + Ex2 * d2);

# v x n and v . n, without rounding, and v.v.
X0 = v1 * n2 - v2 * n1; X1 = v2 * n0 - v0 * n2; X2 = v0 * n1 - v1 * n0;
VN = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * n0 + v1 * n1 + v2 * n2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

{
  # n.n within 10u of one, so each component is at most 1 + 5u.
     NN - 1 in [-20b-54, 20b-54]
  /\ n0 in [-18014398509481994b-54, 18014398509481994b-54] /\ n1 in [-18014398509481994b-54, 18014398509481994b-54]
  /\ n2 in [-18014398509481994b-54, 18014398509481994b-54]
  # Every quaternion component is a finite f64.
  /\ |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # What vector.g, vector1.g and vector2.g prove of each turned component.
  /\ d0 in [-20b-53, 20b-53] /\ d1 in [-20b-53, 20b-53]
  /\ d2 in [-20b-53, 20b-53]
->
  NN - 1 in ? /\ VN in ? /\ 4 * (N4 - 1) * VN in ? /\ F2 in ? /\ DD in ?
  /\ OO - 1 in [-127b-53, 127b-53]
}

# n.n off one is the constructor's, then the turn's own arithmetic, then the
# turn's rounding.
OO - 1 -> (NN - 1) + 4 * (N4 - 1) * VN + F2 + DD;
ExEx -> NN + 4 * (N4 - 1) * VN;

# |v x n|^2 is at least zero, a sum of squares, and at most v.v n.n, since
# what is between is (v . n)^2; v.v is q.q less w^2.
VN -> VV * NN - VD * VD;
VV -> N4 - w * w;

# 2 Ex . d, both ways round: 2ab is c a^2 + b^2 / c less (c a - b)^2 / c,
# and (c a + b)^2 / c less c a^2 + b^2 / c. c is near sqrt(3) * 20u, which
# makes c Ex.Ex + DD / c smallest for Ex.Ex near one and DD at most 3 (20u)^2.
F2 -> (34641b-53 / 1000 * ExEx + DD / (34641b-53 / 1000))
      - ((34641b-53 / 1000 * Ex0 - d0) * (34641b-53 / 1000 * Ex0 - d0)
       + (34641b-53 / 1000 * Ex1 - d1) * (34641b-53 / 1000 * Ex1 - d1)
       + (34641b-53 / 1000 * Ex2 - d2) * (34641b-53 / 1000 * Ex2 - d2))
        / (34641b-53 / 1000);
F2 -> ((34641b-53 / 1000 * Ex0 + d0) * (34641b-53 / 1000 * Ex0 + d0)
     + (34641b-53 / 1000 * Ex1 + d1) * (34641b-53 / 1000 * Ex1 + d1)
     + (34641b-53 / 1000 * Ex2 + d2) * (34641b-53 / 1000 * Ex2 + d2))
        / (34641b-53 / 1000)
      - (34641b-53 / 1000 * ExEx + DD / (34641b-53 / 1000));
