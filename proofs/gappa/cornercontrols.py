"""Write the controls of ball.g, plane.g, pipe.g and torus.g, pipe.g's run with
the slips at POINT_ROUNDOFF's 36u, and point.g's runs at 25u. Each control
changes one thing: a smaller bound, to see whether the script proved its goal
from hypotheses that contradict each other; or a hypothesis dropped or
weakened, to see whether the proof needs it. r_point_25.g is 464.19's point.g
with its goal at 25u, and r_point1_25.g and r_point2_25.g are its y and z,
turned as 464.19's turn.py turns point.g."""

import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


N4 = "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]\n"
# pipe.g and torus.g open their hypotheses with q.q, so it has no /\.
N4_FIRST = "     N4 in [9007199254740980b-53, 9007199254741004b-53]\n"
# q.q only as large as the bounds on w and v let it be.
N4_WIDE = "     N4 in [0, 16]\n"
WV = ("  # Each of w^2, v0^2, v1^2 and v2^2 is at most q.q, under 1 + 12u, so each\n"
      "  # of w, v0, v1 and v2 is at most 2 in size.\n"
      "  /\\ |w| <= 2 /\\ |v0| <= 2 /\\ |v1| <= 2 /\\ |v2| <= 2\n")
AXIS_GOAL = "/\\ S + TI + DR in [0, 256b-53]"

ball = open("corner_ball.g").read()
# 148u, below the 124.7u + 24u the two shares reach.
write("c_corner_ball_148.g", sub(ball, "/\\ S + T in [0, 256b-53]", "/\\ S + T in [0, 148b-53]"))
# Without |q.q - 1| <= 6 epsilon.
write("c_corner_ball_non4.g", sub(ball, N4, ""))
# Without what point.g proves of the two slips.
write("c_corner_ball_nok.g", sub(ball, (
    "  # What point.g, point1.g and point2.g prove of the moved corner and the\n"
    "  # moved centre, L at least every coordinate of either and of the shift.\n"
    "  /\\ kp0 in [-36b-53, 36b-53] /\\ kp1 in [-36b-53, 36b-53]\n"
    "  /\\ kp2 in [-36b-53, 36b-53]\n"
    "  /\\ kc0 in [-36b-53, 36b-53] /\\ kc1 in [-36b-53, 36b-53]\n"
    "  /\\ kc2 in [-36b-53, 36b-53]\n"), ""))

plane = open("corner_plane.g").read()
# 222u, below the 83.14u + 60u + 62.35u + 17u the four shares reach.
write("c_corner_plane_222.g", sub(plane, "/\\ T / sqrt(NPNP) in [-256b-53, 256b-53]",
                           "/\\ T / sqrt(NPNP) in [-222b-53, 222b-53]"))
# Without |q.q - 1| <= 6 epsilon: the turn's length share.
write("c_corner_plane_non4.g", sub(plane, N4, ""))
# Without what vector.g proves of the turned normal.
write("c_corner_plane_nodn.g", sub(plane, (
    "  # What vector.g, vector1.g and vector2.g prove of the turned normal.\n"
    "  /\\ dn0 in [-20b-53, 20b-53] /\\ dn1 in [-20b-53, 20b-53]\n"
    "  /\\ dn2 in [-20b-53, 20b-53]\n"), ""))
# Without what point.g proves of the moved corner.
write("c_corner_plane_nodp.g", sub(plane, (
    "  # What point.g, point1.g and point2.g prove of the moved corner, over L.\n"
    "  /\\ DP0 in [-36b-53, 36b-53] /\\ DP1 in [-36b-53, 36b-53]\n"
    "  /\\ DP2 in [-36b-53, 36b-53]\n"), ""))
# Without what shiftplane.g proves of the offset.
write("c_corner_plane_nodd.g", sub(plane, (
    "  # What shiftplane.g proves of the offset's rounding over L, 16u times\n"
    "  # the larger of |n'| and one, with |n'| within 124u of one: under 17u.\n"
    "  /\\ DD in [-17b-53, 17b-53]\n"), ""))

pipe = open("corner_pipe.g").read()
# 230u, below the 86.6u + 120u + 24u the three shares reach.
write("c_corner_pipe_230.g", sub(pipe, AXIS_GOAL, "/\\ S + TI + DR in [0, 230b-53]"))
write("c_corner_pipe_non4.g", sub(pipe, N4_FIRST, N4_WIDE))
# Without the bounds on w and v: the first run of pipe.g.
write("c_corner_pipe_nobound.g", sub(pipe, WV, ""))
# Without what vector.g proves of the turned direction.
write("c_corner_pipe_noda.g", sub(pipe, (
    "  # What vector.g, vector1.g and vector2.g prove of the turned direction.\n"
    "  /\\ da0 in [-20b-53, 20b-53] /\\ da1 in [-20b-53, 20b-53]\n"
    "  /\\ da2 in [-20b-53, 20b-53]\n"), ""))
# The six slips at POINT_ROUNDOFF's 36u, as the published constant has them.
assert pipe.count("[-25b-53, 25b-53]") == 6
write("c_corner_pipe_36.g", pipe.replace("[-25b-53, 25b-53]", "[-36b-53, 36b-53]"))

torus = open("corner_torus.g").read()
# 212u, below the 124.7u + 34.6u + 53.7u the three shares reach.
write("c_corner_torus_212.g", sub(torus, AXIS_GOAL, "/\\ S + TI + DR in [0, 212b-53]"))
write("c_corner_torus_non4.g", sub(torus, N4_FIRST, N4_WIDE))

# 464.19's turn.py, whole but for reading and writing the files.
TURNED = {"v", "p", "s", "P", "q", "C", "c", "rq", "rc", "t", "k", "rk"}
NAME = re.compile(r"\b([A-Za-z]+)(\d)(\d?)(_?)\b")


def turn(src):
    def one(m):
        prefix, index, rest, under = m.groups()
        if prefix not in TURNED:
            return m.group(0)
        return prefix + str((int(index) + 1) % 3) + rest + under

    return NAME.sub(one, src)


point = open("point.g").read()
point = sub(point, "x is within 18 * f64::EPSILON", "x is within 12.5 * f64::EPSILON")
point = sub(point, "/\\ (out - Ex) / L in [-36b-53, 36b-53]", "/\\ (out - Ex) / L in [-100b-55, 100b-55]")
write("point_25.g", point)
once = turn(point)
twice = turn(once)
assert turn(twice) == point
X = "# moved point's x is within"
for name, src, axis in (("point1_25.g", once, "y"), ("point2_25.g", twice, "z")):
    write(name, sub(src, X, "# moved point's " + axis + " is within"))
