# restates: crates/rcad-geom/src/de_boor.rs EVALUATION_ROUNDOFF at 812503ed
# restates: crates/rcad-geom/src/de_boor.rs de_boor at 812503ed
# restates: crates/rcad-geom/src/de_boor.rs blended at 812503ed
# stage.g's stage with the computed running values within 2^-60 M of the
# true ones, stage 1's exact control points among them: |out - Ex| <= 9u M,
# which is at most (1 + 9u) E + 9u M for every E >= 0. share is as the
# hypotheses have it: strictly between 0 and 1 here, and at 0 and at 1 in the
# two scripts controls.py writes from this one.
@rnd = float<ieee_64, ne>;

at = rnd(at_); below = rnd(below_); above = rnd(above_);
d0 = rnd(d0_); d1 = rnd(d1_);

num = rnd(at - below);
den = rnd(above - below);
share = rnd(num / den);
rest = rnd(1 - share);
z0 = rest * d0; z1 = share * d1;
P0 = rnd(z0); P1 = rnd(z1);
S = P0 + P1;
out = rnd(S);

A = (at - below) / (above - below);
Ex = (1 - A) * t0 + A * t1;

XMM = (out - S) / M + (P0 - z0) / M + (P1 - z1) / M
    + (rest - (1 - share)) * (t0 / M) + (share - A) * (t1 / M - t0 / M);

{
     |at| <= 1b1024 /\ |below| <= 1b1024 /\ |above| <= 1b1024
  /\ above - below in [1b-1074, 1b1024]
  /\ A in [0, 1]
  /\ share in [1b-1074, 9007199254740991b-53]
  /\ M in [1b-1022, 1b1024]
  /\ t0 / M in [-1, 1] /\ t1 / M in [-1, 1]
  /\ (d0 - t0) / M in [-1b-60, 1b-60] /\ (d1 - t1) / M in [-1b-60, 1b-60]
->
  share - A in ? /\ rest - (1 - share) in ? /\ XMM in ?
  /\ (out - S) / M in ? /\ (P0 - z0) / M in ? /\ (P1 - z1) / M in ?
  /\ (out - Ex) / M in [-67b-57, 67b-57]
}

# (out - Ex) / M is XMM, what this stage rounds per unit of M, plus the
# incoming errors carried through. XMM is bounded as in stage.g.
(out - Ex) / M -> XMM + rest * ((d0 - t0) / M) + share * ((d1 - t1) / M);
(P0 - z0) / M -> ((P0 - z0) / z0) * (z0 / M);
(P1 - z1) / M -> ((P1 - z1) / z1) * (z1 / M);
(out - S) / M -> ((out - S) / S) * (S / M);
z0 / M -> rest * (d0 / M);
z1 / M -> share * (d1 / M);
d0 / M -> t0 / M + (d0 - t0) / M;
d1 / M -> t1 / M + (d1 - t1) / M;
S / M -> (P0 - z0) / M + (P1 - z1) / M + z0 / M + z1 / M;

# Where 1 - share starts to be exact.
$ share in (0.5);
# Where each product and the sum leaves the normal range, below which its
# rounding is absolute.
$ z0 in (-1b-1022, 1b-1022);
$ z1 in (-1b-1022, 1b-1022);
$ S in (-1b-1022, 1b-1022);
