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
Q2658, which took up the two claims the pilot left unbacked, and 464.8 is
Q2659, which proved two more of the bounds. 464.9 is Q2660, on the eleven
`*_ROUNDOFF` constants. 464.10 is Q2661, which proves block 1's hints in Coq
and settles how the scripts join the tree and what notices the Rust moving
under them. 464.11 is Q2675, which runs the branches where one of the nine
differences is zero.*

  464.1   The bugs a bound prover is for
  464.2   What each tool is expected to do, untried here
  464.3   What the pilot answers
  464.4   What the pilot found
  464.5   The scripts, whole
  464.6   What this leaves
  464.7   value past the bound, and every minor at zero run
  464.8   orient2d and the side of a plane
  464.9   Which of the eleven *_ROUNDOFF constants Gappa can state
  464.10  Whether the scripts join the tree and the gate
  464.11  One of the nine differences at zero

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
from zero, but nothing in Coq checks it yet. `field` can, given 35 numbers
nonzero (464.10).

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
a looser bound are the ones §16.3 and §18.5 found too coarse. 464.10 settles
it: each script names the Rust it restates in its header, and a gate step,
`just check-proofs`, fails when that Rust's code moves from what the script's
stamp recorded.

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
difference zeroes its terms in both `y` and `Ex`. Q2675 runs each of the nine,
one at a time, and each proves (464.11). The branch where `t1 + t2` is
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
  was. Done for two, in 464.8: `ORIENT2D_ERROR_BOUND` and
  `SIDE_OF_PLANE_ERROR_BOUND` are proved at the code's constants, orient2d's
  with its products below the normals too, and each of the other six is a row
  of its own.
- **Q2660**, the eleven `*_ROUNDOFF` constants: which of them Gappa can state,
  and proofs of those. Done, in 464.9: three are proved at the code's
  constants, two of them false when every number is below `2⁻¹⁰²²`, and each
  of the others is a row of its own.
- **Q2661**, whether the scripts join the tree and the gate, what notices
  `on_floats` drifting from block 1, and whether `field` proves the 39 hint
  equalities once their divisors are shown nonzero. Done, in 464.10: `field`
  proves all 39, leaving 35 numbers taken to be nonzero; the scripts join the
  tree with a recipe to run them, and a gate step notices the Rust they
  restate moving.

None of the pilot's findings is a wrong answer found on an input. The sign
`decide` certifies is proved right on every input block 1 covers. What is left
is comments that claim more than is proved, zero branches argued on paper and
one of them not run, and 39 equalities Coq takes on trust. Since 464.7 the
comments in `orient3d.rs` say what is proved, and every branch with a minor at
zero is run. Since 464.10 the 39 equalities are proved, and what Coq takes on
trust is 35 facts that a number is not zero, each one step on paper from block
1's hypotheses. The branches where one of the nine differences is zero were on
paper, as 464.4 argues them; since 464.11 each is run, and proves at the code's
bound. Two differences at zero at once are still on paper, and so is the step
from `y` to the sign.

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
  zero, which 464.4 argues on paper only, reusing the scripts in 4. Done, in
  464.11: each of the nine proves at the code's bound.

### 464.8 orient2d and the side of a plane

Q2659 took up 464.6's eight other sign filters, in three lanes on 2026-10-23,
and did two of them: `ORIENT2D_ERROR_BOUND` and `SIDE_OF_PLANE_ERROR_BOUND`.
The first lane ran the proofs. The second ran the controls, `coqc`, and
plane's cases with a product below the normals. The third ran every script
again, wrote up the rest, and filed the other six as rows (7 below).

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
`[-1.33333, 1.33333]`, with `(value − Ex) / Phat` within `4u + 8u²`, and it
fails at `4u + 16u²` too, best `[-1, 1]` as Gappa prints it, rounded. The doc
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
`eb`, and the code's `16u²` has room for it.

A script whose hypotheses contradict each other proves every goal (464.7's
3), so each case was run below the code's `B` as well. At `B = 3u` each of
the four fails, best `[-1, 1]` as printed. At `3u + 14u²` each proves, where
`o2.g` needs `15u²`. `o2tiny.g` without `|eb| >= 1b-1022` fails, best
`[-inf, inf]`, printing `eb` at most `2^(-1050.41)`: what proves `o2tiny.g`
is the contradiction with `eb` normal and nothing else, which is what it is
meant to show.

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
proved. It did not prove it from hypotheses that contradict each other:
`planev.g` fails at `B = 4u`, best `[-1, 1]` as printed, and at `3u + 32u²`,
best `[-1.33333, 1.33333]`. `plane.g`'s own control is the search above,
where `3u + 14u²` fails.

**4. §12.2's check, and what `plane.rs` says about it.** Drop the three
`product_is_normal` hypotheses from `plane.g`, with the three lines block 2
gives from them (`c_plane_nonormal.g`), and the goal fails, best
`±7.2562e+908`. Gappa names where: it finds neither `REL(P, Phat)` nor a bound
on `(y − Ex) / P3`, and `rp0`, what rounding multiplied `n0 * p0` by, is known
only within `±2⁴¹⁴²`. So a script of block 1's kind catches a missing check,
as 464.4's 3 did for orient3d.

That shows the script needs the hypothesis, not that the code needs the
check, and the comment on the check says more. `plane.rs:851-853`: "One of
them in the subnormals and this level certifies a wrong sign." For one product
that is false. `planecases.py` (6 below) writes six scripts, `planes0.g` to
`planes2n.g`: product `k` below the normals, positive or negative, the other
two normal, and what rounding product `k` cost carried as an amount `dk`, not
a ratio, as `o2s.g` carries `G`'s. Each of the six proves at the code's `B`,
printing `dk` within `±2⁻¹⁰⁷⁵`, `P` at least `(2⁵⁰ − 1) · 2⁻¹⁰²¹`, just
under `2⁻⁹⁷¹`, and `(e0 + e1 + e2) / P3x` within `3u + 2u²`. Each fails at
`B = 3u`, best `[-1, 1]`, so none proved from a contradiction. The reason is
orient2d's: `eb` normal keeps `P` at about `2⁻⁹⁷¹` or more, so a product
below the normals costs at most `2⁻¹⁰⁷⁵`, which is at most `u` of `eb`, and
the code's `4u` has room for it over the `3u + 2u²` the products' rounding
takes. Two or three products below the normals at once were not
run, nor were the zero branches: a zero factor, a sum that cancels to zero,
`planev.g`'s zero offset. So whether `on_floats` needs the three checks at all
is a row (7 below). No input was built for any of it.

`plane.g` without `eb` normal, and without what block 2 gives from it
(`c_plane_noeb.g`), proves. With all three products normal Gappa prints `P`
at least `3 · 2⁻¹⁰²²`, and asked for `eb -/ B * P` it finds no bound
(`c_plane_noeb_why.g` fails on that goal alone), so how it gets past `eb`'s
rounding was not traced. On paper, not run: `B · P` is then at least `12 ·
2⁻¹⁰⁷⁵`, which rounding moves by at most `2⁻¹⁰⁷⁵`, a twelfth of it, and `3u`
of the permanent over eleven twelfths of `4u` of it is about `0.82`. `decide`
still checks that `eb` is normal, and that check is what keeps every
intermediate finite (464.4).

**5. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each of the fourteen scripts above that prove: exit 0 each, in 1.47 to 6.19 s
in the third lane's run and 0.67 to 1.32 s in the second's, `nix shell`'s
start counted in both. Gappa does not write the same proof each run (464.4),
so the sizes are the third lane's:

- `o2.g`: 2500 lines, 15 hints.
- `o2s.g`, `o2sn.g`, `o2f.g` and `o2fn.g`: 2735 to 2794 lines, 16 hints each.
- `o2tiny.g`: 524 lines, 15 hints.
- `plane.g`: 3020 lines, 17 hints.
- `planev.g`: 3376 lines, 19 hints.
- `planes0.g` to `planes2n.g`: 3349 to 3498 lines, 18 hints each.

Each hint enters the Coq file as a hypothesis with no condition attached, as
in 464.4, so each proof stands on its hints, and on the divisors Gappa took to
be nonzero. `field` took all 39 of block 1's hints (464.10). It has not been
run on the hints of these scripts, nor on those of 464.7 and 464.9.

**6. The scripts.** `o2.g`, whole:

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

`planecases.py`, whole, writes the six scripts of 4 and their runs at `B =
3u`; it reads no other file:

```python
"""Write plane.g's cases with one product below the normals, which
product_is_normal refuses: planes<k><sign>.g for product k in 0, 1, 2 and
its sign, "" positive or "n" negative, and each again at B = 3u as
c_planes<k><sign>_3u.g. 464.8 quotes planes0.g, the one for n0 * p0
positive, whole."""

R = {0: "rs01 * rs012", 1: "rs01 * rs012", 2: "rs012"}


def make(k, neg, b="B = 4b-53 + 1b-101;"):
    others = [i for i in (0, 1, 2) if i != k]
    abs_t = {i: f"|t{i}|" for i in others}
    abs_t[k] = "-t%d" % k if neg else "t%d" % k
    sign_word = "negative" if neg else "positive"
    lines = [
        f"# plane.g where n{k} * p{k} may be below the normals, which product_is_normal",
        f"# refuses: n{k} * p{k} is under 2^-1022 and {sign_word}, and the other two",
        "# products are normal.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);",
        "p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);",
        "off = rnd(off_);",
        "",
        f"X{k} = n{k} * p{k};",
        "; ".join(f"t{i} = rnd(X{k})" if i == k else f"t{i} = rnd(n{i} * p{i})" for i in (0, 1, 2)) + ";",
        "S01 = t0 + t1;",
        "s01 = rnd(S01);",
        "S012 = s01 + t2;",
        "s012 = rnd(S012);",
        "y = s012 - off;",
        "value = rnd(y);",
        "",
        f"Phat = {abs_t[0]} + {abs_t[1]} + {abs_t[2]} + |off|;",
        f"P = rnd(rnd(rnd({abs_t[0]} + {abs_t[1]}) + {abs_t[2]}) + |off|);",
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "Ex = " + " + ".join(f"X{k}" if i == k else f"n{i} * p{i}" for i in (0, 1, 2)) + " - off;",
        "",
        "; ".join(f"c{i} = -X{k}" if (i == k and neg) else f"c{i} = X{k}" if i == k else f"c{i} = |t{i}|" for i in (0, 1, 2)) + ";",
        f"P3 = {abs_t[0]} + {abs_t[1]} + {abs_t[2]};",
        f"# The three terms' part of the permanent with X{k} in place of t{k}.",
        "P3x = c0 + c1 + c2;",
        "",
        "; ".join(f"rp{i} = t{i} / (n{i} * p{i})" for i in others) + ";",
        "rs01 = s01 / S01; rs012 = s012 / S012;",
        "",
        f"# Each term's error as a fraction of the term, for X{k} and not t{k}, and",
        f"# what rounding X{k} cost, as an amount.",
    ]
    for i in (0, 1, 2):
        lines.append(f"W{i} = {R[i]} - 1;" if i == k else f"W{i} = {R[i]} - 1 / rp{i};")
    lines.append("; ".join(f"e{i} = X{k} * W{i}" if i == k else f"e{i} = t{i} * W{i}" for i in (0, 1, 2)) + ";")
    lines.append(f"d{k} = (t{k} - X{k}) * ({R[k]});")
    lines += [
        "",
        "{",
        "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024",
        "  /\\ |p0| <= 1b1024 /\\ |p1| <= 1b1024 /\\ |p2| <= 1b1024",
        "  /\\ |off| <= 1b1024",
        "  /\\ " + " /\\ ".join(f"|n{i}| >= 1b-1074" for i in others),
        "  /\\ " + " /\\ ".join(f"|p{i}| >= 1b-1074" for i in others),
        "  /\\ |S01| >= 1b-1074 /\\ |S012| >= 1b-1074",
        f"  # The case: X{k} is {sign_word} and below the normals, the other two",
        "  # products are normal. A product of two doubles that are not zero is at",
        "  # least 2^-2148.",
        f"  /\\ X{k} in " + ("[-1b-1022, -1b-2148]" if neg else "[1b-2148, 1b-1022]"),
        "  /\\ " + " /\\ ".join(f"|t{i}| >= 1b-1022" for i in others),
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
    ]
    for i in others:
        lines.append(f"  /\\ t{i} -/ n{i} * p{i} in [-4503599627370497b-105, 4503599627370497b-105]")
    lines += [
        "  /\\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]",
        "  /\\ " + " /\\ ".join(f"t{i} / |t{i}| in [-1, 1]" for i in others),
        "->",
        f"  (e0 + e1 + e2) / P3x in ? /\\ d{k} in ? /\\ P in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
    ]
    for i in others:
        lines.append(f"t{i} / (n{i} * p{i}) -> 1 + (t{i} - n{i} * p{i}) / (n{i} * p{i});")
    lines += [
        "s01 / S01 -> 1 + (s01 - S01) / S01;",
        "s012 / S012 -> 1 + (s012 - S012) / S012;",
        "",
        f"# The relative part is averaged over P3x as in plane.g; what rounding X{k}",
        "# cost is an amount, and eb being normal keeps it small against eb.",
        f"(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + d{k} / eb;",
        "(e0 + e1 + e2) / P3x -> ((P3x + (e0 + e1 + e2)) - P3x) / P3x;",
        "P3x + (e0 + e1 + e2) -> (c0 + e0) + (c1 + e1) + (c2 + e2);",
        "(c0 + e0) - c0 -> e0;",
        "(c1 + e1) - c1 -> e1;",
        "(c2 + e2) - c2 -> e2;",
    ]
    for i in (0, 1, 2):
        if i == k:
            lines.append(f"e{i} / c{i} -> " + (f"-W{i};" if neg else f"W{i};"))
        else:
            lines.append(f"e{i} / c{i} -> (t{i} / |t{i}|) * W{i};")
    lines += [
        f"P3x / P -> (P3 / Phat) * (Phat / P) + " + (f"(t{k} - X{k}) / P;" if neg else f"(X{k} - t{k}) / P;"),
        "P3 / Phat -> 1 / (1 + |off| / P3);",
        "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
        "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        "P -> eb / (B * (1 + (eb - B * P) / (B * P)));",
    ]
    return "\n".join(lines) + "\n"


for k in (0, 1, 2):
    for neg in (False, True):
        name = f"planes{k}{'n' if neg else ''}"
        open(f"{name}.g", "w").write(make(k, neg))
        open(f"c_{name}_3u.g", "w").write(make(k, neg, "B = 3b-53;"))
```

`planes0.g`, what it writes for `n0 * p0` positive, whole:

```gappa
# plane.g where n0 * p0 may be below the normals, which product_is_normal
# refuses: n0 * p0 is under 2^-1022 and positive, and the other two
# products are normal.
@rnd = float<ieee_64, ne>;

n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

X0 = n0 * p0;
t0 = rnd(X0); t1 = rnd(n1 * p1); t2 = rnd(n2 * p2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = s012 - off;
value = rnd(y);

Phat = t0 + |t1| + |t2| + |off|;
P = rnd(rnd(rnd(t0 + |t1|) + |t2|) + |off|);

B = 4b-53 + 1b-101;
eb = rnd(B * P);

Ex = X0 + n1 * p1 + n2 * p2 - off;

c0 = X0; c1 = |t1|; c2 = |t2|;
P3 = t0 + |t1| + |t2|;
# The three terms' part of the permanent with X0 in place of t0.
P3x = c0 + c1 + c2;

rp1 = t1 / (n1 * p1); rp2 = t2 / (n2 * p2);
rs01 = s01 / S01; rs012 = s012 / S012;

# Each term's error as a fraction of the term, for X0 and not t0, and
# what rounding X0 cost, as an amount.
W0 = rs01 * rs012 - 1;
W1 = rs01 * rs012 - 1 / rp1;
W2 = rs012 - 1 / rp2;
e0 = X0 * W0; e1 = t1 * W1; e2 = t2 * W2;
d0 = (t0 - X0) * (rs01 * rs012);

{
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024
  /\ |off| <= 1b1024
  /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074
  # The case: X0 is positive and below the normals, the other two
  # products are normal. A product of two doubles that are not zero is at
  # least 2^-2148.
  /\ X0 in [1b-2148, 1b-1022]
  /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 / |t1| in [-1, 1] /\ t2 / |t2| in [-1, 1]
->
  (e0 + e1 + e2) / P3x in ? /\ d0 in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The relative part is averaged over P3x as in plane.g; what rounding X0
# cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + d0 / eb;
(e0 + e1 + e2) / P3x -> ((P3x + (e0 + e1 + e2)) - P3x) / P3x;
P3x + (e0 + e1 + e2) -> (c0 + e0) + (c1 + e1) + (c2 + e2);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e0 / c0 -> W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> (t2 / |t2|) * W2;
P3x / P -> (P3 / Phat) * (Phat / P) + (X0 - t0) / P;
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
```

`controls.py`, whole, run beside the scripts above once `o2cases.py` has
written its four, writes every control of 1 to 4 but `planecases.py`'s and
`c_plane_noeb_why.g`. `o2v.g` and `planev.g` are the changes 1 and 3
describe:

```python
"""Write the control runs. Each changes one thing in a script that proves: a
smaller B, to see whether it proved its goal from hypotheses that contradict
each other, or a hypothesis dropped, to see whether the proof needs it."""


def sub(src, old, new):
    assert src.count(old) == 1, (old, src[:80])
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


B_O2 = "B = 3b-53 + 1b-102;"
for name in ("o2s", "o2sn", "o2f", "o2fn"):
    write(f"c_{name}_3u.g", sub(open(f"{name}.g").read(), B_O2, "B = 3b-53;"))
    write(f"c_{name}_14.g", sub(open(f"{name}.g").read(), B_O2, "B = 3b-53 + 14b-106;"))

# o2tiny without eb normal.
write("c_o2tiny_noeb.g", sub(open("o2tiny.g").read(), "  /\\ |eb| >= 1b-1022\n", ""))

# The claim on value for orient2d at 4u + 16u^2.
write("c_o2v_4u.g", sub(open("o2v.g").read(), B_O2, "B = 4b-53 + 1b-102;"))

# planev below the code's B.
B_PL = "B = 4b-53 + 1b-101;"
write("c_planev_4u.g", sub(open("planev.g").read(), B_PL, "B = 4b-53;"))
write("c_planev_3u.g", sub(open("planev.g").read(), B_PL, "B = 3b-53 + 1b-101;"))

# plane.g without product_is_normal, and what lemma.g gives from it.
src = open("plane.g").read()
src = sub(src, "  # product_is_normal on the three products.\n  /\\ |t0| >= 1b-1022 /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022\n", "")
src = sub(src, "  /\\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
src = sub(src, "  (y - Ex) / P3 in ? /\\ P -/ Phat in ?\n", "  (y - Ex) / P3 in ? /\\ P -/ Phat in ? /\\ rp0 in ?\n")
write("c_plane_nonormal.g", src)

# plane.g without eb normal.
src = open("plane.g").read()
src = sub(src, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
src = sub(src, "  /\\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
write("c_plane_noeb.g", src)
```

`c_plane_noeb_why.g` is `c_plane_noeb.g` with `P in ? /\ eb -/ B * P in ?
/\ ` put in front of its first goal. The smallest `B` of 1 and 3 was found by
bisection over `m` in `B = 3u + m u²`, each step `o2.g` or `plane.g` with its
`B` line changed and run as above. After the runs, `planecases.py`,
`controls.py` and that one change were run again, and wrote all 27 files byte
for byte as they were run.

**7. What this leaves.** The rows filed with this write-up:

- Six rows, one for each bound 464.6 named that is not proved yet:
  `surface.rs`' `ERROR_BOUND` in `rcad-predicates`, and
  `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND`,
  `SIDE_OF_SPHERE_ERROR_BOUND`, `SIDE_OF_CYLINDER_ERROR_BOUND`,
  `SIDE_OF_CONE_ERROR_BOUND` and `SIDE_OF_TORUS_ERROR_BOUND` in `rcad-geom`.
  Each is to be proved as 1 to 5 here prove orient2d and the side of a plane:
  `on_floats` line for line, the lemma for each product it checks is normal, a
  case split for each product it does not, a run below the code's constant,
  and `coqc` on each proof.
- A row to restate the note "Why this one needs no subnormal guard" on
  `ORIENT2D_ERROR_BOUND` (`orient2d.rs:335-358`) by 2: its conclusion holds,
  but because `eb` normal keeps a product below the normals small against
  `eb`, not by the argument it gives.
- A row to run `planecases.py` with two and three products below the normals
  at once, the zero branches 4 names, and a zero difference in orient2d,
  which `o2.g` assumes away as block 1 does (464.4), and then to restate
  `plane.rs:851-853` and say whether `SideOfAFlatSurface::on_floats` needs its
  three `product_is_normal` checks.

### 464.9 Which of the eleven *_ROUNDOFF constants Gappa can state

Q2660 read the eleven `*_ROUNDOFF` constants in `rcad-geom` at `c3b2160e`,
in three lanes on 2026-10-23.
Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, and `coqc`
under 464.5's pins with gappalib 1.8.0. `u` is `2⁻⁵³` and `ε` is
`f64::EPSILON = 2u`.

**Which Gappa can state.** Gappa proves a bound on one fixed expression in
`+ − × / sqrt fma abs` (464.2). By that test:

- **Stated and proved here:** `NEAREST_F64_POINT_ROUNDOFF` (`lib.rs:134-201`,
  `ε`), one rounding per coordinate; `Plane::SIGNED_DISTANCE_ROUNDOFF`
  (`plane.rs:85-112`, `4ε`), the arithmetic of `SideOfAFlatSurface::on_floats`
  without its checks; `SURFACE_AFTER_SHIFTING_ROUNDOFF`
  (`moved_corner.rs:99-143`, `8ε`), a shift with no turn, which carries the
  normal over exactly. A plane's offset becomes `offset + normal.dot(t)`
  (`plane.rs:544-550`, and `HalfSpace::shifted` at `half_space.rs:244-249`),
  which is `signed.g` with the point `t` and the offset's sign turned. A ball's
  centre or a pipe's axis origin becomes `rnd(cᵢ + tᵢ)`
  (`Isometry::apply_to_point`, `isometry.rs:303-308`, whose `turned_by` at
  515-520 hands the point back bit for bit when the turn is none), which is
  `nearest.g` with `x = c + t`.
- **Statable, each a larger script:** `Isometry::POINT_ROUNDOFF`
  (`isometry.rs:36-92`, `18ε`), Rodrigues' form through nalgebra's
  `transform_vector`, with a hypothesis on the quaternion's length; its doc
  counts `35.78u` against `36u`, so it needs averaging as `signed.g` does.
  `Plane::NORMAL_LENGTH_ROUNDOFF` (`plane.rs:114-165`, `64ε`), the
  constructor's rounding, then a turn, then `n·n`, resting on
  `POINT_ROUNDOFF`'s script; its doc says one share is measured, not
  derived. `CORNER_AFTER_MOVING_ROUNDOFF` (`moved_corner.rs:37-97`, `128ε`),
  one script per surface kind, each on `POINT_ROUNDOFF`'s. The one-stage
  recurrence of `EVALUATION_ROUNDOFF` (`de_boor.rs:28-114`), `E_r ≤
  E_{r−1}(1 + 9u) + 9uM`. The parabola branch of
  `POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF` (`walk.rs:110-214`, `64ε`).
- **Not statable in Gappa:** `EVALUATION_ROUNDOFF` over up to fifteen stages
  at any degree, and `REFINEMENT_ROUNDOFF` (`bspline_curve.rs:55-85`), two
  passes composed at any depth: Lean's (464.2). `Surface::ROUND_TRIP_ROUNDOFF`
  (`surface.rs:85-133`, `256ε`) runs through `atan2`, `cos` and `sin`,
  `POINT_ALONG_A_CROSSING_ROUNDOFF` (`walk.rs:57-108`, `256ε`) through `cos`
  and `sin`, and the hyperbola branch of the endless bend through `cosh` and
  `sinh`, which its doc already calls a hole in the derivation. Gappa has no
  way to write those; FPTaylor or Lean can, under an assumed `libm` error.

**1. `NEAREST_F64_POINT_ROUNDOFF`.** `nearest.g` proves the nearest `f64`
triple within `ε·L` of a point whose coordinates are at most `L`, for `L` at
least `2⁻¹⁰²²`, in 0.11 s, printing each coordinate's slip within `±u·L`. Its
two controls fail: with `L` allowed down to `2⁻¹⁰⁷⁴`, best `[0, 0.866025]`
(each slip within `±L/2`), and asked for `1.6875u`, below the `√3·u` the doc
counts, best `[1.8735e-16, 1.92296e-16]`, whose top is `√3·u`. So the doc's
condition, coordinates normal, is more than the bound needs: `L` normal is
enough, since a subnormal coordinate slips at most `2⁻¹⁰⁷⁵ = u·2⁻¹⁰²²`.
`gappa -Bcoq` writes 408 lines with 3 hints, and `coqc` accepts it, exit 0,
in 0.87 s.

**2. `SIGNED_DISTANCE_ROUNDOFF`.** `signed.g` proves `|value − Ex| ≤ 8u·L`,
the code's `4ε·L`, in 1.53 s, where `Ex` is the true `n·p − offset` of the
same `f64` and `L` is at least every `|pᵢ|` and `|offset|`. The hypotheses are
`SideOfAFlatSurface`'s products normal, which `signed_distance_to` does not
check, and `|n₀| + |n₁| + |n₂| ≤ 1774/1024`, which Cauchy-Schwarz gives on
paper for a normal whose `n·n` is within `NORMAL_LENGTH_ROUNDOFF` of one.
Plain interval bounds give `13u` here (`3·4u + u`); the doc's `(4√3 + 1)u`
comes from averaging, with each term's share of the bound `|nᵢ|·L` and the
offset's `L/4`, since it meets one rounding where a product meets four. It
prints `(value − Ex) / D` within `4u + O(u²)`, `D` the sum of the shares.
Its controls (`controls.py`) fail: at `7.875u`, best `±8.80372e-16`, which is
`7.93u`, the doc's own count; without the `Σ|nᵢ|` hypothesis, best
`±2.39501e+293`; without the products normal, best `±5.25151e+616`. `gappa
-Bcoq` writes 3047 to 3074 lines with 18 hints over four runs, since Gappa
does not write the same proof each run (464.4), and `coqc` accepts each, exit
0, in under two seconds.

The doc comment states no normal-number condition, and without one the bound
is false. `subnormal.py` runs `signed_distance_to`'s arithmetic, in Python's
`f64`, on `n = (0.6, 0.8, 0)`, `p = (2⁻¹⁰⁷⁴, 0, 0)`, offset `0`: the value is
`2⁻¹⁰⁷⁴` against a true `0.6·2⁻¹⁰⁷⁴`, an error of `0.4·L` against the
`8.9e-16·L` promised. The kernel gives the same, in 4 below. The test,
`reading_a_distance_off_a_plane_stays_inside_its_bound.rs`, asserts its
products stay normal, so it never meets this.

**3. `SURFACE_AFTER_SHIFTING_ROUNDOFF`.** The doc counts under `9u·L` for a
plane and under `2√3·u·L` for a ball or a pipe, and the constant is `16u`.

