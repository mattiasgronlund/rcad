## 191. What the KiCad findings say about our generators: a STEP round trip, turned curved families, and solids that touch — 2026-09-21

*Three test-only lanes, dispatched by §175's skill off `8052d105` (green by a
local gate recorded at that sha). Merged onto `f6820853` as `e115b2ed`:
`lane-191-step-rounding` (`7e60bea0`), `lane-191-turned-curves` (`380e2bbd`)
and `lane-191-touching` (`fcb2acfd`), plus two commits of mine. `just gate
quiet` run here on `f0c445ad`: PASS, 2201 run, 2201 passed, 6 skipped. A
number below is a lane's unless it names a command run here. They found five
wrong answers between them; one is fixed here, four are parked behind an
`#[ignore]` naming this section.*

  191.1   Why: every KiCad finding sat where the generators steer away from
  191.2   The STEP writer wrote every placed solid unplaced — fixed
  191.3   Rounded text: a turned pair read back from 13 digits cannot be baked
  191.4   Turned curved families, and a bar leaning through a block
  191.5   Solids that touch
  191.6   What is left

### 191.1 Why: every KiCad finding sat where the generators steer away from

The three corpus findings — the LED dome (§124, §128), the pin-header corner
(§130–§136) and the arc on a flat face (§176) — each needed something no
generator makes. `a_curved_pair` and `a_ball_and_a_block` keep "a whole unit of
room … so no drawn pair is tangent to anything"; every curved family stands on
`z`; every family builds from whole numbers or one clean rotation, where an
exporter writes a few ulps off. Real parts are tangent everywhere (every fillet)
and rounded everywhere (every file). So this batch built three families aimed at
those three habits, each beside the control it differs from by one thing
(§17.7).

The measurement that set the rounding: the KiCad corpus, 323 files and 1 629 759
reals, is written almost entirely by Open CASCADE (FreeCAD 130, kicad StepUp 127,
OCC 7.x 37, KiCad 27). The most significant digits any real in a file carries is
13 in 251 files and 14 in 72; of all reals, 11 digits 1.96%, 12 5.88%, 13 9.71%.
So the round trip rounds to 11, 12 and 13, with 17 as the exact control.

### 191.2 The STEP writer wrote every placed solid unplaced — fixed

`rcad_step::write` took each solid's located `ShellView` off `solid.shells()`,
passed down only its key, and looked it up again with `Model::shell`, which hands
the shell back un-located. Every solid built with a placement — `place_solid`,
the lift `add_extrusion` takes, the shift `add_round_bar` takes — went out where
it was built, not where it was put. A cube moved to centre `(6,1,1)` read back at
`(1,1,1)`. It corrupted rather than refused, and
`a_solid_written_out_and_read_back_is_the_same_solid.rs` never placed a solid, so
nothing saw it.

`71ff51ac` passes the `ShellView` itself. The only other key lookup,
`model.edge(key).is_closed()`, does not depend on placement.
`a_cube_moved_along_x_is_written_where_it_was_moved_to` fails without the fix
and passes with it; `just t -p rcad-step` here: 127 run, 127 passed.

### 191.3 Rounded text: a turned pair read back from 13 digits cannot be baked

`crates/rcad-step/tests/a_fuse_read_back_from_rounded_text.rs`. Each family is
written, every REAL is rewritten to N significant digits, and the file is read
back and fused under `Policy::certified()` against the exact read-back. The
volume bound is derived from the rounding, not tuned; the worst move used
3.59e-7 of it (skew, 13 digits).

- **Turned pair: 109 of 109 decided fuses become refusals at 11, 12 and 13
  digits**, all `Undecidable` at "where three surfaces meet". A **gap**, shown
  by `a_turned_cube_read_back_from_thirteen_digits_cannot_be_united_with_nothing`:
  one turned cube read back passes `Model::validate`, and uniting it with
  nothing — whose answer is the cube, so it exists — is refused. That is §75.6's
  forbidden direction, validate accepting what the fuse cannot bake, and the
  §130–§136 class exactly: after rounding, a corner is no longer where its three
  rounded planes meet. The test passes today and holds the refusal in place.
- Skew pair: nothing introduced at 11 or 12 digits, 77 of 119 at 13.
  Unclassified, and why fewer digits refuse less is not looked into.
- Lattice and bar-through-block: rounding moves no real (every number is whole),
  so those rows are controls.
