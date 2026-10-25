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
464.13 is Q2678, on the side of a flat surface through a point. 464.14 is
Q2679, on the side of rcad-geom's sphere. 464.15 is Q2680, on the side of
rcad-geom's cylinder. 464.16 is Q2681, on the side of rcad-geom's cone, and
464.17 is Q2682, on the side of rcad-geom's torus.*

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
  464.14  The side of rcad-geom's sphere
  464.15  The side of rcad-geom's cylinder
  464.16  The side of rcad-geom's cone
  464.17  The side of rcad-geom's torus

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
`SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT` (464.13), `SIDE_OF_SPHERE` (464.14),
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
  of its own. `surface.rs`' `ERROR_BOUND` is done in 464.12,
  `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND` in 464.13,
  `SIDE_OF_SPHERE_ERROR_BOUND` in 464.14, and `SIDE_OF_CYLINDER_ERROR_BOUND`
  in 464.15 where no factor is zero.
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
  464.13), `SIDE_OF_SPHERE_ERROR_BOUND` (Q2679, done in 464.14),
  `SIDE_OF_CYLINDER_ERROR_BOUND` (Q2680, done in 464.15 where no factor is
  zero), `SIDE_OF_CONE_ERROR_BOUND` and
  `SIDE_OF_TORUS_ERROR_BOUND` in `rcad-geom`.
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
is where that question has a user behind it. 464.14's 6 ran it there, on an
`on_floats` that is this one operation for operation at a smaller `B`, `6u +
48u²`: the bound holds with any of the squares below the normals while one
square is normal. It was not run here, at `8u + 64u²`.

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
  464.14's 6 ran it on `rcad-geom`'s sphere, the same arithmetic at a smaller
  `B`.
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

Q2679 took up `SIDE_OF_SPHERE_ERROR_BOUND` in `crates/rcad-geom/src/sphere.rs`,
in six lanes on 2026-10-25. The first read the code, proved `tinysq.g` and
planned the cases. The second wrote `skincases.py` and `tinyzero.g`, checked
the generator against 464.12's and ran twelve of its scripts. The third ran
every proof, control, bisection and `coqc`, and wrote 1 to 5. The fourth ran
`tinyzero_nohint.g` and checked the figures. The fifth wrote 6 to 8 and began
9, and the sixth finished 9 and wrote 10. The bound is proved at the code's
constant, on the sum before `value`'s last rounding and on `value` too, in
every case where `on_floats` hands `decide` a number. It is proved as well in
every case `on_floats`' check refuses, a square below the normals: with
another square normal the bound still holds, and with none normal `eb` is not
normal, so `decide` does not compare. Coq checks each proof, modulo its hints.
So the comment on the check, which says one square in the subnormals makes
this level certify a wrong sign, is false, and the doc comment on the constant
writes `15ε` where it means `15ε²` and miscounts its own terms. Both are in
`sphere.rs`, and fixing them is two rows (10).

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, against
`sphere.rs`, `point.rs` and `cascade.rs` at `05d44c66`. `git log --oneline
3d60b0de..05d44c66` on the three prints nothing, so the last two are as
464.12 read them, and `git log --oneline 05d44c66..48cbcbde3` on them prints
nothing too, so all three are as they stand at `48cbcbde3`, where this
write-up was finished. `u` is `2⁻⁵³`, and the code's `B = 6u + 48u²` is
written `B = 6b-53 + 3b-102;`. Each script follows 464.8's pattern: one ratio hint per
rounding, the error split into one term per square of the permanent so that
Gappa averages them, 464.5's lemma for each square `on_floats` checks is
normal, and the step from `y`, the sum before `value`'s last rounding, to the
sign as 464.4 gives it.

**1. What the proof guards.** `Sphere::side_of` (`sphere.rs:216-227`) runs
the cascade on `HowFarFromTheSkin`, once a guard has refused a point or a
sphere that is not real numbers. `Surface::Sphere` reaches it from
`three_surfaces.rs:3294` and from `validate.rs:4497`, and `lib.rs:120` exports
`Sphere`, whose `side_of` is `rcad-predicates`' `SideOfSurface` trait. So
unlike 464.12's, this proof guards a predicate a user reaches.

**2. `on_floats`, line for line.** `HowFarFromTheSkin::on_floats`
(`sphere.rs:241-269`) returns `None` for a point that is not exact `f64`s
(`sphere.rs:242`), takes the three gaps from the centre to the point
(`sphere.rs:245`), and returns `None` unless `product_is_normal` holds for
each gap with itself and for the radius with itself (`sphere.rs:251-254`).
Then it squares the four (`sphere.rs:257-258`) and returns `value = s0 + s1 +
s2 − rr` and `error_bound = SIDE_OF_SPHERE_ERROR_BOUND * permanent`, with
`permanent = s0 + s1 + s2 + rr` (`sphere.rs:263-268`). That is 464.12's
`HowFarOut::on_floats` operation for operation, and `skin.g` writes it as
`sphere.g` does: ten roundings, the three gaps, the three squares, `rr`,
`s01`, `s012` and `value`, `y` taking the first nine, and `P = rnd(s012 +
rr)`, since the permanent's first two sums are `value`'s. `sameas.py` (9)
checks that each script of 6 for a case of normal and zero squares alone is
464.12's for the same zero set, once the `restates` lines, the title line and
the `B` line are dropped. It prints `64 pairs, 0 differ`.

The constant (`sphere.rs:425-428`) is `(6 + 48ε) ε` with `ε = f64::EPSILON /
2`, which is `u`. That is `6u + 48u²`, which a double holds exactly, and so
does each step the Rust takes to it, since `48ε` is a multiple of `2⁻⁵⁰`, the
ulp of 6.

**3. The bound on `y`.** `skin.g` (9 below) proves `|y − Ex| ≤ eb` at the
code's `B`, printing `(y − Ex) / Phat` within `5.55112e-16` and `P −/ Phat`
within `3.33067e-16`, `5u` and `3u` to the digits Gappa prints, as `sphere.g`
did. `Phat` is the permanent with no rounding of its own. The smallest `B` it
proves, by bisection over `m` in `B = 5u + m u²`, is `5u + 28u²`, as in
464.12's 3: `5u + 27u²` fails, best `[-1, 1]` as Gappa prints it, rounded,
and so does `5u`.

**4. The claim on `value`.** `skinv.g` makes the same claim on `value`, the
double `on_floats` returns. It proves at the code's `B`, printing `(value −
Ex) / Phat` within `6.66134e-16`, `6u` to the digits printed. Its smallest
`B`, by the same bisection over `B = 6u + m u²`, is `6u + 30u²`, as in
464.12's 4: `6u + 29u²` fails, and so does `6u`. So here `value` itself stays
within `eb`, as the other sphere's does. `decide` does not need it to: the doc
on `Filtered::error_bound` (`cascade.rs:25-35`) asks only for the bound on
the sum before `value`'s last rounding, which 3 proves.

**5. What the comments claim.** The doc comment on the constant,
`sphere.rs:401-424`, counts the roundings and then says:

> Six roundings, each costing at most a factor of `(1 + epsilon)`, come to
> `(1 + epsilon)^6 - 1 <= 6*epsilon + 15*epsilon^2 + ...` times the permanent.
>
> The radius term travels a shorter chain — one rounding to square it and one
> to subtract it — so the six covers it too. The `48*epsilon^2` covers the
> higher-order terms and the rounding of the permanent itself, with room to
> spare: what it has to beat is about `15*epsilon`.
>
> Every step of that argument is a *relative* one, so like every bound of this
> shape it holds only while nothing overflows or falls into the subnormals.
> [`HowFarFromTheSkin::on_floats`] enforces the bottom end product by product
> through [`product_is_normal`], and `decide` the top end by refusing a bound
> that is not itself a normal number.

- Six roundings to `value`, the gap's counted twice since it is squared,
  agrees with the `6u` of 4. On `y` the same count is five, and 3 prints `5u`.
- "What it has to beat is about `15*epsilon`": that is `ε²`, the term of `(1 +
  ε)⁶ − 1` the sentence before names, and it leaves out the two things the
  same sentence says the `48ε²` covers. The permanent's own rounding is its
  three additions, which 3 prints as `P −/ Phat` within `3u`, and that `3ε`
  taken against the `6ε` of the bound costs `18ε²`. The last rounding, of `B ·
  P`, costs `ε` of `eb`, which is `6ε²` of the permanent. So by the comment's
  own terms the count is `15 + 18 + 6 = 39`. `48` is above 39, and above both
  smallest `B`s, `5u + 28u²` and `6u + 30u²`, so the slip costs nothing, but
  the sentence gives the wrong power and the wrong number.
- "It holds only while nothing ... falls into the subnormals", which
  `on_floats` enforces "product by product through `product_is_normal`".
  `product_is_normal(a, b)` is true when `a` or `b` is zero
  (`cascade.rs:67-69`), so a zero square gets through, and the bound holds for
  it (6). A square below the normals, which the check refuses, keeps the bound
  too (6): with another square normal it proves at the code's `B`, and with
  none normal `eb` is not normal. So the bound does not need what "only" says
  it needs.
- The comment on the check, `sphere.rs:248-250`: "The bound below is a
  fraction of the permanent, which only bounds anything while every product
  keeps its relative accuracy. One of them in the subnormals and this level
  certifies a wrong sign." That is the sentence 464.8's 4 found false for one
  product of `SideOfAFlatSurface`. Here it is false for any of the four
  squares, alone or together, by 6: without the check `on_floats` would hand
  `decide` a number in those cases, and `decide` would certify no wrong sign
  from it. What the check does is send them on to level 2. Whether
  `on_floats` keeps it is a row (10).

**6. The squares below the normals.** 464.8's `o2s.g` splits on a product
that `on_floats` does not check. Here there is none, since `on_floats` checks
all four squares before it computes anything, and what the check lets through
is a zero, as in 464.12's 6. What 5 asks about is what the check refuses: a
square below the normals, which the comment on the check says makes this level
certify a wrong sign. So each of the four squares is run in each of four
states, and each case is a script:

- `n`, normal: the square as `on_floats` rounds it is at least `2⁻¹⁰²²`;
- `z`, of a zero factor, which the check lets through;
- `s`, below the normals and rounding to a double that is not zero;
- `u`, below the normals and rounding to zero.

Those four cover every square. Write `X = g · g` for the square of a gap or
the radius `g` before its rounding. If `g` is zero, the square is `z`.
Otherwise `X ≥ 2⁻²¹⁴⁸`. If `rnd(X) ≥ 2⁻¹⁰²²` it is `n`. If not, `X <
2⁻¹⁰²²`, since rounding is monotone and `2⁻¹⁰²²` is a double, and then `|g| <
2⁻⁵¹¹`. Then `rnd(X)` is zero or, being a double that is not zero, at least
`2⁻¹⁰⁷⁴`, which is `s`. `tinyzero.g` proves that a square below the normals
that rounds to zero is at most `2⁻¹⁰⁷⁵`, which is the range a `u` script
assumes. The other way round, `tinysq.g`'s first goal proves that every
square at or below `2⁻¹⁰⁷⁵` rounds to zero. So the `0` a `u` script writes
for the square is `rnd(X)` over the script's whole range of `X`, and the
script is a claim about the code for every square in it.

`tinysq.g`, whole, as the first lane ran it, exits 0, and its `gappa -Bcoq` and
`coqc` exit 0, a proof of 397 lines with no hints:

```gappa
# tinysq.g: a square of two doubles at or below 2^-1075 rounds to zero, and
# one above it rounds to at least 2^-1074, the smallest subnormal.
@rnd = float<ieee_64, ne>;
g = rnd(g_);
X = g * g;
{ |g| <= 1b1024 /\ |g| >= 1b-1074
  -> (X <= 1b-1075 -> rnd(X) in [0, 0]) /\ (X >= 1b-1075 /\ X <= 1b-1022 -> rnd(X) in [0, 1b-1022]) }
```

Its comment claims more than its second goal proves. "One above it rounds to
at least `2^-1074`" is true, but the second goal proves only `rnd(X)` in `[0,
2⁻¹⁰²²]`. What makes the comment true is `tinyzero.g` read the other way
round, and that no double lies between zero and `2⁻¹⁰⁷⁴`. The cover above
does not use the second goal, and the comment the scripts of 9 carry on their
squares below the normals, "tinysq.g and tinyzero.g split it at 2^-1075", is
true read with both. The script is kept here as it was run.

`tinyzero.g`, whole, as the second lane ran it, exits 0, and its `gappa -Bcoq`
and `coqc` exit 0, a proof of 190 lines with 1 hint:

```gappa
# tinyzero.g: a square of two doubles below the normals that rounds to zero
# is at most 2^-1075. With tinysq.g that splits the squares below the normals
# in two at 2^-1075: at or below it they round to zero, above it they do not.
@rnd = float<ieee_64, ne>;
g = rnd(g_);
X = g * g;
s = rnd(X);
{ |g| <= 1b1024 /\ |g| >= 1b-1074 /\ X <= 1b-1022 /\ s in [0, 0] -> X in [0, 1b-1075] }
X -> s - (s - X);
```

Gappa warns "no need for the rewriting at line 9 column 17" on it, in its
output and in its `-Bcoq` print. Line 9 is its one hint, and column 17 is that
line's `;`. `tinyzero_nohint.g`, the same script without the hint, exits 0,
and its `gappa -Bcoq` and `coqc` exit 0, a proof of 175 lines with no hints.
So the warning is right, and the hint is not needed. `tinyzero.g` is kept here
as it was run.

```gappa
# tinyzero.g without its one hint, to read what Gappa's warning on that hint means.
@rnd = float<ieee_64, ne>;
g = rnd(g_);
X = g * g;
s = rnd(X);
{ |g| <= 1b1024 /\ |g| >= 1b-1074 /\ X <= 1b-1022 /\ s in [0, 0] -> X in [0, 1b-1075] }
```

A gap whose square is below the normals still keeps a ratio hint.
`relgap.g`, the first lane's probe, whole, prints `gx −/ Gx` within `±2⁻⁵³`
with no split at `2⁻¹⁰²²`: a difference of two doubles that lands below the
normals is exact, and Gappa finds that itself, as 464.9 found for a sum.

```gappa
@rnd = float<ieee_64, ne>;
px = rnd(px_); cx = rnd(cx_);
Gx = px - cx;
gx = rnd(Gx);
{ |px| <= 1b1024 /\ |cx| <= 1b1024 /\ |gx| >= 1b-1074 /\ |gx| <= 1b-511
  -> gx -/ Gx in ? /\ (gx - Gx) / Gx in ? }