`shiftplane.g` is `signed.g` with the shift `d` for the point, `y = off +
s012` for `y = s012 − off`, and every size measured against `|n|·L` rather
than `L`, since an offset is a distance times `|n|` and
`a_surface_moved_by_a_shift_stays_inside_its_published_bound` divides both
the slip and the size by it. So it covers a half-space, whose normal is any
length, as well as a plane. Its hypotheses are `signed.g`'s, with `(|n₀| +
|n₁| + |n₂|) / |n| ≤ 1774/1024`, Cauchy-Schwarz on paper for every nonzero
`n`, in place of the bound on the sum. It proves the slip within `16u·|n|·L`
in under a second, printing `(value − Ex) / D` within `±2⁻⁵¹`, so `4u·D`, which is
`7.93u·|n|·L`. Its controls (`shiftcontrols.py`) fail: at `7.875u`, best
`±8.80372e-16`, which is `7.93u` again, and without the three products
normal, best `±1.26044e+632`.

`shiftball.g` proves the shifted centre within `16u·L` in under a second,
printing
each coordinate's slip within `±2u·L`, and needs no normal-number condition at
all: a sum of two `f64` that lands below `2⁻¹⁰²²` is exact, and Gappa finds
that without being told. Its controls fail: asked for `3.4u`, best `[0,
3.84593e-16]`, whose top is the doc's `2√3·u`; and with the centre on a grid
of `2⁻¹¹⁰⁰` in place of an `f64`, so that a small sum rounds, best `[0,
0.866025]`. Written as a plain real, `c0 = c0_`, that control made Gappa 1.8.0
abort in `src/parser/ast.cpp:255` while printing a warning, so it is written
with `fixed<-1100, ne>`.

`gappa -Bcoq` writes 2638 to 2651 lines with 18 hints for `shiftplane.g` over
four runs, and 991 lines with 6 hints for `shiftball.g` in both of its two, and
`coqc` accepts each, exit 0, in under two seconds.

**4. The subnormal case through the kernel.** A probe test, run with `just t`
and not kept, built `Plane::from_point_and_normal` through the origin with
normal `(0.6, 0.8, 0)`, which stores `(0.6, 0.8, 0.0)` and offset `0`, and read
the point `(2⁻¹⁰⁷⁴, 0, 0)`. It printed:

```text
normal [0.6, 0.8, 0.0], offset 0e0, computed 5e-324 = 1 * 2^-1074, error / L 0.4, SIGNED_DISTANCE_ROUNDOFF 8.881784197001252e-16
shifted: normal [0.6, 0.8, 0.0], offset 5e-324 = 1 * 2^-1074, slip / L 0.4, SURFACE_AFTER_SHIFTING_ROUNDOFF 1.7763568394002505e-15
```

The second line is the same plane moved by `Isometry::translation((2⁻¹⁰⁷⁴, 0,
0))`. A third lane wrote the probe again from this description, appended to
`crates/rcad-geom/tests/reading_a_distance_off_a_plane_stays_inside_its_bound.rs`,
and it printed the same two lines:

```rust
#[test]
fn q2660_probe_a_point_at_the_smallest_subnormal() {
    let smallest = f64::from_bits(1);
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.0), Vector::new(0.6, 0.8, 0.0))
            .unwrap();
    let normal = plane.normal().components();
    let computed = plane.signed_distance_to(Point::new(smallest, 0.0, 0.0));
    println!(
        "normal {normal:?}, offset {:e}, computed {computed:e} = {} * 2^-1074, error / L {}, SIGNED_DISTANCE_ROUNDOFF {:?}",
        plane.offset(),
        computed / smallest,
        computed / smallest - normal[0],
        Plane::SIGNED_DISTANCE_ROUNDOFF
    );
    let shifted = plane.transformed(
        &rcad_geom::Isometry::translation(Vector::new(smallest, 0.0, 0.0)).unwrap(),
    );
    println!(
        "shifted: normal {:?}, offset {:e} = {} * 2^-1074, slip / L {}, SURFACE_AFTER_SHIFTING_ROUNDOFF {:?}",
        shifted.normal().components(),
        shifted.offset(),
        shifted.offset() / smallest,
        shifted.offset() / smallest - normal[0],
        rcad_geom::SURFACE_AFTER_SHIFTING_ROUNDOFF
    );
}
```

`computed / smallest` and `offset / smallest` are exact, so `error / L` and
`slip / L` are each `1 − 0.6` in `f64`, which prints `0.4`. So both doc comments state a bound that is false when every number in
it is below `2⁻¹⁰²²`: `signed_distance_to` and a shifted plane's offset each
miss by `0.4·L`. That is class wrong/D (§352.3): the size `L` is below
`2.2e-308`, `2.2e-310` times §3.5's nominal `100`, and so far past its `1e11`
spread, and the absolute miss is `0.8·2⁻¹⁰⁷⁵`, nothing a tolerance can see.
The products-normal controls of `signed.g` and `shiftplane.g` must fail, since
this case is inside them. Whether `L` normal is enough, with a product below
`2⁻¹⁰²²`, is not run: such a product's rounding is no longer a relative slip,
which the averaging rests on, so it needs a split by size as `planecases.py`
did (464.8).

**What this leaves.**

- A Gappa script each for `Isometry::POINT_ROUNDOFF`, then
  `Plane::NORMAL_LENGTH_ROUNDOFF` and `CORNER_AFTER_MOVING_ROUNDOFF` on it,
  `EVALUATION_ROUNDOFF`'s one-stage recurrence, and the parabola branch of
  `POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`: five rows.
- `SIGNED_DISTANCE_ROUNDOFF` and the plane's shift with a product below the
  normals and `L` normal: a row.
- The two doc comments of 4, `plane.rs:85-112` and `moved_corner.rs:99-143`,
  restated with the condition the proofs need, once that row says whether `L`
  normal is enough: a row, wrong/D, after it.
- `EVALUATION_ROUNDOFF` and `REFINEMENT_ROUNDOFF` at any degree in Lean, and
  `ROUND_TRIP_ROUNDOFF`, `POINT_ALONG_A_CROSSING_ROUNDOFF` and the hyperbola
  branch in FPTaylor or Lean under an assumed `libm` bound: two rows.
- `NEAREST_F64_POINT_ROUNDOFF`'s doc asks for coordinates normal where `L`
  normal is enough. That asks a caller for more than it needs and promises
  nothing false, so it is no row.

`nearest.g`, whole:

```gappa
# NEAREST_F64_POINT_ROUNDOFF, crates/rcad-geom/src/lib.rs at c3b2160e:
# the nearest f64 triple to a point x lies within f64::EPSILON * L of it,
# L at least the largest |x_i|.
@rnd = float<ieee_64, ne>;

p0 = rnd(x0); p1 = rnd(x1); p2 = rnd(x2);

# Each coordinate's slip, per unit of L.
e0 = (p0 - x0) / L; e1 = (p1 - x1) / L; e2 = (p2 - x2) / L;

{
  # L is at least every |x_i|, and is a normal number no larger than the
  # largest f64.
     x0 / L in [-1, 1] /\ x1 / L in [-1, 1] /\ x2 / L in [-1, 1]
  /\ L in [1b-1022, 1b1024]
->
  e0 in ? /\ e1 in ? /\ e2 in ?
  /\ sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 1b-52]
}

# Above 2^-1022 a slip is relative to x_i, and x_i is at most L. Below it the
# slip is at most 2^-1075, and L is at least 2^-1022.
(p0 - x0) / L -> ((p0 - x0) / x0) * (x0 / L);
(p1 - x1) / L -> ((p1 - x1) / x1) * (x1 / L);
(p2 - x2) / L -> ((p2 - x2) / x2) * (x2 / L);
$ x0 in (-1b-1022, 1b-1022);
$ x1 in (-1b-1022, 1b-1022);
$ x2 in (-1b-1022, 1b-1022);
```

The control at `L` down to `2⁻¹⁰⁷⁴` is `nearest.g` with `L in [1b-1074,
1b1024]`; the one at `1.6875u` has the single goal `sqrt(e0 * e0 + e1 * e1 +
e2 * e2) in [0, 27b-57]`.

`signed.g`, whole:

```gappa
# Plane::signed_distance_to and Plane::SIGNED_DISTANCE_ROUNDOFF,
# crates/rcad-geom/src/plane.rs at c3b2160e: what it returns is within
# 4 * f64::EPSILON * L of the true value, L at least every |p_i| and |offset|.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

# normal.dot(point - origin) - offset. The origin is zero, so subtracting it
# rounds nothing, and nalgebra 0.35's dot on a Vector3 is (a + b) + c.
t0 = rnd(n0 * p0); t1 = rnd(n1 * p1); t2 = rnd(n2 * p2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = s012 - off;
value = rnd(y);

# The true value of the same inputs.
Ex = n0 * p0 + n1 * p1 + n2 * p2 - off;

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * p0); rp1 = t1 / (n1 * p1); rp2 = t2 / (n2 * p2);
rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;

# Each term as value carries it, less the term itself.
W0 = rv * rs012 * rs01 * rp0 - 1;
W1 = rv * rs012 * rs01 * rp1 - 1;
W2 = rv * rs012 * rp2 - 1;
W3 = rv - 1;
e0 = n0 * p0 * W0; e1 = n1 * p1 * W1; e2 = n2 * p2 * W2; e3 = -(off * W3);

# Each term's share of the bound: |n_i| L for a product, and L / 4 for the
# offset, which meets one rounding where a product meets four.
c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; c3 = 1b-2 * L;
D = c0 + c1 + c2 + c3;

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  # No sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074 /\ |y| >= 1b-1074
  # The three products are normal, which signed_distance_to does not check.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # What lemma.g proves each of those three gives.
  /\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the point and the offset.
  /\ L in [1b-1074, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1]
  /\ off / L in [-1, 1]
  # A stored normal has n.n within NORMAL_LENGTH_ROUNDOFF = 64 epsilon of
  # one, so |n0| + |n1| + |n2| <= sqrt(3 (1 + 64 epsilon)) < 1774 / 1024.
  /\ |n0| + |n1| + |n2| in [0, 1774b-10]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  W0 in ? /\ W2 in ? /\ W3 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / L in [-1b-50, 1b-50]
}

