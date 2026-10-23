## 464. A Gappa pilot on the orient3d error bound — 2026-10-22

*Mattias asked what a proof system could buy rcad, and then, verbatim:*

> But which kind of bugs would we expect Gappa, FPTaylor, Lean 4 with Mathlib,
> to help us with? May these tools help us find/fix bugs our current testing
> would not find? May these tools make it cheaper to fix bug? Would we expect it
> to be beneficial when rcad stabilize and will place more focus on perfomance
> optimizations?

*and, after the answer below: "file the Gappa pilot row for orient3d". What
464.1 to 464.3 say about the three tools is what was known of them before
anything was tried here, when none of them was installed. 464.4 to 464.6 are
the pilot, Q2647, and where it found 464.2 wrong 464.2 says so. 464.7 is
Q2658, which took up the two claims the pilot left unbacked.*

  464.1   The bugs a bound prover is for
  464.2   What each tool is expected to do, untried here
  464.3   What the pilot answers
  464.4   What the pilot found
  464.5   The scripts, whole
  464.6   What this leaves
  464.7   value past the bound, and every minor at zero run
  464.8   orient2d and the side of a plane (Q2659, unfinished)

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
  input, and can write the proof out for Rocq to check. It was expected to
  have to show that every value on the way stays in range, so that an
  overflow like §16.9's would stop it. The pilot found it does not: its
  `float<ieee_64>` has a smallest exponent and no largest one, so it never
  sees an overflow, and 464.4 says why `on_floats` does not need it to. It
  can state a bound against a second computed number, which
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

So the pilot is Gappa's. It states rcad's bound as the code writes it, and its
proof gets checked. It was also expected to see an overflow, which it does not
(464.4). FPTaylor, to find tight numbers, and
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

### 464.4 What the pilot found

Q2647 ran the pilot in three relays and wrote it up in a fourth, on 2026-10-22
and 2026-10-23, against `on_floats` as it stands at `d9f931e8`; neither
`orient3d.rs` nor `cascade.rs` has changed since, up to `a1c96c57`. `u` below
is `2⁻⁵³`, the `epsilon` of `ORIENT3D_ERROR_BOUND`'s doc comment, so the
code's constant is `B = (7 + 56u)u = 7u + 56u²`. Block 1 in 464.5 is
`on_floats` and its bound, written out line for line, and block 2 is a lemma
block 1 needs. Every run is at `-Eprecision=200`: at Gappa's default of 60
bits the `u²` terms are lost, and block 1's goal fails with the best bound it
finds at `±1.01562`.

**1. The pin.** Gappa 1.8.0 from nixpkgs at a fixed commit, `nix
--extra-experimental-features 'nix-command flakes' shell
github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe#gappa --command
gappa`. Nix is already on the machine and keeps everything in its store, so
nothing is installed ad hoc, and the commit fixes every byte. The first fetch
took 1m0.9s and 15.25 MiB. Coq 8.20.1 and flocq 4.2.2 come from the same
commit, as `#coqPackages_8_20.coq` and `#coqPackages_8_20.flocq`: 1m10.9s and
583 MiB the first time. The same commit's gappalib, the Coq library Gappa's
proofs are written against, is 1.7.0. So is every nixpkgs' gappalib for Coq
8.20, master included, and `rocqPackages` and `coqPackages_9_*` have none.
1.7.0 lacks four theorems that Gappa 1.8.0's proof of block 1 cites:
`rel_of_lin`, `lin_refl`, `lin_of_lin_rel2_p` and
`lin_float_error_of_lin_abs_ne`. gappalib 1.8.0, tagged 2026-02-19 ("added
support for Gappa 1.7", "compatibility from Coq 8.16 to Rocq 9.2"), has them.
The nix block in 464.5 builds it against the pinned Coq and flocq, in 19.6 s.
The other routes were worse: `mise registry` has no Gappa, conda-forge has
none, opam has gappa 1.6.0 and wants system libraries, and apt has 1.4.1 with
no pin.

**2. The bound.** Block 1 proves `|y − Ex| ≤ eb`, in about 4 s. `y = s12 +
t3` is the sum `on_floats` rounds to get `value`. `Ex` is the exact
determinant of the same `f64` inputs. `eb = rnd(B × P)` is the `error_bound`
`on_floats` returns, and `P` its permanent. The hypotheses are §12.2's two
conditions (the nine products normal, and `eb` normal), the twelve
coordinates finite, no factor zero (the zero cases are below), the
permanent's own three products `r1`, `r2`, `r3` normal, and the thirteen
relative errors block 2 gives from those thirteen normal numbers. On the way
Gappa prints `|y − Ex| / P6 ≤ 7u + 11u²`, where `P6` is the permanent with no
rounding at all, and `P` within `4u + 7u²` of its unrounded value.

That is the claim `decide` needs, but not the one the doc comments make.
`decide` certifies `Pos` when `value > eb`. `eb` is a double and rounding is
monotone, so `y ≤ eb` would give `value = rnd(y) ≤ rnd(eb) = eb`. So `y >
eb`, and `Ex ≥ y − eb > 0`. `Neg` is the same with the signs turned. So level
1's sign is right on every input that meets the hypotheses. This step is on
paper. So is putting block 2 into block 1: block 2's conclusion, taken at each
of the thirteen roundings, is block 1's hypothesis about that rounding.

The claim on `value` itself is not proved. The doc comment above
`level_one_stays_inside_its_error_bound_where_the_rounding_is_worst`
(`orient3d.rs:1392`) says `value` "never falls further from the true
determinant than `7*epsilon` of the permanent". Asked for `|value − Ex| ≤ eb`,
with one more ratio for the last rounding, Gappa fails at the code's `B` (best
`±1.14286`) and at `8u`, and proves it at `8u + 64u²`. `|value − Ex| / P6`
comes out at `8u + 8u²`. Not proving it does not make it false: §18.5's search
found no input past `6.286u`. (464.7 builds one at `7.99999u`, so it is false.)
The doc comment on `ORIENT3D_ERROR_BOUND`
counts "folding the three terms together with two additions" as one rounding,
but `y` takes seven roundings from the inputs and `value` takes eight. Q2658
is the row for it.

**Rocq checks the proof, modulo 39 equalities.** `gappa -Bcoq` writes block
1's proof as 10572 lines of Coq, and `coqc` under the pinned Coq with gappalib
1.8.0 accepts it: exit 0, in 7.1 s. Block 2's proof is 548 lines: exit 0, in
1.8 s. Gappa does not write the same proof each run; four earlier runs wrote
9947 to 10299 lines. Block 2's proof stands on its own. Block 1's rests on its
39 hints: each `a -> b;` becomes `Hypothesis aN : a = b.` in the Coq file,
with no condition attached. Gappa's manual says so (`doc/formalizing.rst` in
1.8.0): "when generating a theorem proof, Gappa adds this equality as a
hypothesis. It is then up to the user to prove it". Gappa checks that each
hint is an identity of fractions, and it warned about none of the 39. It
warned 194 times that it took a divisor to be nonzero. In Coq's reals `x / 0`
is 0, so `adx / Adx = 1 + (adx − Adx) / Adx` is false where `Adx` is zero.
Block 1's hypotheses rule that out on paper, since a rounding gives zero only
from zero, but nothing in Coq checks it yet. Q2661 asks whether `field` can.

