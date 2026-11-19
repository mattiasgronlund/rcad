# rcad

A solid-modelling kernel for CAD, written in Rust.

rcad stores a solid by its skin: faces lying on surfaces, edges lying on curves,
and the corners where edges meet. This is called a **B-rep** (boundary
representation), and it is how CAD programs hold parts. On top of that, rcad can
join, cut and overlap solids (union, difference, intersection), say whether a
point is inside a solid, measure its volume and area, turn it into triangles, and
read and write STEP files.

**What makes rcad different is that it does not guess.** Every geometric question
the kernel asks itself, such as "which side of this plane is this point on?", is
either *proved* or answered with *"I cannot tell from these numbers"*. There is no
hidden tolerance that quietly rounds a close call one way. A refusal is a real
answer that the caller must handle. A wrong answer is the worst thing the kernel
can do, and refusing is better than handing back a broken solid.

This is research code. The version is `0.0.0` and nothing is published. The work
ran from 2026-08-30 to 2026-11-07.

## What it can do today

- **Surfaces:** plane, sphere, cylinder, cone and torus. **Edges:** straight
  lines, circles and ellipses. NURBS curves and surfaces exist in `rcad-geom`,
  but no solid is built from them yet.
- **`Model::fuse`:** union, intersection and difference. One call can take
  several solids on each side. It also says which input face each output face
  came from.
- **`classify_point`:** inside, on the skin, outside, or cannot tell.
- **`measure`:** volume, area and centre of mass, each with a number for how
  much its own sums rounded.
- **`mesh`:** triangles that stay within a distance you choose of the true
  surface.
- **`validate`:** checks the whole model. For example, every edge is used by
  exactly two faces, every wire closes, and nothing is left dangling.
- **STEP read and write** (ISO 10303-21). The writer and reader are checked
  against each other.
- **Two policies.** `Policy::certified()` proves everything or refuses.
  `Policy::tolerant()` accepts a bounded amount of slop to get an answer, and
  records what it gave up. The policy is passed to every call. It is never
  stored in the model, so the same model cannot give a different answer because
  of a setting somewhere else.

**On real files:** the tests use 367 pinned STEP files, 323 from KiCad's 3D
model library and 44 from NIST's test models (`corpus/sources.md`). 165 of the
323 KiCad files import, and all 165 write out and read back as the same solid.
Only 1 of the 44 NIST files imports. Every other file is refused with a named
reason. Nothing is ever "healed" on import.

**Not built yet:** fillets, chamfers, shells and offsets (roadmap §8); repairing
dirty imports (§9.2); curvature and curve fitting (§4.3, §4.4); booleans on NURBS
surfaces; and more than one solid at the top level of a model. `just plan open`
lists every roadmap item that is not done.

## A first look

This is cut down from the example in `crates/rcad/src/lib.rs`, which runs as a
test.

```rust
use rcad::{Containment, Deflection, Model, Point, Policy, ProvenanceLog, Scale};

let scale = Scale::millimetres(100.0)?;
let mut model = Model::new(scale);
let mut log = ProvenanceLog::new();
let cube = model.add_cube(10.0, &Policy::certified(), &mut log)?;
model.validate(&Policy::certified(), &mut log)?;

let measured = model
    .solid(cube)?
    .measure(Deflection::for_scale(scale), &Policy::certified(), &mut log)?;
assert_eq!(measured.volume(), 1000.0);

match model
    .solid(cube)?
    .classify_point(Point::new(5.0, 5.0, 5.0), &Policy::certified(), &mut log)
{
    Containment::Inside => {}
    Containment::OnBoundary | Containment::Outside => panic!("wrong side"),
    // Not an error: the kernel could not prove the answer from the numbers.
    Containment::Undecidable { enclosure } => panic!("could not tell: {enclosure}"),
    // The enum grows. Guessing what a new answer means is worse than stopping.
    other => panic!("unhandled answer: {other:?}"),
}
assert!(log.is_empty(), "nothing had to be approximated");
```

There are three levels of "cannot tell", and they mean different things:

| Where | What "cannot tell" looks like | What you do |
|---|---|---|
| One sign test | `Decision::Undecidable` | Handle it. It is an answer. |
| Is this point inside | `Containment::Undecidable` | Handle it. Same. |
| A whole operation | `Error::Undecidable` | It failed. The error lists which decisions gave up, and by how much. |

## How the code is laid out