- Ball and frustum: the writer refuses every one (`AWireThatStandsStillAtAPole`,
  `ACurveTheFacesEitherSideOfItDoNotName`), so those rows compare nothing and
  assert that.

**2026-10-03, Q727:** the pin this section named is renamed. Q705 (`9615681b`)
turned `a_turned_cube_read_back_from_thirteen_digits_cannot_be_united_with_nothing`
into `a_turned_cube_read_back_from_thirteen_digits_is_united_with_nothing`; it
now asserts one solid and no longer holds the refusal (§368.3). §368.3's table
has the turned and skew families' counts: turned, 109 of 144 decided fuses
from exact text, 107, 105 and 111 at 11, 12 and 13 digits; skew, 122 of 144
from exact text, 122 at each of the three digit counts. Run here on this base,
`just t -p rcad-step --no-capture -E
'test(a_turned_pair_read_back_from_rounded_text)'` and the same for
`a_skew_pair_read_back_from_rounded_text`, still print those counts.

**2026-10-03, Q739:** since Q730 the ball-and-block family is written and read
back too, so that row no longer compares nothing. `just t -p rcad-step
--no-capture -E 'test(a_ball_and_a_block_read_back_from_rounded_text)'` prints
144 of 144 decided fuses from exact text and 144 of 144 at 11, 12 and 13
digits, with no refusal introduced at any digit count. The frustum is still
refused, as this section says.

**2026-10-03, Q829:** since Q662, landed by Q821, the frustum is written too —
all 48 configurations, no longer refused by the writer — so that row now
compares something, though not much of it. Run here,
`mise exec -- just t -p rcad-step --test a_fuse_read_back_from_rounded_text
a_frustum_up_through_a_block --no-capture`:

    as built:            144 decided, refused {}
    read back exactly:   144 decided, refused {}
    11 digits (48 of 48 files had a real moved): 0 decided, refused {}; 0 refusals the rounding introduced {}, 0 rescued; reader refused {"ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn": 48}; worst volume move 0.00e0 of the bound
    12 digits (48 of 48 files had a real moved): 0 decided, refused {}; 0 refusals the rounding introduced {}, 0 rescued; reader refused {"ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn": 48}; worst volume move 0.00e0 of the bound
    13 digits (48 of 48 files had a real moved): 21 decided, refused {"Undecidable": 3}; 3 refusals the rounding introduced {"Undecidable": 3}, 0 rescued; reader refused {"ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn": 40}; worst volume move 2.08e-3 of the bound

So the writer no longer refuses, but the reader refuses most of what it reads
back, `ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn`: all 48 at 11 and 12
digits, 40 of 48 at 13. §366.6 has the same measurement from `rcad-step`'s own
side and traces it no further.

The families bake each placement in before writing — a workaround for §191.2,
now redundant and left in.

### 191.4 Turned curved families, and a bar leaning through a block

`pairs/mod.rs` gains `Turned<P>`, which turns every solid of the bar, ball,
frustum and ball-and-bar fixtures by one shared `Turn` through the same
`turned_by` `Configuration::build` uses. It also gains `LeaningBar`, a bar
through a block on a whole-number lean, a unit clear of every wall. The
properties in `fuses_that_must_agree.rs` are run over both.

| family | refused | its upright control |
|---|---|---|
| turned bar + block | 6/429 | 0 |
| turned ball + block | 3/429 | 0 |
| turned frustum + block | 204/429 | 183/429 |
| bar leaning through a block | 298/429 | 0 (upright bar) |

Every refusal of the turned bar and ball is a **gap**: the upright answer, moved
by the same isometry, validates (bar 6/6, ball 3/3,
`every_fuse_a_turned_curved_fixture_refuses_has_an_answer_turned_from_the_upright_one`).
The leaning bar refuses 69.5%, so its laws compare little — 28 of 126
commutativity comparisons reached — and the doc comments say so rather than
count it as coverage (§14.7).

**One wrong answer fixed, one parked; both are fixed now:**

- `a_ball_and_block_united_and_then_turned_half_over_is_still_a_sound_model` —
  **fixed by Q648 (c1517c49).** A ball-and-block union that validates upright
  failed `Model::validate` (`AClosedEdgeIsWalkedAgainstItsSurface`) after
  `place_solid` by a half turn about `(-2,0,-2)`, or three quarters about
  `-y`. `Model::validate` read a closed edge's winding rule where the solid
  sits, where a ball is wound about the world's `z`, rather than where its
  faces are stored; it now reads it in the stored frame, as the mesher
  already did. The same union turned about every axis `{-1, 0, 1}³` names, by
  every sixty-fourth of a turn short of a whole one — 26 axes and 63 steps,
  1638 turns — is still sound
  (`a_ball_and_block_union_turned_about_any_of_26_axes_is_still_a_sound_model`,
  whose doc comment records that 430 of the 1638 were this refusal while
  `validate` still read the winding where the solid sits, before the fix).
