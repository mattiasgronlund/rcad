"""Check, in exact rationals at random points, the two identities square.g and
vector.g rest on, and look for how far the first-order bound goes.

1. nalgebra's turn of n, worked without rounding, is
   Ex = n(1 - 2|v|^2) + 2w(v x n) + 2v(v . n), and
   Ex . Ex - n . n = 4(q.q - 1)|v x n|^2, with |v x n|^2 = |v|^2 |n|^2 - (v . n)^2.
2. vector.g's D is 10|w|s + 12|v0|s + 12(|v1|^2 + |v2|^2) + 1, s = |v1| + |v2|,
   and its quadratic part is K q.q less four squares, K = 93/5."""

from fractions import Fraction as F
import math
import random

random.seed(1)


def r():
    return F(random.randint(-10**6, 10**6), 10**6)


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


for _ in range(1000):
    w, v, n = r(), [r(), r(), r()], [r(), r(), r()]
    # nalgebra: t = 2 (v x n), out = t w + v x t + n
    t = [2 * c for c in cross(v, n)]
    k = cross(v, t)
    ex = [t[i] * w + k[i] + n[i] for i in range(3)]
    N4 = w * w + dot(v, v)
    vxn = cross(v, n)
    assert dot(ex, ex) - dot(n, n) == 4 * (N4 - 1) * dot(vxn, vxn)
    assert dot(vxn, vxn) == dot(v, v) * dot(n, n) - dot(v, n) ** 2
print("drift identity holds at 1000 random points")

K = F(93, 5)
for _ in range(1000):
    aw, a0, a1, a2 = (abs(r()) for _ in range(4))
    s = a1 + a2
    lhs = 10 * aw * s + 12 * a0 * s + 12 * (a1 * a1 + a2 * a2)
    X = aw - F(25, 93) * s
    Y = a0 - F(10, 31) * s
    Z = a1 - a2
    rhs = K * (aw * aw + a0 * a0 + a1 * a1 + a2 * a2) - (
        K * X * X + K * Y * Y + F(305, 93) * Z * Z + F(19, 465) * (a1 * a1 + a2 * a2)
    )
    assert lhs == rhs
print("vector identity holds at 1000 random points")
print("K =", float(K), "root of K^2 - 12K - 122:", 6 + math.sqrt(158))

best = 0.0
n = 200
for i in range(n + 1):
    for j in range(n + 1):
        for k in range(n + 1):
            aw, a1, a2 = i / n, j / n, k / n
            rr = 1 - aw * aw - a1 * a1 - a2 * a2
            if rr < 0:
                continue
            a0 = math.sqrt(rr)
            s = a1 + a2
            f = 10 * aw * s + 12 * a0 * s + 12 * (a1 * a1 + a2 * a2) + 1
            best = max(best, f)
print("largest D on a 200-step grid of unit quaternions:", best)