**3. §12.2 would have been caught.** Drop the hypothesis that the nine
products are normal, with what block 2 gives from it, and the goal fails, best
`±4.63802e+1218`. Drop only `bdx * cdy`'s and Gappa names the place: it cannot
find `REL(bdxcdy, bdx * cdy)`, so `rp1`, the factor that rounding multiplied
by, is only known to lie within `±2⁴¹⁴⁴`, `W1` has no bound, and the goal
fails. That is a product in the subnormals, which is §12.2. Dropping `r1`,
`r2`, `r3` normal fails at `REL(P, Phat)`. Dropping `eb` normal fails too,
best `[-6.95186e+1070, 1.08623e+1069]`.

**4. A miscount like §16.3's would have been caught.** At `B = 6u` the goal
fails, best `±1.16667`, and the claim on `value` fails at `6u` too, best
`±1.33333`. Gappa failing to prove `6u` does not make it false, but §18.5's
input at `6.286u` does, for `value`. The smallest `B` proved, in steps of
`u²`, is `7u + 47u²`: `7u` fails and so does `7u + 46u²`. The code's `56u²`
has `9u²` to spare over that.

**5. Nothing keeps block 1 the same expression as the Rust.** It restates
`on_floats` by hand and names the commit it was read at. If `on_floats` took
an FMA, another order or another product, every test would run as before and
the script would go on proving the old expression. The tests that would notice
a looser bound are the ones §16.3 and §18.5 found too coarse. Q2661 decides
between a script per bound run by a recipe under these pins, and a test that
fails when `on_floats`' source changes and the script's header sha does not.

**6. The other nineteen.** Eight are sign filters of orient3d's kind, an
`on_floats` whose `error_bound` is a constant times a permanent:
`ORIENT2D_ERROR_BOUND` and `surface.rs`' `ERROR_BOUND` in `rcad-predicates`,
and `SIDE_OF_PLANE`, `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT`,
`SIDE_OF_SPHERE`, `SIDE_OF_CYLINDER`, `SIDE_OF_CONE` and `SIDE_OF_TORUS` in
`rcad-geom`. Block 1's pattern should carry over to them: one ratio hint per
rounding, the error split into one term per product of the permanent so that
Gappa can average them, block 2 for the closed normal end, and
`-Eprecision=200`. That is Q2659. The eleven `*_ROUNDOFF` constants were not
read for this. Two of them, `EVALUATION_ROUNDOFF` and `REFINEMENT_ROUNDOFF`,
hold per de Boor pass at any degree, and a Gappa script states one fixed
expression, so it could only do them one degree at a time. That is Q2660. The
machine's part costs little: seconds for Gappa, under ten for `coqc`. The cost
is in finding the hints, which took orient3d three relays and this write-up;
with the pattern known, each of the eight should take less.

**The zero cases, on paper and not run, but one.** Block 1 assumes no factor is
zero, and `on_floats` does not. A difference of two doubles rounds to zero only
when it is zero, and a product with a zero factor is exactly zero, so a zero
difference zeroes its terms in both `y` and `Ex`. The branch where `t1 + t2` is
zero was run: the goal proves, with `|y − Ex| / P6 ≤ 6u + 13u²`. The branch
where `first` is zero was not. There `bdxcdy == cdxbdy`, so the first term of
`y` is zero while the matching term of `Ex` need not be, and the identity block
1 rests on still holds with `rm1 = rt1 = 1`. But `product_is_normal(adz,
first)` passes on the zero, and `r1 = rnd(q1 × |adz|)` can then fall below the
normals: `bdxcdy = cdxbdy = 2⁻¹⁰⁰⁰` and `adz = 2⁻¹⁰⁰` make `q1 × |adz| =
2⁻¹⁰⁹⁹`, which rounds to zero. So block 1's hypothesis that `r1` is normal is
one the code does not check in that branch, and the comment at
`orient3d.rs:366-368`, that the nine checks cover the permanent's products, is
wrong there. Where `first` is not zero the comment holds: `|first| ≤ q1` by
monotone rounding, so `r1 ≥ |t1| ≥ 2⁻¹⁰²²`. A paper estimate says the bound
survives the zero branch: its error is a few `u` of `q1 × |adz|`, far inside
the slack of an `eb` that is normal. Q2658 runs it, and it proves (464.7).

**Overflow does not reach Gappa.** Its `float<ieee_64, ne>` has no largest
exponent: the rounding it writes for Coq is `rounding_float rndNE (53)
(-1074)`, a precision and a smallest exponent. So block 1 says nothing about an
infinity, and 464.2's expectation was wrong. `on_floats` does not need it to,
on paper.
`decide` compares only when `eb` is normal, so finite, and then `P` is finite.
Every intermediate of `on_floats` that could become infinite or not a number
flows into `P`: each difference is a factor of one of the permanent's
products, each product is one of its terms, and every term is non-negative, so
one infinite or not-a-number term makes the sum infinite or not a number. The
rest of `value`'s intermediates are bounded by `P`'s through monotone
rounding: `|first| ≤ q1`, `|t1| ≤ r1`, `|s12| ≤ rnd(r1 + r2)`, `|value| ≤ P`.
So where `decide` compares, nothing rounded to infinity, and there IEEE gives
what Gappa's model without an overflow gives.

### 464.5 The scripts, whole

To run them again, save the nix block as `gappalib18.nix` and the two gappa
blocks as `block1.g` and `block2.g`, and then:

```sh
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 block1.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq block1.g > block1.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc block1.v
```

and the same for `block2.g`. Gappa exits 0 when every goal is proved. When one
is not, it prints `Error: some properties were not satisfied` and the best
bound it found, and exits 1.

Each variant in 464.4 is block 1 with one change. A `B` is its `B = ...`
line. A dropped hypothesis is its lines deleted, and the run that names where
`bdx * cdy` breaks also asks for `bdxcdy -/ bdx * cdy in ?`, `rp1 in ?` and
`W1 in ?`. The claim on `value` writes `(value - Ex)` for each `(y - Ex)`,
adds `rv = value / y;`, multiplies `rm1 * rt1 * rs` and `rm2 * rt2 * rs` in
`W1` to `W4` and `rm3 * rt3` in `W5` and `W6` by `rv`, and adds the hypothesis
`|y| >= 1b-1074` and the hint `value / y -> 1 + (value - y) / y;`. The `t1 +
t2 = 0` branch writes `s12 = S;`, drops `rs` from `W1` to `W4`, and deletes
`rs`'s definition, its hint and the `|t1 + t2|` hypothesis.

Block 2 fails at `u` instead of `u + 2u²` (an exact value just below `2⁻¹⁰²²`
rounds up to it), and fails with `|p| >= 1b-1074` in place of `1b-1022`, best
`[-0.5, 1]`. Gappa's own theorem for this, `rel_float_of_abs`
(`src/arithmetic/float.cpp:426-437` in 1.8.0), needs the exact value strictly
above `2⁻¹⁰²²`, while `product_is_normal` checks the rounded product, and lets
it equal `2⁻¹⁰²²`. That is why block 2 is a lemma of its own.