| Crate | What it holds |
|---|---|
| `rcad` | The front door. Re-exports everything a caller needs. |
| `rcad-predicates` | Sign tests that prove their answer or say they cannot: first plain `f64` with an error bound, then intervals, then exact arithmetic. |
| `rcad-geom` | Points, moves, surfaces, curves, NURBS, and where two surfaces cross. Knows nothing about faces or edges. |
| `rcad-topo` | The model: vertices, edges, faces, shells and solids in flat arenas with typed keys. `validate`, `classify_point`, `fuse`, `measure`. |
| `rcad-mesh` | Triangles for one trimmed patch of one surface. Cannot see topology at all. |
| `rcad-step` | The STEP reader and writer. |
| `rcad-fuzz-explain`, `rcad-fuzz-timing` | Tools for reading what the fuzzer found and how long it took. |

Elsewhere: `fuzz/` has five fuzz targets, `corpus/` pins the STEP files,
`proofs/` has Gappa proofs of the error bounds, and `tooling/` has the scripts
that run the work.

`docs/roadmap.md` is the long-range plan, written before any code. It is **not**
a description of this code. `docs/decisions.d/` records what was actually decided
and why, one file per section, 464 sections so far. Where this README cites `§N`,
run `just decision N` to read it. `docs/queue.md` is the list of open work.

## Building and testing

The compiler is pinned in `rust-toolchain.toml`. Everything else (`just`,
`cargo-nextest`, `cargo-deny`, and more) is pinned in `mise.toml`. Run commands
through `mise exec --` so the build cache is used:

```console
mise exec -- just                 # every recipe, one line each
mise exec -- just t <test name>   # one test, or a few
mise exec -- just gate quiet      # everything CI runs, summed up in about fifty lines
```

The STEP tests need the corpus. `corpus/fetch.sh <dir>` downloads it and checks
every file against `corpus/manifest.sha256`. The tests look for it in
`RCAD_STEP_CORPUS`, which defaults to `rcad-step-corpus` next to your clone. The
tests **fail** rather than skip when the corpus is missing.

Every crate has `#![forbid(unsafe_code)]`. `cargo-deny` refuses any LGPL
dependency.

## How it was built

Mattias set the direction and made the calls. The code was written by Claude
Code sessions. Big work ran as **lanes**. One agent, in its own git worktree,
takes one row of `docs/queue.md`. A second agent checks that the lane's new test
fails without the change and passes with it. Then the change merges, but only on
a green gate.

In ten weeks that came to 11,766 commits, 1,577 merged lanes, 464 sections of
decisions and 3,220 queue rows, 2,223 of them done. The code is about 218,000
lines of source and 248,000 lines of tests, with about 4,650 test functions.
Many of those are property tests that draw thousands of shapes each.

## What we have learnt

### Highlights

- **We reached the roadmap's "now you have a kernel" point in three days.** The
  roadmap said that union, intersection and difference on curved solids (its M4)
  takes a small team 18 to 30 months. It merged 71 hours after the first commit,
  and STEP import (M5) 28 hours after that. That is agents, not a team, so it is
  not a like-for-like comparison. But §39.9 says it plainly: "the
  architecture is a kernel's and the coverage is not". The ten weeks since went
  into the long tail of near-misses, and that tail is the real work.
- **"Cannot tell" as an answer works.** It fell out of the design and was among
  the easier pieces to build (§12.1). The clean control shapes refuse 0 of
  2,048 pairs, and a test holds them to zero (§30). When the kernel does refuse,
  it names the decision that gave up and by how much, so a refusal can be
  followed to the place in the code that raised it (§110, §113).
- **Every refusal must be shown to be a *limit* or a *gap*.** A limit is a case
  where the answer really cannot be had from `f64` numbers. A gap is a case where
  the answer exists and the arithmetic did not reach it. Gaps get fixed (§16.10).
  Two towers `1e308` tall were refused three separate times, each time a gap.
  Now they unite by proof alone (§17).
- **Being strict costs few answers.** Of the certified refusals on shapes people
  really make, the tolerant policy answered only 51 of 219 (§446). On a sample
  of harder, made-up shapes it answered 9 of 211 (§449). A tolerance helps only
  where two things touch and the numbers miss the touch by a rounding. Most
  refusals are not that: 84 of the 143 that tolerance refused the same way come
  from the mesher (§446).
- **Scaling by a power of two is exact, so it is both a test and a fix** (§18).
  A sign test at `1e-200`, or at `1e300`, used to refuse. Now it rescales and
  answers. The surface-crossing walker stays at 0.577 of its own error bound at
  every scale from 2⁻²⁰⁰ to 2²⁰⁰ (§30.7).
- **A new kind of shape beats more of the same shapes.** Three million fuses
  from the existing generators found nothing in 934 seconds. One new family, a
  profile with a point halfway along a wall, found the bug in 0.02 seconds and
  under forty fuses (§17). The same thing happened again with the first tilted
  family (§22).
