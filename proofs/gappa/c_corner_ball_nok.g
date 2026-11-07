# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/sphere.rs transformed at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a ball,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner x exactly on a
# ball of centre c and radius r, moved, is within 128 * f64::EPSILON * L of
# the moved ball. Sphere::transformed moves the centre by apply_to_point and
# carries r over. Ex, nalgebra's turn without rounding, is linear (sos.py),
# so the moved corner less the moved centre is Ex(x - c) + kp - kc, kp and kc
# the two slips point.g proves. By the triangle inequality the distance to
# the moved ball is at most |kp - kc| + r * | |Ex(y)| - 1 |, y = (x - c) / r,
# which is what this bounds, per unit of L.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;

# nalgebra's turn of the unit vector y without rounding, as point.g's Ex
# writes x, and its length squared.
Ex0 = 2 * w * (v1 * y2 - v2 * y1)
    + v1 * (2 * (v0 * y1 - v1 * y0)) - v2 * (2 * (v2 * y0 - v0 * y2)) + y0;
Ex1 = 2 * w * (v2 * y0 - v0 * y2)
    + v2 * (2 * (v1 * y2 - v2 * y1)) - v0 * (2 * (v0 * y1 - v1 * y0)) + y1;
Ex2 = 2 * w * (v0 * y1 - v1 * y0)
    + v0 * (2 * (v2 * y0 - v0 * y2)) - v1 * (2 * (v1 * y2 - v2 * y1)) + y2;
EE = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;
YY = y0 * y0 + y1 * y1 + y2 * y2;

# v x y and v . y, without rounding, and v.v.
X0 = v1 * y2 - v2 * y1; X1 = v2 * y0 - v0 * y2; X2 = v0 * y1 - v1 * y0;
VY = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * y0 + v1 * y1 + v2 * y2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

# The two slips, the moved corner's and the moved centre's, per unit of L.
k0 = kp0 - kc0; k1 = kp1 - kc1; k2 = kp2 - kc2;
KK = k0 * k0 + k1 * k1 + k2 * k2;

# The distance's two shares, per unit of L: the slips', and the radius's
# times how far the turn's arithmetic takes a unit vector off length one.
S = sqrt(KK);
T = RL * |sqrt(EE) - 1|;

{
  # Every quaternion component is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # y is the corner less the centre over r, a unit vector.
  /\ YY in [1, 1]
  /\ y0 in [-1, 1] /\ y1 in [-1, 1] /\ y2 in [-1, 1]
  # r / L: L is at least the radius.
  /\ RL in [0, 1]
->
  EE - 1 in ? /\ S in ? /\ T in ?
  /\ S + T in [0, 256b-53]
}

# |Ex(y)|^2 is |y|^2 and the turn's length share (464.20's 3), and |v x y|^2
# is at least zero, a sum of squares, and at most v.v y.y; v.v is q.q less w^2.
EE -> YY + 4 * (N4 - 1) * VY;
VY -> VV * YY - VD * VD;
VV -> N4 - w * w;
