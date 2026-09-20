## 67. A piece bounded by two straight sides, recorded 2026-09-07

`two_solids_fused_by_a_stranger` — the fuzz target that reaches `Model::fuse`
with a stranger's bytes — crashed `arrange.rs`:

```
a piece bounded by straight sides has at least three of them, this one has 2
```

The assert's own doc comment said why it could not be reached: *"Two straight
lines enclose no area, so a piece like that is a zero-area face, which §17.4
prevents rather than filters: reaching here means the splitter is wrong."* It was
wrong the way §16.10 says such sentences go wrong — true about the splitter and
silent about the input.

### 67.1 What the bytes are

Two blocks ten across, a long way apart, and the second one has two of its walls
written down carrying planes that are not theirs. Its `x = 5` wall carries the
plane of its `y = 5.1865234375` wall, leaned by `1.8e-304`; its `x = 15` wall
carries the plane `x = -5`. The corners the model stores are a perfectly ordinary
box.

Two things had to fail to see it, and both are load-bearing:

**`Model::validate` measures the corners and allows the model's own declared
size.** `Model::allowed_wobble` is `Scale::default_linear_tolerance` plus what
writing a plane down costs, and the first of those is the model's `nominal_size`
times `Scale::RELATIVE_LINEAR_TOLERANCE`. These bytes declare a part
`1.8446744073709552e21` mm across — two hundred light-years — which buys
`1.8e12` mm of room off a plane. The two walls are twenty and ten out and walk
straight through.

**The fuse does not measure the corners at all.** `bake::derive_a_corner` builds
each corner afresh as the meet of three of the surfaces around it and then checks
that every surface around it passes through *that* point. A solid whose planes
agree with each other and whose stored corners are somewhere else is
self-consistent under that check, so it bakes — into a different solid. Here the
baked bottom face is a quadrilateral with a corner at `x = -5.5e304`, and two of
its four sides run along two lines that meet at one of its corners and are
`9e-304` apart at the next one along, which no `f64` beside a coordinate of five
can show. The walk comes back with a cycle of two straight sides.

`Arrangement::crossing_like` is why the two lines stay two. It folds two
different partners that cut a face in the *same* line into one crossing, by an
exact determinant on stored coefficients (`relation_of_two_lines`). Two planes a
hair apart are not the same line, so nothing folds them, and the two-sided cycle
is what is left.

### 67.2 The judgement: a refusal, not an assert

CLAUDE.md's rule is panic on a broken internal invariant, `Err` on bad caller
input. A cycle of two straight sides is the second, and §38.2 already made this
exact call one bug earlier — *two arcs lying on top of each other hit an assert;
coincident input is data, not a broken invariant*.

`wire_from` now returns `Invalid::WireTooShort { count }`. That is not a new name
for the rule: it is the one `Model::validate` and `SolidBuilder::add_wire`
already state — *three steps, or one edge that closes already, or two different
edges that are not both straight*. The condition the assert tested is that rule
exactly, so the refusal reuses it rather than inventing a synonym.

It is a **limit** and not a gap (§16.10). The piece is a sliver whose two corners
are `9e-304` apart beside coordinates of five: no scaling reaches it, because the
ratio is `1e-304` and an `f64` carries sixteen digits at a point. The answer does
not exist in the arithmetic the kernel is built on, so it is refused rather than
approximated.

### 67.3 Only the tolerant draw ever saw it

`crates/rcad-topo/tests/a_piece_of_a_face_bounded_by_two_straight_sides.rs`
carries both halves. Under `Policy::certified()` the same fuse never reaches the
walk — it gives up on `where two planes cross` for two of the leaning planes and
returns `Error::Undecidable`. It takes a policy with slop to approximate that
crossing and carry on into an arrangement built on it, and the corpus entry drew
`Policy::tolerant(1.8e21)`.

That is worth writing down because it is the shape of what the target is for. The
target's own header argues that a model that *passes* `validate` might still
crash `fuse`; the answer is yes, and the route was neither of the two the header
guessed at. It was the model's declared `Scale` — a number nothing measures
against anything — widening two separate checks at once.

### 67.4 What is still owed, and not done here

Two things, both outside this lane's files, both reported rather than made:

**`Model::fuse`'s `# Errors` list does not mention this refusal.** It already
names `Error::Invalid` for three other reasons, so the variant class is
documented and the sentence is missing rather than wrong. `fuse/mod.rs`.

**Nothing compares a baked corner with the corner the model stored.** That is the
hole the bytes went through, and it is a real one: `bake` reconstructs geometry
the caller supplied and never checks the reconstruction against what was
supplied. Whether that check belongs in `bake` or in `validate` — and whether
`allowed_wobble` should scale with a number the model declares about itself at
all — is a decision, not a patch, and it wants its own lane.

### 67.5 What the test carries, and why it carries bytes