```nix
# gappalib18.nix; build with: nix --extra-experimental-features
# 'nix-command flakes' build --impure --no-link --print-out-paths -f gappalib18.nix
let
  pkgs = (builtins.getFlake "github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe").legacyPackages.x86_64-linux;
in
pkgs.coqPackages_8_20.gappalib.overrideAttrs (old: {
  version = "1.8.0";
  name = "coq8.20-gappalib-1.8.0";
  src = pkgs.fetchurl {
    url = "https://gitlab.inria.fr/gappa/coq/-/archive/gappalib-coq-1.8.0/coq-gappalib-coq-1.8.0.tar.gz";
    hash = "sha256-xSoI9ZHAMbkkfG3Wnajgu7bhCkglNIBngSrz4YnmkA8=";
  };
})
```

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
R1 = q1 * |adz|; R2 = q2 * |bdz|; R3 = q3 * |cdz|;
r1 = rnd(R1); r2 = rnd(R2); r3 = rnd(R3);
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
  # The first two terms do not cancel to zero.
  /\ |t1 + t2| >= 1b-1074
  # §12.2's first condition: product_is_normal on the nine products.
  /\ |bdxcdy| >= 1b-1022 /\ |cdxbdy| >= 1b-1022 /\ |cdxady| >= 1b-1022
  /\ |adxcdy| >= 1b-1022 /\ |adxbdy| >= 1b-1022 /\ |bdxady| >= 1b-1022
  /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022 /\ |t3| >= 1b-1022
  # The permanent's own three products are normal too. The comment above
  # stays_accurate argues it from the nine.
  /\ |r1| >= 1b-1022 /\ |r2| >= 1b-1022 /\ |r3| >= 1b-1022
  # §12.2's second condition: the bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those thirteen gives: the rounding is off by
  # at most u + 2u^2. Gappa's own theorem needs the value strictly above
  # 2^-1022, so it cannot take the step itself.
  /\ bdxcdy -/ bdx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ cdxbdy -/ cdx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ cdxady -/ cdx * ady in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxcdy -/ adx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxbdy -/ adx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bdxady -/ bdx * ady in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ U1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ U2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ U3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r1 -/ R1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r2 -/ R2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r3 -/ R3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ adz / |adz| in [-1, 1] /\ bdz / |bdz| in [-1, 1] /\ cdz / |cdz| in [-1, 1]
  /\ bdxcdy / |bdxcdy| in [-1, 1] /\ cdxbdy / |cdxbdy| in [-1, 1]
  /\ cdxady / |cdxady| in [-1, 1] /\ adxcdy / |adxcdy| in [-1, 1]
  /\ adxbdy / |adxbdy| in [-1, 1] /\ bdxady / |bdxady| in [-1, 1]
->
  (y - Ex) / P6 in ? /\ P -/ Phat in ?
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
(y - Ex) / eb -> ((y - Ex) / P6) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

```gappa
# lemma.g: a double that rounds to at least MIN_POSITIVE in magnitude is
# off by at most u + 2u^2 relatively, u = 2^-53, even when the exact value
# is below MIN_POSITIVE.
@rnd = float<ieee_64, ne>;
p = rnd(x);
{ |x| <= 1b1024 /\ |p| >= 1b-1022
  -> (p - x) / x in [-4503599627370497b-105, 4503599627370497b-105] }
$ x in (-4503599627370497b-1074, 0, 4503599627370497b-1074);
```

### 464.6 What this leaves

- **Q2658**, the two claims in `orient3d.rs` the pilot does not back: that
  `value` stays within `7*epsilon` of the permanent, in the doc comments on
  `ORIENT3D_ERROR_BOUND` and on the bound test, and that the nine
  `product_is_normal` checks cover the permanent's products when `first`,
  `second` or `third` is zero. Either a proof at the code's bound or comments
  that say what is proved, and a run of the zero branch. Done, in 464.7: the
  claim on `value` is false, the comments now say what is proved, and every
  branch with one or two minors at zero proves at the code's bound.
- **Q2659**, the eight other `*_ERROR_BOUND` sign filters, done as block 1
  was.
- **Q2660**, the eleven `*_ROUNDOFF` constants: which of them Gappa can state,
  and proofs of those.
- **Q2661**, whether the scripts join the tree and the gate, what notices
  `on_floats` drifting from block 1, and whether `field` proves the 39 hint
  equalities once their divisors are shown nonzero.

None of the pilot's findings is a wrong answer found on an input. The sign
`decide` certifies is proved right on every input block 1 covers. What is left
is comments that claim more than is proved, zero branches argued on paper and
one of them not run, and 39 equalities Coq takes on trust. Since 464.7 the
comments in `orient3d.rs` say what is proved, and every branch with a minor at
zero is run. The branches where one of the nine differences is zero are still
on paper, as 464.4 argues them, and so is the step from `y` to the sign.

### 464.7 value past the bound, and every minor at zero run

Q2658 took up the two claims 464.4 left unbacked, in two lanes on 2026-10-23.
Neither changed a function body in `orient3d.rs`, so what Gappa ran is still
`on_floats` as 464.4 read it at `d9f931e8`. The `block1.g` the scripts below
start from is 464.5's block 1 byte for byte: `cmp` against lines 326 to 502
of this file finds no difference. Every run is Gappa 1.8.0 under 464.4's pin,
at `-Eprecision=200`, and at the code's `B = 7u + 56u²` unless it names
another `B`.

**1. `value` lands past the bound.** 464.4 proved `y` within `eb` of the true
determinant and could not prove `value` within it. It is not within it, on
this tetrahedron:

- `a = (2⁻²⁷, 1, 1 + 22369622·2⁻⁵²)`
- `b = (1 + 2⁻²⁶ + 2⁻⁵², 1, −2⁻²⁶(1 − 2⁻²⁶))`
- `c = (−(2⁻⁷⁰ + 2⁻⁸⁰), 1 + 2⁻²⁷, −2⁻⁸⁰ + 40·2⁻⁹³)`
- `d = 2⁻⁵³ − 2⁻⁸⁰` on every axis

`d` is just under half an ulp of 1, so `a.z − d.z`, `b.x − d.x` and
`c.y − d.y` each round up by nearly `u` of themselves. From there the first
term rounds up at every step. Eight roundings lie along it, and each goes up
by between `0.999992u` and `1.000000u` of what it rounds: `adz`, `bdx`, `cdy`,
`bdx * cdy`, `first`, `adz * first`, the sum of the first two terms, and
`value`.

For that to come to `8u` of the permanent, the determinant has to be nearly
all of the permanent: no minor may cancel inside itself, and no two terms may
cancel each other. A minor does not cancel when its two products have opposite
signs, and all three minors cannot have that. The first products of the three
minors multiply to the six differences `adx` to `cdy`, and so do the second
products, so the three conditions together say a sign equals minus itself. So
the first two minors are built not to cancel, their terms have the same sign,
and the third term is kept small, with `cdz` about `−2⁻⁵³`. Here `value` and
the rounded permanent `P` are the same double, `0x1.000000755555cp+0`.

A scratch search tried `b.z = −2⁻²⁶(1 + k·2⁻³⁰)` and `c.z = −2⁻⁸⁰ − j·2⁻⁹³`
for `k` and `j` from −40 to 40, and kept the best, `k = −16` and `j = −40`,
which are the points above. On them it printed:

```text
ratio in u: 7.999992112355106
|value - exact| / eb: 1.1428560160507284 err > eb: True
signs: value 1 exact 1 value > eb True
```

So `value` lands `7.99999u` of the permanent from the true determinant, which
is `1.14286` times `eb`: the `±1.14286` Gappa could not get under for `value`
in 464.4, now met by an input. `j = −40` is the edge of the search, so a wider
one may come a little closer to `8u`, and 464.4's proof at `8u + 64u²` is a
bound no input passes. The sign is still right: `value > eb`, the truth is
positive, and `decide` answers `(Pos, Float)`, as 464.4's step from `y` to the
sign says it must.
`level_one_can_land_past_its_error_bound_and_still_certify_the_right_sign`, in
`orient3d.rs`'s tests, keeps the tetrahedron and checks all three: `value`
past `error_bound`, past `7.99` epsilon of the permanent, and `decide`'s
answer. That test is what runs this again; the scratch scripts are not kept.

