# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/cylinder.rs transformed at f95bb965
# restates: crates/rcad-geom/src/point.rs transformed at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a pipe,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner exactly on a
# pipe of radius r about the line through o along a, moved, is within
# 128 * f64::EPSILON * L of the moved pipe. Cylinder::transformed carries r
# over, and Axis::transformed moves o by apply_to_point and turns a by
# apply_to_vector. With y the corner less o, the moved corner less the moved
# origin is Ex(y) + k, k the two slips point.g proves, and the moved
# direction is Ex(a) + da, da the slip vector.g proves, Ex nalgebra's turn
# without rounding (sos.py). By hand, the distance to the moved pipe is at
# most |k| + |Ex(y)| tan(th) + | |Ex(y) x e| - r |, th the angle da tilts
# Ex(a) by and e the unit Ex(a), which is what this bounds, per unit of L.
# Ex turns with the frame (sos.py), so here a is along z and y in the xz
# plane, and w and v are the turn's quaternion in that frame: reals, not f64.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) in that frame, q.q, v.v and two sums of its squares.
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;
VV = v0 * v0 + v1 * v1 + v2 * v2;
S01 = v0 * v0 + v1 * v1;
S12 = v1 * v1 + v2 * v2;

# The stored direction a is (0, 0, al), and y over L is (RL, 0, TL): RL is r
# over L, the corner's distance from the axis, and TL how far along it the
# corner is. Both are written in place below, coordinate by coordinate, since
# gappa -Bcoq aborts in dump_real when names hold them (464.21's 7).
AA = al * al;
YY = RL * RL + 0 * 0 + TL * TL;

# nalgebra's turn of y and of a without rounding, as point.g's Ex writes x,
# and their lengths squared.
Ex0 = 2 * w * (v1 * TL - v2 * 0)
    + v1 * (2 * (v0 * 0 - v1 * RL)) - v2 * (2 * (v2 * RL - v0 * TL)) + RL;
Ex1 = 2 * w * (v2 * RL - v0 * TL)
    + v2 * (2 * (v1 * TL - v2 * 0)) - v0 * (2 * (v0 * 0 - v1 * RL)) + 0;
Ex2 = 2 * w * (v0 * 0 - v1 * RL)
    + v0 * (2 * (v2 * RL - v0 * TL)) - v1 * (2 * (v1 * TL - v2 * 0)) + TL;
Ea0 = 2 * w * (v1 * al - v2 * 0)
    + v1 * (2 * (v0 * 0 - v1 * 0)) - v2 * (2 * (v2 * 0 - v0 * al)) + 0;
Ea1 = 2 * w * (v2 * 0 - v0 * al)
    + v2 * (2 * (v1 * al - v2 * 0)) - v0 * (2 * (v0 * 0 - v1 * 0)) + 0;
Ea2 = 2 * w * (v0 * 0 - v1 * 0)
    + v0 * (2 * (v2 * 0 - v0 * al)) - v1 * (2 * (v1 * al - v2 * 0)) + al;
EE = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;
EaEa = Ea0 * Ea0 + Ea1 * Ea1 + Ea2 * Ea2;

# v x y and v . y, without rounding.
X0 = v1 * TL - v2 * 0; X1 = v2 * RL - v0 * TL; X2 = v0 * 0 - v1 * RL;
VY = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * RL + v1 * 0 + v2 * TL;

# In this frame |Ex(a)|^2 is al^2 DENQ and |Ex(y) x Ex(a)|^2 is
# RL^2 al^2 NUMQ (sos.py), so the distance of Ex(y) from the turned axis is
# RL sqrt(NUMQ / DENQ).
DENQ = 1 + 4 * (N4 - 1) * S01;
NUMQ = 1 + 4 * (N4 - 1) * (VV + v1 * v1) + 16 * (N4 - 1) * (N4 - 1) * VV * (v1 * v1);

# The two slips, the moved corner's and the moved origin's, per unit of L,
# and the moved direction's.
k0 = kp0 - ko0; k1 = kp1 - ko1; k2 = kp2 - ko2;
KK = k0 * k0 + k1 * k1 + k2 * k2;
DADA = da0 * da0 + da1 * da1 + da2 * da2;
DA = sqrt(DADA);

# The distance's three shares, per unit of L: the slips', the tilt's, and
# how far the turn's arithmetic takes the corner off the pipe.
S = sqrt(KK);
TI = sqrt(EE) * DA / (sqrt(EaEa) - DA);
DR = RL * |sqrt(NUMQ / DENQ) - 1|;

{
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53. Turning the frame turns v and keeps q.q.
     N4 in [9007199254740980b-53, 9007199254741004b-53]
  # Each of w^2, v0^2, v1^2 and v2^2 is at most q.q, under 1 + 12u, so each
  # of w, v0, v1 and v2 is at most 2 in size.
  /\ |w| <= 2 /\ |v0| <= 2 /\ |v1| <= 2 /\ |v2| <= 2
  # A stored direction has a.a within NORMAL_LENGTH_ROUNDOFF, 64 epsilon, of
  # one.
  /\ AA in [9007199254740864b-53, 9007199254741120b-53]
  /\ al in [0, 2]
  # L is at least every coordinate of the corner and of the axis origin, and
  # the radius.
  /\ RL in [0, 1] /\ TL in [-4, 4] /\ YY in [0, 12]
  # What r_point_25.g, r_point1_25.g and r_point2_25.g prove of the moved
  # corner and the moved origin, L at least every coordinate of either and
  # of the shift: 25u.
  /\ kp0 in [-25b-53, 25b-53] /\ kp1 in [-25b-53, 25b-53]
  /\ kp2 in [-25b-53, 25b-53]
  /\ ko0 in [-25b-53, 25b-53] /\ ko1 in [-25b-53, 25b-53]
  /\ ko2 in [-25b-53, 25b-53]
  # What vector.g, vector1.g and vector2.g prove of the turned direction.
  /\ da0 in [-20b-53, 20b-53] /\ da1 in [-20b-53, 20b-53]
  /\ da2 in [-20b-53, 20b-53]
->
  DENQ - 1 in ? /\ EaEa - 1 in ? /\ NUMQ / DENQ - 1 in ?
  /\ S in ? /\ TI in ? /\ DR in ?
  /\ S + TI + DR in [0, 256b-53]
}

# |Ex(y)|^2 is |y|^2 and the turn's length share (464.20's 3), and |v x y|^2
# is at least zero, a sum of squares, and at most v.v y.y; v.v is q.q less
# w^2. |Ex(a)|^2 is the same with a along z.
EE -> YY + 4 * (N4 - 1) * VY;
VY -> VV * YY - VD * VD;
VV -> N4 - w * w;
EaEa -> AA * DENQ;
S01 -> N4 - w * w - v2 * v2;

# NUMQ / DENQ off one: the two differ by 4 (q.q - 1) (v1^2 + v2^2) and a
# share in (q.q - 1)^2.
NUMQ / DENQ -> 1 + (NUMQ - DENQ) / DENQ;
NUMQ - DENQ -> 4 * (N4 - 1) * S12 + 16 * (N4 - 1) * (N4 - 1) * VV * (v1 * v1);
S12 -> N4 - w * w - v0 * v0;