```

**How `skincases.py` writes a case.** `skincases.py` (9 below) is 464.12's
`spherecases.py` with the `restates` lines naming `sphere.rs` `on_floats` and
`SIDE_OF_SPHERE_ERROR_BOUND` at `05d44c66`, `B_CODE` the code's `B`, the
prefix `skin`, and the states `s` and `u`. A case is named by four letters,
one for each of the `x`, `y` and `z` gaps and the radius, so `skin.g` is
`nnnn`, and 464.12's `sphere_zxr.g` is `skin_zznz.g` here.

- An `s` square is `X = g * g` in `[2⁻²¹⁴⁸, 2⁻¹⁰²²]`, with `s = rnd(X)` at
  least `2⁻¹⁰⁷⁴`. What its rounding cost is carried as an amount, `d = (s −
  X) · carry`, as 464.8's `planecases.py` carries `dk`, and the rest as a
  fraction, `e = X · W` with `W = carry − 1 / (rg · rg)`. For the radius `d =
  −(rr − X) · carry` and `W = 1 − carry`.
- A `u` square is `X` in `[2⁻²¹⁴⁸, 2⁻¹⁰⁷⁵]`, and the script writes `s = 0;`
  for it, by `tinysq.g`. Its whole true square is its `d`, `−G · G` for a gap,
  with the hint `G * G -> X / (rg * rg);`, and `r · r` for the radius. In the
  sums it is a zero, with 464.12's hints such as `sx + sy -> sy;`.
- Either kind also gets `|g| ≤ 2⁻⁵¹¹`, which follows from `X ≤ 2⁻¹⁰²²`, as
  `relgap.g` has it.

With no `s` or `u` square the script is 464.12's for the same zero set, which
`sameas.py` checks (2). With one, the step is `planecases.py`'s: `(out − Ex) /
eb -> ((Σ e) / Phatx) · (Phatx / P) · (P / eb) + (Σ d) / eb`, `Phatx` being
`Phat` with `X` in place of each `s` square and without each `u` one, `Phatx /
P -> Phat / P + (Σ (X − s)) / P`, and `P -> eb / (B · (1 + (eb − B · P) / (B
· P)));`. `skin_snnu.g`, whole in 9, is that form. Every square is at least
zero, so a sum of them is zero only when each of its addends is, and the
script then writes it unrounded. So no sum needs 464.13's split for a sum
that cancels. That makes 256 cases on `y`, 175 with an `n` square and 81
without, and the 175 with one again on `value`.

**What they print.** Each of the 431 proves at the code's `B`, and its `gappa
-Bcoq` and `coqc` exit 0 (8).

- The 15 cases of `n` and `z` alone print, on `y` and on `value`, what 464.12's
  6 lists for the same zero sets, line for line. `skin_zzzz.g` prints
  "remaining results are pointless", as `sphere_zxyzr.g` did.
- The 160 cases on `y` with an `s` or `u` square and at least one `n`, 104 of
  them with one `n`, 48 with two and 8 with three, print the relative part `(e)
  / Phatx` within `u`, `3u`, `4u` or `5u` to the digits Gappa prints,
  `1.11022e-16` to `5.55112e-16`. They print `P` at least `2^(-971.585)`, the
  least `P` that `eb` normal allows, as 464.8's planes printed, and the `d` sum
  within `±2^(-1073.41)` at the widest, which is `3 · 2⁻¹⁰⁷⁵`: three squares,
  each moved at most `2⁻¹⁰⁷⁵` by its rounding. Where the `x` and `y` squares
  are the only ones left, as in `skin_nnuu.g`, Gappa names `Phatx` by its sum
  and prints `(ex + ey) / (sx + sy)` within `4.44089e-16`.
- The same 160 on `value` print `(e) / Phatx` within each of `u` to `6u`,
  `1.11022e-16` to `6.66134e-16`, and the same `P` and `d`.
- The 80 cases with an `s` or `u` square and no `n` print "remaining results
  are pointless, anything can be proved". With no square normal, `P` is at
  most `4 · 2⁻¹⁰²²`, so `eb` is not normal and `decide` does not compare, and
  the script proves from that contradiction alone (7).

So with at least one square normal the bound holds at the code's `B`,
whatever the other three are: normal, zero, or below the normals either way.
On `y` there is room. The relative part takes at most `5u` of the permanent,
against an `eb` of `6u` of it and more. `eb` normal makes `eb` at least
`2⁻¹⁰²²`, so the `d` sum is at most `3 · 2⁻¹⁰⁷⁵ / 2⁻¹⁰²²`, which is `3u`, of
`eb`, about `18u²` of the permanent. On `value` the relative part takes up to
`6u` of the `6u + 48u²`, and the room the `d` sum leaves there was not argued:
Gappa proves each case at the code's `B`, and the cases with an `s` or `u`
square were not bisected. With no square normal, `eb` is not normal, and
`decide` refuses whatever `on_floats` returns.

So without its check `on_floats` would hand `decide` a number in each case the
check now refuses, and `decide` would certify no wrong sign from it: either
`|y − Ex| ≤ eb`, and 464.4's step from `y` gives the sign, or `eb` is not
normal and `decide` does not compare. The comment on the check (5) is false
for any of the four squares, alone or together. What the check does is send
those cases on to level 2. Whether `on_floats` keeps it is Q2766 (10).

**The shapes.** Which shapes reach these cases was not run. On paper: a
square below the normals needs a gap or the radius under `2⁻⁵¹¹`, and two
different doubles that close are both under `2⁻⁴⁵⁸`, since a double of at
least `2⁻⁴⁵⁸` has no other double within `2⁻⁵¹¹` of it. So the point and the
centre are both under `2⁻⁴⁵⁸` on that axis, or the radius is under `2⁻⁵¹¹`. No
input was built, since every case proves and there is no wrong sign to find
either way. The queue holds one such shape, Q2518 (refuses/D), a ball of
radius `2⁻⁵²⁸` that `Sphere::side_of` refuses. Each of its squares is zero or
near `2⁻¹⁰⁵⁶`, so it is a case with no `n`: `eb` is not normal, and dropping
the check would not answer it.

**7. The controls.** A script whose hypotheses contradict each other proves
every goal (464.7's 3), so each of the 350 scripts of 6 with an `n` square
was run again at `B = u / 2`, written `B = 1b-54;`, as `c_<name>_half.g`. Each
fails, exit 1, with a best that is the relative bound it printed in 6 over `u
/ 2`. `pairs.py` (9) checks each and prints `350 controls at u/2, 0 whose best
is not the relative bound over u/2`. For the 30 of `n` and `z` alone the bests
are 464.12's 7. On `y` with an `s` or `u` they are `[-2, 2]`, `[-6, 6]`, `[-8,
8]` and `[-10, 10]`, and on `value` every even width from `[-2, 2]` to `[-12,
12]`. These controls print `P` at least `2^(-968)`, since `eb` normal asks
more of `P` at a smaller `B`. With 3 and 4's failures at `5u` and `6u`, that
shows none of them proved from a contradiction.

- The 81 `_noeb` controls, `c_skin_zzzz_noeb.g` and one for each of the 80
  cases with an `s` or `u` and no `n`, drop `eb` normal and the lemma line it
  gives. Each fails, best `[-inf, inf]`, printing `eb` in `[0, 0]`, `[0,
  3b-1074]`, `[0, 3b-1073]`, `[0, 3b-1072]` or `[0, 9b-1074]`, so at most `12
  · 2⁻¹⁰⁷⁴`. What proves those 81 is the contradiction with `eb` normal and
  nothing else, as with `sphere_zxyzr.g`.
- `c_skin_nonormal.g` is `skin.g` with the four `product_is_normal`
  hypotheses dropped, and the four lemma lines they give. It fails, best
  `[-4.2731e+909, 4.2731e+909]`. So a script of this kind catches a missing
  check, as 464.4's 3, 464.8's 4, 464.12's 7 and 464.13's 7 did. That does not
  undo 6: without those hypotheses the script has no case split, and 6 shows
  that with one each case proves.
- `c_skin_noeb.g` is `skin.g` without `eb` normal and without the lemma line
  it gives. It proves, printing `(y − Ex) / Phat` within `5u` and `P −/ Phat`
  within `3u`, as `c_sphere_noeb.g` did, and how Gappa gets past `eb`'s
  rounding was not traced. 464.12's 7 argues it on paper, and the same
  argument here is not run either: four squares each at least `2⁻¹⁰²²` put
  `P` at least `2⁻¹⁰²⁰` and `B · P` at least `6 · 2⁻¹⁰⁷³`, which is `24 ·
  2⁻¹⁰⁷⁵`. Rounding moves that by at most `2⁻¹⁰⁷⁵`, a twenty-fourth of it, and
  `5u` of the permanent over twenty-three twenty-fourths of `6u` of it is about
  `0.87`. `decide` still checks that `eb` is normal, and 464.12's 7 argues on
  paper that this check keeps every intermediate finite. `on_floats` is the
  same arithmetic (2), so that carries over.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each of the 431 scripts that prove, and `tinysq.g`, `tinyzero.g` and
`tinyzero_nohint.g`: exit 0 each, the two together in 1.20 to 54.56 s on each
of the 431 in the third lane's run, on a load of 40 to 55 over 24 cores. Gappa
does not write the same proof each run (464.4), so the sizes are that run's.
`skin.g`'s proof is 4047 lines with 18 hints, `skinv.g`'s 4186 with 19, and
`skin_zzzz.g`'s 366 with 2, as `sphere_zxyzr.g`'s was. The others are 301 to
6111 lines with 0 to 21 hints, the smallest `skin_uuuu.v` with 2 and the
largest `skin_snus.v` with 18; `skin_ssss.v` is 567 lines with none.
`tinysq.g`'s is 397 lines with none, `tinyzero.g`'s 190 with 1 and
`tinyzero_nohint.g`'s 175 with none. Each hint enters the Coq file as a
hypothesis with no condition attached, as in 464.4, so each proof stands on
its hints and on what Gappa took to be nonzero: `skin.g`'s `gappa -Bcoq`
prints 59 lines of the form "has been assumed to be nonzero when checking a
rewriting rule", and `skinv.g`'s 66, as `sphere.g`'s and `spherev.g`'s did.
The one other line in any output or `-Bcoq` print is `tinyzero.g`'s warning
that its hint is not needed (6). None says "not trivially equal", which
464.12's 6 met without its hints for a zero, and no `coqc` printed anything.
`field` has not been run on these hints (464.10).

**9. The scripts.** `skincases.py` writes 864 scripts: `skin.g`, `skinv.g`,
the 429 other cases of 6, the 350 controls at `u / 2`, the 81 `_noeb`
controls and the two other controls of 7. After the runs, the fifth lane ran
it again in a fresh directory, and it wrote all 864 byte for byte as they were
run. To run them again, save the nix block of 464.5 as `gappalib18.nix` and
`skincases.py` beside it, and then:

```sh
python3 skincases.py
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 skin.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq skin.g > skin.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc skin.v
```

and the same for every other file it writes, and for the four scripts of 6.
A script that proves exits 0; a control must exit 1. What the third lane ran
is `runall.sh` (below), which runs each through 464.13's `one.sh`, the same
bytes as 464.13's 9 holds, 16 at a time. In that run Gappa took 0.07 to 48.00
s on each script that proves and 0.08 to 155.79 s on each control.

`collect.py`, `brief.py`, `summarize.py` and `pairs.py`, below, read
`run.log` and the outputs, and print the figures 6 to 8 give: `collect.py`
the counts, the exits, the times and the sizes, `brief.py` and `summarize.py`
the groups of 6, and `pairs.py` the check of 7 and the lines of 8 that are not
a result. `sameas.py`, below, checks 2's claim, with 464.12's
`spherecases.py`'s scripts in `ref/`, and prints `64 pairs, 0 differ`.

The smallest `B`s of 3 and 4 come from `bisect.py`, which is 464.13's with
one line changed, `B_CODE` the code's `B` here. `bisects.sh` ran it as
`python3 bisect.py skin.g 5 48` and `python3 bisect.py skinv.g 6 48`. Both
are whole at the end of this part.

`skincases.py`, whole, as the second lane wrote it and the third ran it:

```python
"""Write skin.g, HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND
line for line, and its cases and controls beside it. A case is four letters,
one for each of the gaps x, y, z and the radius r, saying what its square is:
n normal; z of a zero factor, which product_is_normal lets through; s below
the normals and rounding to a double that is not zero, and u below the
normals and rounding to zero, both of which product_is_normal refuses. skin.g
is the case nnnn. It writes:

- skinv.g, the claim on value, the double on_floats returns, not on y;
- skin_<case>.g for every other case, and skinv_<case>.g for each with an n;
- c_<name>_half.g, each script with an n at B = u / 2, which must fail;
- c_skin_<case>_noeb.g, each case with no n without eb normal. With no normal
  square P is too small for eb to be normal, so such a case proves from that
  contradiction, as 464.12's sphere_zxyzr.g does, and without it must fail;
- c_skin_nonormal.g, skin.g without product_is_normal, and c_skin_noeb.g,
  skin.g without eb normal.

A case of n and z alone is 464.12's script for the same zero set, but for its
first lines and B. A case with an s or a u carries what rounding that square
cost as an amount, as 464.8's planecases.py does.

Run as: python3 skincases.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

AXES = ("x", "y", "z")
TERMS = AXES + ("r",)
B_CODE = "B = 6b-53 + 3b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
SAYS = {"z": "is zero",
        "s": "has a square below the normals that rounds to a double, not to zero",
        "u": "has a square below the normals that rounds to zero"}


def make(case="nnnn", on_value=False, b=B_CODE, normal=True, eb_normal=True):
    st = dict(zip(TERMS, case))
    gaps = [a for a in AXES if st[a] != "z"]
    radius = st["r"] != "z"
    flat = [a for a in TERMS if st[a] in "zu"]
    live = [a for a in TERMS if st[a] in "ns"]
    small = [a for a in TERMS if st[a] in "su"]
    norm = [a for a in TERMS if st[a] == "n"]
    out = "value" if on_value else "y"
    sq = {a: f"s{a}" for a in AXES}
    sq["r"] = "rr"
    part = {a: f"X{a}" if st[a] == "s" else sq[a] for a in TERMS}

    src = [
        "# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66",
        "# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66",
        "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.",
    ]
    if on_value:
        src.append("# The claim is on value, the double on_floats returns, not on y.")
    if small:
        src.append("# A case product_is_normal refuses, which on_floats meets only without")
        src.append("# its check. Here:")
        for a in TERMS:
            if st[a] != "n":
                who = "the radius" if a == "r" else f"the {a} gap"
                src.append(f"#   {who} {SAYS[st[a]]};")
        src[-1] = src[-1][:-1] + "."
    elif case != "nnnn":
        names = [f"the {a} gap" for a in AXES if st[a] == "z"] + (["the radius"] if not radius else [])
        src.append(f"# The case where {' and '.join(names)} {'is' if len(names) == 1 else 'are'} zero.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre and the radius, each an f64. A gap is zero when",
        "# the centre's coordinate is the same double as the point's (464.11's",
        "# diffzero.g), and a zero radius is the double 0.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        " ".join(f"c{a} = rnd(c{a}_);" if a in gaps else f"c{a} = p{a};" for a in AXES),
        "r = rnd(r_);" if radius else "r = 0;",
        "",
        "# What on_floats computes, line for line. A zero, a square of one and a",
        "# sum of two round to themselves (464.11's refl.g), so they are left",
        "# unrounded.",
    ]
    if small:
        src += ["# A square below the normals is X. One that rounds to zero is written 0,",
                "# which tinysq.g proves rnd(X) is."]
    src.append(" ".join(f"G{a} = p{a} - c{a};" for a in AXES))
    src.append(" ".join(f"g{a} = rnd(G{a});" if a in gaps else f"g{a} = G{a};" for a in AXES))
    xs = [f"X{a} = g{a} * g{a};" for a in AXES if st[a] in "su"]
    if xs:
        src.append(" ".join(xs))
    square = {"n": "rnd(g{a} * g{a})", "z": "g{a} * g{a}", "s": "rnd(X{a})", "u": "0"}
    src.append(" ".join(f"s{a} = {square[st[a]].format(a=a)};" for a in AXES))
    src.append({"n": "rr = rnd(r * r);", "z": "rr = r * r;",
                "s": "Xr = r * r; rr = rnd(Xr);", "u": "Xr = r * r; rr = 0;"}[st["r"]])
    s01_rounds = not ("x" in flat and "y" in flat)
    s012_rounds = not all(a in flat for a in AXES)
    src += [
        "S01 = sx + sy;",
        "s01 = rnd(S01);" if s01_rounds else "s01 = S01;",
        "S012 = s01 + sz;",
        "s012 = rnd(S012);" if s012_rounds else "s012 = S012;",
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
    if small and norm:
        src += ["# The same with X in place of each square below the normals that does",
                "# not round to zero, and without those that do.",
                "Phatx = " + " + ".join(part[a] for a in live) + ";"]
    if norm:
        src += ["", "# What each rounding multiplied its argument by."]
        if gaps:
            src.append(" ".join(f"rg{a} = g{a} / G{a};" for a in gaps))
        if [a for a in AXES if st[a] == "n"]:
            src.append(" ".join(f"rp{a} = s{a} / (g{a} * g{a});" for a in AXES if st[a] == "n"))
        if st["r"] == "n":
            src.append("rpr = rr / (r * r);")
        rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rs012 = s012 / S012;"] if s012_rounds else [])
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
        if small:
            src += ["# For a square below the normals e and W are of X, and d is what",
                    "# rounding X cost, as an amount. One that rounds to zero is off by",
                    "# its whole true square, which is its d."]
        for a in AXES:
            if st[a] == "n":
                src.append(f"W{a} = {carry[a]} - 1 / (rg{a} * rg{a} * rp{a});")
            elif st[a] == "s":
                src.append(f"W{a} = {carry[a]} - 1 / (rg{a} * rg{a});")
        if st["r"] == "n":
            src.append(f"Wr = 1 / rpr - {carry['r']};")
        elif st["r"] == "s":
            src.append(f"Wr = 1 - {carry['r']};")
        src.append(" ".join(f"e{a} = {part[a]} * W{a};" for a in live))
        ds = []
        for a in AXES:
            if st[a] == "s":
                ds.append(f"d{a} = (s{a} - X{a}) * " + (f"({carry[a]});" if " " in carry[a] else f"{carry[a]};"))
            elif st[a] == "u":
                ds.append(f"d{a} = -(G{a} * G{a});")
        if st["r"] == "s":
            ds.append("dr = Xr - rr;" if carry["r"] == "1" else f"dr = (Xr - rr) * {carry['r']};")
        elif st["r"] == "u":
            ds.append("dr = Xr;")
        if ds:
            src.append(" ".join(ds))

    hyps = []
    finite = [f"|p{a}| <= 1b1024" for a in AXES] + [f"|c{a}| <= 1b1024" for a in gaps]
    if radius:
        finite.append("|r| <= 1b1024")
    hyps.append(("Every input is a finite f64.", [finite[:3], finite[3:]]))
    if gaps or radius:
        nonzero = [f"|g{a}| >= 1b-1074" for a in gaps] + (["|r| >= 1b-1074"] if radius else [])
        hyps.append(("No factor of the squares left is zero. For an f64 that is the same\n"
                     "  # as being at least the smallest subnormal.", [nonzero]))
        if normal and norm:
            hyps.append(("product_is_normal on the squares that pass it." if small else
                         "product_is_normal on the squares left.",
                         [[f"|{sq[a]}| >= 1b-1022" for a in norm]]))
    if small:
        rows = []
        for a in small:
            factor = "r" if a == "r" else f"g{a}"
            if st[a] == "s":
                rows.append([f"X{a} in [1b-2148, 1b-1022]", f"{sq[a]} >= 1b-1074", f"|{factor}| <= 1b-511"])
            else:
                rows.append([f"X{a} in [1b-2148, 1b-1075]", f"|{factor}| <= 1b-511"])
        hyps.append(("Each square below the normals. Of two doubles that are not zero it is\n"
                     "  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and\n"
                     "  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and\n"
                     "  # above, to at least 2^-1074.", rows))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = []
    if normal:
        lemma += [[f"s{a} -/ g{a} * g{a} {LEMMA}"] for a in AXES if st[a] == "n"]
        if st["r"] == "n":
            lemma.append([f"rr -/ r * r {LEMMA}"])
    if eb_normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if on_value and norm:
        hyps.append(("y is not zero. Where it is, value is zero too, and the claim on y is\n"
                     "  # the claim on value.", [["|y| >= 1b-1074"]]))

    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            if not row:
                continue
            src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
            first = False
    src.append("->")
    es = " + ".join(f"e{a}" for a in live)
    dsum = " + ".join(f"d{a}" for a in small)
    if not norm:
        src.append(f"  eb in ? /\\ ({out} - Ex) / eb in [-1, 1]")
    elif not small:
        src += [f"  ({out} - Ex) / Phat in ? /\\ P -/ Phat in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    else:
        src += [f"  ({es}) / Phatx in ? /\\ ({dsum}) in ? /\\ P in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    src += ["}", ""]

    if norm:
        src.append("# How far each rounding is from the identity.")
        for a in gaps:
            src.append(f"g{a} / G{a} -> 1 + (g{a} - G{a}) / G{a};")
        for a in AXES:
            if st[a] == "n":
                src.append(f"s{a} / (g{a} * g{a}) -> 1 + (s{a} - g{a} * g{a}) / (g{a} * g{a});")
        if st["r"] == "n":
            src.append("rr / (r * r) -> 1 + (rr - r * r) / (r * r);")
        if s01_rounds:
            src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
        if s012_rounds:
            src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
        if on_value:
            src.append("value / y -> 1 + (value - y) / y;")
        if not small:
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
        else:
            for a in AXES:
                if st[a] == "u":
                    src.append(f"G{a} * G{a} -> X{a} / (rg{a} * rg{a});")
            shifts = [f"(X{a} - {sq[a]})" for a in small if st[a] == "s"]
            shift = shifts[0] if len(shifts) == 1 else f"({' + '.join(shifts)})"
            src += [
                "",
                "# The relative part is averaged over Phatx as in skin.g. What rounding a",
                "# square below the normals cost is an amount, and eb being normal keeps",
                "# P large enough that it is small against eb.",
                f"({out} - Ex) / eb -> (({es}) / Phatx) * (Phatx / P) * (P / eb) + ({dsum}) / eb;",
                f"({es}) / Phatx -> ((Phatx + ({es})) - Phatx) / Phatx;",
                f"Phatx + ({es}) -> " + " + ".join(f"({part[a]} + e{a})" for a in live) + ";",
            ]
            for a in live:
                src.append(f"({part[a]} + e{a}) - {part[a]} -> e{a};")
            src += [
                f"Phatx / P -> Phat / P + {shift} / P;" if shifts else "Phatx / P -> Phat / P;",
                "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
                "P -> eb / (B * (1 + (eb - B * P) / (B * P)));",
            ]
    zh = []
    if "x" in flat and "y" not in flat:
        zh.append("sx + sy -> sy;")
    if "y" in flat and "x" not in flat:
        zh.append("sx + sy -> sx;")
    if "z" in flat:
        zh.append("s01 + sz -> s01;")
    elif "x" in flat and "y" in flat:
        zh.append("s01 + sz -> sz;")
    if "r" in flat:
        zh.append("s012 + rr -> s012;")
        if on_value:
            zh.append("s012 - rr -> s012;")
    elif not s012_rounds:
        zh.append("s012 + rr -> rr;")
        if on_value:
            zh.append("s012 - rr -> -rr;")
    if zh:
        src += ["", "# A zero square added to a double leaves that double, which then rounds",
                "# to itself."] + zh
    return "\n".join(src) + "\n"


def write(name, text):
    open(name, "w").write(text)


for case in map("".join, itertools.product("nzsu", repeat=4)):
    tag = "" if case == "nnnn" else f"_{case}"
    if "n" in case:
        for on_value in (False, True):
            name = f"skin{'v' if on_value else ''}{tag}"
            write(f"{name}.g", make(case, on_value))
            write(f"c_{name}_half.g", make(case, on_value, b=B_HALF))
    else:
        write(f"skin{tag}.g", make(case))
        write(f"c_skin{tag}_noeb.g", make(case, eb_normal=False))
write("c_skin_nonormal.g", make(normal=False))
write("c_skin_noeb.g", make(eb_normal=False))
```

`skin.g`, whole, as `skincases.py` writes it:

```gappa
# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66
# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.
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

B = 6b-53 + 3b-102;
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

`skin_snnu.g`, whole, the form of a case with a square below the normals:

```gappa
# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# restates: crates/rcad-geom/src/sphere.rs SIDE_OF_SPHERE_ERROR_BOUND at 05d44c66
# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 05d44c66
# restates: crates/rcad-predicates/src/cascade.rs decide at 05d44c66
# HowFarFromTheSkin::on_floats and SIDE_OF_SPHERE_ERROR_BOUND, rcad-geom's sphere.
# A case product_is_normal refuses, which on_floats meets only without
# its check. Here:
#   the x gap has a square below the normals that rounds to a double, not to zero;
#   the radius has a square below the normals that rounds to zero.
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
# A square below the normals is X. One that rounds to zero is written 0,
# which tinysq.g proves rnd(X) is.
Gx = px - cx; Gy = py - cy; Gz = pz - cz;
gx = rnd(Gx); gy = rnd(Gy); gz = rnd(Gz);
Xx = gx * gx;
sx = rnd(Xx); sy = rnd(gy * gy); sz = rnd(gz * gz);
Xr = r * r; rr = 0;
S01 = sx + sy;
s01 = rnd(S01);
S012 = s01 + sz;
s012 = rnd(S012);
y = s012 - rr;
value = rnd(y);

# The permanent sums the same three squares in the same order as value
# does, so its first two sums are s01 and s012.
P = rnd(s012 + rr);

B = 6b-53 + 3b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = Gx * Gx + Gy * Gy + Gz * Gz - r * r;

# The permanent with no rounding of its own.
Phat = sx + sy + sz + rr;
# The same with X in place of each square below the normals that does
# not round to zero, and without those that do.
Phatx = Xx + sy + sz;

# What each rounding multiplied its argument by.
rgx = gx / Gx; rgy = gy / Gy; rgz = gz / Gz;
rpy = sy / (gy * gy); rpz = sz / (gz * gz);
rs01 = s01 / S01; rs012 = s012 / S012;

# What each of the four terms, as y carries it, is off by: e is that
# amount and W the same over the term's square. The radius's term is
# subtracted, so its W is the other way round.
# For a square below the normals e and W are of X, and d is what
# rounding X cost, as an amount. One that rounds to zero is off by
# its whole true square, which is its d.
Wx = rs01 * rs012 - 1 / (rgx * rgx);
Wy = rs01 * rs012 - 1 / (rgy * rgy * rpy);
Wz = rs012 - 1 / (rgz * rgz * rpz);
ex = Xx * Wx; ey = sy * Wy; ez = sz * Wz;
dx = (sx - Xx) * (rs01 * rs012); dr = Xr;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |cx| <= 1b1024 /\ |cy| <= 1b1024 /\ |cz| <= 1b1024 /\ |r| <= 1b1024
  # No factor of the squares left is zero. For an f64 that is the same
  # as being at least the smallest subnormal.
  /\ |gx| >= 1b-1074 /\ |gy| >= 1b-1074 /\ |gz| >= 1b-1074 /\ |r| >= 1b-1074
  # product_is_normal on the squares that pass it.
  /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  # Each square below the normals. Of two doubles that are not zero it is
  # at least 2^-2148, and its factor is at most 2^-511. tinysq.g and
  # tinyzero.g split it at 2^-1075: at or below, it rounds to zero, and
  # above, to at least 2^-1074.
  /\ Xx in [1b-2148, 1b-1022] /\ sx >= 1b-1074 /\ |gx| <= 1b-511
  /\ Xr in [1b-2148, 1b-1075] /\ |r| <= 1b-511
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ sy -/ gy * gy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ gz * gz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
->
  (ex + ey + ez) / Phatx in ? /\ (dx + dr) in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
gx / Gx -> 1 + (gx - Gx) / Gx;
gy / Gy -> 1 + (gy - Gy) / Gy;
gz / Gz -> 1 + (gz - Gz) / Gz;
sy / (gy * gy) -> 1 + (sy - gy * gy) / (gy * gy);
sz / (gz * gz) -> 1 + (sz - gz * gz) / (gz * gz);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;

# The relative part is averaged over Phatx as in skin.g. What rounding a
# square below the normals cost is an amount, and eb being normal keeps
# P large enough that it is small against eb.
(y - Ex) / eb -> ((ex + ey + ez) / Phatx) * (Phatx / P) * (P / eb) + (dx + dr) / eb;
(ex + ey + ez) / Phatx -> ((Phatx + (ex + ey + ez)) - Phatx) / Phatx;
Phatx + (ex + ey + ez) -> (Xx + ex) + (sy + ey) + (sz + ez);
(Xx + ex) - Xx -> ex;
(sy + ey) - sy -> ey;
(sz + ez) - sz -> ez;
Phatx / P -> Phat / P + (Xx - sx) / P;
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));

# A zero square added to a double leaves that double, which then rounds
# to itself.
s012 + rr -> s012;
```

`sameas.py`, whole:

```python
"""Check that each script skincases.py writes for a case of n and z alone is
464.12's spherecases.py script for the same zero set, but for its first lines
and B. Run beside skincases.py's output, with spherecases.py's in ref/."""
import itertools
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
TERMS = "xyzr"


def strip(text, b):
    keep = [l for l in text.splitlines()
            if not l.startswith("# restates:") and not l.startswith("# HowFar")]
    assert keep.count(b) == 1, b
    return [("B" if l == b else l) for l in keep]


bad = 0
pairs = 0
for n in range(len(TERMS) + 1):
    for zero in itertools.combinations(TERMS, n):
        zero = "".join(zero)
        case = "".join("z" if a in zero else "n" for a in TERMS)
        for v in ("", "v"):
            for half in (False, True):
                if zero == "xyzr" and (v or half):
                    continue
                ref = f"sphere{v}{'_z' + zero if zero else ''}"
                new = f"skin{v}{'_' + case if case != 'nnnn' else ''}"
                if half:
                    ref, new = f"c_{ref}_half", f"c_{new}_half"
                a = strip(open(f"ref/{ref}.g").read(), "B = 1b-54;" if half else "B = 8b-53 + 1b-100;")
                b = strip(open(f"{new}.g").read(), "B = 1b-54;" if half else "B = 6b-53 + 3b-102;")
                pairs += 1
                if a != b:
                    bad += 1
                    print(f"differ: {ref}.g {new}.g")
for ref, new, b_ref, b_new in (("c_sphere_nonormal", "c_skin_nonormal", "B = 8b-53 + 1b-100;", "B = 6b-53 + 3b-102;"),
                               ("c_sphere_noeb", "c_skin_noeb", "B = 8b-53 + 1b-100;", "B = 6b-53 + 3b-102;"),
                               ("c_sphere_zxyzr_noeb", "c_skin_zzzz_noeb", "B = 8b-53 + 1b-100;", "B = 6b-53 + 3b-102;")):
    pairs += 1
    if strip(open(f"ref/{ref}.g").read(), b_ref) != strip(open(f"{new}.g").read(), b_new):
        bad += 1
        print(f"differ: {ref}.g {new}.g")
print(f"{pairs} pairs, {bad} differ")
sys.exit(1 if bad else 0)
```

`runall.sh`, whole:

```sh
#!/bin/bash
# Run every script skincases.py writes, 16 at a time: each skin*.g with coq
# after it, each c_*.g without, into run.log.
cd "$(dirname "$0")"
{ ls c_*.g; ls skin*.g | grep -v '_b[0-9]*_[0-9]*\.g$' | sed 's/$/ coq/'; } \
  | xargs -P 16 -L 1 bash one.sh > run.log 2>&1
echo done > runall.done
```

`collect.py`, whole:

```python
"""Read run.log and every skin*.out and c_*.out skincases.py's scripts wrote,
check each proof and control came out as it must, and group the cases by what
Gappa printed. Run as: python3 collect.py, beside the outputs."""
import collections
import glob
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))

log = open("run.log").read().splitlines()
runs = {}
coq = {}
for line in log:
    m = re.fullmatch(r"(\S+) exit (\d+) ([0-9.]+)s", line)
    if m:
        runs[m[1]] = (int(m[2]), float(m[3]))
        continue
    m = re.fullmatch(r"(\S+) coq -Bcoq exit (\d+) coqc exit (\d+) ([0-9.]+)s lines (\d+) hints (\d+)", line)
    if m:
        coq[m[1]] = (int(m[2]), int(m[3]), float(m[4]), int(m[5]), int(m[6]))
        continue
    print("unread line:", line)

proofs = sorted(n[:-2] for n in glob.glob("skin*.g") if not re.search(r"_b\d+_\d+\.g$", n))
controls = sorted(n[:-2] for n in glob.glob("c_*.g"))
print(f"{len(proofs)} proofs, {len(controls)} controls, {len(runs)} runs in run.log, {len(coq)} coq lines")
bad = [n for n in proofs if runs.get(n, (None,))[0] != 0]
print("proofs not exit 0:", bad)
bad = [n for n in proofs if coq.get(n, (None, None))[:2] != (0, 0)]
print("proofs without -Bcoq 0 and coqc 0:", bad)
bad = [n for n in controls if runs.get(n, (None,))[0] != 1]
print("controls not exit 1:", bad)

warn = collections.Counter()
for n in proofs + controls:
    for line in open(f"{n}.out").read().splitlines():
        if "assumed to be nonzero" in line:
            warn["assumed to be nonzero"] += 1
        elif line.startswith("Warning"):
            warn[line] += 1
print("warnings:", dict(warn))

t = [runs[n][1] for n in proofs if n in runs]
print(f"gappa on proofs {min(t):.2f} to {max(t):.2f} s")
t = [runs[n][1] for n in controls if n in runs]
print(f"gappa on controls {min(t):.2f} to {max(t):.2f} s")
t = [coq[n][2] for n in proofs if n in coq]
print(f"-Bcoq and coqc {min(t):.2f} to {max(t):.2f} s")
for n in ("skin", "skinv", "skin_zzzz", "skin_ssss", "skin_uuuu"):
    if n in coq:
        print(f"{n}.v: {coq[n][3]} lines, {coq[n][4]} hints")
lines = [(coq[n][3], coq[n][4], n) for n in proofs if n in coq]
print("smallest proof:", min(lines), "largest:", max(lines))
print("hints from", min(h for _, h, _ in lines), "to", max(h for _, h, _ in lines))


def results(n):
    text = open(f"{n}.out").read()
    out = []
    for line in text.splitlines():
        m = re.match(r"\s+(.*) in \[.*?\{([^,]+), [^}]+\}, .*?\{([^,]+), [^}]+\}\]$", line)
        if m:
            out.append(f"{m[1]} [{m[2]}, {m[3]}]")
        elif "in [" in line:
            out.append(line.strip())
        elif "pointless" in line:
            out.append("pointless")
        elif "best:" in line:
            out.append(line.strip())
    return out


groups = collections.defaultdict(list)
for n in proofs + controls:
    key = tuple(results(n))
    case = n.split("_")[1] if n.count("_") and n.split("_")[1] not in ("half", "noeb") else "nnnn"
    if n.startswith("c_"):
        parts = n.split("_")
        case = parts[2] if len(parts) > 3 else "nnnn"
    kind = ("c" if n.startswith("c_") else "") + ("v" if "skinv" in n else "y") + ("_noeb" if n.endswith("noeb") else "")
    groups[(kind, key)].append(n)
for (kind, key), names in sorted(groups.items()):
    print(f"\n{kind}: {len(names)}: {' '.join(names[:12])}{' ...' if len(names) > 12 else ''}")
    for k in key:
        print("   ", k)
```

Its print starts with these 15 lines. `c_skin_noeb` is the one control that
proves (7), and the 43108 warnings are the "assumed to be nonzero" lines of 8,
counted over every proof's and control's output:

```text
431 proofs, 433 controls, 864 runs in run.log, 431 coq lines
proofs not exit 0: []
proofs without -Bcoq 0 and coqc 0: []
controls not exit 1: ['c_skin_noeb']
warnings: {'assumed to be nonzero': 43108}
gappa on proofs 0.07 to 48.00 s
gappa on controls 0.08 to 155.79 s
-Bcoq and coqc 1.20 to 54.56 s
skin.v: 4047 lines, 18 hints
skinv.v: 4186 lines, 19 hints
skin_zzzz.v: 366 lines, 2 hints
skin_ssss.v: 567 lines, 0 hints
skin_uuuu.v: 301 lines, 2 hints
smallest proof: (301, 2, 'skin_uuuu') largest: (6111, 18, 'skin_snus')
hints from 0 to 21
```

The rest of its print, 117054 bytes in all, is each group of outputs that
print the same results, with the names in it and the results, and is not
appended here.

`brief.py`, whole:

```python
"""One line per group of summarize.py's: for each kind and number of normal
squares, with or without a square below the normals, the distinct relative
bounds, the least P, the widest d, the distinct bests and the run count.
Run as: python3 brief.py, beside the outputs."""
import collections
import glob
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def case_of(name):
    m = re.search(r"_([nzsu]{4})(_|$)", name)
    return m[1] if m else "nnnn"


names = sorted(n[:-2] for n in glob.glob("skin*.g") + glob.glob("c_*.g") if not re.search(r"_b\d+_\d+\.g$", n))
g = collections.defaultdict(lambda: {"n": 0, "rel": set(), "pp": set(), "P": set(), "d": set(), "best": set(), "other": set()})
for n in names:
    case = case_of(n)
    kind = ("c_" if n.startswith("c_") else "") + ("v" if "skinv" in n else "y") + ("_noeb" if n.endswith("_noeb") else "") + ("_nonormal" if "nonormal" in n else "")
    small = "small" if set(case) & set("su") else "nz"
    key = (kind, small, case.count("n"))
    e = g[key]
    e["n"] += 1
    for line in open(f"{n}.out").read().splitlines():
        m = re.match(r"\s+(.*) in \[.*?\{([^,]+), ([^}]+)\}, .*?\{([^,]+), ([^}]+)\}\]$", line)
        if m:
            what = m[1]
            if what == "P":
                e["P"].add(m[3])
            elif what == "P -/ Phat":
                e["pp"].add(m[4])
            elif "Ex" in what or "Phatx" in what or what.startswith("e") and "/" in what:
                e["rel"].add(m[4])
            elif what.startswith("d") or what == "r * r" or what.endswith("r * r"):
                e["d"].add(f"{m[3]}..{m[5]}")
            else:
                e["other"].add(f"{what} {m[2]}..{m[4]}")
        elif "in [0, 0]" in line:
            e["other"].add(line.strip())
        elif "pointless" in line:
            e["other"].add("pointless")
        elif "best:" in line:
            e["best"].add(line.strip().split("best: ")[1])
        elif "in [" in line:
            e["other"].add(line.strip()[:80])
for key in sorted(g):
    e = g[key]
    parts = [f"{key[0]} {key[1]} {key[2]}n: {e['n']}"]
    for k in ("rel", "pp", "P", "d", "best", "other"):
        if e[k]:
            parts.append(f"{k}={sorted(e[k])}")
    print("  ".join(parts))
```

Its print is 34 lines, one for each kind of script, with or without a square
below the normals, and count of `n` squares, and its longest is 533
characters, so it is given here in words and not appended. 6 reads its counts,
relative bounds, least `P` and widest `d` from it, such as `y small 1n: 104`
with `P` at least `2^(-971.585)`, and `y small 0n: 80`, each "pointless"; 7
reads the controls' bests from it.

`summarize.py`, whole:

```python
"""Group what each of skincases.py's proofs and controls printed by how many of
its four squares are n, z, s and u, and print each group's distinct prints.
Run as: python3 summarize.py, beside the outputs, after collect.py."""
import collections
import glob
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def case_of(name):
    m = re.search(r"_([nzsu]{4})(_|$)", name)
    return m[1] if m else "nnnn"


def prints(name):
    out = []
    for line in open(f"{name}.out").read().splitlines():
        m = re.match(r"\s+(.*) in \[.*?\{([^,]+), [^}]+\}, .*?\{([^,]+), [^}]+\}\]$", line)
        if m:
            what = m[1]
            what = re.sub(r"^\(e[xyzr]( \+ e[xyzr])*\) / Phatx$", "(e) / Phatx", what)
            what = re.sub(r"^d[xyzr].*|^r \* r$", "d", what) if what != "P" else what
            if what in ("P",):
                out.append(f"P from {m[2]}")
            else:
                out.append(f"{what} [{m[2]}, {m[3]}]")
        elif "in [" in line:
            out.append(line.strip())
        elif "pointless" in line:
            out.append("pointless")
        elif "best:" in line:
            out.append(line.strip().split("best: ")[1])
    return tuple(out)


names = sorted(n[:-2] for n in glob.glob("skin*.g") + glob.glob("c_*.g") if not re.search(r"_b\d+_\d+\.g$", n))
groups = collections.defaultdict(lambda: collections.defaultdict(list))
for n in names:
    case = case_of(n)
    counts = "".join(f"{case.count(k)}{k}" for k in "nzsu")
    kind = ("c_" if n.startswith("c_") else "") + ("v" if "skinv" in n else "y") + ("_noeb" if n.endswith("_noeb") else "") + ("_nonormal" if "nonormal" in n else "")
    groups[(kind, counts)][prints(n)].append(n)
for (kind, counts), by in sorted(groups.items()):
    total = sum(len(v) for v in by.values())
    print(f"{kind} {counts}: {total}")
    for p, ns in by.items():
        print(f"    {len(ns)}x {' | '.join(p)}   e.g. {ns[0]}")
```

Its print, 380 lines and 35435 bytes, is each group's distinct prints, and is
not appended here.

`pairs.py`, whole:

```python
"""Check each control at u / 2 fails with a best that is its proof's relative
bound over u / 2, and list every line of any output or -Bcoq print that is not
a result, a best or an 'assumed to be nonzero' warning. Run as: python3
pairs.py, beside the outputs."""
import glob
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))

U = 2.0 ** -53
bad = 0
count = 0
for c in sorted(glob.glob("c_*_half.g")):
    name = c[2:-len("_half.g")]
    rel = None
    for line in open(f"{name}.out").read().splitlines():
        m = re.match(r"\s+(.*) in \[.*?\{([^,]+), [^}]+\}, .*?\{([^,]+), [^}]+\}\]$", line)
        if m and ("Ex" in m[1] or "Phatx" in m[1] or "/ (s" in m[1] or "/ float" in m[1]):
            rel = float(m[3])
    best = re.search(r"best: \[(-?[0-9.e+]+), ([0-9.e+]+)\]", open(f"c_{name}_half.out").read())
    count += 1
    want = round(rel / (U / 2))
    if best is None or float(best[2]) != want:
        bad += 1
        print(f"c_{name}_half: best {best[0] if best else None}, relative bound {rel} is {want} of u/2")
print(f"{count} controls at u/2, {bad} whose best is not the relative bound over u/2")

odd = set()
for f in glob.glob("*.out") + glob.glob("*.bcoq"):
    if re.search(r"_b\d+_\d+\.", f):
        continue
    for line in open(f).read().splitlines():
        if ("assumed to be nonzero" in line or line.startswith("Results:") or line.startswith("  ")
                or line.startswith("Error: some properties were not satisfied")):
            continue
        odd.add(line[:120])
print("other lines:", sorted(odd))
for n in ("skin", "skinv"):
    print(n, "-Bcoq nonzero warnings:", open(f"{n}.bcoq").read().count("assumed to be nonzero"))
for f in sorted(glob.glob("*.coqc")):
    text = open(f).read().strip()
    if text:
        odd.add(f)
        print("coqc printed something:", f, text[:200])
```

It prints:

```text
350 controls at u/2, 0 whose best is not the relative bound over u/2
other lines: ['Warning: no need for the rewriting at line 9 column 17']
skin -Bcoq nonzero warnings: 59
skinv -Bcoq nonzero warnings: 66
```

`bisect.py`, whole, as run. It is 464.13's with one line changed, `B_CODE`,
and as there its `best` line is never read:

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
B_CODE = "B = 6b-53 + 3b-102;"
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

`bisects.sh`, whole:

```sh
#!/bin/bash
# Run the two bisections of 464.14's 3 and 4 side by side, each into its own
# log, then write bisects.done.
cd "$(dirname "$0")"
rm -f bisects.done
python3 bisect.py skin.g 5 48 > bisect_skin.log 2>&1 &
python3 bisect.py skinv.g 6 48 > bisect_skinv.log 2>&1 &
wait
echo done > bisects.done
```

`skin.g`'s run printed, in order, `5u` failing, `5u + 48u²` proving, `24`
failing, `36` and `30` proving, `27` failing and `28` proving, and ended
`smallest: 5u + 28u^2 proves, 5u + 27u^2 fails`. `skinv.g`'s printed `6u`
failing, `6u + 48u²` proving, `24` failing, `36` and `30` proving, and `27`,
`28` and `29` failing, and ended `smallest: 6u + 30u^2 proves, 6u + 29u^2
fails`. Each failure was at best `[-1, 1]`.

`one.sh` is 464.13's, whole in its 9.

**10. What this leaves.** The rows filed from this write-up while it was
written:

- Q2765: restate `sphere.rs:416-418` by 5. What the `48ε²` has to beat is
  `ε²`, not `ε`, and by the comment's own terms `15 + 18 + 6` make 39.
- Q2766, after it: restate `sphere.rs:248-250` and `420-424` by 6, which
  proves the bound at the code's constant with any of the four squares below
  the normals while one is normal, and finds `eb` below the normals when none
  is. Then say whether `on_floats` needs its four `product_is_normal` checks.
- Q2767: the doc of `product_is_normal` (`cascade.rs:46-49`) says that a
  product below the normals fed into a relative bound makes the filter certify
  a sign it has not earned, sometimes the wrong one. 464.8's 4 and 6 here
  found that false for their predicates while one product stays normal. That
  the doc holds for some other predicate is a guess nobody has run, so the
  row has no class (§360.5). It waits on Q2684, Q2760 and Q2766.

And what is not a row:

- Q2700 already waits on Q2679, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- Q2518 (refuses/D) is already open. By 6 it is a case with no normal square,
  so `eb` is not normal, and level 1 cannot decide it with the check or
  without it.
- How `c_skin_noeb.g` gets past `eb`'s rounding (7) was not traced, as with
  464.12's `c_sphere_noeb.g`.
- `skin.g`'s first `restates` line needs no row, unlike `flat.g`'s in 464.13's
  10: `just src on_floats crates/rcad-geom/src/sphere.rs` prints the one item,
  at `sphere.rs:241`.
- `field` on these scripts' hints (464.10).

### 464.15 The side of rcad-geom's cylinder

Q2680 took up `SIDE_OF_CYLINDER_ERROR_BOUND` in
`crates/rcad-geom/src/cylinder.rs`, in ten lanes on 2026-10-25. The first
four were the first lane and its three relays. The first wrote `cyl.g`. The
second split its hint, wrote `cylcases.py` and ran `run1.sh`. The third ran
the bisection from 61 to 69, wrote `m2.g`, `m3.g`, `vgen.py` and
`cylsplit.py`, and ran `run2.sh`. The fourth wrote `dbl.g` and ran `run3.sh`.
The fifth to the eighth were the second lane and its three relays: they
merged the first lane's branch, fixed `tally3.py`'s block, ran `run4.sh`,
fixed `same.py`, checked every figure and drafted 1 to 8. The ninth and the
tenth were the third lane and its relay: they merged the second lane's branch
and wrote 1 to 10. The bound is proved on `y`, the sum before `value`'s last
rounding, at the code's constant wherever no factor is zero: with no component
of the cross product cancelling, and with any of them cancelling, whatever its
reach square rounds to. Coq checks each proof, modulo its hints. The claim on
`value` itself is not proved at the code's constant in either form tried, only
at `10u + 82u²`, and `decide` does not need it (4). The branches where a
difference, a direction component or the radius is zero, which
`product_is_normal` lets through, are not run. And the comments claim more
than is proved. The doc comment on the constant writes `45ε` where it means
`45ε²` and miscounts its own terms. The comment on the check says that
checking the components covers the permanent's squares, which is false where a
component cancels, though the bound holds there (5). Fixing them, and running
what is not run, is three rows (10).

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, against
`cylinder.rs` and `cascade.rs` at `2a7ca9e7`. `git log --oneline
2a7ca9e76..8e87e43e6` on `cylinder.rs`, `cascade.rs`, `three_surfaces.rs`,
`validate.rs` and `rcad-geom`'s `lib.rs` prints nothing, so each is as it
stands at `8e87e43e6`, where this write-up was finished. `u` is `2⁻⁵³`, and the
code's `B = (10 + 80ε) ε` (`cylinder.rs:559-562`) is `10u + 80u²`, written `B
= 10b-53 + 5b-102;`. `80ε` is `5 · 2⁻⁴⁹`, a multiple of the ulp of 10, so the
Rust's sum and product are exact and a double holds `B`. Each script follows
464.8's pattern: one ratio hint per rounding; the error split into one term
per piece of the permanent, so that Gappa averages them; 464.5's lemma for
each product `on_floats` checks is normal; and the step from `y` to the sign
as 464.4 gives it.

**1. What the proof guards.** `Cylinder::side_of` (`cylinder.rs:272-286`)
runs the cascade on `HowFarFromTheSkin`, once a guard has refused a point or a
cylinder that is not real numbers. `Surface::Cylinder` reaches it from
`three_surfaces.rs:3295` and from `validate.rs:4498`, and `lib.rs:96` exports
`Cylinder`, whose `side_of` is `rcad-predicates`' `SideOfSurface` trait. So,
like 464.14's, this proof guards a predicate a user reaches.

**2. `on_floats`, line for line.** `HowFarFromTheSkin::on_floats`
(`cylinder.rs:300-349`) returns `None` for a point that is not exact `f64`s
(`cylinder.rs:301`) and takes the three differences from the axis origin to
the point (`cylinder.rs:305`). It forms the six halves of their cross product
with the direction, returning `None` unless `product_is_normal` holds for each
half's two factors (`cross_product_halves_if_normal`, `cylinder.rs:467-480`,
around
`cross_product_halves`, `cylinder.rs:486-495`). It then takes the three
components `h − k` and the direction's squared length
(`cylinder.rs:316-318`), and returns `None` unless `product_is_normal` holds
for each component with itself, each direction component with itself, the
radius with itself, and `rr` with the squared length (`cylinder.rs:319-326`).
Last it returns `value = s0 + s1 + s2 − skin` and `error_bound =
SIDE_OF_CYLINDER_ERROR_BOUND * permanent` (`cylinder.rs:345-348`), where `skin
= rr · D` (`cylinder.rs:331`), the squares are the components'
(`cylinder.rs:332`), and the permanent is each component's reach squared,
`(|h| + |k|)²`, summed with `skin` (`cylinder.rs:339-343`).

`cyl.g` (9) writes that operation for operation: three differences `f =
rnd(point − origin)`; six halves `h`, `k`, each a rounded product; three
components `q = rnd(h − k)`; three squares `s = rnd(q · q)`, summed left to
right into `s012`; the direction's three squares `t = rnd(d · d)`, summed left
to right into `D`; `rr = rnd(r · r)`; `skin = rnd(rr · D)`; and `value =
rnd(s012 − skin)`. The permanent squares each component's reach, `v = rnd(n ·
n)` with `n = rnd(|h| + |k|)`, and sums `rnd(rnd(rnd(vx + vy) + vz) + skin)`.
On the longest chain that is ten roundings to `value`, `f`, `h` and `q`
counted twice since the component is squared, and nine to `y`.

`product_is_normal` is checked on everything but the three reach squares `v`.
The comment at `cylinder.rs:312-314` argues them from the components', since a
reach is never below its component. That holds while the component is not
zero. A component that cancels to zero, `h == k` with both normal, passes the
check, since `product_is_normal(a, b)` is true when `a` or `b` is zero
(`cascade.rs:67-69`). It leaves `v = rnd(4h²)`, which can be below the
normals. That is the kind of product 464.8's `o2s.g` splits on, and 6 splits
on it. `cylsplit.py` (9) writes the case with no component cancelling,
`cyl_nnn.g`, byte for byte as `cyl.g`, which `same.py` (9) checks.

**How the error splits.** The square `y` carries for one component is `(h −
k)² K`, `K` the product of the roundings of `q`, `s` and the sums after it.
The true one is `(h a − k b)²`, `a` and `b` undoing the roundings of `f` and
of `h` and `k`. Their difference is `h²(K − a²) + k²(K − b²) − 2hk(K − ab)`,
three terms, one for each piece of the reach squared, `h²`, `k²` and `2|h||k|`.
The radius term is three pieces that sum to `skin`, one for each direction
square, each true piece `r² d²`. So the error is averaged over twelve pieces.

**3. The bound on `y`.** `cyl.g` (9) proves `|y − Ex| ≤ eb` at the code's
`B`. In `run1.sh` (9), 24 Gappa runs at once on a load of about 30 over 24
cores, it exits 0 in `213.82s`, printing `(y − Ex) / P12` within
`9.99201e-16` and `P −/ Phat` within `6.66134e-16`, `9u` and `6u` to the
digits Gappa prints. `Phat` is the permanent with no rounding of its own, and
`P12` is the same quantity written as its twelve pieces. The smallest `B` it
proves, over `m` in `B = 9u + m u²`, is `9u + 70u²`. `run1.sh` ran `m` of 0,
40, 60, 70, 80, 90, 100, 120 and 160: 0, 40 and 60 fail, best `[-1, 1]` as
Gappa prints it, rounded, and the rest prove. `par.sh` (9) then ran the nine
`b_cyl_<m>.g` that `python3 cylcases.py 61 62 63 64 65 66 67 68 69` writes,
and each exits 1, best `[-1, 1]`, printing the same `9u` and `6u`.

How `cyl.g` got its form. The first lane's `cyl.g`, with every product normal
and nothing zero and all twelve pieces in one hint, did not finish: `one.sh`
(464.13's 9, the same bytes) printed `cyl exit 124 620.51s`, Gappa killed at
its 600 s timeout, on a load of about 24 over 24 cores. The second of the nine
lanes found where the time goes. With the goal replaced by `px in ?` and every
hint kept, Gappa still did not finish in 90 s (`t_triv exit 124 94.80s`), and
with that goal and the twelve-piece hint dropped it finished in `16.64s`. So
what is slow is checking that one hint's identity, over twelve pieces at once,
before any goal is tried. `t_axis.g` (9), one component alone with its three
pieces, proves in `0.58s` and prints the component's square within
`7.77156e-16`, `7u`. The permanent alone (`t_perm2.g` (9): six normal halves
and a normal `skin` as free doubles) proves in `0.22s`, printing `P −/ Phat`
within `6.66134e-16`, `6u`. `cyl.g` as it stands is the first lane's with that
hint split in four, one per block: the three components' reaches squared and
the radius term, with one hint over the blocks and one over each block's
pieces. It also drops three hints `eG / cG -> WG` that Gappa warned it did not
need. Its first form, the same but with those three hints kept, exits 0 in
`94.33s` on a load of about 30 over 24 cores, printing the same two figures
and proving `(y − Ex) / eb` in `[−1, 1]` at the code's `B`.

**4. The claim on `value`.** Two forms were tried, and neither proves at the
code's `B`.

- `cylv.g`, which `cylcases.py` (9) writes from `cyl.g` with `rv = value / y`
  on every carry, exits 124 at `run1.sh`'s 600 s. With `T=1800` in `run2.sh`
  (9) it exits 1 in `541.97s`, best `[-1, 1]`, printing `(value − Ex) / P12`
  within `1.11022e-15`, `10u`, and the same `6u`. `t_vtriv.g` (`vgen.py`, 9),
  `cylv.g` with its goal replaced by `px in ?` and every hint kept, exits 0 in
  `334.59s` on a load of about 29 over 24 cores. So in this form Gappa spends
  335 s checking the hints before it tries any goal. Which hint costs that was
  not split.
- `cylv2.g` keeps `cyl.g`'s blocks for `y` and takes `value`'s own rounding as
  a fraction of `y`, and `y` as a fraction of the permanent: `(value − Ex) /
  eb -> (y − Ex) / eb + (value − y) / eb;` and `(value − y) / eb -> ((value −
  y) / y) · (y / P12) · (Phat / P) · (P / eb);`, with `y / P12` averaged over
  the same four blocks. Its first form exits 1 in `131.20s`, printing `(value
  − y) / y` within `1b-53` and the same `9u` and `6u`, but `y / P12` only in
  `[-3b5118, 3b5122]`. `m2.g` (9), one component and `skin` alone, shows why:
  the piece `K h²` over `h²` reaches `−1` once Gappa takes `(h / |h|)²` as
  `[−1, 1]`, and with a relative error that reaches `−1` Gappa does not
  average a sum. `m3.g` (9), the same with every piece doubled so that each
  such ratio is at most about a half, prints `y / P12` within `[-1, 1]` as
  Gappa rounds it, in `0.22s`. `vgen.py` writes `cylv2.g` in that doubled
  form. It exits 1 at the code's `B` in `332.03s`, best `[-1, 1]`, printing `y
  / P12` within `[-1, 1]`, `(value − y) / y` within `1b-53` and the same `9u`
  and `6u`. Over `m` in `B = 10u + m u²`, 81 fails (`b_cylv2_81 exit 1
  542.42s`, best `[-1, 1]`), and 82 (`447.87s`), 83 (`447.09s`), 84, 88, 96,
  112 and 144 (182.63 to 186.20 s each) prove. So the doubled form proves
  `|value − Ex| ≤ eb` at `10u + 82u²`, two `u²` above the code's `10u +
  80u²`.

The third lane had worked that out on paper before the run: the doubled form
costs `u (1 + 5u)` of the permanent on top of `y`'s `9u + (63 + c) u²`, and
with `(Phat / P) (P / eb) ≈ (1 + 7u) / B` and `c` between 6 and 7 by the
bisection of 3, that is about `10u + 82u²`. So these forms lose `u²`s, and it
is not shown that `value` is ever past the bound at the code's `B`: nobody
searched for an input where it is. `decide` does not need it: the doc on
`Filtered::error_bound` (`cascade.rs:25-35`) asks only for the bound on the
sum before `value`'s last rounding, which 3 proves, as in 464.14's 4.

**5. What the comments claim.** The doc comment on the constant,
`cylinder.rs:529-558`, counts ten roundings on the longest chain to `value`
and six on the radius term's, and then says:

> `(1 + epsilon)^10 - 1 <= 10*epsilon + 45*epsilon^2 + ...` times the
> permanent.
>
> [...]
>
> The `80*epsilon^2` covers the higher-order terms and the rounding of the
> permanent itself; what it has to beat is about `45*epsilon`.
>
> Every step of that argument is a *relative* one, so as with every bound of
> this shape it holds only while nothing overflows or falls into the
> subnormals. [`HowFarFromTheSkin::on_floats`] enforces the bottom end product
> by product through [`product_is_normal`], and `decide` the top end by
> refusing a bound that is not itself a normal number.

The comment above the check, at `cylinder.rs:308-314`, says:

> The bound below is a fraction of the permanent, which only bounds anything
> while every product keeps its relative accuracy. One of them in the
> subnormals and this level certifies a wrong sign.
>
> The permanent squares `|left| + |right|` where the value squares `left -
> right`, and the first is never below the second, so checking the components
> here covers the permanent's squares too.

Against what is proved:

- The ten roundings to `value` agree with `cylv.g`'s `10u` (4). On `y` the
  count is nine, and `cyl.g` prints `9u` (3).
- "About `45*epsilon`" is the `ε²` term of `(1 + ε)¹⁰ − 1`, and it leaves out
  what the same sentence says the `80ε²` covers. The permanent's rounding,
  `6u` (`cyl.g`, `t_perm2.g`), taken against `10ε` costs `60ε²`. The rounding
  of `B · P` costs `ε` of `eb`, which is `10ε²`. So by the comment's own terms
  the count is `45 + 60 + 10 = 115`, above its `80`. On `y` that costs
  nothing, since `y` proves at `9u + 70u²`, about a `u` below `10u`. On
  `value` the doubled form proves only at `82`, and the count says nothing
  either way at `80`. So the sentence gives the wrong power and the wrong
  number.
- "Only while nothing ... falls into the subnormals", enforced "product by
  product": `product_is_normal` lets a zero through (`cascade.rs:67-69`), and
  the branches where a difference, a direction component or the radius is
  zero are not run (10). A cancelling component gets through too, and its
  reach square, which nothing checks, can then fall below the normals or
  round to zero, and the bound holds there (6). So the reach squares do not
  need what "only" says.
- "One of them in the subnormals and this level certifies a wrong sign" is
  false for the reach squares: by 6, one, two or three of them below the
  normals or rounding to zero, and the bound still holds. For the products
  `on_floats` checks it was not run, as 464.14's 6 ran the sphere's squares
  (10).
- "Checking the components here covers the permanent's squares too" is true
  where the component does not cancel, and `cyl.g` writes the argument beside
  its `v` hypothesis. It is false where the component cancels: its square is
  0, which the check passes, and `v = rnd(4h²)` can be below the normals with
  nothing checking it. The bound holds anyway (6).

So the comments claim more than is proved, which is what the row asks.
Restating them is a row (10).

**6. The components that cancel.** As 464.8's `o2s.g` split on the product
its predicate does not check, `cylsplit.py` (9) splits `cyl.g` on the reach
squares `v` (2). Each component of the cross product is in one of four
states:

- `n`: it does not cancel. `q = rnd(h − k)` is not zero, so its square `s` is
  normal, which `on_floats` checks, and so is its reach square `v`, which is
  never below `s`.
- `c`: it cancels to zero, `h == k`, and its reach square `v = rnd(4h²)` is
  normal.
- `s`: it cancels, and `v` is below the normals and does not round to zero.
- `u`: it cancels, and `v` rounds to zero.

The four cover every component whose halves are normal, which
`cross_product_halves_if_normal` checks: `q` is zero exactly when `h == k`,
and for `n`, `|q| ≤ n`, so `v ≥ s`. That makes 64 cases, written
`cyl_<case>.g`, and `cyl_nnn.g` is `cyl.g`. A component that cancels writes
`k` as `h`, drops `Q`, `q`, `s` and their ratios, writes each sum without its
zero squares, writes its reach `|h| + |h|` unrounded, and is one block off by
`eC = −C²`, with `eC / P3 -> −((h / |h|)² (a − b)² / 4)`. A reach square in
`s` or `u` takes 464.14's step through `Phatx`: `Phat` with `v` in place of
each `s` reach square and without each `u` one, added as `P` adds.

The unrounded reach stands on a double added to itself rounding to itself,
and `dbl.g` (9) shows that: it exits 0 in `0.03s`. Overflow is outside it:
`2|h|` overflows only when `|h| ≥ 2¹⁰²³`, and then `v`, `P` and `eb` are
infinite and `decide` refuses `eb` as not normal, which is 464.12's 7
argument on paper and was not run here.

What they print. `run3.sh` (9) ran the 63 cases other than `nnn` on a load of
about 47 over 24 cores, and `tally3.py` (9) counts them:

- Each of the 63 exits 0 at the code's `B`, in 65.74 s (`cyl_ucc.g`) to
  190.10 s (`cyl_snn.g`).
- Each prints `(y − Ex) / P12` within `8.88178e-16` in the 9 cases with two
  components `n`, `7.77156e-16` in the 27 with one and `5.55112e-16` in the
  27 with none: `8u`, `7u` and `5u`, all below `cyl.g`'s `9u`.
- `P −/ Phat`, or `P −/ Phatx` where a reach square is `s` or `u`, is within
  `6u` in 14 cases, `5u` in 19, `4u` in 9, `3u` in 10, `2u` in 3 and `u` in 7.
  `cyl_uuu.g`, whose `P` is `skin`, prints `skin` against itself in `[0, 0]`.
- No `.out`, `.bcoq` or `.coqc` of the 63 has a line other than its results
  and "assumed to be nonzero" warnings: no "not trivially equal" and no hint
  Gappa calls unneeded.

So on `y` the bound holds at the code's constant wherever no factor is zero:
with no component cancelling, and with any of them cancelling, its reach
square normal, below the normals, or rounding to zero. The comment at
`cylinder.rs:312-314` is false when a component cancels, and nothing wrong
follows from it.

The shapes. None of these cases was run as Rust input. By hand arithmetic,
not run: with the axis origin at 0, the direction `(1, 2, 3)` and the point
`(2, 4, 6)`, a point on the axis, the halves are `(12, 12)`, `(6, 6)` and
`(4, 4)`, each product exact, so that is `ccc`. With the direction `(1, 1,
1)` and the point `(1, 2, 2)`, `x`'s halves `(2, 2)` cancel, `y`'s are `(2,
1)` and `z`'s `(1, 2)`, so that is `cnn`. Both are shapes a designer makes. A
reach square below the normals needs `|h| < 2⁻⁵¹²`, since `rnd(4h²) <
2⁻¹⁰²²`. Every case proves, so there is no wrong sign to look for.

**7. The controls.** At `B = 1b-54;`, `u / 2`, each must fail, and does:

- `c_cyl_half.g` exits 1, best `[-18, 18]`, `cyl.g`'s `9u` over `u / 2`.
- `c_cylv_half.g` exits 1 in `379.28s` under `T=1800`, best `[-20, 20]`, and
  `c_cylv2_half.g` exits 1, best `[-20, 20]`: the `10u` of the claim on
  `value` over `u / 2`.
- The 63 `c_cyl_<case>_half.g` exit 1, best `[-16, 16]` in 9, `[-14, 14]` in
  27 and `[-10, 10]` in 27, each case's own bound over `u / 2`.

Together with the failures at `9u + 69u²` on `y` and at `10u + 81u²` on
`value`, that says none of the proofs came from a contradiction in its
hypotheses. Two more controls take a hypothesis away:

- `c_cyl_nonormal.g`, `cyl.g` without every `1b-1022` hypothesis but `|eb|`'s
  and without the lemma line each of those products gives, the reach squares'
  among them, exits 1, best `[-1.16728e+1527, 1.16728e+1527]`, and fails
  `REL(P, Phat)` and `BND((y - Ex) / P12)` too. So a script of this kind catches a
  missing check, as 464.14's 7 found.
- `c_cyl_noeb.g`, `cyl.g` without `eb` normal, exits 124 in `600.84s`. Where
  it spends its time was not traced.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each script that proves and was run through them, and no `coqc` printed
anything:

- `cyl.g`: 14703 lines with 54 hints, in `339.29s` (`run1.sh`).
- `dbl.g`: 217 lines with 1 hint, in `12.62s` (`run3.sh`).
- The 63 cases of 6: 8908 (`cyl_uuc`) to 13908 lines (`cyl_nns`), with 31
  (`cyl_ccc`) to 48 hints, in 145.5 to 194.2 s each (`run3.sh`).
- `b_cylv2_82.g`: 19054 lines with 81 hints, in `105.45s` (`run4.sh`, each
  step's timeout at 1500 s, since the Gappa run alone took 447.87 s).

Gappa does not write the same proof each run (464.4), so the sizes are those
runs'. Each hint enters the Coq file as a hypothesis with no condition
attached, as in 464.4, so each proof stands on its hints and on what Gappa
took to be nonzero: `cyl.g`'s `gappa -Bcoq` prints 292 lines of the form "has
been assumed to be nonzero when checking a rewriting rule" and, besides them,
only its results. `cylv.g`, the controls and the other bisection scripts were
not run through `coqc`. `field` has not been run on these hints (464.10).

**9. The scripts.** `cylcases.py` writes `cylv.g`, the controls of 7 and the
bisection scripts of 3 from `cyl.g`; `vgen.py` writes the second form of 4
and its probes; and `cylsplit.py` writes the 63 cases of 6 and their
controls. To run them again, save the nix block of 464.5 as `gappalib18.nix`
and `cyl.g` and the three generators beside it, and then:

```sh
python3 cylcases.py; python3 cylsplit.py; python3 vgen.py
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 cyl.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq cyl.g > cyl.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc cyl.v
```

and the same for every other file they write, and for `dbl.g`. A script that
proves exits 0; a control must exit 1. What the lanes ran is `run1.sh` to
`run4.sh` and `par.sh` (below), each through 464.13's `one.sh`, the same bytes,
or through `one1.sh`, which is `one.sh` with the timeout taken from `T` and no
`coq`. `same.py` (below) checks every script appended here byte for byte
against the file that was run, and that the three generators, run again from
the text here, write what was run; its print ends this part. Three files the
lanes wrote are not appended, since no figure here comes from them:
`cyl_first.g`, the first lane's `cyl.g` with one hint over all twelve pieces;
`t_triv.g`, `cyl_first.g` with its goal replaced by `px in ?`; and `m1.g`, an
eight-line probe of the hint `((G - t) - G) / G`.

`cyl.g`, whole, as the second of the nine lanes split it and every later run
ran it:

```gappa
# restates: crates/rcad-geom/src/cylinder.rs on_floats at 2a7ca9e7
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at 2a7ca9e7
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at 2a7ca9e7
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at 2a7ca9e7
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 2a7ca9e7
# restates: crates/rcad-predicates/src/cascade.rs decide at 2a7ca9e7
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
r = rnd(r_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
hx = rnd(fy * dz); kx = rnd(fz * dy);
hy = rnd(fz * dx); ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = rnd(fy * dx);
Qx = hx - kx; Qy = hy - ky; Qz = hz - kz;
qx = rnd(Qx); qy = rnd(Qy); qz = rnd(Qz);

# direction_squared, left to right as Rust adds it.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; D = rnd(T);

rr = rnd(r * r);
skin = rnd(rr * D);
sx = rnd(qx * qx); sy = rnd(qy * qy); sz = rnd(qz * qz);
S01 = sx + sy; s01 = rnd(S01);
S012 = s01 + sz; s012 = rnd(S012);
y = s012 - skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
nx = rnd(|hx| + |kx|); ny = rnd(|hy| + |ky|); nz = rnd(|hz| + |kz|);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
P = rnd(rnd(rnd(vx + vy) + vz) + skin);

B = 10b-53 + 5b-102;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
     + (|hz| + |kz|) * (|hz| + |kz|) + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rqx = qx / Qx; rqy = qy / Qy; rqz = qz / Qz;
rsx = sx / (qx * qx); rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs01 = s01 / S01; rs012 = s012 / S012;
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rD = D / T;
rrr = rr / (r * r); rsk = skin / (rr * D);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
Kx = rqx * rqx * rsx * rs01 * rs012;
Ky = rqy * rqy * rsy * rs01 * rs012;
Kz = rqz * rqz * rsz * rs012;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
WHx = Kx - ax * ax; WKx = Kx - bx * bx; WXx = Kx - ax * bx;
WHy = Ky - ay * ay; WKy = Ky - by * by; WXy = Ky - ay * by;
WHz = Kz - az * az; WKz = Kz - bz * bz; WXz = Kz - az * bz;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
cHy = |hy| * |hy|; cKy = |ky| * |ky|; cXy = 2 * |hy| * |ky|;
cHz = |hz| * |hz|; cKz = |kz| * |kz|; cXz = 2 * |hz| * |kz|;
eHx = hx * hx * WHx; eKx = kx * kx * WKx; eXx = -(2 * hx * kx * WXx);
eHy = hy * hy * WHy; eKy = ky * ky * WKy; eXy = -(2 * hy * ky * WXy);
eHz = hz * hz * WHz; eKz = kz * kz * WKz; eXz = -(2 * hz * kz * WXz);

# The radius term is three pieces, one for each direction component's
# square, which sum to skin. Each true piece is r^2 d^2.
cGx = rsk * rr * rD * rt01 * tx;
cGy = rsk * rr * rD * rt01 * ty;
cGz = rsk * rr * rD * tz;
WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;
WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;
WGz = 1 / (rrr * rtz * rsk * rD) - 1;
eGx = cGx * WGx; eGy = cGy * WGy; eGz = cGz * WGz;

# The same permanent, as four blocks of the twelve pieces the error is
# averaged over: each component's reach squared, and the radius term.
P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;
SG = cGx + cGy + cGz;
P12 = P3x + P3y + P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Yx = Kx * Qx * Qx; Yy = Ky * Qy * Qy; Yz = Kz * Qz * Qz;
Gt = r * r * (dx * dx + dy * dy + dz * dz);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero, and no component cancels to zero. For an f64 that is
  # the same as being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  /\ |qx| >= 1b-1074 /\ |qy| >= 1b-1074 /\ |qz| >= 1b-1074
  # product_is_normal on the six halves (cross_product_halves_if_normal),
  # and on the three components, the three direction components, the
  # radius and the radius term (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |sx| >= 1b-1022 /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The permanent's three squares are normal too, since no component is
  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,
  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal.
  /\ |vx| >= 1b-1022 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  /\ hy / |hy| in [-1, 1] /\ ky / |ky| in [-1, 1]
  /\ hz / |hz| in [-1, 1] /\ kz / |kz| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
qx / Qx -> 1 + (qx - Qx) / Qx;
qy / Qy -> 1 + (qy - Qy) / Qy;
qz / Qz -> 1 + (qz - Qz) / Qz;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
D / T -> 1 + (D - T) / T;
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of four blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. In blocks so that no hint's identity spans all twelve
# pieces: one hint over all twelve did not finish in 600 s.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3x + (Yx - Cx * Cx)) + (P3y + (Yy - Cy * Cy)) + (P3z + (Yz - Cz * Cz)) + (SG + (Gt - skin));
P3x + (Yx - Cx * Cx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
P3y + (Yy - Cy * Cy) -> (cHy + eHy) + (cKy + eKy) + (cXy + eXy);
P3z + (Yz - Cz * Cz) -> (cHz + eHz) + (cKz + eKz) + (cXz + eXz);
SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy) + (cGz + eGz);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
(cHy + eHy) - cHy -> eHy;
(cKy + eKy) - cKy -> eKy;
(cXy + eXy) - cXy -> eXy;
(cHz + eHz) - cHz -> eHz;
(cKz + eKz) - cKz -> eKz;
(cXz + eXz) - cXz -> eXz;
(cGx + eGx) - cGx -> eGx;
(cGy + eGy) - cGy -> eGy;
(cGz + eGz) - cGz -> eGz;
eHx / cHx -> (hx / |hx|) * (hx / |hx|) * WHx;
eKx / cKx -> (kx / |kx|) * (kx / |kx|) * WKx;
eXx / cXx -> -((hx / |hx|) * (kx / |kx|) * WXx);
eHy / cHy -> (hy / |hy|) * (hy / |hy|) * WHy;
eKy / cKy -> (ky / |ky|) * (ky / |ky|) * WKy;
eXy / cXy -> -((hy / |hy|) * (ky / |ky|) * WXy);
eHz / cHz -> (hz / |hz|) * (hz / |hz|) * WHz;
eKz / cKz -> (kz / |kz|) * (kz / |kz|) * WKz;
eXz / cXz -> -((hz / |hz|) * (kz / |kz|) * WXz);

# From the twelve pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

`cylcases.py`, whole, as the second of the nine lanes wrote it. It writes the
rest of 3, 4 and 7 from `cyl.g` beside it, for the case with no component
cancelling; `cylsplit.py` writes the others:

```python
"""Write the scripts of 464.15 from cyl.g beside it: cylv.g, the claim on
value; the controls c_cyl_half.g and c_cylv_half.g at B = u / 2,
c_cyl_nonormal.g without product_is_normal, and c_cyl_noeb.g without eb
normal; and b_cyl_<m>.g and b_cylv_<m>.g, cyl.g and cylv.g at B = 9u + m u^2
and 10u + m u^2, for the bisection. Each change is a text replacement that
must match exactly once. Run as: python3 cylcases.py [m ...]."""
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
CYL = open("cyl.g").read()
B_CODE = "B = 10b-53 + 5b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def on_value(text):
    text = sub(text, "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n",
               "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n"
               "# The claim is on value, the double on_floats returns, not on y.\n")
    text = sub(text, "rrr = rr / (r * r); rsk = skin / (rr * D);\n",
               "rrr = rr / (r * r); rsk = skin / (rr * D);\nrv = value / y;\n")
    text = sub(text, "# Each component's square, as y carries it,", "# Each component's square, as value carries it,")
    text = sub(text, "Kx = rqx * rqx * rsx * rs01 * rs012;", "Kx = rqx * rqx * rsx * rs01 * rs012 * rv;")
    text = sub(text, "Ky = rqy * rqy * rsy * rs01 * rs012;", "Ky = rqy * rqy * rsy * rs01 * rs012 * rv;")
    text = sub(text, "Kz = rqz * rqz * rsz * rs012;", "Kz = rqz * rqz * rsz * rs012 * rv;")
    text = sub(text, "WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;", "WGx = 1 / (rrr * rtx * rsk * rD * rt01) - rv;")
    text = sub(text, "WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;", "WGy = 1 / (rrr * rty * rsk * rD * rt01) - rv;")
    text = sub(text, "WGz = 1 / (rrr * rtz * rsk * rD) - 1;", "WGz = 1 / (rrr * rtz * rsk * rD) - rv;")
    text = sub(text, "# The radius term is three pieces,", "# The radius term, as value carries it times 1 / rv, is three pieces,")
    text = sub(text, "# Each component's square as y carries it,", "# Each component's square as value carries it,")
    text = sub(text, "  # The bound is normal, which decide checks.\n",
               "  # y is not zero. Where it is, value is zero too, and the claim on y is\n"
               "  # the claim on value.\n"
               "  /\\ |y| >= 1b-1074\n"
               "  # The bound is normal, which decide checks.\n")
    text = sub(text, "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);\n",
               "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);\nvalue / y -> 1 + (value - y) / y;\n")
    text = sub(text, "SG + (Gt - skin) ->", "SG + (Gt - rv * skin) ->")
    text = sub(text, "(SG + (Gt - skin));", "(SG + (Gt - rv * skin));")
    return text.replace("(y - Ex)", "(value - Ex)")


def at(text, b):
    return sub(text, B_CODE, b)


def without_normal(text):
    keep = []
    lemma_of = ("hx", "kx", "hy", "ky", "hz", "kz", "sx", "sy", "sz", "tx", "ty", "tz",
                "rr", "skin", "vx", "vy", "vz")
    for line in text.split("\n"):
        if "1b-1022" in line and "|eb|" not in line:
            continue
        if line.startswith("  /\\ ") and line.split()[1] in lemma_of and LEMMA in line:
            continue
        keep.append(line)
    text = "\n".join(keep)
    text = sub(text, "  # product_is_normal on the six halves (cross_product_halves_if_normal),\n"
               "  # and on the three components, the three direction components, the\n"
               "  # radius and the radius term (stays_accurate).\n", "")
    text = sub(text, "  # The permanent's three squares are normal too, since no component is\n"
               "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,\n"
               "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal.\n", "")
    return sub(text, ", rcad-geom's cylinder.\n", ", rcad-geom's cylinder.\n# A control: without product_is_normal and the lemma lines it gives.\n")


def without_eb(text):
    text = sub(text, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
    text = sub(text, f"  /\\ eb -/ B * P {LEMMA}\n", "")
    return sub(text, ", rcad-geom's cylinder.\n", ", rcad-geom's cylinder.\n# A control: without eb normal and the lemma line it gives.\n")


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


CYLV = on_value(CYL)
write("cylv.g", CYLV)
write("c_cyl_half.g", at(CYL, B_HALF))
write("c_cylv_half.g", at(CYLV, B_HALF))
write("c_cyl_nonormal.g", without_normal(CYL))
write("c_cyl_noeb.g", without_eb(CYL))
for m in sys.argv[1:]:
    write(f"b_cyl_{m}.g", at(CYL, f"B = 9b-53 + {m}b-106;"))
    write(f"b_cylv_{m}.g", at(CYLV, f"B = 10b-53 + {m}b-106;"))
```

`run1.sh`, whole, which the second of the nine lanes started detached:

```sh
#!/bin/bash
# First batch of 464.15: cyl.g and cylv.g with coq, the controls, and a grid
# of B for the bisection, all at once. Prints to run1.txt.
cd "$(dirname "$0")"
python3 cylcases.py 0 40 60 70 80 90 100 120 160
{
  ./one.sh cyl.g coq &
  ./one.sh cylv.g coq &
  for f in c_cyl_half.g c_cylv_half.g c_cyl_nonormal.g c_cyl_noeb.g b_cyl_*.g b_cylv_*.g; do
    ./one1.sh "$f" &
  done
  wait
} > run1.txt 2>&1
echo done >> run1.txt
```

`m2.g`, whole, one component and `skin` alone (4):

```gappa
@rnd = float<ieee_64, ne>;
hx = rnd(hx_); kx = rnd(kx_); skin = rnd(sk_);
Qx = hx - kx; qx = rnd(Qx); sx = rnd(qx * qx);
rqx = qx / Qx; rsx = sx / (qx * qx);
Kx = rqx * rqx * rsx;
Yx = Kx * Qx * Qx;
y = sx - skin;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
P3x = cHx + cKx + cXx;
SG = skin * 1;
P12 = P3x + SG;
fHx = Kx * hx * hx; fKx = Kx * kx * kx; fXx = -(2 * Kx * hx * kx);
{ |hx| in [1b-1022, 1b1024] /\ |kx| in [1b-1022, 1b1024] /\ skin in [1b-1022, 1b1024]
  /\ |qx| >= 1b-1074 /\ |sx| >= 1b-1022
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  -> y / P12 in ? /\ Yx / P3x in ? /\ fHx / cHx in ? /\ ((P12 + y) - P12) / P12 in ? }
qx / Qx -> 1 + (qx - Qx) / Qx;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
y / P12 -> ((P12 + y) - P12) / P12;
P12 + y -> (P3x + Yx) + (SG - skin);
P3x + Yx -> (cHx + fHx) + (cKx + fKx) + (cXx + fXx);
(cHx + fHx) - cHx -> fHx;
(cKx + fKx) - cKx -> fKx;
(cXx + fXx) - cXx -> fXx;
fHx / cHx -> Kx * (hx / |hx|) * (hx / |hx|);
fKx / cKx -> Kx * (kx / |kx|) * (kx / |kx|);
fXx / cXx -> -(Kx * (hx / |hx|) * (kx / |kx|));
((SG - skin) - SG) / SG -> -(skin / SG);
skin / SG -> 1;
Yx / P3x -> ((P3x + Yx) - P3x) / P3x;
```

It prints `y / P12` in `[-1b2046, 1b3072]`, `Yx / P3x` in `[-2^(4092),
2^(4092)]` and `fHx / cHx` in `[-1, 1]`.

`m3.g`, whole, `m2.g` with every piece doubled:

```gappa
@rnd = float<ieee_64, ne>;
hx = rnd(hx_); kx = rnd(kx_); skin = rnd(sk_);
Qx = hx - kx; qx = rnd(Qx); sx = rnd(qx * qx);
rqx = qx / Qx; rsx = sx / (qx * qx);
Kx = rqx * rqx * rsx;
Yx = Kx * Qx * Qx;
y = sx - skin;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
P3x = cHx + cKx + cXx;
SG = skin * 1;
P12 = P3x + SG;
fHx = Kx * hx * hx; fKx = Kx * kx * kx; fXx = -(2 * Kx * hx * kx);
dHx = 2 * cHx; dKx = 2 * cKx; dXx = 2 * cXx;
P3dx = dHx + dKx + dXx; SGd = 2 * SG;
P12d = P3dx + SGd;
{ |hx| in [1b-1022, 1b1024] /\ |kx| in [1b-1022, 1b1024] /\ skin in [1b-1022, 1b1024]
  /\ |qx| >= 1b-1074 /\ |sx| >= 1b-1022
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  -> y / P12 in ? /\ ((P3dx + Yx) - P3dx) / P3dx in ? /\ fHx / dHx in ? }
qx / Qx -> 1 + (qx - Qx) / Qx;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
y / P12 -> 2 * (((P12d + y) - P12d) / P12d);
P12d + y -> (P3dx + Yx) + (SGd - skin);
P3dx + Yx -> (dHx + fHx) + (dKx + fKx) + (dXx + fXx);
(dHx + fHx) - dHx -> fHx;
(dKx + fKx) - dKx -> fKx;
(dXx + fXx) - dXx -> fXx;
fHx / dHx -> Kx * (hx / |hx|) * (hx / |hx|) / 2;
fKx / dKx -> Kx * (kx / |kx|) * (kx / |kx|) / 2;
fXx / dXx -> -(Kx * (hx / |hx|) * (kx / |kx|)) / 2;
((SGd - skin) - SGd) / SGd -> -(skin / SGd);
skin / SGd -> 1 / 2;
```

It prints `y / P12` within `[-1, 1]`, the block `(P3dx + Yx - P3dx) / P3dx`
within `[-0.5, 0.5]` and `fHx / dHx` within `[-0.5, 0.5]`, rounded.

`vgen.py`, whole, beside `cyl.g` and the second lane's `cylv.g`:

```python
"""Probes for the claim on value: t_vtriv.g is the second lane's cylv.g with
the goal px in ? and every hint kept; cylv2.g is cyl.g with value's own
rounding taken as a fraction of y, and y as a fraction of the permanent."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


