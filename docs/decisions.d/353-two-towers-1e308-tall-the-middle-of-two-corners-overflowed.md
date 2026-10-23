## 353. Two towers 1e308 tall: the middle of two corners overflowed — 2026-09-28

*Queue row Q335, from §324.6. Lane `lane-q335-two-towers-1e308-tall`, off
`425c8911`. The certified fuse of two towers `1e308` tall refused on "a point
inside a piece of a face" once §324 had decided the `orient3d` it used to stop
on. **It was a gap, in `select.rs`, and it is fixed.** Every candidate point on
the towers' top is half way between two corners at `z = 1e308`, and
`midway_between` added the two before halving them. `2e308` is past the largest
double, so each candidate came back as a box reaching from `9e307` to infinity,
and no winding could place one. The two towers now unite under
`Policy::certified()` with an empty log. It is not §346's piece narrower than
its round-trip box. And criterion 7's refusing half has lost its demonstration
for the third time (353.5).*

  353.1   Which step made the infinite enclosure
  353.2   A gap, and the fix
  353.3   Not the piece narrower than its round-trip box
  353.4   The pins
  353.5   What this leaves

### 353.1 Which step made the infinite enclosure

**The record's infinite enclosure is not a measurement.** `gave_up` in
`fuse/mod.rs` writes every "a point inside a piece of a face" record at
`Level::Interval` with `f64::INFINITY`, whatever went wrong, because it names a
search and not one predicate with a width. What had to be traced was why the
search found nothing.

A throwaway probe in `place_the_piece` printed the reference point, every
candidate and every turn of the winding whenever it gave up. It was run on the
base with `Q335_PROBE=1 mise exec -- just t -p rcad-topo --test merged -E
'test(~fusing_solids::a_fuse_that_needs_a_decision_nothing_can_prove_refuses)' --no-capture`, then taken
out.

- The piece is the quarter `[0.5, 1] × [0, 0.5]` of the first tower's top, on
  the plane `z = 1e308`. Its four corners are exact.
- `InPlane::of` got its reference point from the second reach:
  `Exact([0, 0, 0])`, below the plane. The first reach, `1e308` further up,
  overflows.
- All ten candidates were `Constructed`, each with `z` in
  `[8.988465674311579e307, inf]`. `x` and `y` were single numbers.
- For every candidate, the turn about the side from `(0.5, 0.5)` to
  `(0.5, −0)` was `Undecidable { enclosure: inf }` at `Level::Interval`. A
  constructed point never reaches level 3 of `orient3d`. Two candidates had a
  second side come out the same way.

So `InPlane::winding_around` said `None` for every candidate, `the_piece_holds`
said `None`, and `place_the_piece` gave up. The step that made the infinite box
is `midway_between`: `(one + other) × ½` in interval arithmetic, and
`1e308 + 1e308` rounds up to infinity.

### 353.2 A gap, and the fix

**The answer exists.** The middle of two corners at `z = 1e308` is `1e308`, a
double, and `(0.625, 0.125, 1e308)` is a point of doubles inside the piece. The
arithmetic could not reach it only because it added before it halved. By
§16.10 that is a gap.

`midway_between` now takes the middle both ways, adding first and halving
first, and keeps the part the two boxes share. Both boxes hold the true middle,
so the part they share does too. **Each way is exact where the other is not**:

- Adding first overflows when the two ends add up past the largest double.
- Halving first loses the last bit of a subnormal end. Half of `2⁻¹⁰⁷⁴` is no
  double.

The claim that one of them always hits the middle, whenever both ends are
exact and the middle is a double, is a sufficiency claim. So here is why it
covers every case. Halving only moves the exponent, so if the middle is a double
then so is the sum, except where the sum's exponent reaches `1024`. With no
overflow, adding first is exact. With overflow, `|a + b| ≥ 2¹⁰²⁴`, so both ends
are above `2¹⁰²⁴ − f64::MAX = 2⁹⁷¹`. Both are then normal, both halves are
exact, and the halves add up to the middle exactly. Two unit tests pin one half
each (353.4).

With the fix the fuse answers: one solid, ten faces (eight walls, a top and a
bottom), and an empty log.

### 353.3 Not the piece narrower than its round-trip box

§346.4 already said so, and the probe agrees. The piece is flat, and a flat
piece's candidates never go through `back_onto_the_surface`: `candidate_points`
returns before that step for a plane. The box here is not a round trip's width
at all. It is an overflow, and it reaches to infinity along the plane's normal.
The only thing the two refusals share is how they look from outside: a
candidate box that no winding can place.

### 353.4 The pins

`a_fuse_that_needs_a_decision_nothing_can_prove_refuses` in `fusing_solids.rs`
is now `two_towers_1e308_tall_are_united_by_proof_alone`. It asserts an empty
log, one solid, ten faces, the top's eight corners, and where eight points are:
four inside, three outside, one on the bottom face.