**2. The comments now say what is proved.** `922db1fd` adds the test above.
In `orient3d.rs`, `813f8a09` and `c827c89b` change only comments and the names
of test items:

- `ORIENT3D_ERROR_BOUND`'s doc says it bounds `y`, the sum `value` is rounded
  from, counts `y`'s seven roundings, and gives the step from `y` to the sign.
  It says `value` takes an eighth rounding and names the test above.
- The bound test's doc no longer says `value` never falls further than
  `7*epsilon` of the permanent from the truth. It says what the search found,
  6.29 epsilon, and that the built tetrahedron rounds worse.
- Three names said the search's tetrahedra round as badly as any can, and the
  built one rounds worse, so they are renamed:
  `level_one_stays_inside_its_error_bound_where_the_rounding_is_worst` to
  `level_one_stays_inside_its_error_bound_on_the_tetrahedra_a_search_found`,
  `the_worst_rounding_tetrahedra_are_settled_by_level_one_itself` to
  `the_searched_tetrahedra_are_settled_by_level_one_itself`, and
  `tetrahedra_whose_arithmetic_rounds_as_badly_as_it_can` to
  `tetrahedra_a_search_found_rounding_badly`. Outside Q2658's own row in
  the queue, only 464.4 named one of them, the first, by its old name and
  line. It keeps them, as a record of what it read at `d9f931e8`.
- The comment above `stays_accurate` says the nine checks cover the
  permanent's matching product only for a minor that is not zero. It gives
  464.4's example: `bdxcdy == cdxbdy == 2⁻¹⁰⁰⁰` makes `q1 = 2⁻⁹⁹⁹`, and with
  `adz == 2⁻¹⁰⁰`, `q1 × |adz| = 2⁻¹⁰⁹⁹`, which rounds to zero. It says the sign
  is proved with any one or any two minors at zero, which 3 below runs.

**3. Every minor at zero, run.** A minor is zero when its two products round
to the same double. Then the minor is exactly zero and so is its term in `y`,
while its term in the true determinant need not be: what is left of it is the
two products' rounding errors, which block 1 already carries. `product_is_normal`
passes on the zero, so nothing checks that the permanent's matching product is
normal. Each script is block 1 with that minor's two products made one double,
the minor and its term left unrounded, their two ratios set to 1, and the
hypotheses the term and the permanent's product being normal gave dropped.
Where that leaves one of `t1` and `t2` zero, their sum is the other one and
does not round, so `rs` goes too.

- **`first` zero**, `zero2.g`: proves, with `(y − Ex) / P6` within
  `6u + 13u²`. At `B = 5u` it fails, best `±1.2`.
- **`second` zero**: proves, within `6u + 13u²`. At `B = 5u` it fails, best
  `±1.2`.
- **`third` zero**: with block 1's hints alone it fails, best `±1`. With six
  more hints it proves, with `(y − Ex) / P6` within `7u + 11u²`. It fails at
  `B = 6u`, best `±1.16667`, and at `7u + 46u²`, best `±1`: the same two places
  block 1 fails (464.4).
- **Two at zero**: `first` and `second` with block 1's hints alone fail where
  block 1 does without `r1`, `r2` and `r3` normal, at `REL(P, Phat)`, best
  `[-2.29099e+538, 7.5465e+645]`. The three scripts `make_two_zero.py` writes,
  one for each pair, each prove, with `(y − Ex) / P6` within `6u + 13u²` and
  `P −/ Phat` within `8u + 14u²`. Its script for `first` and `second` fails at
  `B = 5u`, best `±1.2`.
- **All three at zero**: `y` is zero, so `value` is zero, and `decide`
  certifies no sign from it: `value > eb` and `value < −eb` are both false for
  any `eb` that is not negative. The same holds where `third` is zero and
  `t1 + t2` is zero.

The runs below `B` are there because a script whose hypotheses contradict each
other proves every goal. Each script that proves at `B` fails below it, so none
of them proved its goal from a contradiction.

The hints `third` needs, and the seven `make_two_zero.py` adds, carry the error
of a permanent product that may be below the normals as an amount against
`Phat`, not as a fraction of itself, and keep `Phat` away from zero through a
product that is normal: `P12` for `third`, and the one minor not at zero for a
pair. Why block 1's hints are enough when `first` or `second` is zero and not
when `third` is was not traced.

`gappa -Bcoq` writes each of the six proofs (one minor at zero, three ways, and
two at zero, three ways), and `coqc` under 464.5's pins, with gappalib 1.8.0,
accepts each: exit 0, in 2.8 to 4.5 s. Like block 1's, each rests on its
hints, 36 to 43 of them, which enter Coq as hypotheses (464.4).

**4. The scripts.** Each runs as 464.5 runs block 1. A run at another `B` is
the script with its `B = ...` line changed: `B = 5b-53;`, `B = 6b-53;`, or
`B = 7b-53 + 23b-105;` for `7u + 46u²`.

`zero2.g`, `first` zero, whole:

