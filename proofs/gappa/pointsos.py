"""Check the identity point.g's D / L hint rests on, in exact rationals, at
random points: 12|w|s + 14|v0|s + 14(|v1|^2 + |v2|^2), s = |v1| + |v2|,
equals K N4 less four squares, K = 109/5, N4 = |w|^2 + |v0|^2 + |v1|^2 + |v2|^2.
Then the largest D / L over unit quaternions, by a grid, against K + 3."""

from fractions import Fraction as F
import math
import random

K = F(109, 5)
random.seed(1)
for _ in range(1000):
    aw, a0, a1, a2 = (F(random.randint(0, 10**6), 10**6) for _ in range(4))
    s = a1 + a2
    lhs = 12 * aw * s + 14 * a0 * s + 14 * (a1 * a1 + a2 * a2)
    X = aw - F(30, 109) * s
    Y = a0 - F(35, 109) * s
    Z = a1 - a2
    rhs = K * (aw * aw + a0 * a0 + a1 * a1 + a2 * a2) - (
        K * X * X + K * Y * Y + F(425, 109) * Z * Z + F(1, 545) * (a1 * a1 + a2 * a2)
    )
    assert lhs == rhs
print("identity holds at 1000 random points")
print("K =", float(K), "root of K^2 - 14K - 170:", 7 + math.sqrt(219))

best = 0.0
n = 200
for i in range(n + 1):
    for j in range(n + 1):
        for k in range(n + 1):
            aw, a1, a2 = i / n, j / n, k / n
            r = 1 - aw * aw - a1 * a1 - a2 * a2
            if r < 0:
                continue
            a0 = math.sqrt(r)
            s = a1 + a2
            f = 12 * aw * s + 14 * a0 * s + 14 * (a1 * a1 + a2 * a2) + 3
            best = max(best, f)
print("largest D / L on a 200-step grid of unit quaternions:", best)