# How far each rounding is from the identity.
t0 / (n0 * p0) -> 1 + (t0 - n0 * p0) / (n0 * p0);
t1 / (n1 * p1) -> 1 + (t1 - n1 * p1) / (n1 * p1);
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors, each a fraction of its own
# share of D; add_rr then averages them.
(value - Ex) / L -> ((value - Ex) / D) * (D / L);
D / L -> |n0| + |n1| + |n2| + 1b-2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e0 / c0 -> (n0 / |n0|) * (p0 / L) * W0;
e1 / c1 -> (n1 / |n1|) * (p1 / L) * W1;
e2 / c2 -> (n2 / |n2|) * (p2 / L) * W2;
e3 / c3 -> -(4 * (off / L) * W3);
```

`controls.py`, whole, run beside `signed.g`:

```python
"""Write signed.g's controls. Each changes one thing in it: a smaller bound,
to see whether it proved its goal from hypotheses that contradict each other,
or a hypothesis dropped, to see whether the proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


signed = open("signed.g").read()
GOAL = "/\\ (value - Ex) / L in [-1b-50, 1b-50]"

# 7.875u, below the (4 sqrt(3) + 1)u = 7.93u the doc comment counts.
write("c_signed_7875.g", sub(signed, GOAL, "/\\ (value - Ex) / L in [-63b-56, 63b-56]"))

# Without |n0| + |n1| + |n2| bounded.
write("c_signed_nosum.g", sub(signed, "  /\\ |n0| + |n1| + |n2| in [0, 1774b-10]\n", ""))

# Without the three products normal, and what lemma.g gives from them.
src = sub(signed, "  /\\ |t0| >= 1b-1022 /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022\n", "")
src = sub(src, "  /\\ t0 -/ n0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t1 -/ n1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
write("c_signed_nonormal.g", src)
```

`subnormal.py`, whole:

```python
"""signed_distance_to's arithmetic, (n0*p0 + n1*p1) + n2*p2 - off in f64, on
a point whose largest coordinate is the smallest subnormal, against the
exact value of the same f64 inputs and against 4 * EPSILON * L."""

from fractions import Fraction as F
import sys

eps = sys.float_info.epsilon
n = (0.6, 0.8, 0.0)
p = (5e-324, 0.0, 0.0)
off = 0.0
L = max(abs(c) for c in p + (off,))

value = (n[0] * p[0] + n[1] * p[1]) + n[2] * p[2] - off
exact = sum(F(a) * F(b) for a, b in zip(n, p)) - F(off)
err = abs(F(value) - exact)
bound = F(4 * eps) * F(L)
print("n.n - 1 in epsilon:", float((sum(F(a) * F(a) for a in n) - 1) / F(eps)))
print("value:", value.hex(), "exact / 2^-1074:", float(exact / F(5e-324)))
print("error / L:", float(err / F(L)), "bound / L:", float(bound / F(L)))
print("error within 4 epsilon L:", err <= bound)
```

It printed:

```text
n.n - 1 in epsilon: 0.2
value: 0x0.0000000000001p-1022 exact / 2^-1074: 0.6
error / L: 0.4 bound / L: 8.881784197001252e-16
error within 4 epsilon L: False
```

`shiftplane.g`, whole:

```gappa
# Plane::transformed by a shift with no turn, HalfSpace::shifted, and
# SURFACE_AFTER_SHIFTING_ROUNDOFF, crates/rcad-geom/src at c3b2160e: the new
# offset is within 8 * f64::EPSILON * |n| * L of offset + n.t, L at least
# every |t_i| and |offset| / |n|.
@rnd = float<ieee_64, ne>;

# The normal, the shift and the offset, each an f64.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);
off = rnd(off_);

# self.offset + normal.dot(shift). A movement with no turn hands the normal
# back bit for bit, and nalgebra 0.35's dot on a Vector3 is (a + b) + c.
t0 = rnd(n0 * d0); t1 = rnd(n1 * d1); t2 = rnd(n2 * d2);
S01 = t0 + t1;
s01 = rnd(S01);
S012 = s01 + t2;
s012 = rnd(S012);
y = off + s012;
value = rnd(y);

# The true new offset of the same inputs, and the normal's true length.
Ex = off + n0 * d0 + n1 * d1 + n2 * d2;
N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);

# What each rounding multiplied its argument by.
rp0 = t0 / (n0 * d0); rp1 = t1 / (n1 * d1); rp2 = t2 / (n2 * d2);
rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;

# Each term as value carries it, less the term itself.
W0 = rv * rs012 * rs01 * rp0 - 1;
W1 = rv * rs012 * rs01 * rp1 - 1;
W2 = rv * rs012 * rp2 - 1;
W3 = rv - 1;
e0 = n0 * d0 * W0; e1 = n1 * d1 * W1; e2 = n2 * d2 * W2; e3 = off * W3;

# Each term's share of the bound: |n_i| L for a product, and |n| L / 4 for
# the offset, which meets one rounding where a product meets four.
c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; c3 = 1b-2 * (N * L);
D = c0 + c1 + c2 + c3;

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |d0| <= 1b1024 /\ |d1| <= 1b1024 /\ |d2| <= 1b1024
  /\ |off| <= 1b1024
  # No factor of the three products is zero.
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  /\ |d0| >= 1b-1074 /\ |d1| >= 1b-1074 /\ |d2| >= 1b-1074
  # No sum cancels to zero.
  /\ |S01| >= 1b-1074 /\ |S012| >= 1b-1074 /\ |y| >= 1b-1074
  # The three products are normal, which neither caller checks.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022
  # What lemma.g proves each of those three gives.
  /\ t0 -/ n0 * d0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ n1 * d1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ n2 * d2 in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the shift and the offset over |n|.
  /\ L in [1b-1074, 1b1024]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1]
  /\ off / (N * L) in [-1, 1]
  # |n0| + |n1| + |n2| <= sqrt(3) |n| < 1774 / 1024 |n|, by Cauchy-Schwarz.
  /\ (|n0| + |n1| + |n2|) / N in [0, 1774b-10]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ n0 / |n0| in [-1, 1] /\ n1 / |n1| in [-1, 1] /\ n2 / |n2| in [-1, 1]
->
  W0 in ? /\ W2 in ? /\ W3 in ? /\ (value - Ex) / D in ?
  /\ (value - Ex) / (N * L) in [-1b-49, 1b-49]
}

# How far each rounding is from the identity.
t0 / (n0 * d0) -> 1 + (t0 - n0 * d0) / (n0 * d0);
t1 / (n1 * d1) -> 1 + (t1 - n1 * d1) / (n1 * d1);
t2 / (n2 * d2) -> 1 + (t2 - n2 * d2) / (n2 * d2);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
value / y -> 1 + (value - y) / y;

# The error is the sum of the four terms' errors, each a fraction of its own
# share of D; add_rr then averages them.
(value - Ex) / (N * L) -> ((value - Ex) / D) * (D / (N * L));
D / (N * L) -> (|n0| + |n1| + |n2|) / N + 1b-2;
(value - Ex) / D -> ((D + (value - Ex)) - D) / D;
D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
e0 / c0 -> (n0 / |n0|) * (d0 / L) * W0;
e1 / c1 -> (n1 / |n1|) * (d1 / L) * W1;
e2 / c2 -> (n2 / |n2|) * (d2 / L) * W2;
e3 / c3 -> 4 * (off / (N * L)) * W3;
```

`shiftball.g`, whole:

```gappa
# Isometry::apply_to_point with no turn, as Sphere::transformed and
# Axis::transformed use it, and SURFACE_AFTER_SHIFTING_ROUNDOFF,
# crates/rcad-geom/src at c3b2160e: the shifted centre is within
# 8 * f64::EPSILON * L of c + t, L at least every |c_i| and |t_i|.
@rnd = float<ieee_64, ne>;

# The centre and the shift, each an f64.
c0 = rnd(c0_); c1 = rnd(c1_); c2 = rnd(c2_);
d0 = rnd(d0_); d1 = rnd(d1_); d2 = rnd(d2_);

# turned_by hands the centre back bit for bit when there is no turn, and the
# translation is one addition per coordinate.
x0 = c0 + d0; x1 = c1 + d1; x2 = c2 + d2;
p0 = rnd(x0); p1 = rnd(x1); p2 = rnd(x2);

# Each coordinate's slip, per unit of L.
e0 = (p0 - x0) / L; e1 = (p1 - x1) / L; e2 = (p2 - x2) / L;

{
  # L is at least every |c_i| and |t_i|, and need not be a normal number.
     c0 / L in [-1, 1] /\ c1 / L in [-1, 1] /\ c2 / L in [-1, 1]
  /\ d0 / L in [-1, 1] /\ d1 / L in [-1, 1] /\ d2 / L in [-1, 1]
  /\ L in [1b-1074, 1b1024]
->
  e0 in ? /\ e1 in ? /\ e2 in ?
  /\ sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 1b-49]
}

# A slip is relative to x_i, and x_i is at most 2L. Gappa needs no split at
# 2^-1022: a sum of two f64 that lands below it is exact.
(p0 - x0) / L -> ((p0 - x0) / x0) * (x0 / L);
(p1 - x1) / L -> ((p1 - x1) / x1) * (x1 / L);
(p2 - x2) / L -> ((p2 - x2) / x2) * (x2 / L);
x0 / L -> c0 / L + d0 / L;
x1 / L -> c1 / L + d1 / L;
x2 / L -> c2 / L + d2 / L;
```

`shiftcontrols.py`, whole, run beside the two:

```python
"""Write shiftplane.g's and shiftball.g's controls. Each changes one thing in
its script: a smaller bound, to see whether it proved its goal from
hypotheses that contradict each other, or a hypothesis dropped, to see
whether the proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


plane = open("shiftplane.g").read()
ball = open("shiftball.g").read()

# 7.875u, below the (4 sqrt(3) + 1)u = 7.93u the averaging reaches.
write("c_shiftplane_7875.g", sub(plane, "/\\ (value - Ex) / (N * L) in [-1b-49, 1b-49]",
                                 "/\\ (value - Ex) / (N * L) in [-63b-56, 63b-56]"))

