# restates: crates/rcad-geom/src/de_boor.rs EVALUATION_ROUNDOFF at 812503ed
# restates: crates/rcad-geom/src/de_boor.rs de_boor at 812503ed
# restates: crates/rcad-geom/src/de_boor.rs blended at 812503ed
# One coordinate of one de Boor stage as de_boor and blended compute it in
# f64: share = (at - below) / (above - below), rest = 1 - share, and
# out = rest * d0 + share * d1, each step rounded once. d0 and d1 are the
# computed running values, each within E of the true ones t0 and t1, and M is
# at least |t0| and |t1|. This proves the doc's recurrence,
# |out - Ex| <= (1 + 9u) E + 9u M, for E from 2^-60 M to 2^-40 M and share as
# the hypotheses have it: strictly between 0 and 1 here, and at 0 and at 1 in
# the two scripts controls.py writes from this one. stage_e0.g takes E below
# 2^-60 M.
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

# The stage worked without rounding, on the true running values.
A = (at - below) / (above - below);
Ex = (1 - A) * t0 + A * t1;

# out - Ex is XE + XM: XE carries the incoming errors through, and XM is what
# this stage rounds, XMM per unit of M. The bound is cE + cM.
XE = rest * (d0 - t0) + share * (d1 - t1);
cR = 2 * rest * E; cS = 2 * share * E;
V = cR + cS;
XM = (out - S) + (P0 - z0) + (P1 - z1) + (rest - (1 - share)) * t0 + (share - A) * (t1 - t0);
XMM = (out - S) / M + (P0 - z0) / M + (P1 - z1) / M
    + (rest - (1 - share)) * (t0 / M) + (share - A) * (t1 / M - t0 / M);
cE = (1 + 9b-53) * E;
cM = 17b-54 * M;
D = cE + cM;

{
     |at| <= 1b1024 /\ |below| <= 1b1024 /\ |above| <= 1b1024
  /\ above - below in [1b-1074, 1b1024]
  /\ A in [0, 1]
  /\ share in [1b-1074, 9007199254740991b-53]
  /\ M in [1b-1022, 1b1024]
  /\ t0 / M in [-1, 1] /\ t1 / M in [-1, 1]
  /\ E / M in [1b-60, 1b-40]
  /\ (d0 - t0) / E in [-1, 1] /\ (d1 - t1) / E in [-1, 1]
->
  share - A in ? /\ rest - (1 - share) in ? /\ XE / V in ? /\ XMM in ?
  /\ (out - S) / M in ? /\ (P0 - z0) / M in ? /\ (P1 - z1) / M in ?
  /\ XE / cE in ? /\ XM / cM in ?
  /\ (out - Ex) / D in [-1, 1]
}

# (out - Ex) / D is the average of XE / cE and XM / cM, weighted by cE and cM.
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (cE + XE) + (cM + XM);
(cE + XE) - cE -> XE;
(cM + XM) - cM -> XM;

# XE / cE through the average of the two incoming errors, over twice their
# weights. Over the weights themselves, as r_stage_plain.g has it, Gappa
# bounds XE / V by 1.5 and not by 1.
XE / cE -> (XE / V) * (2 * (rest + share) / (1 + 9b-53));
rest + share -> 1 + (rest - (1 - share));
XE / V -> ((V + XE) - V) / V;
V + XE -> (cR + rest * (d0 - t0)) + (cS + share * (d1 - t1));
(cR + rest * (d0 - t0)) - cR -> rest * (d0 - t0);
(cS + share * (d1 - t1)) - cS -> share * (d1 - t1);
rest * (d0 - t0) / cR -> ((d0 - t0) / E) / 2;
share * (d1 - t1) / cS -> ((d1 - t1) / E) / 2;

# XM a share at a time, each rounding relative to what it rounds and that
# per unit of M.
XM / cM -> XMM / 17b-54;
(P0 - z0) / M -> ((P0 - z0) / z0) * (z0 / M);
(P1 - z1) / M -> ((P1 - z1) / z1) * (z1 / M);
(out - S) / M -> ((out - S) / S) * (S / M);
z0 / M -> rest * (d0 / M);
z1 / M -> share * (d1 / M);
d0 / M -> t0 / M + ((d0 - t0) / E) * (E / M);
d1 / M -> t1 / M + ((d1 - t1) / E) * (E / M);
S / M -> (P0 - z0) / M + (P1 - z1) / M + z0 / M + z1 / M;

# Where 1 - share starts to be exact.
$ share in (0.5);
# Where each product and the sum leaves the normal range, below which its
# rounding is absolute.
$ z0 in (-1b-1022, 1b-1022);
$ z1 in (-1b-1022, 1b-1022);
$ S in (-1b-1022, 1b-1022);