```gappa
# Block 1 where first is zero: bdx * cdy and cdx * bdy round to the same
# double, so first = rnd(0) = 0 and t1 = rnd(adz * 0) = 0, and r1 is not
# checked by any product_is_normal.
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
# The same double as bdxcdy; that it is cdx * bdy rounded comes in as its
# relative error below.
cdxbdy = bdxcdy;
cdxady = rnd(cdx * ady);
adxcdy = rnd(adx * cdy);
adxbdy = rnd(adx * bdy);
bdxady = rnd(bdx * ady);

M1 = bdxcdy - cdxbdy; M2 = cdxady - adxcdy; M3 = adxbdy - bdxady;
first = M1; second = rnd(M2); third = rnd(M3);

U1 = adz * first; U2 = bdz * second; U3 = cdz * third;
t1 = U1; t2 = rnd(U2); t3 = rnd(U3);
# t1 is zero, so t1 + t2 is the double t2 and rounds to itself.
S = t1 + t2;
s12 = S;
y = s12 + t3;
value = rnd(y);

q1 = rnd(|bdxcdy| + |cdxbdy|);
q2 = rnd(|cdxady| + |adxcdy|);
q3 = rnd(|adxbdy| + |bdxady|);
R1 = q1 * |adz|; R2 = q2 * |bdz|; R3 = q3 * |cdz|;
r1 = rnd(R1); r2 = rnd(R2); r3 = rnd(R3);
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
rm1 = 1; rm2 = second / M2; rm3 = third / M3;
rt1 = 1; rt2 = t2 / U2; rt3 = t3 / U3;

# Each of the six terms of the determinant, as y carries it, less the
# term itself.
W1 = rm1 * rt1 - 1 / (radz * rbdx * rcdy * rp1);
W2 = rm1 * rt1 - 1 / (radz * rcdx * rbdy * rp2);
W3 = rm2 * rt2 - 1 / (rbdz * rcdx * rady * rp3);
W4 = rm2 * rt2 - 1 / (rbdz * radx * rcdy * rp4);
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
  # No factor of the nine products is zero but first. For an f64 that is
  # the same as being at least the smallest subnormal.
  /\ |adx| >= 1b-1074 /\ |ady| >= 1b-1074 /\ |adz| >= 1b-1074
  /\ |bdx| >= 1b-1074 /\ |bdy| >= 1b-1074 /\ |bdz| >= 1b-1074
  /\ |cdx| >= 1b-1074 /\ |cdy| >= 1b-1074 /\ |cdz| >= 1b-1074
  /\ |second| >= 1b-1074 /\ |third| >= 1b-1074
  # §12.2's first condition: product_is_normal on the nine products. The
  # check on adz and first passes on the zero, and says nothing.
  /\ |bdxcdy| >= 1b-1022 /\ |cdxbdy| >= 1b-1022 /\ |cdxady| >= 1b-1022
  /\ |adxcdy| >= 1b-1022 /\ |adxbdy| >= 1b-1022 /\ |bdxady| >= 1b-1022
  /\ |t2| >= 1b-1022 /\ |t3| >= 1b-1022
  # r2 and r3 are normal because |second| <= q2 and |third| <= q3. Nothing
  # says r1 is.
  /\ |r2| >= 1b-1022 /\ |r3| >= 1b-1022
  # §12.2's second condition: the bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those normal numbers gives: the rounding is
  # off by at most u + 2u^2. Gappa's own theorem needs the value strictly
  # above 2^-1022, so it cannot take the step itself.
  /\ bdxcdy -/ bdx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ cdxbdy -/ cdx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ cdxady -/ cdx * ady in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxcdy -/ adx * cdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ adxbdy -/ adx * bdy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bdxady -/ bdx * ady in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ U2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ U3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r2 -/ R2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ r3 -/ R3 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ adz / |adz| in [-1, 1] /\ bdz / |bdz| in [-1, 1] /\ cdz / |cdz| in [-1, 1]
  /\ bdxcdy / |bdxcdy| in [-1, 1] /\ cdxbdy / |cdxbdy| in [-1, 1]
  /\ cdxady / |cdxady| in [-1, 1] /\ adxcdy / |adxcdy| in [-1, 1]
  /\ adxbdy / |adxbdy| in [-1, 1] /\ bdxady / |bdxady| in [-1, 1]
->
  (y - Ex) / P6 in ? /\ P -/ Phat in ?
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
second / M2 -> 1 + (second - M2) / M2;
third / M3 -> 1 + (third - M3) / M3;
t2 / U2 -> 1 + (t2 - U2) / U2;
t3 / U3 -> 1 + (t3 - U3) / U3;

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
(y - Ex) / eb -> ((y - Ex) / P6) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

**`second` zero** is block 1 with these changes:

- `adxcdy = rnd(adx * cdy);` becomes `adxcdy = cdxady;`. In the next lines,
  `second = rnd(M2)` becomes `second = M2`, `t2 = rnd(U2)` becomes `t2 = U2`,
  and `s12 = rnd(S);` becomes `s12 = S;`.
- `rm2 = second / M2` becomes `rm2 = 1`, and `rt2 = t2 / U2` becomes
  `rt2 = 1`. `rs = s12 / S;` goes, and so does the ` * rs` in `W1` to `W4`.
- In the hypotheses, `|second| >= 1b-1074`, `|t1 + t2| >= 1b-1074`,
  `|t2| >= 1b-1022` and `|r2| >= 1b-1022` go, and so do the `t2 -/ U2` and
  `r2 -/ R2` lines.
- The hints `second / M2 -> ...`, `t2 / U2 -> ...` and `s12 / S -> ...` go.

**`third` zero** is block 1 with these changes:

- `bdxady = rnd(bdx * ady);` becomes `bdxady = adxbdy;`. In the next lines,
  `third = rnd(M3)` becomes `third = M3`, and `t3 = rnd(U3)` becomes
  `t3 = U3`.
- `rm3 = third / M3` becomes `rm3 = 1`, and `rt3 = t3 / U3` becomes
  `rt3 = 1`.
- `P = rnd(rnd(r1 + r2) + r3);` becomes two lines, and `Phat` four:

  ```gappa
  P12 = rnd(r1 + r2);
  P = rnd(P12 + r3);
  Phat12 = (|bdxcdy| + |cdxbdy|) * |adz|
         + (|cdxady| + |adxcdy|) * |bdz|;
  Phat3 = (|adxbdy| + |bdxady|) * |cdz|;
  Phat = Phat12 + Phat3;
  ```

- In the hypotheses, `|third| >= 1b-1074`, `|t3| >= 1b-1022` and
  `|r3| >= 1b-1022` go, and so do the `t3 -/ U3` and `r3 -/ R3` lines.
- The hints `third / M3 -> ...` and `t3 / U3 -> ...` go, and these six are
  added at the end:

  ```gappa
  # r3 may be below the normals, so its error is an amount and not a
  # fraction. Carry it as an amount against Phat, which eb being normal keeps
  # far above it.
  (P - Phat) / Phat -> ((P - (P12 + r3)) / (P12 + r3)) * (1 + ((P12 + r3) - Phat) / Phat) + ((P12 + r3) - Phat) / Phat;
  ((P12 + r3) - Phat) / Phat -> ((P12 - Phat12) / Phat12) * (Phat12 / Phat) + (r3 - R3) / Phat + ((q3 - (|adxbdy| + |bdxady|)) / (|adxbdy| + |bdxady|)) * (Phat3 / Phat);
  Phat12 / Phat -> 1 / (1 + Phat3 / Phat12);
  Phat3 / Phat -> 1 / (1 + Phat12 / Phat3);
  Phat12 -> P12 / (1 + (P12 - Phat12) / Phat12);
  P12 + r3 -> P / (1 + (P - (P12 + r3)) / (P12 + r3));
  ```

The run with block 1's hints alone is this without the six, and with `P` and
`Phat` as block 1 writes them.

**Two at zero.** `make_two_zero.py`, whole, run as `python3 make_two_zero.py 12
13 23` beside `block1.g`, writes `zero12d.g`, `zero13d.g` and `zero23d.g`. The
run with block 1's hints alone is `zero12d.g` without what the script appends
at the end, and with `P` and `Phat` as block 1 writes them.

```python
"""Write block 1 with two of first, second and third zero, as zeroJK.g."""
import sys