- **It fails without the fix.** With `midway_between` put back to adding first
  and nothing else changed, it panics at the fuse's `expect`.
- **The volume is not the oracle**, as it is for the `1e200` companion.
  `SolidView::measure` refuses the answer with `SurfaceAreaTooLargeToMeasure`,
  because the walls' area adds up past the largest double.
- **No point on the top is classified.** `classify_point` at
  `(0.75, 0.25, 1e308)` says `CannotFollowAFacesTrimming` on the top face. That
  is `holds.rs`, not `select.rs`. A single tower `1e308` tall, not fused at all,
  says the same at `(0.5, 0.5, 1e308)`. Its top face's `face_holds_the_point` is
  `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`, which for a flat face comes
  from `a_point_off_the_plane`. That function's step starts at the corner's own
  magnitude, and `1e308 + 1e308` overflows on every doubling. Its doc names this
  gap already, but it also says "Nothing a builder here makes reaches it —
  every constructor refuses a model whose dynamic range is that wide". That is
  false: `ProfileBuilder` and `Model::add_extrusion` build this tower. Both
  findings come from a throwaway test that was taken out again.

**The doc on `InPlane::of`** named the old test as the place where the second
reach runs, and said that with only the first try the fuse stops before any
arithmetic. That is still true, and it was measured again: with the second
reach removed, the new test fails with one record, "a point off a face's plane",
on `Exact([0, 0, 1])`. The doc now names the new test and says the fuse there
answers.

Two unit tests in `select.rs` pin `midway_between`.
`the_middle_of_two_corners_past_half_the_largest_double_is_exact` fails when the
function only adds first, as it did before. The other,
`the_middle_of_two_subnormal_corners_is_exact`, fails when it only halves first.
Both results were measured by hand-mutating the function.

`mise exec -- just t -p rcad-topo --no-fail-fast` on `74bb8619`: 1145 tests
run, 1145 passed, 11 skipped.

### 353.5 What this leaves

- **Criterion 7's refusing half has no demonstration.** §17's exit table names
  `a_fuse_that_needs_a_decision_nothing_can_prove_refuses` as that half. It has
  been a gap three times: `InPlane::of` at `1e200` (§19.5), `orient3d` at
  `1e308` (§324.6), and now this. The table is in `017-the-planar-fuse.md`,
  which is outside this lane's write set, so it still names the old test. This
  lane did not choose a replacement. The certified refusals it looked at, such
  as `a_block_edge_grazing_a_ball.rs` at `h = 16`, are themselves recorded as
  gaps of the fuse (§346.3). A refusal worth having as a demonstration is one
  where the answer does not exist (§15.8). Finding one is a row of its own.

  *2026-10-09, Q1440: it has none again, and this time not because the
  refusal was a gap.* Q490 (`48d66021`) found one and pointed §17's table at
  it on 2026-10-05:
  `what_two_turned_cubes_a_rounding_deep_in_each_other_share_is_too_thin_to_hold_and_is_refused`
  in `fusing_solids.rs`, two turned cubes sharing a slab about `2⁻⁵²` thick,
  refused `Undecidable` in selection. Q1353 (§417.3) taught selection to
  place that slab, and the certified intersection is now refused
  `Invalid::DynamicRangeTooWide` with a ratio past `1e16`, as the test
  asserts and passes on `7fddc415`. **That refusal is a limit.** The test
  itself shows that however an answer places its corners inside their boxes,
  its largest coordinate over its shortest edge is past `7.9e14`, beyond
  §3.5's `1e11`, so no `Model` holds the answer. **But it is not criterion
  7's refusal.** Criterion 7's refusing half is `Policy::max_wobble` judged
  against `ProvenanceLog::worst_wobble` and refusing, and `fused_under` in
  `fuse/mod.rs` asks that first, returning `Error::Undecidable`, and asks
  `check_dynamic_range` only after it. A fuse refused `DynamicRangeTooWide`
  has passed criterion 7's judgment; what it demonstrates is §3.5's range
  check. §17's amendment said that if selection ever placed the strip the
  test would fail "so this half is looked at again"; Q1353 repinned it to
  the new refusal instead, so it passes, and §17's table and the test's doc
  comment ("Exit criterion 7, the refusing half") still name it. Both are
  outside this lane's write set. Finding a certified fuse whose answer does
  not exist and which is refused `Undecidable`, or writing in §17 that none
  is known, is a row of its own again.
- **`holds.rs`'s `a_point_off_the_plane` is reachable** through the public API
  (353.4). The fix would be the same as `InPlane::of`'s second reach: step the
  same distance towards the origin. Its doc's "nothing a builder here makes
  reaches it" needs to go at the same time.
- **`middle_of_the_corners` adds before it divides too.** It is only used for a
  curved piece, and four corners past about `4.5e307` would overflow it. Nobody
  checked whether a curved piece that far out can reach it, so this is a
  question and not a finding.
