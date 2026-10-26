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
464.17 is Q2682, on the side of rcad-geom's torus. 464.18 is Q2684, on the
side of a plane with every product anywhere, 464.19 is Q2689, on
`Isometry::POINT_ROUNDOFF`, and 464.20 is Q2690, on
`Plane::NORMAL_LENGTH_ROUNDOFF`.*

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
  464.18  The side of a plane with every product anywhere
  464.19  Isometry::POINT_ROUNDOFF
  464.20  Plane::NORMAL_LENGTH_ROUNDOFF

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
one at a time, and each proves (464.11). Q2747 runs each set of two or more
that leaves `y` not zero by its shape, and each proves (464.25). The branch
where `t1 + t2` is zero was run: the goal proves, with `|y − Ex| / P6 ≤ 6u +
13u²`. The branch
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
bound. Since 464.25 each set of two or more differences at zero at once that
leaves `y` not zero is run too, and proves at the code's bound. Since 464.26
each branch with one difference at zero and one minor cancelling to zero is
run too, and proves at the code's bound. The step from `y` to the sign is
still on paper.

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
or more differences at zero at once are run in 464.25. One difference at zero
with one minor at zero is run in 464.26.

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
  minor too: `adx` and `bdx` zero both of `third`'s products. A row. Done, in
  464.25: each of the 237 branches proves at the code's bound.
- One difference at zero with one minor at zero, 464.7's case, such as `adx`
  with `first`. A row. Done, in 464.26: each of the 15 branches proves at the
  code's bound, and the other 12 pairs cannot happen.
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

### 464.18 The side of a plane with every product anywhere

Q2684 took up `SIDE_OF_PLANE_ERROR_BOUND` again, in four lanes on
2026-10-25, for what 464.8's 4 left unrun: two and three of
`SideOfAFlatSurface`'s products below the normals at once, the zero branches
(a zero factor, a sum that cancels to zero, `planev.g`'s zero offset), and a
zero difference in orient2d. The first lane wrote `planeall.py`. The second
ran every script it writes. The third wrote the controls for the cases with
no normal product and the orient2d scripts, ran everything again, restated
`plane.rs` and wrote 1 to 8. The fourth wrote 9 and 10, and ran every script
again as 9 gives it, in a fresh tree.

The bound is proved at the code's constant in every case of where the three
products land, normal, zero or below the normals, with any of the sums
cancelling, and on `value` with the offset zero. So `on_floats` does not need
its three `product_is_normal` checks to give a right sign. They are kept, as
Q2766 kept the sphere's (464.14), and the comment on them and the doc on the
constant now say why (7). orient2d with a zero difference proves too, or has
no normal bound for `decide` to compare against (5). Coq checks each proof,
modulo its hints (8).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, against
`plane.rs`, `orient2d.rs` and `cascade.rs` at `6a4a814c`, through 464.13's
`one.sh`, eight at a time. `u` is `2⁻⁵³`. Plane's `B = 4u + 32u²` is written
`B = 4b-53 + 1b-101;` and orient2d's `3u + 16u²` `B = 3b-53 + 1b-102;`. Each
script follows 464.8's pattern: one ratio hint per rounding of a normal
number, the error split into one term per product so that Gappa averages
them, 464.5's lemma for each product that rounds to a normal number and for
`eb`, and what rounding a product below the normals cost carried as an
amount, as `o2s.g` carries it.

**1. What the proof guards.** `SideOfAFlatSurface` is the cascade's predicate
for `which_side_of_a_flat_surface` (`plane.rs:816-832`), which
`Plane::side_of_point` (`plane.rs:621-623`) and `HalfSpace::side_of_point`
(`half_space.rs:352-360`) call, both public, and `three_surfaces.rs:3288` for
a `Surface::Plane`.

**2. `on_floats`, line for line.** `SideOfAFlatSurface::on_floats`
(`plane.rs:847-869` at `6a4a814c`) returns `None` for a point that is not
exact `f64`s, and unless `product_is_normal` holds for each of the three
products (`plane.rs:854-856`). Then it takes the three products, `value = t0
+ t1 + t2 − offset` folded left to right, and `permanent = |t0| + |t1| + |t2|
+ |offset|` folded the same way (`plane.rs:858-868`). Each script writes it as
464.8's `plane.g` does, with the check left out: a rounding for each product,
one for each of the two sums, `y = s2 − off`, which the code rounds to
`value`, and `P` the permanent with each of its three additions rounded.
`product_is_normal(a, b)` is true when `a` or `b` is zero (`cascade.rs:67-69`),
so a zero product gets through the check. `decide` compares only against a
bound that is a normal number (`cascade.rs:125`), and the doc on
`Filtered::error_bound` asks for the bound on `y`, the sum before `value`'s
last rounding.

**3. Every place the three products can land.** Each product is in one of
four states:

- `n`, normal: it rounds to at least `2⁻¹⁰²²`, as `product_is_normal` lets
  through;
- `z`, of a zero factor, which `product_is_normal` lets through too;
- `p` and `m`, below the normals and positive or negative: the exact product
  `X` is in `[2⁻²¹⁴⁸, 2⁻¹⁰²²]` or its negative, which `product_is_normal`
  refuses.

Those cover every product. A product of two doubles that are not zero is at
least `2⁻²¹⁴⁸`, and one that rounds below `2⁻¹⁰²²` is below it itself, since
rounding is monotone and `2⁻¹⁰²²` is a double. A `p` or `m` product is
written `t = rnd(X)`, which may be zero, and what that rounding cost is an
amount, `d`, that Gappa bounds whatever `t` is, so no split at `2⁻¹⁰⁷⁵` is
needed, as 464.14's 6 needed one for its hints. A zero product is left out of
both folds, adding zero to a double giving the double back (464.13's
`addzero.g`).

Each rounded sum of the products left is either at least `2⁻¹⁰⁷⁴`, since a
sum of two doubles is a multiple of `2⁻¹⁰⁷⁴`, or it cancels to zero, rounds to
itself, and is written unrounded, as 464.13's `flatcases.py` writes it. Every
sum may cancel, not only one of two next to each other as in 464.13: a
product below the normals can round to zero, so the product after a cancelled
sum can be zero too. That makes 172 cases from the 64 states, 109 with a
normal product and 63 without.

- With a normal product, the step is 464.8's `planecases.py`'s, `(y − Ex) /
  eb -> ((Σ e) / P3x) · (P3x / P) · (P / eb) + (Σ d) / eb`: the relative part
  averaged over `P3x`, the products' part of the permanent with `X` in place
  of each `t` below the normals, and what rounding each `X` cost an amount
  that `eb` being normal keeps small. `plane_ppn.g`, whole in 9, is that form.
- With none, a case is not vacuous, unlike the sphere's with no normal square
  (464.14's 6): the offset is no product, so `|off|` alone can make `eb`
  normal. Those are written with every error an amount, `y − Ex` the sum of
  what each rounding cost, against `eb` at least `2⁻¹⁰²²`. `plane_ppp.g`,
  whole in 9, is that form.

`planev_off0.g` is the claim on `value` with the offset the double 0, all
three products normal: `y` is then `s2` less zero, which is `s2`, so `value`
is `y`.

**4. What they print.** `allplane.sh` (9) runs them, on a load of 7 to 22 over
24 cores.

- Each of the 172 cases and `planev_off0.g` proves at the code's `B`, and
  each `gappa -Bcoq` and `coqc` exits 0, in 45.4 s for all 173, and 43.0 s in
  the fourth lane's run. None prints "pointless".
- `plane_nnn.g` prints 464.8's `3u + 2u²` and `P −/ Phat` within `3u + 3u²`.
- The 96 cases with a normal product and one below the normals each print
  `P` at least `1125899906842623b-1021`, `(2⁵⁰ − 1) · 2⁻¹⁰²¹`, just under
  `2⁻⁹⁷¹`; the `d` sum within `2⁻¹⁰⁷⁵` for one product below the normals and
  `2⁻¹⁰⁷⁴` for two, each with a little more; and `(Σ e) / P3x` within `ku`
  and some `u²`, `k` the most roundings a normal product's term goes through:
  `3u` with no sum cancelling, `2u` with one or with the two below the normals
  first, `u` with both.
- The 63 cases with no normal product print `y − Ex` within `k · 2⁻¹⁰⁷⁵`, a
  share of `2⁻¹⁰⁷⁵` for each product below the normals and each rounded sum:
  `k` is 1 for 6 of them, 2 for 24, 3 for 28 and 5 for 4. `plane_zzz.g`
  prints `[0, 0]`. With `eb` at least `2⁻¹⁰²²`, 5 of them is `5u` of `eb`.
- `planev_off0.g` prints `value − y` in `[0, 0]`.

So the bound holds with any number of products below the normals because a
normal `eb` is `2⁻¹⁰²²` or more, which puts `P` at about `2⁻⁹⁷¹` or more: a
product below the normals then costs at most `2⁻¹⁰⁷⁵` against it, `u` of
`eb`, as 464.8's 2 and 4 found for one.

**5. orient2d's zero differences.** 464.8's 2 split orient2d on where its two
products land, and kept every difference not zero. A difference of two
doubles is zero only when the two are equal, and is then exactly zero, and so
is its product, so a zero difference makes `forwards` or `backwards` the
double 0. `o2zero.py` (9) writes five cases, split at `2⁻¹⁰⁰⁰` as 464.8's 2
splits:

- `o2zf.g`, `forwards` zero and `acy · bcx` at least `2⁻¹⁰⁰⁰` in magnitude,
  and `o2zb.g`, the same way round, each prove at the code's `B`, printing `e
  / P` within `3.33067e-16`, `3u + 6u²` by `summary.py`'s count, as `o2s.g`
  printed its relative part.
- `o2zf_tiny.g` and `o2zb_tiny.g`, the other product at most `2⁻¹⁰⁰⁰`, and
  `o2zfb.g`, both zero, each print "remaining results are pointless": `eb`
  cannot then be normal, so `decide` does not compare.

With 464.8's 2 that covers every input on which `decide` compares
`Orient2d`'s value against its bound. Each `gappa -Bcoq` and `coqc` exits 0.

**6. The controls.** A script whose hypotheses contradict each other proves
every goal (464.7's 3), so each case is run again where it must fail:

- Each of the 109 plane cases with a normal product, at `B = u / 2`
  (`c_<name>_half.g`), exits 1, best `[−2k, 2k]` for its case's `ku`, which
  `controls.py` checks for each. `c_planev_off0_half.g` exits 1, best
  `[−6, 6]`.
- Each of the 63 without, without `eb` normal (`c_<name>_noeb.g`), exits 1,
  best `[-inf, inf]`. That shows the case needs `eb` normal, not that its
  hypotheses do not contradict, so each is run again with every hypothesis
  and the last goal `(y − Ex) / eb` within `2⁻⁶⁰` (`c_<name>_tight.g`), which
  a real input misses: a product moved by `2⁻¹⁰⁷⁵` against `eb = 2⁻¹⁰²²` is
  `2⁻⁵³` of `eb`. `plane_zzz.g`'s `y − Ex` is zero, so its tight goal is `off`
  within `2⁻⁹⁰⁰` instead. Each of the 63 exits 1 and none prints "pointless".
  Each best is `k · 2⁻⁵³` for its case's `k · 2⁻¹⁰⁷⁵`, which `controls.py`
  checks, and `plane_zzz`'s the largest double.
- `o2zf.g` and `o2zb.g` at `u / 2` exit 1, best `[-6, 6]`. The three
  pointless ones without `eb` normal exit 1, best `[-inf, inf]`, the two tiny
  ones printing `eb` at most `3b-1053`, `2^(-1051.41)`, and `o2zfb.g` `eb` in
  `[0, 0]`. So what proves those three is the contradiction with `eb` normal
  and nothing else, as 464.8's `o2tiny.g`.

**7. What `plane.rs` said, and says now.** At `6a4a814c` the comment on the
check, `plane.rs:851-853`, said "The bound below is a fraction of the
permanent, which only bounds anything while every product keeps its relative
accuracy. One of them in the subnormals and this level certifies a wrong
sign." The doc on the constant, `plane.rs:924-928`, said the bound "holds
only while nothing overflows or falls into the subnormals", and that
`on_floats` "enforces the bottom end product by product through
`product_is_normal`". By 4 both are false: without the check, `on_floats`
would hand `decide` a number with one, two or three products below the
normals, and `decide` would certify no wrong sign from it.

So the three checks are not needed for a right sign. They are kept, a
decision taken here, as Q2766 took it for the sphere (`edbb4601`): with them,
level 1 answers only where the plain relative argument on the constant holds,
and the 172 cases stay a second proof rather than the only one. Dropping them
would move the inputs with a product below the normals from level 2 to level
1, and whether that turns any refusal into an answer was not run. The comment
on the check now says it is not needed for a right sign and what it does,
and the doc on the constant keeps the relative argument and says how far
down the bound holds, and why, citing this subsection. Comment only: lines
that run 0.

**8. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0, exit 0 on each script of 4
and 5 that proves or prints "pointless". Gappa does not write the same proof
each run (464.4), so the sizes are the third lane's:

- the 109 plane cases with a normal product: 903 to 3782 lines, 10 to 18
  hints;
- the 63 without: 271 to 1070 lines, 1 hint each, the `y − Ex ->` one;
- `planev_off0.g`: 3112 lines, 19 hints (3168 in the second lane's run);
- `o2zf.g` and `o2zb.g`: 1111 and 1095 lines, 6 hints each;
- `o2zf_tiny.g` and `o2zb_tiny.g`: 309 and 305 lines, 6 hints each, and
  `o2zfb.g` 135 lines with no hints.

The fourth lane's run gave the same but for three sizes: 903 to 3821 lines
for the first, 3117 for `planev_off0.g` and 1109 for `o2zb.g`.

Each hint enters the Coq file as a hypothesis with no condition attached, as
in 464.4, so each proof stands on its hints, and on the divisors Gappa took to
be nonzero, which its warnings name. `field` has not been run on these hints
(464.10).

**9. The scripts.** `planeall.py` writes the 172 cases, `planev_off0.g` and
the 236 plane controls; `o2zero.py` the five orient2d cases and their five
controls; `allplane.sh` and `o2run.sh` run them through `runmany.sh` and
464.13's `one.sh`, unchanged; `summary.py` and `controls.py` read what they
print. To run them again, save these seven and `one.sh` in one directory and
run `bash allplane.sh` and `bash o2run.sh` there. The fourth lane took each from
the text here, found it the same byte for byte as the file the third lane
ran, and ran the two in a fresh tree: they printed the same counts as 4 to 6
give, each case proving and each control failing. `plane_ppn.g`,
`plane_ppp.g` and `o2zf.g`, whole at the end, are as that run wrote them.

`planeall.py`, whole, as the third lane ran it: the first lane's, with the
`_tight` controls added.

```python
"""Write SideOfAFlatSurface::on_floats in every case of where its three
products land, as 464.8's planecases.py did for one product at a time:

- each product in one of four states: n, normal, as product_is_normal lets
  through; z, of a zero factor, which it lets through too; p and m, below the
  normals and positive or negative, which it refuses;
- each rounded sum of the products left either rounded, and then at least
  2^-1074, or cancelling to zero, which rounds to itself, and is then written
  unrounded;

plane_<states>[_c<set>].g for each, <states> one letter per product, <set>
naming the product each cancelling sum adds; c_<name>_half.g, each case with
a normal product again at B = u / 2, which must fail; c_<name>_noeb.g, each
case with none again without eb normal, which must fail; c_<name>_tight.g,
each case with none again with every hypothesis but the last goal 2^-60 of eb,
or for plane_zzz off within 2^-900, which must fail too, since a real input
misses it; planev_off0.g, the claim on value with the offset zero; and
c_planev_off0_half.g.

Run as: python3 planeall.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

B_CODE = "B = 4b-53 + 1b-101;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
STATES = "npmz"


TIGHT = "  /\\ (y - Ex) / eb in [-1b-60, 1b-60]"
TIGHT_ZZZ = "  /\\ off in [-1b-900, 1b-900]"


def make(states, cancel=(), b=B_CODE, eb_normal=True, tight=False):
    live = [k for k in range(3) if states[k] != "z"]
    small = [k for k in live if states[k] in "pm"]
    normal = [k for k in live if states[k] == "n"]

    def x(k):
        return f"X{k}" if k in small else f"n{k} * p{k}"

    def abs_t(k):
        return {"n": f"|t{k}|", "p": f"t{k}", "m": f"-t{k}"}[states[k]]

    src = [
        "# restates: crates/rcad-geom/src/plane.rs on_floats at 6a4a814c",
        "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c",
        "# SideOfAFlatSurface::on_floats without its three product_is_normal checks:",
        "# the sign of n . p - off. Products: "
        + ", ".join(f"n{k} * p{k} {dict(n='normal', p='below the normals and positive', m='below the normals and negative', z='zero')[states[k]]}"
                    for k in range(3)) + ".",
    ]
    if cancel:
        src.append(f"# The sum{'s' if len(cancel) > 1 else ''} adding product{'s' if len(cancel) > 1 else ''} "
                   f"{', '.join(map(str, cancel))} cancel{'' if len(cancel) > 1 else 's'} to zero, which rounds to itself.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The normal, the point and the offset, each an f64. A zero product has",
        "# the double 0 for its point's coordinate; with the normal's zero instead,",
        "# every number below is the same.",
        "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);",
        " ".join(f"p{k} = 0;" if states[k] == "z" else f"p{k} = rnd(p{k}_);" for k in range(3)),
        "off = rnd(off_);",
        "",
        "# What on_floats computes, line for line. Adding a zero product to a double",
        "# gives that double back (464.13's addzero.g), so zero products are left",
        "# out of both folds.",
    ]
    if small:
        src.append(" ".join(f"X{k} = n{k} * p{k};" for k in small))
    if live:
        src.append(" ".join(f"t{k} = rnd({x(k)});" for k in live))
    rounded = []
    if live:
        running = f"t{live[0]}"
        for k in live[1:]:
            src.append(f"S{k} = {running} + t{k};")
            if k in cancel:
                src.append(f"s{k} = S{k};")
            else:
                src.append(f"s{k} = rnd(S{k});")
                rounded.append(k)
            running = f"s{k}"
        src.append(f"y = {running} - off;")
    else:
        src.append("y = -off;")
    src.append("value = rnd(y);")
    src += ["", "# The permanent folds the same products' magnitudes in the same order."]
    if live:
        perm = abs_t(live[0])
        for k in live[1:]:
            perm = f"rnd({perm} + {abs_t(k)})"
        src.append(f"P = rnd({perm} + |off|);")
        src.append("Phat = " + " + ".join(abs_t(k) for k in live) + " + |off|;")
    else:
        src.append("P = |off|;")
    src += ["", b, "eb = rnd(B * P);", "", "# The true value of the same inputs."]
    src.append("Ex = " + " + ".join(x(k) if k in small else f"n{k} * p{k}" for k in range(3)) + " - off;")

    carry = {}
    for i, k in enumerate(live):
        enters = live[max(i, 1):]
        carry[k] = [f"rs{j}" for j in rounded if j in enters]

    def r(k):
        return " * ".join(carry[k]) if carry[k] else "1"

    hint_lines = []
    goal = []
    if normal:
        src += ["", "# The products' part of the permanent, with no rounding of its own, and",
                "# the same with X in place of what each product below the normals rounds to."]
        src.append("P3 = " + " + ".join(abs_t(k) for k in live) + ";")
        cs = []
        for k in live:
            cs.append({"n": f"c{k} = |t{k}|;", "p": f"c{k} = X{k};", "m": f"c{k} = -X{k};"}[states[k]])
        src.append(" ".join(cs))
        src.append("P3x = " + " + ".join(f"c{k}" for k in live) + ";")
        src += ["", "# What each rounding multiplied its argument by."]
        src.append(" ".join(f"rp{k} = t{k} / (n{k} * p{k});" for k in normal))
        if rounded:
            src.append(" ".join(f"rs{j} = s{j} / S{j};" for j in rounded))
        src += ["", "# Each term as y carries it less the term: e as a fraction of the term,",
                "# with X for each product below the normals, and d what rounding X cost."]
        for k in live:
            src.append(f"W{k} = {r(k)} - 1 / rp{k};" if k in normal else f"W{k} = {r(k)} - 1;")
        src.append(" ".join(f"e{k} = t{k} * W{k};" if k in normal else f"e{k} = X{k} * W{k};" for k in live))
        if small:
            src.append(" ".join(f"d{k} = (t{k} - X{k}) * ({r(k)});" for k in small))
        esum = " + ".join(f"e{k}" for k in live)
        dsum = " + ".join(f"d{k}" for k in small)
        hint_lines.append("# How far each rounding is from the identity.")
        for k in normal:
            hint_lines.append(f"t{k} / (n{k} * p{k}) -> 1 + (t{k} - n{k} * p{k}) / (n{k} * p{k});")
        for j in rounded:
            hint_lines.append(f"s{j} / S{j} -> 1 + (s{j} - S{j}) / S{j};")
        hint_lines += [
            "",
            "# The relative part is averaged over P3x as in plane.g; what rounding each",
            "# X cost is an amount, and eb being normal keeps it small against eb.",
            f"(y - Ex) / eb -> (({esum}) / P3x) * (P3x / P) * (P / eb)" + (f" + ({dsum}) / eb;" if small else ";"),
            f"({esum}) / P3x -> ((P3x + ({esum})) - P3x) / P3x;",
            f"P3x + ({esum}) -> " + " + ".join(f"(c{k} + e{k})" for k in live) + ";",
        ]
        for k in live:
            hint_lines.append(f"(c{k} + e{k}) - c{k} -> e{k};")
        for k in live:
            hint_lines.append({"n": f"e{k} / c{k} -> (t{k} / |t{k}|) * W{k};",
                               "p": f"e{k} / c{k} -> W{k};", "m": f"e{k} / c{k} -> -W{k};"}[states[k]])
        moved = "".join(f" + (X{k} - t{k}) / P" if states[k] == "p" else f" + (t{k} - X{k}) / P" for k in small)
        hint_lines += [
            f"P3x / P -> (P3 / Phat) * (Phat / P){moved};",
            "P3 / Phat -> 1 / (1 + |off| / P3);",
            "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
            "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        ]
        if small:
            hint_lines.append("P -> eb / (B * (1 + (eb - B * P) / (B * P)));")
        goal.append(f"({esum}) / P3x in ?")
        if small:
            goal += [f"{dsum} in ?", "P in ?"]
        else:
            goal.append("P -/ Phat in ?")
    elif live:
        parts = [f"(t{k} - X{k})" for k in small] + [f"(s{j} - S{j})" for j in rounded]
        src += ["", "# No product is normal, so every error is an amount: what rounding each",
                "# X cost and what rounding each sum cost."]
        hint_lines += [
            "# y less the truth is what each rounding cost, as an amount.",
            "y - Ex -> " + " + ".join(parts) + ";",
        ]
        goal.append("y - Ex in ? /\\ P in ?")
    else:
        hint_lines += ["# Every product is zero, so y is the truth.", "y - Ex -> 0;"]
        goal.append("y - Ex in ?")

    hyps = [("Every input is a finite f64.",
             [[f"|n{k}| <= 1b1024" for k in range(3)],
              [f"|p{k}| <= 1b1024" for k in range(3) if states[k] != "z"] + ["|off| <= 1b1024"]])]
    if normal:
        hyps.append(("No factor of a normal product is zero.",
                     [[f"|n{k}| >= 1b-1074" for k in normal] + [f"|p{k}| >= 1b-1074" for k in normal]]))
    if rounded:
        hyps.append(("No rounded sum cancels to zero.", [[f"|S{j}| >= 1b-1074" for j in rounded]]))
    if small:
        hyps.append(("The products below the normals. A product of two doubles that are not\n"
                     "  # zero is at least 2^-2148.",
                     [[f"X{k} in [1b-2148, 1b-1022]" if states[k] == "p" else f"X{k} in [-1b-1022, -1b-2148]"
                       for k in small]]))
    if normal:
        hyps.append(("The normal products, as product_is_normal has them.",
                     [[f"|t{k}| >= 1b-1022" for k in normal]]))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = [[f"t{k} -/ n{k} * p{k} {LEMMA}"] for k in normal]
    if eb_normal and normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if normal:
        hyps.append(("x / |x| is in [-1, 1] for any x that is not zero.",
                     [[f"t{k} / |t{k}| in [-1, 1]" for k in normal]]))

    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
            first = False
    src.append("->")
    src.append("  " + " /\\ ".join(goal))
    if not tight:
        src.append("  /\\ (y - Ex) / eb in [-1, 1]")
    else:
        src.append(TIGHT if live else TIGHT_ZZZ)
    src += ["}", ""]
    src += hint_lines
    return "\n".join(src) + "\n"


def make_value_off0(b=B_CODE):
    """plane.g's claim taken on value, with the offset the double 0. Then y is
    s012 less zero, which is s012 (addzero.g), so value is y."""
    text = make("nnn", b=b)
    text = text.replace("# the sign of n . p - off. Products: n0 * p0 normal, n1 * p1 normal, n2 * p2 normal.",
                        "# the sign of n . p - off. Products: n0 * p0 normal, n1 * p1 normal, n2 * p2 normal.\n"
                        "# The offset is zero, and the claim is on value, the double on_floats returns.")
    for old, new in [
        ("off = rnd(off_);", "off = 0;"),
        ("y = s2 - off;\nvalue = rnd(y);", "y = s2 - off;\nvalue = rnd(y);"),
        ("  /\\ (y - Ex) / eb in [-1, 1]", "  /\\ value - y in ? /\\ (value - Ex) / eb in [-1, 1]"),
        ("P = rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |off|);", "P = rnd(rnd(rnd(|t0| + |t1|) + |t2|) + |off|);"),
    ]:
        assert text.count(old) == 1, old
        text = text.replace(old, new)
    text = text.replace("     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024\n  /\\ |p0| <= 1b1024 /\\ |p1| <= 1b1024 /\\ |p2| <= 1b1024 /\\ |off| <= 1b1024",
                        "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024\n  /\\ |p0| <= 1b1024 /\\ |p1| <= 1b1024 /\\ |p2| <= 1b1024")
    old = "(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb);"
    assert text.count(old) == 1
    text = text.replace(old, "# value less zero is value, and so is its rounding.\n"
                        "y -> s2;\n"
                        "(value - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + (value - y) / eb;")
    return text


def name_of(states, cancel):
    return f"plane_{states}" + (f"_c{''.join(map(str, cancel))}" if cancel else "")


if __name__ == "__main__":
    proving, half, noeb = [], [], []
    for states in ("".join(s) for s in itertools.product(STATES, repeat=3)):
        live = [k for k in range(3) if states[k] != "z"]
        sums = live[1:]
        for n in range(len(sums) + 1):
            for cancel in itertools.combinations(sums, n):
                name = name_of(states, cancel)
                open(f"{name}.g", "w").write(make(states, cancel))
                proving.append(name)
                if "n" in states:
                    open(f"c_{name}_half.g", "w").write(make(states, cancel, b=B_HALF))
                    half.append(name)
                else:
                    open(f"c_{name}_noeb.g", "w").write(make(states, cancel, eb_normal=False))
                    open(f"c_{name}_tight.g", "w").write(make(states, cancel, tight=True))
                    noeb.append(name)
    open("planev_off0.g", "w").write(make_value_off0())
    open("c_planev_off0_half.g", "w").write(make_value_off0(b=B_HALF))
    print(f"{len(proving)} cases, {len(half)} controls at u / 2, {len(noeb)} controls without eb normal, "
          f"{len(noeb)} with the goal too tight, and planev_off0.g")
```

`python3 planeall.py` prints `172 cases, 109 controls at u / 2, 63 controls
without eb normal, 63 with the goal too tight, and planev_off0.g`.

`summary.py`, which printed the figures of 4 from the `.out` files:

```python
"""Read every plane_*.out beside this file and print, one line per case, what
Gappa printed for each goal: a relative goal as a u and u^2 count, an amount
as a multiple of 2^-1075, and P's lower end as a power of two."""
import math
import os
import re
import sys
from fractions import Fraction

os.chdir(os.path.dirname(os.path.abspath(__file__)))
U = Fraction(1, 2**53)
NUM = re.compile(r"(-?\d+)(?:b(-?\d+))?")


def value(text):
    m = NUM.fullmatch(text)
    mant, exp = int(m.group(1)), int(m.group(2) or 0)
    return Fraction(mant) * (Fraction(2) ** exp)


def interval(line):
    m = re.search(r" in \[(-?\d+(?:b-?\d+)?)(?: \{[^}]*\})?, (-?\d+(?:b-?\d+)?)(?: \{[^}]*\})?\]", line)
    return value(m.group(1)), value(m.group(2))


def as_u(x):
    k = math.floor(x / U)
    rest = (x - k * U) / (U * U)
    return f"{k}u+{float(rest):.4g}u2"


def read(name):
    text = open(f"{name}.out").read()
    out = {}
    if "Results:" not in text:
        return {"ERR": "no results"}
    for line in text.split("Results:", 1)[1].splitlines():
        line = line.strip()
        if " in " not in line:
            continue
        goal = line.split(" in ")[0]
        lo, hi = interval(line)
        if goal == "P":
            out["P>="] = f"2^{math.log2(lo):.3f}" if lo > 0 else str(lo)
        elif goal.startswith("y - Ex"):
            out["y-Ex"] = f"{max(-lo, hi) * 2**1075}*2^-1075"
        elif "/" in goal:
            out[goal] = as_u(max(-lo, hi))
        else:
            out[goal] = f"{max(-lo, hi) * 2**1075}*2^-1075"
    if "pointless" in text:
        out["POINTLESS"] = "yes"
    return out


names = sys.argv[1:] or sorted(f[:-4] for f in os.listdir(".") if f.startswith("plane_") and f.endswith(".out"))
for name in names:
    r = read(name)
    print(name, "  ".join(f"{k}: {v}" for k, v in r.items()))
```

`controls.py`, which printed `_half controls whose best is twice their case's
u count: 109`, `_noeb controls: 63, bests: ['inf']` and `_tight controls whose
best is their case's y - Ex over 2^-1022: 63`, with both lists of those that
are not empty:

```python
"""For each control beside this file, print its best and, for each _half
control, whether that best is twice the u count its case printed: a case that
proves at B = 4u + 2^-101 with e within k u of P3x fails at B = u / 2 with a
best of 2k. For each _tight control, whether its best is k 2^-53 for the
k 2^-1075 its case printed y - Ex within, eb being 2^-1022 at the least; and
for plane_zzz's, the largest double, since off is anything eb normal allows."""
import os
import re
import subprocess
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
summary = subprocess.run([sys.executable, "summary.py"], capture_output=True, text=True).stdout
ucount, ycount = {}, {}
for line in summary.splitlines():
    name = line.split()[0]
    m = re.search(r"(?:/ P3x|/ \(c[0-9 +c]*\)|/ \|t\d\|): (\d+)u", line)
    ucount[name] = int(m.group(1)) if m else None
    m = re.search(r"y-Ex: (\d+)\*2\^-1075", line)
    ycount[name] = int(m.group(1)) if m else None

agree, disagree, noeb, tight, tight_not = 0, [], [], 0, []
for f in sorted(os.listdir(".")):
    if not (f.startswith("c_plane_") and f.endswith(".out")):
        continue
    text = open(f).read()
    best = re.search(r"best: \[([^,]+), ([^\]]+)\]", text).group(2)
    name = f[2:-4]
    if name.endswith("_half"):
        case = name[: -len("_half")]
        if ucount[case] is not None and float(best) == 2 * ucount[case]:
            agree += 1
        else:
            disagree.append((name, best, ucount[case]))
    elif name.endswith("_tight"):
        case = name[: -len("_tight")]
        want = 1.79769e308 if case == "plane_zzz" else ycount[case] * 2.0**-53
        if abs(float(best) - want) <= 1e-5 * want:
            tight += 1
        else:
            tight_not.append((name, best, ycount[case]))
    else:
        noeb.append((name, best))
print(f"_half controls whose best is twice their case's u count: {agree}")
print("_half controls that are not:", disagree)
print(f"_noeb controls: {len(noeb)}, bests: {sorted(set(b for _, b in noeb))}")
print(f"_tight controls whose best is their case's y - Ex over 2^-1022: {tight}")
print("_tight controls that are not:", tight_not)
```

`o2zero.py`, which writes the five cases of 5 and their controls, whole:

```python
"""Write Orient2d::on_floats with a difference zero, the branch 464.8 left on
paper. A difference of two doubles is zero only when the two are equal, and is
then exactly zero, and so is its product; so a zero difference makes forwards
or backwards the double 0.

- o2zf.g: forwards zero and acy * bcx at least 2^-1000, which must prove.
- o2zf_tiny.g: forwards zero and acy * bcx at most 2^-1000, where eb cannot be
  normal, which must print that the rest is pointless.
- o2zb.g and o2zb_tiny.g: the same with backwards zero.
- o2zfb.g: both zero, which must print pointless too.
- c_o2zf_half.g and c_o2zb_half.g: the two that prove again at B = u / 2,
  which must fail; c_<name>_noeb.g: the three pointless ones again without eb
  normal, which must fail.

Run as: python3 o2zero.py; it writes beside itself.
"""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

B_CODE = "B = 3b-53 + 1b-102;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HEAD = """# restates: crates/rcad-predicates/src/orient2d.rs on_floats at 6a4a814c
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
"""

# Each one-zero case, by which product is zero: the differences the other
# product is made of, its name in on_floats, and its sign in y.
ONE = {
    "f": dict(zero="forwards", why="a and c share their x, so acx is zero; with b and c sharing\n# their y instead, bcy is zero",
              ins=("ay", "bx", "cx", "cy"), d1=("Acy", "ay - cy", "acy"), d2=("Bcx", "bx - cx", "bcx"),
              live="backwards", y="y = -backwards;", ex="Ex = -(Acy * Bcx);",
              y_why="0 less backwards is -backwards", e="e = -(backwards * W);",
              e_hint="e / P -> -((backwards / |backwards|) * W);"),
    "b": dict(zero="backwards", why="a and c share their y, so acy is zero; with b and c sharing\n# their x instead, bcx is zero",
              ins=("ax", "by", "cx", "cy"), d1=("Acx", "ax - cx", "acx"), d2=("Bcy", "by - cy", "bcy"),
              live="forwards", y="y = forwards;", ex="Ex = Acx * Bcy;",
              y_why="forwards less 0 is forwards", e="e = forwards * W;",
              e_hint="e / P -> (forwards / |forwards|) * W;"),
}


def one_zero(which, tiny=False, b=B_CODE, eb_normal=True):
    c = ONE[which]
    (D1, d1_def, d1), (D2, d2_def, d2) = c["d1"], c["d2"]
    live, zero = c["live"], c["zero"]
    lines = [
        HEAD.rstrip("\n"),
        f"# Orient2d::on_floats where {c['why']}, and every number below is the",
        f"# same. Either way {zero} is the double 0. {d1} * {d2} is at "
        + ("most" if tiny else "least") + " 2^-1000.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The four coordinates left, each an f64.",
        " ".join(f"{v} = rnd({v}_);" for v in c["ins"]),
        "",
        "# What on_floats computes, line for line. The zero difference is exactly",
        f"# 0, and so is {zero}; {c['y_why']}, and |forwards| plus",
        f"# |backwards| is |{live}| (464.13's addzero.g).",
        f"{D1} = {d1_def}; {D2} = {d2_def};",
        f"{d1} = rnd({D1}); {d2} = rnd({D2});",
        f"G = {d1} * {d2};",
        f"{live} = rnd(G);",
        c["y"],
        "value = rnd(y);",
        f"P = |{live}|;",
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "# The true determinant of the same inputs, the zero difference being zero.",
        c["ex"],
        "",
        "# What each rounding multiplied its argument by.",
        f"r1 = {d1} / {D1}; r2 = {d2} / {D2}; rp = {live} / G;",
        "",
        f"# {live} less the product of the exact differences, as a fraction of it.",
        "W = 1 - 1 / (r1 * r2 * rp);",
        c["e"],
        "",
        "{",
        "  # Every coordinate is a finite f64.",
        "     " + " /\\ ".join(f"|{v}| <= 1b1024" for v in c["ins"]),
        "  # The other two differences are not zero.",
        f"  /\\ |{d1}| >= 1b-1074 /\\ |{d2}| >= 1b-1074",
    ]
    if tiny:
        lines += [f"  # The case: {d1} * {d2} is at most 2^-1000.", "  /\\ |G| <= 1b-1000"]
    else:
        lines += [f"  # The case: {d1} * {d2} is far above the normals.", "  /\\ |G| >= 1b-1000"]
    if eb_normal:
        lines += ["  # The bound is normal, which decide checks.", "  /\\ |eb| >= 1b-1022"]
        if not tiny:
            lines += ["  # What lemma.g proves that gives.", f"  /\\ eb -/ B * P {LEMMA}"]
    if not tiny:
        lines += ["  # x / |x| is in [-1, 1] for any x that is not zero.", f"  /\\ {live} / |{live}| in [-1, 1]"]
    lines += [
        "->",
        "  eb in ?" if tiny else "  e / P in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
        "# How far each rounding is from the identity.",
        f"{d1} / {D1} -> 1 + ({d1} - {D1}) / {D1};",
        f"{d2} / {D2} -> 1 + ({d2} - {D2}) / {D2};",
        f"{live} / G -> 1 + ({live} - G) / G;",
        "",
        f"# {zero} is zero, so y less the truth is {live}'s error alone.",
        "(y - Ex) / eb -> (e / P) * (P / eb);",
        c["e_hint"],
        "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
    ]
    return "\n".join(lines) + "\n"


def both_zero(eb_normal=True):
    lines = [
        HEAD.rstrip("\n"),
        "# Orient2d::on_floats where both products are the double 0: one of acx and",
        "# bcy is zero, and one of acy and bcx. Then forwards less backwards is 0,",
        "# and so are the permanent and the true determinant, a zero difference",
        "# being exactly zero.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "y = 0;",
        "value = rnd(y);",
        "P = 0;",
        "",
        B_CODE,
        "eb = rnd(B * P);",
        "",
        "Ex = 0;",
        "",
        "{",
    ]
    if eb_normal:
        lines += ["  # The bound is normal, which decide checks.", "     |eb| >= 1b-1022"]
    else:
        lines += ["  # Nothing: without eb normal there is no hypothesis left.", "     P in [0, 0]"]
    lines += ["->", "  eb in ?", "  /\\ (y - Ex) / eb in [-1, 1]", "}"]
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    for which in ONE:
        open(f"o2z{which}.g", "w").write(one_zero(which))
        open(f"c_o2z{which}_half.g", "w").write(one_zero(which, b=B_HALF))
        open(f"o2z{which}_tiny.g", "w").write(one_zero(which, tiny=True))
        open(f"c_o2z{which}_tiny_noeb.g", "w").write(one_zero(which, tiny=True, eb_normal=False))
    open("o2zfb.g", "w").write(both_zero())
    open("c_o2zfb_noeb.g", "w").write(both_zero(eb_normal=False))
    print("5 cases and 5 controls")
```

`runmany.sh`, which runs a list of scripts through `one.sh` eight at a time:

```sh
#!/bin/bash
# Run every .g named in $1 through one.sh, 8 at a time, appending to $1.log.
cd "$(dirname "$0")"
rm -f "$1.log"
xargs -P 8 -I{} bash one.sh {} "$2" < "$1" >> "$1.log" 2>&1
echo "done $(wc -l < "$1") files from $1"
```

`allplane.sh`, which printed `cases took 45.4s`, `cases exit 0: 173 of 173`,
`cases coq ok: 173`, `cases pointless: 0`, `controls exit 1: 236 of 236`,
`controls pointless: 0` and the lines of 4 and 6:

```sh
#!/bin/bash
# Run every plane script planeall.py writes: the cases and planev_off0.g with
# coq, the controls without, then count what each printed.
cd "$(dirname "$0")"
python3 planeall.py
ls plane_*.g planev_off0.g > list_plane
ls c_plane*.g > list_c
start=$(date +%s.%N)
bash runmany.sh list_plane coq
end=$(date +%s.%N)
printf 'cases took %.1fs\n' "$(echo "$end - $start" | bc)"
bash runmany.sh list_c nocoq
echo "cases exit 0: $(grep -c ' exit 0 [0-9.]*s$' list_plane.log) of $(wc -l < list_plane)"
echo "cases coq ok: $(grep -c 'coq -Bcoq exit 0 coqc exit 0' list_plane.log)"
echo "cases pointless: $(grep -l pointless plane_*.out planev_off0.out | wc -l)"
echo "controls exit 1: $(grep -c ' exit 1 ' list_c.log) of $(wc -l < list_c)"
echo "controls pointless: $(grep -l pointless c_plane*.out | wc -l)"
grep '^planev_off0 coq' list_plane.log
grep -h 'value - y in' planev_off0.out
grep -h 'best' c_planev_off0_half.out
python3 controls.py
python3 summary.py > summary.txt
echo "summary lines: $(wc -l < summary.txt)"
```

`o2run.sh`, which printed the lines of 5 and 6 for the orient2d scripts:

```sh
#!/bin/bash
# Write and run the orient2d zero-difference scripts and their controls.
cd "$(dirname "$0")"
python3 o2zero.py
ls o2z*.g > list_o2z
ls c_o2z*.g > list_o2zc
bash runmany.sh list_o2z coq
bash runmany.sh list_o2zc nocoq
sort list_o2z.log list_o2zc.log
for f in o2z*.out c_o2z*.out; do
  printf '%s pointless: %s\n' "${f%.out}" "$(grep -c pointless "$f")"
  grep -E ' in \[|best' "$f"
done
```

`plane_ppn.g`, as `planeall.py` writes it: the first two products below the
normals and positive, the third normal, no sum cancelling. It prints `P` at
least `1125899906842623b-1021`, `(e0 + e1 + e2) / P3x` within `2u + 3u²` by
`summary.py`'s count, and `d0 + d1` within `9007199254740993b-1127`,
`2⁻¹⁰⁷⁴ + 2⁻¹¹²⁷`.

```gappa
# restates: crates/rcad-geom/src/plane.rs on_floats at 6a4a814c
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# SideOfAFlatSurface::on_floats without its three product_is_normal checks:
# the sign of n . p - off. Products: n0 * p0 below the normals and positive, n1 * p1 below the normals and positive, n2 * p2 normal.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64. A zero product has
# the double 0 for its point's coordinate; with the normal's zero instead,
# every number below is the same.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

# What on_floats computes, line for line. Adding a zero product to a double
# gives that double back (464.13's addzero.g), so zero products are left
# out of both folds.
X0 = n0 * p0; X1 = n1 * p1;
t0 = rnd(X0); t1 = rnd(X1); t2 = rnd(n2 * p2);
S1 = t0 + t1;
s1 = rnd(S1);
S2 = s1 + t2;
s2 = rnd(S2);
y = s2 - off;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(rnd(t0 + t1) + |t2|) + |off|);
Phat = t0 + t1 + |t2| + |off|;

B = 4b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = X0 + X1 + n2 * p2 - off;

# The products' part of the permanent, with no rounding of its own, and
# the same with X in place of what each product below the normals rounds to.
P3 = t0 + t1 + |t2|;
c0 = X0; c1 = X1; c2 = |t2|;
P3x = c0 + c1 + c2;

# What each rounding multiplied its argument by.
rp2 = t2 / (n2 * p2);
rs1 = s1 / S1; rs2 = s2 / S2;

# Each term as y carries it less the term: e as a fraction of the term,
# with X for each product below the normals, and d what rounding X cost.
W0 = rs1 * rs2 - 1;
W1 = rs1 * rs2 - 1;
W2 = rs2 - 1 / rp2;
e0 = X0 * W0; e1 = X1 * W1; e2 = t2 * W2;
d0 = (t0 - X0) * (rs1 * rs2); d1 = (t1 - X1) * (rs1 * rs2);

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No factor of a normal product is zero.
  /\ |n2| >= 1b-1074 /\ |p2| >= 1b-1074
  # No rounded sum cancels to zero.
  /\ |S1| >= 1b-1074 /\ |S2| >= 1b-1074
  # The products below the normals. A product of two doubles that are not
  # zero is at least 2^-2148.
  /\ X0 in [1b-2148, 1b-1022] /\ X1 in [1b-2148, 1b-1022]
  # The normal products, as product_is_normal has them.
  /\ |t2| >= 1b-1022
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ t2 -/ n2 * p2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ t2 / |t2| in [-1, 1]
->
  (e0 + e1 + e2) / P3x in ? /\ d0 + d1 in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
t2 / (n2 * p2) -> 1 + (t2 - n2 * p2) / (n2 * p2);
s1 / S1 -> 1 + (s1 - S1) / S1;
s2 / S2 -> 1 + (s2 - S2) / S2;

# The relative part is averaged over P3x as in plane.g; what rounding each
# X cost is an amount, and eb being normal keeps it small against eb.
(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + (d0 + d1) / eb;
(e0 + e1 + e2) / P3x -> ((P3x + (e0 + e1 + e2)) - P3x) / P3x;
P3x + (e0 + e1 + e2) -> (c0 + e0) + (c1 + e1) + (c2 + e2);
(c0 + e0) - c0 -> e0;
(c1 + e1) - c1 -> e1;
(c2 + e2) - c2 -> e2;
e0 / c0 -> W0;
e1 / c1 -> W1;
e2 / c2 -> (t2 / |t2|) * W2;
P3x / P -> (P3 / Phat) * (Phat / P) + (X0 - t0) / P + (X1 - t1) / P;
P3 / Phat -> 1 / (1 + |off| / P3);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
P -> eb / (B * (1 + (eb - B * P) / (B * P)));
```

`plane_ppp.g`, as `planeall.py` writes it: all three products below the
normals and positive, no sum cancelling. It prints `y − Ex` within
`5b-1075`, one `2⁻¹⁰⁷⁵` for each of its five roundings.

```gappa
# restates: crates/rcad-geom/src/plane.rs on_floats at 6a4a814c
# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# SideOfAFlatSurface::on_floats without its three product_is_normal checks:
# the sign of n . p - off. Products: n0 * p0 below the normals and positive, n1 * p1 below the normals and positive, n2 * p2 below the normals and positive.
@rnd = float<ieee_64, ne>;

# The normal, the point and the offset, each an f64. A zero product has
# the double 0 for its point's coordinate; with the normal's zero instead,
# every number below is the same.
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
off = rnd(off_);

# What on_floats computes, line for line. Adding a zero product to a double
# gives that double back (464.13's addzero.g), so zero products are left
# out of both folds.
X0 = n0 * p0; X1 = n1 * p1; X2 = n2 * p2;
t0 = rnd(X0); t1 = rnd(X1); t2 = rnd(X2);
S1 = t0 + t1;
s1 = rnd(S1);
S2 = s1 + t2;
s2 = rnd(S2);
y = s2 - off;
value = rnd(y);

# The permanent folds the same products' magnitudes in the same order.
P = rnd(rnd(rnd(t0 + t1) + t2) + |off|);
Phat = t0 + t1 + t2 + |off|;

B = 4b-53 + 1b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Ex = X0 + X1 + X2 - off;

# No product is normal, so every error is an amount: what rounding each
# X cost and what rounding each sum cost.

{
  # Every input is a finite f64.
     |n0| <= 1b1024 /\ |n1| <= 1b1024 /\ |n2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |off| <= 1b1024
  # No rounded sum cancels to zero.
  /\ |S1| >= 1b-1074 /\ |S2| >= 1b-1074
  # The products below the normals. A product of two doubles that are not
  # zero is at least 2^-2148.
  /\ X0 in [1b-2148, 1b-1022] /\ X1 in [1b-2148, 1b-1022] /\ X2 in [1b-2148, 1b-1022]
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
->
  y - Ex in ? /\ P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# y less the truth is what each rounding cost, as an amount.
y - Ex -> (t0 - X0) + (t1 - X1) + (t2 - X2) + (s1 - S1) + (s2 - S2);
```

`o2zf.g`, as `o2zero.py` writes it: `forwards` zero and `acy · bcx` at least
`2⁻¹⁰⁰⁰` in magnitude. It prints `e / P` within `3.33067e-16`.

```gappa
# restates: crates/rcad-predicates/src/orient2d.rs on_floats at 6a4a814c
# restates: crates/rcad-predicates/src/orient2d.rs ORIENT2D_ERROR_BOUND at 6a4a814c
# restates: crates/rcad-predicates/src/cascade.rs decide at 6a4a814c
# Orient2d::on_floats where a and c share their x, so acx is zero; with b and c sharing
# their y instead, bcy is zero, and every number below is the
# same. Either way forwards is the double 0. acy * bcx is at least 2^-1000.
@rnd = float<ieee_64, ne>;

# The four coordinates left, each an f64.
ay = rnd(ay_); bx = rnd(bx_); cx = rnd(cx_); cy = rnd(cy_);

# What on_floats computes, line for line. The zero difference is exactly
# 0, and so is forwards; 0 less backwards is -backwards, and |forwards| plus
# |backwards| is |backwards| (464.13's addzero.g).
Acy = ay - cy; Bcx = bx - cx;
acy = rnd(Acy); bcx = rnd(Bcx);
G = acy * bcx;
backwards = rnd(G);
y = -backwards;
value = rnd(y);
P = |backwards|;

B = 3b-53 + 1b-102;
eb = rnd(B * P);

# The true determinant of the same inputs, the zero difference being zero.
Ex = -(Acy * Bcx);

# What each rounding multiplied its argument by.
r1 = acy / Acy; r2 = bcx / Bcx; rp = backwards / G;

# backwards less the product of the exact differences, as a fraction of it.
W = 1 - 1 / (r1 * r2 * rp);
e = -(backwards * W);

{
  # Every coordinate is a finite f64.
     |ay| <= 1b1024 /\ |bx| <= 1b1024 /\ |cx| <= 1b1024 /\ |cy| <= 1b1024
  # The other two differences are not zero.
  /\ |acy| >= 1b-1074 /\ |bcx| >= 1b-1074
  # The case: acy * bcx is far above the normals.
  /\ |G| >= 1b-1000
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves that gives.
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ backwards / |backwards| in [-1, 1]
->
  e / P in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
acy / Acy -> 1 + (acy - Acy) / Acy;
bcx / Bcx -> 1 + (bcx - Bcx) / Bcx;
backwards / G -> 1 + (backwards - G) / G;

# forwards is zero, so y less the truth is backwards's error alone.
(y - Ex) / eb -> (e / P) * (P / eb);
e / P -> -((backwards / |backwards|) * W);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
```

**10. What this leaves.** One row, filed from this write-up when it merges:

- A row to restate the last paragraph of `ORIENT2D_ERROR_BOUND`'s doc
  (`orient2d.rs:359-360` at `6a4a814c`), which says the bound "still holds
  only while nothing overflows or falls into the subnormals". 464.8's 2
  proves it with one or both products below the normals, and 5 here with a
  difference zero, and 7 found the same words false in `plane.rs`. The
  paragraph above it in the same doc already says so; the last one reads
  against it.

And what is not a row:

- Q2760 is the same work for `SideOfAFlatSurfaceThroughAPoint`'s six
  products, and is open.
- Q2767, on the doc of `product_is_normal` in `cascade.rs`, already waits on
  Q2684. 7 here is a second predicate, after 464.14's sphere, where that doc's
  claim does not hold: without its checks, a product below the normals would
  not make this filter certify a sign it has not earned.
- Q2700 already waits on Q2684, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- Whether dropping the three checks would turn a refusal at level 1 into an
  answer was not run (7). The checks stay, so nothing asks for it (§13).
- `ORIENT2D_ERROR_BOUND`'s doc says Gappa proves the bound "in every case of
  where the two products can land" and cites 464.8 alone. With 5 that
  sentence holds as it stands.
- `field` on these scripts' hints (464.10).

### 464.19 Isometry::POINT_ROUNDOFF

Q2689 proved `Isometry::POINT_ROUNDOFF` in Gappa, in four lanes on
2026-10-25 at `ba0a654e`. The first wrote `point.g` and `sos.py` and ran
`point.g`. The second wrote `doubling.g`, `controls.py`, `turn.py` and
`all.sh`, took the doubling's rounding out of `point.g`, and ran them all.
The third ran them again in a tree of its own and checked each place this
cites in the code. The fourth wrote `same.py`, ran everything a third time,
and wrote this.

Each coordinate of a moved point is proved within the code's `18ε · L` where
no factor is zero, no sum cancels to zero, the nine products of the
arithmetic are normal numbers, and the turn's `q·q` is within `6ε` of one. The
proof reaches `24.80u · L` where the doc counts `35.78u · L` (3, 4). A turn of
a cube is one addition per coordinate, which `shiftball.g` already proved
(6). The zero branches and the products below the normals are not run (10),
and one public route to a movement does not hold its turn near length one
(5). Coq checks the proof, modulo its hints (8).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, unchanged, against `isometry.rs` at `ba0a654e` and
nalgebra 0.35.0, the version `Cargo.lock` holds. `u` is `2⁻⁵³` and `ε` is
`f64::EPSILON = 2u`.

**1. What the proof guards.** `Isometry::apply_to_point`
(`isometry.rs:303-308`) promises each coordinate of a moved point within
`POINT_ROUNDOFF · L` of where the movement's own numbers, worked without
rounding, put it, `L` the largest coordinate of the point and of the shift
(`isometry.rs:37-92`). `rcad-topo`'s mesher charges a placed face that much
(`how_far_placing_this_face_rounds`, `mesh.rs:10360-10364`), so
`Mesh::deviation` rests on it, and tests in `rcad-geom` and `rcad-topo`
measure against it. Until now the 18 rested on the doc's own count (4) and
on `a_moved_point_lands_inside_the_published_roundoff_of_the_exact_answer`
(`isometry.rs:1002-1035`), which samples it: its worst of 3072 coordinates
uses an eighth of the bound, and a sweep a thousand times longer a quarter
(its doc, `isometry.rs:984-1001`).

**2. `apply_to_point`, line for line.** `apply_to_point` adds the shift to
`turned_by(rotation, p)` (`isometry.rs:515-520`). For a turn that is not one
of a cube's 24, `turned_by` calls nalgebra's `transform_vector`
(`src/geometry/quaternion.rs:1559`), which is `self * v`, the `UnitQuaternion
× Vector3` of `src/geometry/quaternion_ops.rs:376-389`. Its body, 383-387, is
`t = v.cross(p) * 2`, then `v.cross(&t)`, then `t * w + cross + p`, and
`cross` writes component `i` as `v[i+1] · p[i+2] − v[i+2] · p[i+1]`, indices
mod 3 (`src/base/matrix.rs:2101-2106`). So the moved point's `x` is, with
`rnd` each rounding:

- `c₀ = rnd(rnd(v₁p₂) − rnd(v₂p₁))`, and `c₁` and `c₂` the same way turned;
- `t = c · 2`, which rounds nothing: `doubling.g` proves `rnd(c · 2) − c · 2`
  in `[0, 0]` for every `f64` `c` with `|c| ≤ 2¹⁰²³`, so `point.g` writes `t`
  unrounded;
- `k₀ = rnd(rnd(v₁t₂) − rnd(v₂t₁))`;
- `x = rnd(rnd(rnd(rnd(t₀w) + k₀) + p₀) + s₀)`: the product and the two
  additions of `t * w + k + p`, left to right, then the shift's.

That is nine products, four differences and three additions, sixteen
roundings, and `point.g` writes each with its own `rnd` and its own hint.
`Ex` is the same arithmetic with none. `y` and `z` are the same lines with
every index turned `0 → 1 → 2 → 0`, once and twice, since `cross` turns its
indices that way and the last line works each coordinate on its own; `turn.py`
writes them as `point1.g` and `point2.g`.

**3. The bound.** `out − Ex` is the sum over the eight terms of `Ex` of each
term times the product of the roundings it passes through, less one: `W1` to
`W8`. The two terms through `w`, `2w·v₁p₂` and `2w·v₂p₁`, pass six
roundings; the four through `k`, such as `2v₁·v₀p₁`, seven; `p₀` two; `s₀`
one. Each term gets a share of the bound, its size with `L` for the point's
coordinate times the roundings it meets, as `signed.g` shares its terms
(464.9's 2): `12|w||v_a|L` for each through `w`, `14|v_a||v_c|L` for each
through `k`, and `3L`, `2L + L`, for `p₀` and `s₀` together, for the reason
`point.g`'s comment gives. Each term's
error is within `u` of its share and a little more, so the sum is within
that of the sum of the shares,

`D = L · (12|w|s + 14|v₀|s + 14(|v₁|² + |v₂|²) + 3)`, with `s = |v₁| + |v₂|`.

What is left is the largest `D / L`. Its quadratic part, in `(|w|, |v₀|,
|v₁|, |v₂|)`, has the matrix `[[0, 0, 6, 6], [0, 0, 7, 7], [6, 7, 14, 0], [6,
7, 0, 14]]`, whose largest eigenvalue is `7 + √219 = 21.7986`, a root of `λ²
− 14λ − 170`. `109/5 = 21.8` is just above it, so the quadratic is `109/5 ·
q·q` less four squares with positive weights. The hint on `D / L` writes it
that way, and Gappa bounds each square below by zero, so `D / L ≤ 109/5 · q·q
+ 3`. `sos.py` (9) checks that identity at 1000 random points in exact
rationals, and finds the largest `D / L` on a 200-step grid of unit
quaternions to be `24.798`, against `10 + √219 = 24.7986`.

`point.g` prints, and `point1.g` and `point2.g` print the same:

- `W1` within `±6.66134e-16`, `6u` and a little more, and `W3` within
  `±7.77156e-16`, `7u`;
- `W7` within `±2.22045e-16`, `2u`, and `W8` within `±1.11022e-16`, `u`;
- `D / L` in `[3, 24.8]`;
- `(out − Ex) / D` within `±1.11022e-16`, `u` and a little more.

So `(out − Ex) / L` is within `24.8u`, and the goal, the code's `36u`,
proves. Each of the three prints 163 warnings that a divisor has been
assumed to be nonzero, and no hint is reported as not an identity.

**4. What the doc's `35.78u` counts.** The doc (`isometry.rs:42-79`) charges
each stage at its own worst case: `|c_i| ≤ √2·L`, `|t_i·w| ≤ 2√2·L` and
`|k_i| ≤ 4L`, each from `|v_a| + |v_c| ≤ √2` or `|w| ≤ 1` on its own. It adds
the stages to `(8√2 + 21 + 2√3)·u·L ≈ 35.78u·L`, and keeps the `0.6%` up to
`36u` for the second-order terms and for the turn being of length one only
to within rounding. Those worst cases cannot all happen at once: `|w| = 1`
forces `v = 0`, and so `k = 0`. 3 bounds the sum of the shares once, with
`q·q` near one, and gets `24.80u`, which leaves `11.2u` of room where the doc
left `0.22u`. Gappa does not check the doc's count step by step; it proves
the constant the count was for.

**5. The hypotheses.** `point.g`'s, in its order:

- every input a finite `f64`;
- no factor of a product zero, and no sum cancelling to zero: the zero
  branches, which 10 leaves;
- the nine products normal, which `apply_to_point` does not check, and
  464.5's lemma for each. The doc says its count holds only while nothing
  falls into the subnormals, and so far so does this (10);
- `L` at least every `|p_i|` and `|s_i|`;
- `|q·q − 1| ≤ 6ε` worked out exactly, written `N4` in `[1 − 12u, 1 + 12u]`;
- `x / |x|` in `[−1, 1]` for `w` and each `v_i`, so that the hint on a
  term's share can bound its sign.

The `6ε` is what the record says of each route by which a turn reaches
`apply_to_point`, not anything the code checks there:

- `compose` holds its product with `held_near_length_one`
  (`isometry.rs:274`), whose doc (`isometry.rs:482-488`) says `|q·q − 1| ≤
  6ε`, worked out exactly, and
  `a_turn_composed_of_any_number_of_turns_stays_within_six_epsilon_of_length_one`
  samples it over 200 chains of 256 turns.
- `Isometry::rotation` (`isometry.rs:153-168`) keeps nalgebra's
  `from_axis_angle` as it comes. `TURN_LENGTH_DRIFT`'s doc
  (`isometry.rs:440-461`) measures that at most `2ε` off by the `f64` sum over
  100 000 draws, and `3ε` over the 100 000 of a probe, and the sum is within
  about `2ε` of the exact `q·q`. Measured, not proved.
- Reading a movement back (`TryFrom`, `isometry.rs:364-382`) divides the turn
  by its length (`from_quaternion`, at 379), about `4ε` off by
  `held_near_length_one`'s count.
- `inverse` (`isometry.rs:283-286`) turns by the conjugate, whose `q·q` is
  the same.
- Since Q2803, the public route is `TryFrom<na::Isometry3<f64>>`
  (`isometry.rs:401-414`), not `From`: it keeps a turn within
  `TURN_LENGTH_DRIFT` of one bit for bit, and divides any other by its length
  as stored parts are read (`isometry.rs:417-436`), refusing four zeros. The
  `6ε` hypothesis holds on it too.

**6. A turn of a cube.** When the turn is one of a cube's 24, `turned_by`
moves each coordinate onto another and negates some, rounding nothing
(`TurnOfACube::turned`, `isometry.rs:633-641`), and `apply_to_point` is one
addition per coordinate, `rnd(±p_j + s_i)`. That is `shiftball.g`'s shift of
a centre with `±p_j` for the centre (464.9's 3), which proves each coordinate
within `2u·L` with no normal-number condition, under the `36u`. Nothing new
was run for it.

**7. The controls.** A script whose hypotheses contradict each other proves
every goal (464.7's 3), so `controls.py` writes three that each change one
thing, and each exits 1:

- the goal at `24.75u` (`99b-55`), below the `24.80u` of 3: best
  `±2.75335e-15`, which is `24.80u`. So the hypotheses do not contradict each
  other. It does not show that any input reaches `24.80u`: that is how low
  this averaging goes, since `D / L` reaches `10 + √219` on unit quaternions
  (3), and the longest sweep of 1 found about a quarter of `36u`.
- without `|q·q − 1| ≤ 6ε`: best `±2.87033e+602`, with `D / L` up to
  `2.58536e+618`. The proof needs the turn near length one.
- without the nine products normal and the lemma they give: best
  `±1.06019e+633`, with `W1` and `W3` unproved and `(out − Ex) / D` within
  `±3.53398e+632`. The proof needs the products normal, or another argument
  for them (10).

**8. Coq checks the proof, modulo its hints.** `gappa -Bcoq` and `coqc`,
through `one.sh` against gappalib 1.8.0 and Flocq 4.2.2, exit 0 on `point.g`
and on `doubling.g`. Gappa does not write the same proof each run (464.4):
`point.v` came to 8257, 8158 and 8205 lines over the three runs, with 34
hints each time, and `doubling.v` to 149 lines with none. Each hint enters
the Coq file as a hypothesis with no condition attached, as in 464.4, so the
proof stands on its 34 hints and on the 163 divisors Gappa took to be
nonzero, which its warnings name. `field` has not been run on these hints
(464.10). `point1.g` and `point2.g` ran without `-Bcoq`; they are `point.g`
with its names turned.

**9. The scripts.** `point.g` is the proof and `doubling.g` the doubling;
`controls.py` writes the controls of 7 and `turn.py` `point1.g` and
`point2.g`; `sos.py` checks the identity of 3; `all.sh` runs them all through
464.13's `one.sh`; `same.py` checks this text against the files that ran. To
run them again, save these seven and `one.sh` in one directory and run `bash
all.sh` there, which takes about two minutes. The fourth lane ran `same.py`
on this text: each script here is byte for byte the file that ran, and
`controls.py` and `turn.py`, run again from the text, write the scripts that
ran.

`point.g`, whole, as it ran:

```gappa
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs turned_by at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs POINT_ROUNDOFF at ba0a654e
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at ba0a654e
# restates: Cargo.lock nalgebra at ba0a654e
# Isometry::apply_to_point and Isometry::POINT_ROUNDOFF,
# crates/rcad-geom/src/isometry.rs at ba0a654e, through nalgebra 0.35's
# UnitQuaternion * Vector3 (src/geometry/quaternion_ops.rs:377-388): the
# moved point's x is within 18 * f64::EPSILON * L of what the same arithmetic
# gives without rounding, L at least every |p_i| and |s_i|. y and z are this
# with the indices turned 0 -> 1 -> 2 -> 0, as turn.py writes them.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), the point and the shift's x, each an f64.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);
s0 = rnd(s0_);

# c = v x p, as nalgebra's cross writes each component: a*b - c*d.
P01 = v1 * p2; q01 = rnd(P01); P02 = v2 * p1; q02 = rnd(P02);
C0 = q01 - q02; c0 = rnd(C0);
P11 = v2 * p0; q11 = rnd(P11); P12 = v0 * p2; q12 = rnd(P12);
C1 = q11 - q12; c1 = rnd(C1);
P21 = v0 * p1; q21 = rnd(P21); P22 = v1 * p0; q22 = rnd(P22);
C2 = q21 - q22; c2 = rnd(C2);

# t = c * 2, written without its rounding: doubling.g proves rnd(c * 2) is
# c * 2 for every f64 c that does not overflow when doubled.
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;

# k = v x t, its x.
K1 = v1 * t2; k01 = rnd(K1); K2 = v2 * t1; k02 = rnd(K2);
KK = k01 - k02; k0 = rnd(KK);

# t * w + k + p, folded left to right, then + the shift.
M = t0 * w; m0 = rnd(M);
A = m0 + k0; a0 = rnd(A);
R = a0 + p0; r0 = rnd(R);
O = r0 + s0; out = rnd(O);

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * p2 - v2 * p1)
   + v1 * (2 * (v0 * p1 - v1 * p0)) - v2 * (2 * (v2 * p0 - v0 * p2))
   + p0 + s0;

# What each rounding multiplied its argument by.
rq01 = q01 / P01; rq02 = q02 / P02; rq11 = q11 / P11; rq12 = q12 / P12;
rq21 = q21 / P21; rq22 = q22 / P22;
rc0 = c0 / C0; rc1 = c1 / C1; rc2 = c2 / C2;
rk01 = k01 / K1; rk02 = k02 / K2; rk0 = k0 / KK;
rm = m0 / M; ra = a0 / A; rr = r0 / R; ro = out / O;

# The eight terms of Ex, each as out carries it, less the term itself: six
# roundings for the two through w, seven for the four through k, two for p0
# and one for s0.
W1 = ro * rr * ra * rm * rc0 * rq01 - 1;
W2 = ro * rr * ra * rm * rc0 * rq02 - 1;
W3 = ro * rr * ra * rk0 * rk01 * rc2 * rq21 - 1;
W4 = ro * rr * ra * rk0 * rk01 * rc2 * rq22 - 1;
W5 = ro * rr * ra * rk0 * rk02 * rc1 * rq11 - 1;
W6 = ro * rr * ra * rk0 * rk02 * rc1 * rq12 - 1;
W7 = ro * rr - 1;
W8 = ro - 1;
e1 = 2 * w * v1 * p2 * W1; e2 = -(2 * w * v2 * p1 * W2);
e3 = 2 * v1 * v0 * p1 * W3; e4 = -(2 * v1 * v1 * p0 * W4);
e5 = -(2 * v2 * v2 * p0 * W5); e6 = 2 * v2 * v0 * p2 * W6;
e7 = p0 * W7 + s0 * W8;

# Each term's share of the bound: its size with L for the point's coordinate,
# times the roundings it meets. p0 and s0 share one, 2L + L, since Gappa
# 1.8.0 aborts in ast.cpp printing a warning about a D that holds both.
d1 = 12 * |w| * |v1| * L; d2 = 12 * |w| * |v2| * L;
d3 = 14 * |v1| * |v0| * L; d4 = 14 * |v1| * |v1| * L;
d5 = 14 * |v2| * |v2| * L; d6 = 14 * |v2| * |v0| * L;
d7 = 3 * L;
D = d1 + d2 + d3 + d4 + d5 + d6 + d7;

# q.q, each square written as |x| * |x|, and the four squares D / L is
# written with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 30/109 * (|v1| + |v2|);
Y = |v0| - 35/109 * (|v1| + |v2|);
Z = |v1| - |v2|;

{
  # Every input is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  /\ |p0| <= 1b1024 /\ |p1| <= 1b1024 /\ |p2| <= 1b1024 /\ |s0| <= 1b1024
  # No factor of a product is zero.
  /\ |w| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |v1| >= 1b-1074 /\ |v2| >= 1b-1074
  /\ |p0| >= 1b-1074 /\ |p1| >= 1b-1074 /\ |p2| >= 1b-1074
  # No sum cancels to zero.
  /\ |C0| >= 1b-1074 /\ |C1| >= 1b-1074 /\ |C2| >= 1b-1074 /\ |KK| >= 1b-1074
  /\ |A| >= 1b-1074 /\ |R| >= 1b-1074 /\ |O| >= 1b-1074
  # The nine products are normal, which apply_to_point does not check.
  /\ |q01| >= 1b-1022 /\ |q02| >= 1b-1022 /\ |q11| >= 1b-1022
  /\ |q12| >= 1b-1022 /\ |q21| >= 1b-1022 /\ |q22| >= 1b-1022
  /\ |k01| >= 1b-1022 /\ |k02| >= 1b-1022 /\ |m0| >= 1b-1022
  # What lemma.g proves each of those nine gives.
  /\ q01 -/ P01 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q02 -/ P02 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q11 -/ P11 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q12 -/ P12 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q21 -/ P21 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q22 -/ P22 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k01 -/ K1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k02 -/ K2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]
  # L is at least every coordinate of the point and the shift.
  /\ L in [1b-1074, 1b1024]
  /\ p0 / L in [-1, 1] /\ p1 / L in [-1, 1] /\ p2 / L in [-1, 1]
  /\ s0 / L in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v0 / |v0| in [-1, 1]
  /\ v1 / |v1| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  W1 in ? /\ W3 in ? /\ W7 in ? /\ W8 in ? /\ D / L in ?
  /\ (out - Ex) / D in ?
  /\ (out - Ex) / L in [-36b-53, 36b-53]
}

# How far each rounding is from the identity.
q01 / P01 -> 1 + (q01 - P01) / P01;
q02 / P02 -> 1 + (q02 - P02) / P02;
q11 / P11 -> 1 + (q11 - P11) / P11;
q12 / P12 -> 1 + (q12 - P12) / P12;
q21 / P21 -> 1 + (q21 - P21) / P21;
q22 / P22 -> 1 + (q22 - P22) / P22;
c0 / C0 -> 1 + (c0 - C0) / C0;
c1 / C1 -> 1 + (c1 - C1) / C1;
c2 / C2 -> 1 + (c2 - C2) / C2;
k01 / K1 -> 1 + (k01 - K1) / K1;
k02 / K2 -> 1 + (k02 - K2) / K2;
k0 / KK -> 1 + (k0 - KK) / KK;
m0 / M -> 1 + (m0 - M) / M;
a0 / A -> 1 + (a0 - A) / A;
r0 / R -> 1 + (r0 - R) / R;
out / O -> 1 + (out - O) / O;

# The error is the sum of the seven shares' errors, each a fraction of its
# own share of D; add_rr then averages them.
(out - Ex) / L -> ((out - Ex) / D) * (D / L);
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (d1 + e1) + (d2 + e2) + (d3 + e3) + (d4 + e4)
                + (d5 + e5) + (d6 + e6) + (d7 + e7);
(d1 + e1) - d1 -> e1;
(d2 + e2) - d2 -> e2;
(d3 + e3) - d3 -> e3;
(d4 + e4) - d4 -> e4;
(d5 + e5) - d5 -> e5;
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e1 / d1 -> (w / |w|) * (v1 / |v1|) * (p2 / L) * (W1 / 6);
e2 / d2 -> -((w / |w|) * (v2 / |v2|) * (p1 / L) * (W2 / 6));
e3 / d3 -> (v1 / |v1|) * (v0 / |v0|) * (p1 / L) * (W3 / 7);
e4 / d4 -> -((v1 / |v1|) * (v1 / |v1|) * (p0 / L) * (W4 / 7));
e5 / d5 -> -((v2 / |v2|) * (v2 / |v2|) * (p0 / L) * (W5 / 7));
e6 / d6 -> (v2 / |v2|) * (v0 / |v0|) * (p2 / L) * (W6 / 7);
e7 / d7 -> ((p0 / L) * W7 + (s0 / L) * W8) / 3;

# D / L is 3 and a quadratic in |w|, |v0|, |v1|, |v2|, and that quadratic is
# 109/5 q.q less four squares: Gappa bounds each square below by zero.
D / L -> 109/5 * N4 - (109/5 * (X * X) + 109/5 * (Y * Y) + 425/109 * (Z * Z)
                       + 1/545 * (|v1| * |v1| + |v2| * |v2|)) + 3;
```

`doubling.g`, which proves the doubling rounds nothing (2):

```gappa
# nalgebra's t = c * 2 rounds nothing: for any f64 c that does not overflow
# when doubled, rnd(c * 2) is c * 2. point.g writes t without the rounding on
# the strength of this.
@rnd = float<ieee_64, ne>;

c = rnd(c_);
t = rnd(c * 2);

{
  |c| <= 1b1023
->
  t - c * 2 in [0, 0]
}
```

`controls.py`, which writes the three controls of 7:

```python
"""Write point.g's controls. Each changes one thing in it: a smaller bound,
to see whether it proved its goal from hypotheses that contradict each other,
or a hypothesis dropped, to see whether the proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


point = open("point.g").read()
GOAL = "/\\ (out - Ex) / L in [-36b-53, 36b-53]"

# 24.75u, below the 10 + sqrt(219) = 24.80u the averaging reaches.
write("c_point_2475.g", sub(point, GOAL, "/\\ (out - Ex) / L in [-99b-55, 99b-55]"))

# Without |q.q - 1| <= 6 epsilon.
write("c_point_non4.g",
      sub(point, "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]\n", ""))

# Without the nine products normal, and what lemma.g gives from them.
start = point.index("  # The nine products are normal")
end = point.index("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
end += len("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
block = point[start:end]
assert block.count(">= 1b-1022") == 9 and block.count(" -/ ") == 9, block
write("c_point_nonormal.g", point[:start] + point[end:])
```

`turn.py`, which writes `point1.g` and `point2.g`, the moved point's `y` and
`z`:

```python
"""Write point1.g and point2.g, the moved point's y and z: point.g with every
coordinate index turned 0 -> 1 -> 2 -> 0, once and twice. nalgebra's cross
writes component i as v[i+1] * p[i+2] - v[i+2] * p[i+1], indices mod 3, and
the last line works each coordinate on its own, so y is x with the indices
turned once and z with them turned twice.

A name whose first digit is a coordinate index is turned there and keeps any
second digit, which counts products. W, e, d, K and N number terms, shares,
products and the four squares, and are left alone."""

import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))

TURNED = {"v", "p", "s", "P", "q", "C", "c", "rq", "rc", "t", "k", "rk"}
NAME = re.compile(r"\b([A-Za-z]+)(\d)(\d?)(_?)\b")


def turn(src):
    def one(m):
        prefix, index, rest, under = m.groups()
        if prefix not in TURNED:
            return m.group(0)
        return prefix + str((int(index) + 1) % 3) + rest + under

    return NAME.sub(one, src)


point = open("point.g").read()
once = turn(point)
twice = turn(once)
assert turn(twice) == point
X = "# moved point's x is within"
assert point.count(X) == 1
for name, src, axis in (("point1.g", once, "y"), ("point2.g", twice, "z")):
    open(name, "w").write(src.replace(X, "# moved point's " + axis + " is within"))
    print(name, "written,", len(NAME.findall(src)), "names")
```

`sos.py`, which checks the identity of 3 in exact rationals and looks for the
largest `D / L` on a grid:

```python
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
```

It printed:

```text
identity holds at 1000 random points
K = 21.8 root of K^2 - 14K - 170: 21.798648586948744
largest D / L on a 200-step grid of unit quaternions: 24.798304247747573
```

`all.sh`, which writes the controls and the `y` and `z` scripts and runs
every script at once, each through 464.13's `one.sh`, unchanged:

```sh
#!/bin/bash
# Write the controls and the y and z scripts, then run every script at once,
# each through 464.13's one.sh: point.g and doubling.g with coq, and the
# controls, point1.g and point2.g without.
cd "$(dirname "$0")"
python3 controls.py
python3 turn.py
for f in point.g doubling.g; do
  ./one.sh "$f" coq > "${f%.g}.run" 2>&1 &
done
for f in c_point_2475.g c_point_non4.g c_point_nonormal.g point1.g point2.g; do
  ./one.sh "$f" > "${f%.g}.run" 2>&1 &
done
wait
cat point.run doubling.run c_point_2475.run c_point_non4.run c_point_nonormal.run point1.run point2.run
```

Its third run, the last lane's, printed:

```text
point1.g written, 470 names
point2.g written, 470 names
point exit 0 46.04s
point coq -Bcoq exit 0 coqc exit 0 58.58s lines 8205 hints 34
doubling exit 0 0.01s
doubling coq -Bcoq exit 0 coqc exit 0 0.47s lines 149 hints 0
c_point_2475 exit 1 51.77s
c_point_non4 exit 1 51.07s
c_point_nonormal exit 1 43.12s
point1 exit 0 45.42s
point2 exit 0 43.85s
```

`same.py`, whole:

```python
"""Check that each script 464.19 appends whole is byte for byte the file that
was run, in q2689/; that one.sh is 464.13's block; that sos.py, run again,
prints what 464.19 gives, and that all.log is the print 464.19 gives for
all.sh; and that controls.py and turn.py, run again in a fresh directory
beside point.g, each from the appended text, write the scripts that ran.
Run as: python3 same.py <doc> <q2689 dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, ran = sys.argv[1], sys.argv[2]
text = open(doc).read()
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)


def section(n):
    start = text.index(f"### 464.{n} ")
    end = text.find("\n### ", start + 1)
    return text[start:] if end < 0 else text[start:end + 1]


sec = section(19)
blocks = FENCED.findall(sec)
print("code blocks in 464.19:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
marks = {
    "point.g": "# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e\n",
    "doubling.g": "# nalgebra's t = c * 2 rounds nothing",
    "controls.py": '"""Write point.g\'s controls',
    "turn.py": '"""Write point1.g and point2.g',
    "sos.py": '"""Check the identity point.g\'s D / L hint',
    "all.sh": "#!/bin/bash\n# Write the controls and the y and z scripts",
    "same.py": '"""Check that each script 464.19 appends whole',
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
    path = os.path.abspath(__file__) if name == "same.py" else os.path.join(ran, name)
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")


def block_of(n, first):
    got = [body for lang, body in FENCED.findall(section(n)) if body.startswith(first)]
    assert len(got) == 1, (n, first, len(got))
    return got[0]


def printed(first):
    got = [body for lang, body in blocks if lang == "text" and body.startswith(first)]
    assert len(got) == 1, (first, len(got))
    return got[0]


print("one.sh is 464.13's block:", open(os.path.join(ran, "one.sh")).read()
      == block_of(13, "#!/bin/bash\n# Run one Gappa script under 464.4's pin"))
again = subprocess.run(["python3", os.path.join(ran, "sos.py")], capture_output=True, text=True,
                       check=True).stdout
print("sos.py run again prints what 464.19 gives:", again == printed("identity holds at"))
print("all.log is what 464.19 gives:",
      open(os.path.join(ran, "all.log")).read() == printed("point1.g written"))

fresh = tempfile.mkdtemp()
for name in ("point.g", "controls.py", "turn.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "controls.py"], cwd=fresh, check=True)
subprocess.run(["python3", "turn.py"], cwd=fresh, check=True, capture_output=True)
written = sorted(f for f in os.listdir(fresh) if f.endswith(".g") and f not in found)
for name in written:
    print(name, "written again is the one that ran:",
          filecmp.cmp(os.path.join(fresh, name), os.path.join(ran, name), shallow=False))
print(".g files that ran and are neither appended nor written again:",
      sorted(f for f in os.listdir(ran) if f.endswith(".g") and f not in found and f not in written))
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.19: 10
point.g same
doubling.g same
controls.py same
turn.py same
sos.py same
all.sh same
same.py same
one.sh is 464.13's block: True
sos.py run again prints what 464.19 gives: True
all.log is what 464.19 gives: True
c_point_2475.g written again is the one that ran: True
c_point_non4.g written again is the one that ran: True
c_point_nonormal.g written again is the one that ran: True
point1.g written again is the one that ran: True
point2.g written again is the one that ran: True
.g files that ran and are neither appended nor written again: []
```

**10. What this leaves.** Four rows, filed from this write-up when it merges,
each from Q2689:

- Run `point.g` in the zero branches (5): a zero factor, as a turn about a
  coordinate axis stores two of `v` as zero and a point on a coordinate
  plane has a zero coordinate, and each sum that may cancel to zero, as 464.11
  and 464.18 ran the plane's. These are common inputs. Not run, so no class
  (§360.5).
- Run `point.g` with one or more of the nine products below the normals and
  `L` normal, as 464.8's `planecases.py` split the plane's, and say whether
  `POINT_ROUNDOFF` needs the products normal or only `L`, as `nearest.g` found
  `L` normal enough (464.9's 1). Not run, so no class.
- Whether `apply_to_point` keeps within `POINT_ROUNDOFF` on a movement that
  came in through `From<na::Isometry3<f64>>` (5) with a turn whose `q·q` is
  further from one than `6ε`, and if not, hold that route to the length as
  `compose` holds its own. Not run, so no class.
- Small: cite this subsection in `POINT_ROUNDOFF`'s doc (`isometry.rs:37-91`),
  which says the 18 covers "the fact that a stored rotation has length one
  only to within rounding" without saying how far: Gappa proves the 18 with
  the turn's `q·q` within `6ε` of one and the nine products normal, and
  reaches `24.80u` per coordinate where the doc counts `35.78u` (4).

And what is not a row:

- Q2690 and Q2691 wait on Q2689 and rest on `point.g`: `NORMAL_LENGTH_ROUNDOFF`
  and `CORNER_AFTER_MOVING_ROUNDOFF` (464.9).
- Q2700 already waits on Q2689, and moves these scripts into `proofs/gappa/`
  with the others that rows of 464 appended.
- Overflow, which does not reach Gappa (464.4). The doc's own condition, that
  nothing overflows, stands as it is.
- `field` on these scripts' hints (464.10).

### 464.20 Plane::NORMAL_LENGTH_ROUNDOFF

Q2690 proved `Plane::NORMAL_LENGTH_ROUNDOFF` in Gappa on 2026-10-26 at
`4b642567`, on 464.19's `point.g`. A normal a constructor stores, turned once,
has `n·n` within `59.64ε` of one where the code allows `64ε`, while the
turn's `q·q` is within `6ε` of one and the hypotheses of 464.19's 5 hold. The
share the doc measures and does not derive, Gappa derives, from the turn's
length (3). Coq checks the proof, modulo its hints (7).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, unchanged, against `plane.rs`, `point.rs` and
`isometry.rs` at `4b642567` and nalgebra 0.35.0. `isometry.rs` differs from
464.19's `ba0a654e` in `POINT_ROUNDOFF`'s doc alone. `u` is `2⁻⁵³` and `ε` is
`f64::EPSILON = 2u`.

**1. What the proof guards.** `NORMAL_LENGTH_ROUNDOFF` (`plane.rs:114-165`)
is how far `n·n` may sit from one and still be a turned unit normal:
`has_length_one_to_within_a_turn` (`plane.rs:777-807`) keeps a normal inside
it, for `Plane::from_point_and_normal_as_written` and `Axis::as_written`, and
scales one outside it. A moved plane's normal is `Plane::transformed`
(`plane.rs:544-550`), which is `Isometry::apply_to_vector`
(`isometry.rs:335-337`) and so `turned_by` (`isometry.rs:527-532`): 464.19's
arithmetic with no shift. The doc counts two roundings, the constructor's,
`ε + ε²/4`, and the turn's, `2√3 · 18ε ≈ 62.35ε` from `POINT_ROUNDOFF`,
`63.4ε` in all. It says one share is left out and measured instead: the
turn's arithmetic, done exactly, keeps a length only when `q·q` is exactly
one.

**2. The scripts.** `vector.g` is `point.g` with no shift and every
component of the vector at most one, which a constructor's normal is. It
proves each turned component within `20u` of the same arithmetic without
rounding. Its terms pass one rounding fewer than `point.g`'s, five through
`w`, six through `k` and one for `n₀`, so its `D` is `10|w|s + 12|v₀|s +
12(|v₁|² + |v₂|²) + 1`. The quadratic's largest eigenvalue is `6 + √158 =
18.5698`, a root of `λ² − 12λ − 122`, and `93/5 = 18.6` is just above it, so
`D ≤ 93/5 · q·q + 1 = 19.6`. `sos.py` (8) checks the identity, and finds the
largest `D` on a 200-step grid of unit quaternions to be `19.5695`. `turn.py`
writes `vector1.g` and `vector2.g`, the `y` and `z`.

`square.g` takes the constructor's `nᵢ = rnd(mᵢ)` of an exact unit vector
`m`, the turn's arithmetic `Ex` without rounding, and the turned vector as
`Ex + d` with each `|dᵢ| ≤ 20u`, which is `vector.g`'s goal: the two join by
hand, at `d`, as 464.19's `doubling.g` joins `point.g`. It proves `|n·n − 1| ≤
128u`, the code's `64ε`, by

`out·out − 1 = (n·n − 1) + 4(q·q − 1)|v × n|² + 2 Ex·d + d·d`,

and prints each part:

- `n·n − 1` within `±2.22045e-16`, `2u`, the constructor's `ε`: the doc's
  first share, by 464.19's averaging over the `mᵢ²`, which add to one;
- `4(q·q − 1)|v × n|²` within `±5.32907e-15`, `48u`, which is `24ε`;
- `2 Ex·d` within `±7.69185e-15`, `69.28u`, which is `2√3 · 20u`;
- `d·d` at most `75b-102`, `1200u²`.

**3. The share that was measured.** nalgebra's `t·w + v × t + n` with `t =
2(v × n)` is `n(1 − 2|v|²) + 2w(v × n) + 2v(v·n)`, and its square length less
`n·n` is exactly `4(q·q − 1)|v × n|²`: `sos.py` checks that in rationals at
1000 points, and Gappa takes it as a hint without reporting it as not an
identity. `|v × n|²` is a sum of squares and at most `|v|²|n|²`, since what
lies between is `(v·n)²`. So with `|q·q − 1| ≤ 6ε` the share is within `24ε`,
and Gappa derives it. What it does not derive is the `6ε` itself, the same
hypothesis as 464.19's 5: `compose` holds its turns there by
`held_near_length_one`'s count, `Isometry::rotation`'s is measured
(`TURN_LENGTH_DRIFT`), and since Q2803 `TryFrom<na::Isometry3<f64>>` keeps a
turn within `TURN_LENGTH_DRIFT` of one bit for bit and divides any other by
its length as stored parts are read, so the `6ε` hypothesis holds on it too.

The 64 holds for a turn within `7ε` of length one and not by this argument at
`7.5ε`: `r_square_14.g`, `|q·q − 1| ≤ 14u`, proves, and `r_square_15.g`,
`15u`, fails with best `±1.45752e-14`, `131.28u`. With `q·q` exactly one,
`r_square_one.g` proves and the share prints `[0, 0]`.

**4. What the doc's `63.4ε` counts.** It counts the constructor's `ε` and
`2√3 · 18ε` for the turn's rounding, and leaves the `0.6ε` up to `64ε`. The
share of 3 does not fit in that: added at Gappa's `24ε`, the doc's count is
`87.4ε`, past the bound. What makes room is the turn's rounding for a vector,
`19.6u` per component where the doc charges `POINT_ROUNDOFF`'s `36u`, a
point's bound with a shift and `L` for every coordinate. Gappa's sum is `2u +
48u + 69.28u = 119.28u`, `59.64ε`, which leaves `4.36ε`. The test
`every_normal_a_turn_leaves_sits_well_inside_the_bound` finds `22.16ε` at
worst (`plane.rs:141-143`), which is under both.

**5. The hypotheses.** `vector.g`'s are `point.g`'s (464.19's 5) less the
shift, and each `|nᵢ| ≤ 1`. `square.g`'s, in its order:

- `m` of length exactly one, so each `|mᵢ| ≤ 1`;
- each `|mᵢ| ≥ 2⁻¹⁰¹²`, where `scaled_to_length_one_from_anywhere`
  (`point.rs:335-398`) promises the nearest `f64`, and so no component zero;
- the quaternion's components finite, and `|q·q − 1| ≤ 6ε` worked out exactly;
- each `|dᵢ| ≤ 20u`, what `vector.g`, `vector1.g` and `vector2.g` prove.

**6. The controls.** Each changes one thing and exits 1:

- `c_vector_195.g`, the goal at `19.5u`: best `±2.17604e-15`, `19.6u`, so
  `vector.g`'s hypotheses do not contradict each other;
- `c_vector_non4.g`, without the turn near length one: best `±2.43978e+602`;
- `c_vector_nonormal.g`, without the nine products normal: best `±29.1613`,
  with `W1` and `W3` unproved;
- `c_square_119.g`, the goal at `119u`: best `±1.3243e-14`, `119.28u`, so
  `square.g`'s do not either;
- `c_square_non4.g`, without the turn near length one: best `[-1,
  5.01307e+1234]`. The proof needs the turn's length, which is the share;
- `c_square_nod.g`, without `vector.g`'s bound on `d`: best `[-inf, inf]`.

**7. Coq checks the proof, modulo its hints.** `gappa -Bcoq` and `coqc`,
through `one.sh`, exit 0 on `vector.g` and `square.g`: on the last run of
`all.sh` (8), `vector.v` came to 6740 lines with 33 hints and `square.v` to
4398 with 18. The lines are that run's and no other's: `gappa -Bcoq` does not
write the same file each time. `again.sh` (8) ran `square.g` through
`one.sh` three times more and got 4394, 4394 and 4405 lines, two different
files, each with 18 hints and each accepted by `coqc`; the first lane's run
wrote 6399 and 4595. What Gappa proves does not move with it: every bound 2,
3 and 6 quote, read off the first lane's run, is what the last run printed
too.

As in 464.19's 8, each hint enters the Coq file as a hypothesis with no
condition, so the proof stands on its hints and on the divisors Gappa took to
be nonzero: 134 warnings that a divisor is "assumed to be nonzero" in
`vector.out`, as in `vector1.out` and `vector2.out`, and 15 in `square.out`,
counted on the last run with `grep -c 'assumed to be nonzero'`. None of the
four says a hint's two sides are "not trivially equal". `vector.out` also
says there is "no need for the rewriting at line 136", the hint `out - Ex ->
((out - Ex) / D) * D`, if `D != 0`. `vector1.g` and `vector2.g` ran without
`-Bcoq`.

**8. The scripts, whole, as they ran.** To run them again, save the first
six, `vector.g` to `all.sh`, and `one.sh` in one directory and run `bash
all.sh` there, which took two to two and a half minutes. `again.sh` and
`same.py`, after them, are checks, and `all.sh` needs neither.

`vector.g`:

```gappa
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs turned_by at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 4b642567
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: Cargo.lock nalgebra at 4b642567
# Isometry::apply_to_vector, crates/rcad-geom/src/isometry.rs at 4b642567,
# through nalgebra 0.35's UnitQuaternion * Vector3: point.g with no shift and
# a vector of components at most one. The turned vector's x is within
# 20u of what the same arithmetic gives without rounding. y and z are this
# with the indices turned 0 -> 1 -> 2 -> 0, as turn.py writes them.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) and the vector n, each an f64.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);

# c = v x n, as nalgebra's cross writes each component: a*b - c*d.
P01 = v1 * n2; q01 = rnd(P01); P02 = v2 * n1; q02 = rnd(P02);
C0 = q01 - q02; c0 = rnd(C0);
P11 = v2 * n0; q11 = rnd(P11); P12 = v0 * n2; q12 = rnd(P12);
C1 = q11 - q12; c1 = rnd(C1);
P21 = v0 * n1; q21 = rnd(P21); P22 = v1 * n0; q22 = rnd(P22);
C2 = q21 - q22; c2 = rnd(C2);

# t = c * 2, written without its rounding, on the strength of 464.19's
# doubling.g.
t0 = c0 * 2; t1 = c1 * 2; t2 = c2 * 2;

# k = v x t, its x.
K1 = v1 * t2; k01 = rnd(K1); K2 = v2 * t1; k02 = rnd(K2);
KK = k01 - k02; k0 = rnd(KK);

# t * w + k + n, folded left to right. A vector takes no shift.
M = t0 * w; m0 = rnd(M);
A = m0 + k0; a0 = rnd(A);
R = a0 + n0; out = rnd(R);

# The same arithmetic without rounding.
Ex = 2 * w * (v1 * n2 - v2 * n1)
   + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2))
   + n0;

# What each rounding multiplied its argument by.
rq01 = q01 / P01; rq02 = q02 / P02; rq11 = q11 / P11; rq12 = q12 / P12;
rq21 = q21 / P21; rq22 = q22 / P22;
rc0 = c0 / C0; rc1 = c1 / C1; rc2 = c2 / C2;
rk01 = k01 / K1; rk02 = k02 / K2; rk0 = k0 / KK;
rm = m0 / M; ra = a0 / A; rr = out / R;

# The seven terms of Ex, each as out carries it, less the term itself: five
# roundings for the two through w, six for the four through k, one for n0.
W1 = rr * ra * rm * rc0 * rq01 - 1;
W2 = rr * ra * rm * rc0 * rq02 - 1;
W3 = rr * ra * rk0 * rk01 * rc2 * rq21 - 1;
W4 = rr * ra * rk0 * rk01 * rc2 * rq22 - 1;
W5 = rr * ra * rk0 * rk02 * rc1 * rq11 - 1;
W6 = rr * ra * rk0 * rk02 * rc1 * rq12 - 1;
W7 = rr - 1;
e1 = 2 * w * v1 * n2 * W1; e2 = -(2 * w * v2 * n1 * W2);
e3 = 2 * v1 * v0 * n1 * W3; e4 = -(2 * v1 * v1 * n0 * W4);
e5 = -(2 * v2 * v2 * n0 * W5); e6 = 2 * v2 * v0 * n2 * W6;
e7 = n0 * W7;

# Each term's share of the bound: its size with 1 for the vector's
# component, times the roundings it meets.
d1 = 10 * |w| * |v1|; d2 = 10 * |w| * |v2|;
d3 = 12 * |v1| * |v0|; d4 = 12 * |v1| * |v1|;
d5 = 12 * |v2| * |v2|; d6 = 12 * |v2| * |v0|;
d7 = 1;
D = d1 + d2 + d3 + d4 + d5 + d6 + d7;

# q.q, each square written as |x| * |x|, and the four squares D is written
# with below.
N4 = |w| * |w| + |v0| * |v0| + |v1| * |v1| + |v2| * |v2|;
X = |w| - 25/93 * (|v1| + |v2|);
Y = |v0| - 10/31 * (|v1| + |v2|);
Z = |v1| - |v2|;

{
  # Every input is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # No factor of a product is zero.
  /\ |w| >= 1b-1074 /\ |v0| >= 1b-1074 /\ |v1| >= 1b-1074 /\ |v2| >= 1b-1074
  /\ |n0| >= 1b-1074 /\ |n1| >= 1b-1074 /\ |n2| >= 1b-1074
  # No sum cancels to zero.
  /\ |C0| >= 1b-1074 /\ |C1| >= 1b-1074 /\ |C2| >= 1b-1074 /\ |KK| >= 1b-1074
  /\ |A| >= 1b-1074 /\ |R| >= 1b-1074
  # The nine products are normal, which apply_to_vector does not check.
  /\ |q01| >= 1b-1022 /\ |q02| >= 1b-1022 /\ |q11| >= 1b-1022
  /\ |q12| >= 1b-1022 /\ |q21| >= 1b-1022 /\ |q22| >= 1b-1022
  /\ |k01| >= 1b-1022 /\ |k02| >= 1b-1022 /\ |m0| >= 1b-1022
  # What lemma.g proves each of those nine gives.
  /\ q01 -/ P01 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q02 -/ P02 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q11 -/ P11 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q12 -/ P12 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q21 -/ P21 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ q22 -/ P22 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k01 -/ K1 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ k02 -/ K2 in [-4503599627370497b-105, 4503599627370497b-105]
  /\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]
  # Every component of the vector is at most one: a constructor's normal is
  # the nearest f64 to a unit vector's components.
  /\ n0 in [-1, 1] /\ n1 in [-1, 1] /\ n2 in [-1, 1]
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ w / |w| in [-1, 1] /\ v0 / |v0| in [-1, 1]
  /\ v1 / |v1| in [-1, 1] /\ v2 / |v2| in [-1, 1]
->
  W1 in ? /\ W3 in ? /\ W7 in ? /\ D in ?
  /\ (out - Ex) / D in ?
  /\ out - Ex in [-20b-53, 20b-53]
}

# How far each rounding is from the identity.
q01 / P01 -> 1 + (q01 - P01) / P01;
q02 / P02 -> 1 + (q02 - P02) / P02;
q11 / P11 -> 1 + (q11 - P11) / P11;
q12 / P12 -> 1 + (q12 - P12) / P12;
q21 / P21 -> 1 + (q21 - P21) / P21;
q22 / P22 -> 1 + (q22 - P22) / P22;
c0 / C0 -> 1 + (c0 - C0) / C0;
c1 / C1 -> 1 + (c1 - C1) / C1;
c2 / C2 -> 1 + (c2 - C2) / C2;
k01 / K1 -> 1 + (k01 - K1) / K1;
k02 / K2 -> 1 + (k02 - K2) / K2;
k0 / KK -> 1 + (k0 - KK) / KK;
m0 / M -> 1 + (m0 - M) / M;
a0 / A -> 1 + (a0 - A) / A;
out / R -> 1 + (out - R) / R;

# The error is the sum of the seven shares' errors, each a fraction of its
# own share of D; add_rr then averages them.
out - Ex -> ((out - Ex) / D) * D;
(out - Ex) / D -> ((D + (out - Ex)) - D) / D;
D + (out - Ex) -> (d1 + e1) + (d2 + e2) + (d3 + e3) + (d4 + e4)
                + (d5 + e5) + (d6 + e6) + (d7 + e7);
(d1 + e1) - d1 -> e1;
(d2 + e2) - d2 -> e2;
(d3 + e3) - d3 -> e3;
(d4 + e4) - d4 -> e4;
(d5 + e5) - d5 -> e5;
(d6 + e6) - d6 -> e6;
(d7 + e7) - d7 -> e7;
e1 / d1 -> (w / |w|) * (v1 / |v1|) * n2 * (W1 / 5);
e2 / d2 -> -((w / |w|) * (v2 / |v2|) * n1 * (W2 / 5));
e3 / d3 -> (v1 / |v1|) * (v0 / |v0|) * n1 * (W3 / 6);
e4 / d4 -> -((v1 / |v1|) * (v1 / |v1|) * n0 * (W4 / 6));
e5 / d5 -> -((v2 / |v2|) * (v2 / |v2|) * n0 * (W5 / 6));
e6 / d6 -> (v2 / |v2|) * (v0 / |v0|) * n2 * (W6 / 6);
e7 / d7 -> n0 * W7;

# D is 1 and a quadratic in |w|, |v0|, |v1|, |v2|, and that quadratic is
# 93/5 q.q less four squares: Gappa bounds each square below by zero.
D -> 93/5 * N4 - (93/5 * (X * X) + 93/5 * (Y * Y) + 305/93 * (Z * Z)
                  + 19/465 * (|v1| * |v1| + |v2| * |v2|)) + 1;
```

`square.g`:

```gappa
# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at 4b642567
# restates: crates/rcad-geom/src/plane.rs transformed at 4b642567
# restates: crates/rcad-geom/src/point.rs scaled_to_length_one_from_anywhere at 4b642567
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567
# restates: crates/rcad-geom/src/isometry.rs held_near_length_one at 4b642567
# Plane::NORMAL_LENGTH_ROUNDOFF, crates/rcad-geom/src/plane.rs at 4b642567:
# a constructor's normal, turned once, has n.n within 64 * f64::EPSILON of
# one. The constructor rounds each component of an exact unit vector m to
# the nearest f64; the turn is nalgebra's arithmetic, and vector.g,
# vector1.g and vector2.g prove each turned component within 20u of what
# that arithmetic gives without rounding, which is the hypothesis on d0, d1
# and d2 here.
@rnd = float<ieee_64, ne>;

# The constructor: n is the nearest f64 to the exact unit vector m.
n0 = rnd(m0); n1 = rnd(m1); n2 = rnd(m2);
M2 = m0 * m0 + m1 * m1 + m2 * m2;
NN = n0 * n0 + n1 * n1 + n2 * n2;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;

# nalgebra's turn of n without rounding, as vector.g's Ex writes x.
Ex0 = 2 * w * (v1 * n2 - v2 * n1)
    + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2)) + n0;
Ex1 = 2 * w * (v2 * n0 - v0 * n2)
    + v2 * (2 * (v1 * n2 - v2 * n1)) - v0 * (2 * (v0 * n1 - v1 * n0)) + n1;
Ex2 = 2 * w * (v0 * n1 - v1 * n0)
    + v0 * (2 * (v2 * n0 - v0 * n2)) - v1 * (2 * (v1 * n2 - v2 * n1)) + n2;
ExEx = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;

# The turned vector the f64 arithmetic gives: Ex and each rounding's slip.
o0 = Ex0 + d0; o1 = Ex1 + d1; o2 = Ex2 + d2;
OO = o0 * o0 + o1 * o1 + o2 * o2;
DD = d0 * d0 + d1 * d1 + d2 * d2;
F2 = 2 * (Ex0 * d0 + Ex1 * d1 + Ex2 * d2);

# v x n and v . n, without rounding, and v.v.
X0 = v1 * n2 - v2 * n1; X1 = v2 * n0 - v0 * n2; X2 = v0 * n1 - v1 * n0;
VN = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * n0 + v1 * n1 + v2 * n2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

# The constructor's shares: m_i^2, and what rounding m_i adds to its square.
g0 = m0 * m0; g1 = m1 * m1; g2 = m2 * m2;
h0 = n0 * n0 - m0 * m0; h1 = n1 * n1 - m1 * m1; h2 = n2 * n2 - m2 * m2;

{
  # m is a unit vector, so each component is at most one.
     M2 in [1, 1]
  /\ m0 in [-1, 1] /\ m1 in [-1, 1] /\ m2 in [-1, 1]
  # Above 2^-1012 each component is the nearest f64 to the exact ratio
  # (scaled_to_length_one_from_anywhere's doc), and none is zero.
  /\ |m0| >= 1b-1012 /\ |m1| >= 1b-1012 /\ |m2| >= 1b-1012
  # Every quaternion component is a finite f64.
  /\ |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # What vector.g, vector1.g and vector2.g prove of each turned component.
  /\ d0 in [-20b-53, 20b-53] /\ d1 in [-20b-53, 20b-53]
  /\ d2 in [-20b-53, 20b-53]
->
  NN - 1 in ? /\ VN in ? /\ 4 * (N4 - 1) * VN in ? /\ F2 in ? /\ DD in ?
  /\ OO - 1 in [-128b-53, 128b-53]
}

# n.n off one is the constructor's, then the turn's own arithmetic, then the
# turn's rounding.
OO - 1 -> (NN - 1) + 4 * (N4 - 1) * VN + F2 + DD;
ExEx -> NN + 4 * (N4 - 1) * VN;

# The constructor: each n_i^2 is m_i^2 times a factor within 2u + u^2 of
# one, and the m_i^2 add to one; add_rr averages them.
NN - 1 -> ((NN - M2) / M2) * M2 + (M2 - 1);
(NN - M2) / M2 -> ((M2 + (NN - M2)) - M2) / M2;
M2 + (NN - M2) -> (g0 + h0) + (g1 + h1) + (g2 + h2);
(g0 + h0) - g0 -> h0;
(g1 + h1) - g1 -> h1;
(g2 + h2) - g2 -> h2;
h0 / g0 -> (n0 / m0) * (n0 / m0) - 1;
h1 / g1 -> (n1 / m1) * (n1 / m1) - 1;
h2 / g2 -> (n2 / m2) * (n2 / m2) - 1;
n0 / m0 -> 1 + (n0 - m0) / m0;
n1 / m1 -> 1 + (n1 - m1) / m1;
n2 / m2 -> 1 + (n2 - m2) / m2;

# |v x n|^2 is at least zero, a sum of squares, and at most v.v n.n, since
# what is between is (v . n)^2; v.v is q.q less w^2.
VN -> VV * NN - VD * VD;
VV -> N4 - w * w;

# 2 Ex . d, both ways round: 2ab is c a^2 + b^2 / c less (c a - b)^2 / c,
# and (c a + b)^2 / c less c a^2 + b^2 / c. c is near sqrt(3) * 20u, which
# makes c Ex.Ex + DD / c smallest for Ex.Ex near one and DD at most 3 (20u)^2.
F2 -> (34641b-53 / 1000 * ExEx + DD / (34641b-53 / 1000))
      - ((34641b-53 / 1000 * Ex0 - d0) * (34641b-53 / 1000 * Ex0 - d0)
       + (34641b-53 / 1000 * Ex1 - d1) * (34641b-53 / 1000 * Ex1 - d1)
       + (34641b-53 / 1000 * Ex2 - d2) * (34641b-53 / 1000 * Ex2 - d2))
        / (34641b-53 / 1000);
F2 -> ((34641b-53 / 1000 * Ex0 + d0) * (34641b-53 / 1000 * Ex0 + d0)
     + (34641b-53 / 1000 * Ex1 + d1) * (34641b-53 / 1000 * Ex1 + d1)
     + (34641b-53 / 1000 * Ex2 + d2) * (34641b-53 / 1000 * Ex2 + d2))
        / (34641b-53 / 1000)
      - (34641b-53 / 1000 * ExEx + DD / (34641b-53 / 1000));
```

`turn.py`, which writes `vector1.g` and `vector2.g`:

```python
"""Write vector1.g and vector2.g, the turned vector's y and z: vector.g with
every coordinate index turned 0 -> 1 -> 2 -> 0, once and twice, as 464.19's
turn.py writes point1.g and point2.g from point.g, with n for its p and no
shift.

A name whose first digit is a coordinate index is turned there and keeps any
second digit, which counts products. W, e, d, K and N number terms, shares,
products and q.q, and are left alone."""

import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))

TURNED = {"v", "n", "P", "q", "C", "c", "rq", "rc", "t", "k", "rk"}
NAME = re.compile(r"\b([A-Za-z]+)(\d)(\d?)(_?)\b")


def turn(src):
    def one(m):
        prefix, index, rest, under = m.groups()
        if prefix not in TURNED:
            return m.group(0)
        return prefix + str((int(index) + 1) % 3) + rest + under

    return NAME.sub(one, src)


vector = open("vector.g").read()
once = turn(vector)
twice = turn(once)
assert turn(twice) == vector
X = "# a vector of components at most one. The turned vector's x is within"
assert vector.count(X) == 1
for name, src, axis in (("vector1.g", once, "y"), ("vector2.g", twice, "z")):
    open(name, "w").write(src.replace(X, X[:-len("x is within")] + axis + " is within"))
    print(name, "written,", len(NAME.findall(src)), "names")
```

`controls.py`, which writes the controls of 6 and the runs of 3:

```python
"""Write vector.g's and square.g's controls, and square.g's runs on how far
the turn may sit from length one. Each changes one thing: a smaller bound, to
see whether the script proved its goal from hypotheses that contradict each
other; a hypothesis dropped, to see whether the proof needs it; or the turn's
q.q held to another width, to see where the 64 stops holding."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


N4 = "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]\n"

vector = open("vector.g").read()
VGOAL = "/\\ out - Ex in [-20b-53, 20b-53]"
# 19.5u, below the 18.6 q.q + 1 = 19.6u D reaches.
write("c_vector_195.g", sub(vector, VGOAL, "/\\ out - Ex in [-39b-54, 39b-54]"))
# Without |q.q - 1| <= 6 epsilon.
write("c_vector_non4.g", sub(vector, N4, ""))
# Without the nine products normal, and what lemma.g gives from them.
start = vector.index("  # The nine products are normal")
end = vector.index("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
end += len("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
block = vector[start:end]
assert block.count(">= 1b-1022") == 9 and block.count(" -/ ") == 9, block
write("c_vector_nonormal.g", vector[:start] + vector[end:])

square = open("square.g").read()
SGOAL = "/\\ OO - 1 in [-128b-53, 128b-53]"
# 119u, below the 2u + 48u + 2 sqrt(3) 20u = 119.28u the sum reaches.
write("c_square_119.g", sub(square, SGOAL, "/\\ OO - 1 in [-119b-53, 119b-53]"))
# Without |q.q - 1| <= 6 epsilon: the share the doc measures.
write("c_square_non4.g", sub(square, N4, ""))
# Without what vector.g proves of the turn's rounding.
D = ("  # What vector.g, vector1.g and vector2.g prove of each turned component.\n"
     "  /\\ d0 in [-20b-53, 20b-53] /\\ d1 in [-20b-53, 20b-53]\n"
     "  /\\ d2 in [-20b-53, 20b-53]\n")
write("c_square_nod.g", sub(square, D, ""))
# q.q held to 14u and to 15u of one, 7 and 7.5 epsilon, and to exactly one,
# which is what the doc's count of two roundings assumes.
write("r_square_14.g", sub(square, N4, "  /\\ N4 in [9007199254740978b-53, 9007199254741006b-53]\n"))
write("r_square_15.g", sub(square, N4, "  /\\ N4 in [9007199254740977b-53, 9007199254741007b-53]\n"))
write("r_square_one.g", sub(square, N4, "  /\\ N4 in [1, 1]\n"))
```

`sos.py`, which checks the identities of 2 and 3 in exact rationals and looks
for the largest `D` on a grid:

```python
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
```

It printed:

```text
drift identity holds at 1000 random points
vector identity holds at 1000 random points
K = 18.6 root of K^2 - 12K - 122: 18.569805089976533
largest D on a 200-step grid of unit quaternions: 19.569532313805855
```

`all.sh`, which writes the controls and the `y` and `z` scripts and runs
every script at once, each through 464.13's `one.sh`, unchanged:

```sh
#!/bin/bash
# Write the controls and the y and z scripts, then run every script at once,
# each through 464.13's one.sh: vector.g and square.g with coq, and the
# controls, the runs on q.q's width, vector1.g and vector2.g without.
cd "$(dirname "$0")"
python3 controls.py
python3 turn.py
for f in vector.g square.g; do
  ./one.sh "$f" coq > "${f%.g}.run" 2>&1 &
done
for f in vector1.g vector2.g c_vector_195.g c_vector_non4.g c_vector_nonormal.g \
         c_square_119.g c_square_non4.g c_square_nod.g \
         r_square_14.g r_square_15.g r_square_one.g; do
  ./one.sh "$f" > "${f%.g}.run" 2>&1 &
done
wait
cat vector.run square.run vector1.run vector2.run c_vector_195.run c_vector_non4.run \
    c_vector_nonormal.run c_square_119.run c_square_non4.run c_square_nod.run \
    r_square_14.run r_square_15.run r_square_one.run
```

Its last run, the second lane's, saved as `all.log`, printed:

```text
vector1.g written, 446 names
vector2.g written, 446 names
vector exit 0 13.40s
vector coq -Bcoq exit 0 coqc exit 0 17.34s lines 6740 hints 33
square exit 0 4.61s
square coq -Bcoq exit 0 coqc exit 0 7.35s lines 4398 hints 18
vector1 exit 0 12.16s
vector2 exit 0 12.58s
c_vector_195 exit 1 120.68s
c_vector_non4 exit 1 110.62s
c_vector_nonormal exit 1 12.39s
c_square_119 exit 1 117.55s
c_square_non4 exit 1 7.10s
c_square_nod exit 1 4.74s
r_square_14 exit 0 5.22s
r_square_15 exit 1 16.08s
r_square_one exit 0 4.23s
```

The first lane's run printed the same exits, with `lines 6399` for `vector`
and `lines 4595` for `square`.

`again.sh`, which runs `square.g` with coq three times more (7):

```sh
#!/bin/bash
# Run square.g through one.sh with coq three times, in a directory of its own
# so that all.sh's square.v stays as it ran, and print a checksum of each
# square.v beside one.sh's line: the Coq file is not the same from run to run.
cd "$(dirname "$0")"
d=$(mktemp -d)
cp one.sh square.g "$d"
for i in 1 2 3; do
  "$d/one.sh" square.g coq | grep coq
  md5sum < "$d/square.v"
done
rm -r "$d"
```

It printed:

```text
square coq -Bcoq exit 0 coqc exit 0 7.33s lines 4394 hints 18
e62ab5f91d1b330a17cf537df845d9dd  -
square coq -Bcoq exit 0 coqc exit 0 7.26s lines 4394 hints 18
e62ab5f91d1b330a17cf537df845d9dd  -
square coq -Bcoq exit 0 coqc exit 0 6.73s lines 4405 hints 18
e694ba43c4053cf42cf306c0acbd1b02  -
```

`same.py`, whole:

```python
"""Check that each script 464.20 appends whole is byte for byte the file that
was run, in q2690/; that one.sh is 464.13's block; that sos.py, run again,
prints what 464.20 gives, and that all.log is the print 464.20 gives for
all.sh; and that controls.py and turn.py, run again in a fresh directory
beside vector.g and square.g, each from the appended text, write the scripts
that ran.
Run as: python3 same.py <doc> <q2690 dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, ran = sys.argv[1], sys.argv[2]
text = open(doc).read()
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)


def section(n):
    start = text.index(f"### 464.{n} ")
    end = text.find("\n### ", start + 1)
    return text[start:] if end < 0 else text[start:end + 1]


sec = section(20)
blocks = FENCED.findall(sec)
print("code blocks in 464.20:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
marks = {
    "vector.g": "# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at 4b642567\n",
    "square.g": "# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at 4b642567\n",
    "turn.py": '"""Write vector1.g and vector2.g',
    "controls.py": '"""Write vector.g\'s and square.g\'s controls',
    "sos.py": '"""Check, in exact rationals at random points, the two identities',
    "all.sh": "#!/bin/bash\n# Write the controls and the y and z scripts",
    "again.sh": "#!/bin/bash\n# Run square.g through one.sh with coq three times",
    "same.py": '"""Check that each script 464.20 appends whole',
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
    path = os.path.abspath(__file__) if name == "same.py" else os.path.join(ran, name)
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")


def block_of(n, first):
    got = [body for lang, body in FENCED.findall(section(n)) if body.startswith(first)]
    assert len(got) == 1, (n, first, len(got))
    return got[0]


def printed(first):
    got = [body for lang, body in blocks if lang == "text" and body.startswith(first)]
    assert len(got) == 1, (first, len(got))
    return got[0]


print("one.sh is 464.13's block:", open(os.path.join(ran, "one.sh")).read()
      == block_of(13, "#!/bin/bash\n# Run one Gappa script under 464.4's pin"))
again = subprocess.run(["python3", os.path.join(ran, "sos.py")], capture_output=True, text=True,
                       check=True).stdout
print("sos.py run again prints what 464.20 gives:", again == printed("drift identity holds at"))
print("all.log is what 464.20 gives:",
      open(os.path.join(ran, "all.log")).read() == printed("vector1.g written"))

fresh = tempfile.mkdtemp()
for name in ("vector.g", "square.g", "controls.py", "turn.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "controls.py"], cwd=fresh, check=True)
subprocess.run(["python3", "turn.py"], cwd=fresh, check=True, capture_output=True)
written = sorted(f for f in os.listdir(fresh) if f.endswith(".g") and f not in found)
for name in written:
    print(name, "written again is the one that ran:",
          filecmp.cmp(os.path.join(fresh, name), os.path.join(ran, name), shallow=False))
print(".g files that ran and are neither appended nor written again:",
      sorted(f for f in os.listdir(ran) if f.endswith(".g") and f not in found and f not in written))
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.20: 12
vector.g same
square.g same
turn.py same
controls.py same
sos.py same
all.sh same
again.sh same
same.py same
one.sh is 464.13's block: True
sos.py run again prints what 464.20 gives: True
all.log is what 464.20 gives: True
c_square_119.g written again is the one that ran: True
c_square_nod.g written again is the one that ran: True
c_square_non4.g written again is the one that ran: True
c_vector_195.g written again is the one that ran: True
c_vector_non4.g written again is the one that ran: True
c_vector_nonormal.g written again is the one that ran: True
r_square_14.g written again is the one that ran: True
r_square_15.g written again is the one that ran: True
r_square_one.g written again is the one that ran: True
vector1.g written again is the one that ran: True
vector2.g written again is the one that ran: True
.g files that ran and are neither appended nor written again: []
```

**9. What this leaves.** Rows, filed from this write-up when it merges, each
from Q2690:

- Small: cite this subsection in `NORMAL_LENGTH_ROUNDOFF`'s doc
  (`plane.rs:114-165`), whose "One share is not derived, and is measured
  instead" is no longer so given the turn's length: Gappa derives the share,
  `4(q·q − 1)|v × n|²`, within `24ε` for `|q·q − 1| ≤ 6ε`, and the doc's own
  `63.4ε` does not hold it, where Gappa's `59.64ε` does (4).
- Run `vector.g` and `square.g` in the zero branches: a normal along an axis
  has two components zero, a turn about an axis two of `v`, and a sum may
  cancel to zero. These are common inputs. Not run, so no class (§360.5). It
  waits on Q2801, which runs `point.g` there, since `vector.g` is `point.g`
  with no shift and can follow that run's split.
- Whether a normal that reaches a turn other than from a constructor stays
  within the bound: one `from_point_and_normal_as_written` kept at up to
  `64ε`, or a plane stored after one turn and turned again. `square.g`
  assumes the constructor's `ε` before the turn. Not run, so no class.

And what is not a row: the hypotheses 464.19 already leaves, the nine
products below the normals (Q2802) and the route through
`From<na::Isometry3<f64>>` (Q2803), which `vector.g` shares with `point.g`;
Q2700, which moves these scripts into `proofs/gappa/` with 464.19's.

### 464.21 CORNER_AFTER_MOVING_ROUNDOFF

Q2691 proved `CORNER_AFTER_MOVING_ROUNDOFF` in Gappa on 2026-10-26 at
`f95bb965`, on 464.19's `point.g` and 464.20's `vector.g`, for four of the
five kinds of surface. A corner exactly on a plane, a ball, a pipe or a torus,
once moved, is within `222.5u · L`, `148.7u · L`, `230.6u · L` and `213.0u · L`
of the moved surface as the code stores it, and the box the code draws round
the moved corner meets that surface whenever it is within `256u · L` (1, 3).
The pipe gets there only with the slips at the `25u` that `point.g` reaches,
not at `POINT_ROUNDOFF`'s `36u`. The cone is not proved: the same argument, on
paper, comes to about `290u`, past the bound, and nobody has searched for a
corner that gets there (3). The doc's own count miscounts in three ways (4).
Coq checks the four proofs, modulo their hints (7).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, unchanged, against `moved_corner.rs`, `plane.rs`,
`sphere.rs`, `cylinder.rs`, `torus.rs`, `point.rs` and `isometry.rs` at
`f95bb965` and nalgebra 0.35.0. `u` is `2⁻⁵³` and `ε` is `f64::EPSILON = 2u`.
Every length the scripts work with is per unit of `L`.

**1. What the proof guards.** `where_a_corner_lands_after_moving`
(`moved_corner.rs:208-244`) moves a corner by `Isometry::apply_to_point` and,
unless the move rounded nothing, hands back a box round the moved corner of
half-width `CORNER_AFTER_MOVING_ROUNDOFF · L` on each coordinate. `L` is
`the_largest_number_involved` (`moved_corner.rs:423-441`): the largest
magnitude among the corner before and after the move, the shift, and what
`how_big_a_surface_is` (`moved_corner.rs:443-468`) gives for each surface
before and after — a plane's offset, a ball's centre and radius, a pipe's
axis origin and radius, a cone's apex and slope, a torus's centre and `ring +
tube`. The box is to meet the moved surface as the code stores it,
`Surface::transformed`. It does whenever the moved corner is within `128ε · L
= 256u · L` of that surface, since a ball of that radius lies inside the box,
and that distance is each script's goal. The doc (`moved_corner.rs:38-97`)
counts five `POINT_ROUNDOFF`s for the worst kind, a plane, and publishes
seven.

**2. The scripts.** Each script takes the corner exactly on the surface,
bounds a few shares of the moved corner's distance from the moved surface,
and proves their sum within `256u`. What joins the shares to the distance is
done by hand, below, and `sos.py` (8) checks the six identities the scripts
rest on, in exact rationals at 1000 random points each. `Ex` is nalgebra's
turn worked without rounding. It is linear (identity 1), so a moved corner
less a moved centre or axis origin is `Ex(y) + k`, with `y` the corner less
the centre or origin and `k = kp − ko` the two slips `point.g` proves.

- `ball.g`: `Sphere::transformed` (`sphere.rs:78`) moves the centre and
  carries the radius `r` over. By the triangle inequality the distance is at
  most `|k| + r · ||Ex(y / r)| − 1|`, and the second share is the turn's
  length share of 464.20's 3, for a unit vector.
- `plane.g`: `Plane::transformed` (`plane.rs:544`) turns the normal to `n' =
  Ex(n) + dn`, `dn` what `vector.g` proves, and gives the offset `off' = off +
  n'·s + dd` by `shiftplane.g`'s arithmetic (464.9). With the moved corner
  `Ex(x) + s + dp`, identity 2 makes `n'·p' − off'` exactly `4(q·q − 1)(v ×
  n)·(v × x) + dn·Ex(x) + n'·dp − dd`, and the distance is that over `|n'|`.
  Gappa bounds the whole quotient, and no inequality is taken by hand.
- `pipe.g` and `torus.g`: `Cylinder::transformed` (`cylinder.rs:74`) and
  `Torus::transformed` (`torus.rs:152`) carry the radii over and move the axis
  by `Axis::transformed` (`point.rs:784`): the origin by `apply_to_point`, and
  the direction by `apply_to_vector`, to `Ex(a) + da`. With `e` the unit
  `Ex(a)` and `θ` the angle `da` tilts it by, the distance is at most `|k| +
  θ · |Ex(y)| + |(ρ' − ρ, t' − t)|`, by three steps by hand:
  - moving a point by `k` moves its distance from a surface by at most `|k|`;
  - turning the axis by `θ` about the moved origin moves a point `|Ex(y)|`
    from it by at most `θ · |Ex(y)|`, and `θ ≤ tan θ ≤ |da| / (|Ex(a)| −
    |da|)`, which makes the scripts' `TI`;
  - with `(ρ, t)` how far the corner is from the axis and along it, and `(ρ',
    t')` the same of `Ex(y)` about `e`, the distance from the surface in the
    plane of across and along is 1-Lipschitz. For a pipe it is `|ρ − r|`,
    which leaves `|ρ' − ρ|`, `pipe.g`'s `DR`. For a torus it is `||(ρ − R, t)|
    − r|`, and `|(ρ' − R, t')| − |(ρ − R, t)| ≤ |(ρ' − ρ, t' − t)|`,
    `torus.g`'s `DR`.

  Identities 5 and 6 let the two scripts put the axis along `z` and the corner
  in the `xz` plane. `Ex` turns with the frame (identity 5), so there `w` and
  `v` are the turn's quaternion in the turned frame, reals with `q·q`
  unchanged, and `ρ' = ρ · sqrt(NUMQ / DENQ)` and `t' = t · sqrt(DENQ) −
  4(q·q − 1)v₀v₂ρ / sqrt(DENQ)` (identity 6). Turning the frame turns `k` and
  `da` too, and that is sound because the two enter the scripts only by their
  lengths, `S = sqrt(KK)` and `DA = sqrt(DADA)`. `point.g`'s and `vector.g`'s
  boxes, on each coordinate in the world's frame, bound `|k|` by `√3` times
  twice a slip's bound and `|da|` by `√3 · 20u`, and the scripts' boxes, read
  in the turned frame, hold a `k` and a `da` of every such length.

**3. The shares.** Gappa's bounds, as each script printed them:

- ball, at `36u` slips: the slips' `S` `1.38453e-14` (`124.7u`, `√3 · 72u`)
  and the radius's `T` `2.66454e-15` (`24.0u`), so `148.7u`;
- plane, at `36u`: the turn's length share `T1` within `±9.23022e-15`
  (`83.14u`), the normal's slip against the turned corner `T2` within
  `±6.66134e-15` (`60u`), the corner's slip against the turned normal `T3`
  within `±6.92267e-15` (`62.35u`), and the offset's `17u` taken from
  `shiftplane.g`'s goal, all over `|n'|`, with `n'·n' − 1` within
  `±2.72318e-14` (`245.3u`): about `222.5u`, where `c_plane_222.g` finds the
  best `±2.47016e-14`;
- pipe, at `25u`: `S` `9.61481e-15` (`86.6u`), the tilt `TI` `1.33227e-14`
  (`120.0u`, `2√3 · √3 · 20u`) and the drift `DR` `2.66454e-15` (`24.0u`), so
  `230.6u`; at `36u`, `r_pipe_36.g` fails with best `2.98325e-14`, `268.7u`;
- torus, at `36u`: `S` `1.38453e-14` (`124.7u`), `TI` `3.84593e-15`
  (`34.6u`) and `DR` `5.95808e-15` (`53.7u`), from `DRr` within
  `±2.66454e-15` and `DRt` within `±5.32907e-15`, so `213.0u`.

A torus's tilt and drift are small because its corner is within about `L` of
its centre, where a pipe's can be `2√3 L` from its origin. Each axis script
also proves `DENQ − 1` within `±5.32907e-15`, `48u`, which keeps its one
divisor off zero.

The cone is on paper and was not run. `Cone::transformed` (`cone.rs:156-161`)
is `Axis::transformed` too, and its distance in the plane of across and along
is `|ρ − st| / h`, `h = sqrt(1 + s²)`, also 1-Lipschitz. To first order in `η
= q·q − 1`, identity 6's `ρ'` and `t'` take a corner on the cone off it by
`2ηρ(v₂² − v₀² + 2s v₀v₂) / h`. The form's eigenvalues are `±h`, since `h(v₀² +
v₂²)` less it is `((h + 1)v₀ − s v₂)² / (h + 1)` and `h(v₀² + v₂²)` plus it is
`((h + 1)v₂ + s v₀)² / (h + 1)`, so the drift is at most `2|η|ρ · q·q`, about
`24u · ρ`. A cone's `L` counts its apex and slope and not how far it reaches
(`moved_corner.rs:456-460`), so `ρ ≤ |y| ≤ 2√3 L` and the drift reaches
`83.1u`. With the pipe's `86.6u` and `120u` that is about `290u`, past `256u`,
even at `25u` slips. That is an argument and not a demonstration that the
bound fails (§16.10): the three shares need not reach their worst at once.
The test that searches,
`a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands.rs`,
finds about `5ε` (its lines 44-46), and nobody has searched for a corner on a
cone past `256u`. One of the rows 9 files is that search.

**4. What the doc counts wrong.** The doc (`moved_corner.rs:38-97`) counts
one `POINT_ROUNDOFF · L` for the point, one more for a ball, three for a pipe
and four for a plane, and takes the plane's four and the point's one, `90ε ·
L`, as the worst. The 128 holds for the four kinds Gappa proved, but not by
that count, which is wrong in three ways:

- It treats `POINT_ROUNDOFF`, a bound on each coordinate, as a bound on a
  length. `point.g` bounds each coordinate of a slip by `36u · L`, so a slip
  is up to `√3 · 36u · L = 62.35u · L` long, and the two slips a ball's
  distance adds up to `124.7u · L`, where the doc gives them `72u · L`.
- It leaves the turn's length share out. Even worked without rounding,
  nalgebra's turn keeps a length only when `q·q` is exactly one (464.20's 3),
  and `|q·q − 1|` can be `6ε`. That share is `83.14u` for a plane, `24u` for a
  ball, the drift `DR`, `24u` for a pipe and `53.7u` for a torus, and on paper
  about `24u · ρ` for a cone.
- Its worst is not the plane. On the counts Gappa proved, the pipe's `230.6u`
  is over the plane's `222.5u`, and the pipe gets there only with `point.g`'s
  `25u` slips: with `POINT_ROUNDOFF`'s `36u` its argument comes to `268.7u`.
  The doc's "a point at most `2L` away" misses that `|y|` can reach `2√3 L`,
  which, with `vector.g`'s `20u` on each component, makes the tilt `120u`
  where the doc has `72u`. The cone's `290u` on paper is past them both.

**5. The hypotheses.** Each script takes `point.g`'s goal as the bound on its
slips, and each but `ball.g` takes `vector.g`'s as the bound on its turned
normal or direction, so they stand on `point.g`'s hypotheses (464.19's 5) and
`vector.g`'s (464.20's 5). Their own, in each script's order:

- in every script, a stored turn's `|q·q − 1| ≤ 6ε`, worked out exactly, as
  464.19's 5 has it; in `pipe.g` and `torus.g`, each of `|w|` and `|vᵢ|` at
  most two, since each square is at most `q·q` in any frame;
- `ball.g`: `y` a unit vector, each slip coordinate within `36u`, and `L` at
  least the radius;
- `plane.g`: the stored normal's components at most one and `n·n` within
  `NORMAL_LENGTH_ROUNDOFF`, `64ε`, of one; the corner's coordinates at most
  `L`; `dn` within `vector.g`'s `20u`; the corner's slip within `36u`; and the
  offset's rounding within `shiftplane.g`'s `16u` times the larger of `|n'|`
  and one, under `17u`;
- `pipe.g`: the stored direction's `a·a` within `64ε` of one; `L` at least the
  radius, and `|y| ≤ 2√3 L`, since `L` is at least every coordinate of the
  corner and of the origin; the six slip coordinates within `25u`; and `da`
  within `vector.g`'s `20u`. The `25u` rests on `r_point_25.g`,
  `r_point1_25.g` and `r_point2_25.g`, 464.19's `point.g` and its `y` and `z`
  with the goal at `25u`, which prove under `point.g`'s own hypotheses with
  `D / L` at most `24.8`;
- `torus.g`: the axis as `pipe.g`'s; `|y| / L ≤ 1 + u`, since the corner is at
  most `ring + tube` from the centre and `L` is at least `ring + tube` as
  `f64` adds them (`moved_corner.rs:463-466`), taken with room as `ρ / L` and
  `|t| / L` at most `1 + 2u` and `|y|² / L²` at most `1 + 8u`; and the slips
  within `36u`. The draft before this run took `|y| / L ≤ 1`, which the
  rounded sum does not promise; every bound it printed is unchanged to the six
  digits Gappa prints.

Two hypotheses are left unproved:

- `vector.g` takes each component of the vector it turns at most one. A
  normal a constructor stores has that, but a stored axis direction is only
  within `64ε` of length one, so a component can be about `1 + 64u`, and
  `pipe.g` and `torus.g` take `da` from `vector.g` all the same. Q2826 asks the
  like of a normal, and one of the rows 9 files asks it of an axis.
- `point.g` and `vector.g` are proved away from their zero branches and with
  their products normal, as 464.19's 5 says; Q2801, Q2825 and Q2802 run them
  there.

**6. The controls.** Each changes one thing and exits 1. A goal just under
what the script reaches shows its hypotheses do not contradict each other,
and a hypothesis dropped shows the proof needs it.

- `c_ball_148.g`, the goal at `148u`: best `1.65099e-14`, `148.7u`;
- `c_ball_non4.g`, without the turn near length one: best `2.23899e+617`;
- `c_ball_nok.g`, without `point.g`'s slips: best `[-inf, inf]`, with `S`
  unproved;
- `c_plane_222.g`, the goal at `±222u`: best `±2.47016e-14`, `222.49u`;
- `c_plane_non4.g`, without the turn near length one: best `[-inf, inf]`;
- `c_plane_nodn.g`, without `vector.g`'s `dn`: best `[-inf, inf]`, with `T2`,
  `T3` and `n'·n' − 1` unproved;
- `c_plane_nodp.g`, without `point.g`'s slip: best `[-inf, inf]`, with `T3`
  unproved;
- `c_plane_nodd.g`, without `shiftplane.g`'s offset: best `[-inf, inf]`;
- `c_pipe_230.g`, the goal at `230u`: best `2.5602e-14`, `230.6u`;
- `c_pipe_non4.g`, `q·q` only within `[0, 16]`, what the bounds on `w` and `v`
  allow: best `[-inf, inf]`, with `TI`, `DR` and `NUMQ / DENQ − 1` unproved
  and `DENQ − 1` within `[-1, 480]`;
- `c_pipe_nobound.g`, without `|w|` and each `|vᵢ|` at most two, the first run
  of `pipe.g`: best `[-inf, inf]`, with every share but `S` unproved;
- `c_pipe_noda.g`, without `vector.g`'s `da`: best `[-inf, inf]`, with `TI`
  unproved;
- `c_torus_212.g`, the goal at `212u`: best `2.36493e-14`, `213.0u`;
- `c_torus_non4.g`, `q·q` only within `[0, 16]`: best `[-inf, inf]`, with
  `TI`, `DRr`, `DRt`, `DR` and `NUMQ / DENQ − 1` unproved.

`r_pipe_36.g` is a run and not a control: the pipe with the slips at `36u`,
best `2.98325e-14`, `268.7u` (3).

**7. Coq checks the proofs, modulo their hints.** `gappa -Bcoq` and `coqc`,
through `one.sh`, exit 0 on `ball.g`, `plane.g`, `pipe.g`, `torus.g`,
`r_point_25.g`, `r_point1_25.g` and `r_point2_25.g`. `coqc` printed nothing,
and `gappa -Bcoq` only its results and warnings. On the last run of `all.sh`
(8) the `.v` files came to 1347 lines with 3 hints, 8504 with 17, 3422 with 8,
3556 with 11, and 8222, 8162 and 8195 with 34 each. The lines are that run's
and no other's (464.20's 7): the run before it, of the same `ball.g`,
`plane.g`, `pipe.g` and `r_point` scripts, wrote 1347, 8503, 3280, 8215, 8244
and 8199, with the same hints.

`pipe.g` and `torus.g` needed `a` and `y` written in place. With the names
`a0 = 0; a1 = 0; a2 = al;` and `y0 = RL; y1 = 0; y2 = TL;`, `gappa -Bcoq`
stopped after it had written every hint, with ``src/parser/ast.cpp:255:
std::string dump_real(const ast_real*, unsigned int): Assertion `false'
failed``. `one.sh` printed `pipe coq -Bcoq exit 134 coqc exit 1 1.74s lines 166
hints 8`, and the torus the same with 188 lines and 11 hints, and `coqc` said
`Error: The section Generated_by_Gappa needs to be closed.` Two variants of
`pipe.g` were run. `p_inline.g`, with the six names dropped and each written
in place, printed `p_inline exit 0 0.36s` and `p_inline coq -Bcoq exit 0 coqc
exit 0 2.31s lines 3280 hints 8`, with the same six bounds. `p_named.g`, meant
to put the hint on `NUMQ / DENQ` on a name of its own, tested nothing: it
defined that name from `NUMQ` above `NUMQ`'s own line, and Gappa stopped with
`Error: redefinition of identifier 'NUMQ' at line 62 column 4`. `pipe.g` and
`torus.g` in 8 are the in-place form. Which of the six names `-Bcoq` trips on
was not looked for.

As in 464.19's 8, each hint enters the Coq file as a hypothesis with no
condition, so a proof stands on its hints and on the divisors Gappa took to be
nonzero. `grep -c 'assumed to be nonzero'` counts 0 in `ball.out` and
`plane.out`; 2 each in `pipe.out` and `torus.out`, for `DENQ`, which the two
scripts prove within `48u` of one; and 163 in each `r_point` run's. None says a
hint's two sides are "not trivially equal". `plane.out` says there is "no need
for the rewriting at line 114 column 13" and "at line 126 column 13", the
hints `T2 -> F2 / 2` and `T3 -> F3 / 2`. The controls and `r_pipe_36.g` ran
without `-Bcoq`.

**8. The scripts, whole, as they ran.** To run them again, save `ball.g`,
`plane.g`, `pipe.g`, `torus.g`, `controls.py` and `all.sh` in one directory
with 464.13's `one.sh` and 464.19's `point.g`, and run `bash all.sh` there.
`controls.py` writes the controls, `r_pipe_36.g` and the three runs at `25u`
from them. The longest run, `r_point2_25.g` with `coqc`, took 102.15 s, as
`all.sh`'s print shows. `sos.py` and `same.py` are checks, and `all.sh` needs
neither.

`ball.g`:

```gappa
# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/sphere.rs transformed at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a ball,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner x exactly on a
# ball of centre c and radius r, moved, is within 128 * f64::EPSILON * L of
# the moved ball. Sphere::transformed moves the centre by apply_to_point and
# carries r over. Ex, nalgebra's turn without rounding, is linear (sos.py),
# so the moved corner less the moved centre is Ex(x - c) + kp - kc, kp and kc
# the two slips point.g proves. By the triangle inequality the distance to
# the moved ball is at most |kp - kc| + r * | |Ex(y)| - 1 |, y = (x - c) / r,
# which is what this bounds, per unit of L.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;

# nalgebra's turn of the unit vector y without rounding, as point.g's Ex
# writes x, and its length squared.
Ex0 = 2 * w * (v1 * y2 - v2 * y1)
    + v1 * (2 * (v0 * y1 - v1 * y0)) - v2 * (2 * (v2 * y0 - v0 * y2)) + y0;
Ex1 = 2 * w * (v2 * y0 - v0 * y2)
    + v2 * (2 * (v1 * y2 - v2 * y1)) - v0 * (2 * (v0 * y1 - v1 * y0)) + y1;
Ex2 = 2 * w * (v0 * y1 - v1 * y0)
    + v0 * (2 * (v2 * y0 - v0 * y2)) - v1 * (2 * (v1 * y2 - v2 * y1)) + y2;
EE = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;
YY = y0 * y0 + y1 * y1 + y2 * y2;

# v x y and v . y, without rounding, and v.v.
X0 = v1 * y2 - v2 * y1; X1 = v2 * y0 - v0 * y2; X2 = v0 * y1 - v1 * y0;
VY = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * y0 + v1 * y1 + v2 * y2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

# The two slips, the moved corner's and the moved centre's, per unit of L.
k0 = kp0 - kc0; k1 = kp1 - kc1; k2 = kp2 - kc2;
KK = k0 * k0 + k1 * k1 + k2 * k2;

# The distance's two shares, per unit of L: the slips', and the radius's
# times how far the turn's arithmetic takes a unit vector off length one.
S = sqrt(KK);
T = RL * |sqrt(EE) - 1|;

{
  # Every quaternion component is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # y is the corner less the centre over r, a unit vector.
  /\ YY in [1, 1]
  /\ y0 in [-1, 1] /\ y1 in [-1, 1] /\ y2 in [-1, 1]
  # What point.g, point1.g and point2.g prove of the moved corner and the
  # moved centre, L at least every coordinate of either and of the shift.
  /\ kp0 in [-36b-53, 36b-53] /\ kp1 in [-36b-53, 36b-53]
  /\ kp2 in [-36b-53, 36b-53]
  /\ kc0 in [-36b-53, 36b-53] /\ kc1 in [-36b-53, 36b-53]
  /\ kc2 in [-36b-53, 36b-53]
  # r / L: L is at least the radius.
  /\ RL in [0, 1]
->
  EE - 1 in ? /\ S in ? /\ T in ?
  /\ S + T in [0, 256b-53]
}

# |Ex(y)|^2 is |y|^2 and the turn's length share (464.20's 3), and |v x y|^2
# is at least zero, a sum of squares, and at most v.v y.y; v.v is q.q less w^2.
EE -> YY + 4 * (N4 - 1) * VY;
VY -> VV * YY - VD * VD;
VV -> N4 - w * w;
```

`plane.g`:

```gappa
# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/plane.rs transformed at f95bb965
# restates: crates/rcad-geom/src/plane.rs NORMAL_LENGTH_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a plane,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner x exactly on
# the plane n.x = off, moved, is within 128 * f64::EPSILON * L of the moved
# plane. Plane::transformed turns the normal, n' = Ex(n) + dn, and gives the
# offset off' = off + n'.s + dd by shiftplane.g's arithmetic; the moved
# corner is Ex(x) + s + dp. Then n'.p' - off' is
#   4 (q.q - 1) (v x n) . (v x x) + dn . Ex(x) + n' . dp - dd,
# with Ex nalgebra's turn without rounding (sos.py), and the distance is that
# over |n'|. Every length is per unit of L, which divides nothing that rounds.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v), each an f64, and q.q worked out exactly.
w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;
VV = v0 * v0 + v1 * v1 + v2 * v2;

# The stored normal n, and the corner over L.
NN = n0 * n0 + n1 * n1 + n2 * n2;
XX = X0 * X0 + X1 * X1 + X2 * X2;

# nalgebra's turn of n and of the corner without rounding, as vector.g's Ex
# writes x.
An0 = 2 * w * (v1 * n2 - v2 * n1)
    + v1 * (2 * (v0 * n1 - v1 * n0)) - v2 * (2 * (v2 * n0 - v0 * n2)) + n0;
An1 = 2 * w * (v2 * n0 - v0 * n2)
    + v2 * (2 * (v1 * n2 - v2 * n1)) - v0 * (2 * (v0 * n1 - v1 * n0)) + n1;
An2 = 2 * w * (v0 * n1 - v1 * n0)
    + v0 * (2 * (v2 * n0 - v0 * n2)) - v1 * (2 * (v1 * n2 - v2 * n1)) + n2;
E0 = 2 * w * (v1 * X2 - v2 * X1)
   + v1 * (2 * (v0 * X1 - v1 * X0)) - v2 * (2 * (v2 * X0 - v0 * X2)) + X0;
E1 = 2 * w * (v2 * X0 - v0 * X2)
   + v2 * (2 * (v1 * X2 - v2 * X1)) - v0 * (2 * (v0 * X1 - v1 * X0)) + X1;
E2 = 2 * w * (v0 * X1 - v1 * X0)
   + v0 * (2 * (v2 * X0 - v0 * X2)) - v1 * (2 * (v1 * X2 - v2 * X1)) + X2;
AnAn = An0 * An0 + An1 * An1 + An2 * An2;
EE = E0 * E0 + E1 * E1 + E2 * E2;

# v x n and v x x, v . n and v . x, without rounding.
a0 = v1 * n2 - v2 * n1; a1 = v2 * n0 - v0 * n2; a2 = v0 * n1 - v1 * n0;
b0 = v1 * X2 - v2 * X1; b1 = v2 * X0 - v0 * X2; b2 = v0 * X1 - v1 * X0;
VN = a0 * a0 + a1 * a1 + a2 * a2;
VX = b0 * b0 + b1 * b1 + b2 * b2;
VDn = v0 * n0 + v1 * n1 + v2 * n2;
VDx = v0 * X0 + v1 * X1 + v2 * X2;

# The turned normal as the code stores it, and the slips.
np0 = An0 + dn0; np1 = An1 + dn1; np2 = An2 + dn2;
NPNP = np0 * np0 + np1 * np1 + np2 * np2;
DNDN = dn0 * dn0 + dn1 * dn1 + dn2 * dn2;
DPDP = DP0 * DP0 + DP1 * DP1 + DP2 * DP2;

# The four shares of n'.p' - off', per unit of L, and twice the three dot
# products they and n'.n' are written with below.
T1 = 4 * (N4 - 1) * (a0 * b0 + a1 * b1 + a2 * b2);
T2 = dn0 * E0 + dn1 * E1 + dn2 * E2;
T3 = np0 * DP0 + np1 * DP1 + np2 * DP2;
T4 = -DD;
F1 = 2 * (a0 * b0 + a1 * b1 + a2 * b2);
F2 = 2 * (dn0 * E0 + dn1 * E1 + dn2 * E2);
F3 = 2 * (np0 * DP0 + np1 * DP1 + np2 * DP2);
F4 = 2 * (An0 * dn0 + An1 * dn1 + An2 * dn2);
T = T1 + T2 + T3 + T4;

{
  # Every quaternion component is a finite f64.
     |w| <= 1b1024 /\ |v0| <= 1b1024 /\ |v1| <= 1b1024 /\ |v2| <= 1b1024
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53.
  /\ N4 in [9007199254740980b-53, 9007199254741004b-53]
  # A stored normal has every component at most one and n.n within
  # NORMAL_LENGTH_ROUNDOFF, 64 epsilon, of one.
  /\ n0 in [-1, 1] /\ n1 in [-1, 1] /\ n2 in [-1, 1]
  /\ NN in [9007199254740864b-53, 9007199254741120b-53]
  # L is at least every coordinate of the corner.
  /\ X0 in [-1, 1] /\ X1 in [-1, 1] /\ X2 in [-1, 1]
  # What vector.g, vector1.g and vector2.g prove of the turned normal.
  /\ dn0 in [-20b-53, 20b-53] /\ dn1 in [-20b-53, 20b-53]
  /\ dn2 in [-20b-53, 20b-53]
  # What point.g, point1.g and point2.g prove of the moved corner, over L.
  /\ DP0 in [-36b-53, 36b-53] /\ DP1 in [-36b-53, 36b-53]
  /\ DP2 in [-36b-53, 36b-53]
  # What shiftplane.g proves of the offset's rounding over L, 16u times
  # the larger of |n'| and one, with |n'| within 124u of one: under 17u.
  /\ DD in [-17b-53, 17b-53]
->
  T1 in ? /\ T2 in ? /\ T3 in ? /\ NPNP - 1 in ?
  /\ T / sqrt(NPNP) in [-256b-53, 256b-53]
}

# The turn's length share: Ex(n) . Ex(x) - n . x is 4 (q.q - 1)
# (v x n) . (v x x) (sos.py), and 2ab is c a^2 + b^2 / c less (c a - b)^2 / c,
# and (c a + b)^2 / c less c a^2 + b^2 / c, with c near sqrt(3), which makes
# c |v x n|^2 + |v x x|^2 / c smallest for n.n near one and x.x at most 3.
T1 -> 2 * (N4 - 1) * F1;
F1 -> (1732/1000 * VN + VX / (1732/1000))
      - ((1732/1000 * a0 - b0) * (1732/1000 * a0 - b0)
       + (1732/1000 * a1 - b1) * (1732/1000 * a1 - b1)
       + (1732/1000 * a2 - b2) * (1732/1000 * a2 - b2)) / (1732/1000);
F1 -> ((1732/1000 * a0 + b0) * (1732/1000 * a0 + b0)
     + (1732/1000 * a1 + b1) * (1732/1000 * a1 + b1)
     + (1732/1000 * a2 + b2) * (1732/1000 * a2 + b2)) / (1732/1000)
      - (1732/1000 * VN + VX / (1732/1000));
VN -> VV * NN - VDn * VDn;
VX -> VV * XX - VDx * VDx;
VV -> N4 - w * w;

# The normal's slip against the turned corner, c = 20u.
T2 -> F2 / 2;
F2 -> (20b-53 * EE + DNDN / 20b-53)
      - ((20b-53 * E0 - dn0) * (20b-53 * E0 - dn0)
       + (20b-53 * E1 - dn1) * (20b-53 * E1 - dn1)
       + (20b-53 * E2 - dn2) * (20b-53 * E2 - dn2)) / 20b-53;
F2 -> ((20b-53 * E0 + dn0) * (20b-53 * E0 + dn0)
     + (20b-53 * E1 + dn1) * (20b-53 * E1 + dn1)
     + (20b-53 * E2 + dn2) * (20b-53 * E2 + dn2)) / 20b-53
      - (20b-53 * EE + DNDN / 20b-53);
EE -> XX + 4 * (N4 - 1) * VX;

# The corner's slip against the turned normal, c = sqrt(3) * 36u.
T3 -> F3 / 2;
F3 -> (62354b-53 / 1000 * NPNP + DPDP / (62354b-53 / 1000))
      - ((62354b-53 / 1000 * np0 - DP0) * (62354b-53 / 1000 * np0 - DP0)
       + (62354b-53 / 1000 * np1 - DP1) * (62354b-53 / 1000 * np1 - DP1)
       + (62354b-53 / 1000 * np2 - DP2) * (62354b-53 / 1000 * np2 - DP2))
        / (62354b-53 / 1000);
F3 -> ((62354b-53 / 1000 * np0 + DP0) * (62354b-53 / 1000 * np0 + DP0)
     + (62354b-53 / 1000 * np1 + DP1) * (62354b-53 / 1000 * np1 + DP1)
     + (62354b-53 / 1000 * np2 + DP2) * (62354b-53 / 1000 * np2 + DP2))
        / (62354b-53 / 1000)
      - (62354b-53 / 1000 * NPNP + DPDP / (62354b-53 / 1000));

# n'.n' as square.g writes it: n.n, the turn's length share, 2 Ex(n) . dn
# with c = sqrt(3) * 20u, and dn . dn.
NPNP -> AnAn + F4 + DNDN;
AnAn -> NN + 4 * (N4 - 1) * VN;
F4 -> (34641b-53 / 1000 * AnAn + DNDN / (34641b-53 / 1000))
      - ((34641b-53 / 1000 * An0 - dn0) * (34641b-53 / 1000 * An0 - dn0)
       + (34641b-53 / 1000 * An1 - dn1) * (34641b-53 / 1000 * An1 - dn1)
       + (34641b-53 / 1000 * An2 - dn2) * (34641b-53 / 1000 * An2 - dn2))
        / (34641b-53 / 1000);
F4 -> ((34641b-53 / 1000 * An0 + dn0) * (34641b-53 / 1000 * An0 + dn0)
     + (34641b-53 / 1000 * An1 + dn1) * (34641b-53 / 1000 * An1 + dn1)
     + (34641b-53 / 1000 * An2 + dn2) * (34641b-53 / 1000 * An2 + dn2))
        / (34641b-53 / 1000)
      - (34641b-53 / 1000 * AnAn + DNDN / (34641b-53 / 1000));
```

`sos.py` printed:

```text
identities 1 to 4 hold at 1000 random points
identity 5 holds at 1000 random points
identity 6 holds at 1000 random points
```

`sos.py`:

```python
"""Check, in exact rationals at random points, the identities 464.21's scripts
rest on, for nalgebra's turn worked without rounding, Ex(y) = t w + v x t + y
with t = 2 (v x y), N4 = q.q and eta = N4 - 1:

1. Ex is linear: Ex(a) - Ex(b) = Ex(a - b), so a shift added after it cancels
   from a difference of two moved points exactly.
2. Ex(a) . Ex(b) - a . b = 4 eta (v x a) . (v x b), 464.20's identity polarized.
3. |Ex(y) x Ex(a)|^2 = |W|^2 + 4 eta (|v|^2 |W|^2 + (v . W)^2)
                       + 16 eta^2 |v|^2 (v . W)^2, with W = y x a.
4. (v x a) . (v x b) = (v . v)(a . b) - (v . a)(v . b), Lagrange's identity,
   which with a = b is 464.20's |v x n|^2 = |v|^2 |n|^2 - (v . n)^2.
5. Ex turns with the frame: for a rotation G, Ex with v turned to G v, of
   G y, is G Ex(y). So pipe.g and torus.g may put the axis along z and the
   corner in the xz plane.
6. In that frame, a = (0, 0, al) and y = (rho, 0, t), with
   DENQ = 1 + 4 eta (v0^2 + v1^2) and NUMQ = 1 + 4 eta (v.v + v1^2)
   + 16 eta^2 (v.v) v1^2:
   |Ex(a)|^2 = al^2 DENQ, |Ex(y) x Ex(a)|^2 = rho^2 al^2 NUMQ, and
   Ex(y) . Ex(a) = al (t DENQ - 4 eta v0 v2 rho)."""

from fractions import Fraction as F
import random

random.seed(1)


def r():
    return F(random.randint(-10**6, 10**6), 10**6)


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def ex(w, v, y):
    t = [2 * c for c in cross(v, y)]
    k = cross(v, t)
    return [t[i] * w + k[i] + y[i] for i in range(3)]


for _ in range(1000):
    w, v = r(), [r(), r(), r()]
    a, b = [r(), r(), r()], [r(), r(), r()]
    eta = w * w + dot(v, v) - 1
    assert [p - q for p, q in zip(ex(w, v, a), ex(w, v, b))] == ex(w, v, [p - q for p, q in zip(a, b)])
    assert dot(ex(w, v, a), ex(w, v, b)) - dot(a, b) == 4 * eta * dot(cross(v, a), cross(v, b))
    W = cross(a, b)
    big = cross(ex(w, v, a), ex(w, v, b))
    assert dot(big, big) == (dot(W, W) + 4 * eta * (dot(v, v) * dot(W, W) + dot(v, W) ** 2)
                             + 16 * eta * eta * dot(v, v) * dot(v, W) ** 2)
    assert dot(cross(v, a), cross(v, b)) == dot(v, v) * dot(a, b) - dot(v, a) * dot(v, b)
print("identities 1 to 4 hold at 1000 random points")


def rotation(g):
    """The rotation matrix of the rational quaternion g, which is rational."""
    s, x, y, z = g
    n = s * s + x * x + y * y + z * z
    return [[(s * s + x * x - y * y - z * z) / n, 2 * (x * y - s * z) / n, 2 * (x * z + s * y) / n],
            [2 * (x * y + s * z) / n, (s * s - x * x + y * y - z * z) / n, 2 * (y * z - s * x) / n],
            [2 * (x * z - s * y) / n, 2 * (y * z + s * x) / n, (s * s - x * x - y * y + z * z) / n]]


def apply(m, y):
    return [dot(row, y) for row in m]


for _ in range(1000):
    w, v, y = r(), [r(), r(), r()], [r(), r(), r()]
    G = rotation([r(), r(), r(), r()])
    assert dot(apply(G, [1, 0, 0]), apply(G, [0, 1, 0])) == 0
    assert dot(apply(G, [1, 0, 0]), apply(G, [1, 0, 0])) == 1
    assert ex(w, apply(G, v), apply(G, y)) == apply(G, ex(w, v, y))
print("identity 5 holds at 1000 random points")

for _ in range(1000):
    w, v = r(), [r(), r(), r()]
    al, rho, t = r(), r(), r()
    eta = w * w + dot(v, v) - 1
    a, y = [0, 0, al], [rho, 0, t]
    DENQ = 1 + 4 * eta * (v[0] ** 2 + v[1] ** 2)
    NUMQ = 1 + 4 * eta * (dot(v, v) + v[1] ** 2) + 16 * eta * eta * dot(v, v) * v[1] ** 2
    ea, ey = ex(w, v, a), ex(w, v, y)
    assert dot(ea, ea) == al * al * DENQ
    c = cross(ey, ea)
    assert dot(c, c) == rho * rho * al * al * NUMQ
    assert dot(ey, ea) == al * (t * DENQ - 4 * eta * v[0] * v[2] * rho)
print("identity 6 holds at 1000 random points")
```

`pipe.g`:

```gappa
# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/cylinder.rs transformed at f95bb965
# restates: crates/rcad-geom/src/point.rs transformed at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a pipe,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner exactly on a
# pipe of radius r about the line through o along a, moved, is within
# 128 * f64::EPSILON * L of the moved pipe. Cylinder::transformed carries r
# over, and Axis::transformed moves o by apply_to_point and turns a by
# apply_to_vector. With y the corner less o, the moved corner less the moved
# origin is Ex(y) + k, k the two slips point.g proves, and the moved
# direction is Ex(a) + da, da the slip vector.g proves, Ex nalgebra's turn
# without rounding (sos.py). By hand, the distance to the moved pipe is at
# most |k| + |Ex(y)| tan(th) + | |Ex(y) x e| - r |, th the angle da tilts
# Ex(a) by and e the unit Ex(a), which is what this bounds, per unit of L.
# Ex turns with the frame (sos.py), so here a is along z and y in the xz
# plane, and w and v are the turn's quaternion in that frame: reals, not f64.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) in that frame, q.q, v.v and two sums of its squares.
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;
VV = v0 * v0 + v1 * v1 + v2 * v2;
S01 = v0 * v0 + v1 * v1;
S12 = v1 * v1 + v2 * v2;

# The stored direction a is (0, 0, al), and y over L is (RL, 0, TL): RL is r
# over L, the corner's distance from the axis, and TL how far along it the
# corner is. Both are written in place below, coordinate by coordinate, since
# gappa -Bcoq aborts in dump_real when names hold them (464.21's 7).
AA = al * al;
YY = RL * RL + 0 * 0 + TL * TL;

# nalgebra's turn of y and of a without rounding, as point.g's Ex writes x,
# and their lengths squared.
Ex0 = 2 * w * (v1 * TL - v2 * 0)
    + v1 * (2 * (v0 * 0 - v1 * RL)) - v2 * (2 * (v2 * RL - v0 * TL)) + RL;
Ex1 = 2 * w * (v2 * RL - v0 * TL)
    + v2 * (2 * (v1 * TL - v2 * 0)) - v0 * (2 * (v0 * 0 - v1 * RL)) + 0;
Ex2 = 2 * w * (v0 * 0 - v1 * RL)
    + v0 * (2 * (v2 * RL - v0 * TL)) - v1 * (2 * (v1 * TL - v2 * 0)) + TL;
Ea0 = 2 * w * (v1 * al - v2 * 0)
    + v1 * (2 * (v0 * 0 - v1 * 0)) - v2 * (2 * (v2 * 0 - v0 * al)) + 0;
Ea1 = 2 * w * (v2 * 0 - v0 * al)
    + v2 * (2 * (v1 * al - v2 * 0)) - v0 * (2 * (v0 * 0 - v1 * 0)) + 0;
Ea2 = 2 * w * (v0 * 0 - v1 * 0)
    + v0 * (2 * (v2 * 0 - v0 * al)) - v1 * (2 * (v1 * al - v2 * 0)) + al;
EE = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;
EaEa = Ea0 * Ea0 + Ea1 * Ea1 + Ea2 * Ea2;

# v x y and v . y, without rounding.
X0 = v1 * TL - v2 * 0; X1 = v2 * RL - v0 * TL; X2 = v0 * 0 - v1 * RL;
VY = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * RL + v1 * 0 + v2 * TL;

# In this frame |Ex(a)|^2 is al^2 DENQ and |Ex(y) x Ex(a)|^2 is
# RL^2 al^2 NUMQ (sos.py), so the distance of Ex(y) from the turned axis is
# RL sqrt(NUMQ / DENQ).
DENQ = 1 + 4 * (N4 - 1) * S01;
NUMQ = 1 + 4 * (N4 - 1) * (VV + v1 * v1) + 16 * (N4 - 1) * (N4 - 1) * VV * (v1 * v1);

# The two slips, the moved corner's and the moved origin's, per unit of L,
# and the moved direction's.
k0 = kp0 - ko0; k1 = kp1 - ko1; k2 = kp2 - ko2;
KK = k0 * k0 + k1 * k1 + k2 * k2;
DADA = da0 * da0 + da1 * da1 + da2 * da2;
DA = sqrt(DADA);

# The distance's three shares, per unit of L: the slips', the tilt's, and
# how far the turn's arithmetic takes the corner off the pipe.
S = sqrt(KK);
TI = sqrt(EE) * DA / (sqrt(EaEa) - DA);
DR = RL * |sqrt(NUMQ / DENQ) - 1|;

{
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53. Turning the frame turns v and keeps q.q.
     N4 in [9007199254740980b-53, 9007199254741004b-53]
  # Each of w^2, v0^2, v1^2 and v2^2 is at most q.q, under 1 + 12u, so each
  # of w, v0, v1 and v2 is at most 2 in size.
  /\ |w| <= 2 /\ |v0| <= 2 /\ |v1| <= 2 /\ |v2| <= 2
  # A stored direction has a.a within NORMAL_LENGTH_ROUNDOFF, 64 epsilon, of
  # one.
  /\ AA in [9007199254740864b-53, 9007199254741120b-53]
  /\ al in [0, 2]
  # L is at least every coordinate of the corner and of the axis origin, and
  # the radius.
  /\ RL in [0, 1] /\ TL in [-4, 4] /\ YY in [0, 12]
  # What r_point_25.g, r_point1_25.g and r_point2_25.g prove of the moved
  # corner and the moved origin, L at least every coordinate of either and
  # of the shift: 25u.
  /\ kp0 in [-25b-53, 25b-53] /\ kp1 in [-25b-53, 25b-53]
  /\ kp2 in [-25b-53, 25b-53]
  /\ ko0 in [-25b-53, 25b-53] /\ ko1 in [-25b-53, 25b-53]
  /\ ko2 in [-25b-53, 25b-53]
  # What vector.g, vector1.g and vector2.g prove of the turned direction.
  /\ da0 in [-20b-53, 20b-53] /\ da1 in [-20b-53, 20b-53]
  /\ da2 in [-20b-53, 20b-53]
->
  DENQ - 1 in ? /\ EaEa - 1 in ? /\ NUMQ / DENQ - 1 in ?
  /\ S in ? /\ TI in ? /\ DR in ?
  /\ S + TI + DR in [0, 256b-53]
}

# |Ex(y)|^2 is |y|^2 and the turn's length share (464.20's 3), and |v x y|^2
# is at least zero, a sum of squares, and at most v.v y.y; v.v is q.q less
# w^2. |Ex(a)|^2 is the same with a along z.
EE -> YY + 4 * (N4 - 1) * VY;
VY -> VV * YY - VD * VD;
VV -> N4 - w * w;
EaEa -> AA * DENQ;
S01 -> N4 - w * w - v2 * v2;

# NUMQ / DENQ off one: the two differ by 4 (q.q - 1) (v1^2 + v2^2) and a
# share in (q.q - 1)^2.
NUMQ / DENQ -> 1 + (NUMQ - DENQ) / DENQ;
NUMQ - DENQ -> 4 * (N4 - 1) * S12 + 16 * (N4 - 1) * (N4 - 1) * VV * (v1 * v1);
S12 -> N4 - w * w - v0 * v0;
```

`torus.g`:

```gappa
# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965
# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965
# restates: crates/rcad-geom/src/torus.rs transformed at f95bb965
# restates: crates/rcad-geom/src/point.rs transformed at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_point at f95bb965
# restates: crates/rcad-geom/src/isometry.rs apply_to_vector at f95bb965
# CORNER_AFTER_MOVING_ROUNDOFF for a corner on a torus,
# crates/rcad-geom/src/moved_corner.rs at f95bb965: a corner exactly on a
# torus of ring radius R and tube radius r about the centre o and the axis a,
# moved, is within 128 * f64::EPSILON * L of the moved torus.
# Torus::transformed carries R and r over, and Axis::transformed moves o by
# apply_to_point and turns a by apply_to_vector. With y the corner less o, the
# moved corner less the moved centre is Ex(y) + k, k the two slips point.g
# proves, and the moved axis is Ex(a) + da, da the slip vector.g proves, Ex
# nalgebra's turn without rounding (sos.py). The distance from a torus is
# | |(rho - R, t)| - r |, rho how far from the axis and t how far along it. By
# hand, the distance to the moved torus is at most |k| + |Ex(y)| tan(th) +
# |(rho' - rho, t' - t)|, th the angle da tilts Ex(a) by, and rho' and t' those
# of Ex(y) about the unit Ex(a); this bounds that, per unit of L. Ex turns
# with the frame (sos.py), so here a is along z and y in the xz plane, and w
# and v are the turn's quaternion in that frame: reals, not f64.
@rnd = float<ieee_64, ne>;

# The quaternion (w, v) in that frame, q.q, v.v and three sums of its squares.
N4 = w * w + v0 * v0 + v1 * v1 + v2 * v2;
VV = v0 * v0 + v1 * v1 + v2 * v2;
S01 = v0 * v0 + v1 * v1;
S12 = v1 * v1 + v2 * v2;
S02 = v0 * v0 + v2 * v2;
P02 = 2 * v0 * v2;

# The stored axis a is (0, 0, al), and y over L is (RL, 0, TL): RL is how far
# the corner is from the axis, and TL how far along it. Both are written in
# place below, coordinate by coordinate, since gappa -Bcoq aborts in dump_real
# when names hold them (464.21's 7).
AA = al * al;
YY = RL * RL + 0 * 0 + TL * TL;

# nalgebra's turn of y and of a without rounding, as point.g's Ex writes x,
# and their lengths squared.
Ex0 = 2 * w * (v1 * TL - v2 * 0)
    + v1 * (2 * (v0 * 0 - v1 * RL)) - v2 * (2 * (v2 * RL - v0 * TL)) + RL;
Ex1 = 2 * w * (v2 * RL - v0 * TL)
    + v2 * (2 * (v1 * TL - v2 * 0)) - v0 * (2 * (v0 * 0 - v1 * RL)) + 0;
Ex2 = 2 * w * (v0 * 0 - v1 * RL)
    + v0 * (2 * (v2 * RL - v0 * TL)) - v1 * (2 * (v1 * TL - v2 * 0)) + TL;
Ea0 = 2 * w * (v1 * al - v2 * 0)
    + v1 * (2 * (v0 * 0 - v1 * 0)) - v2 * (2 * (v2 * 0 - v0 * al)) + 0;
Ea1 = 2 * w * (v2 * 0 - v0 * al)
    + v2 * (2 * (v1 * al - v2 * 0)) - v0 * (2 * (v0 * 0 - v1 * 0)) + 0;
Ea2 = 2 * w * (v0 * 0 - v1 * 0)
    + v0 * (2 * (v2 * 0 - v0 * al)) - v1 * (2 * (v1 * al - v2 * 0)) + al;
EE = Ex0 * Ex0 + Ex1 * Ex1 + Ex2 * Ex2;
EaEa = Ea0 * Ea0 + Ea1 * Ea1 + Ea2 * Ea2;

# v x y and v . y, without rounding.
X0 = v1 * TL - v2 * 0; X1 = v2 * RL - v0 * TL; X2 = v0 * 0 - v1 * RL;
VY = X0 * X0 + X1 * X1 + X2 * X2;
VD = v0 * RL + v1 * 0 + v2 * TL;

# In this frame |Ex(a)|^2 is al^2 DENQ, |Ex(y) x Ex(a)|^2 is
# RL^2 al^2 NUMQ and Ex(y) . Ex(a) is al (TL DENQ - 2 (q.q - 1) P02 RL)
# (sos.py), so Ex(y) is RL sqrt(NUMQ / DENQ) from the turned axis and
# TL sqrt(DENQ) - 2 (q.q - 1) P02 RL / sqrt(DENQ) along it.
DENQ = 1 + 4 * (N4 - 1) * S01;
NUMQ = 1 + 4 * (N4 - 1) * (VV + v1 * v1) + 16 * (N4 - 1) * (N4 - 1) * VV * (v1 * v1);

# The two slips, the moved corner's and the moved centre's, per unit of L,
# and the moved axis's.
k0 = kp0 - ko0; k1 = kp1 - ko1; k2 = kp2 - ko2;
KK = k0 * k0 + k1 * k1 + k2 * k2;
DADA = da0 * da0 + da1 * da1 + da2 * da2;
DA = sqrt(DADA);

# The distance's three shares, per unit of L: the slips', the tilt's, and
# how far the turn's arithmetic takes the corner off the torus, from how far
# it moves the corner off the axis and along it.
S = sqrt(KK);
TI = sqrt(EE) * DA / (sqrt(EaEa) - DA);
DRr = RL * (sqrt(NUMQ / DENQ) - 1);
DRt = TL * (sqrt(DENQ) - 1) - 2 * (N4 - 1) * P02 * RL / sqrt(DENQ);
DR = sqrt(DRr * DRr + DRt * DRt);

{
  # A stored turn has |q.q - 1| <= 6 epsilon (held_near_length_one), and
  # 6 epsilon is 12b-53. Turning the frame turns v and keeps q.q.
     N4 in [9007199254740980b-53, 9007199254741004b-53]
  # Each of w^2, v0^2, v1^2 and v2^2 is at most q.q, under 1 + 12u, so each
  # of w, v0, v1 and v2 is at most 2 in size.
  /\ |w| <= 2 /\ |v0| <= 2 /\ |v1| <= 2 /\ |v2| <= 2
  # A stored axis has a.a within NORMAL_LENGTH_ROUNDOFF, 64 epsilon, of one.
  /\ AA in [9007199254740864b-53, 9007199254741120b-53]
  /\ al in [0, 2]
  # The corner is at most ring + tube from the centre, and L is at least
  # ring + tube as f64 adds them, which is at least (ring + tube) / (1 + u).
  # So |y| / L is at most 1 + u: RL and TL within 1 + 2u, and YY within
  # (1 + u)^2, under 1 + 8u.
  /\ RL in [0, 4503599627370497b-52]
  /\ TL in [-4503599627370497b-52, 4503599627370497b-52]
  /\ YY in [0, 1125899906842625b-50]
  # What point.g, point1.g and point2.g prove of the moved corner and the
  # moved centre, L at least every coordinate of either and of the shift.
  /\ kp0 in [-36b-53, 36b-53] /\ kp1 in [-36b-53, 36b-53]
  /\ kp2 in [-36b-53, 36b-53]
  /\ ko0 in [-36b-53, 36b-53] /\ ko1 in [-36b-53, 36b-53]
  /\ ko2 in [-36b-53, 36b-53]
  # What vector.g, vector1.g and vector2.g prove of the turned axis.
  /\ da0 in [-20b-53, 20b-53] /\ da1 in [-20b-53, 20b-53]
  /\ da2 in [-20b-53, 20b-53]
->
  DENQ - 1 in ? /\ EaEa - 1 in ? /\ NUMQ / DENQ - 1 in ? /\ P02 in ?
  /\ S in ? /\ TI in ? /\ DRr in ? /\ DRt in ? /\ DR in ?
  /\ S + TI + DR in [0, 256b-53]
}

# |Ex(y)|^2 is |y|^2 and the turn's length share (464.20's 3), and |v x y|^2
# is at least zero, a sum of squares, and at most v.v y.y; v.v is q.q less
# w^2. |Ex(a)|^2 is the same with a along z.
EE -> YY + 4 * (N4 - 1) * VY;
VY -> VV * YY - VD * VD;
VV -> N4 - w * w;
EaEa -> AA * DENQ;
S01 -> N4 - w * w - v2 * v2;

# NUMQ / DENQ off one: the two differ by 4 (q.q - 1) (v1^2 + v2^2) and a
# share in (q.q - 1)^2.
NUMQ / DENQ -> 1 + (NUMQ - DENQ) / DENQ;
NUMQ - DENQ -> 4 * (N4 - 1) * S12 + 16 * (N4 - 1) * (N4 - 1) * VV * (v1 * v1);
S12 -> N4 - w * w - v0 * v0;

# 2 v0 v2 is v0^2 + v2^2 less a square, and a square less v0^2 + v2^2, so at
# most v0^2 + v2^2 in size, which is at most q.q.
P02 -> S02 - (v0 - v2) * (v0 - v2);
P02 -> (v0 + v2) * (v0 + v2) - S02;
S02 -> N4 - w * w - v1 * v1;
```

`controls.py`:

```python
"""Write the controls of ball.g, plane.g, pipe.g and torus.g, pipe.g's run with
the slips at POINT_ROUNDOFF's 36u, and point.g's runs at 25u. Each control
changes one thing: a smaller bound, to see whether the script proved its goal
from hypotheses that contradict each other; or a hypothesis dropped or
weakened, to see whether the proof needs it. r_point_25.g is 464.19's point.g
with its goal at 25u, and r_point1_25.g and r_point2_25.g are its y and z,
turned as 464.19's turn.py turns point.g."""

import os
import re

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


N4 = "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]\n"
# pipe.g and torus.g open their hypotheses with q.q, so it has no /\.
N4_FIRST = "     N4 in [9007199254740980b-53, 9007199254741004b-53]\n"
# q.q only as large as the bounds on w and v let it be.
N4_WIDE = "     N4 in [0, 16]\n"
WV = ("  # Each of w^2, v0^2, v1^2 and v2^2 is at most q.q, under 1 + 12u, so each\n"
      "  # of w, v0, v1 and v2 is at most 2 in size.\n"
      "  /\\ |w| <= 2 /\\ |v0| <= 2 /\\ |v1| <= 2 /\\ |v2| <= 2\n")
AXIS_GOAL = "/\\ S + TI + DR in [0, 256b-53]"

ball = open("ball.g").read()
# 148u, below the 124.7u + 24u the two shares reach.
write("c_ball_148.g", sub(ball, "/\\ S + T in [0, 256b-53]", "/\\ S + T in [0, 148b-53]"))
# Without |q.q - 1| <= 6 epsilon.
write("c_ball_non4.g", sub(ball, N4, ""))
# Without what point.g proves of the two slips.
write("c_ball_nok.g", sub(ball, (
    "  # What point.g, point1.g and point2.g prove of the moved corner and the\n"
    "  # moved centre, L at least every coordinate of either and of the shift.\n"
    "  /\\ kp0 in [-36b-53, 36b-53] /\\ kp1 in [-36b-53, 36b-53]\n"
    "  /\\ kp2 in [-36b-53, 36b-53]\n"
    "  /\\ kc0 in [-36b-53, 36b-53] /\\ kc1 in [-36b-53, 36b-53]\n"
    "  /\\ kc2 in [-36b-53, 36b-53]\n"), ""))

plane = open("plane.g").read()
# 222u, below the 83.14u + 60u + 62.35u + 17u the four shares reach.
write("c_plane_222.g", sub(plane, "/\\ T / sqrt(NPNP) in [-256b-53, 256b-53]",
                           "/\\ T / sqrt(NPNP) in [-222b-53, 222b-53]"))
# Without |q.q - 1| <= 6 epsilon: the turn's length share.
write("c_plane_non4.g", sub(plane, N4, ""))
# Without what vector.g proves of the turned normal.
write("c_plane_nodn.g", sub(plane, (
    "  # What vector.g, vector1.g and vector2.g prove of the turned normal.\n"
    "  /\\ dn0 in [-20b-53, 20b-53] /\\ dn1 in [-20b-53, 20b-53]\n"
    "  /\\ dn2 in [-20b-53, 20b-53]\n"), ""))
# Without what point.g proves of the moved corner.
write("c_plane_nodp.g", sub(plane, (
    "  # What point.g, point1.g and point2.g prove of the moved corner, over L.\n"
    "  /\\ DP0 in [-36b-53, 36b-53] /\\ DP1 in [-36b-53, 36b-53]\n"
    "  /\\ DP2 in [-36b-53, 36b-53]\n"), ""))
# Without what shiftplane.g proves of the offset.
write("c_plane_nodd.g", sub(plane, (
    "  # What shiftplane.g proves of the offset's rounding over L, 16u times\n"
    "  # the larger of |n'| and one, with |n'| within 124u of one: under 17u.\n"
    "  /\\ DD in [-17b-53, 17b-53]\n"), ""))

pipe = open("pipe.g").read()
# 230u, below the 86.6u + 120u + 24u the three shares reach.
write("c_pipe_230.g", sub(pipe, AXIS_GOAL, "/\\ S + TI + DR in [0, 230b-53]"))
write("c_pipe_non4.g", sub(pipe, N4_FIRST, N4_WIDE))
# Without the bounds on w and v: the first run of pipe.g.
write("c_pipe_nobound.g", sub(pipe, WV, ""))
# Without what vector.g proves of the turned direction.
write("c_pipe_noda.g", sub(pipe, (
    "  # What vector.g, vector1.g and vector2.g prove of the turned direction.\n"
    "  /\\ da0 in [-20b-53, 20b-53] /\\ da1 in [-20b-53, 20b-53]\n"
    "  /\\ da2 in [-20b-53, 20b-53]\n"), ""))
# The six slips at POINT_ROUNDOFF's 36u, as the published constant has them.
assert pipe.count("[-25b-53, 25b-53]") == 6
write("r_pipe_36.g", pipe.replace("[-25b-53, 25b-53]", "[-36b-53, 36b-53]"))

torus = open("torus.g").read()
# 212u, below the 124.7u + 34.6u + 53.7u the three shares reach.
write("c_torus_212.g", sub(torus, AXIS_GOAL, "/\\ S + TI + DR in [0, 212b-53]"))
write("c_torus_non4.g", sub(torus, N4_FIRST, N4_WIDE))

# 464.19's turn.py, whole but for reading and writing the files.
TURNED = {"v", "p", "s", "P", "q", "C", "c", "rq", "rc", "t", "k", "rk"}
NAME = re.compile(r"\b([A-Za-z]+)(\d)(\d?)(_?)\b")


def turn(src):
    def one(m):
        prefix, index, rest, under = m.groups()
        if prefix not in TURNED:
            return m.group(0)
        return prefix + str((int(index) + 1) % 3) + rest + under

    return NAME.sub(one, src)


point = open("point.g").read()
point = sub(point, "x is within 18 * f64::EPSILON", "x is within 12.5 * f64::EPSILON")
point = sub(point, "/\\ (out - Ex) / L in [-36b-53, 36b-53]", "/\\ (out - Ex) / L in [-100b-55, 100b-55]")
write("r_point_25.g", point)
once = turn(point)
twice = turn(once)
assert turn(twice) == point
X = "# moved point's x is within"
for name, src, axis in (("r_point1_25.g", once, "y"), ("r_point2_25.g", twice, "z")):
    write(name, sub(src, X, "# moved point's " + axis + " is within"))
```

`all.sh`:

```sh
#!/bin/bash
# Write the controls and point.g's runs at 25u, then run every script at once,
# each through 464.13's one.sh: the four surfaces and the three runs at 25u
# with coq, and the controls and r_pipe_36.g without.
cd "$(dirname "$0")"
python3 controls.py
for f in ball.g plane.g pipe.g torus.g r_point_25.g r_point1_25.g r_point2_25.g; do
  ./one.sh "$f" coq > "${f%.g}.run" 2>&1 &
done
for f in c_ball_148.g c_ball_non4.g c_ball_nok.g \
         c_plane_222.g c_plane_non4.g c_plane_nodn.g c_plane_nodp.g c_plane_nodd.g \
         c_pipe_230.g c_pipe_non4.g c_pipe_nobound.g c_pipe_noda.g r_pipe_36.g \
         c_torus_212.g c_torus_non4.g; do
  ./one.sh "$f" > "${f%.g}.run" 2>&1 &
done
wait
cat ball.run plane.run pipe.run torus.run r_point_25.run r_point1_25.run r_point2_25.run \
    c_ball_148.run c_ball_non4.run c_ball_nok.run \
    c_plane_222.run c_plane_non4.run c_plane_nodn.run c_plane_nodp.run c_plane_nodd.run \
    c_pipe_230.run c_pipe_non4.run c_pipe_nobound.run c_pipe_noda.run r_pipe_36.run \
    c_torus_212.run c_torus_non4.run
```

It printed, on the last run:

```text
ball exit 0 0.13s
ball coq -Bcoq exit 0 coqc exit 0 1.00s lines 1347 hints 3
plane exit 0 0.93s
plane coq -Bcoq exit 0 coqc exit 0 3.40s lines 8504 hints 17
pipe exit 0 0.24s
pipe coq -Bcoq exit 0 coqc exit 0 1.55s lines 3422 hints 8
torus exit 0 0.30s
torus coq -Bcoq exit 0 coqc exit 0 1.76s lines 3556 hints 11
r_point_25 exit 0 45.85s
r_point_25 coq -Bcoq exit 0 coqc exit 0 78.49s lines 8222 hints 34
r_point1_25 exit 0 48.21s
r_point1_25 coq -Bcoq exit 0 coqc exit 0 73.58s lines 8162 hints 34
r_point2_25 exit 0 45.56s
r_point2_25 coq -Bcoq exit 0 coqc exit 0 102.15s lines 8195 hints 34
c_ball_148 exit 1 0.16s
c_ball_non4 exit 1 0.12s
c_ball_nok exit 1 0.13s
c_plane_222 exit 1 1.36s
c_plane_non4 exit 1 1.21s
c_plane_nodn exit 1 0.79s
c_plane_nodp exit 1 1.25s
c_plane_nodd exit 1 1.07s
c_pipe_230 exit 1 0.24s
c_pipe_non4 exit 1 0.25s
c_pipe_nobound exit 1 0.23s
c_pipe_noda exit 1 0.23s
r_pipe_36 exit 1 0.23s
c_torus_212 exit 1 0.35s
c_torus_non4 exit 1 0.30s
```

`same.py`, a check that what this subsection appends is what ran:

```python
"""Check that each script 464.21 appends whole is byte for byte the file that
was run, in q2691/; that one.sh is 464.13's block and point.g 464.19's; that
sos.py, run again, prints what 464.21 gives, and that all.log is the print
464.21 gives for all.sh; and that controls.py, run again in a fresh directory
beside the four surface scripts from the appended text and point.g from
464.19's, writes the scripts that ran.
Run as: python3 same.py <doc> <q2691 dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, ran = sys.argv[1], sys.argv[2]
text = open(doc).read()
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)


def section(n):
    start = text.index(f"### 464.{n} ")
    end = text.find("\n### ", start + 1)
    return text[start:] if end < 0 else text[start:end + 1]


sec = section(21)
blocks = FENCED.findall(sec)
print("code blocks in 464.21:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
HEAD = ("# restates: crates/rcad-geom/src/moved_corner.rs CORNER_AFTER_MOVING_ROUNDOFF at f95bb965\n"
        "# restates: crates/rcad-geom/src/moved_corner.rs the_largest_number_involved at f95bb965\n")
marks = {
    "ball.g": HEAD + "# restates: crates/rcad-geom/src/sphere.rs",
    "plane.g": HEAD + "# restates: crates/rcad-geom/src/plane.rs",
    "pipe.g": HEAD + "# restates: crates/rcad-geom/src/cylinder.rs",
    "torus.g": HEAD + "# restates: crates/rcad-geom/src/torus.rs",
    "sos.py": '"""Check, in exact rationals at random points, the identities 464.21',
    "controls.py": '"""Write the controls of ball.g, plane.g, pipe.g and torus.g',
    "all.sh": "#!/bin/bash\n# Write the controls and point.g's runs at 25u",
    "same.py": '"""Check that each script 464.21 appends whole',
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
    path = os.path.abspath(__file__) if name == "same.py" else os.path.join(ran, name)
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")


def block_of(n, first):
    got = [body for lang, body in FENCED.findall(section(n)) if body.startswith(first)]
    assert len(got) == 1, (n, first, len(got))
    return got[0]


def printed(first):
    got = [body for lang, body in blocks if lang == "text" and body.startswith(first)]
    assert len(got) == 1, (first, len(got))
    return got[0]


print("one.sh is 464.13's block:", open(os.path.join(ran, "one.sh")).read()
      == block_of(13, "#!/bin/bash\n# Run one Gappa script under 464.4's pin"))
point = block_of(19, "# restates: crates/rcad-geom/src/isometry.rs apply_to_point at ba0a654e\n")
print("point.g is 464.19's block:", open(os.path.join(ran, "point.g")).read() == point)
again = subprocess.run(["python3", os.path.join(ran, "sos.py")], capture_output=True, text=True,
                       check=True).stdout
print("sos.py run again prints what 464.21 gives:", again == printed("identities 1 to 4 hold"))
print("all.log is what 464.21 gives:",
      open(os.path.join(ran, "all.log")).read() == printed("ball exit "))

fresh = tempfile.mkdtemp()
for name in ("ball.g", "plane.g", "pipe.g", "torus.g", "controls.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
open(os.path.join(fresh, "point.g"), "w").write(point)
subprocess.run(["python3", "controls.py"], cwd=fresh, check=True)
written = sorted(f for f in os.listdir(fresh) if f.endswith(".g") and f not in found and f != "point.g")
for name in written:
    print(name, "written again is the one that ran:",
          filecmp.cmp(os.path.join(fresh, name), os.path.join(ran, name), shallow=False))
print(".g files that ran and are neither appended, 464.19's nor written again:",
      sorted(f for f in os.listdir(ran)
             if f.endswith(".g") and f not in found and f != "point.g" and f not in written))
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.21: 11
ball.g same
plane.g same
pipe.g same
torus.g same
sos.py same
controls.py same
all.sh same
same.py same
one.sh is 464.13's block: True
point.g is 464.19's block: True
sos.py run again prints what 464.21 gives: True
all.log is what 464.21 gives: True
c_ball_148.g written again is the one that ran: True
c_ball_nok.g written again is the one that ran: True
c_ball_non4.g written again is the one that ran: True
c_pipe_230.g written again is the one that ran: True
c_pipe_nobound.g written again is the one that ran: True
c_pipe_noda.g written again is the one that ran: True
c_pipe_non4.g written again is the one that ran: True
c_plane_222.g written again is the one that ran: True
c_plane_nodd.g written again is the one that ran: True
c_plane_nodn.g written again is the one that ran: True
c_plane_nodp.g written again is the one that ran: True
c_plane_non4.g written again is the one that ran: True
c_torus_212.g written again is the one that ran: True
c_torus_non4.g written again is the one that ran: True
r_pipe_36.g written again is the one that ran: True
r_point1_25.g written again is the one that ran: True
r_point2_25.g written again is the one that ran: True
r_point_25.g written again is the one that ran: True
.g files that ran and are neither appended, 464.19's nor written again: []
```

**9. What this leaves.** Rows, filed from this write-up when it merges, each
from Q2691:

- Restate `CORNER_AFTER_MOVING_ROUNDOFF`'s doc (`moved_corner.rs:38-97`)
  against this subsection: the shares of 3, the three miscounts of 4, the
  pipe's need of `point.g`'s `25u`, and the cone left unproved.
- Search for a corner on a cone that a move puts past `128ε · L`, or prove the
  cone within it by a sharper argument than 3's. Not run, so no class
  (§360.5).
- Run `vector.g` with each component up to about `1 + 64u`, as a stored axis
  direction can have, and say whether its `20u` holds; `pipe.g` and `torus.g`
  take it. Not run, so no class. Q2826 is its twin for a normal.

And what is not a row: the zero branches (Q2801, Q2825), the products below
normal (Q2802) and the route through `From<na::Isometry3<f64>>` (Q2803), which
these scripts share with `point.g` and `vector.g`; and Q2700, which moves these
scripts into `proofs/gappa/` with the rest of 464's.

### 464.22 EVALUATION_ROUNDOFF's one stage

Q2692 proved `EVALUATION_ROUNDOFF`'s one-stage recurrence in Gappa on
2026-10-26 at `812503ed`. One coordinate of one de Boor stage, as `de_boor`
and `blended` compute it in `f64`, is within `(1 + 9u)·E + 9u·M` of the stage
worked without rounding on the true running values. That holds for every `E`
from 0 to `2⁻⁴⁰ M` and for each of the three things the computed coefficient
can be (2). Gappa's count of the `M` part is `8.5u`, so the doc's `9u` holds
with `0.5u` to spare, but by another split than the doc's (3, 4). Coq checks
five of the six scripts, modulo their hints, and `stage_e0_1.g` is proved by
Gappa and not checked by Coq (7). The sum over fifteen stages is not here: it
stays Lean's (464.9).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, unchanged, against `de_boor.rs` and `knot_vector.rs` at
`812503ed`. `u` is `2⁻⁵³`.

**1. What the proof guards.** `EVALUATION_ROUNDOFF` (`de_boor.rs:29-114`)
bounds how far each homogeneous coordinate of one de Boor pass lands from the
true one, as a fraction of `M`, the largest control coordinate the evaluation
touches. Its doc gets there from a recurrence for one stage, `E_r ≤
E_{r−1}·(1 + 9u) + 9u·M` (`de_boor.rs:41-64`), and that recurrence is what
this proves. One stage is `de_boor`'s inner loop (`de_boor.rs:216-227`): the
coefficient `share` at line 225, then `blended` at 226, which is `rest·before
+ share·after` on each coordinate (`de_boor.rs:594-596`). `Coefficient for
f64` (`de_boor.rs:141-157`) is plain `f64` arithmetic, so one coordinate of a
stage rounds seven times: the two subtractions, the divide, `1 − share`, the
two products and the sum.

**2. The scripts, and how they are cut.** In the scripts `t0` and `t1` are the
true running values, `d0` and `d1` the computed ones, `A` the true coefficient
`(at − below) / (above − below)`, and `Ex = (1 − A)·t0 + A·t1` the stage
worked without rounding. `out − Ex` is exactly `XE + XM`, with `XE = rest·(d0
− t0) + share·(d1 − t1)` the incoming errors carried through, and `XM = (out −
S) + (P0 − z0) + (P1 − z1) + (rest − (1 − share))·t0 + (share − A)·(t1 − t0)`
what this stage rounds. That is an identity of real numbers, checked by hand.

`E` is cut in two:

- `stage.g` takes `E / M` in `[2⁻⁶⁰, 2⁻⁴⁰]` and proves `|out − Ex| ≤ (1 +
  9u)·E + 9u·M`.
- `stage_e0.g` takes each incoming error within `2⁻⁶⁰ M` and proves `|out −
  Ex| ≤ 9u·M`, which is at most `(1 + 9u)·E + 9u·M` for every `E ≥ 0`. Stage
  1 starts from the control points themselves, `E = 0`, and is one of these.

No stage gets past `2⁻⁴⁰ M`: while the stages before it hold the recurrence,
`E_{r−1} ≤ EVALUATION_ROUNDOFF · M = 1.4988e-14 M`, which is `2⁻⁴⁵·⁹² M`. The
cut is needed. `r_stage_e_from_0.g`, `stage.g` with `E / M` from 0, leaves
`XE / V` and `XE / cE` unproved and fails with best `±1.61714e+631`.

`share` is cut in three. `0 ≤ at − below ≤ above − below`, and rounding keeps
order, so the computed `share` is an `f64` in `[0, 1]`: 0, or in `[2⁻¹⁰⁷⁴, 1
− 2⁻⁵³]`, or 1. The two main scripts take the middle range, and `controls.py`
(8) writes the edges from them: `stage_0.g`, `stage_1.g`, `stage_e0_0.g` and
`stage_e0_1.g`, with `share` at `[0, 0]` and at `[1, 1]`. The cut is needed
too. `r_e0_whole.g`, `stage_e0.g` with `share` over all of `[0, 1]`, fails with
best `±1.0005e-15`, `9.01u`, in 51.41 s.

The hints. `(out − Ex) / D`, with `D = cE + cM`, `cE = (1 + 9u)·E` and `cM =
9u·M`, is the average of `XE / cE` and `XM / cM` weighted by `cE` and `cM`, so
it is within one when both are. `XE / cE` goes through `XE / V`, the average
of the two incoming errors over twice their weights, `cR = 2·rest·E` and `cS =
2·share·E`, which Gappa bounds by `0.5`. `r_stage_plain.g`, averaging over the
weights themselves, bounds `XE / V` by `1.5` and fails with best
`±1.10213e+619`; why it is `1.5` was not traced. `XM / cM` is `XMM / 9u`, with
`XMM` the five shares of `XM` per unit of `M`, and each rounding is taken
relative to what it rounds. `$ share in (0.5)` splits where `1 − share` starts
to be exact, by Sterbenz's lemma, and `z0`, `z1` and `S` split at `±2⁻¹⁰²²`,
below which their rounding is absolute. The edges need no average: with
`share` at 0 or 1 one incoming error has no weight, and `controls.py` puts one
hint in place of the eight.

**3. The shares.** `stage.g`'s, as it printed them, over the middle range of
`share`:

- `share − A` within `±2.77556e-16`, `2.5u`;
- `rest − (1 − share)` within `±5.55112e-17`, `0.5u`;
- `(out − S) / M` within `±1.66533e-16`, `1.5u`, and `(P0 − z0) / M` and `(P1
  − z1) / M` within `±1.11022e-16`, `u`, each;
- `XMM` within `±9.4369e-16`, `8.500000000002274u`, and so `XM / cM` within
  `±0.944444`, `8.5 / 9`;
- `XE / V` within `±0.5`, and `XE / cE` within `±1`.

`stage_e0.g` prints the same shares, with `XMM` within `±9.4369e-16`, exactly
`8.5u`. A probe with its goal left open, and `c_e0_8375.g`'s best, both print
`±9.44991e-16`, `8.51171875u`, and `r_e0_85.g` proves the goal at `8.5u`.

At the edges, at `share = 0` `share − A` is in `[−2.47033e-324, 0]`, the two
products are exact, and `XMM` is within `±1.11022e-16`, `u`, so `XM / cM` is
within `±0.111111`. At `share = 1` `share − A` is in `[0, 3.33067e-16]`, `3u`,
`XMM` within `±7.77156e-16`, `7u`, and `XM / cM` within `±0.777778`. In both
`rest − (1 − share)` is 0, and `stage_e0_0.g` and `stage_e0_1.g` give the same
`XMM` as `stage_0.g` and `stage_1.g`.

**4. What the doc counts differently.** The doc (`de_boor.rs:48-64`) charges
`4u` for `b = 1 − a`, `3u` for `a`, and `2u` for the two products and the sum,
`9u` in all. None of its terms is shown wrong. It is a first-order count, and
it says so (`de_boor.rs:70-77`): its `18225u²` pays for "the second-order terms
dropped from the per-stage argument above". Gappa's count, which drops
nothing, splits the `8.5u` another way:

- `share − A` is `2.5u` over the middle range. By hand, the divide rounds a
  quotient below one by at most `0.5u`, and the two subtractions cost about
  `2u·A`. At `share = 1` it is `3u`, the doc's figure.
- The two coefficient errors are one error. `rest`'s is `share`'s with its
  sign turned, plus `rest`'s own rounding, so together they enter as `(share −
  A)·(t1 − t0) + (rest − (1 − share))·t0`. On `|t1 − t0| ≤ 2M` the first is
  `5u`, where the doc has `7u`. The second is 0 above `share = 1/2`, where `1 −
  share` is exact, and `0.5u` below it, where the doc charges `u`.
- The two products and the sum are `u`, `u` and `1.5u`, `3.5u`, each rounding
  bounded on its own. The doc takes the three together, at `2u`, and these
  scripts do not check that.

Above `share = 1/2` that is `5u + 0 + 3.5u = 8.5u`. Below it, the shares as
printed, each over the whole middle range, add to `9u`, and `XMM`, Gappa's
bound on their sum, is still `8.5u`.

**5. The hypotheses.** Each script's, in its order:

- `at`, `below` and `above` are `f64` values, at most `2¹⁰²⁴` in magnitude.
- `above − below` is in `[2⁻¹⁰⁷⁴, 2¹⁰²⁴]`: positive by the assert at
  `de_boor.rs:220`, and in range because `KnotVector::new` refuses a knot
  vector whose first and last knots cannot be subtracted, and every difference
  the recursion forms sits between those two (precondition 3,
  `de_boor.rs:95-102`).
- `A` is in `[0, 1]`: `span_containing` (`knot_vector.rs:193`) gives a span
  with `knots[span] ≤ parameter ≤ knots[span + 1]`, and `below` and `above`
  sit on either side of it (`de_boor.rs:190-199`).
- `share` is in the middle range, or at 0 or 1 in the edges, as 2 says.
- `M` is in `[2⁻¹⁰²², 2¹⁰²⁴]`, normal by precondition 1 (`de_boor.rs:90-93`).
- `|t0|` and `|t1|` are at most `M`, since the true running values are
  averages, with weights in `[0, 1]` that sum to one, of control coordinates
  at most `M` in magnitude.
- In `stage.g`, `E / M` is in `[2⁻⁶⁰, 2⁻⁴⁰]` and each incoming error is at
  most `E`; in `stage_e0.g`, each incoming error is at most `2⁻⁶⁰ M`.

Gappa uses neither `A in [0, 1]` nor the knots' difference. `r_stage_noa.g`
and `r_stage_noknots.g`, `stage.g` with each dropped, prove, with `share − A`
within `±2.77556e-16` as before. Both are needed all the same, by hand: `A`'s
range for `share`'s in 2, and the difference for the divisor `−(below) +
(above)`, which `stage.out` takes to be nonzero 4 times (7). Overflow of the
running values is not modelled: Gappa's `float<ieee_64, ne>` has no largest
number.

**6. The controls.** Each changes one thing and exits 1. A goal just under
what the script reaches shows its hypotheses do not contradict each other,
and a hypothesis dropped or widened shows the proof needs it.

- `c_stage_85.g`, `stage.g` with `cM` at `8.5u`: best `±7.12221e+615`. `XMM`
  reaches `8.500000000002274u`, so `XM / cM` comes out `2.7e-13` over one.
- `c_stage_m.g`, `M` allowed below the normals: best `±1.00984e+631`.
- `c_stage_noe.g`, without `E / M`'s range: best `[-inf, inf]`, with `(out −
  S) / M`, `(P0 − z0) / M`, `(P1 − z1) / M`, `XMM`, `XE / V`, `XE / cE` and
  `XM / cM` unproved.
- `c_stage_noshare.g`, without `share`'s range: best `±1.10289e+619`.
- `c_stage_not.g`, without `|t0|` and `|t1|` at most `M`: best `[-inf, inf]`,
  with the three roundings, `XMM` and `XM / cM` unproved.
- `c_stage_nod.g`, without the incoming errors at most `E`: best `[-inf,
  inf]`, with the same seven unproved as `c_stage_noe.g`.
- `c_e0_8375.g`, `stage_e0.g` with its goal at `8.375u`: best `±9.44991e-16`,
  `8.51171875u`.
- `c_e0_m.g`, `M` allowed below the normals: best `[-1, 1]`, in 37.26 s.
- `c_e0_nod.g`, without the incoming errors at most `2⁻⁶⁰ M`: best `[-inf,
  inf]`, with the three roundings and `XMM` unproved.

Runs, and not controls: `r_stage_noa.g` and `r_stage_noknots.g` (5) and
`r_e0_85.g`, `stage_e0.g` with its goal at `8.5u`, prove; `r_stage_e_from_0.g`,
`r_stage_plain.g` and `r_e0_whole.g` fail, as 2 says. A probe tried
`stage_e0.g`'s goal at `8u`, `8.25u` and `8.375u` before the controls were
written, and each failed.

**7. Coq checks five of the six, modulo their hints.** `gappa -Bcoq` and
`coqc`, through `one.sh`, exit 0 on `stage.g`, `stage_0.g`, `stage_1.g`,
`stage_e0.g` and `stage_e0_0.g`. On the last run of `all.sh` (8) their `.v`
files came to 4529 lines with 21 hints, 2572 with 14, 2479 with 14, 1825 with
9 and 1388 with 9, and `coqc` printed nothing.

`stage_e0_1.g`, the small-`E` script at `share = 1`, proves, and `coqc`
refuses its `.v`. `one.sh` printed `stage_e0_1 coq -Bcoq exit 0 coqc exit 1
1.15s lines 1543 hints 9`, and `coqc` said:

```text
File "./stage_e0_1.v", line 468, characters 29-30:
Error: Cannot interpret this number as a value of type positive
```

Line 468 is `Definition p49 := FLT _rest (0). (* FLT(rest, 0) *)`. `FLT(rest,
0)` says `rest` needs no bits, which holds since `rest` is exactly 0 there, and
characters 29-30 are that `0`, which Coq will not read as a `positive`.
`p_e0_1_inline.g`, with `rest` written in place, gave the same error at its
line 468, at `FLT r7 (0)`, and printed `p_e0_1_inline coq -Bcoq exit 0 coqc
exit 1 0.79s lines 1543 hints 9`; it is not appended. `stage_1.g` has `rest`
exactly 0 too, and its `.v` passes; why was not looked for. So that edge is
proved by Gappa and not checked by Coq. The fault is in what `-Bcoq` writes,
not in the kernel, and a row in 9 asks for the check.

As in 464.21's 7, each hint enters the `.v` as a hypothesis with no
condition, so a proof stands on its hints and on the divisors Gappa took to be
nonzero. `grep -c 'assumed to be nonzero'` counts 46 in `stage.out`, 41 in
each of `stage_0.out` and `stage_1.out`, and 33 in each of `stage_e0.out`,
`stage_e0_0.out` and `stage_e0_1.out`. `stage.out`'s, by divisor: `M` 26, `E`
4, `−(below) + (above)` 4, `V` 3, `D` 2, and once each `cR`, `cS`, `(1 +
9u)·E`, `9·M`, `z0`, `z1` and `S`, which Gappa writes `2 * (rest) * (E)`,
`9007199254741001 * (E)`, `(d0) * (rest)` and `(P1) + (P0)` and so on. Each is
positive under the hypotheses but `z0`, `z1` and `S`. Those three come from
the hints `(P0 − z0) / M -> ((P0 − z0) / z0) * (z0 / M)` and its two twins,
which `stage.v:165-179` holds as `Hypothesis a14 : r101 = r108.`, `a15` and
`a16`. Where `z0` is 0, `P0 = rnd(0) = 0`, and both sides are 0 whatever `x /
0` is taken to be; the same holds for `z1` and for `S`. No `.out` says "no
need" or "trivially". The controls and runs ran without `-Bcoq`.

**8. The scripts, whole, as they ran.** To run them again, save `stage.g`,
`stage_e0.g`, `controls.py` and `all.sh` in one directory with 464.13's
`one.sh`, and run `bash all.sh` there. `controls.py` writes the four edges,
the controls of 6 and the runs from the two scripts. The longest run,
`r_e0_whole.g`, took 51.41 s, as `all.sh`'s print shows. `same.py` is a check,
and `all.sh` does not need it.

`stage.g`:

```gappa
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
cM = 9b-53 * M;
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
XM / cM -> XMM / 9b-53;
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
```

`stage_e0.g`, the incoming errors within `2⁻⁶⁰ M`, stage 1's `E = 0` among
them:

```gappa
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
  /\ (out - Ex) / M in [-9b-53, 9b-53]
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
```

`controls.py`, which writes the four edges, the controls of 6 and the runs:

```python
"""Write stage.g's and stage_e0.g's two share edges, their controls, and the
runs that show what the scripts do not use and why they are cut as they are.
An edge takes share at 0 or at 1, the two values the f64 divide can give
outside the range the main scripts take. Each control changes one thing: a
smaller bound, to see whether the script proved its goal from hypotheses that
contradict each other; or a hypothesis dropped or weakened, to see whether the
proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


SHARE = "  /\\ share in [1b-1074, 9007199254740991b-53]\n"
SPLIT = "# Where 1 - share starts to be exact.\n$ share in (0.5);\n"

stage = open("stage.g").read()
small = open("stage_e0.g").read()

# The edges. With share at 0 or at 1, one of the two incoming errors has no
# weight, so XE / cE needs no average, and there is no half to split at.
AVERAGE = stage[stage.index("# XE / cE through the average"):stage.index("# XM a share at a time")]
DIRECT = ("# share is 0 or 1, so XE / cE needs no average.\n"
          "XE / cE -> (rest * ((d0 - t0) / E) + share * ((d1 - t1) / E)) / (1 + 9b-53);\n\n")
for name, at in (("stage_0.g", "[0, 0]"), ("stage_1.g", "[1, 1]")):
    text = sub(stage, SHARE, "  /\\ share in " + at + "\n")
    text = sub(text, AVERAGE, DIRECT)
    text = sub(text, " /\\ XE / V in ?", "")
    write(name, sub(text, SPLIT, ""))
for name, at in (("stage_e0_0.g", "[0, 0]"), ("stage_e0_1.g", "[1, 1]")):
    text = sub(small, SHARE, "  /\\ share in " + at + "\n")
    write(name, sub(text, SPLIT, ""))

# 8.5u for the M part, below the 8.5u and a hair XMM reaches.
text = sub(stage, "cM = 9b-53 * M;", "cM = 17b-54 * M;")
write("c_stage_85.g", sub(text, "XM / cM -> XMM / 9b-53;", "XM / cM -> XMM / 17b-54;"))
# M allowed below the normal range.
write("c_stage_m.g", sub(stage, "/\\ M in [1b-1022, 1b1024]", "/\\ M in [1b-1074, 1b1024]"))
# Without the incoming error's size against M.
write("c_stage_noe.g", sub(stage, "  /\\ E / M in [1b-60, 1b-40]\n", ""))
# Without share's range: what the case split gives.
write("c_stage_noshare.g", sub(stage, SHARE, ""))
# Without M at least the true running values.
write("c_stage_not.g", sub(stage, "  /\\ t0 / M in [-1, 1] /\\ t1 / M in [-1, 1]\n", ""))
# Without the incoming errors at most E.
write("c_stage_nod.g", sub(stage, "  /\\ (d0 - t0) / E in [-1, 1] /\\ (d1 - t1) / E in [-1, 1]\n", ""))
E0_GOAL = "/\\ (out - Ex) / M in [-9b-53, 9b-53]"
# 8.375u for the small-E case, below the 8.5u it proves.
write("c_e0_8375.g", sub(small, E0_GOAL, "/\\ (out - Ex) / M in [-67b-57, 67b-57]"))
# M allowed below the normal range.
write("c_e0_m.g", sub(small, "/\\ M in [1b-1022, 1b1024]", "/\\ M in [1b-1074, 1b1024]"))
# Without the incoming errors at most 2^-60 M.
write("c_e0_nod.g", sub(small, "  /\\ (d0 - t0) / M in [-1b-60, 1b-60] /\\ (d1 - t1) / M in [-1b-60, 1b-60]\n",
                        ""))

# The runs. Without the parameter between the two knots, and without the
# knots' difference positive and in range: neither is used by Gappa, and both
# enter by hand.
write("r_stage_noa.g", sub(stage, "  /\\ A in [0, 1]\n", ""))
write("r_stage_noknots.g", sub(stage, "  /\\ above - below in [1b-1074, 1b1024]\n", ""))
# The small-E case at 8.5u, which it reaches.
write("r_e0_85.g", sub(small, E0_GOAL, "/\\ (out - Ex) / M in [-17b-54, 17b-54]"))
# stage.g with E allowed to be zero, which is why stage_e0.g exists.
write("r_stage_e_from_0.g", sub(stage, "/\\ E / M in [1b-60, 1b-40]", "/\\ E / M in [0, 1b-40]"))
# stage.g averaging over the weights themselves, which is why it takes twice
# them.
text = sub(stage, "cR = 2 * rest * E; cS = 2 * share * E;", "cR = rest * E; cS = share * E;")
text = sub(text, "XE / cE -> (XE / V) * (2 * (rest + share) / (1 + 9b-53));",
           "XE / cE -> (XE / V) * ((rest + share) / (1 + 9b-53));")
text = sub(text, "rest * (d0 - t0) / cR -> ((d0 - t0) / E) / 2;", "rest * (d0 - t0) / cR -> (d0 - t0) / E;")
write("r_stage_plain.g",
      sub(text, "share * (d1 - t1) / cS -> ((d1 - t1) / E) / 2;", "share * (d1 - t1) / cS -> (d1 - t1) / E;"))
# stage_e0.g with share over all of [0, 1], which is why share is cut in three.
write("r_e0_whole.g", sub(small, SHARE, "  /\\ share in [0, 1]\n"))
```

`all.sh`:

```sh
#!/bin/bash
# Write the edges, the controls and the runs, then run every script at once,
# each through 464.13's one.sh: the two stage scripts and their four edges with
# coq, and the controls and the runs without.
cd "$(dirname "$0")"
python3 controls.py
for f in stage.g stage_0.g stage_1.g stage_e0.g stage_e0_0.g stage_e0_1.g; do
  ./one.sh "$f" coq > "${f%.g}.run" 2>&1 &
done
for f in c_stage_85.g c_stage_m.g c_stage_noe.g c_stage_noshare.g c_stage_not.g c_stage_nod.g \
         c_e0_8375.g c_e0_m.g c_e0_nod.g \
         r_stage_noa.g r_stage_noknots.g r_e0_85.g r_stage_e_from_0.g r_stage_plain.g r_e0_whole.g; do
  ./one.sh "$f" > "${f%.g}.run" 2>&1 &
done
wait
cat stage.run stage_0.run stage_1.run stage_e0.run stage_e0_0.run stage_e0_1.run \
    c_stage_85.run c_stage_m.run c_stage_noe.run c_stage_noshare.run c_stage_not.run \
    c_stage_nod.run c_e0_8375.run c_e0_m.run c_e0_nod.run \
    r_stage_noa.run r_stage_noknots.run r_e0_85.run r_stage_e_from_0.run r_stage_plain.run \
    r_e0_whole.run
```

It printed, on the last run:

```text
stage exit 0 1.96s
stage coq -Bcoq exit 0 coqc exit 0 2.91s lines 4529 hints 21
stage_0 exit 0 0.98s
stage_0 coq -Bcoq exit 0 coqc exit 0 2.52s lines 2572 hints 14
stage_1 exit 0 0.96s
stage_1 coq -Bcoq exit 0 coqc exit 0 2.64s lines 2479 hints 14
stage_e0 exit 0 0.33s
stage_e0 coq -Bcoq exit 0 coqc exit 0 1.84s lines 1825 hints 9
stage_e0_0 exit 0 0.28s
stage_e0_0 coq -Bcoq exit 0 coqc exit 0 1.50s lines 1388 hints 9
stage_e0_1 exit 0 0.26s
stage_e0_1 coq -Bcoq exit 0 coqc exit 1 1.15s lines 1543 hints 9
c_stage_85 exit 1 3.07s
c_stage_m exit 1 3.37s
c_stage_noe exit 1 1.40s
c_stage_noshare exit 1 2.82s
c_stage_not exit 1 2.23s
c_stage_nod exit 1 1.94s
c_e0_8375 exit 1 3.14s
c_e0_m exit 1 37.26s
c_e0_nod exit 1 0.42s
r_stage_noa exit 0 1.85s
r_stage_noknots exit 0 1.84s
r_e0_85 exit 0 0.84s
r_stage_e_from_0 exit 1 2.88s
r_stage_plain exit 1 2.93s
r_e0_whole exit 1 51.41s
```

`same.py`, a check that what this subsection appends is what ran:

```python
"""Check that each script 464.22 appends whole is byte for byte the file that
was run, in q2692/ran/; that one.sh is 464.13's block; that all.log is the
print 464.22 gives for all.sh; and that controls.py, run again in a fresh
directory beside the two stage scripts from the appended text, writes the
scripts that ran.
Run as: python3 same.py <doc> <q2692/ran dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, ran = sys.argv[1], sys.argv[2]
text = open(doc).read()
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)


def section(n):
    start = text.index(f"### 464.{n} ")
    end = text.find("\n### ", start + 1)
    return text[start:] if end < 0 else text[start:end + 1]


sec = section(22)
blocks = FENCED.findall(sec)
print("code blocks in 464.22:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
HEAD = ("# restates: crates/rcad-geom/src/de_boor.rs EVALUATION_ROUNDOFF at 812503ed\n"
        "# restates: crates/rcad-geom/src/de_boor.rs de_boor at 812503ed\n"
        "# restates: crates/rcad-geom/src/de_boor.rs blended at 812503ed\n")
marks = {
    "stage.g": HEAD + "# One coordinate",
    "stage_e0.g": HEAD + "# stage.g's stage",
    "controls.py": '"""Write stage.g\'s and stage_e0.g\'s two share edges',
    "all.sh": "#!/bin/bash\n# Write the edges, the controls and the runs",
    "same.py": '"""Check that each script 464.22 appends whole',
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
    path = os.path.abspath(__file__) if name == "same.py" else os.path.join(ran, name)
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")


def block_of(n, first):
    got = [body for lang, body in FENCED.findall(section(n)) if body.startswith(first)]
    assert len(got) == 1, (n, first, len(got))
    return got[0]


def printed(first):
    got = [body for lang, body in blocks if lang == "text" and body.startswith(first)]
    assert len(got) == 1, (first, len(got))
    return got[0]


print("one.sh is 464.13's block:", open(os.path.join(ran, "one.sh")).read()
      == block_of(13, "#!/bin/bash\n# Run one Gappa script under 464.4's pin"))
print("all.log is what 464.22 gives:",
      open(os.path.join(ran, "all.log")).read() == printed("stage exit "))

fresh = tempfile.mkdtemp()
for name in ("stage.g", "stage_e0.g", "controls.py"):
    open(os.path.join(fresh, name), "w").write(found[name])
subprocess.run(["python3", "controls.py"], cwd=fresh, check=True)
written = sorted(f for f in os.listdir(fresh) if f.endswith(".g") and f not in found)
for name in written:
    print(name, "written again is the one that ran:",
          filecmp.cmp(os.path.join(fresh, name), os.path.join(ran, name), shallow=False))
print(".g files that ran and are neither appended nor written again:",
      sorted(f for f in os.listdir(ran) if f.endswith(".g") and f not in found and f not in written))
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.22: 8
stage.g same
stage_e0.g same
controls.py same
all.sh same
same.py same
one.sh is 464.13's block: True
all.log is what 464.22 gives: True
c_e0_8375.g written again is the one that ran: True
c_e0_m.g written again is the one that ran: True
c_e0_nod.g written again is the one that ran: True
c_stage_85.g written again is the one that ran: True
c_stage_m.g written again is the one that ran: True
c_stage_nod.g written again is the one that ran: True
c_stage_noe.g written again is the one that ran: True
c_stage_noshare.g written again is the one that ran: True
c_stage_not.g written again is the one that ran: True
r_e0_85.g written again is the one that ran: True
r_e0_whole.g written again is the one that ran: True
r_stage_e_from_0.g written again is the one that ran: True
r_stage_noa.g written again is the one that ran: True
r_stage_noknots.g written again is the one that ran: True
r_stage_plain.g written again is the one that ran: True
stage_0.g written again is the one that ran: True
stage_1.g written again is the one that ran: True
stage_e0_0.g written again is the one that ran: True
stage_e0_1.g written again is the one that ran: True
.g files that ran and are neither appended nor written again: []
```

**9. What this leaves.** Rows, filed from this write-up when it merges, each
from Q2692:

- Restate `EVALUATION_ROUNDOFF`'s per-stage count (`de_boor.rs:41-77`)
  against 3 and 4: `share − A` at `2.5u` over the middle range and `3u` at
  `share = 1`; the two coefficient errors as one error, `5u` on `|t1 − t0| ≤
  2M` where the doc has `7u`; `1 − share` exact above one half; the two
  products and the sum at `3.5u`, bounded one at a time, where the doc takes
  `2u` for the three together; `8.5u` in all. None of the doc's terms is shown
  wrong, and the doc's `9u` holds.
- Check `stage_e0_1.g` in Coq. `gappa -Bcoq` writes `Definition p49 := FLT
  _rest (0).` at `stage_e0_1.v` line 468, and `coqc` says `Error: Cannot
  interpret this number as a value of type positive` (7). `stage_1.g` has
  `rest` exactly 0 too, and its `.v` passes. Find why, and get the edge
  checked by Coq or record why it cannot be.

And what is not a row: the sum over fifteen stages, which is Q2696's, in Lean
(464.9); a stage whose running values overflow, which no script here models,
since Gappa's `float<ieee_64, ne>` has no largest number, and which nobody has
run, so it has no class (§360.5); and Q2700, which moves these scripts into
`proofs/gappa/` with the rest of 464's and already waits on Q2692.

### 464.23 POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF's parabola

Q2693 proved the parabola branch of `POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF` in
Gappa on 2026-10-26, for cone slopes `t` from `2⁻⁵⁰⁰` to 5. A point
`Parabola::at` hands back is within `116.05u·L` of the true parabola's point at
the same `s`, under the constant's `128u·L`, which is `64 ε·L`. That holds given
the doc's `4u` for the two directions `and` and `across`, which this does not
prove (5, 9). Past a slope of about 6.7, Gappa's bound on the scalars' share is
over what the other terms leave (3): that is a proof that does not reach that
far, not a kernel that fails there. The doc's count of about `98u·L` is not
what is proved, and 4 says where the two part. Coq checks all five scripts,
modulo their hints (7). The hyperbola branch, with its `cosh` and `sinh`, is
Q2697's.

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, unchanged. The scripts' `restates` lines name `7485253e`.
`walk.rs` there differs from the base `68949f13` only by `c4a2384f`'s 22 doc
lines, all in hunks at or before `7485253e`'s line 2161: `git diff --stat
7485253e 68949f135 -- crates/rcad-geom/src/walk.rs Cargo.lock` printed `1 file
changed, 22 insertions(+)`, so `Cargo.lock` is unchanged. Line numbers below
are `walk.rs`'s at `68949f13`. `u` is `2⁻⁵³` and `ε` is `2u`.

**1. What the proof guards.** `POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`
(`walk.rs:116-220`) bounds how far a point `walk` hands back can sit off the
parabola, or the branch of a hyperbola, a plane cuts a cone in, as a multiple
of `L`. `L` (`walk.rs:137-140`) is the largest of the vertex's and the apex's
coordinates, `|K|`, `|C/K|` and `|h|`; `how_big_this_is`
(`walk.rs:6133-6143`) is handed that list at line 5114. The arithmetic it
guards, on the parabola:

- `where_a_plane_cuts_a_cone` (`walk.rs:4994-5089`) takes exact sums, each
  rounded once, at 5006-5023; the `sqrt` calls at 5030-5031,
  `two_directions_in` at 5032 and the foot at 5033; `h`, how far the apex
  stands off the plane, at 5048, `axis_up` at 5049, `bend_up` at 5050, and `K`,
  worked left to right, at 5051. It tests `leaning == 0.0` at 5052 and works
  `C` out at 5053-5054.
- `the_parabola_a_plane_along_a_ruling_cuts` (`walk.rs:5100-5116`) works
  `off = C/K` out at 5108, the vertex at 5109 and `bend = and·(−1/K)` at 5113.
- `Parabola::at` (`walk.rs:2780-2782`) is `vertex + across·s + (bend·s)·s`.
- `walk_an_endless_bend` (`walk.rs:2611`) walks `s` over
  `±how_far_out_it_is_walked`, which is `1/‖bend‖` (`walk.rs:2788-2790`), or
  between `along_of` (`walk.rs:2784-2786`) of the two named ends
  (`walk.rs:2630-2636`). It works each `s` out at 2648.
- `Vector::dot` is nalgebra's (`point.rs:296-298`), a plain `f64` dot.

**2. The scripts, and how they are cut.** `P(s)` is the point `Parabola::at`
works out. `Q(s) = foot* − and*·offS + across*·s − and*·s²/KS` is the true
parabola's point at the same `f64` `s`, where a name with a `*`, or ending in
`S`, is the same quantity worked out without rounding. The curve's nearest
point to `P(s)` is no farther than `Q(s)`, so `‖P − Q‖` bounds how far `P(s)`
sits off it. `P − Q` is four kinds of term, each bounded by its norm:

- **Ten coordinate roundings**, each at most `u` times the norm of what it
  rounds: `n·q`, `1`, since `h ≤ L`; `apex − n·q`, `√3 + 1`; `and·off`, `1`;
  `foot − and·off`, `√3`; `bend`'s own rounding, times `s²`, `1`; `across·s`,
  `1`; `bend·s` and `(bend·s)·s`, `2`; the first add, `√3 + 1`; the second add,
  `√3 + 2`. That is `10 + 4√3 = 16.93u·L`, using `‖apex‖` and `‖vertex‖ ≤
  √3·L`, and `h`, `|off|`, `|s|` and `|mrk|·s²` all `≤ L`, where `mrk` is the
  computed `1/K`.
- **The foot's quotient**, `n·(q − qS)`, whose norm is `h·|(q − qS)/qS| ≤
  3.125u·L`.
- **The two directions**: `(and − and*)·(off + mrk·s²)`, at `8u·L`, and
  `(across − across*)·s`, at `4u·L`, both on the doc's `4u` for each
  (`walk.rs:161-162`).
- **The scalars' share along `and*`**, `(off − offS) + s²·(mrk − 1/KS)`. Over
  `|KS|` it is `G = (off − offS)/KS + r²·(m − 1)`, with `r = s/KS` and
  `m = mrk·KS`. Below `t = 7/16` the scripts take it over `|offS|` instead,
  where `KS/offS = 4t²/(1 − t²)`; `|KS|` and `|offS|` are equal at
  `t = 1/√5 ≈ 0.447`. `|K|` and `|off|` are in `L`, so `|G| ≤ 84u` puts the
  share at `84u·L`.

The total is `16.928 + 3.125 + 8 + 4 + 84 = 116.05u·L`. The factors `1 + 2⁻⁴⁰`
and `1 + 84u` between the true `|KS|`, `|offS|` and `|s|` and what `L` holds
move that by under `0.01u·L`.

The scripts cut `t` in five, where the share changes the length it is taken
over and where Gappa needs a finer split:

- `s1.g`, `t` in `[2⁻⁵⁰⁰, 1/4]`, over `|offS|`, with no split;
- `s2.g`, `[1/4, 7/16]`, over `|offS|`, split in 16;
- `s3.g`, `[7/16, 1]`, over `|KS|`, split in 32;
- `s4.g`, `[1, 2]`, over `|KS|`, split in 8;
- `s5.g`, `[2, 5]`, over `|KS|`, split in 8.

Each proves `|G| ≤ 84u`, `|(q − qS)/qS| ≤ 3.125u` and `|(K − KS)/KS| ≤ 2⁻⁴⁴`.
The third is what holds `r` (5). The goal on `G` alone does not, since `r²`'s
factor `4t²/(1 − t²)` on `s1.g` goes down to `2⁻⁹⁹⁸`.

The hints: `~` ties each computed name to its true twin. `hS`, `auS`, `buS`,
`c2S` and `KS` are rewritten to their closed forms, `hs`, `t/√(1 + t²)`,
`−1/√(1 + t²)`, `t²` and `−2·hs·t`. `C`'s error is split into `hh`'s, `br`'s
and the product's, and `m − 1` into `mrk`'s and `K`'s. `BIG_HINTS` and
`SMALL_HINTS` split `G` over `|KS|` and over `|offS|`.

**3. The shares.** `G`'s range with no goal and no split, on a stretch of `t`,
as `all.sh` printed it (8), in `u`:

- `p_s1`, `t` in `[2⁻⁵⁰⁰, 1/4]`: `51.69u`;
- `p_s4`, `[1, 2]`: `48.28u`;
- `p_7_16_below`, `[0.4296875, 0.4375]`: `76.07u`;
- `p_7_16_above`, `[0.4375, 0.4453125]`: `80.52u`;
- `p_5`, `[4.96875, 5]`: `76.21u`;
- `p_8`, `[7.9375, 8]`: `110.33u`;
- `p_16`, `[15.875, 16]`: `200.95u`;
- `p_32`, `[31.75, 32]`: `381.12u`.

`84u` is above the largest of those up to `t = 5`, `80.52u`, just above `7/16`.
Past 5 the bound nearly doubles with each doubling of `t`. The other terms of 2
leave `128 − 32.05 = 95.947u` for `G`, and a straight line between `p_5` and
`p_8` passes `95.947u` at `t = 6.735`. That is a line through two probes, not
a probe at 6.735. `c_s5_to8.g` (6), `s5.g` with `t` up to 8, fails with a best
of `84u` to `115.74u`. Whether the kernel's points stray that far past 5 is
not run (9).

**4. What the doc counts differently.** The doc (`walk.rs:142-167`) charges
the foot `12u·L` (142-143); `n̂·ŵ` and `and·ŵ` a relative `10u` each (144-146);
`K` `25u` (147-148); `C` `2u·L` (149-154); `C/K` `25u·L`, which with the
vertex's own roundings makes the vertex `43u·L` (155-157); `bend` `27u` (158);
evaluating, `15u·L` plus `across`'s `4u` and `bend`'s `27u` (159-162); and the
parameter `9u·L` (163-165): about `98u·L` in all (167). `116.05u·L` is a
bound, not a measurement, so it does not show `98u` wrong. Where the two part:

- (i) `bend_up` is a plain `f64` dot (`walk.rs:5050`, through
  `point.rs:296-298`), not the exactly-summed dot lines 144-146 say. The
  scripts take the computed `and·w` within `7.5u·|w|` of the true one: the
  doc's `4u` on `and`, the plain dot's `3u`, and `0.5u` over. Over
  `|and*·w| = |w|/√(1 + t²)` that is a relative `7.5u·√(1 + t²)`, which is
  `38u` at `t = 5`, where the doc has `10u`.
- (ii) The parameter's `9u·L` drops out. The proof compares `P(s)` with `Q(s)`
  at the same `s`, so an error in `s` slides the point along the curve and not
  off it, which is what lines 102-103 already say of the angle.
- (iii) The doc counts a coordinate and the proof a norm, so a rounding of a
  point is `u` times up to `√3·L`, where the doc has `u·L`.
- (iv) The doc's `C`, `C/K` and `bend` terms come to `2 + 25 + 27 = 54u·L`.
  `G`, which holds all three, reaches `80.52u` just above `t = 7/16`.

**5. The hypotheses.** Each script's, in its order:

- `hs`, `nN` and `wW` in `[1, 2]`. Scaling any of them by a power of two
  leaves `G` and both ratios as they are, while every value stays normal. `hh`
  leaves the normal range for `|h|` below `2⁻⁵¹¹` (9).
- They are positive. Negating `h`, `n·w` or `and·w` negates the computed and
  the true value of each name it reaches, exactly, since round-to-nearest is
  symmetric, and leaves `G` as it is. That was checked through `c1`, `c2`,
  `br`, `C`, `K`, `off`, `mrk` and `m`.
- `t` an `f64` in the script's range, as `t = rnd(t_)` says.
- `(Bw − Bws)/wW` within `±7.5u`, the computed `and·w` of 4 (i). This is where
  the doc's `4u` on `and` comes in, unproved (9).
- `r` within `±(1 + 2⁻⁴⁰)`, written `1099511627777b-40`. A run with no named
  ends walks `|s| ≤ 1/‖bend‖`, which is within about `10u` of `|K|` by hand,
  with three roundings at line 2648 on top, and the third goal gives
  `|K/KS − 1| ≤ 2⁻⁴⁴`. With named ends nothing holds `s` (9).
- `E = 0` is written in, through `As` and `Bs`: the plane lies exactly along a
  ruling. A leaning that is not zero but rounds to `0.0` takes the parabola
  branch at line 5052, and is not modelled (9).

**6. The controls.** Each changes one thing and exits 1, as `all.sh` printed
(8):

- `c_s4_40.g`, `s4.g` with its goal on `G` at `40u`, after 266.16 s, with a
  best of `40u` to `48.28u`. So `s4.g`'s hypotheses do not contradict each
  other.
- `c_s1_nobw.g`, `s1.g` without the hypothesis on `Bw`, with a best of
  `[-inf, inf]` on `|G|` and on `|(K − KS)/KS|`.
- `c_s1_nor.g`, `s1.g` without the range on `r`, with a best of `[-inf, inf]`
  on `|G|`.
- `c_s5_to8.g`, `s5.g` with `t` up to 8, with a best of `84u` to `115.74u`
  (3).

The goal control on the small shape, `s1.g` at `40u`, ran past `one.sh`'s
600 s, so it is not here. `s1.g`'s hypotheses meet at a point instead:
`hs = nN = wW = 1`, `t = 1/8`, `Bw` the `f64` nearest `−8/√65`, and `r = 0`.
The hypotheses are ranges on six independent inputs, and this point is inside
each. The same point with `t` moved into each range, and `Bw` the `f64`
nearest `−1/√(1 + t²)`, meets `s2.g` to `s5.g`'s.

Earlier runs, whose files are not appended and which `all.sh` does not run:
`c_s1_40`, `s1.g` at `40u`, exited 124 at 600.02 s; `c_s2_72` and `c_s3_76`,
`s2.g` at `72u` and `s3.g` at `76u`, exited 124 at 600.03 s; `c_s5_nobw`
exited 1 at 464.38 s and `c_s3_nor` exited 1 at 403.78 s.

**7. Coq checks all five, modulo their hints.** `gappa -Bcoq` and `coqc`,
through `one.sh`, exit 0 on all five. On the last run of `all.sh` (8) their
`.v` files came to 5393 lines with 14 hints, 6683 with 14, 9842 with 15, 7508
with 15 and 11388 with 15, and every `.coqc` file, which holds what `coqc`
printed, is empty.

As in 464.22's 7, each hint enters the `.v` as a hypothesis with no condition,
so a proof stands on its hints and on the divisors Gappa took to be nonzero.
`ids.py` (8) works out each difference Gappa prints for a hint it calls "not
trivially equal", in exact rationals, at twenty points, with the square roots
taken. It finds 7, 7, 9, 9 and 9 such hints in `s1.out` to `s5.out`, and every
one zero at all twenty points. Its control is `s3.out` with only line 22,
`hS`'s difference, changed from `(hs) * (nN)` to `(hs) * (wW)`. On it,
`ids.py` prints `9 hints not trivially equal, 8 of them zero at all twenty
points ['hS']`.

`grep -c 'assumed to be nonzero'` counts 75 in each of `s1.out` and `s2.out`,
and 90 in each of `s3.out` to `s5.out`. `s1.out`'s, by divisor:
`√(1 + t²)` 18, `nlS·wlS` 12, `K` 9, `nlS` 8, `wlS` 6, `hs²·nN²` 4;
`−2t·hs·nN²·wW²·(1 + t²)`, expanded, 3; `1 − t²` 2; `t²·(1 + t²)·nN²·wW²`,
expanded, 2; `√(1 + t²)⁴·K·nlS⁴·wlS⁴` 2; `hh·br` 2; three expansions of
`(1 + t²)(1 − t²)` times a positive factor, 2, 2 and 1; `1 − c2` 1; and `C` 1.
`s2.out`'s are the same. `s3.out`'s: `√(1 + t²)` 22, `nlS·wlS` 13, `K` 13,
`nlS` 10, `wlS` 9, `−2t·hs·nN²·wW²·(1 + t²)` 8, `hs²·nN²` 4, `4t²` 3, the same
`−2t·…` times `K` 3, `√(1 + t²)⁴·K·nlS⁴·wlS⁴` 2, `t²·(1 + t²)·nN²·wW²` 1,
`hh·br` 1 and `C` 1, and `s4.out`'s and `s5.out`'s are the same. Each is
nonzero under its script's hypotheses but `hh·br` and `C` in `s3.g` to `s5.g`.
Those are zero where `c2` rounds to exactly 1, near `t = 1`. There `C = 0` and
`off = 0`, so both sides of the hint `(off − C/K)/KS -> …` are 0, and both
sides of `(C − CS)/hhS -> …` are `−brS`, whatever `x/0` is taken to be.

**8. The scripts, whole, as they ran.** To run them again, save
`parabola.py`, `ids.py` and `all.sh` in one directory with 464.13's `one.sh`,
and run `bash all.sh` there. `parabola.py` writes the five scripts, the
controls of 6 and the probes of 3. The `.g` files are not appended:
`parabola.py` writes them, and `same.py` checks that what it writes is what
ran. The longest run, `c_s4_40.g`, took 266.16 s, as `all.sh`'s print shows.
`same.py` is a check, and `all.sh` does not need it.

`parabola.py`:

```python
"""Write 464.23's five scripts, s1.g to s5.g, their controls and the probes
that show how far the chain reaches. Each script is the same chain, the scalars
of the parabola a plane along a ruling cuts a cone in, over one range of the
slope t; the ranges are cut where the share G is taken over a different length
and where Gappa needs a finer split. Each control changes one thing in one
script and must fail. Each probe asks for G's range, with no goal, on a short
stretch of t."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

HEAD = """# restates: crates/rcad-geom/src/walk.rs POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF at 7485253e
# restates: crates/rcad-geom/src/walk.rs where_a_plane_cuts_a_cone at 7485253e
# restates: crates/rcad-geom/src/walk.rs the_parabola_a_plane_along_a_ruling_cuts at 7485253e
# restates: Cargo.lock nalgebra at 7485253e
# The scalars of the parabola a plane along a ruling cuts a cone in, as
# where_a_plane_cuts_a_cone and the_parabola_a_plane_along_a_ruling_cuts work
# them out in f64, against the same scalars without rounding, whose names end
# in S. hs is the apex's true height over the plane, nN and wW the true lengths
# of the plane's normal n and of the cone's axis w, t the cone's slope. R, Nn,
# Ww and U are the exact sums walk.rs rounds once each. Bw is the f64 dot of
# the computed `and` with w, and Bws the true one, -|w| / sqrt(1 + t^2) on a
# parabola. G is what the scalars put into the point at s = r * KS along `and`,
# over |KS| or over |offS|; q is the foot's quotient.
"""

CHAIN = """@rnd = float<ieee_64, ne>;

t = rnd(t_);
Bw = rnd(Bw_);

Ts = 1 + t * t;
As = t / sqrt(Ts);
Bs = -1 / sqrt(Ts);

R = hs * nN;
Nn = nN * nN;
Ww = wW * wW;
U = As * (nN * wW);
Bws = Bs * wW;

reach = rnd(R); sqn = rnd(Nn); sqw = rnd(Ww); up = rnd(U);
q = rnd(reach / sqn);
nl = rnd(sqrt(sqn)); wl = rnd(sqrt(sqw));
h = rnd(reach / nl);
nw = rnd(nl * wl);
au = rnd(up / nw);
bu = rnd(Bw / wl);
tt = rnd(t * t);
t2 = rnd(1 + tt);
k1 = rnd(2 * t2);
k2 = rnd(k1 * h);
k3 = rnd(k2 * au);
K = rnd(k3 * bu);
hh = rnd(h * h);
c1 = rnd(t2 * au);
c2 = rnd(c1 * au);
br = rnd(1 - c2);
C = rnd(hh * br);
off = rnd(C / K);
mrk = rnd(1 / K);

qS = R / Nn;
nlS = sqrt(Nn); wlS = sqrt(Ww);
hS = R / nlS;
nwS = nlS * wlS;
auS = U / nwS;
buS = Bws / wlS;
t2S = 1 + t * t;
k1S = 2 * t2S;
k2S = k1S * hS;
k3S = k2S * auS;
KS = k3S * buS;
hhS = hS * hS;
c1S = t2S * auS;
c2S = c1S * auS;
brS = 1 - c2S;
CS = hhS * brS;
offS = CS / KS;
m = mrk * KS;
"""

# t at least 7/16: G is the scalars' share along `and` over |KS|.
BIG = """G = (off - offS) / KS + r * r * (m - 1);
"""

# t at most 7/16: the same share over |offS|, which is the larger there.
SMALL = """G = (off - offS) / offS + r * r * ((4 * t * t) / (1 - t * t)) * (m - 1);
"""

T = "  /\\ t in [TLO, THI]\n"
BW = "  /\\ (Bw - Bws) / wW in [-15b-54, 15b-54]\n"
RS = "  /\\ r in [-1099511627777b-40, 1099511627777b-40]\n"
GOAL = "  |G| <= 84b-53 /\\ |(q - qS) / qS| <= 25b-56 /\\ |(K - KS) / KS| <= 1b-44\n"

HYP = """
{
     hs in [1, 2]
  /\\ nN in [1, 2] /\\ wW in [1, 2]
""" + T + BW + RS + """->
""" + GOAL + """}

reach ~ R; sqn ~ Nn; sqw ~ Ww; up ~ U; Bw ~ Bws; q ~ qS;
nl ~ nlS; wl ~ wlS; h ~ hS; nw ~ nwS; au ~ auS; bu ~ buS;
tt ~ t * t; t2 ~ t2S; k1 ~ k1S; k2 ~ k2S; k3 ~ k3S; K ~ KS;
hh ~ hhS; c1 ~ c1S; c2 ~ c2S;

hS -> hs;
auS -> As;
buS -> Bs;
c2S -> t * t;
KS -> -2 * hs * t;

(C - CS) / hhS -> ((C - hh * br) / (hh * br)) * ((hh * br) / hhS) + ((hh - hhS) / hhS) * br + (br - brS);
br - brS -> (br - (1 - c2)) - (c2 - c2S);
c2 - c2S -> ((c2 - c2S) / c2S) * (t * t);
m - 1 -> ((mrk - 1 / K) / (1 / K)) * (KS / K) - (K - KS) / K;
KS / K -> 1 / (1 + (K - KS) / KS);
(K - KS) / K -> ((K - KS) / KS) / (1 + (K - KS) / KS);
"""

BIG_HINTS = """G -> (off - C / K) / KS + (C - CS) / (K * KS) + r * r * (((mrk - 1 / K) / (1 / K)) * (KS / K))
     - ((1 - t * t) / (4 * t * t) + r * r) * ((K - KS) / K);
(off - C / K) / KS -> ((off - C / K) / (C / K)) * ((C / K) / KS);
(C / K) / KS -> (C - CS) / (K * KS) + ((1 - t * t) / (4 * t * t)) * (KS / K);
(C - CS) / (K * KS) -> ((C - CS) / hhS) * ((KS / K) / (4 * t * t));
"""

SMALL_HINTS = """G -> ((off - C / K) / (C / K)) * (1 + (C - CS) / CS) * (KS / K) + ((C - CS) / CS) * (KS / K)
     + r * r * ((4 * t * t) / (1 - t * t)) * (((mrk - 1 / K) / (1 / K)) * (KS / K))
     - (1 + r * r * ((4 * t * t) / (1 - t * t))) * ((K - KS) / K);
(C - CS) / CS -> ((C - hh * br) / (hh * br)) * (1 + (hh - hhS) / hhS) * (1 + (br - brS) / brS)
     + ((hh - hhS) / hhS) * (1 + (br - brS) / brS) + (br - brS) / brS;
(br - brS) / brS -> ((br - (1 - c2)) / (1 - c2)) * (1 - ((c2 - c2S) / c2S) * ((t * t) / (1 - t * t)))
     - ((c2 - c2S) / c2S) * ((t * t) / (1 - t * t));
"""


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def script(lo, hi, small, split):
    body, hints = (SMALL, SMALL_HINTS) if small else (BIG, BIG_HINTS)
    text = HEAD + CHAIN + body + HYP + hints
    text = sub(text, "TLO", lo)
    text = sub(text, "THI", hi)
    return text + ("\n$ t in " + split + ";\n" if split else "")


def write(name, text):
    open(name, "w").write(text)


SCRIPTS = {
    "s1.g": script("1b-500", "0.25", True, None),
    "s2.g": script("0.25", "0.4375", True, "16"),
    "s3.g": script("0.4375", "1", False, "32"),
    "s4.g": script("1", "2", False, "8"),
    "s5.g": script("2", "5", False, "8"),
}
for name, text in SCRIPTS.items():
    write(name, text)

# The controls. A goal under what s4 reaches, to see that its hypotheses do
# not contradict each other; then a hypothesis dropped or widened, to see that
# the proof needs it. The small shape's goal control, s1.g at 40u, ran past
# one.sh's 600 s, so it is not here: 464.23's 6 gives a point that meets every
# hypothesis instead.
G84 = "|G| <= 84b-53"
write("c_s4_40.g", sub(SCRIPTS["s4.g"], G84, "|G| <= 40b-53"))
write("c_s1_nobw.g", sub(SCRIPTS["s1.g"], BW, ""))
write("c_s1_nor.g", sub(SCRIPTS["s1.g"], RS, ""))
write("c_s5_to8.g", sub(SCRIPTS["s5.g"], "t in [2, 5]", "t in [2, 8]"))

# The probes: G with no goal, no split, on a short stretch of t.
PROBE_GOAL = "  G in ?\n"
write("p_s1.g", sub(SCRIPTS["s1.g"], GOAL, PROBE_GOAL))
write("p_s4.g", sub(SCRIPTS["s4.g"], GOAL, PROBE_GOAL))
for name, lo, hi, small in (
    ("p_7_16_below.g", "0.4296875", "0.4375", True),
    ("p_7_16_above.g", "0.4375", "0.4453125", False),
    ("p_5.g", "4.96875", "5", False),
    ("p_8.g", "7.9375", "8", False),
    ("p_16.g", "15.875", "16", False),
    ("p_32.g", "31.75", "32", False),
):
    write(name, sub(script(lo, hi, small, None), GOAL, PROBE_GOAL))
```

`ids.py`, the check on the hints of 7:

```python
"""Check that each hint Gappa could not prove by itself is an identity once the
square roots are taken: sqrt(nN * nN) = nN and sqrt(wW * wW) = wW, since both
lengths are positive, and sqrt(1 + t * t) is S. For each "not trivially
equal" warning in each script's .out, the Difference Gappa prints is worked
out in exact rationals at twenty points with t = (a^2 - b^2) / (2ab), so that
S = (a^2 + b^2) / (2ab) is rational too, and every other name a random
rational. A difference that is not zero as a rational function is zero at
such a point only by an accident of its numerator, which the twenty points
make vanishingly unlikely; this is a check, not a proof.
Run as: python3 ids.py s1.out s2.out s3.out s4.out s5.out"""
import random
import re
import sys
from fractions import Fraction

random.seed(2693)
WARNING = re.compile(r"Warning: (.*?) and (.*?) are not trivially equal\.\n\s*Difference: (.*)")


class Point(dict):
    def __missing__(self, name):
        value = Fraction(random.randint(1, 10**12), random.randint(1, 10**12))
        self[name] = value
        return value


def at_a_point(expr):
    a, b = random.randint(2, 10**9), random.randint(1, 10**9)
    if a <= b:
        a, b = b + 1, a
    point = Point()
    point["t"] = Fraction(a * a - b * b, 2 * a * b)
    point["S"] = Fraction(a * a + b * b, 2 * a * b)
    assert point["S"] ** 2 == 1 + point["t"] ** 2 and point["S"] > 0
    point["nN"] = point["nlS"] = Fraction(random.randint(1, 10**12), random.randint(1, 10**12))
    point["wW"] = point["wlS"] = Fraction(random.randint(1, 10**12), random.randint(1, 10**12))
    return eval(expr, {"__builtins__": {}}, point)


for out in sys.argv[1:]:
    found = WARNING.findall(open(out).read())
    nonzero = []
    for left, right, difference in found:
        expr = difference.replace("sqrt(1 + t * t)", "S").replace("^", "**")
        assert "sqrt" not in expr, expr
        if any(at_a_point(expr) != 0 for _ in range(20)):
            nonzero.append(left)
    print(out, len(found), "hints not trivially equal,", len(found) - len(nonzero),
          "of them zero at all twenty points", nonzero if nonzero else "")
```

`all.sh`:

```bash
#!/bin/bash
# Write the five scripts, the controls and the probes, then run every one at
# once, each through 464.13's one.sh: the five with coq, the controls and the
# probes without. A control's line is followed by the best range Gappa found
# for what it could not prove, and a probe's by the range it printed for G.
# Last, ids.py checks the hints of the five that Gappa could not prove itself.
cd "$(dirname "$0")"
python3 parabola.py
for f in s1.g s2.g s3.g s4.g s5.g; do
  ./one.sh "$f" coq > "${f%.g}.run" 2>&1 &
done
for f in c_s4_40.g c_s1_nobw.g c_s1_nor.g c_s5_to8.g \
         p_s1.g p_s4.g p_7_16_below.g p_7_16_above.g p_5.g p_8.g p_16.g p_32.g; do
  ./one.sh "$f" > "${f%.g}.run" 2>&1 &
done
wait
for c in c_s4_40 c_s1_nobw c_s1_nor c_s5_to8; do
  grep ', best: ' "$c.out" >> "$c.run"
done
for p in p_s1 p_s4 p_7_16_below p_7_16_above p_5 p_8 p_16 p_32; do
  grep '^  G in ' "$p.out" \
    | sed -E 's/\[[-0-9]+b[-0-9]+ \{([^,]+), [^}]*\}, [-0-9]+b[-0-9]+ \{([^,]+), [^}]*\}\]/[\1, \2]/' >> "$p.run"
done
cat s1.run s2.run s3.run s4.run s5.run \
    c_s4_40.run c_s1_nobw.run c_s1_nor.run c_s5_to8.run \
    p_s1.run p_s4.run p_7_16_below.run p_7_16_above.run p_5.run p_8.run p_16.run p_32.run
python3 ids.py s1.out s2.out s3.out s4.out s5.out
```

It printed, on the last run:

```text
s1 exit 0 2.40s
s1 coq -Bcoq exit 0 coqc exit 0 5.39s lines 5393 hints 14
s2 exit 0 2.45s
s2 coq -Bcoq exit 0 coqc exit 0 6.52s lines 6683 hints 14
s3 exit 0 18.95s
s3 coq -Bcoq exit 0 coqc exit 0 69.44s lines 9842 hints 15
s4 exit 0 1.99s
s4 coq -Bcoq exit 0 coqc exit 0 6.27s lines 7508 hints 15
s5 exit 0 10.21s
s5 coq -Bcoq exit 0 coqc exit 0 27.97s lines 11388 hints 15
c_s4_40 exit 1 266.16s
  BND(|G|), best: [4.44089e-15, 5.35968e-15]
c_s1_nobw exit 1 219.55s
  BND(|G|), best: [-inf, inf]
  BND(|(K - KS) / KS|), best: [-inf, inf]
c_s1_nor exit 1 153.53s
  BND(|G|), best: [-inf, inf]
c_s5_to8 exit 1 158.81s
  BND(|G|), best: [9.32587e-15, 1.285e-14]
p_s1 exit 0 2.98s
  G in [-5.73909e-15, 5.73909e-15]
p_s4 exit 0 21.06s
  G in [-5.35968e-15, 5.35968e-15]
p_7_16_below exit 0 3.58s
  G in [-8.44565e-15, 8.44565e-15]
p_7_16_above exit 0 3.62s
  G in [-8.93913e-15, 8.93913e-15]
p_5 exit 0 3.74s
  G in [-8.46127e-15, 8.46127e-15]
p_8 exit 0 3.67s
  G in [-1.22487e-14, 1.22487e-14]
p_16 exit 0 2.81s
  G in [-2.23102e-14, 2.23102e-14]
p_32 exit 0 3.13s
  G in [-4.23131e-14, 4.23131e-14]
s1.out 7 hints not trivially equal, 7 of them zero at all twenty points 
s2.out 7 hints not trivially equal, 7 of them zero at all twenty points 
s3.out 9 hints not trivially equal, 9 of them zero at all twenty points 
s4.out 9 hints not trivially equal, 9 of them zero at all twenty points 
s5.out 9 hints not trivially equal, 9 of them zero at all twenty points 
```

`same.py`, a check that what this subsection appends is what ran:

```python
"""Check that each script 464.23 appends whole is byte for byte the file that
was run, in q2693/ran/; that one.sh is 464.13's block; that all.log is the
print 464.23 gives for all.sh; and that parabola.py, run again in a fresh
directory from the appended text, writes every .g file that ran.
Run as: python3 same.py <doc> <q2693/ran dir>"""
import filecmp
import os
import re
import shutil
import subprocess
import sys
import tempfile

doc, ran = sys.argv[1], sys.argv[2]
text = open(doc).read()
FENCED = re.compile(r"^```(\w+)\n(.*?)^```$", re.S | re.M)


def section(n):
    start = text.index(f"### 464.{n} ")
    end = text.find("\n### ", start + 1)
    return text[start:] if end < 0 else text[start:end + 1]


sec = section(23)
blocks = FENCED.findall(sec)
print("code blocks in 464.23:", len(blocks))
assert len(re.findall(r"^```", sec, re.M)) == 2 * len(blocks)

# Which appended block is which file: by its first lines.
marks = {
    "parabola.py": '"""Write 464.23\'s five scripts',
    "ids.py": '"""Check that each hint Gappa could not prove',
    "all.sh": "#!/bin/bash\n# Write the five scripts, the controls and the probes",
    "same.py": '"""Check that each script 464.23 appends whole',
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
    path = os.path.abspath(__file__) if name == "same.py" else os.path.join(ran, name)
    print(name, "same" if open(path).read() == found[name] else "DIFFERS")


def block_of(n, first):
    got = [body for lang, body in FENCED.findall(section(n)) if body.startswith(first)]
    assert len(got) == 1, (n, first, len(got))
    return got[0]


def printed(first):
    got = [body for lang, body in blocks if lang == "text" and body.startswith(first)]
    assert len(got) == 1, (first, len(got))
    return got[0]


print("one.sh is 464.13's block:", open(os.path.join(ran, "one.sh")).read()
      == block_of(13, "#!/bin/bash\n# Run one Gappa script under 464.4's pin"))
print("all.log is what 464.23 gives:",
      open(os.path.join(ran, "all.log")).read() == printed("s1 exit "))

fresh = tempfile.mkdtemp()
open(os.path.join(fresh, "parabola.py"), "w").write(found["parabola.py"])
subprocess.run(["python3", "parabola.py"], cwd=fresh, check=True)
written = sorted(f for f in os.listdir(fresh) if f.endswith(".g"))
for name in written:
    print(name, "written again is the one that ran:",
          filecmp.cmp(os.path.join(fresh, name), os.path.join(ran, name), shallow=False))
print(".g files that ran and are not written again:",
      sorted(f for f in os.listdir(ran) if f.endswith(".g") and f not in written))
shutil.rmtree(fresh)
```

It prints:

```text
code blocks in 464.23: 6
parabola.py same
ids.py same
all.sh same
same.py same
one.sh is 464.13's block: True
all.log is what 464.23 gives: True
c_s1_nobw.g written again is the one that ran: True
c_s1_nor.g written again is the one that ran: True
c_s4_40.g written again is the one that ran: True
c_s5_to8.g written again is the one that ran: True
p_16.g written again is the one that ran: True
p_32.g written again is the one that ran: True
p_5.g written again is the one that ran: True
p_7_16_above.g written again is the one that ran: True
p_7_16_below.g written again is the one that ran: True
p_8.g written again is the one that ran: True
p_s1.g written again is the one that ran: True
p_s4.g written again is the one that ran: True
s1.g written again is the one that ran: True
s2.g written again is the one that ran: True
s3.g written again is the one that ran: True
s4.g written again is the one that ran: True
s5.g written again is the one that ran: True
.g files that ran and are not written again: []
```

**9. What this leaves.** Rows, filed from this write-up when it merges, each
from Q2693:

- Restate `POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF` (`walk.rs:116-220`) against 2
  to 4: `bend_up` is a plain `f64` dot (`walk.rs:5050`), not the
  exactly-summed dot lines 144-146 say, and its relative error grows like
  `√(1 + t²)`; the per-term count becomes 2's, which reaches `64 ε` for slopes
  up to 5; and the parameter's `9u·L` (163-165) goes, since an error in `s`
  slides a point along the curve and not off it. It is a row of its own
  because `walk.rs` is outside this row's write set.
- Probe the kernel on the parabola at slopes from 6 to 32, where 3's chain
  does not prove `64 ε`: walk a plane along a ruling of such a cone, and
  measure each point against the true parabola in exact arithmetic.
- Probe a parabola run whose named ends lie past `K`: nothing holds its `s` to
  `|K|` (5), and the doc's evaluating term assumes it (159-162).
- Probe the parabola branch where its arithmetic leaves the normal range: `|h|`
  below `2⁻⁵¹¹`, where `hh` underflows (5053-5054); a slope below `2⁻⁵⁰⁰`,
  where the scripts stop; and a plane and cone whose exact leaning is not zero
  but rounds to `0.0`, so the parabola branch is taken (5052) for a curve that
  is not a parabola.
- Prove or probe `two_directions_in` (`walk.rs:5346-5369`) against the true
  directions on a parabola. 2 takes the doc's `4u` for `and` and `across` as a
  hypothesis, and the cross product they are built from has length
  `1/√(1 + t²)` of `|w|·|n|` there, so their error may grow with the slope.

And what is not a row: Q2700, which moves these scripts into `proofs/gappa/`
with the rest of 464's and already waits on Q2693; the hyperbola branch, which
is Q2697's; and a run whose values overflow, which no script here models,
since Gappa's `float<ieee_64, ne>` has no largest number. Nobody has run the
kernel on any of these shapes, so none of them has a class (§360.5).

### 464.24 A distance off a plane, and a plane's shift, with one product below the normals

Q2694 asked whether 464.9's `signed.g` and `shiftplane.g` still hold with `L`
at least `2⁻¹⁰²²` and one of the three products below `2⁻¹⁰²²`, which 464.9
left unrun. Four lanes took it on 2026-10-26. The first wrote the first
`signedcases.py`, whose runs failed (6). The second made every case prove, and
ran the controls, `coqc` and a probe through the kernel. The third ran all of
it again and wrote this. The fourth ran the cases, the controls and `coqc` a
third time.

With one product below the normals and the other two normal, both bounds hold
at the code's constants: `signed_distance_to` within `8u·L`, and a plane's or a
half-space's shifted offset within `16u·|n|·L` (2). They need no condition on
`L` of their own, since the two normal products already keep `L`, or `|n|·L`
for the shift, near `2⁻¹⁰²²` or above (3). Coq checks all twelve, modulo their
hints (4). A half-space whose normal is `2⁻⁶⁰` long, shifted by `(2⁻¹⁰²², 0,
0)`, does not move at all where it should move `0.6·L`: for a half-space, `L`
normal is not enough (5). Two or three products below the normals, and a zero
factor, are not run (What this leaves).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, unchanged, and `coqc` under 464.5's pins with gappalib
1.8.0. The code is the base `d084a409`'s. `signed_distance_to`
(`plane.rs:398-400`), `HalfSpace::shifted` (`half_space.rs:244-249`) and the
two constants (`plane.rs:85-112`, `moved_corner.rs:99-143`) do the same
arithmetic there as at 464.9's `c3b2160e`: `git diff --stat c3b2160e d084a409`
on `plane.rs`, `half_space.rs` and `moved_corner.rs` names only `plane.rs`, and
its hunks touch doc comments and one code comment. `u` is `2⁻⁵³` and `ε` is
`2u`.

**1. The cases.** `signedcases.py` (below) writes one script for each product
`k` in 0, 1, 2 and each sign, as 464.8's `planecases.py` did for `plane.g`:
`signeds<k><sign>.g` from `signed.g` and `shifts<k><sign>.g` from
`shiftplane.g`, `<sign>` empty for positive and `n` for negative. Twelve in
all. In each:

- the exact product `X_k = n_k·p_k` (`n_k·d_k` for the shift) is in `[2⁻²¹⁴⁸,
  2⁻¹⁰²²]`, or its negative;
- the other two products round to at least `2⁻¹⁰²²`, and each carries 464.5's
  lemma;
- no factor of the three products is zero, which the first two already say,
  since a double that is not zero is at least `2⁻¹⁰⁷⁴`;
- the rest is `signed.g`'s or `shiftplane.g`'s: `L` at least `2⁻¹⁰²²` and at
  least every coordinate and the offset, no sum cancelling to zero, and the
  bound on `|n₀| + |n₁| + |n₂|`.

Below `2⁻¹⁰²²` a rounding is no longer a slip relative to what it rounds, so
what rounding `X_k` costs is carried as an amount: `a_k`, which is `t_k − X_k`
times the roundings after it, and so at most `2⁻¹⁰⁷⁵` times them. 464.9's
averaging gives each term a share of the bound `D`, `|nᵢ|·L` for a product and
`L/4` for the offset, and proves each term's error within `4u` of its share.
The amount is paid out of the share of a product that is normal, since only a
normal product holds its `|nᵢ|·L` at least `2⁻¹⁰²²`, give or take a rounding:

- For `k` 0 or 1, out of product 2's. Product 2 meets three roundings, its
  own, `s012`'s and `value`'s, where products 0 and 1 meet four, so `u·|n₂|·L`
  of its share is unused, and that is at least `u·2⁻¹⁰²² = 2⁻¹⁰⁷⁵` give or
  take a rounding. The shares are 464.9's, so `D/L ≤ 1774/1024 + 1/4`, and
  `4u·D` is `7.93u·L`.
- For `k = 2`, half out of product 0's and half out of product 1's, each grown
  to `9/8·|nᵢ|·L`. Product 2's own rounding is now the amount, so it meets two
  roundings and its share drops to `|n₂|·L/2`. Then `D/L = 9/8·|n₀| +
  9/8·|n₁| + 1/2·|n₂| + 1/4`, and Cauchy-Schwarz with those weights, on a
  normal whose `n·n` is within `64ε` of one, puts the first three under
  `√178/8·√(1 + 64ε) < 1708/1024`. So `4u·D` is `7.67u·L`.

The shift's scripts are the same with `|n|·L` for `L`, as `shiftplane.g` is
`signed.g`'s, and both sums are bounded over `|n|`.

**2. They prove.** `allcases.sh` writes the twelve and runs them through
`runall.sh`. Each exits 0, in under a second, and prints `a_k` within
`±2⁻¹⁰⁷⁵`, and `(value − Ex) / D` within `±2⁻⁵¹` as Gappa rounds it to print,
which is `4u` and a term of order `u²`. So `signed_distance_to` is within
`7.93u·L` for `k` 0 or 1 and `7.67u·L` for `k = 2`, under the code's `8u·L`,
and the shifted offset within the same multiples of `|n|·L`, under the code's
`16u·|n|·L`. The first lane also ran 464.9's `signed.g` at the base: it still
proves, and `coqc` accepts it.

**3. The controls.** `casecontrols.py` changes one thing in each case, and
`allcontrols.sh` runs them:

- `c_<case>_below.g` asks for just under what the averaging reaches, `63b-56`
  (`7.875u`) for `k` 0 or 1 and `61b-56` (`7.625u`) for `k = 2`. All twelve
  exit 1, with a best of `±8.80372e-16`, which is `7.93u`, for `k` 0 or 1, and
  `±8.51749e-16`, `7.67u`, for `k = 2`, signed and shift alike. So the
  hypotheses do not contradict each other, and the bound proved is the
  averaging's.
- `c_<case>_sumonly.g`, for `k = 2`, bounds the weighted sum only by `9/8` of
  the plain sum's `1774/1024`, which is all that bound gives. The two signed
  ones exit 1, best `±9.76541e-16`, `8.8u`, over the code's `8u`. The two shift
  ones exit 0, since their goal is `16u`. So the signed `k = 2` case needs the
  weighted Cauchy-Schwarz bound, that is, a `Plane`'s normal being near unit
  length.
- `c_<case>_anyl.g` lets `L` go down to `2⁻¹⁰⁷⁴`. All twelve exit 0. A normal
  product, `|nᵢ·pᵢ| ≥ 2⁻¹⁰²²` with `|pᵢ| ≤ L` and `|nᵢ| ≤ 1774/1024`, keeps `L`
  at least about `2⁻¹⁰²²/1.733`; for the shift, `|nᵢ|·L ≥ |nᵢ·dᵢ|` keeps
  `|n|·L` at least about `2⁻¹⁰²²`. So with one product below the normals the
  condition is the other two being normal, and nothing on `L` besides.

**4. Coq checks all twelve, modulo their hints.** `allcoq.sh` runs `gappa
-Bcoq` and `coqc` through `one.sh` on the twelve and on the twelve `_anyl`
controls, and both exit 0 on all 24. Over the second, third and fourth lanes'
runs the `.v` files came to 3200 to 5018 lines, with 18 hints for `k` 0 or 1
and 19 for `k = 2`, and the two steps took 1.55 to 4.06 s a script. As in
464.22's 7, each hint enters the `.v` as a hypothesis with no condition, so a
proof stands on its hints and on the divisors Gappa took to be nonzero. No
`.out` holds anything but those divisor warnings and the results, so Gappa
checked every hint's two sides equal by algebra. The divisors, 65 to 71
warnings a run, are `S01`, `S012` and `y`, which a hypothesis keeps from zero;
and `L`, `N`, `|nᵢ|`, `|nᵢ|·L`, each product `nᵢ·pᵢ`, the `tᵢ` of a normal
product, and `D`'s expansions, each of which is not zero since no factor is.

**5. A half-space through the kernel.** A probe test, appended to
`crates/rcad-geom/tests/reading_a_distance_off_a_plane_stays_inside_its_bound.rs`,
run with `just t q2694_probe_a_short_normal_shifted_by_the_least_normal
--no-capture` and removed, by the second lane and again by the third, printed:

```text
normal [5.204170427930421e-19, 6.938893903907229e-19, 0.0], |n| / 2^-60 1, L = 2^-1022, offset after 0e0 (bits 0x0), true offset / (|n| L) 0.6, SURFACE_AFTER_SHIFTING_ROUNDOFF 1.7763568394002505e-15
```

```rust
#[test]
fn q2694_probe_a_short_normal_shifted_by_the_least_normal() {
    let short = 2f64.powi(-60);
    let half =
        rcad_geom::HalfSpace::new(rcad_geom::Vector::new(0.6 * short, 0.8 * short, 0.0), 0.0)
            .unwrap();
    let least_normal = f64::MIN_POSITIVE;
    let shifted = half.shifted(rcad_geom::Vector::new(least_normal, 0.0, 0.0));
    let n = half.normal().components();
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    println!(
        "normal {n:?}, |n| / 2^-60 {}, L = 2^-1022, offset after {:e} (bits {:#x}), true offset / (|n| L) {}, SURFACE_AFTER_SHIFTING_ROUNDOFF {:?}",
        length / short,
        shifted.offset(),
        shifted.offset().to_bits(),
        n[0] / length,
        rcad_geom::SURFACE_AFTER_SHIFTING_ROUNDOFF
    );
}
```

`HalfSpace::new` stores the normal `(0.6, 0.8, 0)·2⁻⁶⁰` and the offset 0 as
given (`half_space.rs:72-90`), and `HalfSpace::shifted` adds `normal·by` to the
offset. The true new offset is `0.6·2⁻¹⁰⁸²`, under half of `2⁻¹⁰⁷⁴`, so it
rounds to 0. The offset slips `0.6·|n|·L`, which is `0.6·L` as a distance,
against the `16u·|n|·L` the constant promises, with `L = 2⁻¹⁰²²` normal. One
product, `n₀·d₀`, is below the normals and the other two are zero, so the case
is outside the twelve, which ask the other two to be normal. For a half-space,
whose normal is any length, the condition on a shift has to be on `|n|·L` or
on the products, not on `L`. Whether a `Plane`, whose normal is near unit
length, misses with two or three products below the normals is the row below.
This is class wrong/D (§352.3): a product below `2⁻¹⁰²²` needs numbers far
outside §3.5's envelope, here `L = 2⁻¹⁰²²`, `2.2e-310` times its nominal `100`,
and `|n| = 2⁻⁶⁰`. What it makes false is the doc on `moved_corner.rs:99-143`,
which Q2695 restates.

**6. Why the first runs failed.** The first `signedcases.py`, which is not
appended, made `signeds0.g` and `signeds2.g` exit 1 with `(value − Ex) / D`
near `±2¹⁹⁹⁸`: its hint `|n2| * L -> |n2 * p2| / |p2 / L|` needs `|ab| =
|a|·|b|`, which Gappa cannot check, so the floor under `|n₂|·L` never reached
the proof, and the case's own factors were not held nonzero, so its share
`|n_k|·L` could be zero. The hint is now `a0 / (|n2| * L) -> (a0 / t2) * rp2 *
((n2 / |n2|) * (p2 / L))`, which Gappa checks by algebra, every factor is held
at least `2⁻¹⁰⁷⁴`, and the amount is named `a<k>`, since `d0` and `d2` had
named the shift too, and `shifts0.g` and `shifts2.g` did not parse.

**What this leaves.**

- Two or three products below `2⁻¹⁰²²`, or a zero factor, for `signed.g` with
  `L` at least `2⁻¹⁰²²` and for `shiftplane.g` with `|n|·L` at least
  `2⁻¹⁰²²`, split by where each product lands as 464.18's `planeall.py` split
  `plane.g`: a row. The half-space of 5 is one such case.
- Q2695 restates the doc comments of `plane.rs:85-112` and
  `moved_corner.rs:99-143` with the condition the proofs need, and now waits on
  that row. For a half-space that condition is on `|n|·L` or on the products,
  not on `L` (5). That is Q2695's to state, not a new row.
- Q2700 moves these scripts into `proofs/gappa/` with the rest of §464's.

**The scripts, whole, as they ran.** To run them again, save
`signedcases.py`, `casecontrols.py`, `allcases.sh`, `runall.sh`,
`allcontrols.sh` and `allcoq.sh` in one directory with 464.13's `one.sh`, make
the `.sh` files executable, and run `./allcases.sh`, then `./allcontrols.sh`
and `./allcoq.sh`, in that order, since `casecontrols.py` reads the cases
`signedcases.py` writes. The `.g` files are not appended: `signedcases.py`
writes them.

`signedcases.py`, whole:

```python
"""Write signed.g's and shiftplane.g's cases with one product below the
normals, as 464.8's planecases.py did for plane.g: signeds<k><sign>.g and
shifts<k><sign>.g for product k in 0, 1, 2 and its sign, "" positive or "n"
negative, the other two products normal, and what rounding product k cost
carried as an amount a<k>, not a ratio.

That amount is at most 2^-1075 times what rounds after it, and it is spent
out of another product's share of the bound, which that product being normal
keeps at least 2^-1022 / (1 + u): product 2's for k 0 or 1, since product 2
meets one rounding fewer than the bound gives it; for k 2, half each out of
products 0 and 1, whose shares are then 9/8 of |n_i| L, with product 2's,
which then meets two roundings, 1/2.

Run as: python3 signedcases.py; it writes beside itself."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
# What each product meets after its own rounding: two sums and value for
# products 0 and 1, one sum and value for product 2.
AFTER = {0: "rv * rs012 * rs01", 1: "rv * rs012 * rs01", 2: "rv * rs012"}

SIGNED = {
    "name": "signeds",
    "about": "signed.g",
    "q": "p",
    "check": "signed_distance_to does not check",
    "y": "y = s012 - off;",
    "e3": "e3 = -(off * W3);",
    "e3_hint": "e3 / c3 -> -(4 * (off / L) * W3);",
    "size": "L",
    "c3": "c3 = 1b-2 * L;",
    "off_bound": "off / L in [-1, 1]",
    "sum": "|n0| + |n1| + |n2| in [0, 1774b-10]",
    "sum_why": [
        "  # A stored normal has n.n within NORMAL_LENGTH_ROUNDOFF = 64 epsilon of",
        "  # one, so |n0| + |n1| + |n2| <= sqrt(3 (1 + 64 epsilon)) < 1774 / 1024.",
    ],
    "weighted": "9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2| in [0, 1708b-10]",
    "weighted_why": [
        "  # By Cauchy-Schwarz the same way, 9/8 |n0| + 9/8 |n1| + 1/2 |n2| <=",
        "  # sqrt(178) / 8 sqrt(1 + 64 epsilon) < 1708 / 1024.",
    ],
    "ratio": "D / L -> |n0| + |n1| + |n2| + 1b-2;",
    "ratio2": "D / L -> 9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2| + 1b-2;",
    "goal": "(value - Ex) / L in [-1b-50, 1b-50]",
}

SHIFT = {
    "name": "shifts",
    "about": "shiftplane.g",
    "q": "d",
    "check": "neither caller checks",
    "y": "y = off + s012;",
    "e3": "e3 = off * W3;",
    "e3_hint": "e3 / c3 -> 4 * (off / (N * L)) * W3;",
    "size": "(N * L)",
    "c3": "c3 = 1b-2 * (N * L);",
    "off_bound": "off / (N * L) in [-1, 1]",
    "sum": "(|n0| + |n1| + |n2|) / N in [0, 1774b-10]",
    "sum_why": [
        "  # |n0| + |n1| + |n2| <= sqrt(3) |n| < 1774 / 1024 |n|, by Cauchy-Schwarz.",
    ],
    "weighted": "(9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N in [0, 1708b-10]",
    "weighted_why": [
        "  # By Cauchy-Schwarz the same way, 9/8 |n0| + 9/8 |n1| + 1/2 |n2| <=",
        "  # sqrt(178) / 8 |n| < 1708 / 1024 |n|.",
    ],
    "ratio": "D / (N * L) -> (|n0| + |n1| + |n2|) / N + 1b-2;",
    "ratio2": "D / (N * L) -> (9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N + 1b-2;",
    "goal": "(value - Ex) / (N * L) in [-1b-49, 1b-49]",
}


def make(s, k, neg):
    q = s["q"]
    size = s["size"]
    others = [i for i in (0, 1, 2) if i != k]
    sign_word = "negative" if neg else "positive"
    prod = {i: f"X{k}" if i == k else f"n{i} * {q}{i}" for i in (0, 1, 2)}
    lines = [
        f"# {s['about']} where n{k} * {q}{k} is below the normals, which",
        f"# {s['check']}: n{k} * {q}{k} is under 2^-1022 and {sign_word}, and the",
        "# other two products are normal.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);",
        f"{q}0 = rnd({q}0_); {q}1 = rnd({q}1_); {q}2 = rnd({q}2_);",
        "off = rnd(off_);",
        "",
        f"X{k} = n{k} * {q}{k};",
        "; ".join(f"t{i} = rnd({prod[i]})" for i in (0, 1, 2)) + ";",
        "S01 = t0 + t1;",
        "s01 = rnd(S01);",
        "S012 = s01 + t2;",
        "s012 = rnd(S012);",
        s["y"],
        "value = rnd(y);",
        "",
    ]
    if s is SIGNED:
        lines.append("Ex = " + " + ".join(prod[i] for i in (0, 1, 2)) + " - off;")
    else:
        lines.append("Ex = off + " + " + ".join(prod[i] for i in (0, 1, 2)) + ";")
        lines.append("N = sqrt(n0 * n0 + n1 * n1 + n2 * n2);")
    lines += [
        "",
        "; ".join(f"rp{i} = t{i} / (n{i} * {q}{i})" for i in others) + ";",
        "rs01 = s01 / S01; rs012 = s012 / S012; rv = value / y;",
        "",
        f"# Each term as value carries it, less the term itself, with X{k} for t{k};",
        f"# and what rounding X{k} cost, as an amount.",
    ]
    for i in (0, 1, 2):
        lines.append(f"W{i} = {AFTER[i]} - 1;" if i == k else f"W{i} = {AFTER[i]} * rp{i} - 1;")
    lines.append("W3 = rv - 1;")
    lines.append("; ".join(f"e{i} = {prod[i]} * W{i}" for i in (0, 1, 2)) + "; " + s["e3"])
    lines.append(f"a{k} = (t{k} - X{k}) * ({AFTER[k]});")
    lines.append("")
    if k == 2:
        lines += [
            "# Each term's share of the bound. a2 is spent half out of product 0's",
            "# share and half out of product 1's, each 9/8 of |n_i| L; product 2",
            "# meets two roundings, so 1/2 of |n2| L is its share.",
            "c0 = 9b-3 * (|n0| * L); c1 = 9b-3 * (|n1| * L); c2 = 1b-1 * (|n2| * L); " + s["c3"],
        ]
    else:
        lines += [
            f"# Each term's share of the bound, as in {s['about']}. a{k} is spent out of",
            "# product 2's share, since product 2 meets one rounding fewer.",
            "c0 = |n0| * L; c1 = |n1| * L; c2 = |n2| * L; " + s["c3"],
        ]
    lines += [
        "D = c0 + c1 + c2 + c3;",
        "",
        "{",
        "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024",
        f"  /\\ |{q}0| <= 1b1024 /\\ |{q}1| <= 1b1024 /\\ |{q}2| <= 1b1024",
        "  /\\ |off| <= 1b1024",
        "  # No factor of the three products is zero.",
        "  /\\ |n0| >= 1b-1074 /\\ |n1| >= 1b-1074 /\\ |n2| >= 1b-1074",
        f"  /\\ |{q}0| >= 1b-1074 /\\ |{q}1| >= 1b-1074 /\\ |{q}2| >= 1b-1074",
        "  # No sum cancels to zero.",
        "  /\\ |S01| >= 1b-1074 /\\ |S012| >= 1b-1074 /\\ |y| >= 1b-1074",
        f"  # The case: X{k} is {sign_word} and below the normals, the other two",
        "  # products are normal. A product of two doubles that are not zero is at",
        "  # least 2^-2148.",
        f"  /\\ X{k} in " + ("[-1b-1022, -1b-2148]" if neg else "[1b-2148, 1b-1022]"),
        "  /\\ " + " /\\ ".join(f"|t{i}| >= 1b-1022" for i in others),
        "  # What lemma.g proves each of those two gives.",
    ]
    for i in others:
        lines.append(f"  /\\ t{i} -/ n{i} * {q}{i} {LEMMA}")
    lines += [
        "  # L is at least 2^-1022, and at least every coordinate and the offset.",
        "  /\\ L in [1b-1022, 1b1024]",
        f"  /\\ {q}0 / L in [-1, 1] /\\ {q}1 / L in [-1, 1] /\\ {q}2 / L in [-1, 1]",
        "  /\\ " + s["off_bound"],
    ]
    lines += s["sum_why"]
    lines.append("  /\\ " + s["sum"])
    if k == 2:
        lines += s["weighted_why"]
        lines.append("  /\\ " + s["weighted"])
    lines += [
        "  # x / |x| is in [-1, 1] for any x that is not zero.",
        "  /\\ n0 / |n0| in [-1, 1] /\\ n1 / |n1| in [-1, 1] /\\ n2 / |n2| in [-1, 1]",
        "->",
        f"  a{k} in ? /\\ (value - Ex) / D in ?",
        "  /\\ " + s["goal"],
        "}",
        "",
        "# How far each rounding is from the identity.",
    ]
    for i in others:
        lines.append(f"t{i} / (n{i} * {q}{i}) -> 1 + (t{i} - n{i} * {q}{i}) / (n{i} * {q}{i});")
    lines += [
        "s01 / S01 -> 1 + (s01 - S01) / S01;",
        "s012 / S012 -> 1 + (s012 - S012) / S012;",
        "value / y -> 1 + (value - y) / y;",
        "",
        f"# The error is the sum of the four terms' errors and a{k}, each a fraction",
        "# of its share of D; add_rr then averages them.",
        f"(value - Ex) / {size} -> ((value - Ex) / D) * (D / {size});",
        s["ratio2"] if k == 2 else s["ratio"],
        "(value - Ex) / D -> ((D + (value - Ex)) - D) / D;",
    ]
    if k == 2:
        lines += [
            "D + (value - Ex) -> (c0 + (e0 + 1b-1 * a2)) + (c1 + (e1 + 1b-1 * a2)) + (c2 + e2) + (c3 + e3);",
            "(c0 + (e0 + 1b-1 * a2)) - c0 -> e0 + 1b-1 * a2;",
            "(c1 + (e1 + 1b-1 * a2)) - c1 -> e1 + 1b-1 * a2;",
            "(c2 + e2) - c2 -> e2;",
            "(c3 + e3) - c3 -> e3;",
            f"(e0 + 1b-1 * a2) / c0 -> 8 / 9 * ((n0 / |n0|) * ({q}0 / L) * W0) + 4 / 9 * (a2 / (|n0| * L));",
            f"(e1 + 1b-1 * a2) / c1 -> 8 / 9 * ((n1 / |n1|) * ({q}1 / L) * W1) + 4 / 9 * (a2 / (|n1| * L));",
            f"e2 / c2 -> 2 * ((n2 / |n2|) * ({q}2 / L) * W2);",
            s["e3_hint"],
            "# Product i being normal keeps |n_i| L at least |t_i| / (1 + u), and",
            "# |t_i| at least 2^-1022.",
            f"a2 / (|n0| * L) -> (a2 / t0) * rp0 * ((n0 / |n0|) * ({q}0 / L));",
            f"a2 / (|n1| * L) -> (a2 / t1) * rp1 * ((n1 / |n1|) * ({q}1 / L));",
        ]
    else:
        lines += [
            f"D + (value - Ex) -> (c0 + e0) + (c1 + e1) + (c2 + (e2 + a{k})) + (c3 + e3);",
            "(c0 + e0) - c0 -> e0;",
            "(c1 + e1) - c1 -> e1;",
            f"(c2 + (e2 + a{k})) - c2 -> e2 + a{k};",
            "(c3 + e3) - c3 -> e3;",
            f"e{others[0]} / c{others[0]} -> (n{others[0]} / |n{others[0]}|) * ({q}{others[0]} / L) * W{others[0]};",
            f"e{k} / c{k} -> (n{k} / |n{k}|) * ({q}{k} / L) * W{k};",
            f"(e2 + a{k}) / c2 -> (n2 / |n2|) * ({q}2 / L) * W2 + a{k} / (|n2| * L);",
            s["e3_hint"],
            "# Product 2 being normal keeps |n2| L at least |t2| / (1 + u), and |t2|",
            "# at least 2^-1022.",
            f"a{k} / (|n2| * L) -> (a{k} / t2) * rp2 * ((n2 / |n2|) * ({q}2 / L));",
        ]
    return "\n".join(lines) + "\n"


for s in (SIGNED, SHIFT):
    for k in (0, 1, 2):
        for neg in (False, True):
            name = f"{s['name']}{k}{'n' if neg else ''}"
            open(f"{name}.g", "w").write(make(s, k, neg))
```

`casecontrols.py`, whole, run beside the twelve:

```python
"""Write the controls of signedcases.py's twelve cases, run beside them. Each
changes one thing in a case: a smaller bound, to see whether it proved its
goal from hypotheses that contradict each other; the weighted sum of k = 2
bounded only as far as the plain sum bounds it, to see whether the proof needs
the unit normal; or L allowed below 2^-1022, to see whether the proof needs L
normal."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


GOAL = {"signeds": "(value - Ex) / L in [-1b-50, 1b-50]",
        "shifts": "(value - Ex) / (N * L) in [-1b-49, 1b-49]"}
# Just under what the averaging reaches: (1774/1024 + 1/4) 4u = 7.93u for k 0
# or 1, (1708/1024 + 1/4) 4u = 7.67u for k 2.
BELOW = {0: "63b-56", 1: "63b-56", 2: "61b-56"}
WEIGHTED = {"signeds": "9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2| in [0, 1708b-10]",
            "shifts": "(9b-3 * |n0| + 9b-3 * |n1| + 1b-1 * |n2|) / N in [0, 1708b-10]"}

for name in ("signeds", "shifts"):
    for k in (0, 1, 2):
        for sign in ("", "n"):
            case = f"{name}{k}{sign}"
            src = open(f"{case}.g").read()
            lhs = GOAL[name].split(" in ")[0]
            b = BELOW[k]
            write(f"c_{case}_below.g", sub(src, GOAL[name], f"{lhs} in [-{b}, {b}]"))
            write(f"c_{case}_anyl.g", sub(src, "L in [1b-1022, 1b1024]", "L in [1b-1074, 1b1024]"))
            if k == 2:
                # 9/8 of the plain sum's 1774/1024 is all that bound gives.
                write(f"c_{case}_sumonly.g",
                      sub(src, WEIGHTED[name], WEIGHTED[name].replace("1708b-10", "15966b-13")))
```

`allcases.sh`, whole, beside 464.13's `one.sh`:

```sh
#!/bin/bash
# Write the twelve cases and run each through runall.sh.
cd "$(dirname "$0")"
python3 signedcases.py
./runall.sh signeds0.g signeds0n.g signeds1.g signeds1n.g signeds2.g signeds2n.g \
  shifts0.g shifts0n.g shifts1.g shifts1n.g shifts2.g shifts2n.g
```

`runall.sh`, whole:

```sh
#!/bin/bash
# Run each script named on the command line through one.sh, the second
# argument to one.sh being $COQ, and print each one's results, without the
# nonzero warnings.
cd "$(dirname "$0")"
for f in "$@"; do
  ./one.sh "$f" $COQ
  grep -v '^Warning: the expression\|^ *$' "${f%.g}.out" | head -20
done
```

`allcontrols.sh`, whole:

```sh
#!/bin/bash
# Write the controls and run each through one.sh, printing its exit and the
# line Gappa ends a failure with.
cd "$(dirname "$0")"
python3 casecontrols.py
for f in c_signeds*_below.g c_shifts*_below.g c_signeds2*_sumonly.g c_shifts2*_sumonly.g c_signeds*_anyl.g c_shifts*_anyl.g; do
  ./one.sh "$f"
  grep -A1 '^Error' "${f%.g}.out" | grep -v '^Error'
done
```

`allcoq.sh`, whole:

```sh
#!/bin/bash
# Run gappa -Bcoq and coqc, through one.sh, on each case and on each case with
# L allowed below 2^-1022, all of which prove.
cd "$(dirname "$0")"
for f in signeds0.g signeds0n.g signeds1.g signeds1n.g signeds2.g signeds2n.g \
  shifts0.g shifts0n.g shifts1.g shifts1n.g shifts2.g shifts2n.g \
  c_signeds0_anyl.g c_signeds0n_anyl.g c_signeds1_anyl.g c_signeds1n_anyl.g c_signeds2_anyl.g c_signeds2n_anyl.g \
  c_shifts0_anyl.g c_shifts0n_anyl.g c_shifts1_anyl.g c_shifts1n_anyl.g c_shifts2_anyl.g c_shifts2n_anyl.g; do
  ./one.sh "$f" coq
done
```

### 464.25 Two or more of the nine differences at zero

Q2747 ran block 1 with two or more of the nine differences `adx` to `cdz` at
zero at once, in three lanes on 2026-10-26. Each of the 237 branches proves at
the code's `B` and fails below the bound it proves, and Coq checks each proof,
modulo its hints. Nothing in `orient3d.rs` needs to change for it.

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, with `block1.g` lines 358 to 534 of this file, against
`on_floats` as 464.4 read it at `d9f931e8`: no commit touches `orient3d.rs`
between 464.11's `ca9b6493` and `492b00d2`.

**Which sets are branches.** Of the 502 sets of two or more differences, 237
leave `y` not zero by its shape: all 36 pairs, 78 of the 84 threes, 81 of the
126 fours, 36 of the 126 fives, 6 of the 84 sixes, and none of seven or more.
In the other 265 every term is zero by its shape, so `value` is zero and
`decide` certifies no sign. A term is zero by its shape when its z difference
is zero, or when both products of its minor have a zero factor, as `adx` and
`bdx` do for `third`.

**The scripts.** `make_differences_zero.py`, below, writes each branch as
464.11's `make_one_difference_zero.py` writes one difference: on each axis
with a zero difference, `d`'s coordinate is the same double as the point's;
each zero difference, each product with it, and each term zero by its shape is
left unrounded; and a term whose product is zero stays in `P6` and in the
split of `P6 + (y − Ex)` as a bare `c_k`, with no error of its own. A minor
whose two products are both zero is zero, and it and its `q` are left
unrounded. A minor with one product zero rounds a double less a zero, and its
`q` a double plus a zero, as in 464.11. The generator does two things more.

1. **A `q` that rounds a double plus a zero where its term is zero too.**
   464.11 takes such a `q`'s rounding as off by at most `u + 2u²`, as a
   hypothesis, and a goal checks that `|q| ≥ 2⁻¹⁰²²`, for a minor whose term
   is not zero. Here a z difference can zero the term of a minor with one
   product zero, as `bdz` with `adx` does to `second`. Gappa still needs that
   `q`'s rounding, since `P −/ Phat` sets `r2 = q2 × |bdz|` against
   `(|cdxady| + |adxcdy|) × |bdz|`, both zero. 86 of the 237 sets have such a
   minor and not both of `t1` and `t2` zero. With the hypothesis only where
   the term is not zero, as the generator stood at `84462b71`, 82 of them ran
   and each failed at `REL(P, Phat)`; the other 4 did not run. Run again,
   `dz_adx_bdz` fails after 52.91 s with best `[-9.62462e+494, 3.16494e+451]`
   and `dz_adz_bdx` after 52.27 s with best `[-1.54015e+453, 4.31583e+451]`.
   `dz_adx_bdz` with `q2 −/ (|cdxady| + |adxcdy|)` and `r2 −/ Phat2` asked as
   goals, in place of the one on `eb`, fails at those two and then at
   `REL(P, Phat)`, as 464.11's `REL(q2, ...)` did. The 28 other sets with such
   a minor proved, since they have `t1` and `t2` both zero and carry `r1` and
   `r2` as amounts (2 below). With the hypothesis and its goal wherever a
   minor has one product zero, `dz_adx_bdz` proves in 2.19 s and `dz_adz_bdx`
   in 2.39 s, and so does each of the 86.
2. **`t1` and `t2` both zero**, in 57 sets. `P` then adds two zeros first, and
   Gappa finds no `REL(P, Phat)`: two of the three parts it would average are
   zero. The script asks for `(P − Phat) / Phat` in its place, splits `Phat`
   into `Phat1`, `Phat2` and `Phat3`, and carries `r1` and `r2` as amounts
   against `Phat`, which `Phat3` keeps away from zero, as `make_two_zero.py`
   does. With block 1's hints alone, `dz_adz_bdz` failed at `REL(P, Phat)`.
   With `P` written `rnd((r1 + r2) + r3)` and `(P − Phat) / Phat` asked for,
   but without the amounts, it proved only by leaning on the goal's own
   interval: asked as `(y − Ex) / eb in ?` it got `±1.39141e+1218`, and the
   goals `(P − Phat) / Phat in [-1b-50, 1b-50]` and `Phat / P in [0, 2]` each
   ran out the 600 s timeout. With the amounts it proves in 4.46 s.

**What runs, at the code's `B`.** Each of the 237 proves, with `(y − Ex) / P6`
within:

- `7.77156e-16`, `7u` and some `u²`, for 4: `adx` with `cdz`, `ady` with
  `cdz`, `bdx` with `cdz` and `bdy` with `cdz`. At `6u` each fails with best
  `±1.16667`, and at `5u` with `±1.4`.
- `6.66134e-16`, `6u` and some `u²`, for 51, such as `adx` with `bdx` and
  `adz` with `bdz`. At `6u` each fails with best `±1`, and at `5u` with
  `±1.2`.
- `5.55112e-16`, `5u` and some `u²`, for 182. At `6u` each proves, and at `5u`
  each fails with best `±1`.

In each of the 237 the count of `u` is the count of roundings on the longest
path through a term of `y` that is not zero: the three differences, the
product, the minor where its other product is not zero, the term, and the sum
of the first two terms where the term is one of them and the other is not
zero. Seven is block 1's own count (464.4). `summarize.py`, below, works the
count out from the set alone, and its groups split the 237 the same way the
printed bounds do.

`P −/ Phat` is within `4.44089e-16` in the 180 that ask for it, and
`(P − Phat) / Phat` within `3.33067e-16` in the 57 with `t1` and `t2` both
zero.

Every one of the 237 fails at `5u`, and the 55 whose bound is above `6u` fail
there too, so each fails below the bound it proves, and none proves its goal
from hypotheses that contradict each other (464.7). The 222 `dzq_` scripts,
each a script with its `q` hypotheses taken out and only its `|q|` goals left,
each prove: no goal leans on the hypothesis it backs.

`gappa -Bcoq` writes each of the 237 proofs, 4017 to 10070 lines with 16 to
31 hints, and `coqc` under 464.5's commands accepts each, exit 0. Gappa takes
1.17 to 9.11 s on a script, and `-Bcoq` and `coqc` together 3.39 to 17.16 s.
`field` has not been run on these hints (464.10).

`make_differences_zero.py`, run as `python3 make_differences_zero.py` beside
`block1.g`:

```python
"""Write block 1 with two or more of the nine differences adx to cdz zero.

With no arguments, write dz_<names>.g for every set of two or more whose y is
not zero by its shape, and print the names. With arguments, each a set
written as adx_bdx, write that set alone. Beside each, write the controls
c_dz_<names>_6u.g and c_dz_<names>_5u.g, the script at B = 6u and 5u, and,
where a minor rounds a double less a zero, dzq_<names>.g: the script with
its q hypotheses taken out and only its |q| goals left.
"""
import itertools
import sys

DIFFERENCES = ["adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz"]
# The six products: name, the two differences it multiplies, the minor it is in.
PRODUCT = {
    1: ("bdxcdy", "bdx", "cdy", 1),
    2: ("cdxbdy", "cdx", "bdy", 1),
    3: ("cdxady", "cdx", "ady", 2),
    4: ("adxcdy", "adx", "cdy", 2),
    5: ("adxbdy", "adx", "bdy", 3),
    6: ("bdxady", "bdx", "ady", 3),
}
MINOR = {1: "first", 2: "second", 3: "third"}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}
# The z difference each term multiplies its minor by.
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"


def main():
    if sys.argv[1:]:
        sets = [arg.split("_") for arg in sys.argv[1:]]
    else:
        sets = [
            list(zero)
            for n in range(2, 10)
            for zero in itertools.combinations(DIFFERENCES, n)
            if zero_terms(set(zero)) != {1, 2, 3}
        ]
    for zero in sets:
        name = "dz_" + "_".join(zero)
        src = make(set(zero))
        write(f"{name}.g", src)
        write(f"c_{name}_6u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 6b-53;"))
        write(f"c_{name}_5u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 5b-53;"))
        if "  /\\ q" in src:
            write(f"dzq_{'_'.join(zero)}.g", q_goals_alone(src))
        print(name)


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def dead_products(zero):
    return {k for k, (_, f, g, _) in PRODUCT.items() if f in zero or g in zero}


def zero_terms(zero):
    """The terms that are zero by their shape: a z difference zero, or both
    products of the minor zero."""
    dead = dead_products(zero)
    return {m for m in (1, 2, 3) if Z_OF[m] in zero or {2 * m - 1, 2 * m} <= dead}


def make(zero):
    names = [d for d in DIFFERENCES if d in zero]
    dead = dead_products(zero)
    terms = zero_terms(zero)
    # A minor that is not zero but has one product zero rounds a double less
    # a zero, or a zero less a double.
    half = [m for m in (1, 2, 3) if m not in terms and len({2 * m - 1, 2 * m} & dead) == 1]
    # Its q rounds a double plus a zero, whether its term is zero or not.
    half_q = [m for m in (1, 2, 3) if len({2 * m - 1, 2 * m} & dead) == 1]
    src = open("block1.g").read()
    src = sub(
        src,
        "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
        "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
        f"# Block 1 where {' and '.join(names)} are zero. A zero rounds to itself,\n"
        "# and so does a product with a zero factor.\n",
    )
    for axis in "xyz":
        points = [p for p in "abc" if f"{p}d{axis}" in zero]
        if points:
            src = sub(src, f"d{axis} = rnd(d{axis}_);", f"d{axis} = {points[0]}{axis};")
            for p in points[1:]:
                src = sub(src, f"{p}{axis} = rnd({p}{axis}_);", f"{p}{axis} = {points[0]}{axis};")
    for name in names:
        exact = name[0].upper() + name[1:]
        src = sub(src, f"{name} = rnd({exact});", f"{name} = {exact};")
        src = drop_def(src, f"r{name} = {name} / {exact};")
        src = drop_conj(src, f"|{name}| >= 1b-1074")
        src = sub(src, f"{name} / {exact} -> 1 + ({name} - {exact}) / {exact};\n", "")
        if name in Z_OF.values():
            src = drop_conj(src, f"{name} / |{name}| in [-1, 1]")
    for k in sorted(dead):
        prod, f, g, _ = PRODUCT[k]
        factors = f"{f} * {g}"
        src = sub(src, f"{prod} = rnd({factors});", f"{prod} = {factors};")
        src = drop_def(src, f"rp{k} = {prod} / ({factors});")
        src = drop_conj(src, f"|{prod}| >= 1b-1022")
        src = sub(src, f"  /\\ {prod} -/ {factors} {BOUND}", "")
        src = drop_conj(src, f"{prod} / |{prod}| in [-1, 1]")
        src = sub(src, f"{prod} / ({factors}) -> 1 + ({prod} - {factors}) / ({factors});\n", "")
    for m in sorted(terms):
        minor = MINOR[m]
        if {2 * m - 1, 2 * m} <= dead:
            src = sub(src, f"{minor} = rnd(M{m});", f"{minor} = M{m};")
            src = sub(src, f"q{m} = rnd({NOTE[m]});", f"q{m} = {NOTE[m]};")
        src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
        src = sub(src, f"r{m} = rnd(R{m});", f"r{m} = R{m};")
        src = drop_def(src, f"rm{m} = {minor} / M{m};")
        src = drop_def(src, f"rt{m} = t{m} / U{m};")
        src = drop_conj(src, f"|{minor}| >= 1b-1074")
        src = drop_conj(src, f"|t{m}| >= 1b-1022")
        src = drop_conj(src, f"|r{m}| >= 1b-1022")
        src = sub(src, f"  /\\ t{m} -/ U{m} {BOUND}", "")
        src = sub(src, f"  /\\ r{m} -/ R{m} {BOUND}", "")
        src = sub(src, f"{minor} / M{m} -> 1 + ({minor} - M{m}) / M{m};\n", "")
        src = sub(src, f"t{m} / U{m} -> 1 + (t{m} - U{m}) / U{m};\n", "")
    alive = [m for m in (1, 2, 3) if m not in terms]
    src = sub(
        src,
        "  # The permanent's own three products are normal too. The comment above\n"
        "  # stays_accurate argues it from the nine.\n",
        f"  # Each of {', '.join(f'r{m}' for m in alive)} is normal, because its t is and its minor is\n"
        f"  # at most its q. Each of {', '.join(f'r{m}' for m in sorted(terms))} is zero.\n",
    )
    src = sub(
        src,
        "  # No factor of the nine products is zero. For an f64 that is the same as\n"
        "  # being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {' and '.join(names)}. For an\n"
        "  # f64 that is the same as being at least the smallest subnormal.\n",
    )
    src = sub(
        src,
        "  # §12.2's first condition: product_is_normal on the nine products.\n",
        "  # §12.2's first condition: product_is_normal on the nine products. Those\n"
        "  # on a zero pass, and say nothing.\n",
    )
    tail = ""
    if {1, 2} <= terms:
        # Both of t1 and t2 are zero, so s12 is too and nothing rounds; and
        # both of r1 and r2, so their sum in P rounds nothing either.
        src = sub(src, "s12 = rnd(S);", "s12 = S;")
        src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P = rnd((r1 + r2) + r3);")
        src = sub(src, "rs = s12 / S;\n", "")
        src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
        src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
    elif terms & {1, 2}:
        (m,) = terms & {1, 2}
        other = 3 - m
        tail += f"\n# t{m} is zero, so t1 + t2 is the double t{other} and rounds to itself.\nt1 + t2 -> t{other};\n"
    if half_q:
        last = f"  /\\ eb -/ B * P {BOUND}"
        qs = " and ".join(f"q{m}" for m in half_q)
        src = sub(
            src,
            last,
            last + f"  # {qs} each round a double plus a zero, so each is at least 2^-1022\n"
            "  # as that double is, and lemma.g gives the same of it. The goals\n"
            "  # check the first part.\n"
            + "".join(f"  /\\ q{m} -/ ({NOTE[m]}) {BOUND}" for m in half_q),
        )
        goal = "  (y - Ex) / P6 in ? /\\ P -/ Phat in ?\n"
        src = sub(src, goal, goal + "".join(f"  /\\ |q{m}| >= 1b-1022\n" for m in half_q))
    if half:
        hints = []
        for m in half:
            left, right = PRODUCT[2 * m - 1][0], PRODUCT[2 * m][0]
            gone = 2 * m - 1 if 2 * m - 1 in dead else 2 * m
            hints.append(f"{left} - {right} -> -{right};\n" if gone == 2 * m - 1 else f"{left} - {right} -> {left};\n")
        mins = " and ".join(MINOR[m] for m in half)
        tail += (
            f"\n# {mins} each round a double less a zero, or a zero less a double,\n"
            "# and so round to themselves.\n" + "".join(hints)
        )
    if {1, 2} <= terms:
        # Gappa finds no REL(P, Phat) here: two of the three parts it would
        # average are zero. So ask for the ratio as an expression, and carry
        # r1 and r2 as amounts against Phat, as make_two_zero.py does.
        src = sub(src, "/\\ P -/ Phat in ?\n", "/\\ (P - Phat) / Phat in ?\n")
        src = sub(
            src,
            "Phat = (|bdxcdy| + |cdxbdy|) * |adz|\n"
            "     + (|cdxady| + |adxcdy|) * |bdz|\n"
            "     + (|adxbdy| + |bdxady|) * |cdz|;\n",
            "Phat1 = (|bdxcdy| + |cdxbdy|) * |adz|;\n"
            "Phat2 = (|cdxady| + |adxcdy|) * |bdz|;\n"
            "Phat3 = (|adxbdy| + |bdxady|) * |cdz|;\n"
            "Phat = Phat1 + Phat2 + Phat3;\n",
        )
        x = "((r1 + r2) + r3)"
        tail += (
            "\n# r1 and r2 are zero, and so are Phat1 and Phat2. Carry each as an\n"
            "# amount against Phat, which Phat3 keeps away from zero.\n"
            f"(P - Phat) / Phat -> ((P - {x}) / {x}) * (1 + ({x} - Phat) / Phat) + ({x} - Phat) / Phat;\n"
            f"({x} - Phat) / Phat -> (r1 - Phat1) / Phat + (r2 - Phat2) / Phat"
            " + ((r3 - Phat3) / Phat3) * (Phat3 / Phat);\n"
            "Phat3 / Phat -> 1 / (1 + (Phat1 + Phat2) / Phat3);\n"
        )
    # A term whose product is zero carries no error: it is zero in y and in
    # Ex alike, and stays in P6 only as the zero it is.
    for k in sorted(dead | {k for m in terms for k in (2 * m - 1, 2 * m)}):
        src = sub(src, line_starting(src, f"W{k} = ") + "\n", "")
        line = line_starting(src, "", f"e{k} = ")
        part = line[line.index(f"e{k} = "):]
        src = drop_def(src, part[: part.index(";") + 1])
        src = sub(src, f"(c{k} + e{k}) - c{k} -> e{k};\n", "")
        src = sub(src, line_starting(src, f"e{k} / c{k} -> ") + "\n", "")
        src = sub(src, f"(c{k} + e{k})", f"c{k}")
    return src + tail


def q_goals_alone(src):
    """The script with its q hypotheses out and its |q| goals alone left."""
    lines = src.split("\n")
    lines = [l for l in lines if not l.startswith("  /\\ q")]
    start = lines.index("->")
    end = lines.index("}", start)
    goals = [l for l in lines[start + 1 : end] if l.startswith("  /\\ |q")]
    goals[0] = "  " + goals[0][len("  /\\ ") :]
    return "\n".join(lines[: start + 1] + goals + lines[end:])


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def drop_conj(src, item):
    """Take one conjunct out of a hypothesis line, and the line with it when
    nothing is left on it."""
    if src.count("\n  /\\ " + item + "\n") == 1:
        return src.replace("\n  /\\ " + item + "\n", "\n")
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


def line_starting(src, start, containing=""):
    (line,) = [l for l in src.split("\n") if l.startswith(start) and containing in l]
    return line


main()
```

`run_all.sh` runs every script the generator wrote, beside `block1.g` and
464.13's `one.sh`. It runs after `python3 make_differences_zero.py >
names.txt`, which writes the scripts and keeps the names `summarize.py` reads:

```sh
#!/bin/bash
# Run every script make_differences_zero.py wrote, sixteen at a time: each
# dz_ script with its proof checked by coqc, then each control and each
# dzq_ check without. One line per run, as one.sh prints it.
cd "$(dirname "$0")"
rm -f run_done.txt
ls dz_*.g | xargs -P 16 -I{} ./one.sh {} coq > run_dz.txt 2>&1
ls c_dz_*.g dzq_*.g | xargs -P 16 -I{} ./one.sh {} > run_c.txt 2>&1
echo finished > run_done.txt
```

`stats.py` prints the counts of exit codes above, and the ranges of times,
proof lines and hints:

```python
"""Count and range what run_dz.txt and run_c.txt printed."""
import collections

gappa = []
coq = []
for line in open("run_dz.txt"):
    w = line.split()
    if w[1] == "exit":
        gappa.append((w[0], int(w[2]), float(w[3].rstrip("s"))))
    elif w[1] == "coq":
        coq.append((w[0], int(w[4]), int(w[7]), float(w[8].rstrip("s")), int(w[10]), int(w[12])))
print("dz runs", len(gappa), "exit codes", collections.Counter(e for _, e, _ in gappa))
print("dz seconds", min(s for *_, s in gappa), "to", max(s for *_, s in gappa))
print("coq runs", len(coq), "-Bcoq exits", collections.Counter(c[1] for c in coq),
      "coqc exits", collections.Counter(c[2] for c in coq))
print("coq seconds", min(c[3] for c in coq), "to", max(c[3] for c in coq))
print("proof lines", min(c[4] for c in coq), "to", max(c[4] for c in coq))
print("hints", min(c[5] for c in coq), "to", max(c[5] for c in coq))
ctl = collections.Counter()
for line in open("run_c.txt"):
    w = line.split()
    kind = "dzq" if w[0].startswith("dzq_") else w[0].rsplit("_", 1)[1]
    ctl[(kind, w[2])] += 1
for key in sorted(ctl):
    print("control", key, ctl[key])
```

`summarize.py` prints the groups above, each with the count of roundings it
works out from the set:

```python
"""Read every run of the 237 branches and group them by what they printed.

For each branch: the exit codes of its script, its -Bcoq and its coqc, the
bounds Gappa printed for (y - Ex) / P6 and for P against Phat, the exit code
and best of each control, the exit code of its dzq_ check if it has one, and
the number of roundings on its longest path, counted from the set alone.
"""
import collections
import re

DIFFERENCES = ["adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz"]
PRODUCT = {
    1: ("bdx", "cdy", 1),
    2: ("cdx", "bdy", 1),
    3: ("cdx", "ady", 2),
    4: ("adx", "cdy", 2),
    5: ("adx", "bdy", 3),
    6: ("bdx", "ady", 3),
}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}


def longest_path(zero):
    dead = {k for k, (f, g, _) in PRODUCT.items() if f in zero or g in zero}
    terms = {m for m in (1, 2, 3) if Z_OF[m] in zero or {2 * m - 1, 2 * m} <= dead}
    longest = 0
    for k, (f, g, m) in PRODUCT.items():
        if k in dead or m in terms:
            continue
        other = 2 * m if k == 2 * m - 1 else 2 * m - 1
        n = 3 + 1 + (other not in dead) + 1
        if m in (1, 2) and (3 - m) not in terms:
            n += 1
        longest = max(longest, n)
    return longest


def exits(path):
    out = {}
    coq = {}
    for line in open(path):
        w = line.split()
        if w[1] == "exit":
            out[w[0]] = int(w[2])
        elif w[1] == "coq":
            coq[w[0]] = (int(w[4]), int(w[7]))
    return out, coq


def upper(line):
    found = re.findall(r"\{(-?[\d.]+(?:e[-+]?\d+)?),", line)
    if found:
        return found[-1]
    return line.rsplit(",", 1)[1].strip(" ]")


def bound(text, start):
    for line in text.split("\n"):
        if line.strip().startswith(start):
            return upper(line)
    return "-"


def best(text):
    m = re.search(r"best: \[([^\]]*)\]", text)
    return m.group(1).split(",")[1].strip() if m else "-"


def main():
    names = [l.strip() for l in open("names.txt") if l.strip()]
    dz, coq = exits("run_dz.txt")
    c, _ = exits("run_c.txt")
    groups = collections.defaultdict(list)
    for name in names:
        zero = set(name[3:].split("_"))
        text = open(f"{name}.out").read()
        y = bound(text, "(y - Ex) / P6 in")
        p = bound(text, "P -/ Phat in")
        if p == "-":
            p = bound(text, "(P - Phat) / Phat in") + " as (P - Phat) / Phat"
        c6 = f"c_{name}_6u"
        c5 = f"c_{name}_5u"
        q = "dzq_" + name[3:]
        row = (
            longest_path(zero),
            dz.get(name, "missing"),
            coq.get(name, "none"),
            y,
            p,
            c.get(c6, "missing"),
            best(open(f"{c6}.out").read()),
            c.get(c5, "missing"),
            best(open(f"{c5}.out").read()),
            c.get(q, "no dzq"),
        )
        groups[row].append(name)
    print("roundings | exit | (-Bcoq, coqc) | (y-Ex)/P6 | P vs Phat | 6u exit, best | 5u exit, best | dzq exit | count")
    for row, members in sorted(groups.items(), key=lambda kv: tuple(str(x) for x in kv[0])):
        k, e, cq, y, p, e6, b6, e5, b5, eq = row
        print(f"{k} | {e} | {cq} | {y} | {p} | {e6}, {b6} | {e5}, {b5} | {eq} | {len(members)}")
        print("    " + " ".join(m[3:] for m in members))
    print("total", sum(len(m) for m in groups.values()))


main()
```

**What this leaves.**

- One or two of `first`, `second` and `third` cancelling to zero, their two
  products the same double as in 464.7, together with one or more of the
  differences at zero, past the one minor with one difference that 464.26
  runs. A row after Q2748.
- `t1 + t2` cancelling to zero while neither is zero, with one or more of the
  differences at zero. Block 1 assumes it away with `|t1 + t2| ≥ 2⁻¹⁰⁷⁴`.
  464.4 runs it with no difference at zero, and it proves at `6u + 13u²`;
  464.11 and this section keep the hypothesis wherever `t1` and `t2` are both
  alive. Where `t3` is zero too, `y` is zero (464.7), so nothing is left to
  run there. A row.
- The row that moves the scripts into `proofs/gappa/`, Q2700, now waits on
  those two as well, since they append here too.

### 464.26 One difference at zero with one minor at zero

Q2748 ran block 1 with one of the nine differences `adx` to `cdz` at zero and
one of `first`, `second` and `third` cancelling to zero at once, in two lanes
on 2026-10-26. Of the 27 pairs, 12 cannot happen, and Gappa proves that for
each; 3 are 464.11's branches; 12 are branches that neither 464.7 nor 464.11
runs. Each of the 15 branches proves at the code's `B` and fails below the
bound it proves, and Coq checks each proof, modulo its hints. Nothing in
`orient3d.rs` needs to change for it.

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, through
464.13's `one.sh`, with `block1.g` 464.5's first `gappa` block, the one headed
`# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,`, against `on_floats` as
464.4 read it at `d9f931e8`: no commit touches `orient3d.rs` between 464.11's
`ca9b6493` and `f60bb262`.

**Which pairs are branches.** A minor cancels to zero when its two products
round to the same double, as in 464.7.

- **12 cannot happen.** An x or y difference is a factor of one product in
  each of two minors: `adx` of `adxcdy` in `second` and of `adxbdy` in
  `third`. With it zero, such a minor is a double less a zero, or a zero less
  a double, and rounds to its other product, which `product_is_normal` has
  checked is at least `2⁻¹⁰²²`. So the minor is not zero. The 12 are `adx`
  and `ady` with `second` or `third`, `bdx` and `bdy` with `first` or
  `third`, and `cdx` and `cdy` with `first` or `second`. For each,
  `dmi_<pair>.g` is `dz_<difference>.g` with the minor's `>= 1b-1074`
  hypothesis out and `|minor| >= 1b-1022` its only goal, and it proves. Its
  control `c_dmi_<pair>.g` takes the other product's `>= 1b-1022` hypothesis
  out too, and fails, for each of the 12, with best `[4.94066e-324,
  2.22507e-308]`: without `product_is_normal`, Gappa keeps the minor only
  off zero.
- **3 are 464.11's branches**: `adz` with `first`, `bdz` with `second` and
  `cdz` with `third`. The term is zero by its shape, in `y` and in `Ex` alike,
  and 464.11's script asks nothing of its minor. Run anyway with the minor's
  two products one double, each proves with 464.11's bounds.
- **12 are new**: `adx` and `ady` with `first`, `bdx` and `bdy` with
  `second`, `cdx` and `cdy` with `third`, and each z difference with each of
  the two minors whose term it does not zero.

**The scripts.** `make_differences_zero.py` (464.25), given the nine
differences one at a time, writes `dz_<difference>.g` for each. Each is
464.11's `zd_<difference>.g` but for its comments: with the comment lines
dropped, `diff` prints nothing for any of the nine. `make_minor_zero_too.py`,
below, writes each pair from its `dz_<difference>.g` the way 464.7 writes a
minor at zero from block 1. The minor's second product is the same double as
its first; the minor and its term are left unrounded, with their two ratios
set to 1; and the hypotheses that the term is normal, and that the
permanent's product `r` for it is, go, since `product_is_normal` passes on the
zero term and checks nothing of `r`. Where the zero term is `t1` or `t2` and
the other of the two is not zero, `t1 + t2` is that other term, a double, and
the script carries 464.11's hint `t1 + t2 -> t2;` or `-> t1;`. Where the
difference already zeroes the minor's term, only the products and the minor
change.

`adz` with `second` and `bdz` with `first` zero both `t1` and `t2`, one by its
shape and one by cancelling. Each script without what `both_first_terms_zero`
adds, so with `P`, `Phat` and the goal as block 1 writes them and the hints
`dz_<difference>.g` has, fails at `REL(P, Phat)`. The goal's best is then no
bound: each of its two ends is more than `1e+400` from zero, and the exact
figures change from one run of the same script to the next under 464.4's pin,
so none is quoted here. Gappa finds no bound on `P −/ Phat` at all. `both_first_terms_zero` does what 464.7 does
for `third` at zero. It asks for `(P − Phat) / Phat`, splits `Phat` into
`Phat1`, `Phat2` and `Phat3`, and adds six hints, which carry the cancelled
term's `r` as an amount against `Phat`, since it may be below the normals, and
keep `Phat` away from zero through `Phat3`. It keeps `P` rounded as the code
rounds it, `P12 = rnd(r1 + r2)` and then `rnd(P12 + r3)`, and no hint divides
by the zeroed term's own part of `Phat`, which is zero.

**What runs, at the code's `B`.** Each of the 15 proves, with `(y − Ex) / P6`
within:

- `5.55112e-16`, `5u` and some `u²`, for 4: `adx` and `ady` with `first`,
  and `bdx` and `bdy` with `second`. At `6u` each proves, and at `5u` each
  fails with best `±1`.
- `6.66134e-16`, `6u` and some `u²`, for 10: the other 8 new ones, and
  464.11's `adz` with `first` and `bdz` with `second`. At `6u` each fails with
  best `±1`, and at `5u` with `±1.2`.
- `7.77156e-16`, `7u` and some `u²`, for 464.11's `cdz` with `third`. At `6u`
  it fails with best `±1.16667`, and at `5u` with `±1.4`.

In each of the 15 the count of `u` is the count of roundings on the longest
path, counted as 464.25 counts it, with one path more: the cancelled term,
which `y` holds as zero and `Ex` does not. What is left of it is its two
products' rounding errors, so its path is its two differences and its
product, three, and its z difference's rounding only scales that error. A
cancelled `t1` or `t2` adds no rounding to the sum of the two.
`summarize_pairs.py`, below, works the count out from the pair alone, and its
groups split the 15 the same way the printed bounds do.

`P −/ Phat` is within `4.44089e-16` in 464.11's three, as 464.11 found, and
within `6.66134e-16` in the 10 new ones that ask for it; `(P − Phat) / Phat`
is within `6.66134e-16` in the two with `t1` and `t2` both zero. Why the new
ones' is wider was not traced.

`third` with `cdx`, `cdy`, `adz` or `bdz` proves with the hints its
`dz_<difference>.g` has and none added, where 464.7's `third` alone needed six
more. Why was not traced either.

Block 1's `|t1 + t2| ≥ 2⁻¹⁰⁷⁴` costs nothing here. Where `first` or `second`
cancels, `t1 + t2` is the other of the two: normal by hypothesis, or zero by
its shape, and then the script drops the hypothesis. Where `third` cancels,
`t1 + t2` at zero makes `y` zero, and `decide` certifies no sign (464.7).

Every one of the 15 fails at `5u`, and the 11 whose bound is above `6u` fail
there too, so each fails below the bound it proves, and none proves its goal
from hypotheses that contradict each other (464.7). A `dmi_` script's
hypotheses are its `dz_<difference>.g`'s less one, and each of the nine
`dz_<difference>.g` fails at `5u`, at `BND((y − Ex) / eb)`, so those do not
contradict each other either. The 6 `dmq_` scripts, each a `dm_` script with
its `q` hypotheses taken out and only its `|q|` goals left, each prove: no goal
leans on the hypothesis it backs.

`gappa -Bcoq` writes each of the 15 proofs, 8429 to 9653 lines with 31 to 35
hints, and each of the 12 `dmi_` proofs, 1434 to 1588 lines with 34, and
`coqc` under 464.5's commands accepts each, exit 0. At sixteen at once, Gappa
takes 4.56 to 8.35 s on a `dm_` script and 3.36 to 6.49 s on a `dmi_` one, and
`-Bcoq` and `coqc` together 10.96 to 15.02 s and 4.54 to 7.43 s. `field` has
not been run on these hints (464.10).

`make_minor_zero_too.py`, run beside the `dz_<difference>.g` scripts as
`run_pairs.sh` below runs it:

```python
"""Write block 1 with one difference zero and one minor cancelling to zero.

Run beside the dz_<difference>.g that make_differences_zero.py writes for one
difference. Each argument is a pair such as adx_first. Where a product of the
minor has the difference as a factor, the minor is that product's partner and
cannot be zero, and the script written is dmi_<pair>.g, which asks only that
the minor is at least 2^-1022. Otherwise it is dm_<pair>.g, with the controls
c_dm_<pair>_6u.g and c_dm_<pair>_5u.g, and, where a minor rounds a double less
a zero, dmq_<pair>.g: the script with its q hypotheses taken out and only its
|q| goals left.
"""
import sys

PRODUCT = {
    1: ("bdxcdy", "bdx", "cdy"),
    2: ("cdxbdy", "cdx", "bdy"),
    3: ("cdxady", "cdx", "ady"),
    4: ("adxcdy", "adx", "cdy"),
    5: ("adxbdy", "adx", "bdy"),
    6: ("bdxady", "bdx", "ady"),
}
MINOR = {"first": 1, "second": 2, "third": 3}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"


def main():
    for pair in sys.argv[1:]:
        difference, minor = pair.split("_")
        m = MINOR[minor]
        src = open(f"dz_{difference}.g").read()
        if difference in PRODUCT[2 * m - 1][1:] + PRODUCT[2 * m][1:]:
            write(f"dmi_{pair}.g", cannot_cancel(src, difference, minor, m))
            write(f"c_dmi_{pair}.g", cannot_cancel(src, difference, minor, m, control=True))
        else:
            src = cancel(src, difference, minor, m)
            write(f"dm_{pair}.g", src)
            write(f"c_dm_{pair}_6u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 6b-53;"))
            write(f"c_dm_{pair}_5u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 5b-53;"))
            if "  /\\ q" in src:
                write(f"dmq_{pair}.g", q_goals_alone(src))
        print(pair)


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def cannot_cancel(src, difference, minor, m, control=False):
    """dz_<difference>.g asking only that the minor is at least 2^-1022.

    One product of the minor is zero, so the minor rounds the other one, a
    double that product_is_normal has checked; the hint that says so is in
    dz_<difference>.g already. The control drops that check, and must fail.
    """
    src = drop_conj(src, f"|{minor}| >= 1b-1074")
    lines = src.split("\n")
    start = lines.index("->")
    end = lines.index("}", start)
    lines = lines[: start + 1] + [f"  |{minor}| >= 1b-1022"] + lines[end:]
    src = "\n".join(lines)
    if control:
        (alive,) = [PRODUCT[k][0] for k in (2 * m - 1, 2 * m) if difference not in PRODUCT[k][1:]]
        src = drop_conj(src, f"|{alive}| >= 1b-1022")
    return src


def cancel(src, difference, minor, m):
    """dz_<difference>.g with the minor's two products the same double, as
    464.7 writes a minor at zero."""
    left, f, g = PRODUCT[2 * m]
    src = sub(src, f"{left} = rnd({f} * {g});", f"{left} = {PRODUCT[2 * m - 1][0]};")
    src = sub(src, f"{minor} = rnd(M{m});", f"{minor} = M{m};")
    src = sub(
        src,
        f"# Block 1 where {difference} are zero. A zero rounds to itself,\n"
        "# and so does a product with a zero factor.\n",
        f"# Block 1 where {difference} is zero and {minor} cancels to zero: its two\n"
        "# products are one double. A zero rounds to itself, and so does a\n"
        "# product with a zero factor.\n",
    )
    src = sub(
        src,
        f"  # No factor of the nine products is zero but {difference}. For an\n"
        "  # f64 that is the same as being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {difference} and {minor}. For\n"
        "  # an f64 that is the same as being at least the smallest subnormal.\n",
    )
    if Z_OF[m] == difference:
        # The term is zero by its shape already, in y and in Ex alike, and
        # dz_<difference>.g carries it so.
        return src
    src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
    src = sub(src, f"rm{m} = {minor} / M{m};", f"rm{m} = 1;")
    src = sub(src, f"rt{m} = t{m} / U{m};", f"rt{m} = 1;")
    src = drop_conj(src, f"|{minor}| >= 1b-1074")
    src = drop_conj(src, f"|t{m}| >= 1b-1022")
    src = drop_conj(src, f"|r{m}| >= 1b-1022")
    start = src.index("  # Each of r")
    end = src.index("\n", src.index("\n", start) + 1) + 1
    zeroed = [k for k, z in Z_OF.items() if z == difference]
    said = f"r{zeroed[0]} is zero, and nothing" if zeroed else "Nothing"
    src = sub(
        src,
        src[start:end],
        "  # Each r asked to be normal here is, because its t is and its minor is\n"
        f"  # at most its q. {said} checks r{m}, since {minor} is zero.\n",
    )
    src = sub(src, f"  /\\ t{m} -/ U{m} {BOUND}", "")
    src = sub(src, f"  /\\ r{m} -/ R{m} {BOUND}", "")
    src = sub(src, f"{minor} / M{m} -> 1 + ({minor} - M{m}) / M{m};\n", "")
    src = sub(src, f"t{m} / U{m} -> 1 + (t{m} - U{m}) / U{m};\n", "")
    if m == 3:
        return src
    other = 3 - m
    if Z_OF[other] == difference:
        # t1 and t2 are both zero, so s12 is too and nothing rounds.
        src = sub(src, "s12 = rnd(S);", "s12 = S;")
        src = sub(src, "rs = s12 / S;\n", "")
        src = src.replace(" * rs - 1", " - 1")
        src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
        src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
        src = sub(src, f"\n# t{other} is zero, so t1 + t2 is the double t{m} and rounds to itself.\nt1 + t2 -> t{m};\n", "")
        return both_first_terms_zero(src, m, other)
    return src + f"\n# t{m} is zero, so t1 + t2 is the double t{other} and rounds to itself.\nt1 + t2 -> t{other};\n"


def both_first_terms_zero(src, m, other):
    """t{m} cancels and t{other} is zero by its shape, as in dz_adz_bdz.

    Gappa then finds no REL(P, Phat): two of the three parts it would average
    may be off the normals. So ask for the ratio as an expression, and carry
    r1 and r2 as amounts against Phat, which Phat3 keeps away from zero, as
    make_two_zero.py does. r{other} is exactly zero, and r{m} may be anything
    from zero up, so the sum P12 rounds is r{m} and is kept as the code rounds
    it. Phat{other} is zero, so no hint divides by it.
    """
    note = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|"}[m]
    src = sub(src, "/\\ P -/ Phat in ?\n", "/\\ (P - Phat) / Phat in ?\n")
    src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P12 = rnd(r1 + r2);\nP = rnd(P12 + r3);")
    src = sub(
        src,
        "Phat = (|bdxcdy| + |cdxbdy|) * |adz|\n"
        "     + (|cdxady| + |adxcdy|) * |bdz|\n"
        "     + (|adxbdy| + |bdxady|) * |cdz|;\n",
        "Phat1 = (|bdxcdy| + |cdxbdy|) * |adz|;\n"
        "Phat2 = (|cdxady| + |adxcdy|) * |bdz|;\n"
        "Phat3 = (|adxbdy| + |bdxady|) * |cdz|;\n"
        "Phat = Phat1 + Phat2 + Phat3;\n",
    )
    x = "(P12 + r3)"
    return src + (
        f"\n# r{m} may be below the normals and r{other} is zero, so their errors are\n"
        "# amounts and not fractions. Carry each as an amount against Phat, which\n"
        "# Phat3 keeps away from zero.\n"
        f"(P - Phat) / Phat -> ((P - {x}) / {x}) * (1 + ({x} - Phat) / Phat) + ({x} - Phat) / Phat;\n"
        f"({x} - Phat) / Phat -> (P12 - (r1 + r2)) / Phat + (r{other} - Phat{other}) / Phat"
        f" + (r{m} - R{m}) / Phat + ((q{m} - ({note})) / ({note})) * (Phat{m} / Phat)"
        " + ((r3 - Phat3) / Phat3) * (Phat3 / Phat);\n"
        f"Phat{m} / Phat -> 1 / (1 + (Phat{other} + Phat3) / Phat{m});\n"
        "Phat3 / Phat -> 1 / (1 + (Phat1 + Phat2) / Phat3);\n"
        "Phat3 -> r3 / (1 + (r3 - Phat3) / Phat3);\n"
        f"{x} -> P / (1 + (P - {x}) / {x});\n"
    )


def q_goals_alone(src):
    """The script with its q hypotheses out and its |q| goals alone left."""
    lines = src.split("\n")
    lines = [l for l in lines if not l.startswith("  /\\ q")]
    start = lines.index("->")
    end = lines.index("}", start)
    goals = [l for l in lines[start + 1 : end] if l.startswith("  /\\ |q")]
    goals[0] = "  " + goals[0][len("  /\\ ") :]
    return "\n".join(lines[: start + 1] + goals + lines[end:])


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def drop_conj(src, item):
    """Take one conjunct out of a hypothesis line, and the line with it when
    nothing is left on it."""
    if src.count("\n  /\\ " + item + "\n") == 1:
        return src.replace("\n  /\\ " + item + "\n", "\n")
    if src.count(item + " /\\ ") == 1:
        return src.replace(item + " /\\ ", "")
    if src.count(" /\\ " + item + "\n") == 1:
        return src.replace(" /\\ " + item + "\n", "\n")
    return sub(src, "  /\\ " + item + "\n", "")


main()
```

`run_pairs.sh` writes every script and runs it, beside `block1.g`, 464.11's
`make_one_difference_zero.py`, 464.25's `make_differences_zero.py`,
`make_minor_zero_too.py` and 464.13's `one.sh`:

```sh
#!/bin/bash
# Write every script for one difference at zero, alone and with each minor
# cancelling, and run them all, sixteen at a time: each dm_ and dmi_ script
# with its proof checked by coqc, then each control, each dmq_ check and each
# dz_ script's own 5u control without. One line per run, as one.sh prints it.
cd "$(dirname "$0")"
set -e
rm -f zd_*.g dz_*.g c_dz_*.g dzq_*.g dm_*.g c_dm_*.g dmq_*.g dmi_*.g c_dmi_*.g
rm -f *.out *.v *.bcoq *.coqc *.vo *.vok *.vos *.glob .*.aux run_dm.txt run_c.txt
python3 make_one_difference_zero.py adx ady adz bdx bdy bdz cdx cdy cdz
python3 make_differences_zero.py adx ady adz bdx bdy bdz cdx cdy cdz > /dev/null
python3 make_minor_zero_too.py \
  adx_first adx_second adx_third ady_first ady_second ady_third \
  adz_first adz_second adz_third bdx_first bdx_second bdx_third \
  bdy_first bdy_second bdy_third bdz_first bdz_second bdz_third \
  cdx_first cdx_second cdx_third cdy_first cdy_second cdy_third \
  cdz_first cdz_second cdz_third > names.txt
set +e
for d in adx ady adz bdx bdy bdz cdx cdy cdz; do
  n=$(diff <(grep -v '^ *#' "zd_$d.g") <(grep -v '^ *#' "dz_$d.g") | wc -l)
  echo "dz_$d against zd_$d, lines differing outside comments: $n"
done
ls dm_*.g dmi_*.g | xargs -P 16 -I{} ./one.sh {} coq > run_dm.txt 2>&1
ls c_dm_*.g c_dmi_*.g dmq_*.g c_dz_*_5u.g | xargs -P 16 -I{} ./one.sh {} > run_c.txt 2>&1
echo "not trivially equal, by file:"
grep -c 'not trivially' *.out | grep -v ':0$'
echo finished
```

`summarize_pairs.py` prints the groups above, each with the count of
roundings it works out from the pair:

```python
"""Read every run of the 27 pairs and group them by what they printed.

For each dm_ branch: the count of roundings on its longest path, worked out
from the pair alone; the exit codes of its script, -Bcoq and coqc; the bounds
Gappa printed for (y - Ex) / P6 and for P against Phat; each control's exit
and best; its dmq_ exit. For each dmi_: its exits, and its control's exit,
best and the goal it failed at.
"""
import collections
import re

PRODUCT = {
    1: ("bdx", "cdy", 1),
    2: ("cdx", "bdy", 1),
    3: ("cdx", "ady", 2),
    4: ("adx", "cdy", 2),
    5: ("adx", "bdy", 3),
    6: ("bdx", "ady", 3),
}
MINOR = {"first": 1, "second": 2, "third": 3}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}


def longest_path(difference, minor):
    """Roundings on the longest path through a term of y, or through the
    cancelled minor's term of Ex where y has a zero."""
    c = MINOR[minor]
    dead = {k for k, (f, g, _) in PRODUCT.items() if difference in (f, g)}
    shape = {m for m in (1, 2, 3) if Z_OF[m] == difference or {2 * m - 1, 2 * m} <= dead}
    zero = shape | {c}
    longest = 0
    for k, (f, g, m) in PRODUCT.items():
        if k in dead or m in shape:
            continue
        if m == c:
            # y holds zero for this term, Ex its exact value: the two
            # differences and the product's rounding are all that is off.
            n = 3
        else:
            other = 2 * m if k == 2 * m - 1 else 2 * m - 1
            n = 3 + 1 + (other not in dead) + 1
            if m in (1, 2) and (3 - m) not in zero:
                n += 1
        longest = max(longest, n)
    return longest


def exits(path):
    out = {}
    coq = {}
    for line in open(path):
        w = line.split()
        if w[1] == "exit":
            out[w[0]] = (int(w[2]), float(w[3].rstrip("s")))
        elif w[1] == "coq":
            coq[w[0]] = (int(w[4]), int(w[7]), float(w[8].rstrip("s")), int(w[10]), int(w[12]))
    return out, coq


def upper(line):
    found = re.findall(r"\{(-?[\d.]+(?:e[-+]?\d+)?),", line)
    if found:
        return found[-1]
    return line.rsplit(",", 1)[1].strip(" ]")


def bound(text, start):
    for line in text.split("\n"):
        if line.strip().startswith(start):
            return upper(line)
    return "-"


def failed(text):
    m = re.search(r"\n  (\S.*), best: \[([^\]]*)\]", text)
    return (m.group(1), m.group(2)) if m else ("-", "-")


def main():
    pairs = [l.strip() for l in open("names.txt") if l.strip()]
    run, coq = exits("run_dm.txt")
    c, _ = exits("run_c.txt")
    groups = collections.defaultdict(list)
    impossible = collections.defaultdict(list)
    for pair in pairs:
        difference, minor = pair.split("_")
        if f"dmi_{pair}" in run:
            ctl = f"c_dmi_{pair}"
            row = (
                run[f"dmi_{pair}"][0],
                coq.get(f"dmi_{pair}", "none")[:2],
                c[ctl][0],
                failed(open(f"{ctl}.out").read()),
            )
            impossible[row].append(pair)
            continue
        name = f"dm_{pair}"
        text = open(f"{name}.out").read()
        y = bound(text, "(y - Ex) / P6 in")
        p = bound(text, "P -/ Phat in")
        if p == "-":
            p = bound(text, "(P - Phat) / Phat in") + " as (P - Phat) / Phat"
        c6, c5 = f"c_{name}_6u", f"c_{name}_5u"
        row = (
            longest_path(difference, minor),
            run[name][0],
            coq.get(name, "none")[:2],
            y,
            p,
            c[c6][0],
            failed(open(f"{c6}.out").read()),
            c[c5][0],
            failed(open(f"{c5}.out").read()),
            c.get(f"dmq_{pair}", ("no dmq",))[0],
        )
        groups[row].append(pair)
    print("roundings | exit | (-Bcoq, coqc) | (y-Ex)/P6 | P vs Phat | 6u exit, failed | 5u exit, failed | dmq exit | count")
    for row, members in sorted(groups.items(), key=lambda kv: tuple(str(x) for x in kv[0])):
        print(" | ".join(str(x) for x in row), "|", len(members))
        print("    " + " ".join(members))
    print("dm total", sum(len(m) for m in groups.values()))
    print("dmi exit | (-Bcoq, coqc) | control exit | control failed at | count")
    for row, members in sorted(impossible.items(), key=lambda kv: str(kv[0])):
        print(" | ".join(str(x) for x in row), "|", len(members))
        print("    " + " ".join(members))
    print("dmi total", sum(len(m) for m in impossible.values()))
    dm = [v for k, v in run.items() if k.startswith("dm_")]
    di = [v for k, v in run.items() if k.startswith("dmi_")]
    cq = [v for k, v in coq.items() if k.startswith("dm_")]
    ci = [v for k, v in coq.items() if k.startswith("dmi_")]
    print("dm gappa seconds", min(s for _, s in dm), "to", max(s for _, s in dm))
    print("dmi gappa seconds", min(s for _, s in di), "to", max(s for _, s in di))
    for label, rows in (("dm", cq), ("dmi", ci)):
        print(label, "coq seconds", min(r[2] for r in rows), "to", max(r[2] for r in rows),
              "lines", min(r[3] for r in rows), "to", max(r[3] for r in rows),
              "hints", min(r[4] for r in rows), "to", max(r[4] for r in rows))
    kinds = collections.Counter()
    for k, v in c.items():
        if k.startswith("c_dm_"):
            kind = "c_dm " + k.rsplit("_", 1)[1]
        elif k.startswith("c_dz_"):
            kind = "c_dz 5u " + failed(open(f"{k}.out").read())[0]
        else:
            kind = k.split("_")[0]
        kinds[(kind, "exit", v[0])] += 1
    for key in sorted(kinds):
        print("control", *key, "count", kinds[key])
    print("control seconds", min(v[1] for v in c.values()), "to", max(v[1] for v in c.values()))


main()
```

**What this leaves.**

- One or two of `first`, `second` and `third` cancelling to zero together with
  one or more of the differences at zero, past the one minor with one
  difference run here. Q2862, filed after Q2748.
- Nothing more for `t1 + t2` cancelling while neither is zero (Q2863, from
  464.25): in these pairs block 1's hypothesis on it costs nothing, as above.
- The row that moves the scripts into `proofs/gappa/`, Q2700, takes
  `make_minor_zero_too.py` and what it writes too. It already waits on Q2748.
- Why `P −/ Phat` is wider in the new ones, and why `third` with `cdx`,
  `cdy`, `adz` or `bdz` needs none of the six hints 464.7 added for `third`.
  No row: each branch proves at the code's `B` either way, and nothing reads
  the answer.
- 464.25's `make_differences_zero.py` ends a comment with
  `Each of  is zero.`, naming nothing, in the six `dz_<difference>.g` for an
  x or y difference, and wherever else no term is zero by its shape. It is a
  Gappa comment and moves no proof, and 464.25 quotes the script as it ran,
  so it stays. In the `dm_` scripts written from those six,
  `make_minor_zero_too.py` puts a line such as
  `Nothing checks r1, since first is zero.` in its place.

### 464.27 The side of a flat surface through a point with every product anywhere

Q2760 took up `SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND` again, in
four lanes on 2026-10-26, for what 464.13's 5 left unrun: products of
`SideOfAFlatSurfaceThroughAPoint` below the normals, which `on_floats`'s six
`product_is_normal` checks refuse, as 464.18 ran them for
`SideOfAFlatSurface`'s three. The first lane wrote `flatall.py` and ran nine
of its scripts. The second wrote `prodzero.g` again so that it proves, and ran
every case and control. The third restated `plane.rs` and wrote this up, and
the fourth read the controls.

The bound is proved at the code's constant in every case of where the six
products land, normal, below the normals or rounding to zero, with any of the
sums cancelling to zero. So `on_floats` does not need its six
`product_is_normal` checks to give a right sign. They are kept, as 464.18's 7
kept plane's and Q2766 the sphere's (464.14), and the comment on them and the
doc on the constant now say why (6). Coq checks each proof, modulo its hints
(7).

Every run is Gappa 1.8.0 under 464.4's pin at `-Eprecision=200`, against
`plane.rs` and `cascade.rs` at `eb125ed0`, through 464.13's `one.sh`, sixteen
at a time. `u` is `2⁻⁵³`, and the code's `B = 6u + 128u²` is written `B =
6b-53 + 1b-99;`. Each script follows 464.13's `flat.g`: one ratio hint per
rounding of a normal number, the error split into one term per product so
that Gappa averages them, and 464.5's lemma for each product that rounds to a
normal number and for `eb`. What rounding a product below the normals cost,
and each product that rounds to zero, is carried as an amount, as 464.18's
cases carry it.

**1. What the proof guards.** As in 464.13's 1, at `eb125ed0`:
`which_side_of_a_flat_surface_through` (`plane.rs:972-988`) runs the cascade
on `SideOfAFlatSurfaceThroughAPoint`. Its one caller is
`HalfSpaceThroughAPoint::side_of_point` (`half_space.rs:668-675`), and
`lib.rs:101` exports `HalfSpaceThroughAPoint`, so a user reaches it.

**2. `on_floats`, line for line.** `SideOfAFlatSurfaceThroughAPoint::on_floats`
(`plane.rs:1003-1026` at `eb125ed0`) returns `None` for a point that is not
exact `f64`s, and unless `product_is_normal` holds for each of the six
products (`plane.rs:1007-1012`). Then it takes the facing times the point on
each axis and the facing times the through point on each axis, negated
(`plane.rs:1014-1017`), folds them from `(0.0, 0.0)` into `value` and
`permanent` (`plane.rs:1018-1020`), and returns `error_bound =
SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND * permanent`
(`plane.rs:1024`). Each script writes that as 464.13's `flat.g` does, with the
check left out. `product_is_normal(a, b)` is `a == 0.0 || b == 0.0 || (a *
b).abs() >= f64::MIN_POSITIVE` (`cascade.rs:67-69`): it lets through a product
with a zero factor and one that rounds to `2⁻¹⁰²²` or more, and refuses every
other. `decide` compares only against a bound that is a normal number
(`cascade.rs:125`), and the doc on `Filtered::error_bound` asks for the bound
on `y`, the sum before `value`'s last rounding. So every claim here is on `y`.
464.13's 4 proved `value` within `eb` too with every product normal or zero;
that was not run with a product below the normals.

**3. Every place the six products can land.** 464.18's `planeall.py` split
each product four ways, normal, zero, and below the normals of either sign.
Carried over to six products that is `4⁶ = 4096` placements before the sums
are split. `flatall.py` (8) cuts the same inputs into 722 cases and one more,
by four facts:

- Each product of two doubles, `X = a · b` rounded to `t = rnd(X)`, is one of
  three. `z`: it rounds to zero, and then `|X| ≤ 2⁻¹⁰⁷⁵` (`prodzero.g`, 8).
  `n`: it rounds to `2⁻¹⁰²²` or more, which `product_is_normal` lets through.
  `s`: it rounds to a double that is neither zero nor normal, and then `|X| ≤
  2⁻¹⁰²²`, since rounding is monotone and `2⁻¹⁰²²` is a double. Those three
  cover every product. A product with a zero factor is a `z`, which the check
  lets through; so is a product below the normals that rounds to zero, which
  the check refuses; every `s` the check refuses.
- A through product enters the fold negated, and `−rnd(f · q)` is `rnd(f ·
  (−q))`, rounding to nearest being symmetric. So each script writes the six
  alike, `t = rnd(a · b)`, each with two factors of its own. That drops the
  facing being shared between two products, which covers more inputs than the
  code can make, not fewer.
- A `z` term is left out of both folds, adding zero to a double giving the
  double back (464.13's `addzero.g`), and stays only in `Ex`, as its exact
  product `Z`. So where the `z` terms sit does not matter, only how many there
  are. The terms left are folded in the order the code adds them.
- Each rounded sum of the terms left is either at least `2⁻¹⁰⁷⁴`, a sum of
  two doubles being a multiple of `2⁻¹⁰⁷⁴`, or it cancels to zero, rounds to
  itself and is written unrounded, as 464.13's 6 writes it. An `n` or `s` term
  is not zero, so the sum after a cancelled one is not zero, and two
  cancelling sums are never next to each other. In 464.18 they could be, since
  a product below the normals could round to zero there; here that is a `z`,
  and left out.

With `L` terms left there are `2^L − 1` strings of `n` and `s` with an `n`
among them, and as many sets of cancelling sums as there are sets of the sums
`1` to `L − 2` with no two next to each other: 1, 1, 2, 3, 5 and 8 for `L`
from 1 to 6. That makes `1·1 + 3·1 + 7·2 + 15·3 + 31·5 + 63·8 = 722` cases,
each `fa_<states>[_c<set>].g`: `<states>` one letter for each term left, in
the fold's order, and `<set>` the place among them of each term whose sum
cancels. `fa_nnnnnn.g` is 464.13's `flat.g` in this form.

- With an `n` term, the step is 464.18's, `(y − Ex) / eb -> ((Σ e) / Phat) ·
  (Phat / P) · (P / eb) + A / eb`. Each term left carries `e = t · W`, `W`
  what the rounded sums after it multiplied it by, less one, and for an `n`
  term less `1 / rp` rather than one, `rp` what its own rounding multiplied it
  by. `Phat` is the permanent with no rounding of its own. What each `s`
  term's own rounding cost, `d = t − X`, less each `Z`, is the amount `A`, so
  `y − Ex` is `Σ e + A`, and `eb` being normal keeps `A` small against `eb`.
  No sign split is needed.
- With none, `fa_none.g` takes every case at once: each of the six products
  is at most `2⁻¹⁰²²`, so `P` is at most about `6 · 2⁻¹⁰²²`, `eb` is far below
  the normals, and `decide` does not compare. Unlike 464.18's plane, no offset
  can make `eb` normal here.

**4. What they print.** `runall.sh` (8) ran the 722 cases, `fa_none.g` and
`prodzero.g` with Coq, from 20:42:34Z to 21:07:17Z on a load of 15 to 33 over
24 cores.

- Each of the 722 and `fa_none.g` proves at the code's `B`, and so does
  `prodzero.g`: `grep -c ' exit 0 [0-9.]*s$' list_fa.log` prints 724. Only
  `fa_none.g` prints "pointless". The slowest is `fa_nnnnnn.g`, 92.72 s.
- `fa_nnnnnn.g` prints 464.13's figures for `flat.g`: `(Σ e) / Phat` within
  `5u + 7u²` and `P −/ Phat` within `5u + 10u²`, each with a little more.
- `summary.py` (8) reads each case. In all 722, `A` is within exactly
  `2⁻¹⁰⁷⁵` for each term that is not `n`, so at most `5 · 2⁻¹⁰⁷⁵`. `(Σ e) /
  Phat` is within `k u + 7u²`, `k` the most roundings a term goes through on
  its way to `y` with every term normal, `L − 1 − c` for `c` sums cancelling
  and at least 1, and `P −/ Phat` within `(L − 1) u + 10u²`, each with a
  little more: `summary.py` prints that little more as `u³` and `10u³` at
  most, to ten digits. The largest are `5u + 7u²`, in 46 cases with
  `fa_nnnnnn` and `fa_nnssss` among them, and `5u + 10u²`, in 384 with
  `fa_snssss_c4` among them (`grep -c -F 'e: 5.000000u = 5u+7.0000u^2'
  summary.txt` and `grep -c -F 'P: 5.000000u = 5u+10.0000u^2' summary.txt`,
  with `summary.txt` what `summary.py` printed). 131 cases print less than
  `k u` (`grep -c 'e: [0-9.]*u = [0-9]u-' summary.txt`), such as `fa_ssssnn`
  at `2.8u` for its `5u`. To six digits, `(Σ e) / Phat` is `u` in 38 cases,
  `1.8u` in 2, `2u` in 112, `2.8u` in 2, `3u` in 288, `4u` in 232 and `5u` in
  48 (`grep -o 'e: [0-9.]*u' summary.txt | sort | uniq -c`).

So the bound holds with any of the products below the normals because a
normal `eb` is `2⁻¹⁰²²` or more, which puts `P` at `2^(-971.585)` or more
(`pmin.g`, 8, with `P` any real at least zero, so every permanent). The
relative part is then at most `5u + 7u²` of `Phat`, and a little more, against
`B = 6u + 128u²`, and `A` at most `5 · 2⁻¹⁰⁷⁵`, which is `5u` of `eb`. That is
464.18's 4 again, with five products not normal where it had at most two
beside a normal one.

**5. The controls.** A script whose hypotheses contradict each other proves
every goal (464.7's 3), so each case is run again where it must fail:

- Each of the 722 cases at `B = u / 2` (`c_<name>_half.g`, `B = 1b-54;`)
  exits 1, and none prints "pointless". Its best is `[−2x, 2x]` for the `x u`
  that `summary.py` prints for its case's `(Σ e) / Phat`, which `controls.py`
  (8) checks for each, to within a thousandth: it prints `_half controls
  whose best is twice their case's u count: 722`, with none under, none over
  and none without a best. `c_fa_nnnnnn_half.g`'s best is `[-10, 10]`. Gappa's
  best for a goal it does not prove can change from one run to the next
  (464.26), so these are this run's figures, each a bound Gappa found on the
  size of `(y − Ex) / eb`, and not an exact value.
- `c_fa_none_noeb.g`, `fa_none.g` without `eb` normal, exits 1, best `[-inf,
  inf]`, printing `eb` in `[0, 9b-1073]` and `P` in `[0, 3b-1021]`. So what
  proves `fa_none.g` is the contradiction with `eb` normal and nothing else,
  as 464.13's `flat_z012345.g`.
- `c_prodzero.g` is `prodzero.g` asking for `2⁻¹⁰⁷⁶`, and exits 1, best
  `[-2.47033e-324, 2.47033e-324]`, `2⁻¹⁰⁷⁵`. That is the bound, met: `2⁻¹⁰⁷⁴
  · 2⁻¹` is a product of two doubles, and rounds to zero, the tie going to
  even.

`runall.sh` (8) ran the 724 controls, the 722 and these two, without Coq,
from 21:07:17Z to 21:27:52Z: `grep -c ' exit 1 ' list_c.log` prints 724, and
`grep -l pointless c_*.out | wc -l` prints 0.

**6. What `plane.rs` said, and says now.** At `eb125ed0` the doc on the
constant ended, `plane.rs:1088-1089`: "Whether a product below the normals,
which `on_floats` refuses, would break it has not been run." The check itself
had no comment. By 4 it would not break it: without the check, `on_floats`
would hand `decide` a number with one to five of the six products below the
normals, and `decide` would certify no wrong sign from it, and with all six
there `decide` does not compare.

So the six checks are not needed for a right sign. They are kept, as 464.18's
7 kept plane's three, for the same reason: with them, level 1 answers only
where the relative argument in the constant's doc holds, and the 722 cases
stay a second proof rather than the only one. Dropping them would move the
inputs with a product below the normals from level 2 to level 1, and whether
that turns any refusal into an answer was not run. The check now carries a
comment saying it is not needed for a right sign and what it does, in the
words of plane's (`plane.rs:863-867`), and the doc on the constant ends with
how far down the bound holds and why, citing this subsection, in place of the
sentence above. Comment only: lines that run 0.

**7. Rocq checks every proof here, modulo its hints.** `gappa -Bcoq` and
`coqc`, under 464.4's pins with gappalib 1.8.0, exit 0 on each of the 722,
`fa_none.g` and `prodzero.g`: `grep -c 'coq -Bcoq exit 0 coqc exit 0'
list_fa.log` prints 724. Gappa does not write the same proof each run (464.4),
so the sizes are the second lane's:

- the 722 cases: 1513 to 6969 lines, 8 to 27 hints; `fa_nnnnnn.g` 5369 lines
  and 27 hints, beside `flat.g`'s 4926 and 27 in 464.13's 8;
- `fa_none.g`: 832 lines, no hints;
- `prodzero.g`: 101 lines, no hints.

The slowest, `fa_nnnnnn.g`, took 104.44 s for the two together. Each hint
enters the Coq file as a hypothesis with no condition attached, as in 464.4,
so each proof stands on its hints, and on the divisors Gappa took to be
nonzero, which its warnings name. `field` has not been run on these hints
(464.10).

**8. The scripts.** `flatall.py` writes the 722 cases, `fa_none.g`, a control
at `u / 2` for each case and `c_fa_none_noeb.g`. `prodzero.g`, `c_prodzero.g`
and `pmin.g` are whole below. `runall.sh` runs every script through
`runmany.sh` and 464.13's `one.sh`, unchanged; `summary.py` and `controls.py`
read what they print. To run them again, save these eight files and `one.sh`
in one directory and run `python3 flatall.py`, `bash runall.sh`, `python3
summary.py` and `python3 controls.py` there.

`flatall.py`, whole, as run:

```python
"""Write SideOfAFlatSurfaceThroughAPoint::on_floats without its six
product_is_normal checks, in every case of where its six products land, as
464.18's planeall.py does for SideOfAFlatSurface's three.

Each of the six terms the fold adds is a rounded product of two doubles: a
through product enters negated, and -rnd(f * q) is rnd(f * -q). A term that
rounds to zero is left out of both folds (464.13's addzero.g), so a case is
the states of the terms left, in the order the fold adds them, and how many
rounded to zero. Each term left is in one of two states:

- n, normal: it rounds to at least 2^-1022, as product_is_normal lets through;
- s, below the normals and rounding to a double that is not zero, which
  product_is_normal refuses;

and each term that rounds to zero is z: a zero factor, which product_is_normal
lets through, or a product below the normals at most 2^-1075 (prodzero.g).
Each rounded sum either rounds, and is then at least 2^-1074, or cancels to
zero and is written unrounded; two cancelling sums are never next to each
other, since the sum after one is a term left alone, which is not zero.

fa_<states>[_c<set>].g for each case with an n term, <states> one letter per
term left, <set> naming each term whose sum cancels by its place among those
left; c_<name>_half.g, each again at B = u / 2, which must fail;
fa_none.g, every case with no n term at once, where eb cannot be normal, and
c_fa_none_noeb.g, that again without eb normal, which must fail.

Run as: python3 flatall.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

B_CODE = "B = 6b-53 + 1b-99;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"
HEAD = [
    "# restates: crates/rcad-geom/src/plane.rs on_floats at eb125ed0",
    "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at eb125ed0",
    "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at eb125ed0",
    "# restates: crates/rcad-predicates/src/cascade.rs decide at eb125ed0",
    "# SideOfAFlatSurfaceThroughAPoint::on_floats without its six product_is_normal",
    "# checks: the sign of facing . point - facing . through, as six rounded",
    "# products of two doubles folded left to right.",
]


def cancellable(left):
    sums = list(range(1, left - 1))
    for n in range(len(sums) + 1):
        for c in itertools.combinations(sums, n):
            if all(b - a > 1 for a, b in zip(c, c[1:])):
                yield c


def fold(src, left, cancel):
    """Write the fold of t0 .. t<left-1> into y; return the sums that round."""
    rounded = []
    running = "t0"
    for k in range(1, left - 1):
        src.append(f"S{k} = {running} + t{k};")
        if k in cancel:
            src.append(f"s{k} = S{k};")
        else:
            src.append(f"s{k} = rnd(S{k});")
            rounded.append(k)
        running = f"s{k}"
    src.append(f"y = {running} + t{left - 1};" if left >= 2 else "y = t0;")
    return rounded


def permanent(left):
    perm = "|t0|"
    for k in range(1, left):
        perm = f"rnd({perm} + |t{k}|)"
    return perm


def hypotheses(src, hyps):
    src += ["", "{"]
    first = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            if row:
                src.append(("     " if first else "  /\\ ") + " /\\ ".join(row))
                first = False
    src.append("->")


def make(states, cancel=(), b=B_CODE):
    left = len(states)
    zeros = 6 - left
    small = [k for k in range(left) if states[k] == "s"]
    normal = [k for k in range(left) if states[k] == "n"]
    src = list(HEAD)
    src.append(f"# The terms left, in the order the fold adds them: {states}"
               + (f"; {zeros} more round{'s' if zeros == 1 else ''} to zero." if zeros else "."))
    if cancel:
        src.append(f"# The sum{'s' if len(cancel) > 1 else ''} adding term{'s' if len(cancel) > 1 else ''} "
                   f"{', '.join(map(str, cancel))} cancel{'' if len(cancel) > 1 else 's'} to zero, which rounds to itself.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The two factors of each term left, each an f64.",
        " ".join(f"a{k} = rnd(a{k}_); b{k} = rnd(b{k}_);" for k in range(left)),
    ]
    if zeros:
        src += ["# The two factors of each term that rounds to zero.",
                " ".join(f"g{j} = rnd(g{j}_); h{j} = rnd(h{j}_);" for j in range(zeros))]
    src += [
        "",
        "# What on_floats computes, line for line. The fold starts at 0.0, and",
        "# adding 0.0 or a term that rounded to zero to a double gives that double",
        "# back (464.13's addzero.g), so those and the first addition are left out.",
        " ".join(f"X{k} = a{k} * b{k};" for k in range(left)),
    ]
    if zeros:
        src.append(" ".join(f"Z{j} = g{j} * h{j};" for j in range(zeros)))
    src.append(" ".join(f"t{k} = rnd(X{k});" for k in range(left)))
    rounded = fold(src, left, cancel)
    src += ["", "# The permanent folds the same terms' magnitudes in the same order.",
            f"P = {permanent(left)};", "", b, "eb = rnd(B * P);", "",
            "# The true value of the same inputs."]
    src.append("Ex = " + " + ".join([f"X{k}" for k in range(left)] + [f"Z{j}" for j in range(zeros)]) + ";")
    src += ["", "# The permanent with no rounding of its own.",
            "Phat = " + " + ".join(f"|t{k}|" for k in range(left)) + ";"]
    src += ["", "# What each rounding multiplied its argument by."]
    if normal:
        src.append(" ".join(f"rp{k} = t{k} / X{k};" for k in normal))
    if rounded:
        src.append(" ".join(f"rs{k} = s{k} / S{k};" for k in rounded))
    src += ["", "# Each term as y carries it less the term: e the part that is a fraction",
            "# of the term as on_floats rounds it, with W that fraction, and for a term",
            "# below the normals d what its own rounding cost, an amount."]
    for k in range(left):
        carry = [f"rs{j}" for j in rounded if j >= k]
        carried = " * ".join(carry) if carry else "1"
        src.append(f"W{k} = {carried} - 1 / rp{k};" if k in normal else f"W{k} = {carried} - 1;")
    src.append(" ".join(f"e{k} = t{k} * W{k};" for k in range(left)))
    src.append(" ".join(f"c{k} = |t{k}|;" for k in range(left)))
    if small:
        src.append(" ".join(f"d{k} = t{k} - X{k};" for k in small))
    amount = " + ".join([f"d{k}" for k in small]) + "".join(f" - Z{j}" for j in range(zeros))
    if not small and zeros:
        amount = "-" + " - ".join(f"Z{j}" for j in range(zeros))
    if amount:
        src.append(f"A = {amount};")

    hyps = [("Every input is a finite f64.",
             [[f"|a{k}| <= 1b1024 /\\ |b{k}| <= 1b1024" for k in range(left)],
              [f"|g{j}| <= 1b1024 /\\ |h{j}| <= 1b1024" for j in range(zeros)]]),
            ("No factor of a normal term is zero.",
             [[f"|a{k}| >= 1b-1074 /\\ |b{k}| >= 1b-1074" for k in normal]])]
    if rounded:
        hyps.append(("No rounded sum cancels to zero.", [[f"|S{k}| >= 1b-1074" for k in rounded]]))
    if small:
        hyps.append(("The terms below the normals that round to a double that is not zero.",
                     [[f"|X{k}| <= 1b-1022 /\\ |t{k}| >= 1b-1074" for k in small]]))
    if zeros:
        hyps.append(("The terms that round to zero (prodzero.g).",
                     [[f"|Z{j}| <= 1b-1075" for j in range(zeros)]]))
    hyps += [("The normal terms, as product_is_normal has them.", [[f"|t{k}| >= 1b-1022" for k in normal]]),
             ("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]),
             ("What lemma.g proves each of those gives.",
              [[f"t{k} -/ X{k} {LEMMA}"] for k in normal] + [[f"eb -/ B * P {LEMMA}"]]),
             ("x / |x| is in [-1, 1] for any x that is not zero.",
              [[f"t{k} / |t{k}| in [-1, 1]" for k in range(left)]])]
    hypotheses(src, hyps)
    esum = " + ".join(f"e{k}" for k in range(left))
    src.append(f"  ({esum}) / Phat in ? /\\ P -/ Phat in ?" + (" /\\ A in ?" if amount else ""))
    src += ["  /\\ (y - Ex) / eb in [-1, 1]", "}", "", "# How far each rounding is from the identity."]
    for k in normal:
        src.append(f"t{k} / X{k} -> 1 + (t{k} - X{k}) / X{k};")
    for k in rounded:
        src.append(f"s{k} / S{k} -> 1 + (s{k} - S{k}) / S{k};")
    src += [
        "",
        "# The relative part is averaged over the permanent as in flat.g; what",
        "# rounding a term below the normals cost, and each term that rounded to",
        "# zero, is an amount, and eb being normal keeps it small against eb.",
        f"(y - Ex) / eb -> (({esum}) / Phat) * (Phat / P) * (P / eb)" + (" + A / eb;" if amount else ";"),
        f"({esum}) / Phat -> ((Phat + ({esum})) - Phat) / Phat;",
        f"Phat + ({esum}) -> " + " + ".join(f"(c{k} + e{k})" for k in range(left)) + ";",
    ]
    for k in range(left):
        src.append(f"(c{k} + e{k}) - c{k} -> e{k};")
    for k in range(left):
        src.append(f"e{k} / c{k} -> (t{k} / |t{k}|) * W{k};")
    src += [
        "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
        "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
    ]
    return "\n".join(src) + "\n"


def make_none(eb_normal=True):
    """Every case with no normal term: each of the six rounds below 2^-1022 or
    to zero, so each product is at most 2^-1022, the permanent at most about
    six times that, and eb far below the normals."""
    src = list(HEAD)
    src += ["# No term is normal: each product is at most 2^-1022, and may be zero.",
            "@rnd = float<ieee_64, ne>;", "",
            " ".join(f"a{k} = rnd(a{k}_); b{k} = rnd(b{k}_);" for k in range(6)), "",
            "# What on_floats computes, every term in the fold.",
            " ".join(f"X{k} = a{k} * b{k};" for k in range(6)),
            " ".join(f"t{k} = rnd(X{k});" for k in range(6))]
    fold(src, 6, ())
    src += ["", f"P = {permanent(6)};", "", B_CODE, "eb = rnd(B * P);", "",
            "Ex = " + " + ".join(f"X{k}" for k in range(6)) + ";"]
    hyps = [("Every input is a finite f64.", [[f"|a{k}| <= 1b1024 /\\ |b{k}| <= 1b1024" for k in range(6)]]),
            ("No term is normal.", [[f"|X{k}| <= 1b-1022" for k in range(6)]])]
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    hypotheses(src, hyps)
    src += ["  eb in ? /\\ P in ?", "  /\\ (y - Ex) / eb in [-1, 1]", "}"]
    return "\n".join(src) + "\n"


def name_of(states, cancel):
    return f"fa_{states}" + (f"_c{''.join(map(str, cancel))}" if cancel else "")


if __name__ == "__main__":
    proving = []
    for left in range(1, 7):
        for states in ("".join(s) for s in itertools.product("ns", repeat=left)):
            if "n" not in states:
                continue
            for cancel in cancellable(left):
                name = name_of(states, cancel)
                open(f"{name}.g", "w").write(make(states, cancel))
                open(f"c_{name}_half.g", "w").write(make(states, cancel, b=B_HALF))
                proving.append(name)
    open("fa_none.g", "w").write(make_none())
    open("c_fa_none_noeb.g", "w").write(make_none(eb_normal=False))
    print(f"{len(proving)} cases with a normal term, each with a control at u / 2, and fa_none.g")
```

`python3 flatall.py` prints `722 cases with a normal term, each with a
control at u / 2, and fa_none.g`.

`prodzero.g` proves the cover's `z`. The first lane wrote it with both
directions as implications inside the goal, and it failed, `BND(X)` best
within `2.22507e-308`: Gappa reads an implication in the goal by negating it,
and the edge at `2⁻¹⁰⁷⁵` is not closed. The cover needs one direction only,
and with `t` zero among the hypotheses it proves with no hint:

```gappa
# prodzero.g: a product of two doubles, of either sign, that rounds to zero
# is at most 2^-1075 in magnitude.
@rnd = float<ieee_64, ne>;
a = rnd(a_); b = rnd(b_);
X = a * b;
t = rnd(X);
{ |a| <= 1b1024 /\ |b| <= 1b1024 /\ t in [0, 0]
  -> X in [-1b-1075, 1b-1075] }
```

`c_prodzero.g`, its control (5):

```gappa
# c_prodzero.g: prodzero.g asking for half its bound, which must fail: 2^-1075
# is a product of two doubles and rounds to zero, ties going to even.
@rnd = float<ieee_64, ne>;
a = rnd(a_); b = rnd(b_);
X = a * b;
t = rnd(X);
{ |a| <= 1b1024 /\ |b| <= 1b1024 /\ t in [0, 0]
  -> X in [-1b-1076, 1b-1076] }
```

`pmin.g`, which `bash one.sh pmin.g nocoq` runs, printing `P in
[1071292029505990860761435525475052308156628044440098539197007b-1171
{3.34028e-293, 2^(-971.585)}, 1b1000 {1.07151e+301, 2^(1000)}]`:

```gappa
# pmin.g: how small the permanent can be with eb = rnd(B * P) normal, at the
# code's B. P is any non-negative real here, which covers every permanent.
@rnd = float<ieee_64, ne>;
B = 6b-53 + 1b-99;
eb = rnd(B * P);
{ P >= 0 /\ P <= 1b1000 /\ eb >= 1b-1022 /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  -> P in ? }
P -> (eb / B) / (1 + (eb - B * P) / (B * P));
```

`runall.sh`:

```sh
#!/bin/bash
# Run every case and prodzero.g with Coq, then every control without, through
# runmany.sh, and print when each list is done.
cd "$(dirname "$0")"
{ ls fa_*.g; echo prodzero.g; } > list_fa
ls c_*.g > list_c
date -u +%FT%TZ
bash runmany.sh list_fa coq
date -u +%FT%TZ
bash runmany.sh list_c nocoq
date -u +%FT%TZ
```

What it printed, `runall.log`:

```text
2026-10-26T20:42:34Z
done 724 files from list_fa
2026-10-26T21:07:17Z
done 724 files from list_c
2026-10-26T21:27:52Z
```

`runmany.sh`, 464.18's at sixteen at a time rather than eight:

```sh
#!/bin/bash
# Run every .g named in $1 through one.sh, 16 at a time, appending to $1.log.
cd "$(dirname "$0")"
rm -f "$1.log"
xargs -P 16 -I{} bash one.sh {} "$2" < "$1" >> "$1.log" 2>&1
echo "done $(wc -l < "$1") files from $1"
```

`summary.py`:

```python
"""Read every fa_*.out beside this file and print, one line per case, what
Gappa printed for each goal: (sum e) / Phat and P -/ Phat in u, each with how
far past k u it is in u^2, and A as a multiple of 2^-1075. Then check each
case: (sum e) / Phat within k u + 64 u^2, k the most roundings a term goes
through on its way to y; P -/ Phat within (left - 1) u + 64 u^2, left - 1 the
additions that fold the permanent; and A within exactly 2^-1075 for each
product that is not normal. Gappa prints a bound with a little more than its
u^2 part (464.13's 5), so the counts within 7 u^2 and 10 u^2 are of cases
whose u^2 part is at most 7.0001 and 10.0001."""
import os
import re
import sys
from fractions import Fraction

os.chdir(os.path.dirname(os.path.abspath(__file__)))
U = Fraction(1, 2**53)
TINY = Fraction(1, 2**1075)
ROOM = Fraction(1, 10000)
NUM = re.compile(r"(-?\d+)(?:b(-?\d+))?")
NAME = re.compile(r"fa_([ns]+)(?:_c(\d+))?")


def value(text):
    m = NUM.fullmatch(text)
    mant, exp = int(m.group(1)), int(m.group(2) or 0)
    return Fraction(mant) * (Fraction(2) ** exp)


def interval(line):
    m = re.search(r" in \[(-?\d+(?:b-?\d+)?)(?: \{[^}]*\})?, (-?\d+(?:b-?\d+)?)(?: \{[^}]*\})?\]", line)
    return value(m.group(1)), value(m.group(2))


def chain(left, cancel):
    """The most roundings a term goes through on its way to y with every term
    normal: its own, and each rounded sum, as 464.13's 6 counts them."""
    return max(1, left - 1 - cancel)


def read(name):
    text = open(f"{name}.out").read()
    out = {}
    if "Results:" not in text or "pointless" in text:
        return out
    for line in text.split("Results:", 1)[1].splitlines():
        line = line.strip()
        if " in " not in line:
            continue
        goal = line.split(" in ")[0]
        lo, hi = interval(line)
        top = max(-lo, hi)
        # Gappa prints a goal by what it names, so with one term left P -/ Phat
        # is |...| -/ |...|, and with one amount A is t5 - X5 or the like.
        if " -/ " in goal:
            out["P"] = top
        elif " / " in goal:
            out["e"] = top
        else:
            out["A"] = top
    return out


def past(x, k):
    """How far x is past k u, in u^2."""
    return (x - k * U) / (U * U)


names = sys.argv[1:] or sorted(f[:-4] for f in os.listdir(".")
                               if f.endswith(".out") and NAME.fullmatch(f[:-4]))
odd, rows = [], []
for name in names:
    r = read(name)
    m = NAME.fullmatch(name)
    states, cancel = m.group(1), m.group(2) or ""
    left = len(states)
    if "e" not in r or "P" not in r:
        odd.append(name)
        continue
    k = chain(left, len(cancel))
    rows.append((name, states, left, k, r))
    print(name, f"e: {float(r['e'] / U):.6f}u = {k}u{float(past(r['e'], k)):+.4f}u^2",
          f"P: {float(r['P'] / U):.6f}u = {left - 1}u{float(past(r['P'], left - 1)):+.4f}u^2",
          f"A: {r['A'] / TINY}*2^-1075" if "A" in r else "")

print()
print(f"cases read: {len(rows)}; no figures or pointless: {odd}")
for label, key, ku, cap in (("(sum e) / Phat", "e", "k u", 7), ("P -/ Phat", "P", "(left - 1) u", 10)):
    excess = [(past(r[key], k if key == "e" else left - 1), name) for name, _, left, k, r in rows]
    within = sum(1 for x, _ in excess if x <= 64)
    small = {c: sum(1 for x, _ in excess if x <= c + ROOM) for c in (7, 10)}
    most = max(excess)
    whole = max((r[key] / U, name) for name, _, _, _, r in rows)
    print(f"{label} within {ku} + 64 u^2: {within} of {len(rows)}; "
          f"within {ku} + 7 u^2 and a little more: {small[7]}; within {ku} + 10 u^2 and a little more: {small[10]}")
    print(f"  most past {ku}: {most[1]}, {float(most[0]):.4f}u^2, "
          f"and past {ku} + {cap} u^2 by {float((most[0] - cap) / U):.10g}u^3 at most; "
          f"largest: {whole[1]}, {float(whole[0]):.6f}u, {float(past(whole[0] * U, 5)):.6f}u^2 over 5u")
not_a = [name for name, states, _, _, r in rows if r.get("A", 0) / TINY != 6 - states.count("n")]
print(f"A within exactly 2^-1075 for each product not normal: {len(rows) - len(not_a)}; not: {not_a}")
```

`python3 summary.py` printed one line for each case, such as

```text
fa_nnssss e: 5.000000u = 5u+7.0000u^2 P: 5.000000u = 5u+10.0000u^2 A: 4*2^-1075
fa_snssss_c4 e: 4.000000u = 4u+2.0000u^2 P: 5.000000u = 5u+10.0000u^2 A: 5*2^-1075
fa_ssssnn e: 2.800000u = 5u-19815838360430180.0000u^2 P: 2.000000u = 5u-27021597764222952.0000u^2 A: 4*2^-1075
```

and then:

```text
cases read: 722; no figures or pointless: []
(sum e) / Phat within k u + 64 u^2: 722 of 722; within k u + 7 u^2 and a little more: 722; within k u + 10 u^2 and a little more: 722
  most past k u: fa_nnssss, 7.0000u^2, and past k u + 7 u^2 by 1u^3 at most; largest: fa_nnssss, 5.000000u, 7.000000u^2 over 5u
P -/ Phat within (left - 1) u + 64 u^2: 722 of 722; within (left - 1) u + 7 u^2 and a little more: 338; within (left - 1) u + 10 u^2 and a little more: 722
  most past (left - 1) u: fa_snssss_c4, 10.0000u^2, and past (left - 1) u + 10 u^2 by 10u^3 at most; largest: fa_snssss_c4, 5.000000u, 10.000000u^2 over 5u
A within exactly 2^-1075 for each product not normal: 722; not: []
```

`controls.py`:

```python
"""For each c_fa_*_half control beside this file, read its best and say
whether it is twice the u count its case printed for (sum e) / Phat: a case
that proves at B = 6u + 2^-99 with that sum within x u fails at B = u / 2 with
a best near 2x. Gappa's best for a goal it does not prove can change from run
to run (464.26), so this counts the bests within a thousandth of 2x, under it
and over it, and says nothing finer."""
import os
import re
import subprocess
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
summary = subprocess.run([sys.executable, "summary.py"], capture_output=True, text=True).stdout
ucount = {}
for line in summary.splitlines():
    m = re.match(r"(fa_\S+) e: ([0-9.]+)u", line)
    if m:
        ucount[m.group(1)] = float(m.group(2))

at, under, over, pointless, nobest = 0, [], [], [], []
for f in sorted(os.listdir(".")):
    if not (f.startswith("c_fa_") and f.endswith("_half.out")):
        continue
    text = open(f).read()
    case = f[2 : -len("_half.out")]
    if "pointless" in text:
        pointless.append(case)
    # With one term left Gappa names y by what it is, float<...>(X0).
    m = re.search(r"BND\(\(\S+ - Ex\) / eb\), best: \[([^,]+), ([^\]]+)\]", text)
    if not m:
        nobest.append(case)
        continue
    best, want = float(m.group(2)), 2 * ucount[case]
    if abs(best - want) <= 1e-3:
        at += 1
    elif best < want:
        under.append((case, best, want))
    else:
        over.append((case, best, want))
print(f"_half controls read: {at + len(under) + len(over) + len(nobest)}")
print(f"_half controls whose best is twice their case's u count: {at}")
print("under it:", under)
print("over it:", over)
print("pointless:", pointless)
print("no best for (y - Ex) / eb:", nobest)
```

`python3 controls.py` printed:

```text
_half controls read: 722
_half controls whose best is twice their case's u count: 722
under it: []
over it: []
pointless: []
no best for (y - Ex) / eb: []
```

**9. What this leaves.** No row. What is not a row:

- Q2767, on the doc of `product_is_normal` in `cascade.rs`, already waits on
  Q2760. 6 here is a third predicate, after 464.14's sphere and 464.18's
  plane, where that doc's claim does not hold: without its checks, a product
  below the normals would not make this filter certify a sign it has not
  earned.
- Q2700 moves the Gappa scripts that rows of 464 appended into
  `proofs/gappa/`, and that takes in the eight of 8.
- Whether dropping the six checks would turn a refusal at level 1 into an
  answer was not run (6). The checks stay, so nothing asks for it (§13).
- The claim on `value`, rather than `y`, with a product below the normals was
  not run. `decide` needs only `y` (2).
- `field` on these scripts' hints (464.10).
