"""Write two controls for each case signedall.py writes, as 464.24's
casecontrols.py did for its twelve:

- c_<case>_below.g asks for 1/256 under the bound the case's averaging plans,
  rounded down to a multiple of u/1024. It must fail: if it proved, the
  hypotheses would contradict each other, or the bound proved would not be the
  averaging's.
- c_<case>_anyl.g lets L go down to 2^-1074 (signed), or |n| L to 2^-1100 (the
  shift). It must fail where an amount is paid out of L; elsewhere it proves,
  so it is no control and is not written.

Three below controls prove, as 464.42 4 says: signed_ssz_u01, signed_szs_u02
and signed_zss_u12. They are not controls and are not written.

Run as: python3 signedallcontrols.py; it writes beside itself and prints one
line a control."""
from fractions import Fraction as F
import math

import signedall as S

PROVES_BELOW = {"signed_ssz_u01", "signed_szs_u02", "signed_zss_u12"}

for kind, st, unrounded, big in S.cases():
    name = S.name_of(kind, st, unrounded, big)
    live = [i for i in (0, 1, 2) if st[i] != "z"]
    rounded = [o for o in S.ops(live) if o not in unrounded]
    _, _, (bound, pay, m, _, _, _) = S.plan(kind, st, live, rounded, big)
    under = math.floor(bound * (1 - F(1, 256)) * 1024)
    lhs = S.KIND[kind]["goal_lhs"]
    goal = f"{lhs} in [-{under}b-63, {under}b-63]"
    src, _, _, _ = S.make(kind, st, unrounded, big, goal=goal)
    if name not in PROVES_BELOW:
        open(f"c_{name}_below.g", "w").write(src)
        print(f"c_{name}_below fail")
    if m["L"]:
        src, _, _, _ = S.make(kind, st, unrounded, big, anyl=True)
        open(f"c_{name}_anyl.g", "w").write(src)
        print(f"c_{name}_anyl fail")