# Without the three products normal, and what lemma.g gives from them.
src = sub(plane, "  /\\ |t0| >= 1b-1022 /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022\n", "")
src = sub(src, "  /\\ t0 -/ n0 * d0 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t1 -/ n1 * d1 in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ t2 -/ n2 * d2 in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
write("c_shiftplane_nonormal.g", src)

# 3.4u, below the 2 sqrt(3) u = 3.46u the doc comment counts.
write("c_shiftball_34.g", sub(ball, "sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 1b-49]",
                              "sqrt(e0 * e0 + e1 * e1 + e2 * e2) in [0, 435b-60]"))

# The centre on a grid of 2^-1100 rather than an f64: a sum below 2^-1022 is
# then no longer exact, and L may be below 2^-1022. (Written as a plain real,
# c0 = c0_, Gappa 1.8.0 aborts in ast.cpp printing a warning.)
write("c_shiftball_finer.g", sub(ball, "c0 = rnd(c0_); c1 = rnd(c1_); c2 = rnd(c2_);",
                                 "c0 = fixed<-1100, ne>(c0_); c1 = fixed<-1100, ne>(c1_);"
                                 " c2 = fixed<-1100, ne>(c2_);"))
```

### 464.10 Whether the scripts join the tree and the gate

Q2661 took up three questions, in three lanes on 2026-10-23, at `78d223eb`:

- whether Coq's `field` proves the hints `gappa -Bcoq` leaves as hypotheses;
- what would notice `on_floats` drifting from block 1;
- whether the scripts join the tree, and the gate.

Every run is under 464.4's pins, with gappalib 1.8.0 from 464.5's nix block.

**1. `field` proves block 1's 39 hints, given 35 numbers nonzero.** `field`
is the Coq tactic that proves an equation between fractions. It leaves as side
conditions that each divisor is not zero.

- `fieldtry.py`, in 4, asks `field` to prove each of block 1's 39 hints, with
  every definition that is not a rounding unfolded. It took every one, in 0.01
  to 0.61 s each. All it left were side conditions of the form `X <> 0`.
- `makedemo.py`, in 4, turns `block1.v` into `block1_proved.v`. There each
  `Hypothesis aN` becomes a `Lemma aN` that `field` proves. Each number `field`
  needs nonzero becomes a hypothesis of the Coq section, `nz_X : X <> 0`,
  declared just before the first hint that needs it. Which hint needs which
  was read off `fieldtry.py`'s run.
- `coqc` accepts `block1_proved.v`, exit 0, in 5.41 s and 11.55 s on two runs.
  `Print Assumptions l1` lists what block 1's last lemma stands on. It names
  the twelve coordinates, four axioms (the two Coq's reals rest on,
  functional extensionality and the excluded middle), and 35 `nz_` facts. It
  names no hint.
- The same print on `block1.v` as Gappa wrote it names 38 of the 39 hints.
  The one left out is `a38`, `Phat / P`. Its fact, `nz_Phat`, is the 36th
  declared, and the only one `l1` does not use.
- Gappa writes a different proof each run (464.4), so this was run twice: on
  a `block1.v` of 10169 lines and on a fresh one of 10007. On both, with
  `makedemo.py`'s table unchanged, `field` took all 39 hints, `coqc`
  accepted `block1_proved.v`, and `l1` stood on 35 `nz_` facts.

So block 1's proof now rests on 35 facts that a number is not zero, where it
rested on 39 equalities. A hint is false exactly where its divisor is zero, as
464.4 showed for `adx / Adx`, so the 35 are what the 39 stood in for. Each is
one step on paper from block 1's hypotheses:

- 20 are hypotheses read as "not zero". These are the nine differences `adx`
  to `cdz` and the three minors `first`, `second`, `third`, each at least
  `2⁻¹⁰⁷⁴`; the six products `bdxcdy` to `bdxady` and `eb`, each at least
  `2⁻¹⁰²²`; and `S = t1 + t2`, at least `2⁻¹⁰⁷⁴`.
- 13 follow because a rounding of zero is zero, which is Flocq's `round_0`:
  - the nine exact differences `Adx` to `Cdz`, which round to `adx` to `cdz`;
  - `M1` to `M3`, which round to the minors;
  - `P`, since `eb = rnd(B × P)` is not zero.
- 2 are arithmetic. `B` is a constant above zero, and `P6` is a sum of six
  products of absolute values that are not zero.

None of the 35 is proved in Coq yet. They would be proved from `s1`, the Coq
form of block 1's hypotheses: a nested `/\` of `BND <expr> <interval>` terms.
That is a row. `field` was run on block 1 alone. The scripts of 464.7 to 464.9
have not had it run on their hints.

**2. What would notice `on_floats` drifting.** Today nothing does (464.4's 5).
A check that fires when the Rust under a script changes is worth having only if
it fires on real changes, not on every edit to a doc comment. So `drift.sh`
(not kept) went through each commit that touched `orient3d.rs`. For each, it
asked whether three hashes had moved since the commit before:

- the whole file;
- `on_floats`' text, from `fn on_floats` to the first `    }`;
- `on_floats`' code: that text with every `//` line taken out, doc comments
  included, every blank line taken out, and every run of white space made one
  space.

Over the file's whole history, the 14 commits after `4941b88f` wrote it:

- the file's hash moved on all 14;
- the text's moved on 3;
- the code's moved on 1, `1ece0789`. That is §12.2's fix, a real change to the
  arithmetic.

Since the pilot read it at `d9f931e8`, up to `78d223eb`, 3 commits touched the
file. The file's hash moved on all 3 and the text's on 2, `813f8a09` and
`c827c89b`, which are Q2658's comments. The code's moved on none. `on_floats`
is 33 lines of code by that count at `78d223eb`.

So a hash of the file would have had someone re-read block 1 fourteen times
for one real change. A hash of the text would have done it three times. A hash
of the code would have done it once, the time it mattered. `cascade.rs`, whose
`product_is_normal` and `decide` block 1 also restates, changed once after the
pilot: `4eb9f90d`, a doc comment only.

**3. The decision.**

a. **`field` proves the hints**, in every script where it takes them, through
the transform of 4. A hint it does not take stays a hypothesis. The nonzero
facts stay section hypotheses until a row proves them from `s1`.
`prove-bounds`, in c, counts both for each proof, from `Print Assumptions`.

b. **Each proved script joins the tree** as `proofs/gappa/<name>.g`.

- Its controls go beside it as `proofs/gappa/c_<name>_<what>.g`. A control is
  a script that must fail, as each of 464.4's variants must. 464.8 and 464.9
  already name theirs `c_*.g`.
- A script that a Python generator writes is committed as the files it writes,
  with the generator beside them, so that each stamp names a file. The
  generators are `o2cases.py`, `planecases.py`, `controls.py` and
  `shiftcontrols.py`.
- Each script starts with one line per item of Rust it restates. The item is
  named as `just src` takes it, with the commit it was read at:

```gappa
# restates: crates/rcad-predicates/src/orient3d.rs on_floats at d9f931e8
# restates: crates/rcad-predicates/src/orient3d.rs exact_coordinates at d9f931e8
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at d9f931e8
# restates: crates/rcad-predicates/src/orient3d.rs ORIENT3D_ERROR_BOUND at d9f931e8
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at d9f931e8
# restates: crates/rcad-predicates/src/cascade.rs decide at d9f931e8
```

- A dependency whose arithmetic a script restates is a line too. `signed.g`
  restates nalgebra 0.35's `dot` (464.9), so it carries
  `# restates: Cargo.lock nalgebra at c3b2160e`. That line's hash is the
  package's entry in `Cargo.lock`.

c. **`just prove-bounds [script]`** runs each script, or the one named, under
464.4's pins:

- `gappa`, `gappa -Bcoq`, the transform of 4, and `coqc`;
- each of the script's controls, each of which must fail.

Before it writes a stamp, it checks each `restates` line. The item's code now
must be the item's code at the commit the line names. So when the Rust under a
script moves, the script has to be read again against it, and the line moved
to a commit after that reading, before the script can be stamped again.
Without this check, running `prove-bounds` would re-stamp an old script
against new code, and that hides the drift the stamp is there to show.

It then writes `proofs/gappa/<name>.stamp`, holding:

- the script's hash;
- each restated item's code hash;
- the pin;
- the count of hints and of nonzero facts the proof stands on.

`prove-bounds` is not a gate step, for three reasons:

- CI has no nix. `grep -ci nix` counts 0 lines in both workflows, and CI runs
  `just gate` (`ci.yml:301`), so a gate step CI could not run would split the
  two. CI's minutes are paid for, and get spent, too.
- The first fetch is 583 MiB of Coq (464.4).
- So far the verdict has depended only on the script and the pin. Gappa
  writes a different proof each run (464.4), but no run has changed whether a
  script proves. Running an unchanged script under an unchanged pin again
  would learn what the stamp already says.

d. **A gate step, `just check-proofs`**, needs no nix, so CI runs it. It reads
the working tree only, because CI's `actions/checkout@v4` fetches one commit
and no history. For each script it hashes the script. It recomputes each
restated item's code hash, using `just src`'s extent of the item and the way
2 takes comments and blank lines out. It fails, naming the script and `just
prove-bounds <script>`, in four cases:

- a hash differs from the stamp;
- the stamp's pin differs from the one `prove-bounds` uses;
- a script has no stamp;
- an item is no longer found.

`check-proofs-selftest` works in a scratch copy. It changes a restated item's
code, a comment in it, and a script, one at a time, and checks that the code
and the script fail and the comment passes. By 2's figures, `check-proofs`
would have fired once in `orient3d.rs`' fourteen commits, at §12.2's fix, and
not at all since the pilot.

What it misses:

- a change to an item no header names, such as a helper `on_floats` calls;
- a script that got its Rust wrong when it was written or read again. The
  check compares the code with the code, never with the script.

e. **A check that runs the script's expression in `f64` against `on_floats`**
was weighed and not built. It would compare `value` and `eb` bit for bit on
sampled inputs.

- For it: it would catch a script that restates its code wrong, which d does
  not.
- Against it: it needs an evaluator for Gappa's language, written in Rust.
  And it sees only `value` and `eb`, on the inputs sampled. It sees neither the
  checks `product_is_normal` makes nor `decide`'s comparison.

By §13 it is built when a script is found to restate its code wrong.

f. **The fourteen open Gappa rows** go on appending their scripts to this
section, as they were filed: Q2675, Q2677 to Q2682, Q2684, and Q2689 to
Q2694. So the proofs do not wait on the tooling. The row that builds c and d
moves in the scripts this section holds when that row is taken. A row after it,
and after the fourteen, moves in the rest.

**4. The transform, whole.** To run it again, run 464.5's commands to write
`block1.v`, with `nx`, `pin`, `gl` and `fl` set as there. Save the two scripts
below beside it as `fieldtry.py` and `makedemo.py`, and then:

```sh
python3 fieldtry.py block1.v    # 39 hints; 279 definitions unfolded
python3 makedemo.py block1.v    # 39 hints proved by field; 36 nonzero hypotheses
for f in block1_field1.v block1_proved.v block1_asis.v; do
  COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
    $nx shell $pin#coqPackages_8_20.coq --command coqc $f > ${f%.v}.out
done
grep -c 'FIELD FAILED' block1_field1.out    # 0
grep -c '^nz_' block1_proved.out            # 35
grep -c '^a[0-9]' block1_asis.out           # 38
```

`block1_field1.out` lists, for each hint, the side conditions `field` left.
`block1_asis.v` is `block1.v` with only `Print Assumptions l1.` added. A
transform for `prove-bounds` reads that table off `field`'s side conditions,
as `fieldtry.py` prints them, rather than by hand.

Some things were tried and did not work:

- `Require Import Reals` after gappalib's `Require` hides gappalib's `neg`,
  and the proof then fails at a term citing it: "has type R while it is
  expected to have type negreal". So both scripts put `Reals` first.
- `field` with no `Reals` imported fails: "Cannot find a declared field
  structure over R".
- Gappa's `Float1` is not a defined object until gappalib is required.

```python
"""Take the hint section of a gappa -Bcoq file and ask Coq's field to prove
each hint hypothesis aN, with every definition that is not a rounding
unfolded. Writes <stem>_field1.v, which prints for each hint whether field
took it and the side conditions it left, without proving them.

Run as: python3 fieldtry.py block1.v
"""
import re
import sys

src_path = sys.argv[1]
stem = src_path[: -len(".v")]
lines = open(src_path).read().split("\n")

defs = []  # (name, body) in order
hyps = []  # (name, statement)
cut = None
for i, line in enumerate(lines):
    m = re.match(r"^Definition (\w+) := (.*)\.( \(\*.*\*\))?$", line)
    if m:
        defs.append((m.group(1), m.group(2)))
        continue
    m = re.match(r"^Hypothesis (a\d+) : (.*)\.$", line)
    if m:
        hyps.append((m.group(1), m.group(2), i))
last_hyp_line = hyps[-1][2]
# The hint section ends with the last hint's Lemma bN ... Qed.
end = last_hyp_line
while not lines[end].startswith("Qed."):
    end += 1
head = lines[: end + 1]

arith = [n for n, b in defs if "rounding_float" not in b and not b.startswith("BND")
         and not b.startswith("makepairF") and not b.startswith("Float2")
         and not b.startswith("REL") and not b.startswith("ABS")]
# Only those defined in the head.
head_text = "\n".join(head)
arith = [n for n in arith if re.search(rf"^Definition {n} :=", head_text, flags=re.M)]
unfold = ", ".join(reversed(arith))

out = [head[0], "From Coq Require Import Reals Field.", "Print Float1.", "Print float2R.", "Print rounding_float.", ""]
out += head[1:]
for name, stmt, _ in hyps:
    out += [
        f"Goal {stmt}.",
        "Proof.",
        f"  unfold {unfold}.",
        "  unfold Float1.",
        f"  Time first [ field | idtac \"{name} FIELD FAILED\" ].",
        f"  all: match goal with |- ?G => idtac \"{name} SIDE\" G end.",
        "Abort.",
    ]
out.append("End Generated_by_Gappa.")
open(f"{stem}_field1.v", "w").write("\n".join(out) + "\n")
print(len(hyps), "hints;", len(arith), "definitions unfolded")
```

```python
"""Turn gappa -Bcoq's block1.v into block1_proved.v: each hint
`Hypothesis aN : S.` becomes `Lemma aN : S.` proved by field, and the
nonzero facts field asks for become Section hypotheses nz_*, each declared
just before the first hint that needs it. Both files get a
`Print Assumptions l1.` before `End Generated_by_Gappa.`, so coqc prints
what the final lemma stands on in each.

Run as: python3 makedemo.py block1.v
"""
import re
import sys

src_path = sys.argv[1]
stem = src_path[: -len(".v")]
lines = open(src_path).read().split("\n")

# The nonzero facts field asked for, by the first hint that asks for each,
# read off block1_field1.v's run. Any other side condition field leaves is
# one of these up to Rabs or ring.
first_needed = {
    1: ["_Adx"], 2: ["_Ady"], 3: ["_Adz"],
    4: ["_Bdx"], 5: ["_Bdy"], 6: ["_Bdz"],
    7: ["_Cdx"], 8: ["_Cdy"], 9: ["_Cdz"],
    10: ["_cdy", "_bdx"], 11: ["_bdy", "_cdx"], 12: ["_ady"], 13: ["_adx"],
    16: ["_M1"], 17: ["_M2"], 18: ["_M3"],
    19: ["_first", "_adz"], 20: ["_second", "_bdz"], 21: ["_third", "_cdz"],
    22: ["_S"],
    23: ["_P6"],
    24: ["_bdxcdy", "_cdxbdy", "_cdxady", "_adxcdy", "_adxbdy", "_bdxady"],
    37: ["_eb", "_P"],
    38: ["_Phat"],
    39: ["_B"],
}

closer = """Ltac close_nz :=
  first [ assumption
        | apply Coq.Reals.Rbasic_fun.Rabs_no_R0; assumption
        | match goal with H : ?X <> 0%R |- ?E <> 0%R =>
            let Hz := fresh in
            intro Hz; apply H; transitivity E; [ring | exact Hz]
          end ]."""

defined = []
out = []
proved = []
for line in lines:
    m = re.match(r"^Definition (\w+) := (.*)\.( \(\*.*\*\))?$", line)
    if m:
        defined.append((m.group(1), m.group(2)))
    m = re.match(r"^Hypothesis a(\d+) : (.*)\.$", line)
    if m:
        n = int(m.group(1))
        for name in first_needed.get(n, []):
            out.append(f"Hypothesis nz{name} : {name} <> 0%R.")
        arith = [d for d, b in defined if "rounding_float" not in b]
        out += [
            f"Lemma a{n} : {m.group(2)}.",
            "Proof.",
            f"  unfold {', '.join(reversed(arith))}.",
            "  unfold Float1.",
            "  field; repeat split; close_nz.",
            "Qed.",
        ]
        proved.append(n)
        continue
    if line == "End Generated_by_Gappa.":
        out.append("Print Assumptions l1.")
    out.append(line)
    if line == "Section Generated_by_Gappa.":
        out.append(closer)

# Before gappalib, so that Reals' `neg` does not shadow the gappalib lemma
# the proof cites.
out.insert(0, "From Coq Require Import Reals Field.")
open(f"{stem}_proved.v", "w").write("\n".join(out))

plain = [l if l != "End Generated_by_Gappa." else "Print Assumptions l1.\n" + l
         for l in lines]
open(f"{stem}_asis.v", "w").write("\n".join(plain))
print(len(proved), "hints proved by field;",
      sum(len(v) for v in first_needed.values()), "nonzero hypotheses")
```

`close_nz` closes a side condition in one of three ways:

- by a hypothesis as it stands;
- through `Rabs_no_R0`, for `|X| <> 0`;
- by `ring`, from a hypothesis about an equal number. This takes
  `Phat + (P − Phat)` and `B × P + (eb − B × P)`.

**What this leaves.**

- The build of 3 b to d: `proofs/gappa/`, `prove-bounds`, the stamps,
  `check-proofs` in the gate, and `check-proofs-selftest`. That is a row.
- Moving in the scripts the fourteen rows of 3 f append after the build row
  is taken: a row, after the build row and the fourteen.
- The 35 nonzero facts of 1, proved in Coq from `s1`, so that block 1's proof
  stands on its hypotheses alone: a row, after the build row.

### 464.11 One of the nine differences at zero

Q2675 ran block 1 with one of the nine differences `adx` to `cdz` at zero, in
three lanes on 2026-10-24. 464.4's paper argument holds there. Each of the
nine branches proves at the code's `B` and fails at `6u`, and Coq checks each
proof, modulo its hints. Nothing in `orient3d.rs` needs to change for it. Two
differences at zero at once, and a difference at zero with a minor at zero, are
still not run.

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200` and the code's
`B = 7u + 56u²`, against `on_floats` as 464.4 read it at `d9f931e8`: no line of
its code has changed since, up to `ca9b6493`.

**The two steps 464.4 takes on paper.** Both prove in Gappa:

```gappa
# diffzero.g: a difference of two doubles that rounds to zero is zero.
@rnd = float<ieee_64, ne>;
x = rnd(x_); z = rnd(z_);
D = x - z;
{ |x| <= 1b1024 /\ |z| <= 1b1024 /\ rnd(D) in [0, 0] -> D in [0, 0] }
```

```gappa
# refl.g: x - x is zero, and so are its rounding and a product with it.
@rnd = float<ieee_64, ne>;
x = rnd(x_); y = rnd(y_);
a = x - x;
p = a * y;
{ |x| <= 1b1024 /\ |y| <= 1b1024 -> a in [0, 0] /\ rnd(a) in [0, 0] /\ rnd(p) in [0, 0] }
```

Gappa's check that a hint is an identity cancels `x - x`, but takes `|x - x|`
as a number of its own: a hint that needs `|x - x|` to be zero draws "not
trivially equal".

**The scripts.** `make_one_difference_zero.py`, below, run as `python3
make_one_difference_zero.py adx ady adz bdx bdy bdz cdx cdy cdz` beside
`block1.g`, writes `zd_<name>.g` for each. In each, `d`'s coordinate is the
same double as the point's, the difference and each product with it are left
unrounded, and a term whose product is zero stays in `P6` and in the split of
`P6 + (y − Ex)` as a bare `c_k`, with no error of its own.

Each of the nine proves at the code's `B`:

- `adx`, `ady`, `bdx`, `bdy` and `cdz` zero: `(y − Ex) / P6` within
  `7.77156e-16`, which is `7u + 11u²` and some `u³`.
- `adz`, `bdz`, `cdx` and `cdy` zero: within `6.66134e-16`, `6u + 13u²` and
  some `u³`.
- `P −/ Phat` within `4.44089e-16` in each: `4u + 9u²` above for the six x
  and y differences, `4u + 7u²` for the three z ones.

Each fails below `B`. At `B = 6b-53` the `7u` scripts fail with best
`±1.16667` and the `6u` ones with best `±1`. At `B = 5b-53` they fail with
`±1.4` and `±1.2`.

Without the last lines the generator writes, eight of the nine failed: `adz`
and `bdz` with `(y − Ex) / P6` only within `±2.3429e+1203`, and the six x and
y differences at `REL(P, Phat)`, with the goal's best `±3.24661e+1218`. Gappa
asked for more bounds says why. With `adx` zero, `REL(second, M2)`,
`REL(third, M3)`, `REL(q2, |cdxady| + |adxcdy|)` and `REL(q3, ...)` fail. With
`adz` zero, `rs` gets only `±2.31901e+1217`. Each of these rounds a sum one of
whose parts is the unrounded zero, and Gappa has no `FIX` for that part, so it
does not know the sum is a double. The hints `cdxady - adxcdy -> cdxady;` and
`t1 + t2 -> t2;` pass its check, since `adx` is `ax − ax`, and then the
rounding is of a double. The same kind of hint for `q2`, `|cdxady| + |adxcdy|
-> |cdxady|;`, does not pass it: Gappa warns that the two sides are "not
trivially equal", with the difference `|adxcdy|`, as the `|x − x|` above
says. It then uses the hint anyway, and with it and `q3`'s the `adx` script
proves with the same two bounds. But that hint would go into Coq as an
equality that holds only because `adx` is zero, not as an identity Gappa
checked, and 464.4's 39 hints are all identities. So the scripts instead take
`q`'s rounding as off by at most `u + 2u²`, as a hypothesis, which lemma.g
gives once `|q| ≥ 2⁻¹⁰²²`, and a goal checks that `|q| ≥ 2⁻¹⁰²²`. The goal
does not lean on the hypothesis it backs. With the `q` hypotheses taken out
and the `|q|` goals alone left, each of the six x and y scripts still proves
them, in 2.5 to 2.6 s. The same goals at `2⁻¹⁰²¹`, false where `cdxady` is
`2⁻¹⁰²²`, do not prove for `adx` or `cdy` within 100 s.

Putting the zero `c`'s last in `P6` and in the split, as `cdz`'s are, changed
nothing: without the hints `adz` still printed `±2.3429e+1203`.

`gappa -Bcoq` writes each of the nine proofs, 8591 to 10060 lines over two
runs, since Gappa does not write the same proof each run, with 32 to 34 hints.
`coqc` under 464.5's commands accepts each, exit 0, in 2.6 to 9.8 s. `field`
has not been run on these hints (464.10).

```python
"""Write block 1 with one of the nine differences adx to cdz zero, as zd_<name>.g."""
import sys

