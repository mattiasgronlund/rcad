# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/plane.rs transformed at f95bb965
# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a plane,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner x exactly on
# the plane n.x = off, moved, is within 128 * f64::EPSILON * L of the moved
# plane. Plane::transformed turns the normal, n' = Ex(n) + dn, and gives the
# offset off' = off + n'.s + dd by shiftplane.g's arithmetic; the moved
# corner is Ex(x) + s + dp. Then n'.p' - off' is
#   4 (q.q - 1) (v x n) . (v x x) + dn . Ex(x) + n' . dp - dd,
# with Ex nalgebra's turn without rounding (sos.py), and the distance is that
# over |n'|. Every length is per unit of L, which divides nothing that rounds.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

# The stored normal n, and the corner over L.
NN = n0 * n0 + n1 * n1 + n2 * n2;
XX = X0 * X0 + X1 * X1 + X2 * X2;

# nalgebra's turn of n and of the corner without rounding, as vector.g's Ex
# writes x.
An0 = 2 * w * (v1 * n2 - v2 * n1)
    + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2)) + n0;
An1 = 2 * w * (v2 * n0 - v0 * n2)
    + v2 * (2 * (v1 * n2 - v2 * n1)) - v0 * (2 * (v0 * n1 - v1 * n0)) + n1;
An2 = 2 * w * (v0 * n1 - v1 * n0)
    + v0 * (2 * (v2 * n0 - v0 * n2)) - v1 * (2 * (v1 * n2 - v2 * n1)) + n2;
E0 = 2 * w * (v1 * X2 - v2 * X1)
   + v1 * (2 * (v0 * X1 - v1 * X0)) - v2 * (2 * (v2 * X0 - v0 * X2)) + X0;
E1 = 2 * w * (v2 * X0 - v0 * X2)
   + v2 * (2 * (v1 * X2 - v2 * X1)) - v0 * (2 * (v0 * X1 - v1 * X0)) + X1;
E2 = 2 * w * (v0 * X1 - v1 * X0)
   + v0 * (2 * (v2 * X0 - v0 * X2)) - v1 * (2 * (v1 * X2 - v2 * X1)) + X2;
AnAn = An0 * An0 + An1 * An1 + An2 * An2;
EE = E0 * E0 + E1 * E1 + E2 * E2;

# v x n and v x x, v . n and v . x, without rounding.
a0 = v1 * n2 - v2 * n1; a1 = v2 * n0 - v0 * n2; a2 = v0 * n1 - v1 * n0;
b0 = v1 * X2 - v2 * X1; b1 = v2 * X0 - v0 * X2; b2 = v0 * X1 - v1 * X0;
VN = a0 * a0 + a1 * a1 + a2 * a2;
VX = b0 * b0 + b1 * b1 + b2 * b2;
VDn = v0 * n0 + v1 * n1 + v2 * n2;
VDx = v0 * X0 + v1 * X1 + v2 * X2;

# The turned normal as the code stores it, and the slips.
np0 = An0 + dn0; np1 = An1 + dn1; np2 = An2 + dn2;
NPNP = np0 * np0 + np1 * np1 + np2 * np2;
DNDN = dn0 * dn0 + dn1 * dn1 + dn2 * dn2;
DPDP = DP0 * DP0 + DP1 * DP1 + DP2 * DP2;

# The four shares of n'.p' - off', per unit of L, and twice the three dot
# products they and n'.n' are written with below.
T1 = 4 * (N4 - 1) * (a0 * b0 + a1 * b1 + a2 * b2);
T2 = dn0 * E0 + dn1 * E1 + dn2 * E2;
T3 = np0 * DP0 + np1 * DP1 + np2 * DP2;
T4 = -DD;
F1 = 2 * (a0 * b0 + a1 * b1 + a2 * b2);
F2 = 2 * (dn0 * E0 + dn1 * E1 + dn2 * E2);
F3 = 2 * (np0 * DP0 + np1 * DP1 + np2 * DP2);
F4 = 2 * (An0 * dn0 + An1 * dn1 + An2 * dn2);
T = T1 + T2 + T3 + T4;

{
  # Every quaternion component is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # A stored normal has every component at most one and n.n within
  # NORMAL_LENGTH_ROUNDOFF, 64 epsilon, of one.
  /\ n0 in [-1, 1] /\ n1 in [-1, 1] /\ n2 in [-1, 1]
  /\ NN in [9007199254740864b-53, 9007199254741120b-53]
  # L is at least every coordinate of the corner.
  /\ X0 in [-1, 1] /\ X1 in [-1, 1] /\ X2 in [-1, 1]
  # What vector.g, vector1.g and vector2.g prove of the turned normal.
  /\ dn0 in [-20b-53, 20b-53] /\ dn1 in [-20b-53, 20b-53]
  /\ dn2 in [-20b-53, 20b-53]
  # What point.g, point1.g and point2.g prove of the moved corner, over L.
  /\ DP0 in [-36b-53, 36b-53] /\ DP1 in [-36b-53, 36b-53]
  /\ DP2 in [-36b-53, 36b-53]
  # What shiftplane.g proves of the offset's rounding over L, 16u times
  # the larger of |n'| and one, with |n'| within 124u of one: under 17u.
  /\ DD in [-17b-53, 17b-53]
->
  T1 in ? /\ T2 in ? /\ T3 in ? /\ NPNP - 1 in ?
  /\ T / sqrt(NPNP) in [-222b-53, 222b-53]
}