GOAL = "  (y - Ex) / P12 in ? /\\ P -/ Phat in ?\n  /\\ (y - Ex) / eb in [-1, 1]\n"
VGOAL = "  (value - Ex) / P12 in ? /\\ P -/ Phat in ?\n  /\\ (value - Ex) / eb in [-1, 1]\n"
cylv = open("cylv.g").read()
open("t_vtriv.g", "w").write(sub(cylv, VGOAL, "  px in ?\n"))

c = open("cyl.g").read()
c = sub(c, "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.\n"
        "# The claim is on value, the double on_floats returns, not on y.\n")
c = sub(c, "Gt = r * r * (dx * dx + dy * dy + dz * dz);\n",
        "Gt = r * r * (dx * dx + dy * dy + dz * dz);\n\n"
        "# y over the permanent: each component's square as y carries it is K\n"
        "# times (h - k)^2, which is K times the three pieces of its reach\n"
        "# squared, the third with a sign; the radius term is taken off whole.\n"
        "fHx = Kx * hx * hx; fKx = Kx * kx * kx; fXx = -(2 * Kx * hx * kx);\n"
        "fHy = Ky * hy * hy; fKy = Ky * ky * ky; fXy = -(2 * Ky * hy * ky);\n"
        "fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);\n")
c = sub(c, "  # The bound is normal, which decide checks.\n",
        "  # y is not zero. Where it is, value is zero too, and the claim on y is\n"
        "  # the claim on value.\n"
        "  /\\ |y| >= 1b-1074\n"
        "  # The bound is normal, which decide checks.\n")
c = sub(c, GOAL,
        "  (y - Ex) / P12 in ? /\\ y / P12 in ? /\\ (value - y) / y in ? /\\ P -/ Phat in ?\n"
        "  /\\ (value - Ex) / eb in [-1, 1]\n")
c = sub(c, "(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);\n",
        "(value - Ex) / eb -> (y - Ex) / eb + (value - y) / eb;\n"
        "(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);\n"
        "(value - y) / eb -> ((value - y) / y) * (y / P12) * (Phat / P) * (P / eb);\n")
c = sub(c, "fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);\n",
        "fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);\n"
        "# The same pieces doubled, so that each f is at most about half its\n"
        "# piece: Gappa averages a sum's relative errors only while each is\n"
        "# above -1, and f against the piece itself reaches -1.\n"
        "dHx = 2 * cHx; dKx = 2 * cKx; dXx = 2 * cXx;\n"
        "dHy = 2 * cHy; dKy = 2 * cKy; dXy = 2 * cXy;\n"
        "dHz = 2 * cHz; dKz = 2 * cKz; dXz = 2 * cXz;\n"
        "P3dx = dHx + dKx + dXx; P3dy = dHy + dKy + dXy; P3dz = dHz + dKz + dXz;\n"
        "SGd = 2 * SG;\n"
        "P12d = P3dx + P3dy + P3dz + SGd;\n")
c += ("\n# value's own rounding is a fraction of y, and y a fraction of the\n"
      "# permanent, averaged over the same four blocks, doubled.\n"
      "y / P12 -> 2 * (((P12d + y) - P12d) / P12d);\n"
      "P12d + y -> (P3dx + Yx) + (P3dy + Yy) + (P3dz + Yz) + (SGd - skin);\n")
for a in "xyz":
    c += f"P3d{a} + Y{a} -> (dH{a} + fH{a}) + (dK{a} + fK{a}) + (dX{a} + fX{a});\n"
for a in "xyz":
    for p in "HKX":
        c += f"(d{p}{a} + f{p}{a}) - d{p}{a} -> f{p}{a};\n"
for a in "xyz":
    c += (f"fH{a} / dH{a} -> K{a} * (h{a} / |h{a}|) * (h{a} / |h{a}|) / 2;\n"
          f"fK{a} / dK{a} -> K{a} * (k{a} / |k{a}|) * (k{a} / |k{a}|) / 2;\n"
          f"fX{a} / dX{a} -> -(K{a} * (h{a} / |h{a}|) * (k{a} / |k{a}|)) / 2;\n")
c += "((SGd - skin) - SGd) / SGd -> -(skin / SGd);\nskin / SGd -> 1 / 2;\n"
open("cylv2.g", "w").write(c)
open("c_cylv2_half.g", "w").write(sub(c, "B = 10b-53 + 5b-102;", "B = 1b-54;"))
for m in (84, 88, 96, 112, 144):
    open(f"b_cylv2_{m}.g", "w").write(sub(c, "B = 10b-53 + 5b-102;", f"B = 10b-53 + {m}b-106;"))
open("t_v2triv.g", "w").write(sub(c, "  (y - Ex) / P12 in ? /\\ y / P12 in ? /\\ (value - y) / y in ? /\\ P -/ Phat in ?\n"
                                    "  /\\ (value - Ex) / eb in [-1, 1]\n", "  px in ?\n"))
```

The first form of `cylv2.g` is what this writes without the `sub` that adds
the doubled pieces, and with the hints written on `cH`, `cK`, `cX`, `P3x` and
`SG` as `m2.g` writes them. `t_v2triv.g`, the first form's hints with the
goal `px in ?`, exits 0 in `127.05s` in the same run.

`run2.sh`, whole:

```sh
#!/bin/bash
# The claim on value two ways, all at once: the second lane's cylv.g (rv on
# every carry) and its control at u/2 with a timeout of 1800 s, and cylv2.g
# (value's rounding as a fraction of y) with its control and a grid in m.
cd "$(dirname "$0")"
python3 vgen.py
{
  T=1800 ./one1.sh cylv.g &
  T=1800 ./one1.sh c_cylv_half.g &
  for f in cylv2.g c_cylv2_half.g b_cylv2_*.g; do
    T=1200 ./one1.sh "$f" &
  done
  wait
} > run2.txt 2>&1
echo done >> run2.txt
```

`cylsplit.py`, whole:

```python
"""Write cyl.g's case split, one script per case and a control at B = u / 2
beside each. A case is three letters, one for each of the components x, y
and z of the cross product, saying what it is:

n  it does not cancel: q = rnd(h - k) is not zero, so its square s is
   normal (stays_accurate), and so is its reach square v, which is never
   below s;
c  it cancels to zero, h == k, and its reach square v = rnd(4 h^2) is
   normal;
s  it cancels, and v is below the normals and does not round to zero;
u  it cancels, and v rounds to zero.

Those four cover every component whose halves are normal, which
cross_product_halves_if_normal checks: q is zero exactly when h == k. nnn
is cyl.g, byte for byte. A component that cancels has no square in y, and
its true square, (h a - h b)^2, is the whole of its block's error. The
names are cyl_<case>.g and c_cyl_<case>_half.g. Run as: python3 cylsplit.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
B_CODE = "B = 10b-53 + 5b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
STATE = {"n": "does not cancel", "c": "cancels, its reach square normal",
         "s": "cancels, its reach square below the normals and not zero",
         "u": "cancels, its reach square rounding to zero"}


def reach(a):
    return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"


def left_sum(terms, rounded):
    """Sum terms left to right as Rust adds them, each sum rounded or not."""
    acc = None
    for t in terms:
        if acc is None:
            acc = t
        else:
            acc = f"rnd({acc} + {t})" if rounded else f"({acc} + {t})"
    return acc


def make(case, b=B_CODE):
    st = dict(zip(AXES, case))
    live = [a for a in AXES if st[a] == "n"]
    dead = [a for a in AXES if st[a] != "n"]
    small = [a for a in AXES if st[a] in "su"]
    live01 = [a for a in "xy" if a in live]
    s01_rounds = len(live01) == 2
    s012_rounds = bool(live01) and "z" in live
    src = [
        "# restates: crates/rcad-geom/src/cylinder.rs on_floats at 2a7ca9e7",
        "# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at 2a7ca9e7",
        "# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at 2a7ca9e7",
        "# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at 2a7ca9e7",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 2a7ca9e7",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 2a7ca9e7",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.",
    ]
    if dead:
        src.append(f"# The case {case}: " + "; ".join(f"{a} {STATE[st[a]]}" for a in AXES) + ".")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the axis origin, the axis direction and the radius, each an f64.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        "ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);",
        "dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);",
        "r = rnd(r_);",
        "",
        "# What on_floats computes, line for line.",
        "Fx = px - ox; Fy = py - oy; Fz = pz - oz;",
        "fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);",
        "",
        "# cross_product_halves: h is each component's left half, k its right half.",
    ]
    if dead:
        src += ["# A component that cancels has the same double for both halves, so k",
                "# is written h, and its q and s are zero and drop out."]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"h{a} = rnd({left}); k{a} = " + (f"h{a};" if a in dead else f"rnd({right});"))
    if live:
        src.append(" ".join(f"Q{a} = h{a} - k{a};" for a in live))
        src.append(" ".join(f"q{a} = rnd(Q{a});" for a in live))
    src += [
        "",
        "# direction_squared, left to right as Rust adds it.",
        "tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);",
        "T01 = tx + ty; t01 = rnd(T01);",
        "T = t01 + tz; D = rnd(T);",
        "",
        "rr = rnd(r * r);",
        "skin = rnd(rr * D);",
    ]
    if dead:
        src += ["# A zero square added to a double leaves that double, which then",
                "# rounds to itself, so a sum is written without its zero squares."]
    if live:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in live))
    if s01_rounds:
        src.append("S01 = sx + sy; s01 = rnd(S01);")
    elif live01:
        src.append(f"s01 = s{live01[0]};")
    if s012_rounds:
        src.append("S012 = s01 + sz; s012 = rnd(S012);")
    elif live01:
        src.append("s012 = s01;")
    elif live:
        src.append("s012 = sz;")
    src.append("y = s012 - skin;" if live else "y = -skin;")
    src += ["value = rnd(y);", "", "# The permanent: each half's reach squared, then the radius term."]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (dbl.g), so it is written unrounded."]
    src.append(" ".join(f"n{a} = rnd(|h{a}| + |k{a}|);" if a in live else f"n{a} = |h{a}| + |k{a}|;"
                        for a in AXES))
    if small:
        src.append(" ".join(f"Xv{a} = n{a} * n{a};" for a in small))
        src += ["# A reach square that rounds to zero is written as the 0 it rounds to",
                "# (464.14's tinysq.g), and the sums leave it out."]
    vs = []
    for a in AXES:
        if st[a] in "nc":
            vs.append(f"v{a} = rnd(n{a} * n{a});")
        elif st[a] == "s":
            vs.append(f"v{a} = rnd(Xv{a});")
    src.append(" ".join(vs))
    kept = [f"v{a}" for a in AXES if st[a] != "u"]
    acc = left_sum(kept, True)
    src.append(f"P = rnd({acc} + skin);" if acc else "P = skin;")
    src += [
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;",
        "Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);",
        "",
        "# The permanent with no rounding of its own.",
        "Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)",
        "     + (|hz| + |kz|) * (|hz| + |kz|) + skin;",
    ]
    if small:
        terms = [f"v{a}" if st[a] == "s" else reach(a) for a in AXES if st[a] != "u"]
        acc = left_sum(terms, False)
        src += ["# The same with v in place of each reach square below the normals that",
                "# does not round to zero, and without each that does, added as P adds.",
                f"Phatx = {acc} + skin;" if acc else "Phatx = skin;"]
    src += [
        "",
        "# What each rounding multiplied its argument by.",
        "rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;",
    ]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"rh{a} = h{a} / ({left}); rk{a} = k{a} / ({right});")
    if live:
        src.append(" ".join(f"rq{a} = q{a} / Q{a};" for a in live))
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in live))
    rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rs012 = s012 / S012;"] if s012_rounds else [])
    if rs:
        src.append(" ".join(rs))
    src += [
        "rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);",
        "rt01 = t01 / T01; rD = D / T;",
        "rrr = rr / (r * r); rsk = skin / (rr * D);",
        "",
        "# Each component's square, as y carries it, is (h - k)^2 times K; the true",
        "# one is (h a - k b)^2, a and b undoing the roundings of h and k.",
    ]
    if dead:
        src.append("# One that cancels has no square in y, and its true one is (h a - h b)^2.")
    for a in live:
        carry = [f"rq{a}", f"rq{a}", f"rs{a}"]
        if a in "xy" and s01_rounds:
            carry.append("rs01")
        if s012_rounds:
            carry.append("rs012")
        src.append(f"K{a} = " + " * ".join(carry) + ";")
    for a in AXES:
        src.append(f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a}); b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});")
    src += [
        "",
        "# So each component's error is three terms, one for each piece of its",
        "# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its",
        "# piece.",
    ]
    if dead:
        src.append("# One that cancels is off by its whole true square, eC.")
    for a in live:
        src.append(f"WH{a} = K{a} - a{a} * a{a}; WK{a} = K{a} - b{a} * b{a}; WX{a} = K{a} - a{a} * b{a};")
    for a in AXES:
        src.append(f"cH{a} = |h{a}| * |h{a}|; cK{a} = |k{a}| * |k{a}|; cX{a} = 2 * |h{a}| * |k{a}|;")
    for a in AXES:
        if a in live:
            src.append(f"eH{a} = h{a} * h{a} * WH{a}; eK{a} = k{a} * k{a} * WK{a}; eX{a} = -(2 * h{a} * k{a} * WX{a});")
        else:
            src.append(f"eC{a} = -(C{a} * C{a});")
    src += [
        "",
        "# The radius term is three pieces, one for each direction component's",
        "# square, which sum to skin. Each true piece is r^2 d^2.",
        "cGx = rsk * rr * rD * rt01 * tx;",
        "cGy = rsk * rr * rD * rt01 * ty;",
        "cGz = rsk * rr * rD * tz;",
        "WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;",
        "WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;",
        "WGz = 1 / (rrr * rtz * rsk * rD) - 1;",
        "eGx = cGx * WGx; eGy = cGy * WGy; eGz = cGz * WGz;",
        "",
        "# The same permanent, as four blocks of the twelve pieces the error is",
        "# averaged over: each component's reach squared, and the radius term.",
        "P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;",
        "SG = cGx + cGy + cGz;",
        "P12 = P3x + P3y + P3z + SG;",
        "",
        "# Each component's square as y carries it, and the radius term as Ex has it.",
    ]
    if live:
        src.append(" ".join(f"Y{a} = K{a} * Q{a} * Q{a};" for a in live))
    src.append("Gt = r * r * (dx * dx + dy * dy + dz * dz);")

    nonzero = "\n  /\\ ".join([
        "|fx| >= 1b-1074 /\\ |fy| >= 1b-1074 /\\ |fz| >= 1b-1074",
        "|dx| >= 1b-1074 /\\ |dy| >= 1b-1074 /\\ |dz| >= 1b-1074",
        "|r| >= 1b-1074"] + ([" /\\ ".join(f"|q{a}| >= 1b-1074" for a in live)] if live else []))
    src += [
        "",
        "{",
        "  # Every input is a finite f64.",
        "     |px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024",
        "  /\\ |ox| <= 1b1024 /\\ |oy| <= 1b1024 /\\ |oz| <= 1b1024",
        "  /\\ |dx| <= 1b1024 /\\ |dy| <= 1b1024 /\\ |dz| <= 1b1024",
        "  /\\ |r| <= 1b1024",
        "  # No factor is zero, and no component cancels to zero. For an f64 that is"
        if not dead else
        "  # No factor is zero, and no component but those the case names cancels to\n"
        "  # zero. For an f64 that is",
        "  # the same as being at least the smallest subnormal.",
        "  /\\ " + nonzero,
        "  # product_is_normal on the six halves (cross_product_halves_if_normal),",
        "  # and on the three components, the three direction components, the",
        "  # radius and the radius term (stays_accurate).",
        "  /\\ |hx| >= 1b-1022 /\\ |kx| >= 1b-1022 /\\ |hy| >= 1b-1022",
        "  /\\ |ky| >= 1b-1022 /\\ |hz| >= 1b-1022 /\\ |kz| >= 1b-1022",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in live))
    src += [
        "  /\\ |tx| >= 1b-1022 /\\ |ty| >= 1b-1022 /\\ |tz| >= 1b-1022",
        "  /\\ |rr| >= 1b-1022 /\\ |skin| >= 1b-1022",
    ]
    if not dead:
        src += [
            "  # The permanent's three squares are normal too, since no component is",
            "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,",
            "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal.",
            "  /\\ |vx| >= 1b-1022 /\\ |vy| >= 1b-1022 /\\ |vz| >= 1b-1022",
        ]
    else:
        src += [
            "  # The reach square of a component that does not cancel is normal: |q| <= n,",
            "  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=",
            "  # rnd(q * q) = s, which is normal. For one that cancels the case says",
            "  # which it is. One below the normals that does not round to zero is at",
            "  # least the smallest subnormal, and tinysq.g and tinyzero.g split them",
            "  # at 2^-1075: at or below, they round to zero.",
        ]
        rows = []
        for a in AXES:
            if st[a] in "nc":
                rows.append(f"|v{a}| >= 1b-1022")
            elif st[a] == "s":
                rows.append(f"Xv{a} <= 1b-1022 /\\ v{a} >= 1b-1074")
            else:
                rows.append(f"Xv{a} <= 1b-1075")
        src.append("  /\\ " + " /\\ ".join(rows))
    src += [
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
        "  # What lemma.g proves each of those gives.",
    ]
    if dead:
        src.append("  # k is h in a component that cancels, so its line is h's against k's product.")
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for a in live:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for p in ("tx -/ dx * dx", "ty -/ dy * dy", "tz -/ dz * dz", "rr -/ r * r", "skin -/ rr * D"):
        src.append(f"  /\\ {p} {LEMMA}")
    for a in AXES:
        if st[a] in "nc":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    src += [
        f"  /\\ eb -/ B * P {LEMMA}",
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
    ]
    for a in AXES:
        src.append(f"  /\\ h{a} / |h{a}| in [-1, 1]" + ("" if a in dead else f" /\\ k{a} / |k{a}| in [-1, 1]"))
    src.append("->")
    if small:
        src.append("  (y - Ex) / P12 in ? /\\ P -/ Phatx in ? /\\ P in ?")
    else:
        src.append("  (y - Ex) / P12 in ? /\\ P -/ Phat in ?")
    src += ["  /\\ (y - Ex) / eb in [-1, 1]", "}", "", "# How far each rounding is from the identity."]
    src += ["fx / Fx -> 1 + (fx - Fx) / Fx;", "fy / Fy -> 1 + (fy - Fy) / Fy;", "fz / Fz -> 1 + (fz - Fz) / Fz;"]
    for a in AXES:
        for h, prod in zip("hk", HALVES[a]):
            src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in live:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    for a in live:
        src.append(f"s{a} / (q{a} * q{a}) -> 1 + (s{a} - q{a} * q{a}) / (q{a} * q{a});")
    if s01_rounds:
        src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
    if s012_rounds:
        src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
    src += [
        "tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);",
        "ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);",
        "tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);",
        "t01 / T01 -> 1 + (t01 - T01) / T01;",
        "D / T -> 1 + (D - T) / T;",
        "rr / (r * r) -> 1 + (rr - r * r) / (r * r);",
        "skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);",
        "",
        "# The error is the sum of four blocks' errors, each the sum of its pieces'",
        "# errors, each a fraction of its own piece of the permanent; add_rr then",
        "# averages them. In blocks so that no hint's identity spans all twelve",
        "# pieces: one hint over all twelve did not finish in 600 s.",
    ]
    if dead:
        src += ["# A component that cancels is one block, off by its true square, which is",
                "# (a - b)^2 / 4 of its reach squared, 4 h^2."]
    src.append("(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;")
    blocks = [f"(P3{a} + (Y{a} - C{a} * C{a}))" if a in live else f"(P3{a} + eC{a})" for a in AXES]
    src.append("P12 + (y - Ex) -> " + " + ".join(blocks) + " + (SG + (Gt - skin));")
    for a in live:
        src.append(f"P3{a} + (Y{a} - C{a} * C{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
    src.append("SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy) + (cGz + eGz);")
    for a in AXES:
        if a in live:
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        else:
            src.append(f"(P3{a} + eC{a}) - P3{a} -> eC{a};")
    for g in AXES:
        src.append(f"(cG{g} + eG{g}) - cG{g} -> eG{g};")
    for a in AXES:
        if a in live:
            src += [f"eH{a} / cH{a} -> (h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a};",
                    f"eK{a} / cK{a} -> (k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a};",
                    f"eX{a} / cX{a} -> -((h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a});"]
        else:
            src.append(f"eC{a} / P3{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * (a{a} - b{a}) * (a{a} - b{a}) / 4);")
    src += ["", "# From the twelve pieces to the permanent on_floats rounds, and on to eb.",
            "(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);"]
    if small:
        shift = " + ".join(f"(Xv{a} - v{a})" if st[a] == "s" else f"Xv{a}" for a in small)
        src += ["# What rounding a reach square below the normals cost is an amount, and",
                "# eb being normal keeps P large enough that it is small against eb.",
                f"Phat / P -> Phatx / P + ({shift}) / P;",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
                "P -> eb / (B * (1 + (eb - B * P) / (B * P)));"]
    else:
        src += ["Phat / P -> 1 / (1 + (P - Phat) / Phat);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));"]
    return "\n".join(src) + "\n"


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


for case in map("".join, itertools.product("ncsu", repeat=3)):
    name = "cyl" if case == "nnn" else f"cyl_{case}"
    write(f"{name}.g" if case != "nnn" else "cyl_nnn.g", make(case))
    if case != "nnn":
        write(f"c_{name}_half.g", make(case, B_HALF))
```

`dbl.g`, whole:

```gappa
# dbl.g: a double added to itself rounds to itself, so the reach |h| + |k| of
# a component whose halves are the same double h is 2 |h| exactly, a double.
# Gappa's float has no top to its exponents, so this says nothing of overflow.
@rnd = float<ieee_64, ne>;
h = rnd(h_);
{ |h| <= 1b1024 /\ |h| >= 1b-1074 -> rnd(|h| + |h|) - (|h| + |h|) in [0, 0] }
|h| + |h| -> 2 * |h|;
```

`run3.sh`, whole:

```sh
#!/bin/bash
# cylsplit.py's 63 cases other than nnn, each with gappa -Bcoq and coqc when
# it proves, their 63 controls at u/2, dbl.g with coqc, and the claim on
# value (cylv2.g doubled) at B = 10u + m u^2 for m of 81, 82 and 83, all at
# once.
cd "$(dirname "$0")"
python3 cylsplit.py
python3 vgen.py
for m in 81 82 83; do
  sed "s/^B = 10b-53 + 5b-102;\$/B = 10b-53 + ${m}b-106;/" cylv2.g > "b_cylv2_$m.g"
done
{
  grep -c '^B = 10b-53 + 8[123]b-106;$' b_cylv2_81.g b_cylv2_82.g b_cylv2_83.g
  ./one.sh dbl.g coq &
  for f in cyl_*.g; do
    [ "$f" = cyl_nnn.g ] && continue
    ./one.sh "$f" coq &
  done
  for f in c_cyl_???_half.g; do
    ./one1.sh "$f" &
  done
  for m in 81 82 83; do
    T=1200 ./one1.sh "b_cylv2_$m.g" &
  done
  wait
} > run3.txt 2>&1
echo done >> run3.txt
```

`tally3.py`, whole:

```python
"""Tally run3.txt and the .out files of cylsplit.py's 63 cases: for each case
its exit, the relative bound on y and on P that it prints, and its coqc line;
for each control its exit and best. Prints one line per case, then counts.
Run as: python3 tally3.py"""
import collections
import itertools
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))
run = open("run3.txt").read()
cases = ["".join(c) for c in itertools.product("ncsu", repeat=3) if "".join(c) != "nnn"]


def short(out, goal):
    m = re.search(re.escape(goal) + r" in \[[^{]*\{([^,]*),[^}]*\}, [^{]*\{([^,]*),", out)
    return f"[{m.group(1)}, {m.group(2)}]" if m else "-"


ybounds = collections.Counter()
pbounds = collections.Counter()
bests = collections.Counter()
coq = []
for c in cases:
    out = open(f"cyl_{c}.out").read()
    ex = re.search(rf"^cyl_{c} exit (\d+) ([\d.]+)s$", run, re.M)
    cq = re.search(rf"^cyl_{c} coq -Bcoq exit (\d+) coqc exit (\d+) ([\d.]+)s lines (\d+) hints (\d+)$", run, re.M)
    ctl = re.search(rf"^c_cyl_{c}_half exit (\d+) ([\d.]+)s$", run, re.M)
    cout = open(f"c_cyl_{c}_half.out").read() if os.path.exists(f"c_cyl_{c}_half.out") else ""
    best = re.search(r"best: (\[[^\]]*\])", cout)
    y = short(out, "(y - Ex) / P12")
    p = short(out, "P -/ Phatx") if "s" in c or "u" in c else short(out, "P -/ Phat")
    ybounds[y] += 1
    pbounds[p] += 1
    bests[best.group(1) if best else "-"] += 1
    if cq:
        coq.append((int(cq.group(4)), int(cq.group(5)), c, cq.group(1), cq.group(2), float(cq.group(3))))
    print(c, "exit", ex.group(1) if ex else "?", ex.group(2) if ex else "", "y", y, "P", p,
          "coq", cq.groups() if cq else "-", "ctl", ctl.group(1) if ctl else "?", best.group(1) if best else "-",
          "pointless" if "pointless" in out else "")
print("y bounds", dict(ybounds))
print("P bounds", dict(pbounds))
print("control bests", dict(bests))
print("cases exit 0:", len(re.findall(r"^cyl_\w\w\w exit 0 ", run, re.M)), "of", len(cases))
print("controls exit 1:", len(re.findall(r"^c_cyl_\w\w\w_half exit 1 ", run, re.M)), "of", len(cases))
print("coq ok:", sum(1 for x in coq if x[3] == "0" and x[4] == "0"), "of", len(coq))
if coq:
    coq.sort()
    print("coq smallest", coq[0][:3], "largest", coq[-1][:3], "times", min(x[5] for x in coq), max(x[5] for x in coq))
def other_lines(f):
    return [line for line in open(f).read().split("\n")
            if line and "assumed to be nonzero" not in line and not line.startswith(("Results:", "  "))]


warn = [f for f in os.listdir(".") if f.startswith("cyl_") and f.endswith((".out", ".bcoq", ".coqc"))
        and other_lines(f)]
print("files with a line other than results and 'assumed to be nonzero':", sorted(warn))
```

After its line for each case, `python3 tally3.py` prints:

```text
y bounds {'[-8.88178e-16, 8.88178e-16]': 9, '[-7.77156e-16, 7.77156e-16]': 27, '[-5.55112e-16, 5.55112e-16]': 27}
P bounds {'[-6.66134e-16, 6.66134e-16]': 14, '[-5.55112e-16, 5.55112e-16]': 19, '[-4.44089e-16, 4.44089e-16]': 9, '[-3.33067e-16, 3.33067e-16]': 10, '[-2.22045e-16, 2.22045e-16]': 3, '[-1.11022e-16, 1.11022e-16]': 7, '-': 1}
control bests {'[-16, 16]': 9, '[-14, 14]': 27, '[-10, 10]': 27}
cases exit 0: 63 of 63
controls exit 1: 63 of 63
coq ok: 63 of 63
coq smallest (8908, 33, 'uuc') largest (13908, 48, 'nns') times 145.5 194.2
files with a line other than results and 'assumed to be nonzero': []
```

The one `-` among the `P` bounds is `cyl_uuu.g`'s, whose goal is `skin`
against itself (6).

`t_axis.g`, whole:

```gappa
# One component of cyl.g alone: its square against the true one, over its
# reach squared, to time Gappa on three pieces.
@rnd = float<ieee_64, ne>;
py = rnd(py_); pz = rnd(pz_);
oy = rnd(oy_); oz = rnd(oz_);
dy = rnd(dy_); dz = rnd(dz_);
Fy = py - oy; Fz = pz - oz;
fy = rnd(Fy); fz = rnd(Fz);
hx = rnd(fy * dz); kx = rnd(fz * dy);
Qx = hx - kx; qx = rnd(Qx);
sx = rnd(qx * qx);
Cx = Fy * dz - Fz * dy;
rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rqx = qx / Qx; rsx = sx / (qx * qx);
Kx = rqx * rqx * rsx;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
WHx = Kx - ax * ax; WKx = Kx - bx * bx; WXx = Kx - ax * bx;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
eHx = hx * hx * WHx; eKx = kx * kx * WKx; eXx = -(2 * hx * kx * WXx);
P3 = cHx + cKx + cXx;
{
     |py| <= 1b1024 /\ |pz| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |qx| >= 1b-1074
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |sx| >= 1b-1022
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
->
  (sx - Cx * Cx) / P3 in ?
}
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
qx / Qx -> 1 + (qx - Qx) / Qx;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
(sx - Cx * Cx) / P3 -> ((P3 + (sx - Cx * Cx)) - P3) / P3;
P3 + (sx - Cx * Cx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
eHx / cHx -> (hx / |hx|) * (hx / |hx|) * WHx;
eKx / cKx -> (kx / |kx|) * (kx / |kx|) * WKx;
eXx / cXx -> -((hx / |hx|) * (kx / |kx|) * WXx);
```

`t_perm2.g`, whole:

```gappa
# The permanent alone: six halves, each a normal double, and skin a normal
# double; P as on_floats rounds it against Phat with no rounding of its own.
@rnd = float<ieee_64, ne>;
hx = rnd(hx_); kx = rnd(kx_); hy = rnd(hy_); ky = rnd(ky_); hz = rnd(hz_); kz = rnd(kz_);
skin = rnd(skin_);
Nx = |hx| + |kx|; Ny = |hy| + |ky|; Nz = |hz| + |kz|;
nx = rnd(Nx); ny = rnd(Ny); nz = rnd(Nz);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
V01 = vx + vy; v01 = rnd(V01);
V012 = v01 + vz; v012 = rnd(V012);
PP = v012 + skin; P = rnd(PP);
Phat = Nx * Nx + Ny * Ny + Nz * Nz + skin;
{
     |hx| <= 1b1024 /\ |kx| <= 1b1024 /\ |hy| <= 1b1024
  /\ |ky| <= 1b1024 /\ |hz| <= 1b1024 /\ |kz| <= 1b1024 /\ |skin| <= 1b1024
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ skin >= 1b-1022
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
->
  P -/ Phat in ?
}
```

`one.sh` is 464.13's, whole in its 9. `one1.sh`, whole:

```sh
#!/bin/bash
# one.sh with the timeout taken from T (default 600) and without coq, for
# trial runs.
cd "$(dirname "$0")"
g=/nix/store/0ihij0570a325g02l4wnf0idk8vqxr9b-gappa-1.8.0/bin/gappa
f=$1
n=${f%.g}
start=$(date +%s.%N)
timeout "${T:-600}" "$g" -Eprecision=200 "$f" > "$n.out" 2>&1
code=$?
end=$(date +%s.%N)
printf '%s exit %d %.2fs\n' "$n" "$code" "$(echo "$end - $start" | bc)"
grep -v 'assumed to be nonzero' "$n.out" | head -40 | sed "s/^/  $n: /"
```

`par.sh`, whole, which ran the nine `b_cyl_<m>.g` of 3 for `m` of 61 to 69:

```sh
#!/bin/bash
# Run each named script through one.sh at once, with a timeout in seconds
# as the first argument, and wait for all of them.
cd "$(dirname "$0")"
t=$1
shift
for f in "$@"; do
  T=$t ./one1.sh "$f" &
done
wait
```

`run4.sh`, whole:

```sh
#!/bin/bash
# one.sh with coq on b_cylv2_82.g, the smallest B that proves the claim on
# value, with each step's timeout at 1500 s instead of 600, since the Gappa
# run alone took 447 s in run3.sh.
cd "$(dirname "$0")"
g=/nix/store/0ihij0570a325g02l4wnf0idk8vqxr9b-gappa-1.8.0/bin/gappa
coqc=/nix/store/cgfd855dwk38jxngg5wdvysdjjn2wv3i-coq-8.20.1/bin/coqc
export COQPATH=/nix/store/1rqb30ra496k5zpw9aji329zn00rjggv-coq8.20-gappalib-1.8.0/lib/coq/8.20/user-contrib:/nix/store/p3ka5vy0zshbknhc9pphsffxxwfkcwq4-coq8.20-flocq-4.2.2/lib/coq/8.20/user-contrib
n=b_cylv2_82
{
  start=$(date +%s.%N)
  timeout 1500 "$g" -Eprecision=200 -Bcoq "$n.g" > "$n.v" 2> "$n.bcoq"
  bcode=$?
  timeout 1500 "$coqc" "$n.v" > "$n.coqc" 2>&1
  ccode=$?
  end=$(date +%s.%N)
  printf '%s coq -Bcoq exit %d coqc exit %d %.2fs lines %d hints %d\n' "$n" "$bcode" "$ccode" \
    "$(echo "$end - $start" | bc)" "$(wc -l < "$n.v")" "$(grep -c '^Hypothesis a[0-9]' "$n.v")"
  echo done
} > run4.txt 2>&1
```

`same.py`, whole:

```python
"""Check that each script 464.15 appends whole is byte for byte the file that
was run, in cyl3/ or, for run1.sh, in cyl2/ beside it, and for the first
lane's probes t_axis.g and t_perm2.g in cyl/ beside it; that one.sh is 464.13's
block; and that cylsplit.py, cylcases.py and vgen.py, run again in fresh
directories from the appended text, write what was run, in cyl3/ for the third
lane's arguments and in cyl2/ for run1.sh's.
Run as: python3 same.py <doc> <cyl3 dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, cyl3 = sys.argv[1], sys.argv[2]
cyl2 = os.path.join(cyl3, "..", "cyl2")
text = open(doc).read()
start = text.index("### 464.15")
sec = text[start:]
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)
blocks = FENCED.findall(sec)
print("code blocks in 464.15:", len(blocks))