PAIR = {
    1: ("cdxbdy = rnd(cdx * bdy);", "cdxbdy = bdxcdy;"),
    2: ("adxcdy = rnd(adx * cdy);", "adxcdy = cdxady;"),
    3: ("bdxady = rnd(bdx * ady);", "bdxady = adxbdy;"),
}
MINOR = {1: "first", 2: "second", 3: "third"}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def make(zero):
    (normal,) = {1, 2, 3} - zero
    src = open("block1.g").read()
    names = " and ".join(MINOR[k] for k in sorted(zero))
    src = sub(
        src,
        "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
        "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
        f"# Block 1 where {names} are both zero. The permanent's products\n"
        f"# for those two are checked by no product_is_normal.\n",
    )
    for k in zero:
        src = sub(src, *PAIR[k])
        src = sub(src, f"{MINOR[k]} = rnd(M{k});", f"{MINOR[k]} = M{k};")
        src = sub(src, f"t{k} = rnd(U{k});", f"t{k} = U{k};")
        src = sub(src, f"rm{k} = {MINOR[k]} / M{k};", f"rm{k} = 1;")
        src = sub(src, f"rt{k} = t{k} / U{k};", f"rt{k} = 1;")
        src = sub(src, f"{MINOR[k]} / M{k} -> 1 + ({MINOR[k]} - M{k}) / M{k};\n", "")
        src = sub(src, f"t{k} / U{k} -> 1 + (t{k} - U{k}) / U{k};\n", "")
        bound = "in [-4503599627370497b-105, 4503599627370497b-105]\n"
        src = sub(src, f"  /\\ t{k} -/ U{k} {bound}", "")
        src = sub(src, f"  /\\ r{k} -/ R{k} {bound}", "")
    # One of t1 and t2 is zero, so their sum is the other and rounds to
    # itself.
    src = sub(src, "s12 = rnd(S);", "s12 = S;")
    src = sub(src, "rs = s12 / S;\n", "")
    src = src.replace(" * rs - 1", " - 1")
    src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
    src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
    m = MINOR[normal]
    src = sub(
        src,
        "  # No factor of the nine products is zero. For an f64 that is the same as\n"
        "  # being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {names}. For an f64\n"
        "  # that is the same as being at least the smallest subnormal.\n",
    )
    src = sub(
        src,
        "  # The permanent's own three products are normal too. The comment above\n"
        "  # stays_accurate argues it from the nine.\n",
        f"  # r{normal} is normal because |{m}| <= q{normal}. Nothing says the other\n"
        "  # two are.\n",
    )
    src = sub(
        src,
        "  /\\ |first| >= 1b-1074 /\\ |second| >= 1b-1074 /\\ |third| >= 1b-1074\n",
        f"  /\\ |{m}| >= 1b-1074\n",
    )
    src = sub(
        src,
        "  /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022 /\\ |t3| >= 1b-1022\n",
        f"  /\\ |t{normal}| >= 1b-1022\n",
    )
    src = sub(
        src,
        "  /\\ |r1| >= 1b-1022 /\\ |r2| >= 1b-1022 /\\ |r3| >= 1b-1022\n",
        f"  /\\ |r{normal}| >= 1b-1022\n",
    )
    src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P12 = rnd(r1 + r2);\nP = rnd(P12 + r3);")
    src = sub(
        src,
        "Phat = (|bdxcdy| + |cdxbdy|) * |adz|\n"
        "     + (|cdxady| + |adxcdy|) * |bdz|\n"
        "     + (|adxbdy| + |bdxady|) * |cdz|;\n",
        "Phat1 = (|bdxcdy| + |cdxbdy|) * |adz|;\n"
        "Phat2 = (|cdxady| + |adxcdy|) * |bdz|;\n"
        "Phat3 = (|adxbdy| + |bdxady|) * |cdz|;\n"
        "Phat12 = Phat1 + Phat2;\n"
        "Phat = Phat12 + Phat3;\n",
    )
    pieces = ["(P12 - (r1 + r2)) / Phat"]
    for k in (1, 2, 3):
        if k in zero:
            pieces.append(f"(r{k} - R{k}) / Phat")
            pieces.append(f"((q{k} - ({NOTE[k]})) / ({NOTE[k]})) * (Phat{k} / Phat)")
        else:
            pieces.append(f"((r{k} - Phat{k}) / Phat{k}) * (Phat{k} / Phat)")
    others = {1: "Phat2 + Phat3", 2: "Phat1 + Phat3", 3: "Phat12"}
    src += (
        "\n"
        "# Two of r1, r2 and r3 may be below the normals, so their errors are\n"
        "# amounts and not fractions. Carry each as an amount against Phat,\n"
        f"# which r{normal} being normal keeps away from zero.\n"
        "(P - Phat) / Phat -> ((P - (P12 + r3)) / (P12 + r3)) * (1 + ((P12 + r3) - Phat) / Phat)"
        " + ((P12 + r3) - Phat) / Phat;\n"
        f"((P12 + r3) - Phat) / Phat -> {' + '.join(pieces)};\n"
        + "".join(
            f"Phat{k} / Phat -> 1 / (1 + ({others[k]}) / Phat{k});\n" for k in (1, 2, 3)
        )
        + f"Phat{normal} -> r{normal} / (1 + (r{normal} - Phat{normal}) / Phat{normal});\n"
        "P12 + r3 -> P / (1 + (P - (P12 + r3)) / (P12 + r3));\n"
    )
    return src


for arg in sys.argv[1:]:
    zero = {int(c) for c in arg}
    open(f"zero{arg}d.g", "w").write(make(zero))
```

**5. For Q2659.** The eight other filters' doc comments may claim what
orient3d's did, that `value` stays within the constant times the permanent,
where a proof like block 1's gives that for the sum before the last rounding.
None of them was read for this.

**6. What this leaves.**

- A row to restate `Filtered::error_bound`'s doc, `cascade.rs:25-28`, which
  says it is the most `value` can differ from the truth. For orient3d `value`
  lands 8/7 of it away (1 above). What `decide`'s sign test needs, and what
  Gappa proves, is a bound on the sum before `value`'s last rounding.
- A row to run block 1 where one of the nine differences `adx` to `cdz` is
  zero, which 464.4 argues on paper only, reusing the scripts in 4.

### 464.8 orient2d and the side of a plane (Q2659, unfinished)

*Q2659's first lane, on 2026-10-23, handed over at its budget with this
written down and its controls not yet run. What is below was run; what is
not run says so.*

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, against
`orient2d.rs` and `plane.rs` at `affe0f01`. `u` is `2⁻⁵³` as in 464.4. Each
script follows block 1's pattern: one ratio hint per rounding, the error split
into one term per product of the permanent so that Gappa averages them, the
lemma (464.5's block 2) for a product that rounds to a normal number, and the
step from `y`, the sum before `value`'s last rounding, to the sign as 464.4
gives it.

**1. `ORIENT2D_ERROR_BOUND`, with both products normal.** `o2.g` proves
`|y − Ex| ≤ eb` at the code's `B = 3u + 16u²`, in 0.33 s, printing
`(y − Ex) / Phat` within `3u + 8u²` and `P −/ Phat` within `u`. The smallest
`B` it proves, in steps of `u²`, is `3u + 15u²`: `3u + 14u²` fails, and so
does `3u`. The claim on `value`, `o2v.g` (block 1's change in 464.5, here
`rv` multiplying the `1` in `W1` and `W2`), fails at the code's `B`, best
`[-1.33333, 1.33333]`, with `(value − Ex) / Phat` within `4u + 8u²`. The doc
comment on `ORIENT2D_ERROR_BOUND` does not claim it: it counts three
roundings and argues the fourth apart. No input was built for it.

**2. orient2d without the products normal.** `on_floats` does not check them
(`orient2d.rs:278-280`), so `o2.g`'s hypotheses are ones the code does not
give. The bound holds anyway, wherever `decide` compares. Split on the exact
products `F = acx * bcy` and `G = acy * bcx`:

- both at least `2⁻¹⁰²²`: `o2.g`.
- `G` under `2⁻¹⁰²²` and `|F|` at least `2⁻¹⁰⁰⁰`: `o2s.g` for `G` positive
  and `o2sn.g` for `G` negative. Each proves at the code's `B`, printing
  `(e1 + e2) / Phat2` within `3u + 6u²`, what rounding `G` cost within
  `±2⁻¹⁰⁷⁵`, and `P` at least `2^(-970.585)`.
- `F` under `2⁻¹⁰²²` and `|G|` at least `2⁻¹⁰⁰⁰`: `o2f.g` and `o2fn.g`, the
  same, and each proves.
- both under `2⁻¹⁰⁰⁰`: `o2tiny.g` prints "remaining results are pointless,
  anything can be proved": `eb` cannot then be normal.

What makes it work is that `eb` normal puts `P` above `2^(-970.585)`, so a
product below the normals loses at most `2⁻¹⁰⁷⁵`, which is at most `u` of
`eb`, and the code's `16u²` has room for it. The controls are written and
not run: each case script at `B = 3u` and at `3u + 14u²`, and `o2tiny.g`
without `|eb| >= 1b-1022`, each of which must fail for its parent not to
have proved from a contradiction.

The doc comment's own argument, `orient2d.rs:343-350`, does not hold as
written: it calls `acx * bcy` and `acy * bcx` exact products, but they are
products of rounded differences, so monotone rounding gives the sign of the
rounded differences' determinant, not the true one. The conclusion, that no
subnormal guard is needed, is what the scripts above show instead, by way of
`eb` being normal. Nothing here ran the paper argument's failure on an input.

**3. `SIDE_OF_PLANE_ERROR_BOUND`.** `plane.g` proves `|y − Ex| ≤ eb` at the
code's `B = 4u + 32u²`, in 0.59 s, printing `(y − Ex) / P3` within `3u + 2u²`
(`P3` the three products' part of the permanent; the offset reaches `y` with
no rounding and only makes the permanent larger) and `P −/ Phat` within
`3u + 3u²`. The smallest `B` proved is `3u + 15u²`: `3u + 14u²` fails, best
`[-1, 1]`. The claim on `value`, `planev.g`, proves at the code's `B`, with
`(value − Ex) / Phat` within `4u + 4u²`: the doc comment on
`SIDE_OF_PLANE_ERROR_BOUND` counts four roundings to `value`, and that is
proved. Not yet run: `plane.g` without `product_is_normal` and without `eb`
normal, and `planev.g` below the code's `B`.

**4. Not started.** `surface.rs`' `ERROR_BOUND`,
`SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT`, `SIDE_OF_SPHERE`,
`SIDE_OF_CYLINDER`, `SIDE_OF_CONE` and `SIDE_OF_TORUS`, and `coqc` on any
proof here.

**5. The scripts.** `o2.g`, whole:

```gappa
# Orient2d::on_floats and ORIENT2D_ERROR_BOUND,
# crates/rcad-predicates/src/orient2d.rs at affe0f01.
@rnd = float<ieee_64, ne>;

