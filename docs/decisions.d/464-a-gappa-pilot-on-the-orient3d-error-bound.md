## 464. A Gappa pilot on the orient3d error bound — 2026-10-22

*Mattias asked what a proof system could buy rcad, and then, verbatim:*

> But which kind of bugs would we expect Gappa, FPTaylor, Lean 4 with Mathlib,
> to help us with? May these tools help us find/fix bugs our current testing
> would not find? May these tools make it cheaper to fix bug? Would we expect it
> to be beneficial when rcad stabilize and will place more focus on perfomance
> optimizations?

*and, after the answer below: "file the Gappa pilot row for orient3d". What this
section says about the three tools is what is known of them, not what has been
tried here: none of them is installed, and the pilot is what checks it.*

  464.1   The bugs a bound prover is for
  464.2   What each tool is expected to do, untried here
  464.3   What the pilot answers

### 464.1 The bugs a bound prover is for

One kind: a bound on rounding error that is wrong, or that holds only while a
condition nobody guarded holds. That kind is rare in the queue. On `main` at
`fd29e6f476839fead505e1a8036db5d9bb8b8b9b`, 2 of the 429 rows that are `todo`,
waiting or running name a `*_ROUNDOFF` or `*_ERROR_BOUND` constant, against 116
whose write set is in `crates/rcad-topo/src/fuse/`. But it is the worst kind
there is, a certified wrong answer, and the suite has let it through before:

- **§12.2.** `orient3d` certified `Pos` on a determinant of `−2⁻⁷⁶`. Its bound
  is relative, and holds only while every product is a normal number. The
  differential test against `robust` could not see it, because `robust` is
  wrong the same way.
- **§16.9.** de Boor evaluation answered `0.0` where the truth is `1.5`, and
  certified it to `4.5e-14`, once the knots passed `f64::MAX`. The suite was 549
  green; review found it. It was §12.2's mistake in a third place.
- **§15.6.** A `3·ε` called rigorous was not a bound. It had its derivation
  written beside it and was still wrong, because nothing ran the derivation.
- **§16.3.** Seven roundings were counted where there are nine. The
  exact-arithmetic test has about 300× of slack, so a bound wrong by a factor of
  two passes it.
- **§18.5.** `ORIENT3D_ERROR_BOUND` could be cut from 7ε to 1ε with the suite
  green. Finding the 6.286ε worst case took about 10⁸ searched inputs.

A test tries inputs, and the worst cases for rounding are rare. A prover covers
every input in the ranges it is given, so it can show a bound holds, which no
test can. §16.3 split the work in two: the derivation is what makes it a bound,
and the test is what stops the expression drifting out from under it. A checked
proof takes the derivation's place and leaves the test where it is.

It cannot see the rest of the queue: the fuse's logic, topology, and which case
a refusal falls in.

### 464.2 What each tool is expected to do, untried here

- **Gappa** proves a bound on one fixed expression, given a range for every
  input, and can write the proof out for Rocq to check. It has to show that
  every value on the way stays in range, so an overflow like §16.9's should
  stop it. It can state a bound against a second computed number, which
  is the shape `ORIENT3D_ERROR_BOUND` has: an error of at most 7ε times the
  permanent, the permanent computed too. Where one input appears many times it
  overestimates unless it is given hints.
- **FPTaylor** finds the tightest bound it can, so it says how much slack a
  published constant carries, such as §28.5's 256ε against a derived 34ε. It
  bounds one expression's error, as an amount or relative to the exact answer,
  and relative to the exact answer is no use for a sign, which is near zero
  just where it matters. As far as is known here it assumes nothing overflows,
  and its number rests on its own code.
- **Lean 4 with Mathlib**, or Rocq with Flocq, proves statements for every size:
  `EVALUATION_ROUNDOFF` per de Boor pass at any degree, the exact arithmetic in
  `crates/rcad-predicates/src/expansion.rs`, and the `inari` fork's outward step
  (§460). That last one is `inari`'s own code, and its proof belongs there.

So the pilot is Gappa's. It states rcad's bound as the code writes it, it should
see an overflow, and its proof gets checked. FPTaylor, to find tight numbers, and
Lean, for the any-degree bounds, wait on what it finds.

There are twenty bound constants outside tests, found by `grep -rnE 'const
[A-Z0-9_]*(ROUNDOFF|ERROR_BOUND)[A-Z0-9_]*\s*:' crates`: 17 in `rcad-geom`, 3 in
`rcad-predicates`. The case for proving them grows once rcad turns to speed.
Every rewrite of an expression for speed (another order, an FMA, two lanes at
once) moves its rounding, so its derivation has to be done again, and §16.3's
test could not tell a bound wrong by a factor of two. A tighter bound may also let the `f64` level decide
more often, but how often each level runs today has not been measured.