# Which appended block is which file: by its first lines. m2.g and m3.g share
# their first twelve, and only m3.g defines the doubled pieces dHx.
marks = {
    "cyl.g": "# restates: crates/rcad-geom/src/cylinder.rs on_floats at 2a7ca9e7\n# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal",
    "cylcases.py": '"""Write the scripts of 464.15 from cyl.g',
    "run1.sh": "#!/bin/bash\n# First batch of 464.15",
    "vgen.py": '"""Probes for the claim on value',
    "cylsplit.py": '"""Write cyl.g\'s case split',
    "run2.sh": "#!/bin/bash\n# The claim on value two ways",
    "run3.sh": "#!/bin/bash\n# cylsplit.py's 63 cases",
    "tally3.py": '"""Tally run3.txt',
    "dbl.g": "# dbl.g:",
    "m2.g": "@rnd = float<ieee_64, ne>;\nhx = rnd(hx_); kx = rnd(kx_); skin = rnd(sk_);",
    "m3.g": "@rnd = float<ieee_64, ne>;\nhx = rnd(hx_); kx = rnd(kx_); skin = rnd(sk_);",
    "one1.sh": "#!/bin/bash\n# one.sh with the timeout taken from T",
    "par.sh": "#!/bin/bash\n# Run each named script through one.sh at once",
    "run4.sh": "#!/bin/bash\n# one.sh with coq on b_cylv2_82.g",
    "same.py": '"""Check that each script 464.15 appends whole',
    "t_axis.g": "# One component of cyl.g alone",
    "t_perm2.g": "# The permanent alone",
}
found = {}
for lang, body in blocks:
    for name, mark in marks.items():
        if not body.startswith(mark):
            continue
        if name in ("m2.g", "m3.g") and (name == "m3.g") != ("dHx" in body):
            continue
        assert name not in found, name
        found[name] = body
for name in marks:
    if name not in found:
        print(name, "not appended")
        continue
    if name == "same.py":
        path = os.path.abspath(__file__)
    elif name == "run1.sh":
        path = os.path.join(cyl2, name)
    elif name.startswith("t_"):
        path = os.path.join(cyl3, "..", "cyl", name)
    else:
        path = os.path.join(cyl3, name)
    if not os.path.exists(path):
        print(name, "appended, no file to compare")
        continue
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")

s13 = text[text.index("### 464.13"):text.index("### 464.14")]
one = [body for lang, body in FENCED.findall(s13)
       if body.startswith("#!/bin/bash\n# Run one Gappa script under 464.4's pin")]
assert len(one) == 1, len(one)
print("one.sh is 464.13's block:", open(os.path.join(cyl3, "one.sh")).read() == one[0])


def compare(fresh, ran):
    written = sorted(f for f in os.listdir(fresh) if f.endswith(".g"))
    same = [f for f in written if os.path.exists(os.path.join(ran, f))
            and filecmp.cmp(os.path.join(fresh, f), os.path.join(ran, f), shallow=False)]
    differ = [f for f in written if os.path.exists(os.path.join(ran, f)) and f not in same]
    absent = [f for f in written if not os.path.exists(os.path.join(ran, f))]
    return written, same, differ, absent


fresh = tempfile.mkdtemp()
for name in ("cyl.g", "cylcases.py", "vgen.py", "cylsplit.py", "dbl.g"):
    if name in found:
        open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "cylsplit.py"], cwd=fresh, check=True)
subprocess.run(["python3", "cylcases.py", "61", "62", "63", "64", "65", "66", "67", "68", "69"], cwd=fresh, check=True)
subprocess.run(["python3", "vgen.py"], cwd=fresh, check=True)
for m in (81, 82, 83):
    t = open(os.path.join(fresh, "cylv2.g")).read()
    assert t.count("B = 10b-53 + 5b-102;\n") == 1
    open(os.path.join(fresh, f"b_cylv2_{m}.g"), "w").write(t.replace("B = 10b-53 + 5b-102;\n", f"B = 10b-53 + {m}b-106;\n"))
written, same, differ, absent = compare(fresh, cyl3)
print("written again:", len(written), "same as run:", len(same), "differ:", differ, "not in cyl3:", absent)
print("cyl_nnn.g is cyl.g:", filecmp.cmp(os.path.join(fresh, "cyl_nnn.g"), os.path.join(fresh, "cyl.g"), shallow=False))
shutil.rmtree(fresh)