# The six input coordinates, each an f64.
ax = rnd(ax_); ay = rnd(ay_);
bx = rnd(bx_); by = rnd(by_);
cx = rnd(cx_); cy = rnd(cy_);

# What on_floats computes, line for line.
Acx = ax - cx; Acy = ay - cy;
Bcx = bx - cx; Bcy = by - cy;
acx = rnd(Acx); acy = rnd(Acy);
bcx = rnd(Bcx); bcy = rnd(Bcy);

forwards = rnd(acx * bcy);
backwards = rnd(acy * bcx);
y = forwards - backwards;
value = rnd(y);

Phat = |forwards| + |backwards|;
P = rnd(Phat);

B = 3b-53 + 1b-102;
eb = rnd(B * P);

# The true determinant of the same inputs.
Ex = Acx * Bcy - Acy * Bcx;

# What each rounding multiplied its argument by.
racx = acx / Acx; racy = acy / Acy;
rbcx = bcx / Bcx; rbcy = bcy / Bcy;
rp1 = forwards / (acx * bcy); rp2 = backwards / (acy * bcx);

# Each of the two products, as y carries it, less the product of the
# exact differences.
W1 = 1 - 1 / (racx * rbcy * rp1);
W2 = 1 - 1 / (racy * rbcx * rp2);
c1 = |forwards|; c2 = |backwards|;
e1 = forwards * W1; e2 = -(backwards * W2);