### 464.3 What the pilot answers

On `Orient3d::on_floats` and `ORIENT3D_ERROR_BOUND`, in
`crates/rcad-predicates/src/orient3d.rs`:

1. **How Gappa is pinned.** `mise registry` names no Gappa, Rocq or FPTaylor,
   only `opam`; Ubuntu offers `gappa` 1.4.1-1 through apt. CLAUDE.md says never
   to install a tool ad hoc, so name the pin, what it installs and how long that
   takes, or stop and say why there is none.
2. **Whether it proves the bound as the code states it**: `|value − exact| ≤
   ORIENT3D_ERROR_BOUND × permanent`, where `value` and `permanent` are what
   `on_floats` computes and `exact` is the true determinant of the same `f64`
   inputs. §12.2's two conditions go in as hypotheses: every product normal,
   which `product_is_normal` checks, and the bound itself normal, which `decide`
   checks. Say how long it takes, and whether Rocq checks the proof it writes.
3. **Whether it would have caught §12.2.** Drop the hypothesis that every product
   is normal, and say whether the proof fails, and where.
4. **Whether it would have caught a miscount like §16.3's.** Ask for 6ε, below
   §18.5's 6.286ε, and the proof must fail. Say the smallest multiple it proves.
5. **What keeps the script the same expression as the Rust.** Say what would
   notice if `on_floats` changed and the script did not.
6. **What the other nineteen would cost** done the same way. File a row for them,
   or say why not.

Quote the script whole in this section, so the result can be run again. Whether
a script joins the tree, and the gate, is for the row the pilot files.

### 464.4 DRAFT, unfinished: the script as Q2647's second lane left it

*Q2647's relay 1 committed this so the next lane does not lose it. It is not
an answer yet. The next lane replaces this subsection with 464.4 onward.*

The pins, both measured:

- Gappa 1.8.0: `nix --extra-experimental-features 'nix-command flakes' shell
  github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe#gappa --command
  gappa --version`. It took 1m0.926s and fetched 15.25 MiB.
- Coq 8.20.1 with gappalib 1.7.0 and flocq 4.2.2, from the same nixpkgs, as
  `#coqPackages_8_20.coq`, `#coqPackages_8_20.gappalib` and
  `#coqPackages_8_20.flocq`. `rocqPackages` and `coqPackages_9_*` have no
  gappalib. It took 1m10.914s, fetched 583.11 MiB and built gappalib locally.

Where the draft stands. Gappa parses it, takes every hint as a field identity
and runs in about 10 to 15 s. It bounds the ratios of the differences, the
minors and `s12` within `[1 - 2^-53, 1 + 2^-53]`. It does not bound the
products (`rp1`, `rt1`), because `rel_float_of_abs` in Gappa's
`src/arithmetic/float.cpp`, lines 426-437, refuses an ABS range that touches
`[-2^-1022, 2^-1022]`. Its own end is included, so `|x| >= 1b-1022` is one ulp
short. The goal `(y - Ex) / eb in [-1, 1]` therefore fails.