- **The layers held.** The sign-test crate did not change one file while curved
  booleans and STEP were built on top of it (§39.5, §43.5).
- **Real files came along.** At M5, 46 of the 323 KiCad files imported. Now 165
  do, and every one of them round-trips (§43, §0).
- **Proofs paid off in an unexpected way.** Gappa proved all nine of the fast
  sign filters correct (§483.2). Writing the cone's proof meant reading its
  code line by line, and that reading found a wrong sign that the fuzzer had
  never reported.
- **Recording why is cheap.** Logging every decision that was not proved costs
  at most 0.097% of fuzzing time (§162). A fully proved operation logs nothing.

### What makes a B-rep kernel hard

None of the maths is deep. What is hard is that every number is rounded, and a
solid is only right if thousands of rounded numbers agree with each other about
which side of which surface everything is on.

- **A corner you computed is a tiny box, not a point.** Three planes meet at a
  corner, but you only know that corner to within a few roundings (§2.5). Most of
  the time the box is far from anything that matters. When a cut passes
  close by, it is not. Storing `1/√2` correctly rounded made one corner four
  roundings wide, and a cut through it now has to refuse. The old answer had
  been luck in the last two bits (§130.5).
- **The same plane written two ways must be the same plane.** Wall normals
  `(1,1,0)` and `(3,3,0)` came out one rounding apart once they were scaled to
  length one. So the answer of a fuse depended on which solid came first (§22).
  The fix was a length-one step that is correctly rounded and depends only on
  the direction (§120, §130).
- **Near-tangency is a subtraction that cancels.** A ball one rounding bigger
  than a pipe, tilted by `2.1e-8`: `1 − c·c` rounded to exactly zero and the
  walker refused. A step later the same number was 50% wrong and nothing
  noticed (§112). It was rewritten as a sum of squares.
- **Far from the origin, small steps vanish.** Two pipes `6.7e-7` apart and
  `1e10` from the origin: the step between their axes rounded to zero, and the
  crossing was lost (§145). A sweep then found four more sign tests with no
  exact fallback under them (§142).
- **Surfaces have special points.** A cone's tip, a sphere's poles, the seam
  where a cylinder's angle wraps round. Long runs of this record are about a
  cone's apex alone. No face patch may span more than a quarter turn, and that
  one rule was the most expensive idea of M2 (§26.2).
- **Designers make tangent, flush and coaxial shapes on purpose.** A ball capping
  a pipe touches it along a whole circle. At one point that was 18 of the 24
  corpus refusals, and every one of them was a true limit (§117).
- **Rounding breaks coincidences that the designer built in.** Turn a cube a
  little about two axes and set it on another cube's edge. The stored planes then
  meet `1.7e-21` apart, and the kernel cannot tell an edge from a gap. That pair
  is a limit (§482). Carrying "these two are the same" over from how the shape
  was built answers some cases like it: since Q2575 the fuse does it for the
  faces round one corner (§448.8).
- **The tail never ends.** Of 201 open rows at one count, 166 were nearly
  degenerate shapes at ordinary sizes (§348.5). The roadmap gave booleans about
  30% of the hard part, and that held. It gave the numeric basics "~0%, use
  libraries", and that did not hold: scaling, normalising a vector and building
  a plane took §18, §20, §22, §120, §121, §130 and §142. And it left out the
  mesher, which raises most of the refusals that tolerance does not fix (§446).
- **Finding bugs is fast. Fixing them is slow.** Roadmap §11.4 predicted this,
  and the growing queue is what it looks like.

### How many bugs can be found without goldens?

A *golden* is a stored expected answer, or another kernel's answer to compare
against. **rcad's own tests use no geometric goldens.** None of them compares a
shape, a volume or a mesh against a stored output file or against another
kernel. The files the tests load are inputs: STEP files, two saved shapes, the
fuzzer's dictionary and proptest's saved seeds. Inside this repo, the one place
another program's answer is used is the STEP *text*: the `step-io` crate reads
the same files, and what our writer wrote, and must agree entity for entity
(2,366,598 entities at M5, §43.5). Outside it, `rcad_testing` runs OpenCASCADE's
own test cases against rcad (§242, and the last section below). The
nearest thing to a golden is a pinned count, such as which corpus files import,
or how often one family of shapes refuses. A pinned count catches a change in
either direction, but it says nothing about whether an answer is right (§40.8).

What the tests use instead:

| Check | What it checks | Needs a known answer? |
|---|---|---|
| Fuse laws | `A ∪ B = B ∪ A`, `(A ∪ B) ∪ C = A ∪ (B ∪ C)`, `A ∪ A = A`, `A \ (A \ B) = A ∩ B`, `vol(A ∪ B) + vol(A ∩ B) = vol(A) + vol(B)` | No |
| What the rule means | Points sampled on a grid must lie where the rule puts them: in A or B for a union, in both for an intersection, in A and not B for a difference. Two families of flat-faced solids only (`fuses_mean_what_their_rule_says.rs`) | No |
| Move, turn and scale | The same shape moved, turned or scaled by a power of two gives the same answer | No |
| The kernel's own checks | `validate`, and the fuse checking its answer before it hands it back | No |
| Round trip | Write to STEP, read back, and get the same solid | No |
| Proof or refusal | Every sign is proved or reported as unknown | No |
| Fuzzing | No input makes it panic, hang, or corrupt the model | No |
| Real files | 367 STEP files must import and validate, or be refused by name | No |
| Counting unit cubes | Boxes on a whole-number grid, whose union or difference can be counted cube by cube (12,600 fuses, §242.8) | Computed independently, not stored |
| Hand-worked answers | A ball nothing cuts is still a ball; a known volume such as `15³ + π(34 − 15√6/2)` | Worked out by hand, not stored |

**How much that catches.** The queue has 1,445 rows tagged as defects. 1,247 of
them (86%) are refusals and 48 (3%) are crashes. Both of those announce
themselves: the kernel says it gave up, or it stops. Deciding whether a refusal
is a bug takes no golden either. It is a bug when an answer exists, and that is
shown by the same shape answering after a move, a turn, a power-of-two scaling,
or under the tolerant policy (§16.10, §191.4).

That leaves 150 wrong answers (10%). These are the only ones that need something
to compare against. We sorted every one of them by what first showed it was
wrong:

| What showed it | Rows |
|---|---|
| A hand-worked answer: "volume 9 where it must be 8", "13 faces where the plate has 16" | 47 |
| Write it out and read it back, and it is not the same | 29 |
| Two routes inside the kernel disagree, such as the exact sign against the fast one | 21 |
| Read in the code; no shape has been run yet | 20 |
| The same shape moved, turned or scaled answers differently | 12 |
| The kernel's own checks: `validate`, or the next operation refusing | 9 |
| A fuse law, or the order of the arguments | 8 |
| An outside reference | 4 |

The four rows in the last line are one defect: which way a torus face points.
The STEP standard and files written by other CAD systems settled it, and showed
that our mesher was the part that was turned round (§329). **So 146 of the 150
were found with no golden at all.** Two cautions: 150 rows are fewer than 150
defects, because a defect that was filed again gets a new row. And the queue
only began on 2026-09-25 (§230). Before then, defects were written up in the
decision sections, and for the planar fuse "every defect that mattered … was
found by a metamorphic property, not by a hand-written case" (§17).

**What these checks cannot catch.** A law only compares two answers, so two
answers that are wrong the same way pass it. `validate` only checks that a solid
is a solid, not that it is the right one. Take a cube with a ball-shaped hollow,
the ball that fills it, and a second ball in the same place: four fuses of those
gave wrong answers that `validate` accepted. Nothing saw it until someone worked
out what the right answers had to be (§242.8). A law also has to be *run* on the case. `A
∪ A = A` was a law here from the start, and yet `fuse([a], [a], Union)` returned
nothing, because the law had only ever been tried on a copy of `a` and never on
`a` named twice (§242.6). Laws cannot say what the answer *is* when every order
refuses. And the checks themselves had bugs (§98, §149).

**The real cost is not goldens, it is shapes.** Laws find bugs only in shapes
someone thought to draw. Three million fuses of the old shapes found nothing; one
new family found the bug in forty (§17). The real KiCad files found three faults
that no generator made (§191.1). And the laws found less as time went on.
They found every defect that mattered in the planar fuse (§17). But they were
the first to show only 8 of the queue's 150 wrong answers, where hand-worked
answers on new shapes showed 47. So the answer
to "how far without goldens" is: almost all of the way, as long as you keep
inventing new kinds of shape and working out what their answers must be.

### What more these methods could give

We sorted every problem we hit by its root cause. Was it a method doing a
different job from the one we asked of it, a check too hard to write, or a
check, rule or setting we had not added or set right? Only one was a method
doing another job: tolerance used to get more answers (see Highlights). None
was shown to be a check too hard to write, and here a claim that something
cannot be done needs a demonstration (§16.10). Every other problem was
something not added or not set right. So these methods have more in them than
our numbers show. This section says where, and gives two ways goldens can give
more than one answer each. What has not been tried yet is marked "Not done
yet".