- `a_bar_leaning_through_a_block_unites_with_it_into_a_solid` — **fixed by
  Q114 (`d6c64ba4`, merged as `6febd69a`), and un-ignored by Q1449
  (`8384c848`).** `LeaningBar{side: 15, radius: 1, at: [6,6], lean: [1,1],
  up: 2}`, `Union`, returned `ResultIsNotManifold` ("an edge would have 8
  faces along it"). Every crossing is transversal, so no edge of the true
  union has more than two faces. The bar's wall below the block crosses the
  planes `x = 0` and `y = 0` in two ellipses that meet, and an arc named
  without its plane put arcs of both onto one edge; an arc of a closed
  crossing is named by its plane as well since §330. The test, in
  `fuses_that_must_agree.rs`, now demands one solid that validates, of volume
  `15³ + π(34 − 15√6/2) ≈ 3424.0994` within the mesh's own error: the cube,
  and the bar's length `34` less the `15·secθ` of it the floor and the roof
  cut, `secθ = √6/2`. Over the whole family the leaning bar now refuses 63 of
  450 fuses, 14.0%
  (`how_often_a_certified_fuse_of_a_bar_leaning_through_a_block_refuses`, on
  `7fddc415`), where the table above has 298 of 429.

The suite's wall time went from 2.8 s to 5.4–6.5 s.

### 191.5 Solids that touch

`crates/rcad-topo/tests/touching/mod.rs` and `fuses_that_touch.rs`: 17 cells of
surface pair × touching position, all whole numbers. Each cell is paired with
the same configuration one unit clear, and each is run under every rule both ways
round.

| cell | touching answered | clear-by-one answered |
|---|---|---|
| bar beside a block's wall | 0/72 | 78/78 |
| bar in an inside corner (the fillet) | 0/78 | 72/72 |
| two bars touching along a line | 0/54 | 96/96 |
| bar's cap flush on a block | 0/66 | 84/84 |
| frustum's cap flush on a block | 0/84 | 66/66 |
| ball on a bar of its radius (the dome) | 0/66 | 12/84 |
| ball resting on a block | 54/72 | 78/78 |
| ball against a block's wall | 57/78 | 72/72 |
| cone's apex on a block face | 30/84 | 60/66 |
| two balls, ball in ball, ball on or in a bar | all | all |

**Every cylinder that touches a plane or another cylinder along a line is
refused**, 0 answered in five cells, and the fillet shape is one of them. These
are §39.4's known "which way a crossing runs" gap and §38.9's coincident-face
row. What is new is how much of a real part they cover. The dome refuses as §124
says, and it is the only cell with no transversal control at equal radius
(§128.3).

New gaps: a radius-2 ball resting on a block it overhangs refuses `Union` and
ball∖block with `NotFinite("point on a plane")` or `ResultIsNotManifold{count:
4}`, where the same ball one unit higher, a radius-1 ball, or a larger block all
answer (`a_ball_resting_on_a_block_it_overhangs_is_refused_though_the_answer_is_both_solids`).
A non-finite value out of whole-number input, reported as a geometry error, is
worth its own look. The cone apex's `CannotNameWhichPieceOfACrossingThisIs` has
no demonstration yet.

**The panic is fixed; both shapes answer now.** A bar standing wholly inside a
thicker bar, touching nothing, used to panic at `fuse/assemble.rs:750` ("a ring
of a joined face came back winding neither way: Zero") under `Union` both ways
round and under thick∖thin, until `d12913d4` stopped the panic and `71feed66`
made the fuse answer. `a_bar_standing_wholly_inside_a_thicker_bar_is_united_and_hollowed_without_a_panic`
now answers the union as the thick bar, `16π`, and the difference as the thick
bar with a bar-shaped hole sealed inside it, `14π`; Q368 (`fuses_that_touch.rs`)
took the ignore off that pin, and the property cell
`a_bar_inside_a_bar_touching_its_wall` runs un-ignored with it, because its
control row is that bar.

A bar standing wholly inside the wall of a tube panicked the same family's way,
in `rings_round`, with *"2 rings enclose no area to say which it is"*: the
bar's pipe crosses a cap's plane in a circle, so the joined cap's outline and
its hole are each a whole circle with one corner, and a corner that encloses no
area cannot say which circle is which. It answers since Q359
(`a_bar_inside_a_tubes_wall.rs`):
`a_bar_inside_a_tubes_wall_united_with_it_is_the_tube` answers the union as the
tube as it stood, `32π`, whichever solid goes first, and
`a_tube_less_a_bar_inside_its_wall_is_hollowed_by_the_bar` answers the tube less
the bar as the tube with a bar-shaped hole sealed inside its wall, `32π − π/2`.

The torus cells never reach their touch — the control refuses on the
plane–torus quartic as `how_often_a_fuse_with_a_torus_refuses.rs` proves — and
are pinned at nothing answered. Not built: a bar lying on its side (a quarter
turn of a round solid is not exact), a plane along a cone's ruling, cone against
anything curved.

**2026-10-05 note (Q1078): the bar's cap and the frustum's cap rows above are
stale.** Since Q1062 (`7376b2f3`) both answer every fuse: `fuses_that_touch.rs`'s
refreshed table gives the bar's cap flush on a block 66 of 66 touching and the
frustum's cap flush on a block 90 of 90 touching. The cause was
`select.rs`'s `a_face_of_the_input_holds`, which asked `SolidView::face_holds_the_point`
where its fan of corners could not wind round a point on a spoke — the point in
the middle of a flush square cap sits on both diagonals, so every apex's fan had
it on a spoke and the winding read `None`.

**2026-10-10 note (Q1588): a half ring with a bar in its hole, level with the
top of its tube, answers every fuse.** The bar, radius `23/8` about `z` from
`z = −1` to `1`, touches nothing of the ring of four round a tube of one, yet
all 24 fuses — on a plate and alone, both seams, every rule both ways round —
were refused 'a point inside a piece of a face' at `(4, 0, 1)`. What kept the
cut: the bar's top plane meets the torus only in the circle `r = 4, z = 1`, and
`cuts.rs`'s `the_cuts_left_on` drops such a crossing only where the face that
carries it, the bar's top, is proved clear of the torus. That proof is
`a_surface_misses_a_band(torus, the disk's band)`, reached through
`a_face_misses_a_surface_by_its_band`, and it answered false for every torus,
so the piece of skin past the circle, on the side of `z = 1` where the torus has
no point, reached selection. The seam plays no part: `bake` leaves it out of the
face's wires, and a trace gave the same cuts with it and without. What drops it
now: `a_torus_on_the_axis_misses_a_band` asks a torus about the band's axis as
one polynomial in `up` and `out`, written two ways, `(out + h² + (R² − r²)|w|²)² −
4R²|w|²·out` and `(out + h² − (R² + r²)|w|²)² + 4R²|w|²(h² − r²|w|²)`. Either
keeping a sign over the band proves it clear. The disk at the top of the tube,
where `h² = r²|w|²`, is proved only by the second; the bar's wall through the
tube's height only by the first. The 24 fuses now give, on the plate, the union
`72π + 4π² + 529π/64` as one solid, the intersection `529π/64`, the group less
the plate `4π²` and `529π/64` apart, and the plate less the group
`72π − 529π/64`; alone, the union is the two solids apart, the intersection
nothing, and each difference its first argument
(`a_half_ring_standing_on_a_round_plate.rs`).

### 191.6 What is left

`ignored_tests` goes 2 → 6 and `wildcard_arms` 87 → 88 (the justfile says which
arm). In order of harm:

1. **The panic** (§191.5): a solid inside a solid crashes the fuse. Crash beats
   corrupt, but this is the plainest configuration of the batch.
2. **A turned sound union fails validate** (§191.4): either the move or the
   check is wrong, and it shares its refusal with §124.
3. **A clean leaning union is called not a solid** (§191.4).
4. **The turned pair from 13 digits** (§191.3): §75.6's direction, the §136.4
   class at ordinary sizes rather than at 100 m, and every OCC-written file
   carries it. Parked §136.4 wants a witness; this is one.
5. Line tangency between a cylinder and a plane or cylinder answers nothing
   (§191.5), which is every fillet.

Each is a lane of its own that writes `src/`. None is dispatched here.

**Done (Q568): item 1 above is fixed.** §191.5 says what the panic's two
configurations answer now.