```gappa
# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,
# crates/rcad-predicates/src/orient3d.rs at d9f931e8.
@rnd = float<ieee_64, ne>;

# The twelve input coordinates, each an f64.
ax = rnd(ax_); ay = rnd(ay_); az = rnd(az_);
bx = rnd(bx_); by = rnd(by_); bz = rnd(bz_);
cx = rnd(cx_); cy = rnd(cy_); cz = rnd(cz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);

# What on_floats computes, line for line.
Adx = ax - dx; Ady = ay - dy; Adz = az - dz;
Bdx = bx - dx; Bdy = by - dy; Bdz = bz - dz;
Cdx = cx - dx; Cdy = cy - dy; Cdz = cz - dz;
adx = rnd(Adx); ady = rnd(Ady); adz = rnd(Adz);
bdx = rnd(Bdx); bdy = rnd(Bdy); bdz = rnd(Bdz);
cdx = rnd(Cdx); cdy = rnd(Cdy); cdz = rnd(Cdz);

bdxcdy = rnd(bdx * cdy);
cdxbdy = rnd(cdx * bdy);
cdxady = rnd(cdx * ady);
adxcdy = rnd(adx * cdy);
adxbdy = rnd(adx * bdy);
bdxady = rnd(bdx * ady);

M1 = bdxcdy - cdxbdy; M2 = cdxady - adxcdy; M3 = adxbdy - bdxady;
first = rnd(M1); second = rnd(M2); third = rnd(M3);

U1 = adz * first; U2 = bdz * second; U3 = cdz * third;
t1 = rnd(U1); t2 = rnd(U2); t3 = rnd(U3);
S = t1 + t2;
s12 = rnd(S);
y = s12 + t3;
value = rnd(y);

q1 = rnd(|bdxcdy| + |cdxbdy|);
q2 = rnd(|cdxady| + |adxcdy|);
q3 = rnd(|adxbdy| + |bdxady|);
r1 = rnd(q1 * |adz|); r2 = rnd(q2 * |bdz|); r3 = rnd(q3 * |cdz|);
P = rnd(rnd(r1 + r2) + r3);

B = 7b-53 + 7b-103;
eb = rnd(B * P);

# The true determinant of the same inputs.
Ex = Adz * (Bdx * Cdy - Cdx * Bdy)
   + Bdz * (Cdx * Ady - Adx * Cdy)
   + Cdz * (Adx * Bdy - Bdx * Ady);

# The permanent with every rounding taken out, twice: as P groups it, and
# as six products.
Phat = (|bdxcdy| + |cdxbdy|) * |adz|
     + (|cdxady| + |adxcdy|) * |bdz|
     + (|adxbdy| + |bdxady|) * |cdz|;
c1 = |adz| * |bdxcdy|; c2 = |adz| * |cdxbdy|;
c3 = |bdz| * |cdxady|; c4 = |bdz| * |adxcdy|;
c5 = |cdz| * |adxbdy|; c6 = |cdz| * |bdxady|;
P6 = c1 + c2 + c3 + c4 + c5 + c6;

# What each rounding multiplied its argument by.
radx = adx / Adx; rady = ady / Ady; radz = adz / Adz;
rbdx = bdx / Bdx; rbdy = bdy / Bdy; rbdz = bdz / Bdz;
rcdx = cdx / Cdx; rcdy = cdy / Cdy; rcdz = cdz / Cdz;
rp1 = bdxcdy / (bdx * cdy); rp2 = cdxbdy / (cdx * bdy);
rp3 = cdxady / (cdx * ady); rp4 = adxcdy / (adx * cdy);
rp5 = adxbdy / (adx * bdy); rp6 = bdxady / (bdx * ady);
rm1 = first / M1; rm2 = second / M2; rm3 = third / M3;
rt1 = t1 / U1; rt2 = t2 / U2; rt3 = t3 / U3;
rs = s12 / S;

# Each of the six terms of the determinant, as y carries it, less the
# term itself.
W1 = rm1 * rt1 * rs - 1 / (radz * rbdx * rcdy * rp1);
W2 = rm1 * rt1 * rs - 1 / (radz * rcdx * rbdy * rp2);
W3 = rm2 * rt2 * rs - 1 / (rbdz * rcdx * rady * rp3);
W4 = rm2 * rt2 * rs - 1 / (rbdz * radx * rcdy * rp4);
W5 = rm3 * rt3 - 1 / (rcdz * radx * rbdy * rp5);
W6 = rm3 * rt3 - 1 / (rcdz * rbdx * rady * rp6);
e1 = adz * bdxcdy * W1; e2 = -(adz * cdxbdy * W2);
e3 = bdz * cdxady * W3; e4 = -(bdz * adxcdy * W4);
e5 = cdz * adxbdy * W5; e6 = -(cdz * bdxady * W6);

{
  # Every coordinate is a finite f64.
     |ax| <= 1b1024 /\ |ay| <= 1b1024 /\ |az| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024 /\ |bz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024 /\ |cz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  # No factor of the nine products is zero. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |adx| >= 1b-1074 /\ |ady| >= 1b-1074 /\ |adz| >= 1b-1074
  /\ |bdx| >= 1b-1074 /\ |bdy| >= 1b-1074 /\ |bdz| >= 1b-1074
  /\ |cdx| >= 1b-1074 /\ |cdy| >= 1b-1074 /\ |cdz| >= 1b-1074
  /\ |first| >= 1b-1074 /\ |second| >= 1b-1074 /\ |third| >= 1b-1074
  # §12.2, first condition: the nine products are normal, which is what
  # product_is_normal checks once neither factor is zero.
  /\ |bdxcdy| >= 1b-1022 /\ |cdxbdy| >= 1b-1022 /\ |cdxady| >= 1b-1022
  /\ |adxcdy| >= 1b-1022 /\ |adxbdy| >= 1b-1022 /\ |bdxady| >= 1b-1022
  /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022 /\ |t3| >= 1b-1022
  # The permanent's own three products are normal. The comment above
  # stays_accurate argues it from the nine; Gappa takes it as given.
  /\ |r1| >= 1b-1022 /\ |r2| >= 1b-1022 /\ |r3| >= 1b-1022
  # §12.2, second condition: the bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # The first two terms do not cancel to zero.
  /\ |t1 + t2| >= 1b-1074
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ adz / |adz| in [-1, 1] /\ bdz / |bdz| in [-1, 1] /\ cdz / |cdz| in [-1, 1]
  /\ bdxcdy / |bdxcdy| in [-1, 1] /\ cdxbdy / |cdxbdy| in [-1, 1]
  /\ cdxady / |cdxady| in [-1, 1] /\ adxcdy / |adxcdy| in [-1, 1]
  /\ adxbdy / |adxbdy| in [-1, 1] /\ bdxady / |bdxady| in [-1, 1]
->
  |bdxcdy| in ? /\ bdxcdy -/ bdx * cdy in ? /\ (bdxcdy - bdx * cdy) / (bdx * cdy) in ? /\ t1 -/ U1 in ?
  /\ radz in ? /\ rp1 in ? /\ rm1 in ? /\ rt1 in ? /\ rs in ? /\ W1 in ? /\ W5 in ?
  /\ e1 / c1 in ? /\ (c1 + e1) -/ c1 in ? /\ c1 / (c1 + c2) in ?
  /\ (y - Ex) / P6 in ? /\ P -/ Phat in ? /\ eb -/ B * P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
adx / Adx -> 1 + (adx - Adx) / Adx;
ady / Ady -> 1 + (ady - Ady) / Ady;
adz / Adz -> 1 + (adz - Adz) / Adz;
bdx / Bdx -> 1 + (bdx - Bdx) / Bdx;
bdy / Bdy -> 1 + (bdy - Bdy) / Bdy;
bdz / Bdz -> 1 + (bdz - Bdz) / Bdz;
cdx / Cdx -> 1 + (cdx - Cdx) / Cdx;
cdy / Cdy -> 1 + (cdy - Cdy) / Cdy;
cdz / Cdz -> 1 + (cdz - Cdz) / Cdz;
bdxcdy / (bdx * cdy) -> 1 + (bdxcdy - bdx * cdy) / (bdx * cdy);
cdxbdy / (cdx * bdy) -> 1 + (cdxbdy - cdx * bdy) / (cdx * bdy);
cdxady / (cdx * ady) -> 1 + (cdxady - cdx * ady) / (cdx * ady);
adxcdy / (adx * cdy) -> 1 + (adxcdy - adx * cdy) / (adx * cdy);
adxbdy / (adx * bdy) -> 1 + (adxbdy - adx * bdy) / (adx * bdy);
bdxady / (bdx * ady) -> 1 + (bdxady - bdx * ady) / (bdx * ady);
first / M1 -> 1 + (first - M1) / M1;
second / M2 -> 1 + (second - M2) / M2;
third / M3 -> 1 + (third - M3) / M3;
t1 / U1 -> 1 + (t1 - U1) / U1;
t2 / U2 -> 1 + (t2 - U2) / U2;
t3 / U3 -> 1 + (t3 - U3) / U3;
s12 / S -> 1 + (s12 - S) / S;

# The error is the sum of the six terms' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / P6 -> ((P6 + (y - Ex)) - P6) / P6;
P6 + (y - Ex) -> (c1 + e1) + (c2 + e2) + (c3 + e3) + (c4 + e4) + (c5 + e5) + (c6 + e6);
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(c4 + e4) - c4 -> e4;
(c5 + e5) - c5 -> e5;
(c6 + e6) - c6 -> e6;
e1 / c1 -> (adz / |adz|) * (bdxcdy / |bdxcdy|) * W1;
e2 / c2 -> -((adz / |adz|) * (cdxbdy / |cdxbdy|) * W2);
e3 / c3 -> (bdz / |bdz|) * (cdxady / |cdxady|) * W3;
e4 / c4 -> -((bdz / |bdz|) * (adxcdy / |adxcdy|) * W4);
e5 / c5 -> (cdz / |cdz|) * (adxbdy / |adxbdy|) * W5;
e6 / c6 -> -((cdz / |cdz|) * (bdxady / |bdxady|) * W6);

# From the six terms' permanent to the one on_floats rounds, and on to eb.
P ~ Phat;
(y - Ex) / eb -> ((y - Ex) / P6) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```
