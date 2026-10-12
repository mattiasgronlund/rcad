## 440. A fuzz finding is run again with ordinary numbers before it is graded — 2026-10-12

*Mattias, 2026-10-12, on hearing that the fuzzer keeps an input only when it
reaches code no kept input reached before (§437.3's note): "keeping only the
first one reaching a code path, and then prioritising the rows built from them
based on how likely that explicit case would be to occur in real files, might
fool us, as the first one might be very unlikely, but there might be many more
that ARE likely." Three answers were put to him: a cheap retry with ordinary
numbers at triage, a second grading by the lane that finds the fault's trigger,
and a second fuzz target that builds only the shapes designers make. He said
yes to the recommendation: the first two now, the third as a row. This amends
§352.3 and §418.3.*

### 440.1 Why the input the fuzzer keeps is so often an extreme one

`two_solids_fused_by_a_stranger` reads its bytes as a saved `Model`
(`postcard::from_bytes::<Model>` in `crates/rcad-topo/tests/fuzzing/fuse.rs`),
so every coordinate is eight raw bytes and any `f64` bit pattern can come out.
The top byte of a double holds its sign and the top seven bits of its
exponent, so a mutation that changes that byte, and keeps the sign, moves the
number by a factor of at least 2^16. Numbers like `2.17e-311` and `2.36e21` are
what the mutations make, not what a designer types.

The fuzzer keeps the first input that reaches a piece of code, and it never
swaps that input for a more ordinary one that reaches the same code. So the
grade of a row filed from a crash file says how likely that one input is, and
not how likely the fault is. §418.3 recorded that the nights' faults "mostly
came out D".

Q1781 shows the gap. Its crash files are two pipes of radius `5.16e-118`, which
would be D. The triage lane scaled the pair by `2^k` for eleven values of `k`.
Multiplying by a power of two is exact in `f64`, so every scaled pair is the
same shape. Every one failed, including the pair of radius `1.302`. So it went
in as `wrong/C`, not D.

### 440.2 The rule: run the shape again with ordinary numbers before grading it

**Before a fault row filed from a fuzz crash file is graded C or D, its lane
builds the shape again with ordinary numbers and runs it.** Each number that
puts the shape outside §3.5's envelope is replaced, and every relation the
shape had is kept: what touches, what is flush, what shares an axis.

- A scale goes by a power of two, which changes no relation: a pipe of radius
  `5e-118` to radius about 1, or a cube `2.36e21` across to about 10.
- A part that is nearly zero goes to zero: a normal `(0, 2.17e-311, 1)` becomes
  `(0, 0, 1)`.

Then:

- **The same failure** (the same panic line, the same refusal name, or the same
  wrong answer) makes the row at least C. The row names the ordinary shape as
  well as the crash file.
- **No failure** leaves the row D. The row then names the number that puts the
  shape outside the envelope, as §352.3 asks, *and* the ordinary shape that was
  tried, so D rests on two inputs and not one.
- **A row filed without the try has no class.** That is §360.5's rule for a
  guess nobody has run. By §418.3 a row with no class sits in tier 1, after the
  A and B rows and ahead of tiers 2 to 4. That is the cost of skipping the try,
  and it is meant to be: the row is taken soon, and its first step is the try.

The try is minutes of work, not a trace. Q1779's lane did it for Q1781 and
triaged two other groups as well, all in one 14-minute run. It needs no root
cause. It asks only "does it still break with ordinary numbers?".

**The try alone does not make a row B.** B is a shape designers make on purpose
(§352.3), and the try shows only that the shape is in range, not that anyone
would make it. Q1781's lane built the obvious designed pairs, along `z` and
along `(1, 1, 1)`, and they held, so it wrote "Not B". The cheap rule moves rows
from D to C, which matters only inside tier 5, where every C row is taken before
any D row (§403.3). It does not by itself find the likely case Mattias worried
about. 440.3 and 440.4 are for that.

The rule applies to rows filed from now on. The open D rows that came from a
fuzz crash file were graded without the try. A row of its own runs it for them
(440.5).

### 440.3 The lane that finds the trigger grades the row again

A lane fixing a fault often finds which feature of the shape it needs: a corner
a rounding inside a face, two axes through one point, a slope 16 ulps off. When
it does, it asks whether a shape designers make on purpose has that feature,
builds one, and runs it. If the designed shape fails the same way, every row
the lane leaves for that fault is graded B. The section it writes says so,
and names the designed shape.

This costs nothing extra, because the lane is reading the fault anyway. But it
comes late. The row is already being worked, so its own place in the order does
not change. What changes is the grade of the rows it leaves, which go to tier 1
as B and not to tier 5, and §352.6's count of A and B rows.

### 440.4 A second fuzz target that builds only designer shapes

**What it would do.** The fuzzer itself is not limited. libFuzzer still mutates
bytes and still keeps an input that reaches new code. What changes is how the
target reads the bytes. Today's fuse target reads them as a saved `Model`, so
the bytes *are* the numbers. The new target would read them as a list of
choices from short menus, and build the shape through the public builders in
`crates/rcad-topo/src/build.rs` (`add_extrusion`, `add_cube`, `add_round_bar`,
`add_ball`, `add_bar_cut_off_by`):

- **which solid:** a block, an extrusion of a polygon (with holes), a round bar,
  a ball, or a bar cut off by a plane;
- **its sizes:** from a list of numbers a designer types, such as whole
  millimetres and halves, `2.5`, `12.5`, or inches as `25.4`;
- **where it goes:** a relation to a solid already placed: flush on one of its
  faces, centred on that face, along one of its edges, sharing its axis, tangent
  to its round face, or moved off by a whole number of millimetres;
- **its turn:** from a list: none, 90°, 45°, 30°, or a fortieth of a turn;
- **whether to round:** every coordinate to 12 significant digits, as a STEP
  file holds it, which is §352.3's "a file written to 11–13 digits rounds
  apart";
- **then the fuses:** which solids against which, and under which rule.

Every byte string decodes to one such shape, so every input it runs is B by
§352.3's own words, and so is every fault it finds. Nobody has to argue it.

**What the fuzzer controls there** is every one of those choices, and their
order. A shape of three or four solids, each with a few dozen sizes, half a
dozen relations to half a dozen faces, and five turns, has far more
combinations than a night runs: the night of 10-11 ran 7 432 655 inputs on the
fuse target. Coverage still steers the search. When a combination reaches new
code (a bar along a ball's axis reaching the code for a ball's pole, say), it is
kept, and its neighbours are tried next: the same shape with another size,
another face or another turn.

**What it gives up** is every number that is not on a menu, and so every fault
that only such a number reaches. Those faults are C or D by definition.
Today's target keeps running for them.

**Why it can find anything.** Designer shapes still break the kernel. Q1582
(`refuses/B`) is a prism resting on a block's top with both turned by `k/40` of
a radian. Q1281's hand-written sweep over `k` found it. The property suites
already build shapes like these with proptest: `a_block_on_a_block`,
`a_ball_against_a_wall` and the rest of `crates/rcad-topo/tests/touching/`.
They draw at random, with no memory and for seconds. The new target would draw
from the same kind of menus, keep what reaches new code, and run all night.
Whether it finds faults the suites have not is not known until it runs.

**What it needs:**
- the decoder, under `crates/rcad-topo/tests/fuzzing/`;
- the target, under `fuzz/fuzz_targets/`;
- an entry in §298.1's table and in the justfile's target lists;
- a test that replays the inputs it kept, as
  `every_model_the_fuse_fuzzer_kept_is_fused_again.rs` does for today's target;
- `rcad-fuzz-explain` support, so a crash file prints as the shape it builds.

The night's CPU is shared, so the hours on the other targets go down. None of
the five builders makes a cone or a torus. The public face-by-face builder in
`crates/rcad-topo/src/builder.rs` (`add_vertex` to `add_shell`) takes any
surface, so the target can make them that way, but each face, edge and wire
has to be built by hand.

The consumer, per §13, is the grading of this section: a fault this target finds
is B on arrival.

### 440.5 What this leaves

- `lane.md`'s grading paragraph and the `remaining` description in
  `.claude/workflows/lanes.js` carry 440.2 and 440.3, which were changed in the
  same commit as this section.
- A row, tier 1 with no class: run 440.2's try for every open D row that came
  from a fuzz crash file, and re-tag each one that fails the same way.
- A row, tier 2: build 440.4's target.