# The turn's length share: Ex(n) . Ex(x) - n . x is 4 (q.q - 1)
# (v x n) . (v x x) (sos.py), and 2ab is c a^2 + b^2 / c less (c a - b)^2 / c,
# and (c a + b)^2 / c less c a^2 + b^2 / c, with c near sqrt(3), which makes
# c |v x n|^2 + |v x x|^2 / c smallest for n.n near one and x.x at most 3.
T1 -> 2 * (N4 - 1) * F1;
F1 -> (1732/1000 * VN + VX / (1732/1000))
      - ((1732/1000 * a0 - b0) * (1732/1000 * a0 - b0)
       + (1732/1000 * a1 - b1) * (1732/1000 * a1 - b1)
       + (1732/1000 * a2 - b2) * (1732/1000 * a2 - b2)) / (1732/1000);
F1 -> ((1732/1000 * a0 + b0) * (1732/1000 * a0 + b0)
     + (1732/1000 * a1 + b1) * (1732/1000 * a1 + b1)
     + (1732/1000 * a2 + b2) * (1732/1000 * a2 + b2)) / (1732/1000)
      - (1732/1000 * VN + VX / (1732/1000));
VN -> VV * NN - VDn * VDn;
VX -> VV * XX - VDx * VDx;
VV -> N4 - w * w;

# The normal's slip against the turned corner, c = 20u.
T2 -> F2 / 2;
F2 -> (20b-53 * EE + DNDN / 20b-53)
      - ((20b-53 * E0 - dn0) * (20b-53 * E0 - dn0)
       + (20b-53 * E1 - dn1) * (20b-53 * E1 - dn1)
       + (20b-53 * E2 - dn2) * (20b-53 * E2 - dn2)) / 20b-53;
F2 -> ((20b-53 * E0 + dn0) * (20b-53 * E0 + dn0)
     + (20b-53 * E1 + dn1) * (20b-53 * E1 + dn1)
     + (20b-53 * E2 + dn2) * (20b-53 * E2 + dn2)) / 20b-53
      - (20b-53 * EE + DNDN / 20b-53);
EE -> XX + 4 * (N4 - 1) * VX;

# The corner's slip against the turned normal, c = sqrt(3) * 36u.
T3 -> F3 / 2;
F3 -> (62354b-53 / 1000 * NPNP + DPDP / (62354b-53 / 1000))
      - ((62354b-53 / 1000 * np0 - DP0) * (62354b-53 / 1000 * np0 - DP0)
       + (62354b-53 / 1000 * np1 - DP1) * (62354b-53 / 1000 * np1 - DP1)
       + (62354b-53 / 1000 * np2 - DP2) * (62354b-53 / 1000 * np2 - DP2))
        / (62354b-53 / 1000);
F3 -> ((62354b-53 / 1000 * np0 + DP0) * (62354b-53 / 1000 * np0 + DP0)
     + (62354b-53 / 1000 * np1 + DP1) * (62354b-53 / 1000 * np1 + DP1)
     + (62354b-53 / 1000 * np2 + DP2) * (62354b-53 / 1000 * np2 + DP2))
        / (62354b-53 / 1000)
      - (62354b-53 / 1000 * NPNP + DPDP / (62354b-53 / 1000));

# n'.n' as square.g writes it: n.n, the turn's length share, 2 Ex(n) . dn
# with c = sqrt(3) * 20u, and dn . dn.
NPNP -> AnAn + F4 + DNDN;
AnAn -> NN + 4 * (N4 - 1) * VN;
F4 -> (34641b-53 / 1000 * AnAn + DNDN / (34641b-53 / 1000))
      - ((34641b-53 / 1000 * An0 - dn0) * (34641b-53 / 1000 * An0 - dn0)
       + (34641b-53 / 1000 * An1 - dn1) * (34641b-53 / 1000 * An1 - dn1)
       + (34641b-53 / 1000 * An2 - dn2) * (34641b-53 / 1000 * An2 - dn2))
        / (34641b-53 / 1000);
F4 -> ((34641b-53 / 1000 * An0 + dn0) * (34641b-53 / 1000 * An0 + dn0)
     + (34641b-53 / 1000 * An1 + dn1) * (34641b-53 / 1000 * An1 + dn1)
     + (34641b-53 / 1000 * An2 + dn2) * (34641b-53 / 1000 * An2 + dn2))
        / (34641b-53 / 1000)
      - (34641b-53 / 1000 * AnAn + DNDN / (34641b-53 / 1000));
