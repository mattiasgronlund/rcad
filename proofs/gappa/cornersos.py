"""Check, in exact rationals at random points, the identities 464.21's scripts
rest on, for nalgebra's turn worked without rounding, Ex(y) = t w + v x t + y
with t = 2 (v x y), N4 = q.q and eta = N4 - 1:

1. Ex is linear: Ex(a) - Ex(b) = Ex(a - b), so a shift added after it cancels
   from a difference of two moved points exactly.
2. Ex(a) . Ex(b) - a . b = 4 eta (v x a) . (v x b), 464.20's identity polarized.
3. |Ex(y) x Ex(a)|^2 = |W|^2 + 4 eta (|v|^2 |W|^2 + (v . W)^2)
                       + 16 eta^2 |v|^2 (v . W)^2, with W = y x a.
4. (v x a) . (v x b) = (v . v)(a . b) - (v . a)(v . b), Lagrange's identity,
   which with a = b is 464.20's |v x n|^2 = |v|^2 |n|^2 - (v . n)^2.
5. Ex turns with the frame: for a rotation G, Ex with v turned to G v, of
   G y, is G Ex(y). So pipe.g and torus.g may put the axis along z and the
   corner in the xz plane.
6. In that frame, a = (0, 0, al) and y = (rho, 0, t), with
   DENQ = 1 + 4 eta (v0^2 + v1^2) and NUMQ = 1 + 4 eta (v.v + v1^2)
   + 16 eta^2 (v.v) v1^2:
   |Ex(a)|^2 = al^2 DENQ, |Ex(y) x Ex(a)|^2 = rho^2 al^2 NUMQ, and
   Ex(y) . Ex(a) = al (t DENQ - 4 eta v0 v2 rho)."""

from fractions import Fraction as F
import random

random.seed(1)


def r():
    return F(random.randint(-10**6, 10**6), 10**6)


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def ex(w, v, y):
    t = [2 * c for c in cross(v, y)]
    k = cross(v, t)
    return [t[i] * w + k[i] + y[i] for i in range(3)]


for _ in range(1000):
    w, v = r(), [r(), r(), r()]
    a, b = [r(), r(), r()], [r(), r(), r()]
    eta = w * w + dot(v, v) - 1
    assert [p - q for p, q in zip(ex(w, v, a), ex(w, v, b))] == ex(w, v, [p - q for p, q in zip(a, b)])
    assert dot(ex(w, v, a), ex(w, v, b)) - dot(a, b) == 4 * eta * dot(cross(v, a), cross(v, b))
    W = cross(a, b)
    big = cross(ex(w, v, a), ex(w, v, b))
    assert dot(big, big) == (dot(W, W) + 4 * eta * (dot(v, v) * dot(W, W) + dot(v, W) ** 2)
                             + 16 * eta * eta * dot(v, v) * dot(v, W) ** 2)
    assert dot(cross(v, a), cross(v, b)) == dot(v, v) * dot(a, b) - dot(v, a) * dot(v, b)
print("identities 1 to 4 hold at 1000 random points")


def rotation(g):
    """The rotation matrix of the rational quaternion g, which is rational."""
    s, x, y, z = g
    n = s * s + x * x + y * y + z * z
    return [[(s * s + x * x - y * y - z * z) / n, 2 * (x * y - s * z) / n, 2 * (x * z + s * y) / n],
            [2 * (x * y + s * z) / n, (s * s - x * x + y * y - z * z) / n, 2 * (y * z - s * x) / n],
            [2 * (x * z - s * y) / n, 2 * (y * z + s * x) / n, (s * s - x * x - y * y + z * z) / n]]


def apply(m, y):
    return [dot(row, y) for row in m]


for _ in range(1000):
    w, v, y = r(), [r(), r(), r()], [r(), r(), r()]
    G = rotation([r(), r(), r(), r()])
    assert dot(apply(G, [1, 0, 0]), apply(G, [0, 1, 0])) == 0
    assert dot(apply(G, [1, 0, 0]), apply(G, [1, 0, 0])) == 1
    assert ex(w, apply(G, v), apply(G, y)) == apply(G, ex(w, v, y))
print("identity 5 holds at 1000 random points")

for _ in range(1000):
    w, v = r(), [r(), r(), r()]
    al, rho, t = r(), r(), r()
    eta = w * w + dot(v, v) - 1
    a, y = [0, 0, al], [rho, 0, t]
    DENQ = 1 + 4 * eta * (v[0] ** 2 + v[1] ** 2)
    NUMQ = 1 + 4 * eta * (dot(v, v) + v[1] ** 2) + 16 * eta * eta * dot(v, v) * v[1] ** 2
    ea, ey = ex(w, v, a), ex(w, v, y)
    assert dot(ea, ea) == al * al * DENQ
    c = cross(ey, ea)
    assert dot(c, c) == rho * rho * al * al * NUMQ
    assert dot(ey, ea) == al * (t * DENQ - 4 * eta * v[0] * v[2] * rho)
print("identity 6 holds at 1000 random points")