fresh = tempfile.mkdtemp()
for name in ("cyl.g", "cylcases.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "cylcases.py", "0", "40", "60", "70", "80", "90", "100", "120", "160"], cwd=fresh, check=True)
written, same, differ, absent = compare(fresh, cyl2)
print("run1.sh's written again:", len(written), "same as run:", len(same), "differ:", differ, "not in cyl2:", absent)
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.15: 20
cyl.g same
cylcases.py same
run1.sh same
vgen.py same
cylsplit.py same
run2.sh same
run3.sh same
tally3.py same
dbl.g same
m2.g same
m3.g same
one1.sh same
par.sh same
run4.sh same
same.py same
t_axis.g same
t_perm2.g same
one.sh is 464.13's block: True
written again: 164 same as run: 164 differ: [] not in cyl3: []
cyl_nnn.g is cyl.g: True
run1.sh's written again: 24 same as run: 24 differ: [] not in cyl2: []
```

**10. What this leaves.** The rows filed from this write-up while it was
written:

- Q2771: prove the bound where a difference, a direction component or the
  radius is zero, the axis-aligned cylinder among them. `product_is_normal(a,
  b)` is true when `a` or `b` is zero (`cascade.rs:67-69`), so these cases pass
  every check `on_floats` makes, and no script here covers them. Not run, so no
  class (§360.5).
- Q2772: restate `cylinder.rs:541-552` and `312-314` by 5. What the `80ε²` has
  to beat is `ε²`, not `ε`, and by the comment's own terms `45 + 60 + 10` make
  115. Checking the components does not cover the permanent's reach squares
  when a component cancels, though the bound holds there (6).
- Q2773: run `cylsplit.py` with one, two and three of the products `on_floats`
  checks below the normals, as 464.14's 6 ran the sphere's squares. Then say
  whether `on_floats` needs its `product_is_normal` checks, and restate
  `cylinder.rs:308-310` by what that shows. Not run, so no class. Q2767 waits
  on it, as well as on Q2684, Q2760 and Q2766.

And what is not a row:

- Q2700 already waits on Q2680, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- The claim on `value` at `10u + 82u²` against the code's `80` (4) is a limit
  of the two forms as run, not a wrong answer: nobody found an input where
  `value` lands past `eb`, and `decide` needs only `y`. Q2772 restates the
  comment.
- How `c_cyl_noeb.g` runs past 600 s (7) was not traced.
- `cyl.g`'s first `restates` line needs no row: `just src on_floats
  crates/rcad-geom/src/cylinder.rs` prints the one item, at `cylinder.rs:300`.
- `field` on these scripts' hints (464.10).

### 464.16 The side of rcad-geom's cone

Q2681 took up `SIDE_OF_CONE_ERROR_BOUND` in `crates/rcad-geom/src/cone.rs`,
in six lanes on 2026-10-25. The first four were the first lane and its three
relays. The first wrote and started `cone.g`, and found the wrong sign of 6
reading `on_floats`. The second ran the bisection of 3 and the controls of 7,
and wrote `sumexact.g` and `conesplit.py`. The third ran the split of 6. The
fourth checked every place and figure here against the source and the logs,
and drafted 1 to 10. The fifth, the second lane, merged the first lane's
branch, wrote 1 to 10 and appended `cone.g`, `sumexact.g` and the two
generators. The sixth, its relay, appended the other scripts and wrote and
ran `same.py`. The bound is proved on `y`,
the sum before `value`'s last rounding, at the code's constant wherever no
factor is zero and the permanent's axial square `R2` does not round to zero.
That covers no component of the cross product cancelling, any of them
cancelling whatever its reach square rounds to, and the dot product down the
axis, `up`, cancelling, so long as `R2` is not zero. Coq checks each proof,
modulo its hints. Where `up` cancels and `R2` rounds to zero, level 1
certifies a wrong sign, and a probe shows one at a slope of `2⁵¹¹`. By hand
arithmetic that shape needs a slope of about `2⁷⁸` or more, so it is class
`wrong/D` (6). The claim on `value` itself was not tried, and `decide` does
not need it (4). The branches where a difference or a direction component is
zero, which `product_is_normal` lets through, are not run. And the comments
claim more than is proved. The doc comment on the constant writes `66ε` where
it means `66ε²` and miscounts its own terms, and it says `on_floats` enforces
the bottom end product by product, which is false: it checks neither `R2` nor
the axial term `A`, and that is where the wrong sign lives (5). Fixing them,
the wrong sign, and running what is not run is four rows (10).

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, against
`cone.rs` and `cascade.rs` at `2538ee89`. `git log --oneline
2538ee89e..61ada239e` on `cone.rs`, `cascade.rs`, `three_surfaces.rs`,
`rcad-topo`'s `validate.rs`, `rcad-step`'s `brep.rs` and `rcad-geom`'s
`lib.rs` prints nothing, so each is as it stands at `61ada239e`, where this
write-up was finished. `u` is `2⁻⁵³`, and the code's `B = (12 + 96ε) ε`
(`cone.rs:664-667`) is `12u + 96u²`, written `B = 12b-53 + 3b-101;`. `96ε` is
`3 · 2⁻⁴⁸`, a multiple of the ulp of 12, so the Rust's sum and product are
exact and a double holds `B`. Each script follows 464.8's pattern: one ratio
hint per rounding; the error split into one term per piece of the permanent,
so that Gappa averages them; 464.5's lemma for each product `on_floats`
checks is normal; and the step from `y` to the sign as 464.4 gives it.

**1. What the proof guards.** `Cone::side_of` (`cone.rs:401-411`), the cone's
`SideOfSurface`, runs the cascade on `HowFarFromTheSkin`, once a guard has
refused a point or a cone that is not real numbers. `Surface::Cone` reaches it
from `three_surfaces.rs:3296`, from `rcad-topo`'s `validate.rs:4499` and from
`rcad-step`'s `brep.rs:3048`, and `lib.rs:95` exports `Cone` and
`SIDE_OF_CONE`. A user makes a cone with `Cone::new` (`cone.rs:87`), which
takes any finite positive slope, or with `Cone::from_half_angle`
(`cone.rs:114`), whose slope is the tangent of a half-angle below a right
angle. So, like 464.15's, this proof guards a predicate a user reaches.

**2. `on_floats`, line for line.** `HowFarFromTheSkin::on_floats`
(`cone.rs:424-469`) returns `None` for a point that is not exact `f64`s
(`cone.rs:425`) and takes the three differences `fi` from the apex to the
point (`cone.rs:429`). It forms the six halves of their cross product with the
direction, returning `None` unless `product_is_normal` holds for each half's
two factors (`cross_product_halves_if_normal`, `cone.rs:584-594`). It takes
the three components `h − k`, the three products `ui = fi · di`, and `up =
rnd(rnd(u0 + u1) + u2)` (`cone.rs:435-438`). It returns `None` unless
`product_is_normal` holds for each component with itself, each `fi` with `di`,
the slope with itself, `up` with itself, and `slope²` with `up²`
(`stays_accurate`, `cone.rs:439-448`). Last it returns `value = s0 + s1 + s2 −
along` and `error_bound = SIDE_OF_CONE_ERROR_BOUND * permanent`
(`cone.rs:465-468`). Here `along = rnd(rnd(slope²) · rnd(up²))`
(`cone.rs:451`), and the permanent is each half pair's reach squared, `(|h| +
|k|)²`, summed with the axial term `A` (`cone.rs:458-463`).

`cone.g` (9) writes that operation for operation. The cross product is
`cyl.g`'s (464.15's 2). Then `ui = rnd(fi · di)`, `u01 = rnd(u0 + u1)`, `up =
rnd(u01 + u2)`, `ss = rnd(sl · sl)`, `w = rnd(up · up)`, `along = rnd(ss · w)`,
`y = s012 − along` and `value = rnd(y)`. The permanent's axial term is `A =
rnd(ss · R2)`, with `R2 = rnd(R · R)` and `R = rnd(rnd(|u0| + |u1|) + |u2|)`,
and `P = rnd(rnd(rnd(vx + vy) + vz) + A)`. On the axial chain that is eleven
roundings to `y` and twelve to `value`: `f`, `u`, `u01` and `up` count twice,
since `up` is squared, then `w`, `ss` and `along`. The cross product's chain
is nine to `y`, as the cylinder's.

Nothing checks `R2` or `A`. Where `up` does not cancel, `|up| ≤ R` by monotone
rounding, so `R2 ≥ w` and `A ≥ along`, both normal. Where `up` cancels to
zero, `w` and `along` are zero, both checks pass on the zero
(`cascade.rs:67-69`), and `R2` can round to zero while the true `slope² · (x ·
w)²` it stood for is not small against the rest.

**How the error splits.** Each component's square splits into three terms, as
in 464.15. The axial term `y` carries is `ss (g0 u0 + g1 u1 + g2 u2)² KA`,
`KA` the roundings of `w` and `along` and `g` those of the two sums. The true
one is `sl² Dt² = ss (e0 u0 + e1 u1 + e2 u2)² / rss`, `e` undoing the roundings
of `f` and of each product, and `rss` that of `ss`. So its error is six terms,
one for each piece of `ss G²`, with `G = |u0| + |u1| + |u2|`: `ss ui²` for each
`i`, and `2 ss |ui| |uj|` for each pair. With the components' nine, the error
is averaged over fifteen pieces, `P15`. They are summed in four blocks, the
three components and the axial term, since 464.15's 3 found one hint over
every piece too slow.

**3. The bound on `y`.** `cone.g` (9) proves `|y − Ex| ≤ eb` at the code's
`B`. In `run0.sh` (9) it exits 0 in `9.23s`, printing `(y − Ex) / P15` within
`1.22125e-15` and `P −/ Phat` within `7.77156e-16`. Written as `k u + r u²`,
the exact bounds Gappa prints are `11u + 10u²` and `7u + 22u²`. `Phat` is the
permanent with no rounding of its own. The bisection: `run1.sh` and `run2.sh`
(9) ran the `b_cone_<m>.g` and `b11_cone_<m>.g` that `conecases.py` (9)
writes, `cone.g` at `B = 12u + m u²` and at `11u + m u²`. At `12u`, every `m`
run proves: 0, 40, 60, 80, 100 and 120. At `11u`, 99, 100, 150, 200, 300 and
400 prove, and 98, 97, 96, 90, 50 and 0 fail, best `[-1, 1]`. By hand, `(11u +
10u²)(1 + 7u)(1 + u)` (`y`'s bound taken through the permanent's `7u` and
through `eb`'s own rounding) is `11u + 98u²` and a `u³`, so the line falls
where that count puts it. So on `y`, where nothing cancels, the bound needs
about a `u` less than the code's `12u`.

**4. The claim on `value`.** Not tried. `decide` does not need it: the doc on
`Filtered::error_bound` (`cascade.rs:25-35`) asks only for the bound on the
sum before `value`'s last rounding, which 3 proves, as in 464.14's 4 and
464.15's 4.

**5. What the comments claim.** The doc comment on the constant,
`cone.rs:643-663`, says:

> The axial half is one step longer: [...] squaring that is a fifth and
> doubles what it carries so it sits at `(1 + epsilon)^9`, and the two
> multiplications by the slope are a tenth and eleventh. Subtracting it from
> the cross-product sum is a twelfth. `(1 + epsilon)^12 − 1 ≤ 12·epsilon +
> 66·epsilon² + …` times the permanent.
>
> The `96·epsilon²` covers the higher-order terms and the rounding of the
> permanent itself; what it has to beat is about `66·epsilon`.
>
> Every step of that argument is a *relative* one, so as with every bound of
> this shape it holds only while nothing overflows or falls into the
> subnormals. [`HowFarFromTheSkin::on_floats`] enforces the bottom end product
> by product through [`product_is_normal`], and `decide` the top end by
> refusing a bound that is not itself a normal number.

The comment above the check, at `cone.rs:432-434`, says:

> The bound below is a fraction of the permanent, which only bounds anything
> while every product keeps its relative accuracy. One of them in the
> subnormals and this level certifies a wrong sign.

Against what is proved:

- Twelve roundings to `value` and eleven to `y` agree with `cone.g`'s `11u` on
  `y` (3).
- "About `66·epsilon`" is the `ε²` term of `(1 + ε)¹² − 1`, so it gives the
  wrong power, as 464.15's 5 found the cylinder's `45` does. It also leaves
  out what the same sentence says the `96ε²` covers. The permanent's rounding,
  `7u` (`cone.g`), taken against `12ε`, costs `84ε²`. The rounding of `B · P`
  costs `ε` of `eb`, which is `12ε²`. So by the comment's own terms the count
  is `66 + 84 + 12 = 162`, above its `96`. On `y` that costs nothing, since
  `y` proves at `11u + 99u²` and at `12u + 0u²` (3).
- "Enforces the bottom end product by product" is false. `on_floats` checks
  neither `R2` nor `A`, and `product_is_normal` lets a zero through
  (`cascade.rs:67-69`). So where `up` cancels, `w` and `along` are zero, both
  checks pass, and `R2` and `A` can fall below the normals or round to zero.
  The bound holds in all of those but one: `R2` rounding to zero, where level
  1 certifies a wrong sign (6). The branches where a difference or a direction
  component is zero are not run (10).
- "One of them in the subnormals and this level certifies a wrong sign" is
  true of `R2` rounding to zero, the probe of 6. It is false of `R2` below the
  normals and not zero, of `A` below the normals or zero, and of the reach
  squares below the normals or zero: by 6 each of those proves. For the
  products `on_floats` checks it was not run, as 464.14's 6 ran the sphere's
  squares (10).

So the comments claim more than is proved, which is what the row asks, and the
claim that the bottom end is enforced is false in a way that gives a wrong
sign. Restating them is a row, and the wrong sign another (10).

**6. The components and the axial term that cancel.** `conesplit.py` (9)
splits `cone.g` as 464.15's `cylsplit.py` splits `cyl.g`, with a fourth letter
for the axial term. Each component of the cross product is `n`, `c`, `s` or
`u`, as in 464.15's 6. The axial term is in one of six states:

- `n`: `up` is not zero, and `u01` is normal. `w` and `along` are normal,
  which `on_floats` checks, and `|up| ≤ R` by monotone rounding, so `R2 ≥ w`
  and `A ≥ along` are normal too.
- `e`: `up` is not zero, and `u01` is below the normals, where it is exact.
  `sumexact.g` (9) shows that a sum of two doubles at most `2⁻¹⁰²²` is a
  double: it exits 0 in `0.03s`.
- `c`: `up` cancels to zero, `u01 + u2 = 0`, and `R2` and `A` are normal.
- `a`: `up` cancels, `R2` is normal, and `A` is below the normals or zero.
- `s`: `up` cancels, `R2` is below the normals and not zero, and `A` is
  normal.
- `t`: `up` cancels, `R2` is below the normals and not zero, and `A` is below
  the normals or zero.

These cover every axial term whose products are normal but one: `up` cancels
and `R2` rounds to zero. That one gets no script, since it is where level 1
certifies a wrong sign (below). Where `up` cancels, `y` carries no axial term,
so its error is the whole of the true one, `sl² Dt²`, where `Dt` is what the
roundings of `f`, of each product and of `u01` left. That error is averaged
over `XA = ss · R2` rather than over `ss G²`, since `R2` below the normals need
not stay near `G²`. That makes `4³ · 6 = 384` cases, written `cone_<case>.g`,
and `cone_nnnn.g` is `cone.g` byte for byte, which `same.py` (9) checks.

What they print. `run3a.sh` (9) first ran `nnne`, `nnnc`, `nnna`, `nnns` and
`nnnt`, and each exits 0, in 8.80 to 10.88 s, and their `u / 2` controls exit
1. `run3.sh` (9) then ran all 384, 22 at a time on a load of about 43 over 24
cores, and `tally3.py` (9) counts them:

- Each of the 384 exits 0 at the code's `B`, in 4.17 s (`cone_ccuc.g`) to
  39.13 s (`cone_nnnt.g`).
- `(y − Ex) / P15` is within `11u` (`1.22125e-15`) in all 64 cases with axial
  `n`, and within `9u` (`9.99201e-16`) in all 64 with `e`. Where `up` cancels
  it is within `9u`, `8u` and `7u` with three, two and one components `n`.
  With none, `y` is 0 and the error is a `u²` of the permanent: within `9u²`
  (`1.10934e-31`) for `c`, `13.5u²` (`1.664e-31`) for `s`, and `4u²`
  (`4.93038e-32`) for `a` and `t`.
- 16 cases, `{s,u}³` with axial `a` or `t`, print "remaining results are
  pointless": their hypotheses contradict. With every reach square and `A` at
  most `2⁻¹⁰²²`, `P ≤ 2⁻¹⁰²⁰`, so `eb = rnd(B · P)` cannot be normal, and
  `decide` refuses there.
- `P −/ Phat`, or `P −/ Phatx` where a square is below the normals, is within
  `7u` in 126 cases, `6u` in 62, `5u` in 76, `4u` in 36, `3u` in 40, `2u` in 26
  and `u` in 2.
- No `.out`, `.bcoq` or `.coqc` of the 384 has a line other than its results
  and "assumed to be nonzero" warnings.

So on `y` the bound holds at the code's constant wherever no factor is zero
and `R2` does not round to zero.

The shapes. None of the 384 was run as Rust input. By hand arithmetic, not
run: put the apex at 0 and the axis along `(1, 1, 1)`, which `Axis::new`
scales to length one as three equal components `c`, and take the point `(1,
1, −2)`. The products are `c`, `c` and `−2c`, each exact, `u01 = 2c` is exact,
and `up` cancels to zero with `R2` normal. The halves are `(c, −2c)`, `(−2c,
c)` and `(c, c)`, so the third component cancels too, with its reach square
normal. So that is `nncc`: a point on the plane through the apex square to the
axis, a shape a designer makes. The fourth lane's draft had the direction `(1,
2, 3)` and the point `(1, 1, −1)` here, but `Axis::new` scales that direction
to length one, so its products are not the `1`, `2` and `−3` that cancel.

**A wrong sign.** The probe below, run once with `mise exec -- just t
probe_q2681_axial_reach_square_rounding_to_zero --no-capture` and then
removed, builds that shape: the apex at the origin, the direction `(2⁻⁶⁰ (1 +
2⁻²⁶), 1, 0)`, which `Axis::new` keeps as the bits `3c30000004000000`,
`3ff0000000000000` and `0`, the slope `2⁵¹¹`, and the point `(f0, −rnd(f0 ·
d0), 0)` with `f0 = 2⁻⁴⁸⁰ (1 + 2⁻²⁸)`. `u0` and `u1` cancel exactly, `R =
2⁻⁵³⁹`, `R²` rounds to zero, and so does `A`. It printed:

```text
on_floats value 1.0261342079698896e-289 error_bound 1.3670853888524055e-304
on_intervals [-2.220446049250313e-16, 1.0261342079698903e-289]
exactly None
decide Pos at Float
side_of Pos
```

`truth.py` below works the same doubles in exact rationals and printed:

```text
dot = 1.542348713665846e-179 = 2^ -594
cross squared = 1.0261342079698896e-289
slope^2 dot^2 = 1.0691058840368783e-50
true value = -1.0691058840368783e-50 sign -1
```

So `Cone::side_of` certifies `Pos`, outside, at level 1, for a point whose
true sign is `Neg`, inside. Class `wrong/D`: on paper, not run, it needs `R² ≤
2⁻¹⁰⁷⁵`, so every `|fi · di|` below about `2⁻⁵³⁷`, and a slope of at least
about `2⁷⁸` so that `slope² · (x · w)²`, at most about `9u² R² slope²`, can pass
`eb ≥ 2⁻¹⁰²²`. A slope of `2⁷⁸` is a cone within `2⁻⁷⁸` of a flat plane, a
ratio far past §3.5's `1e11`, and `Cone::from_half_angle` cannot make one:
`tan` of the double below `π/2` is about `1.6e16`. The probe, whole, as it
stood at the top of `cone.rs`'s `mod tests`:

```rust
#[test]
fn probe_q2681_axial_reach_square_rounding_to_zero() {
    let d0_wanted = 2f64.powi(-60) * (1.0 + 2f64.powi(-26));
    let axis = Axis::new(Point::origin(), Vector::new(d0_wanted, 1.0, 0.0)).unwrap();
    let d = axis.direction().components();
    let f0 = 2f64.powi(-480) * (1.0 + 2f64.powi(-28));
    let u0 = f0 * d[0];
    let f1 = -u0 / d[1];
    let slope = 2f64.powi(511);
    let cone = Cone::new(axis, slope).unwrap();
    let point = PredicatePoint::from([f0, f1, 0.0]);
    let pred = HowFarFromTheSkin { cone, point };
    let filtered = pred.on_floats();
    println!("d = {:?} bits {:x} {:x} {:x}", d, d[0].to_bits(), d[1].to_bits(), d[2].to_bits());
    println!("f = {:?} bits {:x} {:x}", [f0, f1], f0.to_bits(), f1.to_bits());
    println!("slope bits {:x}", slope.to_bits());
    match filtered {
        Some(Filtered { value, error_bound }) => println!("on_floats value {value:e} error_bound {error_bound:e}"),
        None => println!("on_floats None"),
    }
    println!("on_intervals {:?}", pred.on_intervals());
    println!("exactly {:?}", pred.exactly());
    let (decision, level) = decide(&pred);
    println!("decide {decision:?} at {level:?}");
    println!("side_of {:?}", cone.side_of(point));
}
```

It also printed `f = [3.203332964226307e-145, -2.778448488608982e-163] bits
21f0000001000000 9e30000005000000` and `slope bits 5fe0000000000000`, which
`truth.py` reads:

```python
"""The true sign of the cone probe's input, in exact rationals."""
import struct
from fractions import Fraction


def f(bits):
    return Fraction(struct.unpack(">d", bytes.fromhex(bits))[0])


d = [f("3c30000004000000"), f("3ff0000000000000"), Fraction(0)]
p = [f("21f0000001000000"), f("9e30000005000000"), Fraction(0)]
s = f("5fe0000000000000")
c = [p[1] * d[2] - p[2] * d[1], p[2] * d[0] - p[0] * d[2], p[0] * d[1] - p[1] * d[0]]
dot = sum(a * b for a, b in zip(p, d))
cross2 = sum(x * x for x in c)
axial = s * s * dot * dot
value = cross2 - axial
print("dot =", float(dot), "= 2^", dot.numerator.bit_length() - dot.denominator.bit_length())
print("cross squared =", float(cross2))
print("slope^2 dot^2 =", float(axial))
print("true value =", float(value), "sign", (value > 0) - (value < 0))
```

**7. The controls.** At `B = 1b-54;`, `u / 2`, each must fail:

- `c_cone_half.g` exits 1, best `[-22, 22]`, `cone.g`'s `11u` over `u / 2`.
- Of the 383 `c_cone_<case>_half.g`, 275 exit 1, each case's own bound over
  `u / 2`: best `[-22, 22]` in 63, `[-18, 18]` in 68, `[-16, 16]` in 36 and
  `[-14, 14]` in 108.
- The other 108 exit 0. They are the cases with no component `n` and `up`
  cancelling, where `y` is 0 and its error a `u²` that `u / 2` covers. So
  `conesplit.py` writes each a second control, `c_cone_<case>_usq.g`, at `B =
  1b-106;`, `u²`. `run4.sh` (9) first checked by `md5sum` that writing the
  split again left the 767 scripts `run3.sh` ran unchanged. It then ran the
  108: 92 exit 1, best `−4` or `9` or `13.5` over `u²`, and the 16 pointless
  ones exit 0, pointless again.

Together with the failures at `11u + 98u²` (3), that says none of the proofs
came from a contradiction in its hypotheses, but for the 16 that Gappa itself
calls pointless. Two more controls take a hypothesis away:

- `c_cone_nonormal.g` is `cone.g` without every `1b-1022` hypothesis but
  `|eb|`'s, and without the lemma line each of those gives. It exits 1, best
  `[-3.80599e+2143, 3.80599e+2143]`, and fails `REL(P, Phat)` and `BND((y -
  Ex) / P15)` too.
- `c_cone_noeb.g`, `cone.g` without `eb` normal, exits 1 in `121.00s`, best
  `[-3.76818e+1267, 3.99163e+1402]`.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each script that proves and was run through them, and no `coqc` printed
anything:

- `cone.g`: 18049 lines with 64 hints, in `15.75s` (`run0.sh`).
- The 384 cases of 6: 2325 (`cone_uuua`) to 17532 lines (`cone_nnnn`), with 37
  to 64 hints, in 6.68 to 51.46 s each (`run3.sh`).
- `sumexact.g`: 150 lines with no hints. The fifth lane ran it again with
  `./one.sh sumexact.g coq`, which printed `sumexact exit 0 0.03s` and
  `sumexact coq -Bcoq exit 0 coqc exit 0 1.12s lines 150 hints 0`.

Gappa does not write the same proof each run (464.4), which is why
`cone_nnnn.g`, the same bytes as `cone.g`, gave 17532 lines against 18049.
Each hint enters the Coq file as a hypothesis with no condition, as in 464.4.
The controls and the bisection scripts were not run through `coqc`. `field`
has not been run on these hints (464.10).

**9. The scripts.** `conecases.py` writes the controls of 7 and the bisection
scripts of 3 from `cone.g`, and `conesplit.py` writes the 384 cases of 6 and
their controls. To run them again, save the nix block of 464.5 as
`gappalib18.nix` and `cone.g`, `sumexact.g` and the two generators beside it,
and then:

```sh
python3 conecases.py 0 40 60 80 100 120 150 200 300 400; python3 conesplit.py
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 cone.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq cone.g > cone.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc cone.v
```

and the same for every other file they write, and for `sumexact.g`. A script
that proves exits 0; a control must exit 1. What the lanes ran is `run0.sh` to
`run4.sh` and `run3a.sh` (below), each through 464.13's `one.sh`, the same
bytes, or through `one1.sh`, which is `one.sh` with the timeout taken from `T`
and no `coq`. `run3.sh` and `run4.sh` were started through `launch.sh`.
`tally3.py` counts what `run3.sh` printed. `same.py` (below) checks every
script appended here byte for byte against the file that was run, that
`tally3.py` run again prints the counts given here, and that the two
generators, run again from the text here, write what was run; its print and
what its 911 files are end this part. `truth.py` is in 6, and `same.py` checks it too. The
probe's diff is not appended, since 6 gives the test whole.

`cone.g`, whole, as every run of 3, 7 and 8 ran it:

```gappa
# restates: crates/rcad-geom/src/cone.rs on_floats at 2538ee89
# restates: crates/rcad-geom/src/cone.rs cross_product_halves_if_normal at 2538ee89
# restates: crates/rcad-geom/src/cone.rs cross_product_halves at 2538ee89
# restates: crates/rcad-geom/src/cone.rs SIDE_OF_CONE_ERROR_BOUND at 2538ee89
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 2538ee89
# restates: crates/rcad-predicates/src/cascade.rs decide at 2538ee89
# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.
@rnd = float<ieee_64, ne>;

# The point, the apex, the axis direction and the slope, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
sl = rnd(sl_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
hx = rnd(fy * dz); kx = rnd(fz * dy);
hy = rnd(fz * dx); ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = rnd(fy * dx);
Qx = hx - kx; Qy = hy - ky; Qz = hz - kz;
qx = rnd(Qx); qy = rnd(Qy); qz = rnd(Qz);

# up_halves, and up, summed left to right as Rust adds them.
ux = rnd(fx * dx); uy = rnd(fy * dy); uz = rnd(fz * dz);
U01 = ux + uy; u01 = rnd(U01);
U = u01 + uz; up = rnd(U);

sx = rnd(qx * qx); sy = rnd(qy * qy); sz = rnd(qz * qz);
ss = rnd(sl * sl);
w = rnd(up * up);
along = rnd(ss * w);
S01 = sx + sy; s01 = rnd(S01);
S012 = s01 + sz; s012 = rnd(S012);
y = s012 - along;
value = rnd(y);

# The permanent: each half's reach squared, then the slope squared times the
# reach of up squared.
nx = rnd(|hx| + |kx|); ny = rnd(|hy| + |ky|); nz = rnd(|hz| + |kz|);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
R01 = rnd(|ux| + |uy|); R = rnd(R01 + |uz|);
R2 = rnd(R * R);
A = rnd(ss * R2);
P = rnd(rnd(rnd(vx + vy) + vz) + A);

B = 12b-53 + 3b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Dt = Fx * dx + Fy * dy + Fz * dz;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - sl * sl * (Dt * Dt);

# The permanent with no rounding of its own.
G = |ux| + |uy| + |uz|;
Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
     + (|hz| + |kz|) * (|hz| + |kz|) + ss * (G * G);

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rqx = qx / Qx; rqy = qy / Qy; rqz = qz / Qz;
rsx = sx / (qx * qx); rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs01 = s01 / S01; rs012 = s012 / S012;
rux = ux / (fx * dx); ruy = uy / (fy * dy); ruz = uz / (fz * dz);
ru01 = u01 / U01; rU = up / U;
rss = ss / (sl * sl); rw = w / (up * up); ral = along / (ss * w);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
Kx = rqx * rqx * rsx * rs01 * rs012;
Ky = rqy * rqy * rsy * rs01 * rs012;
Kz = rqz * rqz * rsz * rs012;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
WHx = Kx - ax * ax; WKx = Kx - bx * bx; WXx = Kx - ax * bx;
WHy = Ky - ay * ay; WKy = Ky - by * by; WXy = Ky - ay * by;
WHz = Kz - az * az; WKz = Kz - bz * bz; WXz = Kz - az * bz;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
cHy = |hy| * |hy|; cKy = |ky| * |ky|; cXy = 2 * |hy| * |ky|;
cHz = |hz| * |hz|; cKz = |kz| * |kz|; cXz = 2 * |hz| * |kz|;
eHx = hx * hx * WHx; eKx = kx * kx * WKx; eXx = -(2 * hx * kx * WXx);
eHy = hy * hy * WHy; eKy = ky * ky * WKy; eXy = -(2 * hy * ky * WXy);
eHz = hz * hz * WHz; eKz = kz * kz * WKz; eXz = -(2 * hz * kz * WXz);

# The axial term, as y carries it, is ss (gx ux + gy uy + gz uz)^2 times KA,
# g undoing nothing and folding in the roundings of the two sums; the true
# one is sl^2 (Dt)^2 = ss (ex ux + ey uy + ez uz)^2 / rss, e undoing the
# roundings of f and of the product. So its error is six terms, one for each
# piece of ss G^2: ss u^2 for each of the three, and 2 ss |u| |u'| for each
# pair. W is each term's error over its piece.
KA = rw * ral;
gx = ru01 * rU; gy = ru01 * rU; gz = rU;
ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);
WAxx = KA * gx * gx - ex * ex / rss;
WAyy = KA * gy * gy - ey * ey / rss;
WAzz = KA * gz * gz - ez * ez / rss;
WAxy = KA * gx * gy - ex * ey / rss;
WAxz = KA * gx * gz - ex * ez / rss;
WAyz = KA * gy * gz - ey * ez / rss;
cAxx = ss * |ux| * |ux|; cAyy = ss * |uy| * |uy|; cAzz = ss * |uz| * |uz|;
cAxy = 2 * ss * |ux| * |uy|; cAxz = 2 * ss * |ux| * |uz|; cAyz = 2 * ss * |uy| * |uz|;
eAxx = -(ss * ux * ux * WAxx); eAyy = -(ss * uy * uy * WAyy); eAzz = -(ss * uz * uz * WAzz);
eAxy = -(2 * ss * ux * uy * WAxy); eAxz = -(2 * ss * ux * uz * WAxz);
eAyz = -(2 * ss * uy * uz * WAyz);

# The same permanent, as four blocks of the fifteen pieces the error is
# averaged over: each component's reach squared, and the axial term.
P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;
SA = cAxx + cAyy + cAzz + cAxy + cAxz + cAyz;
P15 = P3x + P3y + P3z + SA;

# Each component's square as y carries it, and the axial term as Ex has it.
Yx = Kx * Qx * Qx; Yy = Ky * Qy * Qy; Yz = Kz * Qz * Qz;
At = sl * sl * (Dt * Dt);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |sl| <= 1b1024
  # No factor is zero, no component cancels to zero and nor does up. For an
  # f64 that is the same as being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |sl| >= 1b-1074
  /\ |qx| >= 1b-1074 /\ |qy| >= 1b-1074 /\ |qz| >= 1b-1074
  /\ |up| >= 1b-1074
  # The first sum of up is a normal number; where it is not, it is exact,
  # and that is another case.
  /\ |u01| >= 1b-1022
  # product_is_normal on the six halves (cross_product_halves_if_normal),
  # and on the three components, the three up halves, the slope, up and the
  # axial term (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |sx| >= 1b-1022 /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |ux| >= 1b-1022 /\ |uy| >= 1b-1022 /\ |uz| >= 1b-1022
  /\ |ss| >= 1b-1022 /\ |w| >= 1b-1022 /\ |along| >= 1b-1022
  # The permanent's three squares are normal too, since no component is
  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,
  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal. In the same way
  # |up| <= R, so R2 >= w and A >= along, both normal.
  /\ |vx| >= 1b-1022 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  /\ |R2| >= 1b-1022 /\ |A| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ux -/ fx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ uy -/ fy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ uz -/ fz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ u01 -/ U01 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ss -/ sl * sl in [-4503599627370497b-105, 4503599627370497b-105]
  /\ w -/ up * up in [-4503599627370497b-105, 4503599627370497b-105]
  /\ along -/ ss * w in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ R2 -/ R * R in [-4503599627370497b-105, 4503599627370497b-105]
  /\ A -/ ss * R2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  /\ hy / |hy| in [-1, 1] /\ ky / |ky| in [-1, 1]
  /\ hz / |hz| in [-1, 1] /\ kz / |kz| in [-1, 1]
  /\ ux / |ux| in [-1, 1] /\ uy / |uy| in [-1, 1] /\ uz / |uz| in [-1, 1]
->
  (y - Ex) / P15 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
qx / Qx -> 1 + (qx - Qx) / Qx;
qy / Qy -> 1 + (qy - Qy) / Qy;
qz / Qz -> 1 + (qz - Qz) / Qz;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
ux / (fx * dx) -> 1 + (ux - fx * dx) / (fx * dx);
uy / (fy * dy) -> 1 + (uy - fy * dy) / (fy * dy);
uz / (fz * dz) -> 1 + (uz - fz * dz) / (fz * dz);
u01 / U01 -> 1 + (u01 - U01) / U01;
up / U -> 1 + (up - U) / U;
ss / (sl * sl) -> 1 + (ss - sl * sl) / (sl * sl);
w / (up * up) -> 1 + (w - up * up) / (up * up);
along / (ss * w) -> 1 + (along - ss * w) / (ss * w);

# The error is the sum of four blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. In blocks so that no hint's identity spans all fifteen
# pieces, as cyl.g learned.
(y - Ex) / P15 -> ((P15 + (y - Ex)) - P15) / P15;
P15 + (y - Ex) -> (P3x + (Yx - Cx * Cx)) + (P3y + (Yy - Cy * Cy)) + (P3z + (Yz - Cz * Cz)) + (SA + (At - along));
P3x + (Yx - Cx * Cx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
P3y + (Yy - Cy * Cy) -> (cHy + eHy) + (cKy + eKy) + (cXy + eXy);
P3z + (Yz - Cz * Cz) -> (cHz + eHz) + (cKz + eKz) + (cXz + eXz);
SA + (At - along) -> (cAxx + eAxx) + (cAyy + eAyy) + (cAzz + eAzz) + (cAxy + eAxy) + (cAxz + eAxz) + (cAyz + eAyz);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
(cHy + eHy) - cHy -> eHy;
(cKy + eKy) - cKy -> eKy;
(cXy + eXy) - cXy -> eXy;
(cHz + eHz) - cHz -> eHz;
(cKz + eKz) - cKz -> eKz;
(cXz + eXz) - cXz -> eXz;
(cAxx + eAxx) - cAxx -> eAxx;
(cAyy + eAyy) - cAyy -> eAyy;
(cAzz + eAzz) - cAzz -> eAzz;
(cAxy + eAxy) - cAxy -> eAxy;
(cAxz + eAxz) - cAxz -> eAxz;
(cAyz + eAyz) - cAyz -> eAyz;
eHx / cHx -> (hx / |hx|) * (hx / |hx|) * WHx;
eKx / cKx -> (kx / |kx|) * (kx / |kx|) * WKx;
eXx / cXx -> -((hx / |hx|) * (kx / |kx|) * WXx);
eHy / cHy -> (hy / |hy|) * (hy / |hy|) * WHy;
eKy / cKy -> (ky / |ky|) * (ky / |ky|) * WKy;
eXy / cXy -> -((hy / |hy|) * (ky / |ky|) * WXy);
eHz / cHz -> (hz / |hz|) * (hz / |hz|) * WHz;
eKz / cKz -> (kz / |kz|) * (kz / |kz|) * WKz;
eXz / cXz -> -((hz / |hz|) * (kz / |kz|) * WXz);
eAxx / cAxx -> -((ux / |ux|) * (ux / |ux|) * WAxx);
eAyy / cAyy -> -((uy / |uy|) * (uy / |uy|) * WAyy);
eAzz / cAzz -> -((uz / |uz|) * (uz / |uz|) * WAzz);
eAxy / cAxy -> -((ux / |ux|) * (uy / |uy|) * WAxy);
eAxz / cAxz -> -((ux / |ux|) * (uz / |uz|) * WAxz);
eAyz / cAyz -> -((uy / |uy|) * (uz / |uz|) * WAyz);

# From the fifteen pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P15) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

`sumexact.g`, whole, which the `e` state of 6 rests on:

```gappa
# A sum of two doubles that is below the normals is a double: both are
# multiples of 2^-1074, so their sum is too, and below 2^-1022 that takes at
# most 52 bits. So rnd(a + b) is a + b, and conesplit.py writes u01 unrounded
# where it is below the normals.
@rnd = float<ieee_64, ne>;
a = rnd(a_); b = rnd(b_);
s = rnd(a + b);
{
     |a| <= 1b1024 /\ |b| <= 1b1024
  /\ |a + b| <= 1b-1022
->
  s - (a + b) in [0, 0]
}
```

`conecases.py`, whole:

```python
"""Write the scripts of 464.16 from cone.g beside it: the controls
c_cone_half.g at B = u / 2, c_cone_nonormal.g without product_is_normal, and
c_cone_noeb.g without eb normal; and b_cone_<m>.g and b11_cone_<m>.g, cone.g at
B = 12u + m u^2 and 11u + m u^2, for the bisection. Each change is a text
replacement that must match exactly once. Run as: python3 conecases.py
[m ...]."""
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
CONE = open("cone.g").read()
B_CODE = "B = 12b-53 + 3b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
TITLE = "# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.\n"


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def at(text, b):
    return sub(text, B_CODE, b)


def without_normal(text):
    keep = []
    lemma_of = ("hx", "kx", "hy", "ky", "hz", "kz", "sx", "sy", "sz", "ux", "uy", "uz",
                "u01", "ss", "w", "along", "vx", "vy", "vz", "R2", "A")
    for line in text.split("\n"):
        if "1b-1022" in line and "|eb|" not in line:
            continue
        if line.startswith("  /\\ ") and line.split()[1] in lemma_of and LEMMA in line:
            continue
        keep.append(line)
    text = "\n".join(keep)
    text = sub(text, "  # The first sum of up is a normal number; where it is not, it is exact,\n"
               "  # and that is another case.\n", "")
    text = sub(text, "  # product_is_normal on the six halves (cross_product_halves_if_normal),\n"
               "  # and on the three components, the three up halves, the slope, up and the\n"
               "  # axial term (stays_accurate).\n", "")
    text = sub(text, "  # The permanent's three squares are normal too, since no component is\n"
               "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,\n"
               "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal. In the same way\n"
               "  # |up| <= R, so R2 >= w and A >= along, both normal.\n", "")
    return sub(text, TITLE, TITLE + "# A control: without product_is_normal and the lemma lines it gives.\n")


def without_eb(text):
    text = sub(text, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
    text = sub(text, f"  /\\ eb -/ B * P {LEMMA}\n", "")
    return sub(text, TITLE, TITLE + "# A control: without eb normal and the lemma line it gives.\n")


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


write("c_cone_half.g", at(CONE, B_HALF))
write("c_cone_nonormal.g", without_normal(CONE))
write("c_cone_noeb.g", without_eb(CONE))
for m in sys.argv[1:]:
    write(f"b_cone_{m}.g", at(CONE, f"B = 12b-53 + {m}b-106;"))
    write(f"b11_cone_{m}.g", at(CONE, f"B = 11b-53 + {m}b-106;"))
```

`conesplit.py`, whole, as the third lane ran it and `run4.sh` wrote it again:

```python
"""Write cone.g's case split, one script per case and a control at B = u / 2
beside each. A case is four letters. The first three are the components x, y
and z of the cross product, as cylsplit.py (464.15) has them:

n  it does not cancel: q = rnd(h - k) is not zero, so its square s is
   normal (stays_accurate), and so is its reach square v, which is never
   below s;
c  it cancels to zero, h == k, and its reach square v = rnd(4 h^2) is
   normal;
s  it cancels, and v is below the normals and does not round to zero;
u  it cancels, and v rounds to zero.

The fourth is the axial term, up = rnd(rnd(ux + uy) + uz) and the
permanent's A = rnd(ss * rnd(R * R)):

n  up is not zero, so w and along are normal (stays_accurate), and
   u01 = rnd(ux + uy) is normal; |up| <= R, so R2 >= w and A >= along are
   normal too;
e  up is not zero, and u01 is below the normals, where it is exact
   (sumexact.g), so it is written unrounded;
c  up cancels to zero, u01 + uz == 0, and R2 and A are normal;
a  up cancels, R2 is normal and A is below the normals or zero;
s  up cancels, R2 is below the normals and not zero, and A is normal;
t  up cancels, R2 is below the normals and not zero, and A is below the
   normals or zero.

The six cover every axial term whose halves are normal, which stays_accurate
checks, but one: up cancels and R2 rounds to zero, which is where 464.16's
probe found a wrong sign, so no script is written for it. nnnn is cone.g,
byte for byte. Where up cancels, the axial term's error is the whole
of the true one, sl^2 Dt^2, and it is averaged over XA = ss R2 rather than
ss G^2, since R2 then need not be near G^2. The names are cone_<case>.g and
c_cone_<case>_half.g. Where no component is n and up cancels, y is 0 and its
error is a u^2 of the permanent, which B = u / 2 covers, so those 108 cases
have a second control, c_cone_<case>_usq.g, at B = u^2.
Run as: python3 conesplit.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
B_CODE = "B = 12b-53 + 3b-101;"
B_HALF = "B = 1b-54;"
B_USQ = "B = 1b-106;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
STATE = {"n": "does not cancel", "c": "cancels, its reach square normal",
         "s": "cancels, its reach square below the normals and not zero",
         "u": "cancels, its reach square rounding to zero"}
AXIAL = {"n": "up does not cancel and its first sum is normal",
         "e": "up does not cancel and its first sum is below the normals",
         "c": "up cancels, R2 and A normal",
         "a": "up cancels, R2 normal and A below the normals or zero",
         "s": "up cancels, R2 below the normals and not zero and A normal",
         "t": "up cancels, R2 below the normals and not zero and A below the normals or zero"}
UPS = "ne"


def reach(a):
    return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"


def left_sum(terms, rounded):
    """Sum terms left to right as Rust adds them, each sum rounded or not."""
    acc = None
    for t in terms:
        if acc is None:
            acc = t
        else:
            acc = f"rnd({acc} + {t})" if rounded else f"({acc} + {t})"
    return acc


def make(case, b=B_CODE):
    st = dict(zip(AXES, case[:3]))
    ax = case[3]
    up = ax in UPS
    xr = ax in "st"
    xa_low = ax in "at"
    live = [a for a in AXES if st[a] == "n"]
    dead = [a for a in AXES if st[a] != "n"]
    small = [a for a in AXES if st[a] in "su"]
    shifted = bool(small) or xa_low
    live01 = [a for a in "xy" if a in live]
    s01_rounds = len(live01) == 2
    s012_rounds = bool(live01) and "z" in live
    src = [
        "# restates: crates/rcad-geom/src/cone.rs on_floats at 2538ee89",
        "# restates: crates/rcad-geom/src/cone.rs cross_product_halves_if_normal at 2538ee89",
        "# restates: crates/rcad-geom/src/cone.rs cross_product_halves at 2538ee89",
        "# restates: crates/rcad-geom/src/cone.rs SIDE_OF_CONE_ERROR_BOUND at 2538ee89",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 2538ee89",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 2538ee89",
        "# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.",
    ]
    if case != "nnnn":
        src.append(f"# The case {case}: " + "; ".join(f"{a} {STATE[st[a]]}" for a in AXES)
                   + f"; {AXIAL[ax]}.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the apex, the axis direction and the slope, each an f64.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        "ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);",
        "dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);",
        "sl = rnd(sl_);",
        "",
        "# What on_floats computes, line for line.",
        "Fx = px - ox; Fy = py - oy; Fz = pz - oz;",
        "fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);",
        "",
        "# cross_product_halves: h is each component's left half, k its right half.",
    ]
    if dead:
        src += ["# A component that cancels has the same double for both halves, so k",
                "# is written h, and its q and s are zero and drop out."]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"h{a} = rnd({left}); k{a} = " + (f"h{a};" if a in dead else f"rnd({right});"))
    if live:
        src.append(" ".join(f"Q{a} = h{a} - k{a};" for a in live))
        src.append(" ".join(f"q{a} = rnd(Q{a});" for a in live))
    src += [
        "",
        "# up_halves, and up, summed left to right as Rust adds them.",
        "ux = rnd(fx * dx); uy = rnd(fy * dy); uz = rnd(fz * dz);",
    ]
    if ax == "e":
        src += ["# Its first sum is below the normals, where it is exact (sumexact.g).",
                "u01 = ux + uy;"]
    else:
        src.append("U01 = ux + uy; u01 = rnd(U01);")
    if up:
        src.append("U = u01 + uz; up = rnd(U);")
    else:
        src += ["# up cancels: u01 + uz is zero, so up, w and along are zero, and y is",
                "# the squares alone."]
    src.append("")
    if dead:
        src += ["# A zero square added to a double leaves that double, which then",
                "# rounds to itself, so a sum is written without its zero squares."]
    if live:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in live))
    src.append("ss = rnd(sl * sl);")
    if up:
        src += ["w = rnd(up * up);", "along = rnd(ss * w);"]
    if s01_rounds:
        src.append("S01 = sx + sy; s01 = rnd(S01);")
    elif live01:
        src.append(f"s01 = s{live01[0]};")
    if s012_rounds:
        src.append("S012 = s01 + sz; s012 = rnd(S012);")
    elif live01:
        src.append("s012 = s01;")
    elif live:
        src.append("s012 = sz;")
    if up:
        src.append("y = s012 - along;" if live else "y = -along;")
    else:
        src.append("y = s012;" if live else "y = 0;")
    src += ["value = rnd(y);", "",
            "# The permanent: each half's reach squared, then the slope squared times the",
            "# reach of up squared."]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (dbl.g), so it is written unrounded."]
    src.append(" ".join(f"n{a} = rnd(|h{a}| + |k{a}|);" if a in live else f"n{a} = |h{a}| + |k{a}|;"
                        for a in AXES))
    if small:
        src.append(" ".join(f"Xv{a} = n{a} * n{a};" for a in small))
        src += ["# A reach square that rounds to zero is written as the 0 it rounds to",
                "# (464.14's tinysq.g), and the sums leave it out."]
    vs = []
    for a in AXES:
        if st[a] in "nc":
            vs.append(f"v{a} = rnd(n{a} * n{a});")
        elif st[a] == "s":
            vs.append(f"v{a} = rnd(Xv{a});")
    if vs:
        src.append(" ".join(vs))
    src.append("R01 = rnd(|ux| + |uy|); R = rnd(R01 + |uz|);")
    if xr:
        src.append("XR = R * R; R2 = rnd(XR);")
    else:
        src.append("R2 = rnd(R * R);")
    if up:
        src.append("A = rnd(ss * R2);")
    else:
        src.append("XA = ss * R2; A = rnd(XA);")
    kept = [f"v{a}" for a in AXES if st[a] != "u"]
    acc = left_sum(kept, True)
    src.append(f"P = rnd({acc} + A);" if acc else "P = A;")
    src += [
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;",
        "Dt = Fx * dx + Fy * dy + Fz * dz;",
        "Ex = Cx * Cx + Cy * Cy + Cz * Cz - sl * sl * (Dt * Dt);",
        "",
        "# The permanent with no rounding of its own.",
        "G = |ux| + |uy| + |uz|;",
    ]
    axial_ref = "ss * (G * G)" if up else "XA"
    if not up:
        src += ["# Where up cancels, its axial term is XA, which R2 need not keep near",
                "# ss G^2."]
    src += [
        "Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)",
        f"     + (|hz| + |kz|) * (|hz| + |kz|) + {axial_ref};",
    ]
    if shifted:
        terms = [f"v{a}" if st[a] == "s" else reach(a) for a in AXES if st[a] != "u"]
        acc = left_sum(terms, False)
        last = "A" if xa_low else axial_ref
        if small:
            src += ["# The same with v in place of each reach square below the normals that",
                    "# does not round to zero, and without each that does, "
                    + ("and A in place of XA,\n# " if xa_low else "")
                    + "added as P adds."]
        else:
            src.append("# The same with A in place of XA, added as P adds.")
        src.append(f"Phatx = {acc} + {last};" if acc else f"Phatx = {last};")
    src += [
        "",
        "# What each rounding multiplied its argument by.",
        "rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;",
    ]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"rh{a} = h{a} / ({left}); rk{a} = k{a} / ({right});")
    if live:
        src.append(" ".join(f"rq{a} = q{a} / Q{a};" for a in live))
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in live))
    rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rs012 = s012 / S012;"] if s012_rounds else [])
    if rs:
        src.append(" ".join(rs))
    src.append("rux = ux / (fx * dx); ruy = uy / (fy * dy); ruz = uz / (fz * dz);")
    if ax == "e":
        src.append("rU = up / U;")
    elif up:
        src.append("ru01 = u01 / U01; rU = up / U;")
    else:
        src.append("ru01 = u01 / U01;")
    src.append("rss = ss / (sl * sl); rw = w / (up * up); ral = along / (ss * w);" if up
               else "rss = ss / (sl * sl);")
    src += [
        "",
        "# Each component's square, as y carries it, is (h - k)^2 times K; the true",
        "# one is (h a - k b)^2, a and b undoing the roundings of h and k.",
    ]
    if dead:
        src.append("# One that cancels has no square in y, and its true one is (h a - h b)^2.")
    for a in live:
        carry = [f"rq{a}", f"rq{a}", f"rs{a}"]
        if a in "xy" and s01_rounds:
            carry.append("rs01")
        if s012_rounds:
            carry.append("rs012")
        src.append(f"K{a} = " + " * ".join(carry) + ";")
    for a in AXES:
        src.append(f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a}); b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});")
    src += [
        "",
        "# So each component's error is three terms, one for each piece of its",
        "# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its",
        "# piece.",
    ]
    if dead:
        src.append("# One that cancels is off by its whole true square, eC.")
    for a in live:
        src.append(f"WH{a} = K{a} - a{a} * a{a}; WK{a} = K{a} - b{a} * b{a}; WX{a} = K{a} - a{a} * b{a};")
    for a in AXES:
        src.append(f"cH{a} = |h{a}| * |h{a}|; cK{a} = |k{a}| * |k{a}|; cX{a} = 2 * |h{a}| * |k{a}|;")
    for a in AXES:
        if a in live:
            src.append(f"eH{a} = h{a} * h{a} * WH{a}; eK{a} = k{a} * k{a} * WK{a}; eX{a} = -(2 * h{a} * k{a} * WX{a});")
        else:
            src.append(f"eC{a} = -(C{a} * C{a});")
    src.append("")
    if up:
        src += [
            "# The axial term, as y carries it, is ss (gx ux + gy uy + gz uz)^2 times KA,",
            "# g undoing nothing and folding in the roundings of the two sums; the true",
            "# one is sl^2 (Dt)^2 = ss (ex ux + ey uy + ez uz)^2 / rss, e undoing the",
            "# roundings of f and of the product. So its error is six terms, one for each",
            "# piece of ss G^2: ss u^2 for each of the three, and 2 ss |u| |u'| for each",
            "# pair. W is each term's error over its piece.",
            "KA = rw * ral;",
            "gx = ru01 * rU; gy = ru01 * rU; gz = rU;" if ax == "n" else "gx = rU; gy = rU; gz = rU;",
            "ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);",
            "WAxx = KA * gx * gx - ex * ex / rss;",
            "WAyy = KA * gy * gy - ey * ey / rss;",
            "WAzz = KA * gz * gz - ez * ez / rss;",
            "WAxy = KA * gx * gy - ex * ey / rss;",
            "WAxz = KA * gx * gz - ex * ez / rss;",
            "WAyz = KA * gy * gz - ey * ez / rss;",
            "cAxx = ss * |ux| * |ux|; cAyy = ss * |uy| * |uy|; cAzz = ss * |uz| * |uz|;",
            "cAxy = 2 * ss * |ux| * |uy|; cAxz = 2 * ss * |ux| * |uz|; cAyz = 2 * ss * |uy| * |uz|;",
            "eAxx = -(ss * ux * ux * WAxx); eAyy = -(ss * uy * uy * WAyy); eAzz = -(ss * uz * uz * WAzz);",
            "eAxy = -(2 * ss * ux * uy * WAxy); eAxz = -(2 * ss * ux * uz * WAxz);",
            "eAyz = -(2 * ss * uy * uz * WAyz);",
            "",
            "# The same permanent, as four blocks of the fifteen pieces the error is",
            "# averaged over: each component's reach squared, and the axial term.",
            "P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;",
            "SA = cAxx + cAyy + cAzz + cAxy + cAxz + cAyz;",
            "P15 = P3x + P3y + P3z + SA;",
        ]
    else:
        src += [
            "# Where up cancels, y has no axial term, and the true one, sl^2 Dt^2, is",
            "# all error. Since u01 + uz is zero, Dt is what the roundings left: (e - g)",
            "# u for each half, e undoing the roundings of f and of the product and g",
            "# that of the first sum, so g is ru01, ru01 and 1. Dt is averaged over G,",
            "# and the error over XA.",
            "ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);",
            "Dx = ex - ru01; Dy = ey - ru01; Dz = ez - 1;",
            "D3 = Dx * ux + Dy * uy + Dz * uz;",
            "",
            "# The same permanent, as four blocks: each component's reach squared, and",
            "# the axial term.",
            "P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;",
            "P15 = P3x + P3y + P3z + XA;",
        ]
    src += ["", "# Each component's square as y carries it, and the axial term as Ex has it."]
    if live:
        src.append(" ".join(f"Y{a} = K{a} * Q{a} * Q{a};" for a in live))
    src.append("At = sl * sl * (Dt * Dt);")

    nonzero = "\n  /\\ ".join([
        "|fx| >= 1b-1074 /\\ |fy| >= 1b-1074 /\\ |fz| >= 1b-1074",
        "|dx| >= 1b-1074 /\\ |dy| >= 1b-1074 /\\ |dz| >= 1b-1074",
        "|sl| >= 1b-1074"] + ([" /\\ ".join(f"|q{a}| >= 1b-1074" for a in live)] if live else [])
        + (["|up| >= 1b-1074"] if up else []))
    if not dead and up:
        head = ["  # No factor is zero, no component cancels to zero and nor does up. For an",
                "  # f64 that is the same as being at least the smallest subnormal."]
    elif up:
        head = ["  # No factor is zero, no component but those the case names cancels to",
                "  # zero, and nor does up. For an f64 that is the same as being at least",
                "  # the smallest subnormal."]
    else:
        head = ["  # No factor is zero, and no component but those the case names cancels to",
                "  # zero. For an f64 that is the same as being at least the smallest",
                "  # subnormal."]
    src += [
        "",
        "{",
        "  # Every input is a finite f64.",
        "     |px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024",
        "  /\\ |ox| <= 1b1024 /\\ |oy| <= 1b1024 /\\ |oz| <= 1b1024",
        "  /\\ |dx| <= 1b1024 /\\ |dy| <= 1b1024 /\\ |dz| <= 1b1024",
        "  /\\ |sl| <= 1b1024",
    ] + head + ["  /\\ " + nonzero]
    if ax == "n":
        src += ["  # The first sum of up is a normal number; where it is not, it is exact,",
                "  # and that is another case.",
                "  /\\ |u01| >= 1b-1022"]
    elif ax == "e":
        src += ["  # The first sum of up is below the normals, where it is exact.",
                "  /\\ |u01| <= 1b-1022"]
    else:
        src += ["  # up cancels: u01 + uz is zero, so u01 is -uz, which is normal.",
                "  /\\ u01 + uz in [0, 0] /\\ |u01| >= 1b-1022"]
    src += [
        "  # product_is_normal on the six halves (cross_product_halves_if_normal),",
        "  # and on the three components, the three up halves, the slope, up and the",
        "  # axial term (stays_accurate).",
        "  /\\ |hx| >= 1b-1022 /\\ |kx| >= 1b-1022 /\\ |hy| >= 1b-1022",
        "  /\\ |ky| >= 1b-1022 /\\ |hz| >= 1b-1022 /\\ |kz| >= 1b-1022",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in live))
    src.append("  /\\ |ux| >= 1b-1022 /\\ |uy| >= 1b-1022 /\\ |uz| >= 1b-1022")
    src.append("  /\\ |ss| >= 1b-1022 /\\ |w| >= 1b-1022 /\\ |along| >= 1b-1022" if up
               else "  /\\ |ss| >= 1b-1022")
    if not dead:
        src += [
            "  # The permanent's three squares are normal too, since no component is",
            "  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,",
            "  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal."
            + (" In the same way" if up else ""),
        ]
        if up:
            src.append("  # |up| <= R, so R2 >= w and A >= along, both normal.")
        src.append("  /\\ |vx| >= 1b-1022 /\\ |vy| >= 1b-1022 /\\ |vz| >= 1b-1022")
    else:
        src += [
            "  # The reach square of a component that does not cancel is normal: |q| <= n,",
            "  # rounding being monotone and |h - k| <= |h| + |k|, so v = rnd(n * n) >=",
            "  # rnd(q * q) = s, which is normal. For one that cancels the case says",
            "  # which it is. One below the normals that does not round to zero is at",
            "  # least the smallest subnormal, and tinysq.g and tinyzero.g split them",
            "  # at 2^-1075: at or below, they round to zero.",
        ]
        rows = []
        for a in AXES:
            if st[a] in "nc":
                rows.append(f"|v{a}| >= 1b-1022")
            elif st[a] == "s":
                rows.append(f"Xv{a} <= 1b-1022 /\\ v{a} >= 1b-1074")
            else:
                rows.append(f"Xv{a} <= 1b-1075")
        src.append("  /\\ " + " /\\ ".join(rows))
        if up:
            src.append("  # |up| <= R, so R2 >= w and A >= along, both normal.")
    if up:
        src.append("  /\\ |R2| >= 1b-1022 /\\ |A| >= 1b-1022")
    else:
        src.append("  # Where up cancels, the case says what R2 and A are.")
        r2 = "XR <= 1b-1022 /\\ R2 >= 1b-1074" if xr else "|R2| >= 1b-1022"
        a_ ="XA <= 1b-1022" if xa_low else "|A| >= 1b-1022"
        src.append(f"  /\\ {r2} /\\ {a_}")
    src += [
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
        "  # What lemma.g proves each of those gives.",
    ]
    if dead:
        src.append("  # k is h in a component that cancels, so its line is h's against k's product.")
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for a in live:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for p in ("ux -/ fx * dx", "uy -/ fy * dy", "uz -/ fz * dz"):
        src.append(f"  /\\ {p} {LEMMA}")
    if ax != "e":
        src.append(f"  /\\ u01 -/ U01 {LEMMA}")
    src.append(f"  /\\ ss -/ sl * sl {LEMMA}")
    if up:
        src.append(f"  /\\ w -/ up * up {LEMMA}")
        src.append(f"  /\\ along -/ ss * w {LEMMA}")
    for a in AXES:
        if st[a] in "nc":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    if not xr:
        src.append(f"  /\\ R2 -/ R * R {LEMMA}")
    if not xa_low:
        src.append(f"  /\\ A -/ ss * R2 {LEMMA}" if up else f"  /\\ A -/ XA {LEMMA}")
    src += [
        f"  /\\ eb -/ B * P {LEMMA}",
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
    ]
    for a in AXES:
        src.append(f"  /\\ h{a} / |h{a}| in [-1, 1]" + ("" if a in dead else f" /\\ k{a} / |k{a}| in [-1, 1]"))
    src.append("  /\\ ux / |ux| in [-1, 1] /\\ uy / |uy| in [-1, 1] /\\ uz / |uz| in [-1, 1]")
    src.append("->")
    if shifted:
        src.append("  (y - Ex) / P15 in ? /\\ P -/ Phatx in ? /\\ P in ?")
    else:
        src.append("  (y - Ex) / P15 in ? /\\ P -/ Phat in ?")
    src += ["  /\\ (y - Ex) / eb in [-1, 1]", "}", "", "# How far each rounding is from the identity."]
    src += ["fx / Fx -> 1 + (fx - Fx) / Fx;", "fy / Fy -> 1 + (fy - Fy) / Fy;", "fz / Fz -> 1 + (fz - Fz) / Fz;"]
    for a in AXES:
        for h, prod in zip("hk", HALVES[a]):
            src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in live:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    for a in live:
        src.append(f"s{a} / (q{a} * q{a}) -> 1 + (s{a} - q{a} * q{a}) / (q{a} * q{a});")
    if s01_rounds:
        src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
    if s012_rounds:
        src.append("s012 / S012 -> 1 + (s012 - S012) / S012;")
    src += [
        "ux / (fx * dx) -> 1 + (ux - fx * dx) / (fx * dx);",
        "uy / (fy * dy) -> 1 + (uy - fy * dy) / (fy * dy);",
        "uz / (fz * dz) -> 1 + (uz - fz * dz) / (fz * dz);",
    ]
    if ax != "e":
        src.append("u01 / U01 -> 1 + (u01 - U01) / U01;")
    if up:
        src.append("up / U -> 1 + (up - U) / U;")
    src.append("ss / (sl * sl) -> 1 + (ss - sl * sl) / (sl * sl);")
    if up:
        src += ["w / (up * up) -> 1 + (w - up * up) / (up * up);",
                "along / (ss * w) -> 1 + (along - ss * w) / (ss * w);"]
    src += [
        "",
        "# The error is the sum of four blocks' errors, each the sum of its pieces'",
        "# errors, each a fraction of its own piece of the permanent; add_rr then",
    ]
    if up:
        src += ["# averages them. In blocks so that no hint's identity spans all fifteen",
                "# pieces, as cyl.g learned."]
    else:
        src += ["# averages them. In blocks so that no hint's identity spans every piece,",
                "# as cyl.g learned."]
    if dead:
        src += ["# A component that cancels is one block, off by its true square, which is",
                "# (a - b)^2 / 4 of its reach squared, 4 h^2."]
    src.append("(y - Ex) / P15 -> ((P15 + (y - Ex)) - P15) / P15;")
    blocks = [f"(P3{a} + (Y{a} - C{a} * C{a}))" if a in live else f"(P3{a} + eC{a})" for a in AXES]
    src.append("P15 + (y - Ex) -> " + " + ".join(blocks)
               + (" + (SA + (At - along));" if up else " + (XA + At);"))
    for a in live:
        src.append(f"P3{a} + (Y{a} - C{a} * C{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
    if up:
        src.append("SA + (At - along) -> (cAxx + eAxx) + (cAyy + eAyy) + (cAzz + eAzz) + (cAxy + eAxy) + (cAxz + eAxz) + (cAyz + eAyz);")
    for a in AXES:
        if a in live:
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        else:
            src.append(f"(P3{a} + eC{a}) - P3{a} -> eC{a};")
    if up:
        for p in ("xx", "yy", "zz", "xy", "xz", "yz"):
            src.append(f"(cA{p} + eA{p}) - cA{p} -> eA{p};")
    else:
        src.append("(XA + At) - XA -> At;")
    for a in AXES:
        if a in live:
            src += [f"eH{a} / cH{a} -> (h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a};",
                    f"eK{a} / cK{a} -> (k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a};",
                    f"eX{a} / cX{a} -> -((h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a});"]
        else:
            src.append(f"eC{a} / P3{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * (a{a} - b{a}) * (a{a} - b{a}) / 4);")
    if up:
        src += [
            "eAxx / cAxx -> -((ux / |ux|) * (ux / |ux|) * WAxx);",
            "eAyy / cAyy -> -((uy / |uy|) * (uy / |uy|) * WAyy);",
            "eAzz / cAzz -> -((uz / |uz|) * (uz / |uz|) * WAzz);",
            "eAxy / cAxy -> -((ux / |ux|) * (uy / |uy|) * WAxy);",
            "eAxz / cAxz -> -((ux / |ux|) * (uz / |uz|) * WAxz);",
            "eAyz / cAyz -> -((uy / |uy|) * (uz / |uz|) * WAyz);",
        ]
    else:
        src += [
            "# The axial error over XA: Dt over G, twice, G^2 over R2, and the slope's",
            "# rounding. Dt over G is the average of what each half's roundings left.",
            "At / XA -> (Dt / G) * (Dt / G) * ((G * G) / R2) / rss;",
            "Dt / G -> D3 / G + (u01 + uz) / G;",
            "D3 / G -> ((G + D3) - G) / G;",
            "G + D3 -> (|ux| + Dx * ux) + (|uy| + Dy * uy) + (|uz| + Dz * uz);",
            "(|ux| + Dx * ux) - |ux| -> Dx * ux;",
            "(|uy| + Dy * uy) - |uy| -> Dy * uy;",
            "(|uz| + Dz * uz) - |uz| -> Dz * uz;",
            "(Dx * ux) / |ux| -> Dx * (ux / |ux|);",
            "(Dy * uy) / |uy| -> Dy * (uy / |uy|);",
            "(Dz * uz) / |uz| -> Dz * (uz / |uz|);",
        ]
        if xr:
            src += [
                "# R2 below the normals is off from R^2 by an amount, at most half of",
                "# R2's own least value.",
                "(G * G) / R2 -> ((G * G) / XR) * (XR / R2);",
                "(G * G) / XR -> 1 / (1 + (XR - G * G) / (G * G));",
                "XR / R2 -> 1 + (XR - R2) / R2;",
            ]
        else:
            src.append("(G * G) / R2 -> 1 / (1 + (R2 - G * G) / (G * G));")
    src += ["", "# From the " + ("fifteen pieces" if up else "blocks")
            + " to the permanent on_floats rounds, and on to eb.",
            "(y - Ex) / eb -> ((y - Ex) / P15) * (Phat / P) * (P / eb);"]
    if shifted:
        shift = [f"(Xv{a} - v{a})" if st[a] == "s" else f"Xv{a}" for a in small]
        if xa_low:
            shift.append("(XA - A)")
        src += ["# What rounding a square below the normals cost is an amount, and",
                "# eb being normal keeps P large enough that it is small against eb.",
                f"Phat / P -> Phatx / P + ({' + '.join(shift)}) / P;",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
                "P -> eb / (B * (1 + (eb - B * P) / (B * P)));"]
    else:
        src += ["Phat / P -> 1 / (1 + (P - Phat) / Phat);",
                "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));"]
    return "\n".join(src) + "\n"


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


for cross in map("".join, itertools.product("ncsu", repeat=3)):
    for ax in "necast":
        case = cross + ax
        write(f"cone_{case}.g", make(case))
        if case != "nnnn":
            write(f"c_cone_{case}_half.g", make(case, B_HALF))
        if ax not in UPS and "n" not in cross:
            write(f"c_cone_{case}_usq.g", make(case, B_USQ))
```

`one1.sh`, whole:

```sh
#!/bin/bash
# one.sh with the timeout taken from T (default 600) and without coq, for
# trial runs.
cd "$(dirname "$0")"
g=/nix/store/0ihij0570a325g02l4wnf0idk8vqxr9b-gappa-1.8.0/bin/gappa
f=$1
n=${f%.g}
start=$(date +%s.%N)
timeout "${T:-600}" "$g" -Eprecision=200 "$f" > "$n.out" 2>&1
code=$?
end=$(date +%s.%N)
printf '%s exit %d %.2fs\n' "$n" "$code" "$(echo "$end - $start" | bc)"
grep -v 'assumed to be nonzero' "$n.out" | head -40 | sed "s/^/  $n: /"
```

`launch.sh`, whole:

```sh
#!/bin/bash
# Start the script named by the first argument detached, in its own process
# group, and print its pid.
setsid nohup "$1" > /dev/null 2>&1 &
echo $!
```

`run0.sh`, whole:

```sh
#!/bin/bash
# cone.g, the case where nothing cancels and nothing is zero, at the code's
# B, with gappa -Bcoq and coqc when it proves. Writes run0.txt.
cd "$(dirname "$0")"
{
  ./one.sh cone.g coq
} > run0.txt 2>&1
echo done >> run0.txt
```

`run1.sh`, whole:

```sh
#!/bin/bash
# cone.g's bisection, B = 12u + m u^2 for m of 0, 40, 60, 80, 100 and 120 and
# B = 11u + m u^2 for m of 100, 150, 200, 300 and 400, and its three controls,
# all at once. Writes run1.txt.
cd "$(dirname "$0")"
python3 conecases.py 0 40 60 80 100 120 150 200 300 400
{
  for m in 0 40 60 80 100 120; do
    ./one1.sh "b_cone_$m.g" &
  done
  for m in 100 150 200 300 400; do
    ./one1.sh "b11_cone_$m.g" &
  done
  for f in c_cone_half.g c_cone_nonormal.g c_cone_noeb.g; do
    ./one1.sh "$f" &
  done
  wait
} > run1.txt 2>&1
echo done >> run1.txt
```

`run2.sh`, whole:

```sh
#!/bin/bash
# cone.g's bisection, second round: B = 11u + m u^2 for m of 0, 50, 90, 96,
# 97, 98 and 99, all at once. Writes run2.txt.
cd "$(dirname "$0")"
python3 conecases.py 0 50 90 96 97 98 99
{
  for m in 0 50 90 96 97 98 99; do
    ./one1.sh "b11_cone_$m.g" &
  done
  wait
} > run2.txt 2>&1
echo done >> run2.txt
```

`run3a.sh`, whole:

```sh
#!/bin/bash
# One case of each axial state but n, the cross product's three components
# not cancelling, and the control beside each, all at once, before the whole
# split. Writes run3a.txt.
cd "$(dirname "$0")"
python3 conesplit.py
{
  for a in e c a s t; do
    ./one1.sh "cone_nnn$a.g" &
    ./one1.sh "c_cone_nnn${a}_half.g" &
  done
  wait
} > run3a.txt 2>&1
echo done >> run3a.txt
```

`run3.sh`, whole:

```sh
#!/bin/bash
# conesplit.py's 384 cases, each with gappa -Bcoq and coqc when it proves,
# and their 383 controls at u/2, 22 at a time: the cases first, since they
# take longest. Writes run3.txt.
cd "$(dirname "$0")"
python3 conesplit.py
{
  for f in cone_????.g; do
    echo "./one.sh $f coq"
  done
  for f in c_cone_????_half.g; do
    echo "./one1.sh $f"
  done
} > run3.jobs
{
  wc -l < run3.jobs
  xargs -P 22 -L 1 env < run3.jobs
} > run3.txt 2>&1
echo done >> run3.txt
```

`run4.sh`, whole:

```sh
#!/bin/bash
# The 108 second controls at B = u^2 that conesplit.py writes where y is 0,
# 22 at a time, after checking that writing the split again leaves every
# script run3.sh ran as it was. Writes run4.txt.
cd "$(dirname "$0")"
{
  md5sum cone_????.g c_cone_????_half.g > run4.before
  python3 conesplit.py
  md5sum cone_????.g c_cone_????_half.g > run4.after
  cmp run4.before run4.after && echo "run3.sh's scripts unchanged:" "$(wc -l < run4.after)"
  ls c_cone_????_usq.g | wc -l
  ls c_cone_????_usq.g | xargs -P 22 -L 1 ./one1.sh
} > run4.txt 2>&1
echo done >> run4.txt
```

`tally3.py`, whole:

```python
"""Tally run3.txt and the .out files of conesplit.py's 384 cases: for each case
its exit, the bound on (y - Ex) / P15 and on P against the permanent with no
rounding of its own that it prints, and its coqc line; for each control its
exit and best. Prints one line per case, then counts, grouped by how many
components of the cross product do not cancel and by the axial letter.
Run as: python3 tally3.py"""
import collections
import itertools
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))
run = open("run3.txt").read()
cases = ["".join(c) for c in itertools.product("ncsu", repeat=3)]
cases = [c + a for c in cases for a in "necast"]
RESULT = re.compile(r"^  (.*) in \[[^{]*\{([^,]*),[^}]*\}, [^{]*\{([^,]*),", re.M)


def bounds(out):
    """The y goal's and the P goal's printed ranges, as [lo, hi]."""
    y = p = "-"
    for expr, lo, hi in RESULT.findall(out):
        if expr.endswith("/ P15"):
            y = f"[{lo}, {hi}]"
        elif " -/ " in expr:
            p = f"[{lo}, {hi}]"
    return y, p


ybounds = collections.Counter()
pbounds = collections.Counter()
bests = collections.Counter()
ctl_exits = collections.Counter()
pointless = []
times = []
coq = []
for c in cases:
    out = open(f"cone_{c}.out").read()
    ex = re.search(rf"^cone_{c} exit (\d+) ([\d.]+)s$", run, re.M)
    cq = re.search(rf"^cone_{c} coq -Bcoq exit (\d+) coqc exit (\d+) ([\d.]+)s lines (\d+) hints (\d+)$", run, re.M)
    live = sum(1 for s in c[:3] if s == "n")
    y, p = bounds(out)
    if "pointless" in out:
        pointless.append(c)
        y = p = "pointless"
    ybounds[(live, c[3], y)] += 1
    pbounds[p] += 1
    times.append((float(ex.group(2)), c))
    if cq:
        coq.append((int(cq.group(4)), int(cq.group(5)), c, cq.group(1), cq.group(2), float(cq.group(3))))
    line = [c, "exit", ex.group(1), ex.group(2), "y", y, "P", p, "coq", cq.groups() if cq else "-"]
    if c != "nnnn":
        ctl = re.search(rf"^c_cone_{c}_half exit (\d+) ([\d.]+)s$", run, re.M)
        cout = open(f"c_cone_{c}_half.out").read()
        best = re.search(r"best: (\[[^\]]*\])", cout)
        bests[(ctl.group(1), best.group(1) if best else "pointless" if "pointless" in cout else "-")] += 1
        ctl_exits[(live, c[3], ctl.group(1))] += 1
        line += ["ctl", ctl.group(1), best.group(1) if best else "-"]
    print(*line)
print("cases exit 0:", len(re.findall(r"^cone_\w{4} exit 0 ", run, re.M)), "of", len(cases))
print("controls exit 1:", len(re.findall(r"^c_cone_\w{4}_half exit 1 ", run, re.M)), "of", len(cases) - 1)
print("pointless:", pointless)
print("y bounds by live components and axial letter:")
for k in sorted(ybounds):
    print("  ", k, ybounds[k])
print("P bounds:", dict(pbounds))
print("control exits by live components and axial letter:")
for k in sorted(ctl_exits):
    print("  ", k, ctl_exits[k])
print("control exits and bests:", dict(bests))
times.sort()
print("times", times[0], times[-1])
print("coq ok:", sum(1 for x in coq if x[3] == "0" and x[4] == "0"), "of", len(coq))
coq.sort()
print("coq smallest", coq[0][:3], "largest", coq[-1][:3], "hints", min(x[1] for x in coq), max(x[1] for x in coq),
      "times", min(x[5] for x in coq), max(x[5] for x in coq))


def other_lines(f):
    return [line for line in open(f).read().split("\n")
            if line and "assumed to be nonzero" not in line and not line.startswith(("Results:", "  "))]


warn = [f for f in os.listdir(".") if f.startswith("cone_") and f.endswith((".out", ".bcoq", ".coqc"))
        and other_lines(f)]
print("files with a line other than results and 'assumed to be nonzero':", sorted(warn))
```

After its line for each case, `python3 tally3.py` prints:

```text
cases exit 0: 384 of 384
controls exit 1: 275 of 383
pointless: ['sssa', 'ssst', 'ssua', 'ssut', 'susa', 'sust', 'suua', 'suut', 'ussa', 'usst', 'usua', 'usut', 'uusa', 'uust', 'uuua', 'uuut']
y bounds by live components and axial letter:
   (0, 'a', '[-4.93038e-32, 1.47793e-46]') 19
   (0, 'a', 'pointless') 8
   (0, 'c', '[-2.62743e-46, 1.10934e-31]') 3
   (0, 'c', '[-4.93038e-32, 1.10934e-31]') 19
   (0, 'c', '[-5.25487e-46, 1.10934e-31]') 3
   (0, 'c', '[-7.51646e-62, 1.10934e-31]') 1
   (0, 'c', '[-7.8823e-46, 1.10934e-31]') 1
   (0, 'e', '[-9.99201e-16, 9.99201e-16]') 27
   (0, 'n', '[-1.22125e-15, 1.22125e-15]') 27
   (0, 's', '[-2.62743e-46, 1.664e-31]') 3
   (0, 's', '[-4.93038e-32, 1.664e-31]') 19
   (0, 's', '[-5.25487e-46, 1.664e-31]') 3
   (0, 's', '[-7.51646e-62, 1.664e-31]') 1
   (0, 's', '[-7.8823e-46, 1.664e-31]') 1
   (0, 't', '[-4.93038e-32, 2.2169e-46]') 19
   (0, 't', 'pointless') 8
   (1, 'a', '[-7.77156e-16, 7.77156e-16]') 27
   (1, 'c', '[-7.77156e-16, 7.77156e-16]') 27
   (1, 'e', '[-9.99201e-16, 9.99201e-16]') 27
   (1, 'n', '[-1.22125e-15, 1.22125e-15]') 27
   (1, 's', '[-7.77156e-16, 7.77156e-16]') 27
   (1, 't', '[-7.77156e-16, 7.77156e-16]') 27
   (2, 'a', '[-8.88178e-16, 8.88178e-16]') 9
   (2, 'c', '[-8.88178e-16, 8.88178e-16]') 9
   (2, 'e', '[-9.99201e-16, 9.99201e-16]') 9
   (2, 'n', '[-1.22125e-15, 1.22125e-15]') 9
   (2, 's', '[-8.88178e-16, 8.88178e-16]') 9
   (2, 't', '[-8.88178e-16, 8.88178e-16]') 9
   (3, 'a', '[-9.99201e-16, 9.99201e-16]') 1
   (3, 'c', '[-9.99201e-16, 9.99201e-16]') 1
   (3, 'e', '[-9.99201e-16, 9.99201e-16]') 1
   (3, 'n', '[-1.22125e-15, 1.22125e-15]') 1
   (3, 's', '[-9.99201e-16, 9.99201e-16]') 1
   (3, 't', '[-9.99201e-16, 9.99201e-16]') 1
P bounds: {'[-7.77156e-16, 7.77156e-16]': 126, '[-6.66134e-16, 6.66134e-16]': 62, '[-5.55112e-16, 5.55112e-16]': 76, '[-4.44089e-16, 4.44089e-16]': 36, '[-3.33067e-16, 3.33067e-16]': 40, '[-2.22045e-16, 2.22045e-16]': 26, 'pointless': 16, '[-1.11022e-16, 1.11022e-16]': 2}
control exits by live components and axial letter:
   (0, 'a', '0') 27
   (0, 'c', '0') 27
   (0, 'e', '1') 27
   (0, 'n', '1') 27
   (0, 's', '0') 27
   (0, 't', '0') 27
   (1, 'a', '1') 27
   (1, 'c', '1') 27
   (1, 'e', '1') 27
   (1, 'n', '1') 27
   (1, 's', '1') 27
   (1, 't', '1') 27
   (2, 'a', '1') 9
   (2, 'c', '1') 9
   (2, 'e', '1') 9
   (2, 'n', '1') 9
   (2, 's', '1') 9
   (2, 't', '1') 9
   (3, 'a', '1') 1
   (3, 'c', '1') 1
   (3, 'e', '1') 1
   (3, 's', '1') 1
   (3, 't', '1') 1
control exits and bests: {('1', '[-18, 18]'): 68, ('1', '[-22, 22]'): 63, ('1', '[-16, 16]'): 36, ('1', '[-14, 14]'): 108, ('0', '-'): 92, ('0', 'pointless'): 16}
times (4.17, 'ccuc') (39.13, 'nnnt')
coq ok: 384 of 384
coq smallest (2325, 39, 'uuua') largest (17532, 64, 'nnnn') hints 37 64 times 6.68 51.46
files with a line other than results and 'assumed to be nonzero': []
```

`same.py`, whole:

```python
"""Check that each script 464.16 appends whole is byte for byte the file that
was run, in q2681/; that one.sh is 464.13's block; that tally3.py, run again
there, prints the counts 464.16 gives; and that conesplit.py and conecases.py,
run again in a fresh directory from the appended text with the arguments of
run1.sh and run2.sh, write what was run.
Run as: python3 same.py <doc> <q2681 dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, q2681 = sys.argv[1], sys.argv[2]
text = open(doc).read()
start = text.index("### 464.16")
sec = text[start:]
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)
blocks = FENCED.findall(sec)
print("code blocks in 464.16:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
marks = {
    "cone.g": "# restates: crates/rcad-geom/src/cone.rs on_floats at 2538ee89\n# restates: crates/rcad-geom/src/cone.rs cross_product_halves_if_normal",
    "sumexact.g": "# A sum of two doubles that is below the normals",
    "conecases.py": '"""Write the scripts of 464.16 from cone.g',
    "conesplit.py": '"""Write cone.g\'s case split',
    "one1.sh": "#!/bin/bash\n# one.sh with the timeout taken from T",
    "launch.sh": "#!/bin/bash\n# Start the script named by the first argument detached",
    "run0.sh": "#!/bin/bash\n# cone.g, the case where nothing cancels",
    "run1.sh": "#!/bin/bash\n# cone.g's bisection, B = 12u",
    "run2.sh": "#!/bin/bash\n# cone.g's bisection, second round",
    "run3a.sh": "#!/bin/bash\n# One case of each axial state",
    "run3.sh": "#!/bin/bash\n# conesplit.py's 384 cases",
    "run4.sh": "#!/bin/bash\n# The 108 second controls",
    "tally3.py": '"""Tally run3.txt and the .out files of conesplit',
    "truth.py": '"""The true sign of the cone probe',
    "same.py": '"""Check that each script 464.16 appends whole',
}
found = {}
for lang, body in blocks:
    for name, mark in marks.items():
        if not body.startswith(mark):
            continue
        assert name not in found, name
        found[name] = body
for name in marks:
    if name not in found:
        print(name, "not appended")
        continue
    if name == "same.py":
        path = os.path.abspath(__file__)
    else:
        path = os.path.join(q2681, name)
    if not os.path.exists(path):
        print(name, "appended, no file to compare")
        continue
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")

s13 = text[text.index("### 464.13"):text.index("### 464.14")]
one = [body for lang, body in FENCED.findall(s13)
       if body.startswith("#!/bin/bash\n# Run one Gappa script under 464.4's pin")]
assert len(one) == 1, len(one)
print("one.sh is 464.13's block:", open(os.path.join(q2681, "one.sh")).read() == one[0])

counts = [body for lang, body in blocks if lang == "text" and body.startswith("cases exit 0: ")]
assert len(counts) == 1, len(counts)
tally = subprocess.run(["python3", os.path.join(q2681, "tally3.py")], capture_output=True, text=True, check=True).stdout
print("tally3.py run again ends with 464.16's counts:", tally.endswith("\n" + counts[0]))


def compare(fresh, ran):
    written = sorted(f for f in os.listdir(fresh) if f.endswith(".g"))
    same = [f for f in written if os.path.exists(os.path.join(ran, f))
            and filecmp.cmp(os.path.join(fresh, f), os.path.join(ran, f), shallow=False)]
    differ = [f for f in written if os.path.exists(os.path.join(ran, f)) and f not in same]
    absent = [f for f in written if not os.path.exists(os.path.join(ran, f))]
    return written, same, differ, absent


fresh = tempfile.mkdtemp()
for name in ("cone.g", "conecases.py", "conesplit.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "conesplit.py"], cwd=fresh, check=True)
subprocess.run(["python3", "conecases.py", "0", "40", "60", "80", "100", "120", "150", "200", "300", "400"], cwd=fresh, check=True)
subprocess.run(["python3", "conecases.py", "0", "50", "90", "96", "97", "98", "99"], cwd=fresh, check=True)
written, same, differ, absent = compare(fresh, q2681)
print("written again:", len(written), "same as run:", len(same), "differ:", differ, "not in q2681:", absent)
print("cone_nnnn.g is cone.g:", filecmp.cmp(os.path.join(fresh, "cone_nnnn.g"), os.path.join(fresh, "cone.g"), shallow=False))
ran = sorted(f for f in os.listdir(q2681) if f.endswith(".g"))
print(".g files in q2681 not written again:", [f for f in ran if f not in written])
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.16: 21
cone.g same
sumexact.g same
conecases.py same
conesplit.py same
one1.sh same
launch.sh same
run0.sh same
run1.sh same
run2.sh same
run3a.sh same
run3.sh same
run4.sh same
tally3.py same
truth.py same
same.py same
one.sh is 464.13's block: True
tally3.py run again ends with 464.16's counts: True
written again: 911 same as run: 911 differ: [] not in q2681: []
cone_nnnn.g is cone.g: True
.g files in q2681 not written again: ['sumexact.g']
```

The 911 are `cone.g` itself, the 384 cases, their 383 controls at `u / 2`,
the 108 second controls at `u²`, the three controls of 7, and the 16 scripts
each of `b_cone_` and `b11_cone_`. Neither generator writes `sumexact.g`; it
is checked as appended above.

**10. What this leaves.** Four rows, filed from this write-up, each after
Q2681:

- Q2779, `wrong/D`, tier 5: the wrong sign of 6, on `cone.rs`, so that
  `on_floats` refuses, or bounds, an `R2` that rounds to zero. The `2⁷⁸` that
  puts it in D is hand arithmetic: no run with a slope of at most `2⁷⁷` where
  `R2` rounds to zero has been made, and one would demonstrate the D.
- Q2780, small: restate `cone.rs:643-663` by 5.
- Q2781: prove the bound where a difference or a direction component is zero,
  the axis-aligned cone among them, as Q2771 does for the cylinder. Not run,
  so no class.
- Q2782: run `conesplit.py` with the checked products below the normals, and
  restate `cone.rs:432-434` by what that shows, as Q2773 does for the
  cylinder. Not run, so no class.

And what is not a row:

- Q2700 already waits on Q2681, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- The probe of 6 is a predicate and an input where `product_is_normal`'s doc
  holds: a product below the normals, `R2`, fed into a relative bound makes
  level 1 certify a wrong sign. That is what Q2767 asks for, and Q2767 waits
  on Q2782 already.
- `field` on these scripts' hints (464.10).

### 464.17 The side of rcad-geom's torus

Q2682 took up `SIDE_OF_TORUS_ERROR_BOUND` in `crates/rcad-geom/src/torus.rs`,
in seven lanes on 2026-10-25. The first four were the first lane and its three
relays. The first wrote `tor.g`. The second ran it, its bisection of 3 and its
controls of 7. The third wrote `torsplit.py`. The fourth ran the split of 6
and its controls, and wrote and ran `shapes.py` and `same.py`. The fifth, the
second lane, merged the first lane's branch and ran the second `-Bcoq` of 8.
The sixth, its first relay, checked the comments and wrote 1 to 9. The
seventh, its second, ran `same.py` again and wrote 10. The bound is
proved on `y`, the sum before `value`'s last rounding, at the code's constant
wherever no factor is zero. That covers any of the three components of the
cross product cancelling, whatever its reach square rounds to; `R² − r²`
normal or below the normals; and the permanent's cross product term normal,
below the normals or zero. Coq checks each proof, modulo its hints, one of
them only at its second certificate (8). On `y` the bound needs about a `u`
less than the code's `18u` (3). The claim on `value` itself was not tried, and
`decide` does not need it (4). The branches where a difference or a direction
component is zero, which `product_is_normal` lets through, are not run, and a
point on the axis of an axis-aligned torus is one. And the comments claim more
than is proved. The doc comment on the constant writes `153ε` where it means
`153ε²`, miscounts its own terms, and reaches its seventeen roundings by two
slips that cancel. It also says `on_floats` enforces the bottom end product by
product, which is false: it checks none of the permanent's own products. Unlike
the cone's, no case the split runs gives a wrong sign there (5, 6). Restating
the comment and running what is not run is three rows (10).

Every run is Gappa 1.8.0 under 464.4's pin, at `-Eprecision=200`, against
`torus.rs` and `cascade.rs` at `4be293a5`. `git log --oneline
4be293a5e..cb850cc2b` on `torus.rs`, `cascade.rs`, `three_surfaces.rs`,
`rcad-topo`'s `validate.rs`, `rcad-step`'s `brep.rs` and `rcad-geom`'s
`lib.rs` prints nothing, so each is as it stands at `cb850cc2b`, `main` when
this write-up was finished. `u` is `2⁻⁵³`, and the code's `B = (18 + 220ε) ε`
(`torus.rs:508-511`) is, as a double, `18u + 224u²`, written `B = 18b-53 +
7b-101;`. Unlike the cone's `96ε`, `220ε` is `55 · 2⁻⁵¹`, not a multiple of
`2⁻⁴⁸`, the ulp of 18, so the Rust's sum rounds, up, to `18 + 7 · 2⁻⁴⁸`, and
the product by `ε` is exact. `python3 -c` on `(18.0 + 220.0 * 2.0**-53) *
2.0**-53` printed `0x1.2000000000007p-49 224`, the second figure `(B − 18u) /
u²` in exact rationals. Each script follows 464.8's pattern: one ratio hint
per rounding; the error split into one term per piece of the permanent, so
that Gappa averages them; 464.5's lemma for each product `on_floats` checks is
normal, and for each of the permanent's products that monotone rounding keeps
normal (2); and the step from `y` to the sign as 464.4 gives it.

**1. What the proof guards.** `Torus::side_of` (`torus.rs:309-323`), the
torus's `SideOfSurface`, runs the cascade on `HowFarFromTheRing`, once a guard
has refused a point or a torus that is not real numbers. `Surface::Torus`
reaches it from `three_surfaces.rs:3297`, from `rcad-topo`'s
`validate.rs:4500` and from `rcad-step`'s `brep.rs:3049`, and `lib.rs:126`
exports `Torus` and `SIDE_OF_TORUS`. A user makes a torus with `Torus::new`
(`torus.rs:101`), which takes an axis of finite numbers and two finite
positive radii, and refuses a tube as wide as the ring or wider. So, like
464.15's and 464.16's, this proof guards a predicate a user reaches.

**2. `on_floats`, line for line.** `HowFarFromTheRing::on_floats`
(`torus.rs:336-398`) returns `None` for a point that is not exact `f64`s
(`torus.rs:337`) and takes the three differences `fi` from the centre to the
point (`torus.rs:343`). It forms the six halves of their cross product with the
direction (`cross_product_halves`, `torus.rs:474-483`, called at 344) and the
three components `h − k`, `sideways` (`torus.rs:345`). It returns `None`
unless `product_is_normal` holds for each difference, each direction component
and each component with itself, for each half's two factors, and for each
radius with itself (`stays_accurate`, `torus.rs:350-359`). It forms
`out_and_back`, the sum of the three squares of `fi` (`torus.rs:364-365`),
`hollow = R² − r²` (`torus.rs:366`), `bulge = out_and_back + hollow`
(`torus.rs:367`), `axis_squared`, the direction's `w·w` (`torus.rs:368-369`),
`across`, the sum of the components' squares (`torus.rs:370-371`), and `4R²`
(`torus.rs:372`). It returns `None` unless `product_is_normal` holds for
`bulge` with itself, `w·w` with `bulge²`, and `4R²` with `across`
(`torus.rs:373-377`). Last it returns `value = w·w · bulge² − 4R² · across` and
`error_bound = SIDE_OF_TORUS_ERROR_BOUND * permanent` (`torus.rs:394-397`).
The permanent is `w·w · bulge_reach² + 4R² · spread` (`torus.rs:392`), with
`bulge_reach = out_and_back + R² + r²` (`torus.rs:391`) and `spread` the sum of
each half pair's reach squared, `(|h| + |k|)²` (`torus.rs:384-390`).

`tor.g` (below) writes that operation for operation. The cross product is
`cyl.g`'s (464.15's 2): `hx = rnd(fy · dz)`, `kx = rnd(fz · dy)`, `qx = rnd(hx
− kx)` and the like. Then `gi = rnd(fi · fi)`, `oab = rnd(rnd(gx + gy) + gz)`,
`RR = rnd(R · R)`, `rr = rnd(r · r)`, `hollow = rnd(RR − rr)`, `bulge = rnd(oab
+ hollow)`, `W = rnd(rnd(tx + ty) + tz)` with `ti = rnd(di · di)`, `across =
rnd(rnd(sx + sy) + sz)` with `si = rnd(qi · qi)`, `F4 = rnd(4 · RR)`, `bb =
rnd(bulge · bulge)`, `left = rnd(W · bb)`, `right = rnd(F4 · across)`, `y = left
− right` and `value = rnd(y)`. The permanent is `P = rnd(P1 + P2)`, with `P1 =
rnd(W · brr)`, `brr = rnd(br · br)`, `br = rnd(rnd(oab + RR) + rr)`, `P2 = rnd(F4
· spread)`, `spread = rnd(rnd(vx + vy) + vz)`, `vi = rnd(ni · ni)` and `ni =
rnd(|hi| + |ki|)`. `spread` is a `sum()` from zero, and adding the first square
to zero is exact.

On the longest chain, `fx`'s, that is seventeen roundings to `y` and eighteen
to `value`. `fx` and its square `gx` are three, `fx` counting twice; the two
sums of `out_and_back` make five; `bulge` six, since `RR` and `hollow` run
beside `out_and_back`; `bb` doubles that and adds its own, thirteen; `W`'s
three on `tx` make sixteen, and `left` seventeen. The `4R²` side is eleven:
`f`, the half and `q` are three, `q`'s square seven, the two sums of `across`
nine, and `RR` and `right`'s own rounding make eleven, since `4 · RR` is
exact.

Nothing checks the permanent's own products: the three reach squares `vi`,
`brr`, `P1` and `P2`. Where nothing cancels, each is at least a checked product
by monotone rounding: `vi ≥ si` as in 464.15's 2; `br ≥ bulge`, since `hollow ≤
RR`, so `brr ≥ bb` and `P1 ≥ left`; and `spread ≥ across`, so `P2 ≥ right`.
`brr` and `P1` stay normal in every case, since `bulge` is never zero (below),
so `bb` and `left` are checked on factors that are not zero. The others do not.
Where a component cancels to zero, its `si` is zero, its check passes on the
zero (`cascade.rs:67-69`), and its `vi` is unchecked, as the cylinder's. Where
all three cancel, `across` and `right` are zero too, and `P2` is unchecked: a
point on the axis, a shape a designer makes. Those are the split of 6.

`hollow` is never zero, on paper. `Torus::new` keeps `r < R`, and `on_floats`
checks that both squares are normal. Write `r = m · 2ᵉ`, `m` a whole number in
`[2⁵², 2⁵³)`. If `R` is in the same binade, `R ≥ (m + 1) 2ᵉ`, so `R² − r² ≥
(2m + 1) 2²ᵉ`. That is more than an ulp of the squares, `2²ᵉ⁺⁵²` while `m² <
2¹⁰⁵` and `2²ᵉ⁺⁵³` above it, so the two cannot round to one double. Where `R²`
reaches `2²ᵉ⁺¹⁰⁶`, or `R` is in a binade above, `RR ≥ 2²ᵉ⁺¹⁰⁶`, while `r² ≤
(2⁵³ − 1)² 2²ᵉ` is more than half an ulp below it and rounds below it. So `RR >
rr`, and a difference of two different doubles is never rounded to zero. So
`bulge = rnd(oab + hollow)` is positive too. `hollow` can be below
the normals, where it is exact, as 464.16's `sumexact.g` shows; `tor.g` takes it
normal, and the split takes the other. A zero difference or direction component
is in no script here (10).

**How the error splits.** Two blocks, one for each of the permanent's two
products. The `4R²` block is the cylinder's nine pieces (464.15's 2), each
times `F4`, with `right`'s rounding in `K` and `F4`'s and `RR`'s in `m`. The
`w·w · bulge²` block is six pieces, `W · a · b` for each pair of `oab`, `RR` and
`rr`, which sum to `W · (oab + RR + rr)²`. Each piece's error is its pair's
factor as `y` carries it less its true one, and `om` and `Z`, which are `w·w /
W` and `|x|² / oab`, are each averaged over three pieces of their own. So the
error is averaged over fifteen pieces, `Phat2`, in the two blocks. A first
form averaged `bulge / (oab + RR + rr)` over its three parts instead; it was
dropped before it ran, since `rr`'s part is off by about `−2`, and 464.15's 4
found Gappa does not average a sum whose relative error reaches `−1`.

`tor.g`, whole, as every run of 3, 7 and 8 ran it:

```gappa
# restates: crates/rcad-geom/src/torus.rs on_floats at 4be293a5
# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 4be293a5
# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 4be293a5
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 4be293a5
# restates: crates/rcad-predicates/src/cascade.rs decide at 4be293a5
# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.
@rnd = float<ieee_64, ne>;

# The point, the centre, the axis direction and the two radii, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
rg = rnd(rg_); tb = rnd(tb_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
hx = rnd(fy * dz); kx = rnd(fz * dy);
hy = rnd(fz * dx); ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = rnd(fy * dx);
Qx = hx - kx; Qy = hy - ky; Qz = hz - kz;
qx = rnd(Qx); qy = rnd(Qy); qz = rnd(Qz);

# squares and out_and_back, left to right as Rust adds them.
gx = rnd(fx * fx); gy = rnd(fy * fy); gz = rnd(fz * fz);
O01 = gx + gy; o01 = rnd(O01);
O = o01 + gz; oab = rnd(O);

RR = rnd(rg * rg); rr = rnd(tb * tb);
H = RR - rr; hollow = rnd(H);
Bu = oab + hollow; bulge = rnd(Bu);

# axis_squared.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; W = rnd(T);

# across.
sx = rnd(qx * qx); sy = rnd(qy * qy); sz = rnd(qz * qz);
S01 = sx + sy; s01 = rnd(S01);
S = s01 + sz; across = rnd(S);

F4 = rnd(4 * RR);
bb = rnd(bulge * bulge);
left = rnd(W * bb);
right = rnd(F4 * across);
y = left - right;
value = rnd(y);

# The permanent. spread is a sum() from zero, and adding the first square to
# zero is exact.
nx = rnd(|hx| + |kx|); ny = rnd(|hy| + |ky|); nz = rnd(|hz| + |kz|);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
spread = rnd(rnd(vx + vy) + vz);
br = rnd(rnd(oab + RR) + rr);
brr = rnd(br * br);
P1 = rnd(W * brr);
P2 = rnd(F4 * spread);
P = rnd(P1 + P2);

B = 18b-53 + 7b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Wt = dx * dx + dy * dy + dz * dz;
G2 = Fx * Fx + Fy * Fy + Fz * Fz;
Bt = G2 + rg * rg - tb * tb;
Lt = Wt * (Bt * Bt);
Rt = 4 * (rg * rg) * (Cx * Cx + Cy * Cy + Cz * Cz);
Ex = Lt - Rt;

# The permanent with no rounding of its own.
Rc = oab + RR + rr;
Phat = W * (Rc * Rc)
     + F4 * ((|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
             + (|hz| + |kz|) * (|hz| + |kz|));

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rqx = qx / Qx; rqy = qy / Qy; rqz = qz / Qz;
rgx = gx / (fx * fx); rgy = gy / (fy * fy); rgz = gz / (fz * fz);
ro01 = o01 / O01; ro = oab / O;
rRR = RR / (rg * rg); rrr = rr / (tb * tb);
rH = hollow / H; rBu = bulge / Bu;
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rW = W / T;
rsx = sx / (qx * qx); rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs01 = s01 / S01; rS = across / S;
rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);
rL = left / (W * bb); rR = right / (F4 * across);

# The w.w bulge^2 block. W and oab as their three pieces, which sum to them.
Wx = rW * rt01 * tx; Wy = rW * rt01 * ty; Wz = rW * tz;
W3 = Wx + Wy + Wz;
jx = 1 / (rW * rt01 * rtx) - 1; jy = 1 / (rW * rt01 * rty) - 1; jz = 1 / (rW * rtz) - 1;
om = Wt / W;
Ox = ro * ro01 * gx; Oy = ro * ro01 * gy; Oz = ro * gz;
O3 = Ox + Oy + Oz;
zx = 1 / (ro * ro01 * rgx * rfx * rfx) - 1;
zy = 1 / (ro * ro01 * rgy * rfy * rfy) - 1;
zz = 1 / (ro * rgz * rfz * rfz) - 1;
Z = G2 / oab;

# bulge as y carries it is rBu (oab + rH (RR - rr)); the true one is
# Z oab + RR / rRR - rr / rrr. Squared and times W, that is six pieces, one
# for each pair of oab, RR and rr; W is each piece's error over the piece.
KL = rL * rbb * rBu * rBu;
WOO = KL - om * Z * Z;
WRR = KL * rH * rH - om / (rRR * rRR);
Wrr = KL * rH * rH - om / (rrr * rrr);
WOR = KL * rH - om * Z / rRR;
WOr = KL * rH - om * Z / rrr;
WRr = KL * rH * rH - om / (rRR * rrr);
cOO = W * oab * oab; cRR = W * RR * RR; crr = W * rr * rr;
cOR = 2 * W * oab * RR; cOr = 2 * W * oab * rr; cRr = 2 * W * RR * rr;
eOO = cOO * WOO; eRR = cRR * WRR; err = crr * Wrr;
eOR = cOR * WOR; eOr = -(cOr * WOr); eRr = -(cRr * WRr);
A6 = cOO + cRR + crr + cOR + cOr + cRr;

# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's
# rounding, m undoes F4's and RR's.
Kx = rR * rS * rs01 * rsx * rqx * rqx;
Ky = rR * rS * rs01 * rsy * rqy * rqy;
Kz = rR * rS * rsz * rqz * rqz;
m = 1 / (rF4 * rRR);
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);
WHx = Kx - m * ax * ax; WKx = Kx - m * bx * bx; WXx = Kx - m * ax * bx;
WHy = Ky - m * ay * ay; WKy = Ky - m * by * by; WXy = Ky - m * ay * by;
WHz = Kz - m * az * az; WKz = Kz - m * bz * bz; WXz = Kz - m * az * bz;
cHx = F4 * |hx| * |hx|; cKx = F4 * |kx| * |kx|; cXx = 2 * F4 * |hx| * |kx|;
cHy = F4 * |hy| * |hy|; cKy = F4 * |ky| * |ky|; cXy = 2 * F4 * |hy| * |ky|;
cHz = F4 * |hz| * |hz|; cKz = F4 * |kz| * |kz|; cXz = 2 * F4 * |hz| * |kz|;
eHx = -(F4 * hx * hx * WHx); eKx = -(F4 * kx * kx * WKx); eXx = 2 * F4 * hx * kx * WXx;
eHy = -(F4 * hy * hy * WHy); eKy = -(F4 * ky * ky * WKy); eXy = 2 * F4 * hy * ky * WXy;
eHz = -(F4 * hz * hz * WHz); eKz = -(F4 * kz * kz * WKz); eXz = 2 * F4 * hz * kz * WXz;
C3x = cHx + cKx + cXx; C3y = cHy + cKy + cXy; C3z = cHz + cKz + cXz;
C9 = C3x + C3y + C3z;
Yx = F4 * Kx * Qx * Qx; Yy = F4 * Ky * Qy * Qy; Yz = F4 * Kz * Qz * Qz;
Zx = F4 * m * (Cx * Cx); Zy = F4 * m * (Cy * Cy); Zz = F4 * m * (Cz * Cz);

# The same permanent as the two blocks.
Phat2 = A6 + C9;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |rg| <= 1b1024 /\ |tb| <= 1b1024
  # No factor is zero, and no component cancels to zero. For an f64 that is
  # the same as being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |rg| >= 1b-1074 /\ |tb| >= 1b-1074
  /\ |qx| >= 1b-1074 /\ |qy| >= 1b-1074 /\ |qz| >= 1b-1074
  # hollow is a normal number; below the normals it is exact, and that is
  # another case.
  /\ |hollow| >= 1b-1022
  # product_is_normal on the three squares of from_centre, of direction and
  # of sideways, the six halves, R^2, r^2, bulge^2, axis_squared bulge^2 and
  # 4 R^2 across (stays_accurate and the check after it).
  /\ |gx| >= 1b-1022 /\ |gy| >= 1b-1022 /\ |gz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |sx| >= 1b-1022 /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |RR| >= 1b-1022 /\ |rr| >= 1b-1022
  /\ |bb| >= 1b-1022 /\ |left| >= 1b-1022 /\ |right| >= 1b-1022
  # The permanent's products are normal too, since nothing cancels here, by
  # monotone rounding: v >= s, as no component is zero; br >= bulge, as
  # hollow <= RR, so brr >= bb and P1 >= left; spread >= across, so
  # P2 >= right.
  /\ |vx| >= 1b-1022 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  /\ |brr| >= 1b-1022 /\ |P1| >= 1b-1022 /\ |P2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ hollow -/ H in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gx -/ fx * fx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gy -/ fy * fy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ gz -/ fz * fz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ RR -/ rg * rg in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ tb * tb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ bb -/ bulge * bulge in [-4503599627370497b-105, 4503599627370497b-105]
  /\ left -/ W * bb in [-4503599627370497b-105, 4503599627370497b-105]
  /\ right -/ F4 * across in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ brr -/ br * br in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P1 -/ W * brr in [-4503599627370497b-105, 4503599627370497b-105]
  /\ P2 -/ F4 * spread in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  /\ hy / |hy| in [-1, 1] /\ ky / |ky| in [-1, 1]
  /\ hz / |hz| in [-1, 1] /\ kz / |kz| in [-1, 1]
->
  (y - Ex) / Phat2 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
qx / Qx -> 1 + (qx - Qx) / Qx;
qy / Qy -> 1 + (qy - Qy) / Qy;
qz / Qz -> 1 + (qz - Qz) / Qz;
gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);
gy / (fy * fy) -> 1 + (gy - fy * fy) / (fy * fy);
gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);
o01 / O01 -> 1 + (o01 - O01) / O01;
oab / O -> 1 + (oab - O) / O;
RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);
rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);
hollow / H -> 1 + (hollow - H) / H;
bulge / Bu -> 1 + (bulge - Bu) / Bu;
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
W / T -> 1 + (W - T) / T;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s01 / S01 -> 1 + (s01 - S01) / S01;
across / S -> 1 + (across - S) / S;
F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);
bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);
left / (W * bb) -> 1 + (left - W * bb) / (W * bb);
right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);

