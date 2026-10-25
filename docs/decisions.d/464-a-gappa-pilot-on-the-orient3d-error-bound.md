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
differences is zero. 464.12 is Q2677, on the sphere in `surface.rs`' tests.
464.13 is Q2678, on the side of a flat surface through a point.*

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
  464.12  The sphere of surface.rs' tests
  464.13  The side of a flat surface through a point

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

There are twenty bound constants, found by `grep -rnE 'const
[A-Z0-9_]*(ROUNDOFF|ERROR_BOUND)[A-Z0-9_]*\s*:' crates`: 17 in `rcad-geom`, 3 in
`rcad-predicates`. One of the three, `surface.rs:77`, sits inside `#[cfg(test)]
mod tests` (`surface.rs:20-21`), so nineteen are outside tests, two of them in
`rcad-predicates` (464.12). The case for proving them grows once rcad turns
to speed. Every rewrite of an expression for speed (another order, an FMA, two
lanes at once) moves its rounding, so its derivation has to be done again, and
§16.3's test could not tell a bound wrong by a factor of two. A tighter bound
may also let the `f64` level decide more often, but how often each level runs
today has not been measured.

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
`ORIENT2D_ERROR_BOUND` and `surface.rs`' `ERROR_BOUND` (a test's, 464.12) in
`rcad-predicates`, and `SIDE_OF_PLANE`,
`SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT` (464.13), `SIDE_OF_SPHERE`,
`SIDE_OF_CYLINDER`, `SIDE_OF_CONE` and `SIDE_OF_TORUS` in `rcad-geom`. Block
1's pattern should carry over to them: one ratio hint per rounding, the error
split into one term per product of the permanent so that Gappa can average
them, block 2 for the closed normal end, and `-Eprecision=200`. That is Q2659.
The eleven `*_ROUNDOFF` constants were not read for this. Two of them,
`EVALUATION_ROUNDOFF` and `REFINEMENT_ROUNDOFF`, hold per de Boor pass at any
degree, and a Gappa script states one fixed expression, so it could only do
them one degree at a time. That is Q2660. The
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
  of its own. `surface.rs`' `ERROR_BOUND` is done in 464.12, and
  `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND` in 464.13.
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
  `surface.rs`' `ERROR_BOUND` in `rcad-predicates` (Q2677, done in 464.12),
  and `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND` (Q2678, done in
  464.13), `SIDE_OF_SPHERE_ERROR_BOUND`, `SIDE_OF_CYLINDER_ERROR_BOUND`,
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
asked for more bounds says why. With `adx` zero, and `second -/ M2`, `third -/
M3`, `q2 -/ (|cdxady| + |adxcdy|)` and `q3 -/ (|adxbdy| + |bdxady|)` added to
the goal as `in ?`, `REL(second, M2)`, `REL(third, M3)`, `REL(q2, |cdxady| +
|adxcdy|)` and `REL(q3, |adxbdy| + |bdxady|)` fail. Asked for `second` and
`M2` alone, it bounds each by `±2²⁰⁵⁰` and no closer. With `adz` zero, `rs`
gets only `±2.31901e+1217`. Each of these rounds a sum one of
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

### 464.12 The sphere of surface.rs' tests

Q2677 took up the `ERROR_BOUND` in `crates/rcad-predicates/src/surface.rs`,
in three lanes on 2026-10-24. The first ran every proof, control and `coqc`.
The second wrote every script again from its generator and ran each one
again, with the same results. The third wrote this up. The bound is proved at
the code's constant, on the sum before `value`'s last rounding and on `value`
too, in every case where `on_floats` hands `decide` a number, and Coq checks
each proof, modulo its hints. Nothing in `surface.rs` needs to change.

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, against
`surface.rs`, `point.rs` and `cascade.rs` at `3d60b0de`. `u` is `2⁻⁵³`, and
the code's `B = 8u + 64u²` is written `B = 8b-53 + 1b-100;`. Each script
follows 464.8's pattern: one ratio hint per rounding, the error split into one
term per square of the permanent so that Gappa averages them, 464.5's lemma
for each square `on_floats` checks is normal, and the step from `y`, the sum
before `value`'s last rounding, to the sign as 464.4 gives it.

**1. What the proof guards.** `HowFarOut` is a demo. It sits inside
`#[cfg(test)] mod tests` (`surface.rs:20-21`), and the `Sphere` it reads
stands in "for the quadrics `rcad-geom` will bring" (`surface.rs:28-30`). No
path a user reaches runs it, so the proof guards a test's predicate.
`rcad-geom`'s own sphere, `SIDE_OF_SPHERE_ERROR_BOUND` (`sphere.rs:425`), is a
row of its own, Q2679. 464.2 counted this constant among twenty outside tests.
It is the one of the twenty inside them, and 464.2 now says so.

**2. `on_floats`, line for line.** `HowFarOut::on_floats`
(`surface.rs:58-87`) takes the three gaps from the centre to the point. It
returns `None` unless `product_is_normal` holds for each gap with itself and
for the radius with itself (`surface.rs:65-66`). Then it squares the four and
returns `value = s0 + s1 + s2 - rr` (`surface.rs:84`) and `error_bound =
ERROR_BOUND * permanent`, with `permanent = s0 + s1 + s2 + rr`
(`surface.rs:81`). The code writes both sums out in full, but the same
additions in the same order round the same, so the permanent's first two sums
are `value`'s, and `sphere.g` writes `P = rnd(s012 + rr)`. That makes ten
roundings: the three gaps, the three squares, `rr`, `s01`, `s012` and
`value`. `y` takes the first nine.

Block 1 gives Gappa each product's sign as a hypothesis, such as `adz / |adz|
in [-1, 1]`. `sphere.g` needs none. Each term of its permanent is a square,
and Gappa proves a square and its rounding are never negative. `sqsign.g`,
whole, exits 0:

```gappa
@rnd = float<ieee_64, ne>;
x = rnd(x_);
s = rnd(x * x);
{ |x| <= 1b1024 /\ |x| >= 1b-1074 -> s in [0, 1b2100] /\ x * x in [0, 1b2100] }
```

**3. The bound on `y`.** `sphere.g` (9 below) proves `|y − Ex| ≤ eb` at the
code's `B`, printing `(y − Ex) / Phat` within `5u + 7u²` and some `u³`, and
`P −/ Phat` within `3u + 3u²` and some `u³`. `Phat` is the permanent with no
rounding of its own. The smallest `B` it proves, by bisection over `m` in `B
= 5u + m u²`, is `5u + 28u²`: `5u + 27u²` fails, best `[-1, 1]` as Gappa
prints it, rounded, and so does `5u`.

**4. The claim on `value`.** `spherev.g` makes the same claim on `value`, the
double `on_floats` returns. It proves at the code's `B`, printing `(value −
Ex) / Phat` within `6u + 5u²` and some `u³`. Its smallest `B`, by the same
bisection over `B = 6u + m u²`, is `6u + 30u²`: `6u + 29u²` fails, and so
does `6u`. So here `value` itself stays within `eb`, as plane's does (464.8's
3) and orient2d's does not (464.8's 1). `decide` does not need it to: the doc
on `Filtered::error_bound` (`cascade.rs:25-35`) asks only for the bound on
the sum before `value`'s last rounding, which 3 proves.

**5. What the comment claims.** `ERROR_BOUND` has no `///` doc comment. The
only prose on it is the plain `//` comment above it, `surface.rs:74-76`:
"Deliberately looser than the six roundings this actually takes. Too large a
bound only costs level 1 hits; too small a one costs correctness." Six is
what a gap's term goes through on its way to `value` (the gap, its square,
`s01`, `s012` and `value`) with the gap's rounding counted twice, since it is
squared. It agrees with the `6u` Gappa prints in 4. On `y` the same count is
five, and Gappa prints `5u`. The code's `8u + 64u²` is above both smallest
`B`s, `5u + 28u²` and `6u + 30u²`, so "deliberately looser" holds, and the
comment claims nothing that is not proved.

The comment on the check, `surface.rs:63-64`, says "A relative bound says
nothing about a product that fell into the subnormals, so this level declines
rather than certify one." That says what the level does, not that it would
certify a wrong sign without the check. 7 shows that the script needs the
check's hypotheses. Whether the code needs the check, a square below the
normals with the other three normal as 464.8's 4 ran for plane's products, was
not run. With the check in place `on_floats` never meets that case, and
`HowFarOut` is a test's demo, so it is not a row. `rcad-geom`'s sphere, Q2679,
is where that question has a user behind it.

**6. The zeros `product_is_normal` lets through.** 464.8's `o2s.g` splits on
a product that `on_floats` does not check. Here there is none, since
`on_floats` checks all four squares before it computes anything. What the
check lets through instead is a zero. `product_is_normal(a, b)` is true when
`a` or `b` is zero (`cascade.rs:67-69`), so a zero gap or a zero radius
reaches the sum with a zero square, which 464.5's lemma does not cover. A zero
gap is a shape designers make: the test's point `(2, 0, 0)` on the unit
sphere centred at the origin (`surface.rs:162`) has two.

