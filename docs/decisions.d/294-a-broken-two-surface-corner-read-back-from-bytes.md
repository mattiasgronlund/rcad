## 294. A broken two-surface corner read back from bytes fuses safely, and blames itself — 2026-09-27

*Queue row Q218, from §286.4. Lane `lane-q218-broken-corner-from-bytes`, restarted
as `lane-q218-broken-corner-from-bytes-2` after the first run stopped at the
lane budget hook with the measurement done and pinned. §286.4 left open what a
fuse does with a two-surface corner moved off its surfaces in a model's bytes:
a wrong `Ok`, `TheFuseWroteABrokenAnswer`, or a panic — the answer that decides
whether §286's option (c), asking such a corner at the door with
`Model::validate`'s own room, is owed. It is not: no wrong `Ok` and no panic
came out of any move or partner tried, pinned in
`crates/rcad-topo/tests/a_broken_two_surface_corner_read_back_from_bytes.rs`.
No code changed.*

### 294.1 The corner, moved where no builder can put it

A round bar of radius three and height ten has, at the top of its seam, a
corner with only two surfaces round it: the top cap's plane and the wall's
cylinder. `add_round_bar` puts it at `(3, 0, 10)`. Nothing in the public API can
write it anywhere else — every builder validates what it built (§286) — so the
test writes the bar out with `postcard`, finds the one place its bytes hold
`(3, 0, 10)` as three little-endian `f64`s in a row (asserted unique in the
file, so the overwrite cannot land on a surface's own numbers), and overwrites
it with one of three moves, then reads the bytes back:

- **off the wall**, `(4, 0, 10)` — a millimetre out, still on the cap's plane.
- **off the cap**, `(3, 0, 11)` — a millimetre up, still on the wall's cylinder.
- **a quarter turn round and off the wall**, `(0, 4, 10)` — off the wall only;
  it still sits on the cap's own plane, `z = 10`.

`Deserialize` does not call `validate`, so all three read back. `Model::validate`
under `Policy::certified` refuses each: off the wall and the quarter turn as
`Invalid::VertexOffItsFaceSurface`, off the cap as
`Invalid::VertexOffItsFacePlane { distance: 1.0 }`. The same bytes with the
corner left alone validate. All four checked in
`a_bar_whose_top_corner_is_moved_a_millimetre_in_its_file_reads_back_and_does_not_validate`,
which passed re-run here (`mise exec -- just t -E
'binary(a_broken_two_surface_corner_read_back_from_bytes)'`, 4 tests run, 4
passed, 0 skipped — the whole file, including this one).

The first run's probe (a temporary `eprintln` behind `RCAD_Q218_PROBE` in
`place_a_corner`, since removed) printed, for the moved corner in each of its
first twelve fuses, the two surfaces `place_a_corner` hands on unasked at the
moved coordinates — confirming every fuse below reaches `place_a_corner` for
the moved corner rather than, say, one built somewhere else along the way.

### 294.2 Thirty fuses, three outcomes, no wrong `Ok` and no panic

The file fuses the broken bar against ten `(partner, rule)` pairs, each move
against each pair — three moves × ten pairs, thirty fuse calls with a moved
corner, tallied by reading the file's loops (`EVERY_MOVE` crossed with the
pairs in each of the three fusing tests). All thirty passed as one of three
kinds, and none came back a wrong `Ok` or a panic — a kernel panic during a
`#[test]` aborts the test process rather than letting nextest report a pass, so
the 4-of-4 passed re-run above is itself the evidence for "no panic" across all
thirty:

1. **An answer that keeps the moved corner** (`FAR_AWAY` ∪, `OVER_THE_BOTTOM`
   −, six calls: three moves × two pairs) comes back
   `Err(TheFuseWroteABrokenAnswer { broken })`, where `broken` is exactly the
   `Invalid` `validate` gave the unfused bar (asserted equal in
   `a_fuse_that_keeps_the_moved_corner_blames_itself_for_the_broken_file`). The
   model is byte-identical before and after (asserted against a `postcard`
   snapshot taken before the call) — the fuse's "Atomic" promise held.
2. **A partner that cuts the moved corner away** (`OVER_THE_TOP`,
   `OVER_THE_RIM_BY_THE_SEAM` × {−, ∪, ∩}, `OVER_THE_RIM_AWAY_FROM_THE_SEAM` −,
   `A_NOTCH_A_QUARTER_TURN_ROUND` − — those six pairs × all three moves, eighteen
   calls — plus the quarter-turn move against `THE_HALF_PAST_THE_SEAM` × {−, ∩},
   two more: twenty calls) comes back `Ok` with the
   sound bar's answer: the same corners as a set (`corners` sorted and
   deduplicated bit patterns), the same volume within `1e-12` relative, and
   `validate_solid` accepts the answer on its own.
3. **A partner whose face runs through where the corner should be**
   (`THE_HALF_PAST_THE_SEAM`, whose face lies along the seam, × {−, ∩} against
   the wall- and cap-moved corners: four calls) refuses by name of the
   arrangement, not the file — see 294.3.

### 294.3 A face along the seam meets the moved corner head-on

`THE_HALF_PAST_THE_SEAM`'s face runs along the seam and crosses the top circle
at `(3, 0, 10)`, exactly where the unmoved corner sits and where the fuse
places a corner of its own regardless of what the bar's bytes say. Three
distinct things happen, none of them the wrong `Ok` a bug would produce and
none of them a panic:

- **Off the wall**, the moved corner sits inside the block's face as well as on
  the bar's, and the two regions of that face cannot be told apart:
  `Err(CannotTellTheCellsOfAFaceApart { .. })`. `error.rs` calls this a gap —
  "the regions are there and telling them apart takes a point inside each one,
  which is a construction this arrangement does not make" — naming the
  arrangement, not the caller's file.