# w.w over W, and |x|^2 over oab, each averaged over three pieces.
Wt / W -> 1 + ((W3 + (Wt - W)) - W3) / W3;
W3 + (Wt - W) -> (Wx + Wx * jx) + (Wy + Wy * jy) + (Wz + Wz * jz);
(Wx + Wx * jx) - Wx -> Wx * jx;
(Wy + Wy * jy) - Wy -> Wy * jy;
(Wz + Wz * jz) - Wz -> Wz * jz;
G2 / oab -> 1 + ((O3 + (G2 - oab)) - O3) / O3;
O3 + (G2 - oab) -> (Ox + Ox * zx) + (Oy + Oy * zy) + (Oz + Oz * zz);
(Ox + Ox * zx) - Ox -> Ox * zx;
(Oy + Oy * zy) - Oy -> Oy * zy;
(Oz + Oz * zz) - Oz -> Oz * zz;

# The error is the sum of the two blocks' errors, each the sum of its
# pieces' errors, each a fraction of its own piece of the permanent; add_rr
# then averages them.
(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;
Phat2 + (y - Ex) -> (A6 + (left - Lt)) + (C9 + (Rt - right));
A6 + (left - Lt) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + eOR) + (cOr + eOr) + (cRr + eRr);
(cOO + eOO) - cOO -> eOO;
(cRR + eRR) - cRR -> eRR;
(crr + err) - crr -> err;
(cOR + eOR) - cOR -> eOR;
(cOr + eOr) - cOr -> eOr;
(cRr + eRr) - cRr -> eRr;
eOr / cOr -> -WOr;
eRr / cRr -> -WRr;
C9 + (Rt - right) -> (C3x + (Zx - Yx)) + (C3y + (Zy - Yy)) + (C3z + (Zz - Yz));
C3x + (Zx - Yx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
C3y + (Zy - Yy) -> (cHy + eHy) + (cKy + eKy) + (cXy + eXy);
C3z + (Zz - Yz) -> (cHz + eHz) + (cKz + eKz) + (cXz + eXz);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
(cHy + eHy) - cHy -> eHy;
(cKy + eKy) - cKy -> eKy;
(cXy + eXy) - cXy -> eXy;
(cHz + eHz) - cHz -> eHz;
(cKz + eKz) - cKz -> eKz;
(cXz + eXz) - cXz -> eXz;
eHx / cHx -> -((hx / |hx|) * (hx / |hx|) * WHx);
eKx / cKx -> -((kx / |kx|) * (kx / |kx|) * WKx);
eXx / cXx -> (hx / |hx|) * (kx / |kx|) * WXx;
eHy / cHy -> -((hy / |hy|) * (hy / |hy|) * WHy);
eKy / cKy -> -((ky / |ky|) * (ky / |ky|) * WKy);
eXy / cXy -> (hy / |hy|) * (ky / |ky|) * WXy;
eHz / cHz -> -((hz / |hz|) * (hz / |hz|) * WHz);
eKz / cKz -> -((kz / |kz|) * (kz / |kz|) * WKz);
eXz / cXz -> (hz / |hz|) * (kz / |kz|) * WXz;

# From the two blocks to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

**3. The bound on `y`.** `tor.g` proves `|y − Ex| ≤ eb` at the code's `B`. In
`run0.sh` (9) it exits 0 in `20.02s`, printing `(y − Ex) / Phat2` within
`1.88738e-15` and `P −/ Phat` within `7.77156e-16`. `ku.py` (9) writes those
exact bounds as `17u + 95u²` and `7u + 22u²`. `Phat` is the permanent with no
rounding of its own, and `Phat2` the same sum as the two blocks of 2. Its
`.out` and `.bcoq` have no line but their results and 492 "assumed to be
nonzero" warnings each. The bisection: `run1.sh` (9) ran the `b_tor_<m>.g` and
`b17_tor_<m>.g` that `torcases.py` (9) writes, `tor.g` at `B = 18u + m u²` and
at `17u + m u²`. At `18u`, every `m` run proves: 0, 100, 200, 228, 230, 231,
232, 233, 236 and 300. At `17u`, 232, 233, 236 and 300 prove, and 231, 230,
228, 200, 100 and 0 fail, best `[-1, 1]`. By hand, `(17u + 95u²)(1 + 7u +
22u²)(1 + u)` (`y`'s bound taken through the permanent's `7u + 22u²` and
through `eb`'s own rounding) is `17u + 231u²` and a `u³`, so the line falls
where that count puts it. So on `y`, where nothing cancels, the bound needs
about a `u` less than the code's `18u`, as the count of 2 says.

A first form of `tor.g`, run through `one1.sh`, exited 0 in `15.34s`, but Gappa
called ten of its hints unneeded ("no need for the rewriting ... if ... `Wx !=
0`"): the three last steps of each of the two averages over three pieces, and
the ratio hints of four of the six pair pieces, `OO`, `RR`, `rr` and `OR`. Its
two block goals, `(left − Lt) / A6` and `(Rt − right) / C9`, printed bounds of
about `1e2773`, which say nothing. All twelve were dropped, and `tor.g` is the
script after that.

**4. The claim on `value`.** Not tried. `decide` does not need it: the doc on
`Filtered::error_bound` (`cascade.rs:25-35`) asks only for the bound on the
sum before `value`'s last rounding, which 3 proves, as in 464.14's 4, 464.15's
4 and 464.16's 4.

**5. What the comments claim.** The doc comment on the constant,
`torus.rs:485-507`, says:

> The chain, counted step by step. `point − centre` is one rounding; squaring
> a component is a second and a third; the two sums that fold the three
> squares are a fourth and fifth; `R²`, `r²` and their difference are a sixth,
> and adding that on is a seventh. Squaring the total doubles what it carries,
> so it sits at `(1 + epsilon)^15`; `w·w` is three roundings of its own and
> multiplying it in is a sixteenth and seventeenth. The other side is shorter
> — a cross-product component is three roundings, its square a seventh, the
> two sums a ninth, and `4R²` times it an eleventh — so the final subtraction,
> the eighteenth, is what the whole thing is charged at.
>
> `(1 + epsilon)^18 − 1 ≤ 18·epsilon + 153·epsilon² + …` times the permanent.
> The `220·epsilon²` covers the higher-order terms and the rounding of the
> permanent itself; what it has to beat is about `153·epsilon`.
>
> Every step of that argument is a *relative* one, so as with every bound of
> this shape it holds only while nothing overflows or falls into the
> subnormals. [`HowFarFromTheRing::on_floats`] enforces the bottom end product
> by product through [`product_is_normal`], and `decide` the top end by
> refusing a bound that is not itself a normal number.

The comment above the check, at `torus.rs:347-349`, says:

> The bound below is a fraction of the permanent, which only bounds anything
> while every product keeps its relative accuracy. One of them in the
> subnormals and this level certifies a wrong sign.

Against what is proved:

- Eighteen roundings to `value` and seventeen to `y` agree with `tor.g`'s `17u`
  on `y` (3), but the chain reaches them by two slips that cancel. It puts
  `bulge` at seven roundings, counting `R²`, `r²` and their difference after
  `out_and_back`'s five, where they run beside it and `bulge` carries six (2).
  So its square sits at fifteen where it is thirteen. Then it counts `w·w`'s
  three roundings and the product's own as two, the sixteenth and seventeenth,
  where they are four. Its `4R²` side at eleven is right (2).
- "About `153·epsilon`" is the `ε²` term of `(1 + ε)¹⁸ − 1`, so it gives the
  wrong power, as 464.15's 5 found the cylinder's `45` does and 464.16's 5 the
  cone's `66`. It also leaves out what the same sentence says the `220ε²`
  covers. The permanent's rounding, `7u` (3), taken against `18ε`, costs
  `126ε²`. The rounding of `B · P` costs `ε` of `eb`, which is `18ε²`. So by
  the comment's own terms the count is `153 + 126 + 18 = 297`, above its `220`
  and above the `224` the double holds. On `y` that costs nothing, since `y`
  proves at `17u + 232u²` and at `18u + 0u²` (3).
- "Enforces the bottom end product by product" is false. `on_floats` checks
  none of the permanent's own products, the reach squares `vi`, `brr`, `P1` and
  `P2`, and `product_is_normal` lets a zero through (`cascade.rs:67-69`). `brr`
  and `P1` stay normal (2). But where a component cancels, its check passes on
  the zero and its `vi` can fall below the normals or round to zero, and where
  all three cancel, `P2` can too. Unlike the cone's, the bound holds in each of
  those (6). The branches where a difference or a direction component is zero
  are not run (10).
- "One of them in the subnormals and this level certifies a wrong sign" is
  false of each reach square `vi` and of `P2`, below the normals or zero: by 6
  each of those proves. `brr` and `P1` never fall there (2). For the products
  `on_floats` checks it was not run, as 464.14's 6 ran the sphere's squares
  (10).

So the comments claim more than is proved, which is what the row asks. Unlike
the cone's, the claim that the bottom end is enforced gives no wrong sign in
any case the split runs. Restating them is a row (10).

**6. The components that cancel, `hollow` and `P2`.** `torsplit.py` (9) splits
`tor.g` as 464.15's `cylsplit.py` splits `cyl.g`, with two more letters. Each
component of the cross product is `n`, `c`, `s` or `u`, as in 464.15's 6: it
does not cancel; it cancels and its reach square is normal; it cancels and its
reach square is below the normals and not zero; or it cancels and its reach
square rounds to zero. The fourth letter is `hollow`'s: `n`, normal, or `e`,
below the normals, where it is exact (464.16's `sumexact.g`) and is written
unrounded. The fifth is `P2`'s:

- `n`: `P2` is normal. Where a component does not cancel, `spread ≥ across`,
  so `P2 ≥ right`, which `on_floats` checks, and `n` is the only state.
- `b`: all three components cancel, so `right` is zero, and `P2` is below the
  normals or zero. Where all three reach squares round to zero, `uuu`, `spread`
  and `P2` are zero and `P` is `P1`, and `b` is the only state.

That makes `37 · 2 + 26 · 2 · 2 + 2 = 180` cases: the 37 with a component `n`
take either `hollow`, the 26 others but `uuu` take either `hollow` and either
`P2`, and `uuu` takes either `hollow`. They are written `tor_<case>.g`, and
`tor_nnnnn.g` is `tor.g` byte for byte, which `same.py` (9) checks. A
component that cancels has no square in `y`, so its error is the whole of its
true term, `4R² C²`. That error is averaged over the piece `2W · oab · RR` of
the `w·w · bulge²` block rather than over its own reach square, so that what
`vi` or `P2` rounds to does not matter to it.

What they print. `trial.sh` (9) first ran eight cases, `cnnnn`, `cccnn`,
`nnnen`, `uuueb`, `cccnb`, `scnnn`, `ssuen` and `uuunb`, and their `u / 2`
controls, all at once through `one1.sh`. Each case exited 0, in 18.22 to 29.59
s, and each control exited 1, best `[-34, 34]`, so no hint of `torsplit.py`
needed changing. `run3.sh` (9) then ran all 180 with `coq`, and the 179
controls, 20 at a time on a load of about 40 over 24 cores, and `tally3.py`
(9) counts them. After its line for each case it prints:

```text
cases exit 0: 180 of 180
controls exit 1: 179 of 179
pointless: []
y bounds by live components and hollow and P2 letters:
   (0, 'eb', '[-1.88738e-15, 1.88738e-15]') 27
   (0, 'en', '[-1.88738e-15, 1.88738e-15]') 26
   (0, 'nb', '[-1.88738e-15, 1.88738e-15]') 27
   (0, 'nn', '[-1.88738e-15, 1.88738e-15]') 26
   (1, 'en', '[-1.88738e-15, 1.88738e-15]') 27
   (1, 'nn', '[-1.88738e-15, 1.88738e-15]') 27
   (2, 'en', '[-1.88738e-15, 1.88738e-15]') 9
   (2, 'nn', '[-1.88738e-15, 1.88738e-15]') 9
   (3, 'en', '[-1.88738e-15, 1.88738e-15]') 1
   (3, 'nn', '[-1.88738e-15, 1.88738e-15]') 1
P bounds: {'[-7.77156e-16, 7.77156e-16]': 178, '[-6.66134e-16, 6.66134e-16]': 2}
control exits by live components and hollow and P2 letters:
   (0, 'eb', '1') 27
   (0, 'en', '1') 26
   (0, 'nb', '1') 27
   (0, 'nn', '1') 26
   (1, 'en', '1') 27
   (1, 'nn', '1') 27
   (2, 'en', '1') 9
   (2, 'nn', '1') 9
   (3, 'en', '1') 1
control exits and bests: {('1', '[-34, 34]'): 179}
times (15.5, 'cucnn') (61.65, 'nnsen')
coq ok: 179 of 180
coq smallest (14324, 67, 'uuunb') largest (22872, 81, 'nnnnn') hints 66 81 times 24.62 99.14
files with a line other than results and 'assumed to be nonzero': ['tor_cusen.coqc']
```

- Each of the 180 exits 0 at the code's `B`, in 15.5 s (`tor_cucnn.g`) to
  61.65 s (`tor_nnsen.g`), and Gappa calls none of them pointless.
- `(y − Ex) / Phat2` is within `1.88738e-15` in all 180, whichever components
  cancel and whatever `hollow` and `P2` are, and `ku.py` writes each of the
  180 exact bounds as `17u + 95u²`. A component that cancels costs `y`
  nothing, since its longest chain is on `left` (2).
- `P` against the permanent with no rounding of its own, `Phat`, or `Phatx` as
  a case that splits writes it, is within `7u` (`7.77156e-16`) in 178 cases:
  `7u + 22u²` in 126 and `7u + 21u²` in 52, by `ku.py`. It is within `6u +
  15u²` (`6.66134e-16`) in the two `uuu` cases, where `P` is `P1`.
- 90 of the 180, `27 + 26 + 27 + 9 + 1`, have `hollow` below the normals.
- No `.out`, `.bcoq` or `.coqc` of the 180 has a line other than its results
  and "assumed to be nonzero" warnings, but `tor_cusen.coqc`, whose certificate
  `coqc` refused (8).

So on `y` the bound holds at the code's constant wherever no factor is zero.

The shapes. `shapes.py` (9) works three shapes in Python's doubles, which
round as Rust's do. A point on the axis of a torus whose axis is `(1, 1, 1)` is
`cccnn`, and one off it at `(1, 1, 2)` is `nncnn`: shapes a designer makes, and
both are proved. A point on the axis of the axis-aligned torus whose axis is
`(1, 0, 0)` has four zero factors, which no case takes, and passes every check
`on_floats` makes (10). It prints:

```text
on the axis (1, 0, 0): direction [1.0, 0.0, 0.0], from_centre [2.0, 0.0, 0.0], halves [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)]
  zero factors ['fy', 'fz', 'dy', 'dz']; case uuunb; on_floats' checks pass: True
on the axis (1, 1, 1): direction [0.5773502691896257, 0.5773502691896257, 0.5773502691896257], from_centre [1.0, 1.0, 1.0], halves [(0.5773502691896257, 0.5773502691896257), (0.5773502691896257, 0.5773502691896257), (0.5773502691896257, 0.5773502691896257)]
  zero factors none; case cccnn; on_floats' checks pass: True
off the axis (1, 1, 1), one component cancelling: direction [0.5773502691896257, 0.5773502691896257, 0.5773502691896257], from_centre [1.0, 1.0, 2.0], halves [(0.5773502691896257, 1.1547005383792515), (1.1547005383792515, 0.5773502691896257), (0.5773502691896257, 0.5773502691896257)]
  zero factors none; case nncnn; on_floats' checks pass: True
```

The letters `uuunb` of the first line are what the classifier reads off its
zeros; `torsplit.py`'s `uuunb` assumes no factor is zero, so it is not that
shape.

**7. The controls.** At `B = 1b-54;`, `u / 2`, each must fail:

- `c_tor_half.g` exits 1, best `[-34, 34]`, `tor.g`'s `17u` over `u / 2`.
- Each of the 179 `c_tor_<case>_half.g` exits 1, best `[-34, 34]`, so none
  needs a `u²` control, as 108 of the cone's did.

Together with the failures at `17u + 231u²` (3), that says none of the proofs
came from a contradiction in its hypotheses. Two more controls take a
hypothesis away:

- `c_tor_nonormal.g` is `tor.g` without every `1b-1022` hypothesis but
  `|eb|`'s, and without the lemma line each of those gives. It exits 1, best
  `[-7.69054e+2159, 7.69054e+2159]`, and fails `REL(P, Phat)` and `BND((y -
  Ex) / Phat2)` too.
- `c_tor_noeb.g`, `tor.g` without `eb` normal, exits 1 in `82.44s`, best
  `[-2.78435e+1080, 3.94239e+1124]`.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0 from 464.5's nix block, accept
each script that proves and was run through them:

- `tor.g`: 23062 lines with 81 hints, in `35.83s` (`run0.sh`).
- The 180 cases of 6: 14324 (`tor_uuunb`) to 22872 lines (`tor_nnnnn`), with
  66 to 81 hints, in 24.62 to 99.14 s each (`run3.sh`). `coqc` accepted 179
  of them at their first certificate, and printed nothing.
- `tor_cusen.g` at its second, below.

`tor_cusen.g`'s first certificate, 16904 lines, `coqc` refused. `tor_cusen.coqc`
ends `The term "eq_refl" has type "true = true" while it is expected to have
type "intersect_helper (upper i96) (lower i96) i9 = true".`, at `tor_cusen.v`
line 6766, `apply intersect with (1 := h1) (2 := h2)`, where `h1` and `h2` are
the same lemma, `l337 h0`. Gappa's certificate there is wrong, not its bound:
the same script's Gappa run exited 0. Q2682's second lane then ran `run4.sh`
(9) through 464.16's `launch.sh`. It runs `one.sh` with `coq` on
`tor_cusen2.g`, a copy of `tor_cusen.g`, so that `tor_cusen.coqc`, which
`tally3.py` reads, stays as `run3.sh` left it. It printed:

```text
tor_cusen2.g is tor_cusen.g
tor_cusen2 exit 0 20.99s
tor_cusen2 coq -Bcoq exit 0 coqc exit 0 29.78s lines 16899 hints 68
coqc printed:
done
```

So `coqc` accepts the second certificate, 16899 lines with 68 hints from the
same bytes, and printed nothing, and it accepts a certificate for each of the
180. `ku.py` writes the two bounds of `tor_cusen2.out` as `17u + 95u²` and
`7u + 22u²`, the same as `tor_cusen.out`'s.

Gappa does not write the same proof each run (464.4). That is why
`tor_nnnnn.g`, the same bytes as `tor.g`, gave 22872 lines against 23062, and
why `tor_cusen.v` and `tor_cusen2.v` first differ at line 976. Each hint enters
the Coq file as a hypothesis with no condition, as in 464.4. The controls and
the bisection scripts were not run through `coqc`. `field` has not been run on
these hints (464.10).

**9. The scripts.** `torcases.py` writes the controls of 7 and the bisection
scripts of 3 from `tor.g`, and `torsplit.py` writes the 180 cases of 6 and
their controls. To run them again, save the nix block of 464.5 as
`gappalib18.nix` and `tor.g` and the two generators beside it, and then:

```sh
python3 torcases.py 0 100 200 228 230 231 232 233 236 300; python3 torsplit.py
nx="nix --extra-experimental-features nix-command --extra-experimental-features flakes"
pin=github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe
$nx shell $pin#gappa --command gappa -Eprecision=200 tor.g
$nx shell $pin#gappa --command gappa -Eprecision=200 -Bcoq tor.g > tor.v
gl=$($nx build --impure --no-link --print-out-paths -f gappalib18.nix)
fl=$($nx build --no-link --print-out-paths $pin#coqPackages_8_20.flocq)
COQPATH="$gl/lib/coq/8.20/user-contrib:$fl/lib/coq/8.20/user-contrib" \
  $nx shell $pin#coqPackages_8_20.coq --command coqc tor.v
```

and the same for every other file they write. A script that proves exits 0; a
control must exit 1. What the lanes ran is `run0.sh`, `run1.sh`, `trial.sh`,
`run3.sh` and `run4.sh` (below), in a scratch directory, each through 464.13's
`one.sh`, the same bytes, or through 464.16's `one1.sh`, which is `one.sh`
with the timeout taken from `T` and no `coq`. `ku.py` writes Gappa's bounds as
`k u + r u²`, `tally3.py` counts what `run3.sh` printed, and `shapes.py` reads
the shapes of 6. `same.py` (below) checks every script appended here byte for
byte against the file that was run; that `one.sh` is 464.13's block and
`one1.sh` and `launch.sh` 464.16's; that `tally3.py` and `shapes.py`, run
again, print what 6 gives; and that the two generators, run again from the
text here, write what was run. Its print and what its 383 files are end this
part.

`run0.sh`, whole:

```sh
#!/bin/bash
# tor.g, the case where nothing cancels, through one.sh with coq, and then
# every line of its output, its -Bcoq print and its coqc print that is not an
# "assumed to be nonzero" warning. Prints to run0.txt.
cd "$(dirname "$0")"
{
  ./one.sh tor.g coq
  for x in tor.out tor.bcoq tor.coqc; do
    echo "== $x: $(grep -c 'assumed to be nonzero' "$x") nonzero warnings"
    grep -v 'assumed to be nonzero' "$x"
  done
  echo done
} > run0.txt 2>&1
```

`torcases.py`, whole:

```python
"""Write the scripts of 464.17 from tor.g beside it: the controls c_tor_half.g
at B = u / 2, c_tor_nonormal.g without product_is_normal, and c_tor_noeb.g
without eb normal; and b_tor_<m>.g and b17_tor_<m>.g, tor.g at B = 18u + m u^2
and 17u + m u^2, for the bisection. Each change is a text replacement that
must match exactly once. Run as: python3 torcases.py [m ...]."""
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
TOR = open("tor.g").read()
B_CODE = "B = 18b-53 + 7b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
TITLE = "# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.\n"


def sub(text, old, new):
    assert text.count(old) == 1, old
    return text.replace(old, new)


def at(text, b):
    return sub(text, B_CODE, b)


def without_normal(text):
    keep = []
    for line in text.split("\n"):
        if "1b-1022" in line and "|eb|" not in line:
            continue
        if line.startswith("  /\\ ") and LEMMA in line and not line.startswith("  /\\ eb -/"):
            continue
        keep.append(line)
    text = "\n".join(keep)
    text = sub(text, "  # hollow is a normal number; below the normals it is exact, and that is\n"
               "  # another case.\n", "")
    text = sub(text, "  # product_is_normal on the three squares of from_centre, of direction and\n"
               "  # of sideways, the six halves, R^2, r^2, bulge^2, axis_squared bulge^2 and\n"
               "  # 4 R^2 across (stays_accurate and the check after it).\n", "")
    text = sub(text, "  # The permanent's products are normal too, since nothing cancels here, by\n"
               "  # monotone rounding: v >= s, as no component is zero; br >= bulge, as\n"
               "  # hollow <= RR, so brr >= bb and P1 >= left; spread >= across, so\n"
               "  # P2 >= right.\n", "")
    return sub(text, TITLE, TITLE + "# A control: without product_is_normal and the lemma lines it gives.\n")


def without_eb(text):
    text = sub(text, "  # The bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
    text = sub(text, f"  /\\ eb -/ B * P {LEMMA}\n", "")
    return sub(text, TITLE, TITLE + "# A control: without eb normal and the lemma line it gives.\n")


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


write("c_tor_half.g", at(TOR, B_HALF))
write("c_tor_nonormal.g", without_normal(TOR))
write("c_tor_noeb.g", without_eb(TOR))
for m in sys.argv[1:]:
    write(f"b_tor_{m}.g", at(TOR, f"B = 18b-53 + {m}b-106;"))
    write(f"b17_tor_{m}.g", at(TOR, f"B = 17b-53 + {m}b-106;"))
```

`run1.sh`, whole:

```sh
#!/bin/bash
# tor.g's controls and its bisection, B = 18u + m u^2 and 17u + m u^2, all
# at once through one1.sh with a 600 s timeout. Prints to run1.txt.
cd "$(dirname "$0")"
{
  python3 torcases.py 0 100 200 228 230 231 232 233 236 300
  for f in c_tor_half.g c_tor_nonormal.g c_tor_noeb.g b_tor_*.g b17_tor_*.g; do
    T=600 ./one1.sh "$f" &
  done
  wait
  echo done
} > run1.txt 2>&1
```

`ku.py`, whole:

```python
"""Print each result of the named .out files as k u + r u^2, u = 2^-53, with
k the whole number of u at or below the bound's larger end. Run as:
python3 ku.py <file.out> ..."""
import re
import sys
from fractions import Fraction

U = Fraction(1, 2**53)


def num(text):
    m = re.fullmatch(r"(-?\d+)(?:b(-?\d+))?", text)
    e = int(m.group(2) or 0)
    return Fraction(int(m.group(1))) * (Fraction(2) ** e)


for path in sys.argv[1:]:
    for line in open(path):
        m = re.match(r"\s+(.*) in \[(\S+) \{[^}]*\}, (\S+) \{[^}]*\}\]", line)
        if not m:
            continue
        lo, hi = num(m.group(2)), num(m.group(3))
        top = max(abs(lo), abs(hi))
        k = top // U
        r = (top - k * U) / (U * U)
        print(f"{path}: {m.group(1)}: {k}u + {float(r):.4g}u^2")
```

`torsplit.py`, whole, as `trial.sh` and `run3.sh` ran it:

```python
"""Write tor.g's case split, one script per case and a control at B = u / 2
beside each. A case is five letters. The first three are the components x, y
and z of the cross product, as cylsplit.py (464.15) has them:

n  it does not cancel: q = rnd(h - k) is not zero, so its square s is
   normal (stays_accurate), and so is its reach square v, which is never
   below s;
c  it cancels to zero, h == k, and its reach square v = rnd(4 h^2) is
   normal;
s  it cancels, and v is below the normals and does not round to zero;
u  it cancels, and v rounds to zero.

The fourth is hollow = rnd(RR - rr):

n  it is normal;
e  it is below the normals, where it is exact (464.16's sumexact.g), so it
   is written unrounded.

The fifth is the permanent's P2 = rnd(F4 * spread):

n  it is normal. Where a component does not cancel, spread >= across, so
   P2 >= right, which is normal (stays_accurate), and n is the only state;
b  all three components cancel, so right is zero, and P2 is below the
   normals or zero. Where all three reach squares round to zero, uuu, spread
   and P2 are zero and P is P1, and b is the only state.

That is 37 * 2 + 26 * 2 * 2 + 2 = 180 cases, and tor_nnnnn.g is tor.g, byte
for byte. A component that cancels has no square in y, so its error is the
whole of its true term, eC = 4 R^2 C^2. It is averaged over the piece
cOR = 2 W oab RR of the w.w bulge^2 block rather than over its own reach
square, so that what v or P2 rounds to does not matter to it. The names are
tor_<case>.g and c_tor_<case>_half.g.
Run as: python3 torsplit.py
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
AXES = "xyz"
B_CODE = "B = 18b-53 + 7b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HALVES = {"x": ("fy * dz", "fz * dy"), "y": ("fz * dx", "fx * dz"), "z": ("fx * dy", "fy * dx")}
STATE = {"n": "does not cancel", "c": "cancels, its reach square normal",
         "s": "cancels, its reach square below the normals and not zero",
         "u": "cancels, its reach square rounding to zero"}
HOLLOW = {"n": "hollow normal", "e": "hollow below the normals"}
SPREAD = {"n": "P2 normal", "b": "P2 below the normals or zero"}


def reach(a):
    return f"(|h{a}| + |k{a}|) * (|h{a}| + |k{a}|)"


def left_sum(terms):
    """Sum terms left to right as Rust adds them, each sum rounded."""
    acc = None
    for t in terms:
        acc = t if acc is None else f"rnd({acc} + {t})"
    return acc


def over_sum(v, a, r1, r2, total):
    """v_a / total as 1 over total / v_a, total being r2 (r1 (vx + vy) + vz)."""
    xy = " + ".join("1" if b == a else f"{v}{b} / {v}{a}" for b in "xy")
    z = "1" if a == "z" else f"{v}z / {v}{a}"
    return f"{v}{a} / {total} -> 1 / ({r2} * ({r1} * ({xy}) + {z}));"


def make(case, b=B_CODE):
    st = dict(zip(AXES, case[:3]))
    hol = case[3] == "n"
    p2 = case[4]
    live = [a for a in AXES if st[a] == "n"]
    dead = [a for a in AXES if st[a] != "n"]
    small = [a for a in AXES if st[a] in "su"]
    kept = [a for a in AXES if st[a] != "u"]
    vdead = [a for a in dead if st[a] in "cs"]
    live01 = [a for a in "xy" if a in live]
    s01_rounds = len(live01) == 2
    s_rounds = bool(live01) and "z" in live
    split = case != "nnnnn"
    src = [
        "# restates: crates/rcad-geom/src/torus.rs on_floats at 4be293a5",
        "# restates: crates/rcad-geom/src/torus.rs cross_product_halves at 4be293a5",
        "# restates: crates/rcad-geom/src/torus.rs SIDE_OF_TORUS_ERROR_BOUND at 4be293a5",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 4be293a5",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 4be293a5",
        "# HowFarFromTheRing::on_floats and SIDE_OF_TORUS_ERROR_BOUND, rcad-geom's torus.",
    ]
    if split:
        src.append(f"# The case {case}: " + "; ".join(f"{a} {STATE[st[a]]}" for a in AXES)
                   + f"; {HOLLOW[case[3]]}; {SPREAD[p2]}.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The point, the centre, the axis direction and the two radii, each an f64.",
        "px = rnd(px_); py = rnd(py_); pz = rnd(pz_);",
        "ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);",
        "dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);",
        "rg = rnd(rg_); tb = rnd(tb_);",
        "",
        "# What on_floats computes, line for line.",
        "Fx = px - ox; Fy = py - oy; Fz = pz - oz;",
        "fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);",
        "",
        "# cross_product_halves: h is each component's left half, k its right half.",
    ]
    if dead:
        src += ["# A component that cancels has the same double for both halves, so k",
                "# is written h, and its q and s are zero and drop out."]
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"h{a} = rnd({left}); k{a} = " + (f"h{a};" if a in dead else f"rnd({right});"))
    if live:
        src.append(" ".join(f"Q{a} = h{a} - k{a};" for a in live))
        src.append(" ".join(f"q{a} = rnd(Q{a});" for a in live))
    src += [
        "",
        "# squares and out_and_back, left to right as Rust adds them.",
        "gx = rnd(fx * fx); gy = rnd(fy * fy); gz = rnd(fz * fz);",
        "O01 = gx + gy; o01 = rnd(O01);",
        "O = o01 + gz; oab = rnd(O);",
        "",
        "RR = rnd(rg * rg); rr = rnd(tb * tb);",
    ]
    if hol:
        src.append("H = RR - rr; hollow = rnd(H);")
    else:
        src += ["# hollow is below the normals, where it is exact (464.16's sumexact.g).",
                "H = RR - rr; hollow = H;"]
    src += [
        "Bu = oab + hollow; bulge = rnd(Bu);",
        "",
        "# axis_squared.",
        "tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);",
        "T01 = tx + ty; t01 = rnd(T01);",
        "T = t01 + tz; W = rnd(T);",
        "",
        "# across.",
    ]
    if dead:
        src += ["# A zero square added to a double leaves that double, which then",
                "# rounds to itself, so a sum is written without its zero squares."]
    if live:
        src.append(" ".join(f"s{a} = rnd(q{a} * q{a});" for a in live))
    if s01_rounds:
        src.append("S01 = sx + sy; s01 = rnd(S01);")
    elif live01:
        src.append(f"s01 = s{live01[0]};")
    if s_rounds:
        src.append("S = s01 + sz; across = rnd(S);")
    elif live01:
        src.append("across = s01;")
    elif live:
        src.append("across = sz;")
    else:
        src.append("# All three cancel, so across and right are zero, and y is left.")
    src += [
        "",
        "F4 = rnd(4 * RR);",
        "bb = rnd(bulge * bulge);",
        "left = rnd(W * bb);",
    ]
    if live:
        src += ["right = rnd(F4 * across);", "y = left - right;"]
    else:
        src.append("y = left;")
    src += [
        "value = rnd(y);",
        "",
        "# The permanent. spread is a sum() from zero, and adding the first square to",
        "# zero is exact.",
    ]
    if dead:
        src += ["# A reach of two equal halves is twice one of them, which a double",
                "# holds (464.15's dbl.g), so it is written unrounded."]
    src.append(" ".join(f"n{a} = rnd(|h{a}| + |k{a}|);" if a in live else f"n{a} = |h{a}| + |k{a}|;"
                        for a in AXES))
    if small:
        src.append(" ".join(f"Xv{a} = n{a} * n{a};" for a in small))
    if "u" in case[:3]:
        src += ["# A reach square that rounds to zero is written as the 0 it rounds to",
                "# (464.14's tinysq.g), and the sums leave it out."]
    vs = []
    for a in AXES:
        if st[a] in "nc":
            vs.append(f"v{a} = rnd(n{a} * n{a});")
        elif st[a] == "s":
            vs.append(f"v{a} = rnd(Xv{a});")
    if vs:
        src.append(" ".join(vs))
    if kept:
        src.append(f"spread = {left_sum([f'v{a}' for a in kept])};")
    src += [
        "br = rnd(rnd(oab + RR) + rr);",
        "brr = rnd(br * br);",
        "P1 = rnd(W * brr);",
    ]
    if not kept:
        src.append("# spread and P2 are zero, and adding zero to P1 leaves it.")
        src.append("P = P1;")
    else:
        src.append("P2 = rnd(F4 * spread);" if p2 == "n" else "XP2 = F4 * spread; P2 = rnd(XP2);")
        src.append("P = rnd(P1 + P2);")
    src += [
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true value of the same inputs.",
        "Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;",
        "Wt = dx * dx + dy * dy + dz * dz;",
        "G2 = Fx * Fx + Fy * Fy + Fz * Fz;",
        "Bt = G2 + rg * rg - tb * tb;",
        "Lt = Wt * (Bt * Bt);",
        "Rt = 4 * (rg * rg) * (Cx * Cx + Cy * Cy + Cz * Cz);",
        "Ex = Lt - Rt;",
        "",
        "# The permanent with no rounding of its own.",
        "Rc = oab + RR + rr;",
        "Phat = W * (Rc * Rc)",
        "     + F4 * ((|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)",
        "             + (|hz| + |kz|) * (|hz| + |kz|));",
    ]
    dx_terms = None
    if split:
        if p2 == "b" and kept:
            src += ["# The same with P2 in place of the 4 R^2 product, as P adds them; Dx is",
                    "# what that adds to the two blocks below."]
            src.append("Phatx = W * (Rc * Rc) + P2;")
            dx_terms = "P2"
        elif p2 == "b":
            src.append("# The same with no 4 R^2 product, as P is P1.")
            src.append("Phatx = W * (Rc * Rc);")
        else:
            src += ["# The same with v in place of the reach square of each component that",
                    "# cancels, and without each that rounds to zero; Dx is what that adds",
                    "# to the two blocks below."]
            terms = [reach(a) if a in live else f"v{a}" for a in AXES if st[a] != "u"]
            src.append(f"Phatx = W * (Rc * Rc) + F4 * ({' + '.join(terms)});")
            if vdead:
                dx_terms = f"F4 * ({' + '.join(f'v{a}' for a in vdead)})"
        if dx_terms:
            src.append(f"Dx = {dx_terms};")
    src += [
        "",
        "# What each rounding multiplied its argument by.",
        "rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;",
        "rhx = hx / (fy * dz); rkx = kx / (fz * dy);",
        "rhy = hy / (fz * dx); rky = ky / (fx * dz);",
        "rhz = hz / (fx * dy); rkz = kz / (fy * dx);",
    ]
    if live:
        src.append(" ".join(f"rq{a} = q{a} / Q{a};" for a in live))
    src += [
        "rgx = gx / (fx * fx); rgy = gy / (fy * fy); rgz = gz / (fz * fz);",
        "ro01 = o01 / O01; ro = oab / O;",
        "rRR = RR / (rg * rg); rrr = rr / (tb * tb);",
        "rH = hollow / H; rBu = bulge / Bu;" if hol else "rBu = bulge / Bu;",
        "rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);",
        "rt01 = t01 / T01; rW = W / T;",
    ]
    if live:
        src.append(" ".join(f"rs{a} = s{a} / (q{a} * q{a});" for a in live))
    rs = (["rs01 = s01 / S01;"] if s01_rounds else []) + (["rS = across / S;"] if s_rounds else [])
    if rs:
        src.append(" ".join(rs))
    src.append("rF4 = F4 / (4 * RR); rbb = bb / (bulge * bulge);")
    src.append("rL = left / (W * bb); rR = right / (F4 * across);" if live else "rL = left / (W * bb);")
    src += [
        "",
        "# The w.w bulge^2 block. W and oab as their three pieces, which sum to them.",
        "Wx = rW * rt01 * tx; Wy = rW * rt01 * ty; Wz = rW * tz;",
        "W3 = Wx + Wy + Wz;",
        "jx = 1 / (rW * rt01 * rtx) - 1; jy = 1 / (rW * rt01 * rty) - 1; jz = 1 / (rW * rtz) - 1;",
        "om = Wt / W;",
        "Ox = ro * ro01 * gx; Oy = ro * ro01 * gy; Oz = ro * gz;",
        "O3 = Ox + Oy + Oz;",
        "zx = 1 / (ro * ro01 * rgx * rfx * rfx) - 1;",
        "zy = 1 / (ro * ro01 * rgy * rfy * rfy) - 1;",
        "zz = 1 / (ro * rgz * rfz * rfz) - 1;",
        "Z = G2 / oab;",
        "",
        "# bulge as y carries it is rBu (oab + rH (RR - rr)); the true one is",
        "# Z oab + RR / rRR - rr / rrr. Squared and times W, that is six pieces, one",
        "# for each pair of oab, RR and rr; W is each piece's error over the piece.",
    ]
    if not hol:
        src.append("# hollow is exact here, so rH is 1 and is left out.")
    six = [
        "KL = rL * rbb * rBu * rBu;",
        "WOO = KL - om * Z * Z;",
        "WRR = KL * rH * rH - om / (rRR * rRR);",
        "Wrr = KL * rH * rH - om / (rrr * rrr);",
        "WOR = KL * rH - om * Z / rRR;",
        "WOr = KL * rH - om * Z / rrr;",
        "WRr = KL * rH * rH - om / (rRR * rrr);",
    ]
    if not hol:
        six = [line.replace(" * rH * rH", "").replace(" * rH", "") for line in six]
    src += six
    src += [
        "cOO = W * oab * oab; cRR = W * RR * RR; crr = W * rr * rr;",
        "cOR = 2 * W * oab * RR; cOr = 2 * W * oab * rr; cRr = 2 * W * RR * rr;",
        "eOO = cOO * WOO; eRR = cRR * WRR; err = crr * Wrr;",
        "eOR = cOR * WOR; eOr = -(cOr * WOr); eRr = -(cRr * WRr);",
        "A6 = cOO + cRR + crr + cOR + cOr + cRr;",
        "",
        "# The 4 R^2 block, as 464.15's cylinder, times F4. K folds in right's",
        "# rounding, m undoes F4's and RR's.",
    ]
    if dead:
        src.append("# A component that cancels has no square in y, and is not in it.")
    for a in live:
        carry = ["rR"] + (["rS"] if s_rounds else []) + (["rs01"] if a in "xy" and s01_rounds else [])
        src.append(f"K{a} = " + " * ".join(carry + [f"rs{a}", f"rq{a}", f"rq{a}"]) + ";")
    src.append("m = 1 / (rF4 * rRR);")
    for a in AXES:
        src.append(f"a{a} = 1 / (rf{HALVES[a][0][1]} * rh{a}); b{a} = 1 / (rf{HALVES[a][1][1]} * rk{a});")
    for a in live:
        src.append(f"WH{a} = K{a} - m * a{a} * a{a}; WK{a} = K{a} - m * b{a} * b{a}; WX{a} = K{a} - m * a{a} * b{a};")
    for a in live:
        src.append(f"cH{a} = F4 * |h{a}| * |h{a}|; cK{a} = F4 * |k{a}| * |k{a}|; cX{a} = 2 * F4 * |h{a}| * |k{a}|;")
    for a in live:
        src.append(f"eH{a} = -(F4 * h{a} * h{a} * WH{a}); eK{a} = -(F4 * k{a} * k{a} * WK{a}); "
                   f"eX{a} = 2 * F4 * h{a} * k{a} * WX{a};")
    if live:
        src.append(" ".join(f"C3{a} = cH{a} + cK{a} + cX{a};" for a in live))
        src.append(f"C9 = {' + '.join(f'C3{a}' for a in live)};")
        src.append(" ".join(f"Y{a} = F4 * K{a} * Q{a} * Q{a};" for a in live))
        src.append(" ".join(f"Z{a} = F4 * m * (C{a} * C{a});" for a in live))
    if dead:
        src += ["# What y misses of a component that cancels is its whole true term, and",
                "# EC is their sum."]
        src.append(" ".join(f"eC{a} = F4 * m * (C{a} * C{a});" for a in dead))
        ec = f"eC{dead[0]}"
        if len(dead) > 1:
            src.append(f"EC = {' + '.join(f'eC{a}' for a in dead)};")
            ec = "EC"
    src += [
        "",
        "# The same permanent as the two blocks.",
        "Phat2 = A6 + C9;" if live else "Phat2 = A6;",
        "",
        "{",
        "  # Every input is a finite f64.",
        "     |px| <= 1b1024 /\\ |py| <= 1b1024 /\\ |pz| <= 1b1024",
        "  /\\ |ox| <= 1b1024 /\\ |oy| <= 1b1024 /\\ |oz| <= 1b1024",
        "  /\\ |dx| <= 1b1024 /\\ |dy| <= 1b1024 /\\ |dz| <= 1b1024",
        "  /\\ |rg| <= 1b1024 /\\ |tb| <= 1b1024",
    ]
    if dead:
        src += ["  # No factor is zero, and no component but those the case names cancels to",
                "  # zero. For an f64 that is the same as being at least the smallest",
                "  # subnormal."]
    else:
        src += ["  # No factor is zero, and no component cancels to zero. For an f64 that is",
                "  # the same as being at least the smallest subnormal."]
    src += [
        "  /\\ |fx| >= 1b-1074 /\\ |fy| >= 1b-1074 /\\ |fz| >= 1b-1074",
        "  /\\ |dx| >= 1b-1074 /\\ |dy| >= 1b-1074 /\\ |dz| >= 1b-1074",
        "  /\\ |rg| >= 1b-1074 /\\ |tb| >= 1b-1074",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|q{a}| >= 1b-1074" for a in live))
    if hol:
        src += ["  # hollow is a normal number; below the normals it is exact, and that is",
                "  # another case.",
                "  /\\ |hollow| >= 1b-1022"]
    else:
        src += ["  # hollow is below the normals, where it is exact.",
                "  /\\ |hollow| <= 1b-1022"]
    src += [
        "  # product_is_normal on the three squares of from_centre, of direction and",
        "  # of sideways, the six halves, R^2, r^2, bulge^2, axis_squared bulge^2 and",
        "  # 4 R^2 across (stays_accurate and the check after it).",
        "  /\\ |gx| >= 1b-1022 /\\ |gy| >= 1b-1022 /\\ |gz| >= 1b-1022",
        "  /\\ |tx| >= 1b-1022 /\\ |ty| >= 1b-1022 /\\ |tz| >= 1b-1022",
    ]
    if live:
        src.append("  /\\ " + " /\\ ".join(f"|s{a}| >= 1b-1022" for a in live))
    src += [
        "  /\\ |hx| >= 1b-1022 /\\ |kx| >= 1b-1022 /\\ |hy| >= 1b-1022",
        "  /\\ |ky| >= 1b-1022 /\\ |hz| >= 1b-1022 /\\ |kz| >= 1b-1022",
        "  /\\ |RR| >= 1b-1022 /\\ |rr| >= 1b-1022",
        "  /\\ |bb| >= 1b-1022 /\\ |left| >= 1b-1022 /\\ |right| >= 1b-1022" if live
        else "  /\\ |bb| >= 1b-1022 /\\ |left| >= 1b-1022",
    ]
    if not dead:
        src += [
            "  # The permanent's products are normal too, since nothing cancels here, by",
            "  # monotone rounding: v >= s, as no component is zero; br >= bulge, as",
            "  # hollow <= RR, so brr >= bb and P1 >= left; spread >= across, so",
            "  # P2 >= right.",
            "  /\\ |vx| >= 1b-1022 /\\ |vy| >= 1b-1022 /\\ |vz| >= 1b-1022",
            "  /\\ |brr| >= 1b-1022 /\\ |P1| >= 1b-1022 /\\ |P2| >= 1b-1022",
        ]
    else:
        src += [
            "  # The permanent's products, by monotone rounding: v >= s for a component",
            "  # that does not cancel; for one that does the case says which v is, and",
            "  # 464.14's tinysq.g and tinyzero.g split them at 2^-1075. br >= bulge, as",
            "  # hollow <= RR, so brr >= bb and P1 >= left. "
            + ("spread >= across, so P2 >= right." if live else "The case says what P2 is."),
        ]
        rows = []
        for a in AXES:
            if st[a] in "nc":
                rows.append(f"|v{a}| >= 1b-1022")
            elif st[a] == "s":
                rows.append(f"Xv{a} <= 1b-1022 /\\ v{a} >= 1b-1074")
            else:
                rows.append(f"Xv{a} <= 1b-1075")
        src.append("  /\\ " + " /\\ ".join(rows))
        tail = {"n": " /\\ |P2| >= 1b-1022", "b": " /\\ XP2 <= 1b-1022" if kept else ""}[p2]
        src.append("  /\\ |brr| >= 1b-1022 /\\ |P1| >= 1b-1022" + tail)
    src += [
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
        "  # What lemma.g proves each of those gives.",
    ]
    if dead:
        src.append("  # k is h in a component that cancels, so its line is h's against k's product.")
    if hol:
        src.append(f"  /\\ hollow -/ H {LEMMA}")
    for p in ("gx -/ fx * fx", "gy -/ fy * fy", "gz -/ fz * fz",
              "tx -/ dx * dx", "ty -/ dy * dy", "tz -/ dz * dz"):
        src.append(f"  /\\ {p} {LEMMA}")
    for a in live:
        src.append(f"  /\\ s{a} -/ q{a} * q{a} {LEMMA}")
    for a in AXES:
        left, right = HALVES[a]
        src.append(f"  /\\ h{a} -/ {left} {LEMMA}")
        src.append(f"  /\\ k{a} -/ {right} {LEMMA}")
    for p in ("RR -/ rg * rg", "rr -/ tb * tb", "bb -/ bulge * bulge", "left -/ W * bb"):
        src.append(f"  /\\ {p} {LEMMA}")
    if live:
        src.append(f"  /\\ right -/ F4 * across {LEMMA}")
    for a in AXES:
        if st[a] in "nc":
            src.append(f"  /\\ v{a} -/ n{a} * n{a} {LEMMA}")
    src.append(f"  /\\ brr -/ br * br {LEMMA}")
    src.append(f"  /\\ P1 -/ W * brr {LEMMA}")
    if p2 == "n":
        src.append(f"  /\\ P2 -/ F4 * spread {LEMMA}")
    src.append(f"  /\\ eb -/ B * P {LEMMA}")
    if live:
        src.append("  # x / |x| is in [-1, 1] for any x that is not zero.")
        for a in live:
            src.append(f"  /\\ h{a} / |h{a}| in [-1, 1] /\\ k{a} / |k{a}| in [-1, 1]")
    src += [
        "->",
        "  (y - Ex) / Phat2 in ? /\\ P -/ " + ("Phatx" if split else "Phat") + " in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
        "# How far each rounding is from the identity.",
        "fx / Fx -> 1 + (fx - Fx) / Fx;",
        "fy / Fy -> 1 + (fy - Fy) / Fy;",
        "fz / Fz -> 1 + (fz - Fz) / Fz;",
    ]
    for a in AXES:
        for h, prod in zip("hk", HALVES[a]):
            src.append(f"{h}{a} / ({prod}) -> 1 + ({h}{a} - {prod}) / ({prod});")
    for a in live:
        src.append(f"q{a} / Q{a} -> 1 + (q{a} - Q{a}) / Q{a};")
    src += [
        "gx / (fx * fx) -> 1 + (gx - fx * fx) / (fx * fx);",
        "gy / (fy * fy) -> 1 + (gy - fy * fy) / (fy * fy);",
        "gz / (fz * fz) -> 1 + (gz - fz * fz) / (fz * fz);",
        "o01 / O01 -> 1 + (o01 - O01) / O01;",
        "oab / O -> 1 + (oab - O) / O;",
        "RR / (rg * rg) -> 1 + (RR - rg * rg) / (rg * rg);",
        "rr / (tb * tb) -> 1 + (rr - tb * tb) / (tb * tb);",
    ]
    if hol:
        src.append("hollow / H -> 1 + (hollow - H) / H;")
    src += [
        "bulge / Bu -> 1 + (bulge - Bu) / Bu;",
        "tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);",
        "ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);",
        "tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);",
        "t01 / T01 -> 1 + (t01 - T01) / T01;",
        "W / T -> 1 + (W - T) / T;",
    ]
    for a in live:
        src.append(f"s{a} / (q{a} * q{a}) -> 1 + (s{a} - q{a} * q{a}) / (q{a} * q{a});")
    if s01_rounds:
        src.append("s01 / S01 -> 1 + (s01 - S01) / S01;")
    if s_rounds:
        src.append("across / S -> 1 + (across - S) / S;")
    src += [
        "F4 / (4 * RR) -> 1 + (F4 - 4 * RR) / (4 * RR);",
        "bb / (bulge * bulge) -> 1 + (bb - bulge * bulge) / (bulge * bulge);",
        "left / (W * bb) -> 1 + (left - W * bb) / (W * bb);",
    ]
    if live:
        src.append("right / (F4 * across) -> 1 + (right - F4 * across) / (F4 * across);")
    src += [
        "",
        "# w.w over W, and |x|^2 over oab, each averaged over three pieces.",
        "Wt / W -> 1 + ((W3 + (Wt - W)) - W3) / W3;",
        "W3 + (Wt - W) -> (Wx + Wx * jx) + (Wy + Wy * jy) + (Wz + Wz * jz);",
        "(Wx + Wx * jx) - Wx -> Wx * jx;",
        "(Wy + Wy * jy) - Wy -> Wy * jy;",
        "(Wz + Wz * jz) - Wz -> Wz * jz;",
        "G2 / oab -> 1 + ((O3 + (G2 - oab)) - O3) / O3;",
        "O3 + (G2 - oab) -> (Ox + Ox * zx) + (Oy + Oy * zy) + (Oz + Oz * zz);",
        "(Ox + Ox * zx) - Ox -> Ox * zx;",
        "(Oy + Oy * zy) - Oy -> Oy * zy;",
        "(Oz + Oz * zz) - Oz -> Oz * zz;",
        "",
        "# The error is the sum of the two blocks' errors, each the sum of its",
        "# pieces' errors, each a fraction of its own piece of the permanent; add_rr",
        "# then averages them.",
    ]
    if dead:
        src += ["# What y misses of a component that cancels goes with the piece cOR, and",
                "# is at most a u^2 of it."]
    src.append("(y - Ex) / Phat2 -> ((Phat2 + (y - Ex)) - Phat2) / Phat2;")
    oor = "(cOR + eOR)"
    if dead:
        oor = f"(cOR + (eOR + {ec}))"
        a6 = f"(A6 + ((left - Lt) + {ec}))"
        src.append("Phat2 + (y - Ex) -> " + (f"{a6} + (C9 + (Rt - right - {ec}));" if live else f"{a6};"))
        src.append(f"A6 + ((left - Lt) + {ec}) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + {oor} "
                   "+ (cOr + eOr) + (cRr + eRr);")
    else:
        src.append("Phat2 + (y - Ex) -> (A6 + (left - Lt)) + (C9 + (Rt - right));")
        src.append("A6 + (left - Lt) -> (cOO + eOO) + (cRR + eRR) + (crr + err) + (cOR + eOR) "
                   "+ (cOr + eOr) + (cRr + eRr);")
    src += [
        "(cOO + eOO) - cOO -> eOO;",
        "(cRR + eRR) - cRR -> eRR;",
        "(crr + err) - crr -> err;",
        f"(cOR + (eOR + {ec})) - cOR -> eOR + {ec};" if dead else "(cOR + eOR) - cOR -> eOR;",
        "(cOr + eOr) - cOr -> eOr;",
        "(cRr + eRr) - cRr -> eRr;",
        "eOr / cOr -> -WOr;",
        "eRr / cRr -> -WRr;",
    ]
    if dead:
        src.append(f"(eOR + {ec}) / cOR -> WOR + {ec} / cOR;")
        if len(dead) > 1:
            src.append(f"EC / cOR -> {' + '.join(f'eC{a} / cOR' for a in dead)};")
        for a in dead:
            f_, d_ = HALVES[a][0][1], HALVES[a][0][-1]
            src += [
                f"eC{a} / cOR -> 2 * rF4 * m * (a{a} - b{a}) * (a{a} - b{a}) * rh{a} * rh{a}"
                f" * (f{f_} * f{f_} / oab) * (d{d_} * d{d_} / W);",
                f"f{f_} * f{f_} / oab -> (g{f_} / oab) / rg{f_};",
                over_sum("g", f_, "ro01", "ro", "oab"),
                f"d{d_} * d{d_} / W -> (t{d_} / W) / rt{d_};",
                over_sum("t", d_, "rt01", "rW", "W"),
            ]
    if live:
        rest = f"C9 + (Rt - right - {ec})" if dead else "C9 + (Rt - right)"
        src.append(f"{rest} -> " + " + ".join(f"(C3{a} + (Z{a} - Y{a}))" for a in live) + ";")
        for a in live:
            src.append(f"C3{a} + (Z{a} - Y{a}) -> (cH{a} + eH{a}) + (cK{a} + eK{a}) + (cX{a} + eX{a});")
        for a in live:
            for p in "HKX":
                src.append(f"(c{p}{a} + e{p}{a}) - c{p}{a} -> e{p}{a};")
        for a in live:
            src += [f"eH{a} / cH{a} -> -((h{a} / |h{a}|) * (h{a} / |h{a}|) * WH{a});",
                    f"eK{a} / cK{a} -> -((k{a} / |k{a}|) * (k{a} / |k{a}|) * WK{a});",
                    f"eX{a} / cX{a} -> (h{a} / |h{a}|) * (k{a} / |k{a}|) * WX{a};"]
    src += ["", "# From the two blocks to the permanent on_floats rounds, and on to eb."]
    if not split:
        src += ["(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat / P) * (P / eb);",
                "Phat / P -> 1 / (1 + (P - Phat) / Phat);"]
    elif dx_terms:
        src += ["(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phat2 / P) * (P / eb);",
                "Phat2 / P -> (Phat2 / Phatx) * (Phatx / P);",
                "Phat2 / Phatx -> 1 / (1 + Dx / Phat2);",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);"]
    else:
        src += ["(y - Ex) / eb -> ((y - Ex) / Phat2) * (Phatx / P) * (P / eb);",
                "Phatx / P -> 1 / (1 + (P - Phatx) / Phatx);"]
    src.append("P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));")
    return "\n".join(src) + "\n"


def write(name, text):
    with open(name, "w") as out:
        out.write(text)


def cases():
    for cross in map("".join, itertools.product("ncsu", repeat=3)):
        for hollow in "ne":
            if "n" in cross:
                yield cross + hollow + "n"
            elif cross == "uuu":
                yield cross + hollow + "b"
            else:
                yield cross + hollow + "n"
                yield cross + hollow + "b"


for case in cases():
    write(f"tor_{case}.g", make(case))
    if case != "nnnnn":
        write(f"c_tor_{case}_half.g", make(case, B_HALF))
```

`trial.sh`, whole:

```sh
#!/bin/bash
# Trial runs of torsplit.py's cases, each with its u/2 control, all at once
# through one1.sh. Prints to trial.txt.
cd "$(dirname "$0")"
{
  for c in cnnnn cccnn nnnen uuueb cccnb scnnn ssuen uuunb; do
    T=600 ./one1.sh "tor_$c.g" &
    T=600 ./one1.sh "c_tor_${c}_half.g" &
  done
  wait
  echo done
} > trial.txt 2>&1
```

`run3.sh`, whole:

```sh
#!/bin/bash
# torsplit.py's 180 cases, each with gappa -Bcoq and coqc when it proves,
# and their 179 controls at u/2, 20 at a time: the cases first, since they
# take longest. Writes run3.txt.
cd "$(dirname "$0")"
python3 torsplit.py
{
  for f in tor_?????.g; do
    echo "./one.sh $f coq"
  done
  for f in c_tor_?????_half.g; do
    echo "./one1.sh $f"
  done
} > run3.jobs
{
  wc -l < run3.jobs
  xargs -P 20 -L 1 env < run3.jobs
} > run3.txt 2>&1
echo done >> run3.txt
```

`run4.sh`, whole:

```sh
#!/bin/bash
# gappa -Bcoq and coqc again on tor_cusen.g, the one case whose certificate
# coqc refused in run3.txt. It runs on a copy, tor_cusen2.g, so that
# tor_cusen.coqc, which tally3.py reads, stays as run3.sh left it. Writes
# run4.txt.
cd "$(dirname "$0")"
cp tor_cusen.g tor_cusen2.g
{
  cmp tor_cusen.g tor_cusen2.g && echo "tor_cusen2.g is tor_cusen.g"
  ./one.sh tor_cusen2.g coq
  echo "coqc printed:"
  cat tor_cusen2.coqc
  echo done
} > run4.txt 2>&1
```

`tally3.py`, whole:

```python
"""Tally run3.txt and the .out files of torsplit.py's 180 cases: for each case
its exit, the bound on (y - Ex) / Phat2 and on P against the permanent with
no rounding of its own that it prints, and its coqc line; for each control its
exit and best. Prints one line per case, then counts, grouped by how many
components of the cross product do not cancel and by the hollow and P2
letters. Run as: python3 tally3.py"""
import collections
import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))
run = open("run3.txt").read()
cases = sorted(f[4:9] for f in os.listdir(".") if re.fullmatch(r"tor_\w{5}\.g", f))
RESULT = re.compile(r"^  (.*) in \[[^{]*\{([^,]*),[^}]*\}, [^{]*\{([^,]*),", re.M)


def bounds(out):
    """The y goal's and the P goal's printed ranges, as [lo, hi]. Where y is
    left, Gappa prints the goal with left and Phat2 written out."""
    y = p = "-"
    for expr, lo, hi in RESULT.findall(out):
        if " -/ " in expr:
            p = f"[{lo}, {hi}]"
        elif "- Ex) / " in expr:
            y = f"[{lo}, {hi}]"
    return y, p


ybounds = collections.Counter()
pbounds = collections.Counter()
bests = collections.Counter()
ctl_exits = collections.Counter()
pointless = []
times = []
coq = []
for c in cases:
    out = open(f"tor_{c}.out").read()
    ex = re.search(rf"^tor_{c} exit (\d+) ([\d.]+)s$", run, re.M)
    cq = re.search(rf"^tor_{c} coq -Bcoq exit (\d+) coqc exit (\d+) ([\d.]+)s lines (\d+) hints (\d+)$", run, re.M)
    live = sum(1 for s in c[:3] if s == "n")
    y, p = bounds(out)
    if "pointless" in out:
        pointless.append(c)
        y = p = "pointless"
    ybounds[(live, c[3:], y)] += 1
    pbounds[p] += 1
    times.append((float(ex.group(2)), c))
    if cq:
        coq.append((int(cq.group(4)), int(cq.group(5)), c, cq.group(1), cq.group(2), float(cq.group(3))))
    line = [c, "exit", ex.group(1), ex.group(2), "y", y, "P", p, "coq", cq.groups() if cq else "-"]
    if c != "nnnnn":
        ctl = re.search(rf"^c_tor_{c}_half exit (\d+) ([\d.]+)s$", run, re.M)
        cout = open(f"c_tor_{c}_half.out").read()
        best = re.search(r"best: (\[[^\]]*\])", cout)
        bests[(ctl.group(1), best.group(1) if best else "pointless" if "pointless" in cout else "-")] += 1
        ctl_exits[(live, c[3:], ctl.group(1))] += 1
        line += ["ctl", ctl.group(1), best.group(1) if best else "-"]
    print(*line)
print("cases exit 0:", len(re.findall(r"^tor_\w{5} exit 0 ", run, re.M)), "of", len(cases))
print("controls exit 1:", len(re.findall(r"^c_tor_\w{5}_half exit 1 ", run, re.M)), "of", len(cases) - 1)
print("pointless:", pointless)
print("y bounds by live components and hollow and P2 letters:")
for k in sorted(ybounds):
    print("  ", k, ybounds[k])
print("P bounds:", dict(pbounds))
print("control exits by live components and hollow and P2 letters:")
for k in sorted(ctl_exits):
    print("  ", k, ctl_exits[k])
print("control exits and bests:", dict(bests))
times.sort()
print("times", times[0], times[-1])
print("coq ok:", sum(1 for x in coq if x[3] == "0" and x[4] == "0"), "of", len(coq))
coq.sort()
print("coq smallest", coq[0][:3], "largest", coq[-1][:3], "hints", min(x[1] for x in coq), max(x[1] for x in coq),
      "times", min(x[5] for x in coq), max(x[5] for x in coq))


def other_lines(f):
    return [line for line in open(f).read().split("\n")
            if line and "assumed to be nonzero" not in line and not line.startswith(("Results:", "  "))]


warn = [f for f in os.listdir(".") if re.fullmatch(r"tor_\w{5}\.(out|bcoq|coqc)", f) and other_lines(f)]
print("files with a line other than results and 'assumed to be nonzero':", sorted(warn))
```

`shapes.py`, whole:

```python
"""Which case of torsplit.py a shape falls in: on_floats' doubles, worked in
Python's, which are IEEE doubles rounded to nearest even as Rust's are. The
direction is the nearest double to each component over the exact length,
which is what Axis::new gives above 2^-1012. Prints, for each shape, the
from_centre and direction components that are zero, each cross product
component's letter, hollow's and P2's, and whether on_floats' checks pass.
Run as: python3 shapes.py"""
from decimal import Decimal, getcontext

getcontext().prec = 60
NORMAL = 2.0 ** -1022


def unit(v):
    length = sum(Decimal(c) * Decimal(c) for c in v).sqrt()
    return [float(Decimal(c) / length) for c in v]


def normal(a, b):
    return a == 0.0 or b == 0.0 or abs(a * b) >= NORMAL


def case(name, origin, direction, ring, tube, point):
    d = unit(direction)
    f = [point[i] - origin[i] for i in range(3)]
    halves = [(f[1] * d[2], f[2] * d[1]), (f[2] * d[0], f[0] * d[2]), (f[0] * d[1], f[1] * d[0])]
    q = [h - k for h, k in halves]
    checks = all(normal(f[i], f[i]) and normal(d[i], d[i]) and normal(q[i], q[i]) for i in range(3))
    checks = checks and all(normal(f[(i + 1) % 3], d[(i + 2) % 3]) and normal(f[(i + 2) % 3], d[(i + 1) % 3])
                            for i in range(3))
    checks = checks and normal(ring, ring) and normal(tube, tube)
    oab = f[0] * f[0] + f[1] * f[1] + f[2] * f[2]
    hollow = ring * ring - tube * tube
    bulge = oab + hollow
    w = d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
    across = q[0] * q[0] + q[1] * q[1] + q[2] * q[2]
    f4 = 4.0 * (ring * ring)
    checks = checks and normal(bulge, bulge) and normal(w, bulge * bulge) and normal(f4, across)
    letters = ""
    spread = 0.0
    for (h, k), qa in zip(halves, q):
        v = (abs(h) + abs(k)) * (abs(h) + abs(k))
        spread += v
        letters += "n" if qa != 0.0 else "c" if v >= NORMAL else "s" if v > 0.0 else "u"
    letters += "n" if abs(hollow) >= NORMAL else "e"
    letters += "n" if f4 * spread >= NORMAL else "b"
    zeros = [f"f{'xyz'[i]}" for i in range(3) if f[i] == 0.0] + [f"d{'xyz'[i]}" for i in range(3) if d[i] == 0.0]
    print(f"{name}: direction {d}, from_centre {f}, halves {halves}")
    print(f"  zero factors {zeros or 'none'}; case {letters}; on_floats' checks pass: {checks}")


case("on the axis (1, 0, 0)", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0, 1.0, [2.0, 0.0, 0.0])
case("on the axis (1, 1, 1)", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2.0, 1.0, [1.0, 1.0, 1.0])
case("off the axis (1, 1, 1), one component cancelling", [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2.0, 1.0,
     [1.0, 1.0, 2.0])
```

`same.py`, whole:

```python
"""Check that each script 464.17 appends whole is byte for byte the file that
was run, in tor/; that one.sh is 464.13's block and one1.sh and launch.sh
464.16's; that tally3.py and shapes.py, run again there, print what 464.17
gives; and that torsplit.py and torcases.py, run again in a fresh directory
from the appended text with the arguments of run1.sh, write what was run.
Run as: python3 same.py <doc> <tor dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, tor = sys.argv[1], sys.argv[2]
text = open(doc).read()
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)


def section(n):
    start = text.index(f"### 464.{n} ")
    end = text.find("\n### ", start + 1)
    return text[start:] if end < 0 else text[start:end + 1]


sec = section(17)
blocks = FENCED.findall(sec)
print("code blocks in 464.17:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
marks = {
    "tor.g": "# restates: crates/rcad-geom/src/torus.rs on_floats at 4be293a5\n",
    "torcases.py": '"""Write the scripts of 464.17 from tor.g',
    "torsplit.py": '"""Write tor.g\'s case split',
    "run0.sh": "#!/bin/bash\n# tor.g, the case where nothing cancels",
    "run1.sh": "#!/bin/bash\n# tor.g's controls and its bisection",
    "trial.sh": "#!/bin/bash\n# Trial runs of torsplit.py's cases",
    "run3.sh": "#!/bin/bash\n# torsplit.py's 180 cases",
    "run4.sh": "#!/bin/bash\n# gappa -Bcoq and coqc again",
    "ku.py": '"""Print each result of the named .out files as k u + r u^2',
    "tally3.py": '"""Tally run3.txt and the .out files of torsplit',
    "shapes.py": '"""Which case of torsplit.py a shape falls in',
    "same.py": '"""Check that each script 464.17 appends whole',
}
found = {}
for lang, body in blocks:
    for name, mark in marks.items():
        if not body.startswith(mark):
            continue
        assert name not in found, name
        found[name] = body
for name in marks:
    if name not in found:
        print(name, "not appended")
        continue
    path = os.path.abspath(__file__) if name == "same.py" else os.path.join(tor, name)
    if not os.path.exists(path):
        print(name, "appended, no file to compare")
        continue
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")


def block_of(n, first):
    got = [body for lang, body in FENCED.findall(section(n)) if body.startswith(first)]
    assert len(got) == 1, (n, first, len(got))
    return got[0]


print("one.sh is 464.13's block:", open(os.path.join(tor, "one.sh")).read()
      == block_of(13, "#!/bin/bash\n# Run one Gappa script under 464.4's pin"))
print("one1.sh is 464.16's block:", open(os.path.join(tor, "one1.sh")).read()
      == block_of(16, "#!/bin/bash\n# one.sh with the timeout taken from T"))
print("launch.sh is 464.16's block:", open(os.path.join(tor, "launch.sh")).read()
      == block_of(16, "#!/bin/bash\n# Start the script named by the first argument detached"))

for script, first in (("tally3.py", "cases exit 0: "), ("shapes.py", "on the axis (1, 0, 0): ")):
    printed = [body for lang, body in blocks if lang == "text" and body.startswith(first)]
    assert len(printed) == 1, (script, len(printed))
    again = subprocess.run(["python3", os.path.join(tor, script)], capture_output=True, text=True,
                           check=True).stdout
    print(script, "run again ends with what 464.17 gives:", again.endswith(printed[0]))


def compare(fresh, ran):
    written = sorted(f for f in os.listdir(fresh) if f.endswith(".g"))
    same = [f for f in written if os.path.exists(os.path.join(ran, f))
            and filecmp.cmp(os.path.join(fresh, f), os.path.join(ran, f), shallow=False)]
    differ = [f for f in written if os.path.exists(os.path.join(ran, f)) and f not in same]
    absent = [f for f in written if not os.path.exists(os.path.join(ran, f))]
    return written, same, differ, absent


fresh = tempfile.mkdtemp()
for name in ("tor.g", "torcases.py", "torsplit.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "torsplit.py"], cwd=fresh, check=True)
subprocess.run(["python3", "torcases.py", "0", "100", "200", "228", "230", "231", "232", "233", "236", "300"],
               cwd=fresh, check=True)
written, same, differ, absent = compare(fresh, tor)
print("written again:", len(written), "same as run:", len(same), "differ:", differ, "not in tor:", absent)
print("tor_nnnnn.g is tor.g:", filecmp.cmp(os.path.join(fresh, "tor_nnnnn.g"), os.path.join(fresh, "tor.g"),
                                           shallow=False))
ran = sorted(f for f in os.listdir(tor) if f.endswith(".g"))
print(".g files in tor not written again:", [f for f in ran if f not in written])
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.17: 17
tor.g same
torcases.py same
torsplit.py same
run0.sh same
run1.sh same
trial.sh same
run3.sh same
run4.sh same
ku.py same
tally3.py same
shapes.py same
same.py same
one.sh is 464.13's block: True
one1.sh is 464.16's block: True
launch.sh is 464.16's block: True
tally3.py run again ends with what 464.17 gives: True
shapes.py run again ends with what 464.17 gives: True
written again: 383 same as run: 383 differ: [] not in tor: []
tor_nnnnn.g is tor.g: True
.g files in tor not written again: ['tor_cusen2.g']
```

The 383 are `tor.g` itself, the 180 cases, their 179 controls at `u / 2`, the
three controls of 7, and the ten scripts each of `b_tor_` and `b17_tor_`.
Neither generator writes `tor_cusen2.g`; `run4.sh` makes it with `cp`, and
checks it with `cmp`.

**10. What this leaves.** Three rows, filed from this write-up when it merges,
each from Q2682:

- Small: restate `torus.rs:485-507` by 5. What the `220ε²` has to beat is
  `ε²`, not `ε`, and by the comment's own terms `153 + 126 + 18` make 297,
  against 220, which is 224 as a double. The chain reaches seventeen roundings
  to `y` by two slips that cancel, where Gappa proves `17u + 95u²` on a chain
  where nothing cancels. And `on_floats` does not enforce the bottom end
  product by product, since it checks none of the permanent's own products.
- Prove the bound where a difference or a direction component is zero, the
  axis-aligned torus among them, as Q2771 does for the cylinder and Q2781 for
  the cone. `product_is_normal(a, b)` is true when `a` or `b` is zero
  (`cascade.rs:67-69`), so these cases pass every check `on_floats` makes, as
  `shapes.py`'s first shape does, and no script here covers them. Not run, so
  no class (§360.5).
- Run `torsplit.py` with one, two and three of the products `on_floats` checks
  below the normals, as 464.14's 6 ran the sphere's squares. Then say whether
  `on_floats` needs its `product_is_normal` checks, and restate
  `torus.rs:347-349` by what that shows. Not run, so no class. Q2767 waits on
  it, as well as on Q2684, Q2760, Q2773 and Q2782.

And what is not a row:

- Q2700 already waits on Q2682, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- `tor_cusen.g`'s first certificate, which `coqc` refused (8). Gappa wrote a
  certificate that does not check, for a bound its own run proved, and a
  second certificate for the same bytes checks.
- The claim on `value` (4): `decide` needs only `y`.
- `field` on these scripts' hints (464.10).