{
  # Every coordinate is a finite f64.
     |ax| <= 1b1024 /\ |ay| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  # No difference is zero.
  /\ |acx| >= 1b-1074 /\ |acy| >= 1b-1074
  /\ |bcx| >= 1b-1074 /\ |bcy| >= 1b-1074
  # Both products are normal. on_floats does not check this.
  /\ |forwards| >= 1b-1022 /\ |backwards| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those three gives.
  /\ forwards -/ acx * bcy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ backwards -/ acy * bcx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ forwards / |forwards| in [-1, 1] /\ backwards / |backwards| in [-1, 1]
->
  (y - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
acx / Acx -> 1 + (acx - Acx) / Acx;
acy / Acy -> 1 + (acy - Acy) / Acy;
bcx / Bcx -> 1 + (bcx - Bcx) / Bcx;
bcy / Bcy -> 1 + (bcy - Bcy) / Bcy;
forwards / (acx * bcy) -> 1 + (forwards - acx * bcy) / (acx * bcy);
backwards / (acy * bcx) -> 1 + (backwards - acy * bcx) / (acy * bcx);

# The error is the sum of the two products' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / Phat -> ((Phat + (y - Ex)) - Phat) / Phat;
Phat + (y - Ex) -> (c1 + e1) + (c2 + e2);
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e1 / c1 -> (forwards / |forwards|) * W1;
e2 / c2 -> -((backwards / |backwards|) * W2);

# From the permanent to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

`o2s.g`, whole:

```gappa
# o2.g where backwards may be below the normals, which on_floats does not
# check: acy * bcx is under 2^-1022 and positive, and acx * bcy is at
# least 2^-1000.
@rnd = float<ieee_64, ne>;

ax = rnd(ax_); ay = rnd(ay_);
bx = rnd(bx_); by = rnd(by_);
cx = rnd(cx_); cy = rnd(cy_);

Acx = ax - cx; Acy = ay - cy;
Bcx = bx - cx; Bcy = by - cy;
acx = rnd(Acx); acy = rnd(Acy);
bcx = rnd(Bcx); bcy = rnd(Bcy);

F = acx * bcy;
G = acy * bcx;
forwards = rnd(F);
backwards = rnd(G);
y = forwards - backwards;
value = rnd(y);

# backwards is not negative, as G is not, so |backwards| is backwards.
Phat = |forwards| + backwards;
P = rnd(Phat);

B = 3b-53 + 1b-102;
eb = rnd(B * P);

Ex = Acx * Bcy - Acy * Bcx;

racx = acx / Acx; racy = acy / Acy;
rbcx = bcx / Bcx; rbcy = bcy / Bcy;
rp1 = forwards / F;

# The permanent with G in place of what it rounds to.
c1 = |forwards|; c2 = G;
Phat2 = c1 + c2;

# Each product less the product of the exact differences, as relative
# errors for forwards and for G, and what rounding G cost, as an amount.
W1 = 1 - 1 / (racx * rbcy * rp1);
W2 = 1 - 1 / (racy * rbcx);
e1 = forwards * W1; e2 = -(G * W2);
d2 = -(backwards - G);

{
     |ax| <= 1b1024 /\ |ay| <= 1b1024
  /\ |bx| <= 1b1024 /\ |by| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  /\ |acx| >= 1b-1074 /\ |acy| >= 1b-1074
  /\ |bcx| >= 1b-1074 /\ |bcy| >= 1b-1074
  # The case: G is positive and below the normals, F is far above them.
  # A product of two doubles that are not zero is at least 2^-2148.
  /\ G in [1b-2148, 1b-1022] /\ |F| >= 1b-1000
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  /\ forwards / |forwards| in [-1, 1]
->
  (e1 + e2) / Phat2 in ? /\ d2 in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

acx / Acx -> 1 + (acx - Acx) / Acx;
acy / Acy -> 1 + (acy - Acy) / Acy;
bcx / Bcx -> 1 + (bcx - Bcx) / Bcx;
bcy / Bcy -> 1 + (bcy - Bcy) / Bcy;
forwards / F -> 1 + (forwards - F) / F;

# The relative part is averaged over Phat2 as in o2.g; what rounding G
# cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e1 + e2) / Phat2) * (Phat2 / P) * (P / eb) + d2 / eb;
(e1 + e2) / Phat2 -> ((Phat2 + (e1 + e2)) - Phat2) / Phat2;
Phat2 + (e1 + e2) -> (c1 + e1) + (c2 + e2);
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e1 / c1 -> (forwards / |forwards|) * W1;
e2 / c2 -> -W2;
Phat2 / P -> Phat / P + (G - backwards) / P;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
```

`o2cases.py`, whole, run beside `o2s.g` and `o2.g`, writes `o2sn.g`, `o2f.g`,
`o2fn.g` and `o2tiny.g`:

```python
"""Write o2s.g's other cases: o2sn.g (G negative), o2f.g and o2fn.g (the same
with forwards below the normals instead), and o2tiny.g (both products under
2^-1000)."""


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


base = open("o2s.g").read()

# G negative: |G| is -G and |backwards| is -backwards.
src = base
src = sub(src, "acy * bcx is under 2^-1022 and positive", "acy * bcx is under 2^-1022 and negative")
src = sub(src, "# backwards is not negative, as G is not, so |backwards| is backwards.\nPhat = |forwards| + backwards;",
          "# backwards is not positive, as G is not, so |backwards| is -backwards.\nPhat = |forwards| - backwards;")
src = sub(src, "c2 = G;", "c2 = -G;")
src = sub(src, "G is positive and below", "G is negative and above")
src = sub(src, "G in [1b-2148, 1b-1022]", "G in [-1b-1022, -1b-2148]")
src = sub(src, "e2 / c2 -> -W2;", "e2 / c2 -> W2;")
src = sub(src, "Phat2 / P -> Phat / P + (G - backwards) / P;", "Phat2 / P -> Phat / P + (backwards - G) / P;")
open("o2sn.g", "w").write(src)

# forwards below the normals instead of backwards.
for sign in ("", "n"):
    src = open(f"o2s{sign}.g").read()
    src = sub(src, "# o2.g where backwards may be below", "# o2.g where forwards may be below")
    if sign:
        src = sub(src, "acy * bcx is under 2^-1022 and negative, and acx * bcy is at",
                  "acx * bcy is under 2^-1022 and negative, and acy * bcx is at")
    else:
        src = sub(src, "acy * bcx is under 2^-1022 and positive, and acx * bcy is at",
                  "acx * bcy is under 2^-1022 and positive, and acy * bcx is at")
    src = sub(src, "F = acx * bcy;\nG = acy * bcx;\nforwards = rnd(F);\nbackwards = rnd(G);",
              "G = acx * bcy;\nF = acy * bcx;\nforwards = rnd(G);\nbackwards = rnd(F);")
    if sign:
        src = sub(src, "# backwards is not positive, as G is not, so |backwards| is -backwards.\nPhat = |forwards| - backwards;",
                  "# forwards is not positive, as G is not, so |forwards| is -forwards.\nPhat = -forwards + |backwards|;")
    else:
        src = sub(src, "# backwards is not negative, as G is not, so |backwards| is backwards.\nPhat = |forwards| + backwards;",
                  "# forwards is not negative, as G is not, so |forwards| is forwards.\nPhat = forwards + |backwards|;")
    src = sub(src, "rp1 = forwards / F;", "rp1 = backwards / F;")
    src = sub(src, "c1 = |forwards|;", "c1 = |backwards|;")
    src = sub(src, "W1 = 1 - 1 / (racx * rbcy * rp1);\nW2 = 1 - 1 / (racy * rbcx);",
              "W1 = 1 - 1 / (racy * rbcx * rp1);\nW2 = 1 - 1 / (racx * rbcy);")
    # y - Ex = (G - Acx Bcy) + (forwards - G) - (backwards - Acy Bcx).
    src = sub(src, "e1 = forwards * W1; e2 = -(G * W2);\nd2 = -(backwards - G);",
              "e1 = -(backwards * W1); e2 = G * W2;\nd2 = forwards - G;")
    src = sub(src, "/\\ forwards / |forwards| in [-1, 1]", "/\\ backwards / |backwards| in [-1, 1]")
    src = sub(src, "forwards / F -> 1 + (forwards - F) / F;", "backwards / F -> 1 + (backwards - F) / F;")
    src = sub(src, "e1 / c1 -> (forwards / |forwards|) * W1;", "e1 / c1 -> -((backwards / |backwards|) * W1);")
    if sign:
        src = sub(src, "e2 / c2 -> W2;", "e2 / c2 -> -W2;")
        src = sub(src, "Phat2 / P -> Phat / P + (backwards - G) / P;", "Phat2 / P -> Phat / P + (forwards - G) / P;")
    else:
        src = sub(src, "e2 / c2 -> -W2;", "e2 / c2 -> W2;")
        src = sub(src, "Phat2 / P -> Phat / P + (G - backwards) / P;", "Phat2 / P -> Phat / P + (G - forwards) / P;")
    open(f"o2f{sign}.g", "w").write(src)

# Both products under 2^-1000: then eb is not normal, so decide does not
# compare. Gappa proves the goal from the contradiction.
src = open("o2.g").read()
src = sub(src, "# o2.g\n", "# o2.g\n") if "# o2.g\n" in src else src
src = sub(src, "  # Both products are normal. on_floats does not check this.\n  /\\ |forwards| >= 1b-1022 /\\ |backwards| >= 1b-1022\n",
          "  # The case: both products are under 2^-1000.\n  /\\ |acx * bcy| <= 1b-1000 /\\ |acy * bcx| <= 1b-1000\n")
src = sub(src, "  # What lemma.g proves each of those three gives.\n"
          "  /\\ forwards -/ acx * bcy in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ backwards -/ acy * bcx in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
src = sub(src, "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n  /\\ (y - Ex) / eb in [-1, 1]", "  eb in ? /\\ (y - Ex) / eb in [-1, 1]")
open("o2tiny.g", "w").write(src)
```

`plane.g`, whole:

```gappa
# SideOfAFlatSurface::on_floats and SIDE_OF_PLANE_ERROR_BOUND,
# crates/rcad-geom/src/plane.rs at affe0f01.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

# What on_floats computes, line for line.
t0 = rnd(n0 * p0); t1 = rnd(n1 * p1); t2 = rnd(n2 * p2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = s012 - off;
value = rnd(y);

Phat = |t0| + |t1| + |t2| + |off|;
P = rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |off|);

B = 4b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = n0 * p0 + n1 * p1 + n2 * p2 - off;

# The three terms' part of the permanent, with no rounding.
c0 = |t0|; c1 = |t1|; c2 = |t2|;
P3 = c0 + c1 + c2;

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * p0); rp1 = t1 / (n1 * p1); rp2 = t2 / (n2 * p2);
rs01 = s01 / S01; rs012 = s012 / S012;

# Each of the three terms, as y carries it, less the term itself. The
# offset reaches y with no rounding.
W0 = rs01 * rs012 - 1 / rp0;
W1 = rs01 * rs012 - 1 / rp1;
W2 = rs012 - 1 / rp2;
e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2;

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  # Neither sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074
  # product_is_normal on the three products.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those four gives.
  /\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1] /\ t2 / |t2| in [-1, 1]
->
  (y - Ex) / P3 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The error is the sum of the three terms' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / P3 -> ((P3 + (y - Ex)) - P3) / P3;
P3 + (y - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> (t2 / |t2|) * W2;

# From the three terms to the whole permanent, which the offset only
# makes larger, to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P3) * (P3 / Phat) * (Phat / P) * (P / eb);
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

`planev.g` is `plane.g` with the claim taken on `value`: `rv = value / y;`
multiplies the first term of `W0`, `W1` and `W2`, the offset gets a term of its
own, `W3 = rv - 1;` and `e3 = -(off * W3);` with `c3 = |off|` in the average
in place of `P3`, the hypotheses gain `|off| >= 1b-1074 /\ |y| >= 1b-1074` and
`off / |off| in [-1, 1]`, and the hints gain `value / y -> 1 + (value - y) / y;`
and lose `P3 / Phat -> ...`. The zero offset is then on paper: it leaves
`value`'s error the same three terms.