#### Without goldens

- **A fuzzer finds wrong answers once it has a check that needs no known
  answer.** "Did it crash?" is only the check a fuzzer starts with. In four
  nights at about 160 core-hours each that found nothing new, the reader and
  writer targets asked only that (§265.6, §290.7). Since 2026-09-25 the writer
  target reads back what it wrote (`1b7a673c2`). 14 of the queue's 150 wrong
  answers name a fuzz target as where they came from: 9 the writer, 4 the two
  surface-crossing targets, which check that each point they walk is on both
  surfaces, and 1 the fuse target, which checks that both orders agree. Some of
  the 14 file an earlier row again, so they are 9 defects. The tag only exists
  since 2026-10-11 (§433), so these numbers are a floor. Reaching hard cases is
  not the hard part: one October night's corpus for the fuse target held 4,737
  refusals (§437). And 79 of the 150 wrong answers were first shown by checks
  that need no known answer (the table above). Any of those kinds can run in a
  fuzz target, and the targets already run four of them. Not done yet: the fuse
  target ignores what a refusal says, fuses once, and never reads a model back
  twice. A plain test that did all three found three faults the target would
  not have flagged (§483.3).
- **A wrong answer that is a valid solid could be seen without working out the
  answer.** Above, four such answers were seen only once someone worked out the
  right ones (§242.8). Three of the four put a ball's centre on the wrong side.
  The point check, `fuses_mean_what_their_rule_says.rs`, asks exactly that: it
  samples points and checks that each one is where the fuse's rule puts it.
  Not done yet: it runs on two families of flat-faced solids only, never on
  those shapes, and in no fuzz target.
- **A new kind of shape is worth more than more of the same** (see Highlights).
  Each new family so far found what the old ones did not (§17, §22, §191).
  Proofs are one place new kinds come from. Each hypothesis a Gappa proof needs
  asks whether the code makes sure of something. Four times the answer was no,
  and a shape built to break it found a fault each time (§483.3). Real files
  are another, below.
- **A check is worth what it reaches.** M0's profile builder refused 92% of the
  triangles a test drew, and its 335 tests could not see it, because the test
  that used it threw every refused profile away (§12.3). One probe reported
  197,000 healthy samples and tested nothing (§18.3). `A ∪ A = A`, above, is the
  same story (§242.6). What keeps a check honest is cheap. A new test must fail
  without its change, and a second agent checks that on every lane. And a check
  can count what it reached and fail when a case never came up, as the point
  check does for the four ways a point can sit in or out of two solids (§62).

#### With goldens

- **A real file is worth more as a source of shapes than as one answer.** As an
  answer to compare against, a golden checks one case. Read in as a shape, it
  feeds every check above. Our 367 pinned STEP files were first inputs to
  import, and 77 refusal rows name a file as where they came from. They were
  worth more for what they showed about our generators. Each of the three
  faults the KiCad files found sat where every generator kept away: no drawn
  pair was tangent to anything, and every family was built from whole numbers
  or one clean turn. Real parts are tangent everywhere and rounded everywhere.
  The KiCad files also showed how they are rounded: none writes more than 14
  significant digits, where a double needs 17 to come back exactly. Three
  families aimed at those habits, one of them a round trip rounded to 11, 12
  and 13 digits, found five wrong answers (§191.1, §265.6). The 165 KiCad files
  that import now all write out and read back as the same solid, and fusing 21
  of their solids found where the fuse still refuses (§471). Not done yet: the
  laws, the moves and turns, and the point check on every imported solid,
  rather than on the few that tests pin.
- **A set of goldens kept apart from the fixing measures the kernel.**
  `rcad_testing` runs OpenCASCADE's own DRAW test corpus, 17,879 cases, against
  rcad. It counts "the kernel said it could not" apart from "the kernel got it
  wrong", reports counts per area, names no failing case in its default
  report, and keeps a rotating share of the corpus out of every report
  (§242.1). That keeps the kernel from being fixed toward those cases, so its
  counts say how the kernel does on cases nobody aimed at. On 2026-09-25 it had
  run 4,055 of them, and 133 said anything about rcad (§242.1). What it could
  not ask became seven asks (§242.3), and rcad built four things for it, such
  as `Model::validate_solid` (`docs/consumers.md`). Its later counts are its
  own, and none of them is in this repo. Our own corpus was used the other way:
  a refused file became a row to fix (§433), so a rising import count mixes
  fixes aimed at those files with how the kernel does in general.

## Licence

Apache-2.0, as declared in `Cargo.toml`.