`spherecases.py` (9 below) writes a script for each set of squares that can be
zero at once: `sphere_z<set>.g` on `y` and `spherev_z<set>.g` on `value`,
`<set>` naming which of the gaps `x`, `y`, `z` and the radius `r` are zero. A
zero gap's centre coordinate is the same double as the point's (464.11's
`diffzero.g`), and a zero radius is the double `0`. A zero, its square and a
sum of two zeros are left unrounded (464.11's `refl.g`). Where a zero square
is added to a double, a hint says the sum is that double, such as `sx + sy ->
sy;` with the x gap zero, and the rounding is then of a double. Without those
hints Gappa warned "Phat + (y - Ex) and S01 + (sz + ez) + (rr + er) are not
trivially equal": its check that a hint is an identity took the rounding of a
zero sum as a number of its own, as 464.11 found of `|x − x|`.

Each of the fifteen sets on `y`, and each of the fourteen on `value` with at
least one square left, proves at the code's `B`. They print `(y − Ex) /
Phat`, `(value − Ex) / Phat` and `P −/ Phat` within:

- one gap zero: `4.44089e-16`, `5.55112e-16` and `2.22045e-16`, which is
  `4u`, `5u` and `2u` to the digits Gappa prints.
- the radius zero: `5.55112e-16`, `5.55112e-16` and `2.22045e-16`.
- two gaps zero: `3.33067e-16`, `4.44089e-16` and `1b-53`.
- one gap and the radius zero: `4.44089e-16`, `4.44089e-16` and `1b-53`.
- the three gaps zero: `1.11022e-16`, `1.11022e-16` and `[0, 0]`.
- two gaps and the radius zero: `3.33067e-16`, `3.33067e-16` and `[0, 0]`.
- all four zero, `sphere_zxyzr.g`: "remaining results are pointless, anything
  can be proved." `eb` is then zero, so it is not normal, and `decide` does
  not compare. On `value` the case is the same one, since `value` is then `y`,
  which is zero.

**7. The controls.** A script whose hypotheses contradict each other proves
every goal (464.7's 3), so each of the thirty scripts above but
`sphere_zxyzr.g` was run again at `B = u / 2`, written `B = 1b-54;`, as
`c_<name>_half.g`. Each fails, exit 1, with a best that is what it printed in
6 over `u / 2`:

- on `y`: `sphere` `[-10, 10]`; one gap zero `[-8, 8]`; the radius zero
  `[-10, 10]`; two gaps zero `[-6, 6]`; one gap and the radius `[-8, 8]`; the
  three gaps `[-2, 2]`; two gaps and the radius `[-6, 6]`.
- on `value`: `spherev` `[-12, 12]`; one gap or the radius zero `[-10, 10]`;
  two gaps, or one and the radius, `[-8, 8]`; the three gaps `[-2, 2]`; two
  gaps and the radius `[-6, 6]`.

With 3 and 4's failures at `5u` and `6u`, that shows none of them proved from
a contradiction.

- `c_sphere_nonormal.g` is `sphere.g` with the four `product_is_normal`
  hypotheses dropped, and the four lemma lines they give. It fails, best
  `[-3.22221e+909, 3.22223e+909]`. Gappa finds neither `REL(P, Phat)` nor
  `BND((y - Ex) / Phat)`. `c_sphere_nonormal_why.g` asks for `rpx` and `rpr`
  as well, what rounding multiplied `gx * gx` and `r * r` by, and gets them
  only within `[0, 2^(1073)]` and `[0, 2^(4142)]`. So a script of this kind
  catches a missing check, as 464.4's 3 and 464.8's 4 did.
- `c_sphere_zxyzr_noeb.g` is `sphere_zxyzr.g` without `eb` normal. It fails,
  best `[-inf, inf]`, printing `eb in [0, 0]`. What proves `sphere_zxyzr.g` is
  the contradiction with `eb` normal and nothing else, as with 464.8's
  `o2tiny.g`, which is what it is meant to show.
- `c_sphere_noeb.g` is `sphere.g` without `eb` normal and without the lemma
  line it gives. It proves, as `c_plane_noeb.g` did. `c_sphere_noeb_why.g`
  prints `P` at least `1b-1020` and `eb` at least `1b-1070`.
  `c_sphere_noeb_rel.g`, whose only goal is `eb -/ B * P in ?`, fails on
  `REL(eb, B * P)`, so how Gappa gets past `eb`'s rounding was not traced. On
  paper, and not run: four squares each at least `2⁻¹⁰²²` put `P` at least
  `2⁻¹⁰²⁰` and `B · P` at least `2⁻¹⁰⁷⁰`. Rounding moves that by at most
  `2⁻¹⁰⁷⁵`, a thirty-second of it, and `5u + 7u²` of the permanent over
  thirty-one thirty-seconds of `8u` of it is about `0.65`. `decide` still
  checks that `eb` is normal, and that check is what keeps every intermediate
  finite.

464.4's argument for that carries over, on paper. Each gap is the factor of a
square, each square is a term of the permanent, and every term is
non-negative, so a term that rounds to infinity or is not a number makes `P`
so too. `s01` and `s012` are at most `P`, and `|value|` is at most `P`, by
monotone rounding. So where `decide` compares, nothing rounded to infinity.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each of the 31 scripts that prove: exit 0 each, `coqc` in 0.55 to 1.66 s in
the second lane's run. Gappa does not write the same proof each run (464.4),
so the sizes are the second lane's. `sphere.g`'s proof is 4096 lines with 18
hints (4057 lines in the first lane's run), `spherev.g`'s 5036 with 19, and
`sphere_zxyzr.g`'s 366 with 2. The 28 other zero cases are 1929 to 4820 lines,
with 9 to 19 hints. Each hint enters the Coq file as a hypothesis with no
condition attached, as in 464.4, so each proof stands on its hints and on what
Gappa took to be nonzero: `sphere.g`'s `gappa -Bcoq` prints 59 lines of the
form "the expression -(cx) + (px) has been assumed to be nonzero when checking
a rewriting rule", and `spherev.g`'s 66. It printed no other warning. `field`
has not been run on these hints (464.10).

**9. The scripts.** `spherecases.py` writes 64 scripts: `sphere.g`,
`spherev.g`, the 29 zero cases of 6, the 30 controls at `u / 2` and the three
other controls of 7. `why.py` writes the three `_why` and `_rel` controls of 7
from two of them. After the runs, the second lane ran both again, and they
wrote every file byte for byte as it was run. To run them again, save the nix
block of 464.5 as `gappalib18.nix` and these two files beside it, and then:

```sh
python3 spherecases.py
python3 why.py
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 sphere.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq sphere.g > sphere.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc sphere.v
```

and the same for every other file the two write. A script that proves exits
0; a control must exit 1. In the second lane's run, `nix shell`'s start
counted, Gappa took 0.1 to 6.2 s on each script that proves and 8 to 31 s on
each control.

The smallest `B`s of 3 and 4 come from `bisect.py`, run as `python3
bisect.py sphere.g 5 64` and `python3 bisect.py spherev.g 6 64`. Given a
script, `a` and a top `m`, it checks that the script fails at `B = a u` and
proves at `B = a u + m u²`, and then halves the range of `m` until the two
ends are one apart. It writes each `B` it tries as `<name>_b<a>_<m>.g` and
runs it with the first `gappa` line above. It is whole after `why.py` below.

`spherecases.py`, whole:

```python
"""Write sphere.g, HowFarOut::on_floats and its ERROR_BOUND line for line, and
its cases and controls beside it:

- spherev.g, the claim on value, the double on_floats returns, not on y;
- sphere_z<set>.g and spherev_z<set>.g for each set of the four squares that
  are zero, which product_is_normal lets through: <set> names which of the
  gaps x, y, z and the radius r are zero;
- c_<name>_half.g, each of those at B = u / 2, which must fail;
- c_sphere_nonormal.g, sphere.g without product_is_normal, and
  c_sphere_noeb.g, sphere.g without eb normal.

Run as: python3 spherecases.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

AXES = ("x", "y", "z")
TERMS = AXES + ("r",)
B_CODE = "B = 8b-53 + 1b-100;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"


def make(zero="", on_value=False, b=B_CODE, normal=True, eb_normal=True):
    gaps = [a for a in AXES if a not in zero]
    radius = "r" not in zero
    live = gaps + (["r"] if radius else [])
    out = "value" if on_value else "y"
    sq = {a: f"s{a}" for a in AXES}
    sq["r"] = "rr"

    src = [
        "# restates: crates/rcad-predicates/src/surface.rs on_floats at 3d60b0de",
        "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 3d60b0de",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 3d60b0de",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 3d60b0de",
        "# HowFarOut::on_floats and its ERROR_BOUND, the sphere of surface.rs' tests.",
    ]
    if on_value:
        src.append("# The claim is on value, the double on_floats returns, not on y.")
    if zero:
        names = [f"the {a} gap" for a in AXES if a in zero] + (["the radius"] if not radius else [])
        src.append(f"# The case where {' and '.join(names)} {'is' if len(names) == 1 else 'are'} zero.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre and the radius, each an f64. A gap is zero when",
        "# the centre's coordinate is the same double as the point's (464.11's",
        "# diffzero.g), and a zero radius is the double 0.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        " ".join(f"c{a} = p{a};" if a in zero else f"c{a} = rnd(c{a}_);" for a in AXES),
        "r = rnd(r_);" if radius else "r = 0;",
        "",
        "# What on_floats computes, line for line. A zero, a square of one and a",
        "# sum of two round to themselves (464.11's refl.g), so they are left",
        "# unrounded.",
        " ".join(f"G{a} = p{a} - c{a};" for a in AXES),
        " ".join(f"g{a} = G{a};" if a in zero else f"g{a} = rnd(G{a});" for a in AXES),
        " ".join(f"s{a} = g{a} * g{a};" if a in zero else f"s{a} = rnd(g{a} * g{a});" for a in AXES),
        "rr = rnd(r * r);" if radius else "rr = r * r;",
        "S01 = sx + sy;",
        "s01 = S01;" if "x" in zero and "y" in zero else "s01 = rnd(S01);",
        "S012 = s01 + sz;",
        "s012 = S012;" if not gaps else "s012 = rnd(S012);",
        "y = s012 - rr;",
        "value = rnd(y);",
        "",
        "# The permanent sums the same three squares in the same order as value",
        "# does, so its first two sums are s01 and s012.",
        "P = rnd(s012 + rr);",
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;",
        "",
        "# The permanent with no rounding of its own.",
        "Phat = sx + sy + sz + rr;",
    ]
    if live:
        src += ["", "# What each rounding multiplied its argument by."]
        if gaps:
            src.append(" ".join(f"rg{a} = g{a} / G{a};" for a in gaps))
            src.append(" ".join(f"rp{a} = s{a} / (g{a} * g{a});" for a in gaps))
        if radius:
            src.append("rpr = rr / (r * r);")
        rs = []
        if "x" in gaps or "y" in gaps:
            rs.append("rs01 = s01 / S01;")
        if gaps:
            rs.append("rs012 = s012 / S012;")
        if rs:
            src.append(" ".join(rs))
        if on_value:
            src.append("rv = value / y;")
        v = " * rv" if on_value else ""
        carry = {"x": "rs01 * rs012" + v, "y": "rs01 * rs012" + v, "z": "rs012" + v,
                 "r": "rv" if on_value else "1"}
        src += ["", f"# What each of the four terms, as {out} carries it, is off by: e is that",
                "# amount and W the same over the term's square. The radius's term is",
                "# subtracted, so its W is the other way round."]
        for a in gaps:
            src.append(f"W{a} = {carry[a]} - 1 / (rg{a} * rg{a} * rp{a});")
        if radius:
            src.append(f"Wr = 1 / rpr - {carry['r']};")
        src.append(" ".join(f"e{a} = {sq[a]} * W{a};" for a in live))

    hyps = []
    finite = [f"|p{a}| <= 1b1024" for a in AXES] + [f"|c{a}| <= 1b1024" for a in gaps]
    if radius:
        finite.append("|r| <= 1b1024")
    hyps.append(("Every input is a finite f64.", [finite[:3], finite[3:]]))
    if live:
        nonzero = [f"|g{a}| >= 1b-1074" for a in gaps] + (["|r| >= 1b-1074"] if radius else [])
        hyps.append(("No factor of the squares left is zero. For an f64 that is the same\n"
                     "  # as being at least the smallest subnormal.", [nonzero]))
        if normal:
            hyps.append(("product_is_normal on the squares left.",
                         [[f"|{sq[a]}| >= 1b-1022" for a in live]]))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = []
    if normal:
        lemma += [[f"s{a} -/ g{a} * g{a} {LEMMA}"] for a in gaps]
        if radius:
            lemma.append([f"rr -/ r * r {LEMMA}"])
    if eb_normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if on_value and live:
        hyps.append(("y is not zero. Where it is, value is zero too, and the claim on y is\n"
                     "  # the claim on value.", [["|y| >= 1b-1074"]]))

    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            row = [h for h in row]
            if not row:
                continue
            src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
            first = False
    src.append("->")
    if live:
        src += [f"  ({out} - Ex) / Phat in ? /\\ P -/ Phat in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    else:
        src.append(f"  eb in ? /\\ ({out} - Ex) / eb in [-1, 1]")
    src += ["}", ""]

    if live:
        src.append("# How far each rounding is from the identity.")
        for a in gaps:
            src.append(f"g{a} / G{a} -> 1 + (g{a} - G{a}) / G{a};")
        for a in gaps:
            src.append(f"s{a} / (g{a} * g{a}) -> 1 + (s{a} - g{a} * g{a}) / (g{a} * g{a});")
        if radius:
            src.append("rr / (r * r) -> 1 + (rr - r * r) / (r * r);")
        if "x" in gaps or "y" in gaps:
            src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
        if gaps:
            src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
        if on_value:
            src.append("value / y -> 1 + (value - y) / y;")
        split = " + ".join(f"({sq[a]} + e{a})" if a in live else sq[a] for a in TERMS)
        src += [
            "",
            "# The error is the sum of the terms' errors, each a fraction of its own",
            "# piece of the permanent; add_rr then averages them.",
            f"({out} - Ex) / Phat -> ((Phat + ({out} - Ex)) - Phat) / Phat;",
            f"Phat + ({out} - Ex) -> {split};",
        ]
        for a in live:
            src.append(f"({sq[a]} + e{a}) - {sq[a]} -> e{a};")
        src += [
            "",
            "# From the permanent to the one on_floats rounds, and on to eb.",
            f"({out} - Ex) / eb -> (({out} - Ex) / Phat) * (Phat / P) * (P / eb);",
            "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
            "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        ]
    zh = []
    if "x" in zero and "y" not in zero:
        zh.append("sx + sy -> sy;")
    if "y" in zero and "x" not in zero:
        zh.append("sx + sy -> sx;")
    if "z" in zero:
        zh.append("s01 + sz -> s01;")
    elif "x" in zero and "y" in zero:
        zh.append("s01 + sz -> sz;")
    if not radius:
        zh.append("s012 + rr -> s012;")
        if on_value:
            zh.append("s012 - rr -> s012;")
    elif not gaps:
        zh.append("s012 + rr -> rr;")
        if on_value:
            zh.append("s012 - rr -> -rr;")
    if zh:
        src += ["", "# A zero square added to a double leaves that double, which then rounds",
                "# to itself."] + zh
    return "\n".join(src) + "\n"


def write(name, text):
    open(name, "w").write(text)


for n in range(len(TERMS)):
    for zero in itertools.combinations(TERMS, n):
        zero = "".join(zero)
        tag = f"_z{zero}" if zero else ""
        for on_value in (False, True):
            name = f"sphere{'v' if on_value else ''}{tag}"
            write(f"{name}.g", make(zero, on_value))
            write(f"c_{name}_half.g", make(zero, on_value, b=B_HALF))
# Every square zero: then eb is zero, so decide does not compare, and Gappa
# proves the goal from the contradiction. Its control drops eb normal.
write("sphere_zxyzr.g", make("xyzr"))
write("c_sphere_zxyzr_noeb.g", make("xyzr", eb_normal=False))
write("c_sphere_nonormal.g", make(normal=False))
write("c_sphere_noeb.g", make(eb_normal=False))
```

`sphere.g`, whole, as `spherecases.py` writes it. `on_floats` holds
`ERROR_BOUND` as a `const` inside its body, so the line on `on_floats` covers
it too:

```gappa
# restates: crates/rcad-predicates/src/surface.rs on_floats at 3d60b0de
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 3d60b0de
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 3d60b0de
# restates: crates/rcad-predicates/src/cascade.rs decide at 3d60b0de
# HowFarOut::on_floats and its ERROR_BOUND, the sphere of surface.rs' tests.
@rnd = float<ieee_64, ne>;

# The point, the centre and the radius, each an f64. A gap is zero when
# the centre's coordinate is the same double as the point's (464.11's
# diffzero.g), and a zero radius is the double 0.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
cx = rnd(cx_); cy = rnd(cy_); cz = rnd(cz_);
r = rnd(r_);

# What on_floats computes, line for line. A zero, a square of one and a
# sum of two round to themselves (464.11's refl.g), so they are left
# unrounded.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = rnd(Gx); gy = rnd(Gy); gz = rnd(Gz);
sx = rnd(gx * gx); sy = rnd(gy * gy); sz = rnd(gz * gz);
rr = rnd(r * r);
S01 = sx + sy;
s01 = rnd(S01);
S012 = s01 + sz;
s012 = rnd(S012);
y = s012 - rr;
value = rnd(y);

# The permanent sums the same three squares in the same order as value
# does, so its first two sums are s01 and s012.
P = rnd(s012 + rr);

B = 8b-53 + 1b-100;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;

# The permanent with no rounding of its own.
Phat = sx + sy + sz + rr;

# What each rounding multiplied its argument by.
rgx = gx / Gx; rgy = gy / Gy; rgz = gz / Gz;
rpx = sx / (gx * gx); rpy = sy / (gy * gy); rpz = sz / (gz * gz);
rpr = rr / (r * r);
rs01 = s01 / S01; rs012 = s012 / S012;

# What each of the four terms, as y carries it, is off by: e is that
# amount and W the same over the term's square. The radius's term is
# subtracted, so its W is the other way round.
Wx = rs01 * rs012 - 1 / (rgx * rgx * rpx);
Wy = rs01 * rs012 - 1 / (rgy * rgy * rpy);
Wz = rs012 - 1 / (rgz * rgz * rpz);
Wr = 1 / rpr - 1;
ex = sx * Wx; ey = sy * Wy; ez = sz * Wz; er = rr * Wr;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024 /\ |cz| <= 1b1024 /\ |r| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gx| >= 1b-1074 /\ |gy| >= 1b-1074 /\ |gz| >= 1b-1074 /\ |r| >= 1b-1074
  # product_is_normal on the squares left.
  /\ |sx| >= 1b-1022 /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022 /\ |rr| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ sx -/ gx * gx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ gy * gy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ gz * gz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
->
  (y - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
gx / Gx -> 1 + (gx - Gx) / Gx;
gy / Gy -> 1 + (gy - Gy) / Gy;
gz / Gz -> 1 + (gz - Gz) / Gz;
sx / (gx * gx) -> 1 + (sx - gx * gx) / (gx * gx);
sy / (gy * gy) -> 1 + (sy - gy * gy) / (gy * gy);
sz / (gz * gz) -> 1 + (sz - gz * gz) / (gz * gz);
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The error is the sum of the terms' errors, each a fraction of its own
# piece of the permanent; add_rr then averages them.
(y - Ex) / Phat -> ((Phat + (y - Ex)) - Phat) / Phat;
Phat + (y - Ex) -> (sx + ex) + (sy + ey) + (sz + ez) + (rr + er);
(sx + ex) - sx -> ex;
(sy + ey) - sy -> ey;
(sz + ez) - sz -> ez;
(rr + er) - rr -> er;

# From the permanent to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

`why.py`, whole:

```python
"""Write the two controls again with more goals, to name where each breaks or
how it gets through: c_sphere_nonormal_why.g asks for what the rounding of the
x square and of the radius's square multiplied them by; c_sphere_noeb_why.g
asks for P and eb, and c_sphere_noeb_rel.g for eb's relative error alone."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
GOAL = "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n"


def rewrite(src, dst, goal):
    text = open(src).read()
    assert text.count(GOAL) == 1, src
    open(dst, "w").write(text.replace(GOAL, goal))


rewrite("c_sphere_nonormal.g", "c_sphere_nonormal_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ rpx in ? /\\ rpr in ?\n")
rewrite("c_sphere_noeb.g", "c_sphere_noeb_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ P in ? /\\ eb in ?\n")
text = open("c_sphere_noeb.g").read()
head, rest = text.split("->\n", 1)
_, hints = rest.split("}\n", 1)
open("c_sphere_noeb_rel.g", "w").write(head + "->\n  eb -/ B * P in ?\n}\n" + hints)
```

`bisect.py`, whole. Its `best` line is never read; the line after it is what
prints a failing run's best:

```python
"""Find the smallest B = a u + m u^2 a script proves at, in steps of u^2, by
bisection on m, as 464.8 did. Run as: python3 bisect.py <file.g> <a> <hi m>.
Writes <name>_b<a>_<m>.g beside itself for each B tried, and prints each run's
exit and the goal's best bound when it fails."""
import os
import re
import subprocess
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
B_CODE = "B = 8b-53 + 1b-100;"
NIX = ["nix", "--extra-experimental-features", "nix-command",
       "--extra-experimental-features", "flakes", "shell",
       "github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe#gappa",
       "--command", "gappa", "-Eprecision=200"]

name, a, hi = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
text = open(name).read()
assert text.count(B_CODE) == 1, f"no single B line in {name}"


def proves(m):
    b = f"B = {a}b-53 + {m}b-106;" if m else f"B = {a}b-53;"
    out_name = f"{name[:-2]}_b{a}_{m}.g"
    open(out_name, "w").write(text.replace(B_CODE, b))
    run = subprocess.run(NIX + [out_name], capture_output=True, text=True, timeout=200)
    best = re.findall(r"best: \[[^\]]*\]|\{[-0-9.e+]+, [-0-9.e+^()]+\}\]", run.stderr + run.stdout)
    fail = [l for l in (run.stderr + run.stdout).splitlines() if "best" in l or "failed" in l.lower()]
    print(f"{out_name}: {b} exit {run.returncode}; {' | '.join(fail[:3])}", flush=True)
    return run.returncode == 0


lo = 0
assert not proves(lo), "proves with no u^2 term"
assert proves(hi), "does not prove at the top of the range"
while hi - lo > 1:
    mid = (lo + hi) // 2
    if proves(mid):
        hi = mid
    else:
        lo = mid
print(f"smallest: {a}u + {hi}u^2 proves, {a}u + {lo}u^2 fails")
```

**10. What this leaves.** No row.

- Q2700 already waits on Q2677, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- A square below the normals with the other three normal (5) was not run.
  `on_floats` never meets it while its check stands, so it is not a row here.
- How `c_sphere_noeb.g` gets past `eb`'s rounding (7) was not traced, as with
  plane's.
- `field` on these scripts' hints (464.10).

### 464.13 The side of a flat surface through a point

Q2678 took up `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND` in
`crates/rcad-geom/src/plane.rs`, in three lanes on 2026-10-25. The first wrote
the generator and ran `flat.g`. The second ran every proof, control, bisection
and `coqc`, and wrote every script again from its generators, with the same
bytes. The third wrote this up. The bound is proved at the code's constant, on
the sum before `value`'s last rounding and on `value` too, in every case where
`on_floats` hands `decide` a number: every product normal, any of them zero,
and any of the sums cancelling to zero. Coq checks each proof, modulo its
hints. The doc comment on the constant miscounts its own terms, and says the
bound needs every product normal, when `on_floats` lets zero products through
and the bound holds for them. That comment is in `plane.rs`, and fixing it is
a row (10).

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, against
`plane.rs`, `point.rs` and `cascade.rs` at `8c943dfd`. `u` is `2⁻⁵³`, and the
code's `B = 6u + 128u²` is written `B = 6b-53 + 1b-99;`. Each script follows
464.8's pattern: one ratio hint per rounding, the error split into one term
per product of the permanent so that Gappa averages them, 464.5's lemma for
each product `on_floats` checks is normal, and the step from `y`, the sum
before `value`'s last rounding, to the sign as 464.4 gives it.

**1. What the proof guards.** `which_side_of_a_flat_surface_through`
(`plane.rs:945-961`) runs the cascade on `SideOfAFlatSurfaceThroughAPoint`.
Its one caller is `HalfSpaceThroughAPoint::side_of_point`
(`half_space.rs:668-675`), and `lib.rs:101` exports `HalfSpaceThroughAPoint`.
So unlike 464.12's, this proof guards a predicate a user reaches.

**2. `on_floats`, line for line.**
`SideOfAFlatSurfaceThroughAPoint::on_floats` (`plane.rs:976-998`) returns
`None` unless `product_is_normal` holds for each of the facing's coordinates
with the point's and with the through point's (`plane.rs:980-985`). Then it
takes six products: the facing times the point on each axis, and the facing
times the through point on each axis, negated (`plane.rs:987-990`). It folds
them from `(0.0, 0.0)`, adding each product to `value` and its magnitude to
`permanent` (`plane.rs:991-993`), and returns `error_bound =
SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND * permanent`
(`plane.rs:997`).

Negating a product is exact, so adding it negated is subtracting it, and
`flat.g` writes it that way. The fold's first addition is `0.0` plus the first
product, which gives the product back (`addzero.g` below), so five of the
fold's six additions round. `flat.g` names the products `t0` to `t5`, the
rounded sums `s1` to `s4`, `y = s4 − t5` and `value = rnd(y)`. The first
product's term goes through six roundings on its way to `value`, its own and
five sums, and through five on its way to `y`. The permanent folds the
products' magnitudes in the same order, its first addition exact too, so `P`
is five nested roundings, and `eb = rnd(B * P)`. The constant
(`plane.rs:1054-1057`) is `(6 + 128ε) ε` with `ε = f64::EPSILON / 2`, which is
`u`. That is `6u + 2⁻⁹⁹`, which a double holds exactly, and so does each step
the Rust takes to it.

`addzero.g`, whole, exits 0, and its `gappa -Bcoq` and `coqc` exit 0, a proof
of 309 lines with 3 hints. Each hint reads a sum or difference with `z = 0` as
`x`, which is true of real numbers. Written as `rnd(0 + x) - x in [0, 0]` with
no hints, Gappa ran 151 s without an answer and was stopped; `a = rnd(x + z)`
with `z = 0` ran past 30 s the same way.

```gappa
# addzero.g: adding the double 0 to a double, on either side, or taking 0
# from it, gives that double back. That is the fold's first addition, 0.0
# plus the first product, and each zero product the fold adds.
@rnd = float<ieee_64, ne>;
x = rnd(x_);
z = 0;
L = z + x; R = x + z; D = x - z;
{ |x| <= 1b1024 -> rnd(L) - x in [0, 0] /\ rnd(R) - x in [0, 0] /\ rnd(D) - x in [0, 0] }
# Without these three, Gappa ran rnd(x + z) for 30 s and more and did not stop.
L -> x;
R -> x;
D -> x;
```

**3. The bound on `y`.** `flat.g` (9 below) proves `|y − Ex| ≤ eb` at the
code's `B`, in 5.02 s in the first lane's run and 18.43 s in the second's, on
a loaded machine. It prints `(y − Ex) / Phat` within `5u + 7u²` and `P −/
Phat` within `5u + 10u²`, each with a little more, under `10⁻⁴ u²`, read
exactly off the fraction Gappa prints. `Phat` is the permanent with no
rounding of its own. The smallest `B` it proves, by bisection over `m` in `B =
5u + m u²`, is `5u + 38u²`: `5u + 37u²` fails, best `[-1, 1]` as Gappa prints
it, rounded, and so does `5u`.

**4. The claim on `value`.** `flatv.g` makes the same claim on `value`, the
double `on_floats` returns. It proves at the code's `B`, printing `(value −
Ex) / Phat` within `6u + 11u²`, read the same way. Its smallest `B`, by the
same bisection over `B = 6u + m u²`, is `6u + 48u²`: `6u + 47u²` fails, and so
does `6u`. So here `value` itself stays within `eb`, as plane's does (464.8's
3) and the sphere's (464.12's 4). `decide` does not need it to: the doc on
`Filtered::error_bound` (`cascade.rs:25-35`) asks only for the bound on the
sum before `value`'s last rounding, which 3 proves.

**5. What the comment claims.** The doc comment on the constant,
`plane.rs:1045-1053`:

> The longest chain of roundings from an input to `value` is one product and
> then the five additions of the fold, six in all, so the first-order term is
> `6ε` times the permanent and the higher-order terms of `(1 + ε)⁶ − 1` are
> `15ε²` and less. The permanent is itself folded from six rounded numbers in
> five additions, so the computed one can sit up to about `6ε` under the true
> one, which costs another `36ε²` against the `6ε`, and multiplying by this
> constant rounds once more. The `128ε²` covers those 52 and change with room.
> The same relative argument as [`SIDE_OF_PLANE_ERROR_BOUND`], so it holds only
> while every product is a normal number, which `on_floats` checks.

- "Five additions of the fold, six in all": the fold makes six additions, but
  the first is exact (2), so five round, and six roundings to `value` agrees
  with the `6u` of 4. On `y` the same count is five, and 3 prints `5u`.
- `15ε²` is the `ε²` term of `(1 + ε)⁶ − 1`, six choose two.
- `36ε²` is the permanent's `6ε` taken against the `6ε` of the bound.
- The last rounding, of `B · P`, costs `ε` of `eb`, which is `6ε²` of the
  permanent. So by the comment's own terms the count is `15 + 36 + 6 = 57`,
  not 52. `128` is above 57, and above both smallest `B`s, `5u + 38u²` and
  `6u + 48u²`, so the slip costs nothing, but the sentence gives the wrong
  number.
- "It holds only while every product is a normal number, which `on_floats`
  checks." `on_floats` does not check that. `product_is_normal(a, b)` is true
  when `a` or `b` is zero (`cascade.rs:67-69`), so a zero product gets
  through, and zero is not a normal number. The bound holds for those: 6
  proves it at the code's `B` for every set of zero products. "Only" also
  says that a product below the normals breaks the bound. That was not run
  here. 464.8's 4 found the same sentence false for one product of
  `SideOfAFlatSurface`, because `eb` normal keeps a product below the normals
  small against `eb`. Whether it is false here too is a row (10).

**6. The zeros and the cancelling sums.** 464.8's `o2s.g` splits on a product
that `on_floats` does not check. Here there is none, since `on_floats` checks
all six before it computes anything. What the check lets through instead is a
zero product, and a rounded sum can also cancel to zero. Both come from shapes
designers make. The floor through the origin, facing `(0, 0, 1)`, has the
facing's `x` and `y` zero, which makes four of the six products zero, and the
through point's `z` zero makes a fifth. A facing of `(1, 1, 0)` with the point
`(1, −1, 0)` makes the first two products `1` and `−1`, and the first rounded
sum zero.

`flatcases.py` (9 below) writes a script for each case: `flat_z<set>.g` on `y`
and `flatv_z<set>.g` on `value` for each set of products that are zero,
`<set>` naming them (`0` to `2` the facing times the point, `3` to `5` the
facing times the through point), and each of those again as `_c<set>` for each
set of rounded sums that cancel, `<set>` naming the product each such sum
adds. Where the facing's coordinate is the zero, both its products are zero,
and the script with the point's and the through point's coordinates zero
computes the same numbers. A zero product is left out of both folds, since
adding it gives the sum back (`addzero.g`). Each rounded sum is a sum of two
doubles, so it is either zero or at least `2⁻¹⁰⁷⁴`: the script for the first
writes it unrounded, `s = S`, since zero rounds to itself, and the script for
the second assumes `|S| ≥ 2⁻¹⁰⁷⁴` and rounds it. Two cancelling sums are never
next to each other, since the sum after a cancelled one is one product, which
is not zero. That makes 145 scripts on `y` and 138 on `value`, `flat.g` and
`flatv.g` among them. With one product left there is no script on `value`,
since `value` is then `y`.

Each of the 283 proves at the code's `B`. With `n` products left and `c` sums
cancelling, Gappa prints `(y − Ex) / Phat` within `(n − 1 − c) u`, `(value −
Ex) / Phat` within `(n − c) u` and `P −/ Phat` within `(n − 1) u`, each to the
digits it prints:

- six products: on `y` `5u` with no sum cancelling (`flat.g`), `4u` with one
  (4 scripts) and `3u` with two (3); on `value` `6u`, `5u` and `4u`; `P −/
  Phat` `5u`.
- five products, 6 sets: on `y` `4u`, `3u` with one sum cancelling (18
  scripts) and `2u` with two (6); on `value` `5u`, `4u` and `3u`; `P −/ Phat`
  `4u`.
- four products, 15 sets: on `y` `3u`, and `2u` with one sum cancelling (30
  scripts); on `value` `4u` and `3u`; `P −/ Phat` `3u`.
- three products, 20 sets: on `y` `2u`, and `u` with one sum cancelling (20
  scripts); on `value` `3u` and `2u`; `P −/ Phat` `2u`.
- two products, 15 sets: `u` on `y`, `2u` on `value`, `u` for `P −/ Phat`.
- one product, 6 sets, such as `flat_z01234.g`: `(−t5 − Ex) / |t5|` within
  `u`, and `|t5| −/ |t5|` in `[0, 0]`.
- none, `flat_z012345.g`: "remaining results are pointless, anything can be
  proved." `eb` is then zero, so it is not normal, and `decide` does not
  compare.

464.12's zero squares were written into its sums, and each needed a hint
saying the sum was the other term. Here no zero product is written into a
fold. It stays only in `Ex`, as its factors, such as `f0 * p0` with `p0 = 0;`,
and Gappa's check that each hint is an identity took that with no warning. So
no script needs a hint for a zero. No print warned of anything but divisors
taken to be nonzero (8).

**7. The controls.** A script whose hypotheses contradict each other proves
every goal (464.7's 3), so each of the 283 scripts above but `flat_z012345.g`
was run again at `B = u / 2`, written `B = 1b-54;`, as `c_<name>_half.g`. Each
of the 282 fails, exit 1, with a best that is what it printed in 6 over `u /
2`:

- on `y`: six products `[-10, 10]`, `[-8, 8]` with one sum cancelling and
  `[-6, 6]` with two; five `[-8, 8]`, `[-6, 6]` and `[-4, 4]`; four `[-6, 6]`
  and `[-4, 4]`; three `[-4, 4]` and `[-2, 2]`; two and one `[-2, 2]`.
- on `value`: six products `[-12, 12]`, `[-10, 10]` and `[-8, 8]`; five
  `[-10, 10]`, `[-8, 8]` and `[-6, 6]`; four `[-8, 8]` and `[-6, 6]`; three
  `[-6, 6]` and `[-4, 4]`; two `[-4, 4]`.

With 3 and 4's failures at `5u` and `6u`, that shows none of them proved from
a contradiction.

- `c_flat_nonormal.g` is `flat.g` with the six `product_is_normal` hypotheses
  dropped, and the six lemma lines they give. It fails, best `[-1.93499e+909,
  1.93499e+909]`. Gappa finds neither `REL(P, Phat)` nor `BND((y - Ex) /
  Phat)`. `c_flat_nonormal_why.g` asks for `rp0` as well, what rounding
  multiplied `f0 * p0` by, and gets it only within `±2^(4142)`. So a script of
  this kind catches a missing check, as 464.4's 3, 464.8's 4 and 464.12's 7
  did.
- `c_flat_z012345_noeb.g` is `flat_z012345.g` without `eb` normal. It fails,
  best `[-inf, inf]`, printing `eb in [0, 0]`. What proves `flat_z012345.g` is
  the contradiction with `eb` normal and nothing else, which is what it is
  meant to show.
- `c_flat_noeb.g` is `flat.g` without `eb` normal and without the lemma line
  it gives. It proves, as `c_plane_noeb.g` and `c_sphere_noeb.g` did.
  `c_flat_noeb_why.g` prints `P` at least `3b-1021`, which is `6 · 2⁻¹⁰²²`,
  and `eb` at least `9b-1073`, which is `36 · 2⁻¹⁰⁷⁵`. `c_flat_noeb_rel.g`,
  whose only goal is `eb -/ B * P in ?`, fails on `REL(eb, B * P)`, so how
  Gappa gets past `eb`'s rounding was not traced. On paper, and not run: six
  products each at least `2⁻¹⁰²²` put `P` at least `6 · 2⁻¹⁰²²` and `B · P`
  at least `36 · 2⁻¹⁰⁷⁵`. Rounding moves that by at most `2⁻¹⁰⁷⁵`, a
  thirty-sixth of it, and `5u + 7u²` of the permanent over thirty-five
  thirty-sixths of `6u` of it is about `0.86`. `decide` still checks that
  `eb` is normal, and that check is what keeps every intermediate finite.

464.4's argument for that carries over, on paper. Each product's magnitude is
a term of the permanent, and every term is non-negative, so a product that
rounds to infinity or is not a number makes `P` so too. Each of `value`'s
partial sums is at most the permanent's partial sum in magnitude, by monotone
rounding, so `|value|` is at most `P`. So where `decide` compares, nothing
rounded to infinity.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each of the 283 scripts that prove: exit 0 each, the two together in 1.3 to
41.07 s in the second lane's run, on a load of 25 to 50 over 24 cores. Gappa
does not write the same proof each run (464.4), so the sizes are the second
lane's. `flat.g`'s proof is 4926 lines with 27 hints, `flatv.g`'s 5198 with
28, and `flat_z012345.g`'s 177 with none. The others are 177 to 5829 lines,
the largest `flatv_c24.v` with 26 hints. Each hint enters the Coq file as a
hypothesis with no condition attached, as in 464.4, so each proof stands on
its hints and on what Gappa took to be nonzero: `flat.g`'s `gappa -Bcoq`
prints 112 lines of the form "the expression (f0) * (p0) has been assumed to
be nonzero when checking a rewriting rule", and `flatv.g`'s 127. It printed
no other warning. `field` has not been run on these hints (464.10).

**9. The scripts.** `flatcases.py` writes 568 scripts: `flat.g`, `flatv.g`,
the 280 other cases of 6 and `flat_z012345.g`, the 282 controls at `u / 2` and
the three other controls of 7. `why.py` writes the three `_why` and `_rel`
controls of 7 from two of them. After the runs, the second lane ran both again
in a fresh directory, and they wrote all 571 files byte for byte as they were
run. To run them again, save the nix block of 464.5 as `gappalib18.nix` and
these two files beside it, and then:

```sh
python3 flatcases.py
python3 why.py
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 flat.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq flat.g > flat.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc flat.v
```

and the same for every other file the two write, and for `addzero.g`. A
script that proves exits 0; a control must exit 1. What the lanes ran is
`one.sh` (below), the same three steps with the store paths written in, as
`bash one.sh flat.g coq`. In the second lane's run Gappa took 0.02 to 23.58 s
on each script that proves and 0.04 to 131.32 s on each control.

The smallest `B`s of 3 and 4 come from 464.12's `bisect.py`, with its `B_CODE`
line the code's `B` here and its `NIX` the `gappa` store path, run as `python3
bisect.py flat.g 5 128` and `python3 bisect.py flatv.g 6 128`. It is whole
after `why.py` below.

`flatcases.py`, whole:

```python
"""Write flat.g, SideOfAFlatSurfaceThroughAPoint::on_floats and
SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND line for line, and its cases
and controls beside it:

- flatv.g, the claim on value, the double on_floats returns, not on y;
- flat_z<set>.g and flatv_z<set>.g for each set of the six products that are
  zero, which product_is_normal lets through: <set> names the products, 0 to 2
  the facing times the point and 3 to 5 the facing times the through point;
- each of those again with _c<set> for each set of its rounded sums that may
  cancel to zero, <set> naming the product each such sum adds;
- c_<name>_half.g, each of those at B = u / 2, which must fail;
- c_flat_nonormal.g, flat.g without product_is_normal, c_flat_noeb.g, flat.g
  without eb normal, and c_flat_z012345_noeb.g, every product zero without eb
  normal.

Run as: python3 flatcases.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

TERMS = range(6)
SIGN = "+++---"
FACTORS = [("f0", "p0"), ("f1", "p1"), ("f2", "p2"), ("f0", "q0"), ("f1", "q1"), ("f2", "q2")]
B_CODE = "B = 6b-53 + 1b-99;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"


def product(k):
    f, x = FACTORS[k]
    return f"{f} * {x}"


def make(zero=(), cancel=(), on_value=False, b=B_CODE, normal=True, eb_normal=True):
    live = [k for k in TERMS if k not in zero]
    out = "value" if on_value else "y"
    used = sorted({name for k in live for name in FACTORS[k]})

    src = [
        "# restates: crates/rcad-geom/src/plane.rs on_floats at 8c943dfd",
        "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at 8c943dfd",
        "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 8c943dfd",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 8c943dfd",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 8c943dfd",
        "# SideOfAFlatSurfaceThroughAPoint::on_floats and its error bound: the sign",
        "# of facing . point - facing . through.",
    ]
    if on_value:
        src.append("# The claim is on value, the double on_floats returns, not on y.")
    if zero:
        src.append(f"# The case where product{'s' if len(zero) > 1 else ''} "
                   f"{', '.join(str(k) for k in zero)} {'are' if len(zero) > 1 else 'is'} zero.")
    if cancel:
        src.append(f"# The sum{'s' if len(cancel) > 1 else ''} adding product"
                   f"{'s' if len(cancel) > 1 else ''} {', '.join(str(k) for k in cancel)} "
                   f"may cancel to zero, which rounds to itself.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The facing, the point and the through point, each an f64. A product that",
        "# is zero has the double 0 for its point's or its through point's",
        "# coordinate; where the facing's coordinate is the zero, both its products",
        "# are zero, and every number below is the same as with both of those",
        "# coordinates zero.",
        "f0 = rnd(f0_); f1 = rnd(f1_); f2 = rnd(f2_);",
        " ".join(f"p{i} = 0;" if i in zero else f"p{i} = rnd(p{i}_);" for i in range(3)),
        " ".join(f"q{i} = 0;" if i + 3 in zero else f"q{i} = rnd(q{i}_);" for i in range(3)),
        "",
        "# What on_floats computes, line for line. The fold starts at 0.0, and",
        "# adding 0.0 or a zero product to a double gives that double back",
        "# (addzero.g), so the zero products and the first addition are left out.",
    ]
    if live:
        src.append(" ".join(f"t{k} = rnd({product(k)});" for k in live))

    rounded = []
    if not live:
        src.append("y = 0;")
    else:
        first = live[0]
        running = f"t{first}" if SIGN[first] == "+" else f"-t{first}"
        for k in live[1:-1]:
            src.append(f"S{k} = {running} {SIGN[k]} t{k};")
            if k in cancel:
                src.append(f"s{k} = S{k};")
            else:
                src.append(f"s{k} = rnd(S{k});")
                rounded.append(k)
            running = f"s{k}"
        if len(live) == 1:
            src.append(f"y = {running};")
        else:
            src.append(f"y = {running} {SIGN[live[-1]]} t{live[-1]};")
    value_rounded = len(live) >= 2
    src.append("value = rnd(y);" if value_rounded else "value = y;")

    src += ["", "# The permanent folds the same products' magnitudes in the same order."]
    if not live:
        src.append("P = 0;")
    else:
        perm = f"|t{live[0]}|"
        for k in live[1:]:
            perm = f"rnd({perm} + |t{k}|)"
        src.append(f"P = {perm};")
    src += ["", b, "eb = rnd(B * P);", "", "# The true value of the same inputs."]
    src.append("Ex = " + " + ".join(product(k) for k in (0, 1, 2)) + " - "
               + " - ".join(product(k) for k in (3, 4, 5)) + ";")
    src += ["", "# The permanent with no rounding of its own."]
    src.append("Phat = " + (" + ".join(f"|t{k}|" for k in live) if live else "0") + ";")

    if live:
        src += ["", "# What each rounding multiplied its argument by."]
        src.append(" ".join(f"rp{k} = t{k} / ({product(k)});" for k in live))
        if rounded:
            src.append(" ".join(f"rs{k} = s{k} / S{k};" for k in rounded))
        if on_value and value_rounded:
            src.append("rv = value / y;")
        src += ["", f"# Each product's term, as {out} carries it, less the term itself: e is",
                "# that amount and W the same over the product as on_floats rounds it.",
                "# The through point's terms are subtracted, so their e is the other way",
                "# round."]
        for k in live:
            carry = [f"rs{j}" for j in rounded if j >= k]
            if on_value and value_rounded:
                carry.append("rv")
            src.append(f"W{k} = {' * '.join(carry) if carry else '1'} - 1 / rp{k};")
        src.append(" ".join(f"e{k} = t{k} * W{k};" if SIGN[k] == "+" else f"e{k} = -(t{k} * W{k});"
                            for k in live))
        src.append(" ".join(f"c{k} = |t{k}|;" for k in live))

    hyps = []
    finite = ["|f0| <= 1b1024", "|f1| <= 1b1024", "|f2| <= 1b1024"]
    coords = [f"|p{i}| <= 1b1024" for i in range(3) if i not in zero] + \
             [f"|q{i}| <= 1b1024" for i in range(3) if i + 3 not in zero]
    hyps.append(("Every input is a finite f64.", [finite, coords]))
    if live:
        hyps.append(("No factor of a product left is zero. For an f64 that is the same as\n"
                     "  # being at least the smallest subnormal.",
                     [[f"|{n}| >= 1b-1074" for n in used if n[0] == "f"],
                      [f"|{n}| >= 1b-1074" for n in used if n[0] != "f"]]))
        if rounded:
            hyps.append(("No rounded sum cancels to zero.", [[f"|S{k}| >= 1b-1074" for k in rounded]]))
        if normal:
            hyps.append(("product_is_normal on the products left.",
                         [[f"|t{k}| >= 1b-1022" for k in live]]))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = []
    if normal:
        lemma += [[f"t{k} -/ {product(k)} {LEMMA}"] for k in live]
    if eb_normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if live:
        hyps.append(("x / |x| is in [-1, 1] for any x that is not zero.",
                     [[f"t{k} / |t{k}| in [-1, 1]" for k in live]]))
    if on_value and value_rounded:
        hyps.append(("y is not zero. Where it is, value is zero too, and the claim on y is\n"
                     "  # the claim on value.", [["|y| >= 1b-1074"]]))

    src += ["", "{"]
    first_line = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            if not row:
                continue
            src.append(("     " if first_line else "  /\\ ") + " /\\ ".join(row))
            first_line = False
    src.append("->")
    if live:
        src += [f"  ({out} - Ex) / Phat in ? /\\ P -/ Phat in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    else:
        src.append(f"  eb in ? /\\ ({out} - Ex) / eb in [-1, 1]")
    src += ["}", ""]

    if live:
        src.append("# How far each rounding is from the identity.")
        for k in live:
            src.append(f"t{k} / ({product(k)}) -> 1 + (t{k} - {product(k)}) / ({product(k)});")
        for k in rounded:
            src.append(f"s{k} / S{k} -> 1 + (s{k} - S{k}) / S{k};")
        if on_value and value_rounded:
            src.append("value / y -> 1 + (value - y) / y;")
        src += [
            "",
            "# The error is the sum of the products' errors, each a fraction of its",
            "# own piece of the permanent; add_rr then averages them.",
            f"({out} - Ex) / Phat -> ((Phat + ({out} - Ex)) - Phat) / Phat;",
            f"Phat + ({out} - Ex) -> " + " + ".join(f"(c{k} + e{k})" for k in live) + ";",
        ]
        for k in live:
            src.append(f"(c{k} + e{k}) - c{k} -> e{k};")
        for k in live:
            src.append(f"e{k} / c{k} -> (t{k} / |t{k}|) * W{k};" if SIGN[k] == "+"
                       else f"e{k} / c{k} -> -((t{k} / |t{k}|) * W{k});")
        src += [
            "",
            "# From the permanent to the one on_floats rounds, and on to eb.",
            f"({out} - Ex) / eb -> (({out} - Ex) / Phat) * (Phat / P) * (P / eb);",
            "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
            "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        ]
    return "\n".join(src) + "\n"


def cancellable(live):
    """The sets of rounded sums that can cancel to zero together. A sum that
    cancels leaves the running sum zero, so the next rounded sum is the next
    product alone, which is not zero."""
    sums = live[1:-1]
    for n in range(len(sums) + 1):
        for c in itertools.combinations(sums, n):
            if all(sums.index(b) - sums.index(a) > 1 for a, b in zip(c, c[1:])):
                yield c


def write(name, text):
    open(name, "w").write(text)


def name_of(prefix, zero, cancel):
    tag = (f"_z{''.join(map(str, zero))}" if zero else "") + \
          (f"_c{''.join(map(str, cancel))}" if cancel else "")
    return prefix + tag


if __name__ == "__main__":
    proving = []
    for n in range(6):
        for zero in itertools.combinations(TERMS, n):
            live = [k for k in TERMS if k not in zero]
            for cancel in cancellable(live):
                for on_value in (False, True):
                    if on_value and len(live) < 2:
                        continue
                    name = name_of("flatv" if on_value else "flat", zero, cancel)
                    write(f"{name}.g", make(zero, cancel, on_value))
                    write(f"c_{name}_half.g", make(zero, cancel, on_value, b=B_HALF))
                    proving.append(name)
    # Every product zero: then eb is zero, so decide does not compare, and
    # Gappa proves the goal from the contradiction. Its control drops eb normal.
    write("flat_z012345.g", make(tuple(TERMS)))
    write("c_flat_z012345_noeb.g", make(tuple(TERMS), eb_normal=False))
    write("c_flat_nonormal.g", make(normal=False))
    write("c_flat_noeb.g", make(eb_normal=False))
    print(f"{len(proving)} scripts that must prove with a control at u / 2 each, and flat_z012345.g")
```

`flat.g`, whole, as `flatcases.py` writes it:

```gappa
# restates: crates/rcad-geom/src/plane.rs on_floats at 8c943dfd
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at 8c943dfd
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 8c943dfd
# restates: crates/rcad-predicates/src/cascade.rs decide at 8c943dfd
# SideOfAFlatSurfaceThroughAPoint::on_floats and its error bound: the sign
# of facing . point - facing . through.
@rnd = float<ieee_64, ne>;

# The facing, the point and the through point, each an f64. A product that
# is zero has the double 0 for its point's or its through point's
# coordinate; where the facing's coordinate is the zero, both its products
# are zero, and every number below is the same as with both of those
# coordinates zero.
f0 = rnd(f0_); f1 = rnd(f1_); f2 = rnd(f2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
q0 = rnd(q0_); q1 = rnd(q1_); q2 = rnd(q2_);

# What on_floats computes, line for line. The fold starts at 0.0, and
# adding 0.0 or a zero product to a double gives that double back
# (addzero.g), so the zero products and the first addition are left out.
t0 = rnd(f0 * p0); t1 = rnd(f1 * p1); t2 = rnd(f2 * p2); t3 = rnd(f0 * q0); t4 = rnd(f1 * q1); t5 = rnd(f2 * q2);
S1 = t0 + t1;
s1 = rnd(S1);
S2 = s1 + t2;
s2 = rnd(S2);
S3 = s2 - t3;
s3 = rnd(S3);
S4 = s3 - t4;
s4 = rnd(S4);
y = s4 - t5;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |t3|) + |t4|) + |t5|);

B = 6b-53 + 1b-99;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = f0 * p0 + f1 * p1 + f2 * p2 - f0 * q0 - f1 * q1 - f2 * q2;

# The permanent with no rounding of its own.
Phat = |t0| + |t1| + |t2| + |t3| + |t4| + |t5|;

# What each rounding multiplied its argument by.
rp0 = t0 / (f0 * p0); rp1 = t1 / (f1 * p1); rp2 = t2 / (f2 * p2); rp3 = t3 / (f0 * q0); rp4 = t4 / (f1 * q1); rp5 = t5 / (f2 * q2);
rs1 = s1 / S1; rs2 = s2 / S2; rs3 = s3 / S3; rs4 = s4 / S4;

# Each product's term, as y carries it, less the term itself: e is
# that amount and W the same over the product as on_floats rounds it.
# The through point's terms are subtracted, so their e is the other way
# round.
W0 = rs1 * rs2 * rs3 * rs4 - 1 / rp0;
W1 = rs1 * rs2 * rs3 * rs4 - 1 / rp1;
W2 = rs2 * rs3 * rs4 - 1 / rp2;
W3 = rs3 * rs4 - 1 / rp3;
W4 = rs4 - 1 / rp4;
W5 = 1 - 1 / rp5;
e0 = t0 * W0; e1 = t1 * W1; e2 = t2 * W2; e3 = -(t3 * W3); e4 = -(t4 * W4); e5 = -(t5 * W5);
c0 = |t0|; c1 = |t1|; c2 = |t2|; c3 = |t3|; c4 = |t4|; c5 = |t5|;

{
  # Every input is a finite f64.
     |f0| <= 1b1024 /\ |f1| <= 1b1024 /\ |f2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |q0| <= 1b1024 /\ |q1| <= 1b1024 /\ |q2| <= 1b1024
  # No factor of a product left is zero. For an f64 that is the same as
  # being at least the smallest subnormal.
  /\ |f0| >= 1b-1074 /\ |f1| >= 1b-1074 /\ |f2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074 /\ |q0| >= 1b-1074 /\ |q1| >= 1b-1074 /\ |q2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S1| >= 1b-1074 /\ |S2| >= 1b-1074 /\ |S3| >= 1b-1074 /\ |S4| >= 1b-1074
  # product_is_normal on the products left.
  /\ |t0| >= 1b-1022 /\ |t1| >= 1b-1022 /\ |t2| >= 1b-1022 /\ |t3| >= 1b-1022 /\ |t4| >= 1b-1022 /\ |t5| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t0 -/ f0 * p0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t1 -/ f1 * p1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t2 -/ f2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t3 -/ f0 * q0 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t4 -/ f1 * q1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ t5 -/ f2 * q2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t0 / |t0| in [-1, 1] /\ t1 / |t1| in [-1, 1] /\ t2 / |t2| in [-1, 1] /\ t3 / |t3| in [-1, 1] /\ t4 / |t4| in [-1, 1] /\ t5 / |t5| in [-1, 1]
->
  (y - Ex) / Phat in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t0 / (f0 * p0) -> 1 + (t0 - f0 * p0) / (f0 * p0);
t1 / (f1 * p1) -> 1 + (t1 - f1 * p1) / (f1 * p1);
t2 / (f2 * p2) -> 1 + (t2 - f2 * p2) / (f2 * p2);
t3 / (f0 * q0) -> 1 + (t3 - f0 * q0) / (f0 * q0);
t4 / (f1 * q1) -> 1 + (t4 - f1 * q1) / (f1 * q1);
t5 / (f2 * q2) -> 1 + (t5 - f2 * q2) / (f2 * q2);
s1 / S1 -> 1 + (s1 - S1) / S1;
s2 / S2 -> 1 + (s2 - S2) / S2;
s3 / S3 -> 1 + (s3 - S3) / S3;
s4 / S4 -> 1 + (s4 - S4) / S4;

# The error is the sum of the products' errors, each a fraction of its
# own piece of the permanent; add_rr then averages them.
(y - Ex) / Phat -> ((Phat + (y - Ex)) - Phat) / Phat;
Phat + (y - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + e2) + (c3 + e3) + (c4 + e4) + (c5 + e5);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
(c3 + e3) - c3 -> e3;
(c4 + e4) - c4 -> e4;
(c5 + e5) - c5 -> e5;
e0 / c0 -> (t0 / |t0|) * W0;
e1 / c1 -> (t1 / |t1|) * W1;
e2 / c2 -> (t2 / |t2|) * W2;
e3 / c3 -> -((t3 / |t3|) * W3);
e4 / c4 -> -((t4 / |t4|) * W4);
e5 / c5 -> -((t5 / |t5|) * W5);

# From the permanent to the one on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

`why.py`, whole. It is 464.12's with `sphere` read as `flat`, and the goal
`rpx in ? /\ rpr in ?` read as `rp0 in ?`:

```python
"""Write two controls again with more goals, to name where each breaks or how
it gets through: c_flat_nonormal_why.g asks for what the rounding of the first
product multiplied it by; c_flat_noeb_why.g asks for P and eb, and
c_flat_noeb_rel.g for eb's relative error alone."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
GOAL = "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n"


def rewrite(src, dst, goal):
    text = open(src).read()
    assert text.count(GOAL) == 1, src
    open(dst, "w").write(text.replace(GOAL, goal))


rewrite("c_flat_nonormal.g", "c_flat_nonormal_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ rp0 in ?\n")
rewrite("c_flat_noeb.g", "c_flat_noeb_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ P in ? /\\ eb in ?\n")
text = open("c_flat_noeb.g").read()
head, rest = text.split("->\n", 1)
_, hints = rest.split("}\n", 1)
open("c_flat_noeb_rel.g", "w").write(head + "->\n  eb -/ B * P in ?\n}\n" + hints)
```

`bisect.py`, whole, as run. It is 464.12's with three lines changed: `B_CODE`
is the code's `B` here, `NIX` is the `gappa` store path from 464.4's pin
rather than `nix shell`, and the timeout is 600 s rather than 200. Its `best`
line is never read; the line after it is what prints a failing run's best:

```python
"""Find the smallest B = a u + m u^2 a script proves at, in steps of u^2, by
bisection on m, as 464.8 did. Run as: python3 bisect.py <file.g> <a> <hi m>.
Writes <name>_b<a>_<m>.g beside itself for each B tried, and prints each run's
exit and the goal's best bound when it fails."""
import os
import re
import subprocess
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
B_CODE = "B = 6b-53 + 1b-99;"
NIX = ["/nix/store/0ihij0570a325g02l4wnf0idk8vqxr9b-gappa-1.8.0/bin/gappa", "-Eprecision=200"]

name, a, hi = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
text = open(name).read()
assert text.count(B_CODE) == 1, f"no single B line in {name}"


def proves(m):
    b = f"B = {a}b-53 + {m}b-106;" if m else f"B = {a}b-53;"
    out_name = f"{name[:-2]}_b{a}_{m}.g"
    open(out_name, "w").write(text.replace(B_CODE, b))
    run = subprocess.run(NIX + [out_name], capture_output=True, text=True, timeout=600)
    best = re.findall(r"best: \[[^\]]*\]|\{[-0-9.e+]+, [-0-9.e+^()]+\}\]", run.stderr + run.stdout)
    fail = [l for l in (run.stderr + run.stdout).splitlines() if "best" in l or "failed" in l.lower()]
    print(f"{out_name}: {b} exit {run.returncode}; {' | '.join(fail[:3])}", flush=True)
    return run.returncode == 0


lo = 0
assert not proves(lo), "proves with no u^2 term"
assert proves(hi), "does not prove at the top of the range"
while hi - lo > 1:
    mid = (lo + hi) // 2
    if proves(mid):
        hi = mid
    else:
        lo = mid
print(f"smallest: {a}u + {hi}u^2 proves, {a}u + {lo}u^2 fails")
```

`flat.g`'s run printed, in order, `5u + 128u²`, `64`, `48`, `40` and `38`
proving and `5u`, `32`, `36` and `37` failing, each at best `[-1, 1]`;
`flatv.g`'s `6u + 128u²`, `64` and `48` proving and `6u`, `32`, `40`, `44`,
`46` and `47` failing, the same way.

`one.sh`, whole:

```sh
#!/bin/bash
# Run one Gappa script under 464.4's pin (the store path nix build gives for
# it), writing <name>.out and printing "<name> exit <code> <seconds>s".
# With "coq" as the second argument, also write <name>.v with -Bcoq and check
# it with coqc against gappalib 1.8.0 and flocq 4.2.2, printing that too.
cd "$(dirname "$0")"
g=/nix/store/0ihij0570a325g02l4wnf0idk8vqxr9b-gappa-1.8.0/bin/gappa
coqc=/nix/store/cgfd855dwk38jxngg5wdvysdjjn2wv3i-coq-8.20.1/bin/coqc
export COQPATH=/nix/store/1rqb30ra496k5zpw9aji329zn00rjggv-coq8.20-gappalib-1.8.0/lib/coq/8.20/user-contrib:/nix/store/p3ka5vy0zshbknhc9pphsffxxwfkcwq4-coq8.20-flocq-4.2.2/lib/coq/8.20/user-contrib
f=$1
n=${f%.g}
start=$(date +%s.%N)
timeout 600 "$g" -Eprecision=200 "$f" > "$n.out" 2>&1
code=$?
end=$(date +%s.%N)
printf '%s exit %d %.2fs\n' "$n" "$code" "$(echo "$end - $start" | bc)"
if [ "$2" = coq ] && [ "$code" = 0 ]; then
  start=$(date +%s.%N)
  timeout 600 "$g" -Eprecision=200 -Bcoq "$f" > "$n.v" 2> "$n.bcoq"
  bcode=$?
  timeout 600 "$coqc" "$n.v" > "$n.coqc" 2>&1
  ccode=$?
  end=$(date +%s.%N)
  printf '%s coq -Bcoq exit %d coqc exit %d %.2fs lines %d hints %d\n' "$n" "$bcode" "$ccode" \
    "$(echo "$end - $start" | bc)" "$(wc -l < "$n.v")" "$(grep -c '^Hypothesis a[0-9]' "$n.v")"
fi
```

**10. What this leaves.** The rows filed with this write-up:

- A row to restate `plane.rs:1050-1053` by 5: by its own terms the count is
  57, not 52, and `on_floats` lets zero products through, for which 6 proves
  the bound at the code's constant.
- A row, after that one, to run `flatcases.py` with one, two and three of the
  six products below the normals, as 464.8's `planecases.py` does for
  `SideOfAFlatSurface`, and then to say whether `on_floats` needs its six
  `product_is_normal` checks and to restate `plane.rs:1052-1053` by what that
  shows. Q2684 asks the same of `SideOfAFlatSurface`, not of this predicate.
- A row for `flat.g`'s first `restates` line. 464.10's b names the item "as
  `just src` takes it", and `just src on_floats crates/rcad-geom/src/plane.rs`
  lists two, `SideOfAFlatSurface`'s at `plane.rs:847` and this one's at
  `plane.rs:976`, rather than printing one. So the line names both, and
  464.8's `plane.g` would name the same two.

And what is not a row:

- Q2700 already waits on Q2678, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- How `c_flat_noeb.g` gets past `eb`'s rounding (7) was not traced, as with
  plane's and the sphere's.
- `field` on these scripts' hints (464.10).

### 464.14 The side of rcad-geom's sphere

*Unfinished: Q2679's first lane relayed here, past its budget before any
script but two had run. What follows is its draft and its plan. The lane after
it writes `skincases.py`, runs it, and writes this up in 464.12's form.*

Q2679 takes up `SIDE_OF_SPHERE_ERROR_BOUND` in `crates/rcad-geom/src/sphere.rs`
at `05d44c66`. `git log --oneline 3d60b0de..05d44c66` on `sphere.rs`,
`cascade.rs` and `point.rs` prints nothing, so the last two are as 464.12
read them. `just src on_floats crates/rcad-geom/src/sphere.rs`
prints one item, so a `restates` line naming it is not 464.13's two-item case.

**What the proof guards.** `Sphere::side_of` (`sphere.rs:216-227`) runs the
cascade on `HowFarFromTheSkin`, and `Surface::Sphere` reaches it from
`three_surfaces.rs:3294` and `validate.rs:4497`, so unlike 464.12's this is a
predicate a user reaches.

**`on_floats`, line for line** (`sphere.rs:241-269`). It is 464.12's
`HowFarOut` operation for operation: three gaps, `None` unless
`product_is_normal` holds for each gap with itself and the radius with itself
(`sphere.rs:251-254`), the four squares, `value = s0 + s1 + s2 − rr` and
`permanent = s0 + s1 + s2 + rr`, whose first two sums are `value`'s. So
464.12's `sphere.g` is this `on_floats` with only `B` changed. The constant
(`sphere.rs:425-428`) is `(6 + 48ε) ε` with `ε = f64::EPSILON / 2`, which is
`u`: `6u + 48u²`, written `B = 6b-53 + 3b-102;`. Each step the Rust takes is
exact, since `48ε` is a multiple of `2⁻⁵⁰`, the ulp of 6.

**What the comments claim, to be checked against the runs.**

- `sphere.rs:247-250`, on the check: "One of them in the subnormals and this
  level certifies a wrong sign." It is the sentence 464.8's 4 found false for
  one product of `SideOfAFlatSurface`, and the case 464.12's 5 left to this
  row: a square below the normals with the others normal.
- `sphere.rs:401-424`, on the constant: six roundings to `value`, which
  464.12's 4 found Gappa prints as `6u` for the same arithmetic. "The `48ε²`
  covers the higher-order terms and the rounding of the permanent itself, with
  room to spare: what it has to beat is about `15ε`." That is `ε²`, not `ε`,
  and by the comment's own terms it is `15ε²` from `(1 + ε)⁶`, `18ε²` from the
  permanent's three additions against the `6ε`, and `6ε²` from rounding `B ·
  P`: 39, not 15. Both are under 48, and under 464.12's smallest `B`s for this
  arithmetic, `5u + 28u²` on `y` and `6u + 30u²` on `value`, to be run again
  here with `bisect.py`.
- "It holds only while nothing overflows or falls into the subnormals": a zero
  square is not a subnormal, so 464.12's zero cases do not contradict it; the
  cases below the normals answer it.

**Run so far**, Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, with
464.13's `one.sh`:

- `tinysq.g` exits 0, and its `gappa -Bcoq` and `coqc` exit 0, a proof of 397
  lines with no hints:

  ```gappa
  # tinysq.g: a square of two doubles at or below 2^-1075 rounds to zero, and
  # one above it rounds to at least 2^-1074, the smallest subnormal.
  @rnd = float<ieee_64, ne>;
  g = rnd(g_);
  X = g * g;
  { |g| <= 1b1024 /\ |g| >= 1b-1074
    -> (X <= 1b-1075 -> rnd(X) in [0, 0]) /\ (X >= 1b-1075 /\ X <= 1b-1022 -> rnd(X) in [0, 1b-1022]) }
  ```

- A probe, `gx = rnd(px − cx)` with `|gx|` in `[2⁻¹⁰⁷⁴, 2⁻⁵¹¹]`, prints
  `gx −/ Gx` within `±2⁻⁵³` with no split at `2⁻¹⁰²²`: a difference of two
  doubles that lands below the normals is exact, and Gappa finds that itself,
  as 464.9 found for a sum. So a gap whose square is below the normals keeps a
  ratio hint.

**The plan.** `skincases.py` is 464.12's `spherecases.py` with the
`restates` lines naming `sphere.rs` `on_floats` and
`SIDE_OF_SPHERE_ERROR_BOUND` at `05d44c66`, `B_CODE` the line above, and the
prefix `skin`, plus two more states for each of the four squares besides
normal (`n`) and zero (`z`):

- `s`, below the normals and rounding to a double that is not zero: `X = g *
  g` in `[2⁻²¹⁴⁸, 2⁻¹⁰²²]`, `s = rnd(X)`, `|s| ≥ 2⁻¹⁰⁷⁴`. Its rounding is
  carried as an amount, `d = (s − X) · carry`, as `planecases.py` carries
  `dk`, and `e = X · W` with `W = carry − 1 / (rg · rg)`; for the radius `d =
  −(rr − X) · carry` and `W = 1 − carry`.
- `u`, rounding to zero: `X` in `[2⁻²¹⁴⁸, 2⁻¹⁰⁷⁵]` and `s = 0;` by
  `tinysq.g`. Its whole true square is the error, `−G · G` for a gap (with
  the hint `G * G -> X / (rg * rg);`) and `r · r` for the radius, and it is
  treated as a zero in the sums, with 464.12's hints such as `sx + sy -> sy;`.

With no `s` or `u` term the script is 464.12's form. With one, the step is
`planecases.py`'s: `(out − Ex) / eb -> ((Σ e) / Phatx) · (Phatx / P) · (P /
eb) + (Σ d) / eb`, `Phatx` being `Phat` with `X` in place of each `s` term's
square, `Phatx / P -> Phat / P + (Σ (X − s)) / P`, and `P -> eb / (B · (1 +
(eb − B · P) / (B · P)));`. Each sum then has every addend zero (written
unrounded) or one that is not zero, so no sum needs 464.13's cancelling split.
A case with no `n` term has `P` at most `4 · 2⁻¹⁰²²`, so `eb` is not normal,
and it should prove only from that contradiction, as `sphere_zxyzr.g` does;
its control drops `eb` normal, and every other case's control is at `B = u /
2`. All `4⁴` states on `y`, and those with an `n` term on `value`.