- **Off the cap**, the moved corner sits on the wall above the new corner the
  fuse places at the seam, and ordering the two along the circle's crossing is
  `Err(Undecidable { records })`, whose one record names the predicate `"order
  along a crossing"`. `Undecidable` at the operation level is a failure by
  design (CLAUDE.md); nothing here widens it, because the two points really do
  need distinguishing and neither is closer than roundoff to the other.
- **A quarter turn round**, the moved corner is inside the half the block
  keeps, clear of the seam, and the answer is the sound bar's `Ok` (case 2
  above, folded into the count there).

Both checked in
`a_cut_along_the_seam_refuses_a_corner_moved_in_the_cut_and_answers_one_moved_out_of_it`.

### 294.4 Two doc comments describe a different failure than this one

Two places in the source name what a broken input does to a fuse, and both
describe something other than what 294.2 and 294.3 measured:

- **`Model::fuse`'s `# Panics`**, `crates/rcad-topo/src/fuse/mod.rs:196-199`:
  "An input broken in a way `Model::validate` rejects is not checked, in any
  build... What a broken input does is trip an assert somewhere further in,
  under whatever name that site gives it." Measured here, it never trips an
  assert. Every one of the thirty calls returned `Err` or a sound `Ok`.
- **`TheFuseWroteABrokenAnswer`'s doc**, `crates/rcad-topo/src/error.rs:186`:
  "This is a bug here, not bad input." (The brief that started this write-up
  cited `error.rs:205-211` for this sentence; that range is the paragraph
  about `Invalid::DynamicRangeTooWide` never arriving wrapped in this variant,
  two paragraphs further down — the sentence itself is at line 186, and this
  is corrected here rather than repeated.) `Model::fuse`'s own `# Errors`
  entry for the same variant, `mod.rs:171-172`, says the same thing: "That is
  a bug here rather than bad input." Measured in 294.2's first kind, the
  `Invalid` `TheFuseWroteABrokenAnswer` carries is the caller's own broken
  file, carried through unchanged — `validate` said the same thing about the
  bar before any fuse ran. The refusal is safe (the model is untouched), but
  it is misattributed: it blames the fuse for the caller's file.