The test replays the corpus entry's bytes rather than describing the solid
through `SolidBuilder`. That is a deliberate second choice. Four hand-built
versions were tried — a block with a leaned wall, a block with a wall wearing its
neighbour's plane, two blocks a subnormal apart, and a sweep over both the lean
and the slop — and every one of them was refused somewhere *else* first:
`AFaceMeetsItsNeighbourOnItsOwnSurface`, `ResultIsNotManifold { count: 1 }`,
`CannotTellTheCellsOfAFaceApart`, and a winding the sweep could not decide. Each
refusal is correct; none of them is this bug. A hand-built pair that differs from
the failing one in a way nobody noticed is the `fuse-debugging` skill's own named
trap, and the bytes are the one description that cannot drift.

### 67.6 §70 took the driver away, and three corrections to what is above

§67.4 filed *"nothing compares a baked corner with the corner the model stored"*
and *"whether `allowed_wobble` should scale with a number the model declares
about itself at all"* as owed work. §70 took the second half: the allowance is
now the smaller of what the model declares and what its own corners show. The
consequence for this section is that **its own test lost its driver.** The corpus
entry no longer gets past the sweep, so nothing here reaches the walk any more.

The refusal in `wire_from` is unchanged and still right. What changed is that
today it is guarded by a refusal upstream rather than by a test that drives it,
and the test file now says that in as many words instead of quietly passing.

**Correction one: three walls are out, not two.** §67.1 counts twenty and ten.
Measured on `2d84b9b`, the worst corner of each wall that is off its own plane
is `20`, `10` and `0.3593845367340691` — three walls. The third was missed
because it is small enough to read as noise beside the other two and is not.

**Correction two: the number in the refusal is a witness, not a maximum.** The
sweep returns `VertexOffItsFacePlane { distance: 9.536734069115482e-6 }` on these
bytes, which is *none of those three walls*. `check_flat_face` returns on the
first corner it finds outside the allowance, walking faces in stored order, so
the distance it carries is whichever corner it happened to reach first. Reading
it as "how bad is the worst" is wrong, and `the_sweep_names_a_witness_and_not_the_worst_wall`
now pins both halves. The allowance these are measured against is
`1e-9 × 10.359375 = 1.0359375e-8`, the largest coordinate the model actually has.

**Correction three: §67.5's census is softer than it reads.**
`Plane::from_coefficients` takes `a * x + b * y + c * z + d = 0`, so its fourth
number is *minus* the offset — and every hand-built reconstruction in §67.5 fed
it the offset. Those solids were the wrong shape, and the old `allowed_wobble`
was wide enough to let them through the builder without saying so. So of the four
refusals §67.5 lists, an unknown share were refusing a mistake rather than
refusing this bug. The conclusion §67.5 drew from them — carry the bytes — was
right for a different reason than the one given: the bytes cannot drift, and a
reconstruction that is silently ten units wrong is exactly the drift.

### 67.7 What can still reach the walk, and what is only a census

With the sign fixed, a solid whose wall carries a hair of lean **builds and
passes the tightened sweep**: normal `(-1e-100, 1, 0)` on a wall at `y = 5` puts
that wall's own corners `1e-99` off it, which is nothing beside a coordinate of
ten. So the ingredient the two-sided walk needs is still legal, and
`two_walls_a_hair_apart_are_caught_before_the_walk_gets_a_two_sided_piece` builds
it: two knives whose far walls sit at `y = 5`, one plain and one leaned, cutting
one face in two lines that meet at `x = 0` and are `1e-99` apart at `x = 10`.

It does not reach the walk. Under `Policy::tolerant` it is
`TwoCurvesTouchWithoutCrossing`; under `Policy::certified` it is
`Error::Undecidable` on the two planes. Both are correct, and both are upstream.

**This is a census, not a demonstration, and the difference matters here more
than usual.** Nine shapes have now been tried across §67 and this section — four
in §67.5, five here — against six distinct refusals. That is evidence the walk is
hard to reach. It is not the demonstration §16.10 asks for before writing *this
cannot happen*, and §67 is the section that made §16.10's point about the
original assert, so it is the last place to weaken it. What can be said exactly:

> The walk needs two crossings that are two different lines sharing both nodes.
> `crossing_like` folds same-line partners, so the two lines are genuinely
> different and meet at one point; the second shared node exists only because the
> arrangement could not tell two points apart. Every construction tried that
> produces two planes that near is stopped by `TwoCurvesTouchWithoutCrossing`,
> `CannotTellTheCellsOfAFaceApart` or an undecided plane crossing first.

Nothing rules out a fourth arrangement of those same ingredients that slips
between all three. So the refusal stays, unweakened, and the file that used to
drive it now guards §70's cap instead — which is the other thing §67 asked for.

### 67.8 One more §16.10-shaped doc claim, next door

`bake.rs:535-538` says the stored corner is what picks between multiple meet
points when a triple of surfaces has more than one. The code consults no stored
corner: `let [only] = places.as_slice() else { continue; }` passes over every
multi-place triple without looking. That is a filter described and not written,
in the same function §67.1 blames for never comparing a baked corner with a
stored one — and the two are the same gap seen twice. Not fixed here; `bake.rs`
is not this lane's file.

---