# The six products: name, factors, the minor they are in.
PRODUCT = {
    1: ("bdxcdy", "bdx * cdy", 1),
    2: ("cdxbdy", "cdx * bdy", 1),
    3: ("cdxady", "cdx * ady", 2),
    4: ("adxcdy", "adx * cdy", 2),
    5: ("adxbdy", "adx * bdy", 3),
    6: ("bdxady", "bdx * ady", 3),
}
MINOR = {1: "first", 2: "second", 3: "third"}
# Each minor's two products, and what its q rounds.
MINUS = {1: ("bdxcdy", "cdxbdy"), 2: ("cdxady", "adxcdy"), 3: ("adxbdy", "bdxady")}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}
# The z difference each term multiplies its minor by.
TERM_OF = {"adz": 1, "bdz": 2, "cdz": 3}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def drop_conj(src, item):
    """Take one conjunct out of a hypothesis line."""
    if src.count(item + " /\\ ") == 1:
        return src.replace(item + " /\\ ", "")
    if src.count(" /\\ " + item + "\n") == 1:
        return src.replace(" /\\ " + item + "\n", "\n")
    return sub(src, "  /\\ " + item + "\n", "")


def drop_def(src, item):
    """Take one definition out of a line of definitions."""
    if src.count(item + " ") == 1:
        return src.replace(item + " ", "")
    if src.count(" " + item + "\n") == 1:
        return src.replace(" " + item + "\n", "\n")
    return sub(src, item + "\n", "")


def make(name):
    point, axis = name[0], name[2]
    exact = name[0].upper() + name[1:]
    src = open("block1.g").read()
    src = sub(
        src,
        "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
        "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
        f"# Block 1 where {name} is zero: d{axis} is the same double as {point}{axis}. A\n"
        "# zero rounds to itself, and so does a product with a zero factor.\n",
    )
    src = sub(src, f"d{axis} = rnd(d{axis}_);", f"d{axis} = {point}{axis};")
    src = sub(src, f"{name} = rnd({exact});", f"{name} = {exact};")
    src = drop_def(src, f"r{name} = {name} / {exact};")
    src = drop_conj(src, f"|{name}| >= 1b-1074")
    src = sub(src, f"{name} / {exact} -> 1 + ({name} - {exact}) / {exact};\n", "")
    if name in TERM_OF:
        m = TERM_OF[name]
        gone = [2 * m - 1, 2 * m]
        minor = MINOR[m]
        src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
        src = sub(src, f"r{m} = rnd(R{m});", f"r{m} = R{m};")
        src = drop_def(src, f"rm{m} = {minor} / M{m};")
        src = drop_def(src, f"rt{m} = t{m} / U{m};")
        src = drop_conj(src, f"|{minor}| >= 1b-1074")
        src = drop_conj(src, f"|t{m}| >= 1b-1022")
        src = drop_conj(src, f"|r{m}| >= 1b-1022")
        src = sub(src, f"  /\\ t{m} -/ U{m} {BOUND}", "")
        src = sub(src, f"  /\\ r{m} -/ R{m} {BOUND}", "")
        src = drop_conj(src, f"{name} / |{name}| in [-1, 1]")
        src = sub(src, f"{minor} / M{m} -> 1 + ({minor} - M{m}) / M{m};\n", "")
        src = sub(src, f"t{m} / U{m} -> 1 + (t{m} - U{m}) / U{m};\n", "")
        others = [MINOR[k] for k in (1, 2, 3) if k != m]
        src = sub(
            src,
            "  # The permanent's own three products are normal too. The comment above\n"
            "  # stays_accurate argues it from the nine.\n",
            f"  # The other two of the permanent's products are normal because |{others[0]}|\n"
            f"  # and |{others[1]}| are at most their q. r{m} is zero.\n",
        )
        src = sub(
            src,
            "  # §12.2's first condition: product_is_normal on the nine products.\n",
            "  # §12.2's first condition: product_is_normal on the nine products. The\n"
            f"  # check on {name} and {minor} passes on the zero, and says nothing.\n",
        )
        if m == 3:
            tail = ""
        else:
            other = 3 - m
            tail = (
                f"\n# t{m} is zero, so t1 + t2 is the double t{other} and rounds to itself.\n"
                f"t1 + t2 -> t{other};\n"
            )
    else:
        gone = [k for k, (_, f, _) in PRODUCT.items() if name in f]
        for k in gone:
            prod, factors, _ = PRODUCT[k]
            src = sub(src, f"{prod} = rnd({factors});", f"{prod} = {factors};")
            src = drop_def(src, f"rp{k} = {prod} / ({factors});")
            src = drop_conj(src, f"|{prod}| >= 1b-1022")
            src = sub(src, f"  /\\ {prod} -/ {factors} {BOUND}", "")
            src = drop_conj(src, f"{prod} / |{prod}| in [-1, 1]")
            src = sub(src, f"{prod} / ({factors}) -> 1 + ({prod} - {factors}) / ({factors});\n", "")
        names = " and ".join(PRODUCT[k][0] for k in gone)
        src = sub(
            src,
            "  # §12.2's first condition: product_is_normal on the nine products.\n",
            "  # §12.2's first condition: product_is_normal on the nine products. The\n"
            f"  # checks on {names} pass on the zero, and say nothing.\n",
        )
        minors = [PRODUCT[k][2] for k in gone]
        hints = []
        for k in gone:
            prod, _, m = PRODUCT[k]
            left, right = MINUS[m]
            hints.append(f"{left} - {right} -> -{right};\n" if prod == left else f"{left} - {right} -> {left};\n")
        qs = [f"q{m}" for m in minors]
        last = f"  /\\ eb -/ B * P {BOUND}"
        src = sub(
            src,
            last,
            last + f"  # {qs[0]} and {qs[1]} each round a double plus a zero, so each is at least\n"
            "  # 2^-1022 as that double is, and lemma.g gives the same of them. The\n"
            "  # goals check the first part.\n"
            + "".join(f"  /\\ q{m} -/ ({NOTE[m]}) {BOUND}" for m in minors),
        )
        goal = "  (y - Ex) / P6 in ? /\\ P -/ Phat in ?\n"
        src = sub(src, goal, goal + "".join(f"  /\\ |q{m}| >= 1b-1022\n" for m in minors))
        mins = " and ".join(MINOR[m] for m in minors)
        tail = (
            f"\n# {names} are zero, so {mins} each round a double less a zero, or a\n"
            "# zero less a double, and so round to themselves.\n" + "".join(hints)
        )
    src = sub(
        src,
        "  # No factor of the nine products is zero. For an f64 that is the same as\n"
        "  # being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {name}. For an f64 that is\n"
        "  # the same as being at least the smallest subnormal.\n",
    )
    # A term whose product is zero carries no error: it is zero in y and in
    # Ex alike, and stays in P6 only as the zero it is.
    for k in gone:
        src = sub(src, line_starting(src, f"W{k} = ") + "\n", "")
        line = line_starting(src, "", f"e{k} = ")
        part = line[line.index(f"e{k} = "):]
        src = drop_def(src, part[: part.index(";") + 1])
        src = sub(src, f"(c{k} + e{k}) - c{k} -> e{k};\n", "")
        src = sub(src, line_starting(src, f"e{k} / c{k} -> ") + "\n", "")
        src = sub(src, f"(c{k} + e{k})", f"c{k}")
    return src + tail


def line_starting(src, start, containing=""):
    (line,) = [l for l in src.split("\n") if l.startswith(start) and containing in l]
    return line


for name in sys.argv[1:]:
    open(f"zd_{name}.g", "w").write(make(name))
```

**What this leaves.**

- Two or more of the nine differences at zero at once. Some pairs zero a
  minor too: `adx` and `bdx` zero both of `third`'s products. A row.
- One difference at zero with one minor at zero, 464.7's case, such as `adx`
  with `first`. A row.
- The row that moves the scripts into `proofs/gappa/` after the fourteen of
  464.10 3 f now waits on those two as well, since they append here too.