Fixing these two comments to say what was measured, or building option (c) if
294.5 makes it cheap, is left to a lane with `fuse/mod.rs` and `error.rs` in
its write set (this file's write set has neither).

### 294.5 Whether option (c) is owed

It is not, as a correctness fix. Every `Ok` measured in 294.2 and 294.3 is the
sound bar's answer: a face is its surface (§17.1), so a corner cut away by the
fuse was never more than a cached sample of a surface that is still there, and
an `Ok` that keeps the moved corner never happened — every case that kept it
came back `Err(TheFuseWroteABrokenAnswer)` instead. What (c) would buy is not a
safety fix but attribution: turning that refusal from a claim the fuse is
broken into a claim the input is. It would also have to turn 294.2's second
kind — twenty right `Ok`s, including the two folded in from 294.3's third
case — into refusals, since (c) asks the same two surfaces about the same stored
corner before the fuse can tell whether the answer will keep it or cut it
away — the corner has not been used yet at the door, so the question at the
door cannot be conditioned on where the partner's faces fall. That is the same
shape §286.1 already worked out for a corner that is merely close, not broken:
asking narrower than what `validate` allows refuses solids `validate` accepts.

### 294.6 The largest coordinate `validate`'s room is sized from is not cached

Asked separately from the first run's report: whether `Model::validate`'s
`allowed_wobble` and `not_provably_off`, which size their room from "the
largest coordinate of the model," read a value `Model` keeps up to date, or
scan for it each call.

**It scans, every call, over every vertex in scope.** `Model`'s own fields
(`crates/rcad-topo/src/model.rs:32-38`: `id`, `which_model_this_is`, `data`,
`index`, `worst_approximation`) carry no such cache. The value comes from
`Model::largest_coordinate_in` (`crates/rcad-topo/src/validate.rs:1449-1459`):

```rust
fn largest_coordinate_in(&self, scope: &Scope) -> f64 {
    scope
        .vertices
        .keys()
        .map(|slot| self.data().vertices[slot].point.largest_coordinate_magnitude())
        .fold(0.0, f64::max)
}
```

a fold over every vertex `scope` names, called fresh from `allowed_wobble` and
`not_provably_off`'s call sites (`validate.rs:345`, `:634`) and again from
`check_dynamic_range` (`validate.rs:1237`) and the test helper at
`validate.rs:3268-3269`. So option (c), asking at the door, would cost one
such scan of the model's own vertices per corner it grows and asks about — not
free, but not the model-wide validity sweep either: `place_a_corner` already
walks the model's vertices to place this one corner, and this would be one
more fold alongside it, not a second `Model::validate`. Whether that cost is
cheap enough to build (c) for is a call for whoever picks up 294.4's first
item; this file measured only whether it is cached (no) and did not measure
the fold's cost against a fuse's other work.

### 294.7 Not tried

- **A three-or-more-surface corner moved the same way**, which §286.4 already
  named as a separate, unmeasured gap (`Undecidable` from
  `the_place_the_model_points_at`'s narrower rounding, not this file's
  question).

The three items this section used to list beside that one — a non-identity
placement, balls and cones, and a tolerant policy — are now tried; §294.8
records what Q224 found.

### 294.8 Q224: a placement, a tolerant policy, a ball, a frustum

Q224 tried three more families, and Q419 pinned what it found in the same
file. Every case comes out one of 294.2's three ways, or as a refusal that
names something other than the file — never a wrong `Ok`, never a panic.

- **The bar at a placement** — shifted by `(7, -5, 2.5)`, turned 0.4 about
  `x`, and turned then shifted, the block placed the same way. The corner is
  moved in the stored frame, and `validate` refuses it just as it refuses the
  bar at no placement. Every fuse gives what the bar at no placement gives,
  with two differences. Turned and shifted, the fuse's refusal of the corner
  moved off the cap carries `VertexOffItsFacePlane { distance:
  0.999_999_999_999_998_2 }` where `validate`'s own refusal carries `1.0`
  (`a_turned_and_shifted_bar_blames_itself_for_a_corner_off_the_cap_with_the_distance_rounded`).
  And turned, the sound bar cut along its own seam is `Undecidable` on "side
  of plane" — the box round where its bottom seam corner `(3, 0, 0)` lands is
  `8.5e-14` either way turned, `2.8e-13` turned and shifted — so every corner
  moved there is that refusal too, not an answer about the moved corner
  (`a_turned_sound_bar_cut_along_its_seam_cannot_tell_which_side_of_the_plane_its_corner_is`).
  Turned alone, it answers since §294.9; turned and shifted, it still refuses.
  Only shifted, the sound bar answers, because a corner moved by a shift that
  rounds nothing is handed on exact (Q509 fixed this; before it, it refused
  "side of plane" there as well).
- **A tolerant policy**, `Policy::tolerant(100.0)`, on the bar at no
  placement, matches `certified` in every fuse except the corner moved off the
  cap in a cut along the seam, which refuses with
  `TwoCurvesTouchWithoutCrossing` where `certified` finds the order along a
  crossing `Undecidable`
  (`under_a_tolerant_policy_a_corner_moved_off_the_cap_in_a_cut_along_the_seam_touches_without_crossing`).
- **A ball's south pole** `(0, 0, -3)`, moved to five places. `validate`
  refuses four with `VertexOffItsFaceSurface`, and the one moved onto the
  equator at `(3, 0, 0)` with `APoleStandsWhereTheSurfaceDoesNotPinch`
  (`a_ball_whose_south_pole_is_moved_in_its_file_reads_back_and_does_not_validate`).
  No fuse blames itself. In six pairs, a block far away among them, the answer
  is the sound ball's corner for corner to the last bit, so the moved pole
  never reaches it; the seventh pair, a small block round the pole, refuses
  with `ACurvedFaceGoesRightRoundWithNoSeam` for the sound ball too, under
  both policies
  (`a_ball_whose_south_pole_is_moved_fuses_to_the_sound_balls_answer`).
- **A frustum's top corner** `(3, 0, 6)`, from `cones::a_frustum(model, 0.5,
  2, 6)`, moved off the wall, off the cap and a quarter turn round: the bar's
  outcomes, and one refusal more. Moved off the cap to `(3.5, 0, 7)`, the
  corner sits inside a block over the top (`z` from 5 to 8), and both rules
  tried with that block were `Undecidable` on "a point inside a piece of a
  face". The sound frustum was `Undecidable` on that same predicate against a
  block over the rim by its seam, and so was every moved corner there, under
  both policies
  (`a_frustum_whose_top_corner_is_moved_in_its_file_reads_back_and_does_not_validate`,
  `a_frustum_whose_top_corner_is_moved_gives_what_the_bar_gives`).
  Since Q712 each of those answers or blames itself, the block over the rim
  giving the sound frustum's answer for every corner, because a piece no
  point was found in is placed by a side it shares (§365.3). **Renamed by
  Q1103**: the extra refusal Q712 fixed was the one thing the name's "one
  refusal more" meant: the table still gives `CannotTellTheCellsOfAFaceApart`
  past the seam, the same way the bar's own seam cut does
  (`a_cut_along_the_seam_refuses_a_corner_moved_in_the_cut_and_answers_one_moved_out_of_it`),
  so that was never "more" than the bar and the old name's claim was false.

  **By its seam is not why the block over the rim refused** (Q929). That
  block's wall at `x = 2` cuts the cone below the block's floor too, down to
  `(2, 0, 4)`, and leaves a piece of the cone between that cut and the circle
  at `z = 5`, with corners `(2, ±1.5, 5)` and nowhere more than one unit deep.
  Every step `select.rs`'s `candidate_points` took across a curved piece was
  one radius long, `2.5` there, and each landed on that circle or outside the
  piece. The seam turned to `−x` or `+y`, and the block over the far rim at
  `x = −2`, leave the same piece, with no point found inside it either. The
  block over the rim away from the seam stops at `x = −1`, whose cut reaches
  down to the bottom rim at `(−1, 0, 2)`, and a point the fuse offered was
  inside every piece that cut left. A side the piece shares placed it against
  one block, but not against two solids together: the frustum fused with that
  block and a cube far away as its tools still refused under every rule.
  Since Q929 the fuse
  takes the same steps at a half, a quarter, an eighth and a sixteenth of the
  length after every other point, and half the step towards the side of both
  planes the piece is on comes back on the cone at `(2.06, 0, 4.12)`, inside
  it. Each fuse of the frustum, the block and a second block, far away, over
  the far rim, or the rest of the top, answers and measures its closed form
  (`tests/a_frustums_wall_cut_below_a_blocks_floor.rs`).

### 294.9 Q912: a seam corner is on its moved seam plane by construction

*Queue row Q912, from Q717. Lane `lane-q912-a-seam-corner-on-its-moved-seam-plane`.*

**The case.** The round bar of 294.1, turned 0.4 about `x`, less or
intersected with `THE_HALF_PAST_THE_SEAM` turned the same way, refused
`Undecidable` on "side of plane". Measured again on the base with Q717's probe
(`./target/q717/run.sh none`): both rules refused at the box the turn hands on
round the bottom seam corner, `(3, 0, 0)` give or take `8.526512829121202e-14`
on every axis, asked about the block's face, normal `(0, -0.9210609940028851,
-0.38941834230865047)` and offset `0`. The arrangement asks it while placing
the corners of the bar's bottom cap: the cap's crossing with the block's face
is a line, and whether that line runs through the corner is what
`passes_through` asks of the corner's box. The face straddles the box, so the
box cannot say.

**The answer exists before any rounding.** As stored, the corner `(3, 0, 0)` is
exactly on the plane `y = 0` through the bar's axis, and the turn keeps every
incidence. The block's face is that same stored plane moved by the same
`Plane::transformed`, so it is the same `f64`s: measured, the plane through the
bar's stored axis and its stored corner, turned, is the block's face turned
round, bit for bit (a scratch probe, not kept). So the line the face cuts the
cap in meets the bottom circle at the corner, and not at a second point a
rounding from it.

**The rule.** `bake` now gives a corner read out of the model on a pipe's wall
one more thing beside `around`: `BakedCorner::seam_plane`, the plane through
the pipe's stored axis and the stored corner, moved by the placement with
`Plane::transformed` — only when the stored corner is exactly on it, asked of
the stored numbers. The plane's normal is the axis crossed with the corner,
scaled to length one, which rounds unless the pipe stands along an axis, and a
plane that misses the stored corner by a rounding crosses the circle a rounding
from it, which is another point. `place_the_corners` takes a crossing whose
partner is that plane, either way round, as through the corner without asking,
as it already took the surfaces round a corner
(`BakedCorner::is_on_by_construction`).

It is sound for the same reason the surfaces round the corner are taken
unasked: moved, the corner is no more exactly on its own cap and wall than on
the seam plane, and each is a moved copy of an incidence the model held. And a
corner with two surfaces round it is a cached sample of the curve they share
(§17.1, §31.3): the cap and the wall are bounded by the whole circle and the
seam is no boundary (`bake.rs`'s header), so taking the corner as where the
plane crosses the circle moves no point of the solid. The plane holds the axis,
so its other crossing with the circle is a diameter away.

**Kept out of `around`, on purpose.** `around` promises that a corner with three
or more surfaces in it was asked about every one, and
`bake_placed_the_corner_on_the_three_planes` reads `around.len() >= 3` as that
promise and takes three planes from it as planes the corner was solved among
(§356). A bar's corner with its seam plane in `around` would have three
surfaces, two of them planes, none asked, and §395.7 has already shown what a
fold over three planes that do not pin a point does. A field of its own keeps
that promise as it was; Q1036 can put its own field beside it.

**What it gives.** Turned alone, both rules answer, and each answer is half the
bar, `45π`, measured with a fine mesh, validates on its own, and has the
unturned answer's corners turned, each within `1e-12` of exactly one of them —
for `THE_HALF_PAST_THE_SEAM` and for `THE_HALF_SHORT_OF_THE_SEAM`, whose face
runs along the seam the other way round, so each way round of the comparison
is what answers one of the two
(`a_turned_bar_cut_along_its_seam_gives_what_the_unturned_bar_gives_turned`).
The top corner moved in the file gives the unturned bar's outcomes: off the wall
`CannotTellTheCellsOfAFaceApart`, off the cap `Undecidable` on "order along a
crossing", and a quarter turn round the sound answer — to a rounding, not to
the bit. Cut away, that corner leaves the top seam corner of the answer to be
placed where the block's face crosses the top circle, two ulps in `y` and one
in `z` from where the sound bar's own corner is written, so the same-bits check
of 294.2's second kind does not hold there and the test compares within
`1e-12` instead.

**Turned and then shifted, it still refuses** "side of plane", at the bottom
corner's box `2.8421709430404007e-13` either way round `(10, -5, 2.5)`
(`a_turned_sound_bar_cut_along_its_seam_cannot_tell_which_side_of_the_plane_its_corner_is`,
now the turned-and-shifted bar alone). There the block's face is not the seam
plane moved: its offset is `0x400d0dd7b8f4fb64` and the reversed moved seam
plane's `0x400d0dd7b8f4fb65`, one ulp apart, because the block's placement is
composed with its own shift and rounds differently. That is a different plane,
crossing the circle `3.9e-16` from the corner (Q717's figure), and the rule
does not reach it. Whether that is a limit is Q913's to pin.

**Not tried.** A cone's seam corner: a frustum turned and cut by
`THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM` turned the same way is the same shape
on a cone, and `the_seam_plane_a_corner_is_on` looks only at pipes, so it would
still ask. *Tried by Q1044 (§294.10): it never gets that far. The fuse refuses
earlier, where the block's face crosses the cone, so nothing is asked about the
corner. Since Q1164 it looks at cones too (§294.12).*

### 294.10 Q1044: the turned frustum is refused where the cut runs through its apex — 2026-10-06

*Queue row Q1044, from §294.9's "Not tried". Lane
`lane-q1044-a-turned-frustum-cut-at-its-seam`, base `567e65ee`. No code
changed.*

**The case.** `cones::a_frustum(model, 0.5, 2, 6)`, turned 0.4 about `x`,
less or intersected with `THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM` turned the
same way. This is the bar of §294.9 on a cone: the block's face runs along the
seam, through the top seam corner `(3, 0, 6)` and the bottom one `(1, 0, 2)`.
The row asked whether it refuses `side of plane` at a seam corner's box the way
the bar did before §294.9, and if so, to extend
`the_seam_plane_a_corner_is_on` from pipes to cones.

**It does not refuse there.** Both rules refuse with
`Error::Geometry(CannotNameWhichPieceOfACrossingThisIs)`. So does every moved
corner (off the wall, off the cap, a quarter turn round), and so does all of it
under `Policy::tolerant(100.0)` as well as `certified`
(`a_turned_frustum_cut_along_its_seam_is_refused_where_the_cut_runs_through_its_apex`).

**Where it is raised: `rcad_geom::intersect`, on the frustum's cone and the
block's face along the seam.** Turned, the cone's apex is still the origin and
its axis direction is `(0, -0.38941834230865047, 0.9210609940028851)`; the face
has normal `(0, -0.9210609940028851, -0.38941834230865047)` and offset `0`. The
apex is exactly on the face, and the face is steeper than the cone, so
`where_a_plane_and_a_cone_cross` takes the row of its table where the two cross
in two rulings through the apex and calls `two_lines_through_a_cones_apex`
(`crates/rcad-geom/src/intersect.rs`). That function works out each ruling's
direction in `f64` and then asks, exactly, whether the line through the apex
along it is on both surfaces. It refuses when it is not, at its first
`return Err` (intersect.rs:2993 on the base). The first two facts are pinned
(`the_face_along_a_turned_frustums_seam_crosses_its_cone_in_two_rulings_nothing_names`).
The third was measured with a scratch probe, not kept, which did the same `f64`
arithmetic and asked the public `Plane::side_of_point` and the cone's
`SideOfSurface::side_of`. Turned, both directions come out
`(±0.5, -0.38941834230865047, 0.9210609940028851)`, and each is on the face,
`Zero`, and inside the cone, `Neg`. Unturned they are `(±0.5, 0, 1)`, `Zero` on
both, and the crossing is named.

Why inside: the direction is `±0.5` across and then the axis direction's own
two numbers. Put that into the cone's expression and what is left is a quarter
of `q·(1 − q)`, where `q` is the axis direction's squared length, worked out
exactly. So it is on the cone exactly when `q` is exactly one, and `Neg` says
the turned axis direction is a hair longer than one. That is algebra on the
measured numbers, not a second measurement.

**It is a gap, not a limit (§16.10).** The answer exists. Unturned, both rules
give half the frustum, `26π/3`, and the answer validates and measures that with
a fine mesh under both policies (pinned in the same test). A turn moves no
volume, so the turned answer is that answer turned. `two_lines_through_a_cones_apex`'s
own doc already calls its refusal a gap: the two lines are there and can be
told apart, and what runs out is `f64`'s supply of exact ruling directions.

**`the_seam_plane_a_corner_is_on` is left as it is.** There is no case yet that
reaches a cone's seam corner to ask about it, so extending it would build a
part nothing uses (§13), and nothing could show it right. Whether the turned
frustum would refuse `side of plane` at a seam corner's box once `intersect`
names its two rulings is not known. If it does, the cone's argument is §294.9's
with one change: a cone's seam is a ruling, the line through the apex and the
corner, and not a line parallel to the axis. The plane through the axis and the
corner is built the same way, from the axis origin, which on a cone is the apex.
It crosses the cone in two rulings that meet only at the apex, and a cap square
to the axis in two points a diameter apart, as on a pipe. *It did refuse there,
once Q1171 named the rulings (§294.11), and Q1164 extended the seam plane to
cones on this argument (§294.12).*

**What this leaves.**

- The gap in `two_lines_through_a_cones_apex`: a cone whose axis direction is
  not exactly of length one, cut by a plane through its apex steeper than the
  cone, gets no name for the two rulings. A row of its own, tier 1.
- Q1044's question asked again after that row: whether the turned frustum then
  refuses `side of plane` at a seam corner's box, and if so, the seam plane
  extended to cones. A row that waits on the first.
- Not measured: whether a cone built along a slanted axis at no placement, cut
  by a plane through its axis, meets the same refusal. The algebra above is
  for this turn's numbers only, and nothing has run the other case.

### 294.11 Q1170: the turned frustum's two rulings cannot be named whole, and their four halves can — 2026-10-06

*Queue row Q1170, Q1163 filed again with the nappe tallies added. Lane
`lane-q1170-two-rulings-through-a-turned-cones-apex`, base `e4ea60b8`. No line
that runs changed: a new test file and a doc comment.*

**What refused.** §294.10's pair: the frustum's cone turned 0.4 about `x`, and
the block's face along the seam turned the same way. `intersect` refuses
`CannotNameWhichPieceOfACrossingThisIs` at the first check of
`two_lines_through_a_cones_apex`, and unturned it names two rulings. Both are
pinned again on the two surfaces alone, with no fuse round them
(`turned_the_two_rulings_through_the_apex_are_refused_and_unturned_they_are_named`
in `crates/rcad-geom/tests/a_turned_cone_cut_through_its_axis.rs`).

**No `f64` direction lies on both surfaces, because the true one is
irrational.** Turned, the axis direction is `w = (0, −b, a)` and the face's
normal is `n = (0, −a, −b)`, with `a = 0.9210609940028851` and
`b = 0.38941834230865047`. `n` is `x̂ × w` bit for bit, so the face holds `x̂`
and `w`, and a direction `c·x̂ + w` in it is on the cone of slope `½` exactly
when `c² = q/4`, for `q = a² + b²` worked out exactly. Written as a whole number
times a power of four, `q`'s whole number is not a square (checked in `u128`),
so `√q` is irrational and the two rulings run along `(±½√q, −b, a)`. No three
fractions write that direction, at any scale. That is the measured fact, and
three families of fix fall to it on this pair
(`the_turned_rulings_run_along_directions_no_fraction_writes`):

1. **Working the direction out in another order or at another scale.** Every
   result is three `f64`s, so three fractions. The test asks the plane and the
   cone, exactly, about the function's own `(±0.5, −b, a)`, about `½√q` rounded
   and its four neighbours either way, each at three scales and both signs:
   every one is on the plane and off the cone.
2. **A pair of opposite half-spaces whose flat surface holds the ruling**, which
   is what the function builds. A stored normal `p` square to `(±½√q, −b, a)`
   has `p_x·(±½√q) + p·w = 0`, a fraction times an irrational plus a fraction,
   so `p_x = 0` and `p·w = 0`, and then `p` is along `x̂ × w`, which is `n`. The
   only flat surface through the apex that holds one ruling is the face itself,
   and it holds the other too.
3. **Any other picker.** One that holds a whole endless line and only the apex
   of a second cannot be *outside a ball* or *outside a pipe*: the second line,
   apex left out, would have to lie strictly inside, which a ball never holds,
   and which inside a pipe forces the line to run along the axis and puts the
   apex strictly inside as well. *Inside a pipe* needs its axis along the first
   line, which is irrational again.

So no set of today's pickers names a whole ruling of this pair. In general it is
the pairs where `−E·‖w‖²` is not the square of a fraction; here that comes down
to `q`, which is why §294.10 found the unturned axis, of length exactly one,
named.

**What does tell them apart: the row's candidate, changed.** Leg (a) holds: the
rulings are mirror images across the flat surface through the apex holding the
axis and `n`, whose normal is `n × w`, and they are strictly on its two sides.
The row's worry, that a face holding the axis would mirror them across `n`
instead, does not happen: `n × w` is not nothing whenever the face is steeper
than the cone, and this face holds the axis and is split by it all the same.
**But a ruling is endless both ways and crosses that surface at the apex**, so
a half-space bounded by it holds the forward half of one ruling and the backward
half of the other, not one ruling. The change: add the flat surface through the
apex square to the axis, facing `w`, which cuts each ruling at the apex into the
half in each nappe. Then the four halves lie in four corners of the two surfaces,
strictly, and neither surface needs an exact direction
(`the_turned_rulings_four_halves_lie_in_four_corners_of_two_flat_surfaces_through_the_apex`,
which stands each half for by a box between two exact points of the face that
the cone reads inside and outside). Leg (b): both surfaces must pass exactly
through the apex, and here they do, since the apex is the origin and both
offsets are `0`. Elsewhere each offset is one dot product, and where it rounds
the halves stay refused, as item 3 of the function's doc already refuses.

**Leg (c) is what keeps this from being the fix.** `rcad-topo`'s
`a_ruling_through_an_apex` (`fuse/arrange.rs`) reads a ruling's direction as
`n × p` for its first picker `p`, and `ThroughAnApex::along` carries it into the
order along the crossing and the turns at its nodes. That is right only for a
picker whose flat surface holds the ruling. For a corner's two pickers, `n × p`
runs along the axis or along `n × w`, and neither is a ruling. Naming halves in
`intersect` alone would hand the arrangement a direction that is not the
line's. Nothing has run that, so whether it would answer wrong or refuse is not
known; and `arrange.rs` is outside this row's files. `walk.rs` says it walks a
ruling by its ends and reads no direction off a picker. So the fix is a row of
its own with both files: name the four halves where the direction is
irrational, and let the arrangement take a half's direction as an enclosure of
`‖w‖²·p ± √(−E·‖w‖²)·(n × w)`, its sign read off the corner.

**The pins.** None moved, since nothing that runs changed.
`a_turned_frustum_cut_along_its_seam_is_refused_where_the_cut_runs_through_its_apex`
and `the_face_along_a_turned_frustums_seam_crosses_its_cone_in_two_rulings_nothing_names`
still refuse, and the three tallies of `CannotNameWhichPieceOfACrossingThisIs`
in `a_plane_cut_along_a_cones_other_nappe.rs` stay at 8 each: 0 of each 8
moved. Whether the turned frustum then refuses `side of plane` at a seam
corner's box (Q1164) is still not known.

**What this leaves.**

- The fix above, with `arrange.rs` in its files, and the two pins and three
  tallies turned round under it and held to `26π/3` and to volumes. A row of its
  own, tier 1.
- Q1164 waits on that row now, and not on Q1170.

**2026-10-06, Q1171: the four halves are named, and the arrangement reads
each one's own direction.** Lane
`lane-q1171-the-four-half-rulings-of-a-turned-frustum`, base `d6386e04`.

*In `intersect`.* `two_lines_through_a_cones_apex` names the two whole rulings
where both `f64` directions lie on both surfaces, as before, and otherwise
hands back the four halves (`four_half_rulings_through_a_cones_apex`). Each is
picked by `[±M, ±A]`: `M` through the apex facing `n × w` as `f64` works it
out, `A` facing `w`. Both offsets are asked against the exact dot product with
the apex. Each half's direction is enclosed as
`±(‖w‖²·p ± √(−E·‖w‖²)·(n × w))` from the stored numbers, the root rounded
outwards by `root_enclosing`; its sign against both stored normals is read over
the enclosure and must be strict; and the four corners must be four. Anything
short of that refuses as before.

*In `arrange.rs`.* `a_ruling_through_an_apex` tells the two shapes apart by
whether the second picker is the first one reversed. For a half, `along` is
`the_way_a_half_ruling_leaves_the_apex`: the same expression in intervals, its
root rounded outwards by `root_rounded_outwards` (one step down from
`f64::sqrt` of the low end, clamped at nothing, and one step up from that of the
high end, which encloses because `sqrt` is correctly rounded; pinned by
`the_root_rounded_outwards_encloses_the_root`; Q1234 deleted both and calls
`rcad-geom`'s `root_enclosing`, which steps out only where `sqrt` needs it,
pinned in `a_corner_one_step_off_its_half_ruling_is_read_when_the_root_is_exact`),
and of the four candidate ways
out the one both pickers read strictly in front of them. So the sign comes from
the corner, not from which picker is first. Two more changes were needed and
were measured, not guessed:

- `turns_anticlockwise` reads two halves of one ruling, named by opposite
  corners, as two ways out along one line, `Zero`, as it reads two ways out
  along one crossing. Without it the frustum whose apex is inside the block
  refused `which way round a node two ways out of it turn`: `N · (d × −d)` is
  nothing in the reals and an interval straddling nothing over enclosures.
- `place_where_two_rulings_cross` adds each later pair's crossings to the one
  apex node, where four halves would otherwise have placed six nodes there.

*The pins.* The turned frustum, less or intersected with the half past its
seam (`a_turned_frustum_cut_along_its_seam_is_named_through_its_apex_and_refused_at_its_seam_corner`):
under `certified`, the sound frustum and every moved corner now refuse
`Undecidable` on `side of plane` at the box round the bottom seam corner
`(1, 0, 2)` turned, `(1, −0.7788, 1.8421)` give or take `5.7e-14`. That answers
Q1164's first question: yes, it refuses there, as the bar did before §294.9.
Under `tolerant(100)` the sound frustum answers both rules, validates, holds
`26π/3` on a fine mesh, and its corners are the unturned answer's turned, each
within `1e-12`; off the wall and off the cap give `CannotTellTheCellsOfAFaceApart`
and a quarter turn round gives the sound answer to a rounding, as unturned.
`the_face_along_a_turned_frustums_seam_crosses_its_cone_in_four_half_rulings`
pins two runs unturned and four turned. The three tallies of 8 in
`a_plane_cut_along_a_cones_other_nappe.rs` all moved to right answers, each held
to that file's four checks (validates, solids, volume against the closed form
and the column sum, every clear grid point): the frustum with its apex in the
block 24 right and 8 refused to 32 right; the pointed cone at a quarter 38 right,
8 refused and 2 that do not mesh to 46 right and the same 2; at three eighths 40
and 8 to 48 right. `a_narrow_lune_from_a_pole_corner.rs`, whose
`(45, 1000, Difference)` Q1072's note thought might move, did not: the file
passes unchanged, and it has no cone in it.

*Where `[H, H.reversed()]` was looked for.* `holds.rs`: no place reads a
ruling's pickers. `is_a_piece_of_a_ruling` asks the plane through the apex and
the ends on one line with it; `the_picker_naming_the_circle_through` takes only
a run with **one** picker, so it skips whole rulings and halves alike; and
`the_way_into_an_arc` needs `where_a_closed_crossing_is_centred`, which a cone
refuses for a plane steeper than its side. `validate.rs`: the check that no
corner of an edge is behind any of its pickers holds for a corner of a half,
which is in the half's closed corner; `what_a_corner_past_the_picker_says`
takes one picker only and never a plane against a cone. `length.rs`: an edge
with a cone on either side is `CannotYetMeasureHowLongThisCurveIs` before any
picker is read, whole ruling or half. So nothing there breaks, and no row is
filed for it.

**What this leaves.** Q1164, already filed and waiting on Q1171: extend
`the_seam_plane_a_corner_is_on` to cones, now that the turned frustum is shown
to refuse there.

### 294.12 Q1164: a frustum's seam corner is on its moved seam plane too, and the turn at its apex refuses next — 2026-10-08

*Queue row Q1164, from §294.10's "What this leaves". Lane
`lane-q1164-a-turned-frustum-less-its-half-past-the-seam`, base `4c27dddc`.*

**The case.** `cones::a_frustum(model, 0.5, 2, 6)` turned 0.4 about `x`,
less or intersected with `THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM` turned the
same way. Measured again on the base: under `certified` both rules refused
`side of plane` at the box round the bottom seam corner `(1, 0, 2)` turned, as
§294.11 pinned, and under `tolerant(100)` both answered `26π/3`.

**The rule, and the argument §294.10 sketched.** `the_seam_plane_a_corner_is_on`
(`fuse/bake.rs`) takes a cone's axis where it took a pipe's, and does nothing
else differently: the plane through the axis origin and the stored corner, its
normal the axis direction crossed with the corner, kept only when the stored
corner is exactly on it as stored, and moved with `Plane::transformed`. On a
cone the axis origin is the apex. For the frustum's two seam corners, `(1, 0, 2)`
and `(3, 0, 6)`, the normal is `(0, 1, 0)` exactly and the plane is `y = 0`, so
both carry it. The block's face is that plane turned round, bit for bit, so
`place_the_corners` takes the face through them without asking. That the bits
agree is shown by the refusal moving off the seam corner, since
`is_on_by_construction` compares the two planes' bits and nothing else changed.

It is sound for §294.9's two reasons, and the second is where a cone differs.
Moved, the corner is no more exactly on its cap and its cone than on the seam
plane; each is a moved copy of an incidence the model held. And the corner is a
cached sample of the circle the cap and the cone share, so taking it as where
the plane crosses that circle moves no point of the solid, **provided the plane
crosses the circle in no second point a rounding away**. On a pipe the plane
holds the axis and crosses the circle again a diameter away. On a cone the seam
is a ruling, the line through the apex and the corner, and the plane holds it
because it holds the apex and the corner. It crosses the cone in that ruling
and its mirror across the axis, and the two meet only at the apex; a cap square
to the axis crosses them in two points a diameter apart, as on a pipe. A corner
at the apex itself has no plane through it and the axis, and gets none. Which
of the cone's crossings with the face the corner lies on is still read off the
crossings' own pickers (`nothing_rules_the_point_out`), as it is on a pipe: the
seam plane says the face runs through the corner, and not which curve of the
pair does. The four half rulings of §294.11 are each picked by two flat
surfaces through the apex; which of them the corner's box is strictly on the
side of was not printed.

**What it gives: one step further, to the apex.** Under `certified`, the sound
frustum and the corner moved a quarter turn round now refuse `Undecidable` on
`which way round a node two ways out of it turn`, with one record, whose node
is the apex `(0, 0, 0)` exactly and whose two ways out are the enclosures of
`(∓0.5, −0.38941834230865047, 0.9210609940028851)`, `2.2e-15` across. The
corners moved off the wall and off the cap give `CannotTellTheCellsOfAFaceApart`,
as they do unturned and under `tolerant(100)`. Under `tolerant(100)` nothing
moved: the sound frustum is `26π/3` and its corners the unturned answer's
turned, within `1e-12`
(`a_turned_frustum_cut_along_its_seam_gets_past_its_seam_corner_and_is_refused_at_its_apex`,
which replaces `a_turned_frustum_cut_along_its_seam_is_named_through_its_apex_and_refused_at_its_seam_corner`).

**Why the turn at the apex is nothing.** The face asked is the frustum's wall,
on the cone, at its apex, so `turn_between_two_ways_out` (`fuse/arrange.rs`)
reads the turn about the axis, `−w` for two ways out into the nappe along `w`
(`what_a_turn_at_the_apex_is_about`). The two ways out are the two half
rulings into that nappe, and both are crossings of the block's face, so both lie
in it: `t₁ × t₂` is along the face's normal `n`. `n` is `x̂ × w` bit for bit
(§294.11), so `w · n` is nothing exactly, and so is `−w · (t₁ × t₂)`. About the
axis the two are half a turn apart. Each `t` is an enclosure, so the interval
straddles nothing, and `certified` refuses where `tolerant(100)` takes the
`2.2e-15` as nothing.

**It is a gap.** The answer exists: `tolerant(100)` gives it, and a scratch
probe, not kept, showed `certified` gives it too once that turn is read as
`Zero`. The probe returned `Zero` in `turn_between_two_ways_out` at the apex of
a face on a cone whenever the two ways out were crossings of one plane. With
it and this change, both rules answered under `certified`, held `26π/3` on a
fine mesh, validated, and had the unturned answer's corners turned within
`1e-12`; the corners moved off the wall and off the cap gave
`CannotTellTheCellsOfAFaceApart`, and the one moved a quarter turn round gave
the sound answer to a rounding. Without this change the probe's `Zero` would not
be reached: on the base the fuse stops at `side of plane`, so that is asked
before the turn, and the probe does not touch it. The probe's condition is looser
than a fix can be: a plane through the apex that does not hold the axis meets
the nappe in two rulings that are not half a turn apart about it, and `Zero`
there would be wrong. The fix is to read the turn as `Zero` only when the plane
runs through the apex and holds the axis exactly, `n · w = 0` asked of the
stored numbers. `arrange.rs` is outside this row's files, so it is a row of its
own.

**Not tried.** The frustum turned and then shifted. The bar refuses `side of
plane` there because the block's face is an ulp off the moved seam plane
(§294.9), and nothing here says whether the frustum's face is too.

**What this leaves.**

- The apex turn above: `turn_between_two_ways_out` reads two ways out of a
  cone's apex that lie in one plane holding the axis as `Zero`, and the pin
  turns round to answers held to `26π/3`. A row of its own, tier 1.
- `place_the_corners`' doc comment in `arrange.rs` still names only "a round
  bar's seam plane". That row has the file, and fixes it there.

**2026-10-08, Q1361: two rulings of one plane holding the axis turn by
nothing at the apex, and the turned frustum answers under `certified`.** Lane
`lane-q1361-a-frustums-apex-on-a-plane-holding-its-axis`, base `09cb891b`.

*The rule.* `turn_between_two_ways_out` (`fuse/arrange.rs`), at the apex of a
face on a cone, returns `Zero` when both ways out are rulings through the apex
(`through_an_apex`), both crossings of one partner, that partner a plane, and
that plane holds the cone's axis: `n·w` proved nothing of the stored numbers
(`are_two_rulings_of_one_plane_holding_the_axis`). The sign is
`how_a_plane_leans_against_a_pipe` asked of a pipe round the cone's own axis
(`a_plane_holds_a_cones_axis`), so the exact `n·w` is `rcad-geom`'s one copy
and not a second one; it takes no policy, and a sign it cannot prove is not
`Zero`, so the turn is then worked out as before. The check runs after
`what_a_turn_at_the_apex_is_about`, so two ways out into different nappes are
still `TwoCurvesTouchWithoutCrossing`.

*Why the plane holding the axis and not any plane through the apex.* Both ways
out lie in the plane, so `t₁ × t₂` runs along `n`, and the turn
`−w·(t₁ × t₂)` is `n·w` times something that is nothing only when the two are
along one line. So the turn is nothing exactly when `n·w` is. "Through the apex"
says nothing, since every ruling of a plane runs through the apex. A plane
through the apex that misses the axis crosses the cone in two rulings that are
not mirrors across it, and the turn between them is something. That is
measured, not argued: the frustum turned 0.4 about `x` against the half past
its seam left unturned, whose face `y = 0` runs through the apex, is steeper
than the cone, and misses the turned axis. Under the rule above both rules
answer under `certified`, `Model::validate` accepts the model, and each holds
the closed form on a fine mesh: the face cuts each circle square to the axis
along a chord `h·tan 0.4` from its middle, so the piece past it is
`(acos(2·tan 0.4)/4 − tan 0.4·√(1/4 − tan² 0.4))·(6³ − 2³)/3` and the rest is
`52π/3` less that. With the axis check taken out, which is the scratch probe's
any-plane condition, the difference refuses `TwoCurvesTouchWithoutCrossing`
(`a_plane_through_a_frustums_apex_that_misses_its_axis_turns_its_two_rulings_by_something`).

*What it gives.* The frustum turned 0.4 about `x` less or intersected with
the half past its seam turned the same way answers under both policies:
`26π/3` on a fine mesh, `Model::validate` accepts it, and its corners are the
unturned answer's turned, each within `1e-12`. The corner moved a quarter turn
round gives that answer to a rounding, and the corners moved off the wall and
off the cap give `CannotTellTheCellsOfAFaceApart`, as unturned.
`a_turned_frustum_cut_along_its_seam_gets_past_its_seam_corner_and_is_refused_at_its_apex`
is turned round to these answers and renamed
`a_turned_frustum_cut_along_its_seam_gives_what_the_unturned_frustum_gives_turned`.
Without the rule it refuses at the apex as before, the record's enclosure
`2.0539125955565396e-15`. `place_the_corners`' doc names the seam plane of a pipe or a
cone now.

**What this leaves.** Nothing new. The frustum turned and then shifted is still
not tried, as above.
