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
'test(/^a_broken_two_surface_corner_read_back_from_bytes::/)'`, 4 tests run, 4
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

- **A three-or-more-surface corner moved the same way.** A corner with three
  planes round it is tried now, and refuses before the partner is looked at;
  §294.16 records what Q794 found. A corner with four planes round it, a
  pyramid's apex, is tried too and refuses the same way; §294.17 records what
  Q1812 found. One with a curved surface among its three, a half bar's corner
  where its cap, its round wall and its cut meet, is tried too and refuses the
  same way; §294.18 records what Q1813 found.
  §286.4's own gap beside this one, a three-plane corner refused for sitting
  as far off a plane as `validate` allows, was closed by §368, which grows the
  stored corner by `validate`'s room and not by a rounding alone
  (`a_corner_its_own_surfaces_place_somewhere_else.rs`).

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
not tried, as above. *Tried by Q1362 (§294.13): it is the bar's case again.*

### 294.13 Q1362: the turned and shifted frustum cut along its seam is the bar's case again — 2026-10-08

*Queue row Q1362, from §294.12's "Not tried". Lane
`lane-q1362-a-turned-frustum-shifted-along-its-seam`, base `e40e675a`. No line
that runs changed: two tests and a doc comment.*

**The case.** `cones::a_frustum(model, 0.5, 2, 6)` turned 0.4 about `x` and
then shifted by `(7, −5, 2.5)`, less or intersected with
`THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM` placed the same way: §294.9's turned
and shifted bar, on a cone.

**What it gives under `certified`: the bar's refusal.** The sound frustum and
every moved corner, under both rules, refuse `Undecidable` with one record,
`side of plane`, asked of one box: `x` from `7.999999999999773` to
`8.000000000000227`, `y` from `−5.778836684617528` to `−5.778836684617073`, `z`
from `4.342121988005543` to `4.342121988005998`, enclosure
`5.977440764581843e-13`. That is the box round where the bottom seam corner
`(1, 0, 2)` lands, and the block's face along the seam straddles it
(`a_turned_and_shifted_frustum_cut_along_its_seam_cannot_tell_which_side_of_the_plane_its_corner_is`).
The bar refuses the same predicate round its own bottom seam corner, a box
`2.8e-13` either way.

**Why, checked exactly on the stored numbers.** The frustum's seam plane is
`y = 0`, the bar's, since both stand on `Axis::z()` with their seam corners on
`+x`; moved by the placement and turned round, its normal is
`(0, −0.9210609940028851, −0.38941834230865047)` and its offset
`0x400d0dd7b8f4fb65`. The block's face has that normal bit for bit and the
offset `0x400d0dd7b8f4fb66`, the next `f64` up: one ulp further out along its
own normal, where the bar's block face is one ulp the other way,
`0x400d0dd7b8f4fb64` (§294.9). The block's placement is the shift and the turn
composed with the block's own shift to `(−5, 0, 1)`, where the bar's is to
`(−5, 0, −1)`, and the two round apart. So the face is not the seam plane, the
corner is not taken as on it (`is_on_by_construction` is bit for bit, §294.12),
and the box is asked. Both facts are pinned in the test, with two more asked
exactly of the face and the placed cone: the face runs along the axis
(`how_a_plane_leans_against_a_pipe` on a pipe round it is `Zero`) and misses it,
the apex `(7, −5, 2.5)` behind it (`where_a_cones_apex_stands_against_a_plane`
is `Neg`). A scratch probe, not kept, summed `n·apex − d` exactly with
`rcad_predicates::expansion`: `−4.996003610813204e-16` against the face, and
`−5.551115123125783e-17`, the seam plane's own rounding, against the seam
plane. So the frustum's axis lies inside the block by that much, where the
bar's block falls `3.9e-16` short of the bar's axis (Q913). The face crosses
the cone in a hyperbola a rounding from the two rulings, and `intersect`, asked
of the two as placed, names its two branches under both policies (pinned in the
same test).

**So it is the bar's case again, and Q913 is where it is settled.** Q913 pins
whether the bar's refusal is a limit, by an answer holding the seam corner and
the face's crossing with the bottom circle a rounding apart. Here the face is
on the other side of the seam. If the seam corners stand as the axis does —
not summed here, since the turn is applied as a quaternion and a corner's
exact image under it is not one dot product — then of the two rules the
intersection keeps the corner with the crossing a rounding from it and the
difference cuts the corner away, the other way round from the bar. No second
limit row is filed. What is not run is whether the rule that cuts the corner
away holds any pair a rounding apart at all: its answer's corners near the seam
would be the face's crossings alone. That is a guess nobody has run, a row of
its own after Q913.

**Under `tolerant(100)` it is refused further on, where the bar answers.** The
turned and shifted bar cut by the half past its seam answers both rules, half
the bar on a fine mesh, and validates. The frustum gets past its seam corner
and the sound frustum and the corner moved a quarter turn round refuse
`Error::Geometry(CannotNameWhichPieceOfACrossingThisIs)`; the corners moved off
the wall and off the cap give `CannotTellTheCellsOfAFaceApart`, as at every
placement tried
(`under_a_tolerant_policy_a_turned_and_shifted_frustum_cut_along_its_seam_is_refused_where_the_bar_answers`).
It is not `intersect` on the face and the cone, which names the hyperbola;
where the fuse raises it was not traced. Under `certified` it is not reached,
since `side of plane` comes first.

**What this leaves.**

- The `tolerant(100)` refusal above: trace where the fuse raises
  `CannotNameWhichPieceOfACrossingThisIs` for the turned and shifted frustum,
  and whether the answer the bar gets is there to be had. A row of its own,
  tier 1.
- Whether the rule that cuts the seam corner away has an answer `certified`
  could reach, for the bar and the frustum both. A row after Q913.

Both rules answer under `certified` now, for the bar since Q2374 and for the
frustum since Q2587 (§294.25).

### 294.14 Q1397: the tolerant refusal is a walk asking a hyperbola's vertex a rounding from the apex — 2026-10-08

*Queue row Q1397, from §294.13's "What this leaves". Lane
`lane-q1397-where-the-tolerant-frustum-cut-is-refused`, base `caed9389`. No line
that runs changed: a test, two doc comments and this subsection. The trace
used prints and a backtrace in `rcad-geom`, taken out before the commit.*

**Where it is raised.** Not while the faces are cut up: the fuse puts its
answer together, and then `nest_the_shells` measures each shell by meshing it,
and the mesher refuses walking one edge. Line numbers at `caed9389`:

`Model::fuse` (`fuse/mod.rs:236`) → `fused_under` (`:251`) → `assemble`
(`fuse/assemble.rs:125`) → `nest_the_shells` (`:2570`) →
`how_much_space_it_encloses` (`:3622`) →
`how_much_space_a_curved_solid_encloses` (`validate.rs:2464`) →
`the_coarsest_mesh_whose_sides_do_not_cross` (`:2627`) → `mesh`
(`mesh.rs:409`) → `walk_the_edges_this_face_can_place` (`:705`) → the
mesher's `walk` (`:859`) → `rcad_geom::walk` (`walk.rs:439`) →
`what_a_run_between_these_two_lies_on` (`:759`) → `where_a_plane_cuts_a_cone`
(`:2886`) → `which_branch_of_a_hyperbola_this_run_is` (`:3015`) →
`which_side_of_the_apex`, which returns the error at `:3066`.

All four refusals the pin holds (two corners, two rules) come by that path.

**What it is handed.** The edge the block's face cuts along the cone, from a
corner on the bottom circle to one on the top circle. The first walked is,
under `Difference`, the one down the seam, `(1, 0, 2)` to `(3, 0, 6)` before
the placement, and under `Intersection` the one down the far ruling,
`(−1, 0, 2)` to `(−3, 0, 6)`. The surfaces are the block's face and the
frustum's cone, apex at `(7, −5, 2.5)` and slope `0.5`, each in the frame the
mesher shifts that face into. The face is steeper than the cone and misses its
apex by `4.996e-16`, the number §294.13 summed exactly. So they cross in a
hyperbola, and `where_a_plane_cuts_a_cone` works out a half-axis of
`9.992e-16` towards the vertices and `4.996e-16` across. Its centre is within
`5e-16` of the apex, so each branch's vertex stands about `1e-15` from it, one
each side.

**Why it is refused.** `which_branch_of_a_hyperbola_this_run_is` asks both
ends which side of the plane through the apex square to the axis they are on,
and both are decided, on the side the axis points to. Then it asks each
branch's vertex the same thing, to find the branch the ends are on. A vertex
is placed to `POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF` times the size, `9.9e-14`
here, so its box holds the apex: the side is `Undecidable`, enclosure
`2.6e-13`. `certified` leaves it so and `tolerant(100)` reads it as `Zero`, and
`which_side_of_the_apex` refuses both, since neither names a branch. The new
pin asks `rcad_geom::walk` the same thing with the two seam corners as ends,
under both policies and with each of the two runs `intersect` names, and pins
the one record, `which side of a cone's apex`, whose box holds the apex
(`walking_the_cut_along_a_turned_and_shifted_frustums_seam_asks_a_point_a_rounding_from_the_apex`).
The bar never gets here: a face along a pipe's axis cuts it in two straight
lines. The frustum turned alone does not either: its face holds the apex
exactly, and a face through the apex cuts straight lines too.

**A gap, not a limit.** The answer is there and the arithmetic can reach it
without the vertex. Each branch is built as `centre + and·towards·s·cosh t +
across·r·sinh t`, `s = ±1`, where `across` is square to the axis. So far out,
branch `s` runs along `s·and`, and it lies in the half of the cone on that
side of the apex. `and` is `â × n̂` with `â` along `w × n`, so `and·w =
−|w × n̂|`. That is never positive. It is far from zero on every hyperbola:
a plane crosses a cone in one only when it is steeper than the cone, and then
`|and·ŵ| > 1/√(1 + slope²)`, `0.89` for this cone. So the branch the ends are
on is `s = −1` when they are on the axis's side and `s = +1` when they are
not, and no point a rounding from the apex is asked. Measured with a scratch
probe that picked the branch that way, not kept:

- the sound frustum and the corner moved a quarter turn round answered both
  rules with one solid each, which `validate_solid` accepts and which measures
  `26π/3` on a fine mesh (`holds`); the sound frustum's whole model validates.
  The corners moved off the wall and off the cap still give
  `CannotTellTheCellsOfAFaceApart`.
- with the probe asserting the two ways agree wherever the vertex could be
  told, `rcad-geom` and `rcad-topo` ran 2789 tests, and only the pin above
  failed, because it now answered. 21,510 branch picks were decided by their
  vertices and every one agreed with the sign. 76 could not be told by their
  vertices. `and·ŵ` was negative in all of them, between `−0.894` and `−1`.

The family of fixes here is "ask something other than the vertex". The vertex
of this hyperbola is a rounding from the apex, so asking it more exactly
cannot settle it. A point far out on the branch could be asked instead, but the
sign of `and·w` names the branch with no point at all.

**On the way past.** The fuse's `start_every_closed_crossing_that_has_no_node`
(`arrange.rs:2181`) asks `where_a_closed_crossing_starts` of the branch past
the apex, which reaches neither face. That goes through the same function
with no ends, so the branch is picked by `intersect`'s half-space at the
vertex instead. Under `tolerant(100)` the half-space holds both vertices and
it is refused at `walk.rs:3022`. The fuse drops it with `let Ok(..) else {
continue }`. That is right here, since the branch touches neither face. Q1195,
a walk refused in the same function on a hyperbola a rounding off a parabola,
is not traced and this rule was not run against it.

**What this leaves.**

- The fix: pick a hyperbola's branch for a run with ends by the ends' side and
  the sign of `and·w`, in `which_branch_of_a_hyperbola_this_run_is`, and turn
  the two pins above into answers. A row, tier 1.

### 294.15 Q1405: a hyperbola's branch is named by the side its ends are on — 2026-10-08

*Queue row Q1405, from §294.14's "What this leaves". Lane
`lane-q1405-a-hyperbolas-branch-picked-by-its-ends`, base `ee5b984a`.*

**The fix.** For a run with ends, `which_branch_of_a_hyperbola_this_run_is`
asks only which side of the apex the ends are on, and
`which_way_the_branch_these_ends_are_on_opens` turns that into the branch:
`s = −1` when they are on the side the axis points to, `s = +1` when they are
on the other. No vertex is asked. `which_side_of_the_apex` hands back the
`Decision`, resolved under the policy, and the new function matches it all four
ways: `Pos` and `Neg` name a branch, `Zero` and `Undecidable` refuse
`CannotNameWhichPieceOfACrossingThisIs` as before, and two ends on two sides
refuse as before. The run with no ends is not touched: its pickers still ask
each vertex, as §294.14's "On the way past" describes.

**Why the side names the branch.** The argument is §294.14's, now written on
the function. Branch `s` is `centre + and·towards·s·cosh σ + across·sinh σ`,
`towards` is positive and `across` is square to the axis, so far out the branch
runs along `s·and` and is on the side `s·(and·w)` names. `and` is `â × n̂`,
`â` along `w × n`, and `(w × n) × n · w = (w·n)² − |w|²|n|² = −|w × n|²`, so
`and·w = −|w × n̂|`, never positive, and zero only for a plane square to the
axis, which cuts a circle. The identity holds of the exact `and`, and the
branch built with `s` on the rounded `and` is the rounding of the exact branch
with that `s`, so nothing rounded is read to pick it.

**Measured, with a scratch probe not kept.** The probe kept the fix and, at
each pick with ends, also asked both vertices as the old code did, with a log
of its own so no record changed, and wrote one line per pick:
`agree` where the vertices named exactly one branch and it was the fix's,
`disagree` where they named the other, `untold` otherwise, with `and·ŵ`. Run
as `just t -p rcad-geom -p rcad-topo --no-fail-fast`, 2795 tests, only the two
pins below failed, because they now answered. Of 21,553 picks, 21,536 agreed,
none disagreed, and 17 were untold, all of them from those two pins.
`and·ŵ` was between `−1` and `−0.866` at every pick.

**What it gives.**

- The turned and shifted frustum cut along its seam under `tolerant(100)`
  (§294.13): the sound frustum and the corner moved a quarter turn round answer
  both rules with half the frustum, `26π/3` on a fine mesh, and
  `validate_solid` accepts each; the sound frustum's whole model validates, its
  corners are the turned frustum's answer shifted, to `1e-12`, and the corner
  moved a quarter turn round gives the sound answer's corners to the same. The
  corners moved off the wall and off the cap still give
  `CannotTellTheCellsOfAFaceApart`
  (`under_a_tolerant_policy_a_turned_and_shifted_frustum_cut_along_its_seam_answers_as_the_bar_does`).
  Under `certified` it still refuses `side of plane` first (§294.13, Q913).
- `rcad_geom::walk` on the seam's cut between the two seam corners walks,
  under both policies and with either run `intersect` names, with nothing
  recorded, every point between the bottom and top circles and on the face and
  the cone to twice the walk's deviation
  (`walking_the_cut_along_a_turned_and_shifted_frustums_seam_names_the_branch_its_corners_are_on`).

**Q1195's shape, run after the fix as a scratch probe not kept.** The bar of
`an_edge_between_two_planes_its_curve_leaves.rs` with its lid split by a
half disc on `leaning_off_the_lid_through((0, 0, 4), −1e-14)`, the arc naming
`a_cone_the_lid_crosses_in_a_parabola`, half disc listed first: it builds, and
`Model::validate` under `certified` accepts it. That is the check Q1195 asks to
add. Measuring the solid then refuses
`CannotPlaceAPointThatCloseToACrossing { asked: 0.001875, closest: 0.355 }`:
the hyperbola a `1e-14` lean off a parabola is so long that its published
bound is `0.355`. With the lid listed first it measures `113.003 ± 0.376`,
against `36π = 113.097`. The probe was not run on the base, so which ask
refused there was not seen; the bound fits a vertex's side, since the near
vertex is about `0.5/√2 = 0.354` from the plane through the apex and its box
was `0.355` either way.

**What this leaves.**

- Q1195's shape with the half disc first builds and validates but does not
  measure, on the bound above. Whether that bound can be had smaller for a
  hyperbola so near a parabola is a row of its own, tier 5, after Q1195.

### 294.16 Q794: a broken three-plane corner read back from bytes is refused before any partner is looked at — 2026-10-12

*Queue row Q794, from §294.7. Lane
`lane-q794-a-broken-three-surface-corner-read-back`, base `2af38de7`, carried
on by a second lane after the first passed its budget (§416). Pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`.
No code changed but `Model::fuse`'s doc comment.*

**The corner.** A box, `x` and `y` from −3 to 3 and `z` from 0 to 10, built
with `add_extrusion` at §294.8's four placements: none, the shift
`(7, −5, 2.5)`, the turn 0.4 about `x`, and the turn then the shift. The
placement goes on the shell, so the file stores the top corner at `(3, 3, 10)`
at every placement, where the planes `x = 3`, `y = 3` and `z = 10` meet. As
in §294.1, the test overwrites the one place the bytes hold it, found once at
every placement, with one of four moves: a millimetre off the `x = 3` wall,
`(4, 3, 10)`; a millimetre above the top, `(3, 3, 11)`; a millimetre into the
box from all three planes, `(2, 2, 9)`; and a millimetre out from all three,
`(4, 4, 11)`. `Model::validate` refuses every one, under `certified` and
`tolerant(100)` at every placement, with `VertexOffItsFacePlane { distance:
1.0 }`, and the same bytes with the corner left alone validate.

**The partners.** §294.2's ten partner and rule pairs and `OVER_THE_TOP` −,
eleven, each block placed as the box is. As the control, the sound box answers
all 88 (two policies, four placements, eleven pairs) `Ok`, the model validates
after each, and each answer holds what box arithmetic says the two boxes hold,
to `1e-12` relative: 368 in two solids for `FAR_AWAY` ∪; 288 for
`OVER_THE_BOTTOM` − and for `OVER_THE_TOP` −, 72 for `OVER_THE_TOP` ∩; 348,
498 and 12 for `OVER_THE_RIM_BY_THE_SEAM` −, ∪ and ∩; 348 for
`OVER_THE_RIM_AWAY_FROM_THE_SEAM` −; 356 for `A_NOTCH_A_QUARTER_TURN_ROUND` −;
180 for `THE_HALF_PAST_THE_SEAM` − and ∩. At the turned placements some are a
few ulps off, `287.99999999999994` for one.

**What comes back.** All 352 fuses of a moved box, four moves against those
88, refuse `Error::Undecidable` with one record, whose predicate is "where
three surfaces meet", and leave the model byte for byte as it was. **No wrong
`Ok` and no panic.** Unlike §294.2's bar, whose moved corner gave the sound
answer wherever the partner cut it away, the box refuses with every partner,
`FAR_AWAY` and the partners that cut the corner away among them, and with every
move, placement and policy. `mise exec -- just t -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'` runs the three
pins: 3 tests run, 3 passed.

**Why the partner does not matter.** `bake` places every corner of every input
in `corner_of_every_vertex` before the arrangement sees the partner. A corner
with three or more distinct surfaces round it goes to `derive_a_corner`, which
solves each triple's meet and keeps a place only where
`the_place_the_model_points_at` says the stored corner points at it: the
stored corner, grown by what writing it down cost and by the room `validate`
grants a corner off a plane, `1e-9` of the solid's size (§368), has to overlap
the meet. The box's three planes cross at `(3, 3, 10)` alone, a millimetre is
far outside that room, so no triple places the moved corner, and
`derive_a_corner` gives up on `WHERE_THREE_SURFACES_MEET`. §294's two-surface
corner is not asked at all (§286), which is why the bar's fuse got as far as
its partner.

**Why that is not a defect.** It is §294.4's blame question from the other
side: the refusal says the arithmetic gave up, where `validate` says the file
is broken. It is safe, since the model is untouched and nothing wrong comes
back, and §294.5 already found that asking at the door buys attribution and not
correctness. Turning `the_place_the_model_points_at`'s `None` into `Invalid` in
place would not even buy that. It says `None` for §70.3's corner too: three
planes at a `1e-14` graze, with a corner `1.414` from where they cross, which
`validate` accepts because the corner is within a rounding of each plane
(`a_corner_near_three_grazing_planes_but_far_from_where_they_cross_is_refused`
in `bake.rs`). Calling that corner `Invalid` would call a solid `validate`
accepts broken. A corner `validate` refuses and one it accepts reach the same
`None`, and telling them apart there means asking `validate`'s own question of
the stored corner, which is §294.5's option (c).

**`Model::fuse`'s `# Errors` said otherwise, and is changed.** It said
`Error::Invalid` comes when an input's corners and its own planes disagree by
more than the corners' boxes. A three-plane corner a millimetre off gives
`Undecidable`. The `Invalid` is `derive_a_corner`'s re-check, which asks the
solved corner, grown by the same room, against every surface round it, after
some triple has placed it. The triple's own three surfaces hold the meet they
solved to, so the re-check can only find a further surface off it, and that
takes four or more surfaces round the corner. The doc now says so, and its
`# Panics` paragraph, which ended on the three-surface corner being untried,
now gives this result.

§286.4 named a three-plane corner refused `Undecidable` for sitting as far off
a plane as `validate` allows, because `the_place_the_model_points_at` grew the
stored corner by a rounding alone. §368 grew it by `validate`'s room, and
`a_corner_its_own_surfaces_place_somewhere_else.rs` pins it: `mise exec -- just
t -E 'test(/^a_corner_its_own_surfaces_place_somewhere_else::/)'`, 5 tests run, 5
passed. The corner here is a millimetre off, past any room, and not that gap.

**What this leaves.** Two corners not tried when this was written, each with
no class, since nobody had run them (§360.5):

- **A corner with four or more surfaces round it**, a pyramid's apex moved in
  its bytes. Tried by Q1812: it refuses as the box's corner does, and §294.17
  records it.
- **A corner with a curved surface among its three.** The round bar less
  `THE_HALF_PAST_THE_SEAM` keeps a corner at `(−3, 0, 10)` where the cap's
  plane, the wall's cylinder and the plane `y = 0` meet. Those three meet at two
  places, `(−3, 0, 10)` and `(3, 0, 10)`. A corner moved a millimetre off is
  guessed to refuse as the box's does. Moved onto the other place, the chord's
  other end, `the_place_the_model_points_at` would pick that place, which is
  the one move here that could come back a wrong `Ok`. Tried by Q1813: every
  millimetre move refuses as the box's corner does, and the move onto the
  chord's other end is picked there and refused, with no wrong `Ok`; §294.18
  records it.

### 294.17 Q1812: a pyramid's apex moved in its bytes is refused as the box's corner is, and the sound pyramid cut through its apex refuses — 2026-10-12

*Queue row Q1812, from §294.16. Lane
`lane-q1812-a-pyramid-apex-moved-in-its-bytes`, base `302aea8c`, carried on by
a second lane after the first passed its budget (§416). Pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`
beside §294.16's box. No code changed.*

**The corner.** A square pyramid on the box's footprint, base `x` and `y` from
−3 to 3 at `z = 0` and apex `(0, 0, 10)`, built face by face with
`SolidBuilder`, each side face's plane written through the apex. Four distinct
planes meet at the apex. The row named the pyramid of
`a_corner_with_four_planes_round_it.rs`, whose base is bent at one corner so
two side faces fold by only a little. Its brief proposed fold 0, where those
two faces lie in one plane and the apex has three distinct planes round it, so
the square pyramid was used instead. It is placed with `place_solid` at §294.8's four
placements, so the file stores the apex at `(0, 0, 10)` at every placement, and
the test overwrites the one place the bytes hold it with one of four moves: a
millimetre above, `(0, 0, 11)`; a millimetre down into the pyramid,
`(0, 0, 9)`; a millimetre down the edge towards `(3, 3, 0)`, `(0.3, 0.3, 9)`;
and a millimetre down the `+x` face, `(0.3, 0, 9)`. The last two stay on the
planes they slide along to a rounding of the `0.3`.

`Model::validate` refuses every move under both policies at every placement
with `VertexOffItsFacePlane`. A side plane rises 10 over 3, so a millimetre up
or down is `3/√109 ≈ 0.287` off it, and that is the distance named for the
moves above, into and down the face. Down the edge it names `6/√109 ≈ 0.575`.
The distance is to one plane the apex is off and not the farthest: down the
face the apex is `3/√109` off the `±y` planes and `6/√109` off the `−x` plane,
and `validate` names `3/√109`. The sound bytes validate.

**What comes back.** All 352 fuses of a moved apex, four moves against §294.16's
eleven partner and rule pairs at four placements under two policies, refuse
`Error::Undecidable` with one record, "where three surfaces meet", and leave
the model byte for byte as it was. **No wrong `Ok` and no panic.** That was the
row's guess, and the reason is §294.16's: each three of the four side planes
cross at the apex alone, since their normals `(0, ±10, 3)` and `(±10, 0, 3)`
are three by three independent, so `derive_a_corner` in `fuse/bake.rs` finds
every triple's meet at `(0, 0, 10)`. A millimetre is far past the `1e-9` of
room `the_place_the_model_points_at` grows the stored apex by, so no triple
places it, and `derive_a_corner` gives up before its re-check runs. The
re-check's `Invalid` still wants a stored corner where three surfaces cross and
off a further one, which a moved plane gives and a moved corner does not
(§294.16).

**The control, and one partner it does not answer.** The sound pyramid answers
nine of the eleven pairs at every placement under both policies, 72 fuses, the
model validates after each, and each answer holds what the pyramid and the
block hold, to `1e-12` relative: 128 in two solids for `FAR_AWAY` ∪; 61.44 for
`OVER_THE_BOTTOM` −; 0.96 for `OVER_THE_TOP` ∩ and 119.04 for −. Above
`z = 8` the pyramid is 1.2 across, so the rim and notch blocks miss it: 120 for
`OVER_THE_RIM_BY_THE_SEAM` −, `OVER_THE_RIM_AWAY_FROM_THE_SEAM` − and
`A_NOTCH_A_QUARTER_TURN_ROUND` −, 270 in two solids for
`OVER_THE_RIM_BY_THE_SEAM` ∪, and `Ok` with no solid for its ∩.

The two pairs left out are `THE_HALF_PAST_THE_SEAM` − and ∩, whose face
`y = 0` runs exactly through the apex. **Under `certified` both refuse
`Undecidable` at every placement**, with one record, "side of plane", asked of
the box `bake` solved round the apex: at no placement `x` from `−8.4e-16` to
`8.4e-16` and `y` from `−4.2e-16` to `4.2e-16`, which straddles the cut. Under
`tolerant(100)` both answer half the pyramid, 60, with no placement, and the
model validates; at the other three placements both refuse with four "side of
plane" records round the apex and one "a point inside a piece of a face". The
answer exists and has no feature smaller than a millimetre, so this is a gap
(§16.10), and halving a pyramid through its apex is a shape designers make:
class `refuses/B`, a row of its own. The pin
`a_sound_pyramid_cut_through_its_apex_cannot_tell_which_side_of_the_cut_its_apex_is_on`
holds it as it stands, so the row that closes it turns the pin round. Where
the fuse asks the question was not traced. Q397, dropped, once proposed
deciding "side of plane" exactly for a corner where three planes cross, by the
signs of two determinants.

**A tetrahedron cut through its apex fails differently.** To see whether the
fault needs four planes, a probe that was not kept built a tetrahedron with
apex `(0, 0, 10)` over the base `(−3, −3, 0)`, `(3, −3, 0)`, `(0, 3, 0)`, three
slanted planes round its apex, and fused it with the block `x` from −5 to 5,
`y` from 0 to 5, `z` from −1 to 11, placed as the pyramid is. Each of the 11
of its 16 fuses that answered holds the right volume to a few ulps: the `y ≥ 0` half
is the cone from the apex over its half of the base, 15, and the rest is 45.
Under `tolerant(100)` all eight answer. Under `certified` five of eight refuse
`Invalid::DynamicRangeTooWide`, at ratios from `1.3e15` to `4.5e16`: −
at every placement but none, ∩ at none and at the shift. No "side of plane"
refusal comes back. So the pyramid's refusal is not every slanted corner on a
cut, and the tetrahedron's is a second gap, since its answer has no small
feature either: class `refuses/B`, a row of its own. *§294.22 found otherwise:
the stored front face is a rounding off the apex, the answer the kernel is
handed does have a small feature, and all five refusals are limits.*

`mise exec -- just t -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'` runs the box's
three pins and the pyramid's four: 7 tests run, 7 passed.

**What this leaves.**

- The pyramid cut through its apex refusing "side of plane", above. A row,
  tier 1, `refuses/B`.
- The tetrahedron cut through its apex refusing `DynamicRangeTooWide` under
  `certified`, above. A row, tier 1, `refuses/B`.
- A corner with a curved surface among its three, §294.16's second item, was
  not tried when this was written. Q1813 tried it, and §294.18 records it.

### 294.18 Q1813: a corner where a cap, a round wall and a cut meet, moved in its bytes, is refused at both places they meet — 2026-10-12

*Queue row Q1813, from §294.16. Lane
`lane-q1813-a-curved-corner-moved-in-its-bytes`, base `86192f96`, carried on by
a second lane after the first passed its budget (§416). Pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`
beside §294.16's box and §294.17's pyramid. No code changed.*

**The corner.** `add_round_bar(3, 10)` less `THE_HALF_PAST_THE_SEAM`, the
block `x` from −5 to 5, `y` from 0 to 5 and `z` from −1 to 11, fused under
`certified` with neither placed, and the half bar that comes back placed with
`place_solid` at §294.8's four placements, as the pyramid is. So the file
stores its corner at `(−3, 0, 10)` at every placement, found once in the bytes
at each, where the cap's plane `z = 10`, the wall's cylinder and the cut's
plane `y = 0` meet. Those three meet at `(3, 0, 10)` too, the other end of the
cut across the top, where the half bar keeps a corner of its own. The test
overwrites the stored corner with one of five moves: a millimetre out from the
wall, `(−4, 0, 10)`; a millimetre above the cap, `(−3, 0, 11)`; into the half
bar, `(−2, −1, 9)`; out from all three, `(−4, 1, 11)`; and onto the other end
of the cut, `(3, 0, 10)`. The bar and the block stay in the model beside the
half bar, since nothing in the API removes a solid and a key does not survive
the bytes, so the test finds the half bar in the model it reads back by its
place among the model's solids.

`Model::validate` refuses each millimetre move under both policies at every
placement with `VertexOffItsFaceSurface`, the move off the cap's plane too:
the first fuse wrote each edge at the corner as part of where two surfaces
cross, and `check_edge` asks each end of such an edge about the second surface
before any face asks about its own. It refuses the move onto `(3, 0, 10)` with
`EdgeHasNoLength`, since the cut's straight edge across the top now has both
its ends at one point. The sound bytes validate.

**The control, and one partner it does not answer.** The sound half bar answers
nine of §294.16's eleven partner and rule pairs at every placement under both
policies, 72 fuses, the model validates after each, and each answer holds what
the half bar and the block hold, to `1e-12` relative plus the `volume_error`
the measure publishes at a deflection of `0.001`. The half bar holds
`45π ≈ 141.372`. A block reaching right across its half disc between `x = a`
and `x = b` shares `F(b) − F(a)` per unit of height, with
`F(x) = (x·√(9 − x²) + 9·asin(x/3))/2`. That gives 149.372 in two solids for
`FAR_AWAY` ∪; `36π ≈ 113.097` for `OVER_THE_BOTTOM` − and `OVER_THE_TOP` −,
`9π ≈ 28.274` for `OVER_THE_TOP` ∩; 138.274, 288.274 and 3.0975 for
`OVER_THE_RIM_BY_THE_SEAM` −, ∪ and ∩; 138.274 for
`OVER_THE_RIM_AWAY_FROM_THE_SEAM` −; and `45π` for
`A_NOTCH_A_QUARTER_TURN_ROUND` −, which misses the half bar. `the_volume_of`
measured at the scale's own deflection with no bound before, which a curved
answer cannot meet to `1e-12`; it now measures at `0.001` and returns the
`volume_error` beside the volume. The box's and the pyramid's checks are the
same as they were, since a flat solid's error is only its sum's rounding.

The two pairs left out are `THE_HALF_PAST_THE_SEAM` − and ∩: the block that
cut the half bar, fused with it again, flush along its flat face `y = 0`. The
answers exist, the whole half bar and nothing, and both come back with no
placement and shifted under both policies, and turned under `tolerant(100)`.
**Turned, they refuse.** Under `certified` turned both refuse `Undecidable` on
"orient3d"; turned and shifted, `Undecidable` on "side of plane" under
`certified` and `TwoCurvesTouchWithoutCrossing` under `tolerant(100)`. Each
refusal leaves the model as it was. A cut flush along a face is a shape
designers make, and the answer has nothing small in it, so this is a gap
(§16.10): class `refuses/B`, a row of its own. Where the fuse refuses was not
traced. The turned and shifted "side of plane" may be §294.9's sound bar cut
along its seam
(`a_turned_sound_bar_cut_along_its_seam_cannot_tell_which_side_of_the_plane_its_corner_is`),
which Q1398 is open beside; nobody has run that lead. The pin
`a_sound_half_bar_cut_flush_along_its_flat_face_refuses_when_turned` holds it
as it stands, so the row that closes it turns the pin round.

**2026-10-15, Q1834 (Q2033): the turned, certified case is closed.**
`what_a_flat_face_holds` in `holds.rs` used to aim its half-line at the first
corner the wire walk met; that aim lay on the line through the far corner the
turned test point sits on, §448.5's entry for "Q1842 turned". Q1834's lane
turns the walk round: it now aims at every corner and then at the middle of
each side, so no single aim corner can line up with the test point. `(turn,
&certified)` answers both rules now, same as `(turn, &tolerant)` already did.
Only turned-then-shifted still refuses, under both policies, so this closes
with the pin renamed
`a_sound_half_bar_cut_flush_along_its_flat_face_refuses_when_turned_and_shifted`,
not turned round onto a different case.

**What comes back from a millimetre move.** All 352 fuses, four moves against
the eleven pairs at four placements under two policies, refuse
`Error::Undecidable` with one record, "where three surfaces meet", and leave
the model byte for byte as it was. **No wrong `Ok` and no panic**, as the row
guessed, and for §294.16's reason. With three surfaces round the corner,
`derive_a_corner` in `fuse/bake.rs` has one triple to solve, and its meet is
the two places `(−3, 0, 10)` and `(3, 0, 10)`. `the_place_the_model_points_at`
takes a place only where the stored corner, grown by `validate`'s room,
overlaps it. A millimetre is far past that room, so neither place is taken and
`derive_a_corner` gives up. The curved wall changes nothing here.

**What comes back from the move onto the other end of the cut.** This is the
move that could have answered wrong: the stored corner points at
`(3, 0, 10)`, one of the two places its triple meets, so
`the_place_the_model_points_at` takes it. All 88 fuses refuse, none comes back
`Ok`, none panics, and each leaves the model as it was:

- Under `tolerant(100)` at every placement, and under `certified` with no
  placement and shifted, 66 fuses: `Error::Invalid(EdgeHasNoLength)`, the
  word `validate` has for these bytes.
- Under `certified` turned, all eleven pairs: `Undecidable` on "order along a
  crossing".
- Under `certified` turned and shifted: nine pairs on "order along a crossing",
  and `THE_HALF_PAST_THE_SEAM` − and ∩ on "side of plane", as the sound half
  bar refuses there.

The `EdgeHasNoLength` is `fold_the_nodes`' in `fuse/arrange.rs`, which refuses
with it when two nodes it folds into one were made from two different corners
of an input. The moved corner, placed at `(3, 0, 10)`, and the half bar's own
corner there are those two. So the corner is placed where the row guessed, and
the fuse finds it on top of another and refuses, rather than answer with a cut
that has no length. That refusal names the broken input, as `validate` does,
where the millimetre moves' refusal names the arithmetic (§294.16).

`mise exec -- just t -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'` runs the box's
three pins, the pyramid's four and the half bar's five: 12 tests run, 12
passed.

**What this leaves.**

- The sound half bar cut flush along its flat face refusing when turned,
  above. A row, tier 1, `refuses/B`. **2026-10-15, Q1834 (Q2033):** closed for
  the turned, certified case; the turned-then-shifted refusal left is Q2034.
- `Model::fuse`'s doc in `crates/rcad-topo/src/fuse/mod.rs`, at the end of its
  `# Panics` section, still says a corner with four or more surfaces round it,
  or with a curved one among three, is untried (§294.7). Both are tried now,
  in §294.17 and here. A small row, since that file was outside this row's
  write set.

### 294.19 Q1838: the sound pyramid cut through its apex answers where it stands, and placed cannot order two points by the apex — 2026-10-13

*Queue row Q1838, from §294.17. Lane
`lane-q1838-a-pyramid-apex-on-a-block-face-read-back`, base `9d421909`,
carried on by a second and a third lane as each passed its budget (§416).
Code changed in `fuse/bake.rs` and `fuse/arrange.rs`; pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`
beside §294.17's pyramid.*

**Where the fuse asked.** §294.17 left the sound pyramid fused with
`THE_HALF_PAST_THE_SEAM`, whose face `y = 0` runs exactly through the apex,
refusing "side of plane" under `certified` at all four placements. The record
comes from `Arrangement::passes_through` in `fuse/arrange.rs`, called from
`place_the_corners`, from `Arrangement::of`, `split_a_face` and
`split_both_groups`. It asks whether the apex is on the line where a side face
meets the cut. The box bake solved round the apex reaches `4.2e−16` either side
of `y = 0`, so the box cannot say. For a corner with exactly three planes round
it, `passes_through` then asks the three planes, exactly, with
`side_of_a_plane_where_three_planes_meet`. The apex has four, and
`the_three_planes_a_point_is_the_meet_of` gave `None` for four, because bake
did not say which three it solved the apex from. The other way out,
`side_of_a_plane_holding_a_rings_axis`, is for a corner on a ring's axis, so
the `Undecidable` stood.

**The change.** `BakedCorner` keeps `solved_from`, the three surfaces
`derive_a_corner` placed the corner from: of the threes whose meet the stored
corner points at, the first whose box is least uncertain. `passes_through` asks
those three
where the box cannot say. They are the three to ask because the box is sure to
hold their meet, and holds another three's only give or take a rounding. Every
face round the apex reads the same `BakedCorner`, so each asks the same three
and the faces cannot disagree about which side the apex is on. A corner with
exactly three round it is solved from those three, so it is asked as before. A
corner with fewer is read out of the model and has none.

**Four planes as written need not meet in one point.** A print the second lane
put in and took out showed, for each three of the four side planes, which side
of the cut their meet is on:

- With no placement, every three meets exactly on the cut. All four fuses, −
  and ∩ under both policies, answer half the pyramid, 60, in one solid, and the
  model validates after each.
- Shifted, the threes disagree: planes 0, 1 and 2, 0, 1 and 3, and 0, 2 and 3
  meet on the negative side of the cut, and 1, 2 and 3 on the positive side.
  Bake solved the apex from planes 0, 1 and 2, whose offsets are
  `−8.380979995685074`, `−10.296632566127379` and `1.1972828565264395`, and
  against the cut, `(0, −1, 0, −5)`, that puts it on the negative side, inside
  the block.
- Turned, the three bake solved from put the apex on the positive side,
  outside the block.
- Turned and then shifted, the three bake solved from have offsets
  `−7.573228205953318`, `−0.6160395235268386` and `2.6101501318079823`, and put
  the apex on the positive side.

**What comes back placed.** Shifted, turned, and turned then shifted, − and ∩
both refuse `Undecidable` and leave the model as it was. Under `certified` it
is one record, "order along a crossing". Under `tolerant(100)` it is twenty of
those and then "a point inside a piece of a face"; before the change it was
four "side of plane" and that one. Every "order along a crossing" record was
asked of two points each within `1e-12` of the placed apex in every
coordinate. A print the second lane put in and took out found the first of
them, shifted: on the `−y` side face, along its edge with the `−x` side face,
between the apex and the point where the cut crosses that edge.
`Arrangement::order_along` refuses, called from `nodes_along`, from
`merge_points_that_are_one_point` in `Arrangement::of`. The apex is a rounding
off the cut, so the cut crosses that edge a rounding from the apex, and the
apex, solved from planes 0, 1 and 2, is not exactly on that edge's line, which
is where planes 0 and 3 meet. `as_the_planes_say` reads `.start.around`; made
to read `solved_from` instead, it changed none of the results, since planes 0,
1 and 2 do not hold both planes of that edge, so it was put back.

A pyramid halved through its apex is a shape designers make, and the answer,
60, has nothing small in it. Whether the arithmetic can reach it is not
settled. Taking the apex as a rounding off the cut, the cut would take a
triangle a rounding across off the top of a side face, so deciding these
orders exactly may only move the refusal on to that sliver; nobody has run
that. So it is `refuses/B` and a row, not yet a limit or a gap.

The pins: `a_sound_pyramid_cut_through_its_apex_where_it_stands_answers_half_of_it`
and `a_sound_pyramid_placed_and_cut_through_its_apex_cannot_order_two_points_by_the_apex`,
which replace §294.17's
`a_sound_pyramid_cut_through_its_apex_cannot_tell_which_side_of_the_cut_its_apex_is_on`.
`mise exec -- just t -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'` runs the box's
three pins, the pyramid's five and the half bar's five: 13 tests run, 13
passed.

**What this leaves.**

- The pyramid placed and cut through its apex refusing "order along a
  crossing", above. A row, tier 1, `refuses/B`.

### 294.20 Q1911: placed, the apex is put in order along each edge from its planes, and a speck of the cut is left — 2026-10-13

*Queue row Q1911, from §294.19. Lane
`lane-q1911-a-placed-pyramid-cut-through-its-apex`, base `8fac3497`, carried
on by a second and a third lane (§416). Code changed in `fuse/arrange.rs`;
pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`.*

**The order, decided.** `Arrangement::as_the_planes_say` put two nodes on the
line where the face's plane `S` and a partner plane `Q` meet in order only
when each was the meet of `S`, `Q` and one plane more, and it read a corner's
planes from the ones round it, so a corner with four or more was never asked.
It now reads the three bake solved a corner from (`BakedCorner::solved_from`).
For a corner with exactly three round it those are the three, and a corner
read out of the model has none, so both are asked as before. When the three
hold `S` and `Q`, the corner is on the line exactly and is the meet with the
third. When they do not, as with the apex solved from planes 0, 1 and 2 on the
edge where planes 0 and 3 meet, the corner is a rounding off the line.
`where_a_corner_off_a_line_comes_along_it` then puts it before the meet of
`S`, `Q` and `R` exactly when the sign of `R` at the corner
(`side_of_a_plane_where_three_planes_meet`) and the sign of
`det(n_S, n_Q, n_R)` differ. That is the rule
`which_meet_comes_first_along_a_line` follows for a point on the line, and the
side it reads is the side `passes_through` reads. So the face that puts the
apex on one side of the cut, and the face that puts it in order along where
the cut crosses an edge, cannot disagree.

Two cuts by a corner off a line are each put in order against it by their own
plane, so the two orders need not agree with the order of the two cuts.
`nodes_along` proves only neighbouring pairs. Here there is one cut.

`as_the_planes_say` is asked only where the boxes cannot say, so only an
order that was undecided before can change. Every `rcad-topo` test and the
slow tier pass.

**What comes back placed.** Shifted, turned, and turned then shifted, − and ∩,
under `certified` and under `tolerant(100)` alike, refuse `Undecidable` with
one record, "a point inside a piece of a face", and leave the model as it was.
Every record's points are within `1e-12` of the placed apex. No order is left
for `tolerant(100)` to fold. A print put in `select::choose` and taken out
found the piece: piece 31 of the block's face through the apex, of 53 pieces
shifted and 51 turned, a triangle whose corners lie within `1e-14` of the
apex. Placed, the four side planes cross that face in four lines that pass a
rounding from one another by the apex, so they cut it into specks a rounding
across, and no point inside this one can be written down. Where it stands,
the pyramid still answers 60 under both policies.

**Whether it is a limit.** Not settled. The answer, half the pyramid, 60, has
nothing small in it, and the speck is a rounding across. What was tried is the
exact order above. What was not: folding the specks of the cut's face in its
arrangement, placing the speck by a side it shares with a side face of the
pyramid in `select`, or taking the apex as on the cut under `tolerant` with
`Policy::take_as_one_point`. The last changes what the side faces are cut
into, not the cut's face, where the speck is. So it stays `refuses/B`.

The pin: `a_sound_pyramid_placed_and_cut_through_its_apex_cannot_place_a_speck_of_the_cut`,
which replaces §294.19's
`a_sound_pyramid_placed_and_cut_through_its_apex_cannot_order_two_points_by_the_apex`.
`mise exec -- just t -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'`: 13 tests run,
13 passed.

**What this leaves.**

- The pyramid placed and cut through its apex refusing "a point inside a piece
  of a face" on a speck of the cut's face, above. A row, tier 1, `refuses/B`.

### 294.21 Q1936: placed, each speck of the cut is placed by the planes, and the half pyramid keeps an edge a rounding long — 2026-10-13

*Queue row Q1936, from §294.20. Lane
`lane-q1936-a-pyramid-cut-through-its-apex`, base `743fbe6f`, carried on by
a second lane (§416). Code changed in `fuse/select.rs`; pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`.*

**Why the speck had no placement.** A print put in `select::choose` and taken
out found the piece the fuse gave up on at every one of the twelve fuses:
piece 31 of the block's face shifted, of 53, a triangle bounded by the `−x`,
`+y` and `+x` side planes with every corner within `5e−15` of the apex. On
the block's face the `±y` side planes cross it in two parallel lines a
rounding apart, and the `±x` planes in two lines that cross above both, so
the speck lies above the `+y` line, outside the pyramid. Naming the sides of
the pieces folds two corners of the speck into one, and the one side left is
shared with two other pieces of the block's face and with no piece of the
pyramid's: the `+y` face is not cut by the block's plane at all. So
`SidesItShares` cannot place it. `by_a_plane` can, by the `+y` plane, which
has every corner of the pyramid on or behind it and every corner of the
speck on or in front, one strictly. But it read each corner of the pyramid as
the meet of the planes round it, from `BakedCorner::around`, and with four
round the apex it left the whole pyramid out.

**The change.** `the_planes_meeting_at_each_corner_of_a_solid` reads the
three bake solved each corner from, `BakedCorner::solved_from`. With exactly
three planes round a corner bake has only that triple to solve from, so it is
the same three. A corner with fewer is read out of the model, its
`solved_from` is `None`, and `around` with fewer than three gave `None` too.
So only a solid with four or more planes round a corner is asked where it was
not before. Reading the solved three is sound for the same reason as in the
arrangement: `passes_through` and `as_the_planes_say` ask those three at that
corner (§294.19, §294.20), so the pyramid the pieces were cut against has its
apex at their meet, and `by_a_plane`'s hull argument is about that pyramid.
It changes nothing of a solid with a curved face, which `by_a_plane` leaves
out before it reads a corner: Q1748's bars are round, and every test in
`fuses_that_touch.rs` passes.

**What comes back placed.** Every speck is now placed, and what is built is
half of a pyramid whose apex is a rounding off the cut, which is where bake
put it. The pin counts each answer's faces and the sides of their outlines.

| | − shifted | − turned | − turned, shifted | ∩ shifted | ∩ turned | ∩ turned, shifted |
|---|---|---|---|---|---|---|
| `certified` | range | range | range | range | range | range |
| `tolerant(100)` | 60, 5 and 18 | 60, 6 and 22 | broken | broken | 60, 5 and 18 | 60, 5 and 18 |

"range" is `Invalid::DynamicRangeTooWide`, a ratio of `5.1e15` to `1.5e16`,
which `Assembly::check_dynamic_range` raises on the answer's shortest edge
before anything is written. "60" is `Ok`, one solid holding 60 to `1e−12`
relative, which validates, with one record in the log: the same range check,
which `tolerant` takes. "broken" is `Error::TheFuseWroteABrokenAnswer` with
`OuterWireWindsAgainstTheNormal`, which the sweep of the throwaway model
raises, so nothing is written. Each refusal leaves the model byte for byte as
it was. Where it stands, the pyramid answers 60 under both policies as
before, with nothing in the log, 5 faces and 16 sides: the half base, the
triangle of the cut, the two halves of the side faces it cuts and the whole
side face it does not.

The counts are the two halves of a pyramid whose apex is just off the cut.
The half without the apex has 18 sides: the cut is a four-sided face, its
top edge a rounding long where the side face it runs under crosses it, and
that side face keeps four sides below it. The half with the apex has 22:
the two side faces the cut crosses keep four sides each, and the side face
the cut runs under keeps a triangle a rounding across above it, a sixth
face. Of the three placed halves that hold the apex, one a placement, the
one turned for the difference answers, and the other two are the broken
fuses. A print put in `validate::check_flat_face` and taken out found the face whose
outline winds against its normal in each is that triangle. Shifted, for the
intersection, its corners are `(7.000000000000001, −5, 12.499999999999996)`,
`(6.999999999999999, …)` and `(7.000000000000002, …)`, on one line, and it
winds `Zero`. Turned and shifted, for the difference, they are within
`4.4e−15` of one another and it winds `Neg`. Turned, for the difference, the
same triangle comes out the right way round, and the answer validates.

**Whether it is a limit.** For the numbers as placed, the fuse's answer is
the right one. The three planes bake solved the apex from put it a rounding
off the cut, and the half pyramids above are what a pyramid with its apex
there holds: an edge a rounding long, and on the side holding the apex a
triangle a rounding across. Neither fits beside coordinates near 10 in an
`f64`. `DynamicRangeTooWide` is a limit of that answer by `LimitOrGap`, and
a triangle whose three corners are a rounding apart has no winding its
stored corners can carry, so `tolerant` taking the range check meets the
same limit one step on. Half the pyramid with nothing small in it is the
answer of the shape the designer meant, with the cut through the apex, and
the placed numbers do not say that. What would say it is how the two were
built: one movement placed both solids, and before it the cut ran through
the apex exactly. Q1954 traces that kind of fact, carried from how a shape
was built, for Q1936 among other rows. So folding the specks where the cut
meets two side planes within a rounding of the apex, which this section's
first draft named as the next thing to try, takes them as near enough: a
tolerance, which the theory Q1954 measures rules out. The next step is
Q1954's trace, and then carrying the coincidence if it finds it in hand. It
stays `refuses/B`: a shape designers make.

The pin: `a_sound_pyramid_placed_and_cut_through_its_apex_keeps_a_speck_of_the_cut`,
which replaces §294.20's
`a_sound_pyramid_placed_and_cut_through_its_apex_cannot_place_a_speck_of_the_cut`.
`mise exec -- just t -E
'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'`: 13 tests run,
13 passed.

**What this leaves.**

- The pyramid placed and cut through its apex keeping an edge a rounding
  long, refused by `certified` and twice by `tolerant(100)`, above. A row,
  tier 1, `refuses/B`, after Q1954: carry the fact that one movement placed
  both solids and the cut ran through the apex before it, if Q1954 finds it
  in hand, so the cut meets the apex.

### 294.22 Q1839: the tetrahedron's front face is stored a rounding off its apex, and each refusal is the answer of the numbers it is handed — 2026-10-15

*Queue row Q1839, from §294.17. Lane
`lane-q1839-a-tetrahedron-cut-through-its-apex`, base `872c1b1c`, carried on
by three more lanes (§416). No code changed: a trace put in
`fuse/arrange.rs` was taken out. Pinned in
`crates/rcad-topo/tests/a_tetrahedron_cut_through_its_apex.rs`. The exact
sums below were worked in Python's `Fraction` over the stored numbers, by a
scratch script that is not kept.*

**Why §294.17 called it a gap.** It read "its answer has no small feature"
from the shape as drawn. The block's face `y = 0` runs exactly through the
apex `(0, 0, 10)`, so the cut and the three side planes meet at one point
and nothing is small. That is true of the shape drawn. It is not true of the
planes the build stored.

**The front face misses the apex.** Each face of the tetrahedron gets its
plane from `Plane::through_three_points`. The face facing `−y` runs along
`(0, −10, 3)` scaled to length one, stored as `[0, −0.9578262852211514,
0.28734788556634544, −2.8734788556634543]`. `10 × 0.28734788556634544` is
not an `f64`, so at `(0, 0, 10)` the stored plane is `1.1e−16`, and
`side_of_point` proves it `Pos`. The two other side planes are `Zero`
there. In exact sums over the stored numbers, the three side planes meet at
`A = (0, 5.8e−17, 10 − 1.9e−16)`, on the block's side of the cut. The front
face's two slanted edges cross the cut at `P± = (±5.8e−17, 0, 10 − 3.9e−16)`.
So in the tetrahedron the kernel is handed, the apex is inside the block. Its
intersection with the block keeps a triangle `A P+ P−` of the front face,
with sides `2.1e−16` and `1.2e−16`. Its difference keeps a four-sided cut,
whose top edge `P+ P−` is `1.2e−16` long. Neither fits beside coordinates
near 10 in an `f64`.

**What the fuse made of it, where it was built.** With the range check at
`Warn`, the intersection answered with the `+x` face's outline
`(0, 2.458e−16, 10) → (0, 0, 10) → (1.5, 0, 0) → (0, 3, 0)`. `P+` and `P−`
have one box middle, `(0, 0, 10)`, so `Corners::of` makes them one corner
(§365.7). The triangle `A P+ P−` then has two sides on one edge, and it is
left out. That edge runs from `A`, written at its box middle, to
`(0, 0, 10)`, and is `2.458e−16` long. `11 / 2.458e−16` is the refusal's
`4.47e16`. In the difference, `P+ P−` is the cut's top edge. Both of its ends
are written as `(0, 0, 10)`, so it is left out as §365.7 leaves such a side,
and the difference answers 45 with nothing in the log.

**Placed, the placement rounds each plane again.** Where the three side
planes meet is off the cut at every placement:
`side_of_a_plane_where_three_planes_meet` puts it inside the block where it
was built and shifted, and outside turned and turned then shifted. The half
that holds `A` is refused at every placement. The other half answers when
`P+` and `P−` are written as one triple. Shifted, they are not: both halves
are refused at `1.5199648742375424e16`, which is `13.5 × 2⁵⁰`. That is the
shifted block's top, `z = 13.5`, over an edge `2⁻⁵⁰` long, one `f64` step
at `x = 7`, where 448.5's trace found a speck at the shifted apex.

**The front face's rounding alone does not make the placed refusals.** A
front face can be built through the apex exactly:
`Plane::from_point_and_normal_as_written` through the apex along
`(0, −10K, 3K)`, with `K = 0.09578262852211505`. That is `1/√109` rounded to
a multiple of `2⁻⁵⁰`, so `10K`, `3K` and the offset `30K` are each an `f64`.
The plane holds the apex and `(±3, −3, 0)` exactly, and `109K²` is `8ε` short
of one, inside `Plane::NORMAL_LENGTH_ROUNDOFF`. Where it was built, the four
planes meet at one point, and both halves answer under `certified` with
nothing in the log. Placed first, its three side planes meet on the same
side of the cut as before, at all three placements. It is refused at the
same four fuses as the front face through three points: both halves
shifted, and the difference turned and turned then shifted. So those four
are the placement's rounding, which is §448.7's limit. The kernel is handed
the composed numbers, and their answer holds an edge a rounding long.

This sharpens 448.5's count. The front face's rounding, (a), is the whole of
the unplaced intersection: built through the apex exactly, it answers. The
placement's rounding, (c), is enough for the three placed differences: they
are refused with the exact front face too. The shifted intersection, which
448.5 gave to (a), is refused unless both are taken away. With the exact
front face placed first it is still refused. With the front face through
three points, fused where it was built and then shifted, it is the unplaced
intersection, which is refused.

**Whether it is a limit.** For the numbers the kernel is handed, all five
are. Where it was built, the stored front plane puts the apex inside the
block, and the tetrahedron's intersection with the block holds a triangle
`1e−16` across. The answer `certified` would have to write does not fit an
`f64` beside 10, so `DynamicRangeTooWide` is a limit by `LimitOrGap`.
Placed, §448.7 settled it for the pyramid, and the exact front face shows
the tetrahedron's placed refusals do not need the front face's rounding.
The answer the designer meant, with nothing small in it, is the answer of
other numbers: a front face through the apex, and the cut made before the
placement. There is no gap to close, so the row closes on this
demonstration and no fix is made.

**The road a caller has.** Build the front face through the apex exactly,
fuse the two where they were built, and place the answer. All eight fuses
answer under `certified`, hold 45 and 15 to `1e−12` relative, validate, and
leave nothing in the log. That is §448.7's road with one step more, the front
face, and only the caller can take it: it knows the face was drawn through
the apex, and the plane it handed in says otherwise.

**Under `tolerant(100)`** all eight answer, each with one record, `three
meets of one plane taken as an input corner`.
`fold_each_corner_one_plane_cuts_a_rounding_from_it` takes the points where
the cut meets two of the side planes, a rounding from the apex, as the apex.

**The pins**, in
`crates/rcad-topo/tests/a_tetrahedron_cut_through_its_apex.rs`:

- `the_front_face_through_three_points_is_stored_a_rounding_off_the_apex`,
  and the exact front face `Zero` at the apex and at `(±3, −3, 0)`.
- `where_the_three_side_planes_meet_is_off_the_cut_at_every_placement`, for
  both front faces.
- The 16 fuses by name: `unplaced_`, `shifted_`, `turned_` or
  `turned_then_shifted_`, then `less_the_block_` or `and_the_block_`, then
  `answers_45_`, `answers_15_` or `is_refused_`, then `under_certified` or
  `under_tolerant`. Each refusal is pinned to its ratio within `1e−9`.
- `a_front_face_through_the_apex_fused_where_built_and_then_placed_answers_at_every_placement`,
  the road.
- `a_front_face_through_the_apex_placed_first_is_refused_where_the_placement_rounds`.

`mise exec -- just t -p rcad-topo -E
'test(/^a_tetrahedron_cut_through_its_apex::/)'`: 20 tests run, 20 passed.

**What this leaves.** No row.

- `Plane::through_three_points` storing a plane that misses one of its own
  three points by a rounding. Not filed. It refuses nothing: the refusals
  above are the range check reading an answer the stored planes really
  have. A plane that holds the three points exactly exists for this face,
  by the search for `K`. Whether one exists for any three points was not
  tried. Building it is a feature, and no consumer in `docs/consumers.md`
  asks for it, so §13 defers it.
- The four placed refusals. Not filed: carrying a movement two solids share
  is §448.7's tier 4 row, parked until `docs/consumers.md` names a caller
  that needs it.

### 294.23 Q913: the turned seam is a gap, closed; turned then shifted is a limit — 2026-10-15

*Queue row Q913, from Q717 and §294.9 (Q912). Lane
`lane-q913-a-record-q717-in-294-8`. No code changed. Pinned in
`crates/rcad-topo/tests/a_turned_and_shifted_cut_along_a_seam_misses_it_by_a_rounding.rs`,
straight from the public `Plane`, `Cylinder` and `Isometry` API and
`rcad_predicates::expansion`, without a model or a fuse.*

**Q717's measurement.** §294.9 left one question open at its close: the
turned-then-shifted bar cut along its seam still refuses `Undecidable` "side
of plane" at the bottom seam corner's box, `2.8421709430404007e−13` either
way round `(10, −5, 2.5)`, and whether that is a limit was Q913's to pin.
Q717 (dropped, its measurement kept in its own text and in Q912's) ran the
probe again on the base (`target/q717/q717_probe.rs`, copied in and run with
`Q717_RULE` unset): turned alone, the moved pipe's seam plane and
`THE_HALF_PAST_THE_SEAM`'s face `y = 0` are the same four numbers,
`(0, −0.9210609940028851, −0.38941834230865047, 0)`; turned and shifted, the
block's face holds `3.631759114242799` where the seam plane holds
`3.6317591142427994` — one ulp under, `0x400d0dd7b8f4fb64` against
`0x400d0dd7b8f4fb65` — and the block's placement composed from `(7, −5,
2.5)`, the turn and `(−5, 0, −1)` rounds to the translation `(2,
−4.610581657691349, 1.578939005997115)`, which is where that ulp comes from.

**Turned: a gap, and it is already closed.** The two planes are one plane,
to the bit, wherever the move is only a turn — pinned again here
(`turned_the_seam_plane_is_the_blocks_face_bit_for_bit`), straight from
`the_bars_seam_plane()` (the plane through the bar's axis and its seam
corner, built the way the row's own figures name it:
`(1, 0, 0)` crossed with `(0, 0, 1)`) and `THE_HALF_PAST_THE_SEAM`'s own
face, each `.transformed` by the turn (and, for the block, the turn
composed with the extrusion's own offset). §294.9's rule —
`BakedCorner::seam_plane`, taken as through the corner without asking,
because it is the moved copy of an incidence the model held — answers this
half already. There was a gap here, between Q419 (refused) and Q912 (closed
it); nothing is left open by it.

**Turned then shifted: a limit.** Composing the shift in moves the two
planes' offsets a rounding apart, pinned
(`turned_and_shifted_the_seam_plane_misses_the_blocks_face_by_one_ulp`): the
normals still agree bit for bit (the rotation part of the two placements is
the same, so the one ulp is the composed translation's alone), and the
offsets differ by exactly one ulp, `seam.offset().to_bits() −
face.offset().to_bits() == 1`. §294.9's rule reaches a seam corner's own
plane exactly; it does not reach a plane that misses the corner, however
little, and this one does. So `BakedCorner::seam_plane` cannot be widened to
cover it without taking a crossing as through a corner it is a rounding off
— the same mistake §294.9 itself warns against for `around` (§356).

**Why there is no answer to miss.** The exact miss, `n · p − d` for the
block's face at the moved corner, computed as the sum of an expansion
(`rcad_predicates::expansion`, never dividing) and rounded once at the end,
is `3.885780586188048e−16`
(`turned_and_shifted_the_miss_is_a_limit_the_scale_already_rules_out`) — the
same figure Q717 and Q912 both measured. The placed bar's top corner reaches
`z = 11.710609940028851` there, so holding the seam corner and the true
crossing apart needs a spread of `11.710609940028851 /
3.885780586188048e−16`, about `3.0137e16`. §3.5's own validity ceiling is
`1e11`; a feature written straight into the coordinates is gone by a spread
of about `1e16` (`what_the_dynamic_range_check_is_a_margin_against.rs`,
§3.5). `3.0137e16` is past both: about five and a half orders of magnitude
past the `1e11` ceiling, and about half an order of magnitude past the
`1e16` a feature is already gone by.
The corner's own box, `2.8421709430404007e−13` either way round, already
reaches about 731 times past the `3.9e−16` that separates it from the true
crossing, so no tightening of the box finds the crossing either: the gap the
predicate would have to resolve is smaller than what the placed scene's own
scale leaves an `f64` able to hold, not smaller than what today's arithmetic
reaches. By `LimitOrGap`, this is a limit.

**Q419's guess answers neither half.** Handing on the exact moved point when
the move rounded nothing for it does not reach either refusal — run by
Q717, recorded in Q912's own text (merged, `8ac8cbf2`) and not rerun here,
since re-running it needs the rule swapped in `moved_corner.rs`, outside
this row's write set. Turned alone, `(3, 0, 0)` is then exact, and the fuse
refuses "side of plane" at the *top* seam corner's box instead, round `(3,
−3.894183423086505, 9.210609940028851)` — matching the top corner moved by
the turn alone, reproduced again here with Q717's probe copied in and run
(`mise exec -- just t -p rcad-topo --test q717_probe --no-capture`, not
kept): `top corner moved [3.0, -3.894183423086505, 9.210609940028851]`.
Turned and shifted, the guess changes which refusal is hit — "order along a
crossing" rather than "side of plane" — not whether one is.
Neither guess reaches a point both halves of the cut agree is the corner.

### 294.24 Q2043: the half bar cut again flush along its flat face, turned then shifted, is a gap, and closed — 2026-10-17

*Queue row Q2043, was Q2034. Lane
`lane-q2043-a-half-bar-turned-and-shifted-cut-flush`, carried on by three
relays to `a6256de8`, and `lane-q2043-b-finish-the-half-bar-cut-flush`, which
merged it at `de5e987b` off `5b5532db` and added the unit tests. Pinned in
`crates/rcad-topo/tests/a_broken_three_surface_corner_read_back_from_bytes.rs`.*

**What refused.** The sound half bar, `add_round_bar(3, 10)` less
`THE_HALF_PAST_THE_SEAM` under `certified` and then placed, fused again with
that same block placed the same way, flush along its flat face `y = 0`.
Turned 0.4 about `x` and then shifted by `(7, −5, 2.5)`, the difference and
the intersection were both refused: under `certified`, `Undecidable` "side of
plane" at a corner's box round `(10, −5, 2.5)`, `3.55e−15` across, and under
`tolerant(100)`, `TwoCurvesTouchWithoutCrossing`. Every other placement
answered. The base's source run with this branch's test file prints, for
`a_sound_half_bar_cut_flush_along_its_flat_face_answers_in_every_placement`,
`Difference refused: Undecidable { … predicate: "side of plane" … enclosure:
3.552713678800501e-15 … }`.

**A gap, and not §294.23's limit.** §294.23 measured the same two offsets.
Turned and shifted, the block's face holds `3.631759114242799`
(`0x400d0dd7b8f4fb64`) where the bar's seam plane holds `3.6317591142427994`
(`0x400d0dd7b8f4fb65`), one ulp apart, with the normals equal bit for bit.
There the whole bar is cut, so the block's face crosses the bar's wall a
rounding from the seam corner, and the answer would hold an edge `3.9e−16`
long beside coordinates near 11. That does not fit an `f64`, so it is a
limit. Here the bar was cut by that face where it was built, so its flat face
is the block's face moved by the bar's placement. It is stored as
`0xc00d0dd7b8f4fb65`, which is `3.6317591142427994` along the block's normal
turned round. The block's own placement is composed with its shift
`(−5, 0, −1)` and rounds one ulp lower (§294.23). Along the flat face's
outward normal, the bar's material is `n · x < −3.6317591142427994` and the
block's `n · x > −3.631759114242799`. The two solids are one ulp apart and do
not touch. The answer is the whole half bar for the difference and nothing
for the intersection. Nothing in it is small, and it is the answer of the
numbers the kernel is handed, so the refusal is a gap (§16.10).

**Three rules, each reading two offsets instead of a box.** Each place that
refused asked a box both planes run through:

1. **The side of a plane parallel to one a corner is on.**
   `Arrangement::side_of_a_plane_parallel_to_one_it_is_on`, in
   `fuse/arrange.rs`, is asked first in `passes_through`'s `None =>` arm, and
   only of the three surfaces `bake` solved the corner from
   (`BakedCorner::solved_from`). A corner of the flat face was solved from
   the cap's plane, the wall and the flat face, and the flat face's normal is
   the block's face's turned round, number for number. Every point of a plane
   is on one side of a plane parallel to it, so the corner's side of the
   block's face is which of the two offsets is the larger.
2. **The order of two points on a circle, by two parallel cuts.**
   `Arrangement::as_two_parallel_cuts_through_the_loop_say` is the last
   fallback in `order_along`. Circles on the bar's wall meet the flat face and
   the block's face in pairs of points a rounding apart. Across the two
   planes a circle climbs along one half and falls along the other, so which
   of a pair comes first round it is the sign of the offsets' difference
   against the way the circle runs at the two points.
3. **The one face a box meets.** `by_the_one_face_its_box_meets`, in
   `fuse/select.rs`, is called by `settle_against_each` where the ray answers
   `OnBoundary` or `Undecidable`. A point found inside the piece of the
   block's face across from the flat face sits in a box `4e−15` across both
   planes, which no ray can place. Where the bar's flat face is the only face
   of the bar that the box meets, and that face holds the box, the piece is
   inside the bar where its offset is behind the face's and outside where it
   is in front.

**Each compares two `f64` offsets exactly, with no tolerance.** Each asks
first that one plane's normal is the other's, or the other's turned round,
component for component with `==`, and then compares two stored offsets.
Nothing is divided and nothing is rounded between the stored numbers and the
answer. Where the normals differ at all, each answers `None` and the old path
asks the box as before. The order rule also asks which way the circle runs at
each of the two points, and takes that only where interval arithmetic proves
one sign at both. The one-face rule asks the faces' own tests
(`face_holds_the_point`, `side_of_a_flat_face`, `side_of_surface`) under the
policy, as before, and lets them take nothing more.

**The row's lead was not §294.9's seam plane.** Q2043 guessed that the "side
of plane" might be the sound bar's refusal of §294.9, beside Q1398. It is
not. `BakedCorner::seam_plane` answers a seam corner against its own moved
seam plane, and the partner here is the block's face, one ulp off that plane
(§294.23). What places the corner is a plane among the three it was solved
from, the flat face, which is parallel to the partner.

**Which rule each test needs.** Each rule was taken out on its own, with the
other two left in, and the test file run again (`mise exec -- just t -p
rcad-topo --test merged -E 'test(/^a_broken_three_surface_corner_read_back_from_bytes::/)'`):

- Without the side rule, 12 of 14 pass.
  `a_sound_half_bar_cut_flush_along_its_flat_face_answers_in_every_placement`
  refuses "side of plane" (enclosure `3.55e−15`) under `certified`, and
  `TwoCurvesTouchWithoutCrossing` with the pin cut to `tolerant(100)` alone.
  `a_half_bar_whose_corner_is_moved_onto_the_other_end_of_its_cut_answers_no_fuse`
  refuses "side of plane", turned then shifted under `certified`, where it
  now expects "order along a crossing".
- Without the order rule, 13 of 14 pass. The flush pin refuses "order along a
  crossing" (enclosure `6.6e−15`) under `certified`, and
  `TwoCurvesTouchWithoutCrossing` under `tolerant(100)` alone.
- Without the one-face rule, 13 of 14 pass. The flush pin refuses "a point in
  a piece that every solid could place", and under `tolerant(100)` alone the
  same.

So each rule is needed, under both policies. With the base's source, the
flush pin fails on "side of plane", and the moved-corner pin fails with
`left: Undecidable(["side of plane"])` against `right:
Undecidable(["order along a crossing"])`.

The unit tests: `a_flat_piece_a_step_off_a_blocks_floor_is_placed_by_the_floor`
and `a_box_a_side_leaning_over_a_wall_runs_through_is_not_placed_by_the_wall`
in `fuse/select.rs` pin the one-face rule, the second on a wedge whose long
side leans over its wall and runs through the box, where the rule must
answer `None`.
`a_corner_solved_from_a_plane_parallel_to_the_partner_is_where_the_offsets_say`
in `fuse/arrange.rs` pins the side rule with Q2043's own offsets. The order
rule has no unit test of its own: it needs a whole arrangement with a
crossing round a closed curve and baked corners, and the flush pin is what
fails without it.

**How §39.4 bears.** Its table has no row for "side of plane". It files
"order along a crossing" (the turned notched bar) and the refusals to place a
point in a piece as gaps, which agrees with this. The row that bears is
"`Undecidable` out of `face_holds_the_point`", filed a **limit** where the
box straddles the surface, and never demonstrated ("no generator reaches
either"). Q2043's boxes each straddle a surface, and the answer exists: the
point the box stands for lies on a plane parallel to the surface, one ulp
from it, so its side is known though the box cannot say it. A box across a
surface is a limit only where nothing but the box says where the point is.
§39.4's file is outside this row's write set, so a row adds the note there.

**What this leaves.** A small row to add that note under §39.4's
`face_holds_the_point` row. Nothing else: no new defect was found.

### 294.25 Q2587: a plane steeper than a cone crosses it in nothing closed, and that is said before a branch is named — 2026-10-21

*Queue row Q2587, from Q2585's trace. Lane
`lane-q2587-no-branch-for-a-plane-steeper-than-a-cone`, base `e4565b86`,
carried on by one relay. Pinned in `crates/rcad-geom/src/walk.rs`'s tests and
in `crates/rcad-topo/tests/a_broken_two_surface_corner_read_back_from_bytes.rs`.
Answers Q2396 and Q1398.*

**What refused.** §294.13's shape: `cones::a_frustum(model, 0.5, 2, 6)` turned
0.4 about `x` and then shifted by `(7, −5, 2.5)`, less or intersected with
`THE_HALF_OF_THE_FRUSTUM_PAST_THE_SEAM` placed the same way. Once Q2374 read
the bottom seam corner's side of the block's face off the two offsets, the
sound frustum and the corner moved a quarter turn round refused under
`certified`, both rules: `Undecidable` with two records, each `side of a
half-space` asked of one point, in a box under `5e−13` across that holds where
the apex lands, `(7, −5, 2.5)`, and that the block's face straddles.
`tolerant(100)` answered with half the frustum, `26π/3` (Q2396). The base's pin
of the refusal,
`a_turned_and_shifted_frustum_cut_along_its_seam_cannot_tell_which_side_of_a_half_space_its_apex_is`,
asserts each of those, and passes there: `mise exec -- just t
a_turned_and_shifted_frustum_cut_along_its_seam_cannot_tell_which_side_of_a_half_space_its_apex_is`
prints `1 test run: 1 passed, 4513 skipped`.

**Where the two questions came from** (Q2585's trace). The block's face is
steeper than the cone and misses the apex by `5e−16` (§294.13), so it crosses
the cone in a hyperbola with one branch in each nappe. The frustum's wall is in
one nappe, and the branch in the other has no node in the fuse's arrangement
of the face.

1. `start_every_closed_crossing_that_has_no_node`, in `fuse/arrange.rs`, hands
   that branch — a run `intersect` named, with no ends — to
   `rcad_geom::where_a_closed_crossing_starts`, to find where to put a corner
   on it.
2. That names the run's shape before anything else.
   `where_a_plane_cuts_a_cone` takes its `leaning <= 0.0` arm and calls
   `which_branch_of_a_hyperbola_this_run_is`, which for a run with no ends asks
   the run's picker, the plane through the apex square to the axis, at each
   branch's vertex, through `every_half_space_holds`. Each vertex is about
   `1e−15` from the apex, so the picker runs through both boxes.
   `where_a_closed_crossing_starts` takes no policy and asks under `certified`
   whatever the fuse's policy is, so both questions are undecided, and both
   are logged into the fuse's log.
3. Naming the branch refuses, `CannotNameWhichPieceOfACrossingThisIs`. Had it
   named one, the answer would have been `TheCrossingOfTheseTwoSurfacesIsNotClosed`
   for either branch.
4. `arrange.rs` drops the refusal and goes on, but the two records stay in the
   log, and `a_fuse_logging_this_is_refused`, in `fuse/mod.rs`, refuses the
   finished answer on them under `certified`. A `tolerant(100)` budget covers
   them, which is why that policy answered.

**A gap.** The answer is there, `tolerant(100)` reached it, and the two
questions that stopped `certified` could not change it: no branch of a
hyperbola is closed, so which branch a run is says nothing about whether it
closes.

**The fix: ask how the plane leans first, and stop there.**
`where_a_closed_crossing_starts` now asks `a_plane_at_least_as_steep_as_a_cone`
of its pair before anything else, and where that says yes it answers
`TheCrossingOfTheseTwoSurfacesIsNotClosed` without naming a branch. That is yes
for a plane and a cone, in either order, where `intersect`'s
`how_a_plane_leans_against_a_cone` is `Neg` or `Zero`: `E ≤ 0`, proved, with no
policy and nothing to log.

**Why it changes no answer but these refusals.** The lean's exact arm sums
`(n·w)² − Σ(slope·(n × w)_j)²`, and `where_a_plane_cuts_a_cone`'s `leaning`
sums `(n·w)² − slope²‖n × w‖²`, both exactly and from the same stored numbers.
They are one real number, and `leaning` is the `f64` nearest it. The `f64`
nearest a number at or below zero is at or below zero, so every pair the new
check catches would have taken the `leaning <= 0.0` arm. Every way out of that
arm came back to `where_a_closed_crossing_starts` as
`TheCrossingOfTheseTwoSurfacesIsNotClosed` — straight lines, a run stated as
closed, a parabola, a branch of a hyperbola — or as a refusal: one from naming
the branch, or `CannotWorkOutHowBigACrossingIs` just before the arm, where a
sum leaves `f64`'s range. Every caller, five in `fuse/arrange.rs` and one in
`fuse/assemble.rs`, treats every `Err` alike, so all a fuse sees change is that
nothing is logged. A pair whose `E` the lean cannot prove takes the old road.

**Not by dropping the records.** The row ruled out the other way to make this
pass, dropping the records `arrange.rs` keeps when a start is refused. Those
records are what is left of a question the fuse could not answer about a
crossing it then leaves without a corner, and for a closed crossing that is
one the answer may be missing; Q2588 asks whether it is. This fix asks nothing
where nothing could change the answer, and leaves a start that did ask as it
was.

**What it gives.** Under both policies and both rules the sound frustum gives
half the frustum, `26π/3` on a fine mesh, its whole model validates, and its
corners are the turned frustum's shifted, to a rounding. The corner moved a
quarter turn round is cut away and gives the sound answer to a rounding. The
corners moved off the wall and off the cap refuse `ResultIsNotManifold`, as
they did. One helper asserts all of it for both policies:
`a_turned_and_shifted_frustum_cut_along_its_seam_gives_what_the_turned_frustum_gives_shifted`,
which also asks `where_a_closed_crossing_starts` of both branches under both
policies and gets `NotClosed` and an empty log, calls it under `certified`, and
`under_a_tolerant_policy_a_turned_and_shifted_frustum_cut_along_its_seam_answers_as_the_bar_does`
under `tolerant(100)`. `mise exec -- just t -p rcad-topo --test merged -E
'test(/^a_broken_two_surface_corner_read_back_from_bytes::/)'` prints `19 tests run: 19
passed, 0 skipped`. In `rcad-geom`,
`a_plane_a_rounding_off_a_cones_apex_and_steeper_than_it_is_not_closed_and_asks_nothing`
pins the door itself: a cone with its apex at `(7, −5, 2.5)`, its axis `z` and
slope 0.5, and the plane `x = 7.0.next_up()`. For both runs `intersect` names
and both orders of the pair, the log is empty and the answer `NotClosed`.
Without the fix the log holds `side of a half-space` records.

**Q2396 and Q1398.** Q2396 asked which half-space is asked of the apex and
whether the answer is there: the picker through the apex, asked by the namer
of a branch whose name did not matter, and the answer is there. Q1398 asked
whether the rule that cuts the seam corner away has an answer `certified` can
reach, for the bar and the frustum each turned and then shifted. Both rules do
for both: the bar since Q2374
(`a_turned_and_shifted_bar_cut_along_its_seam_gives_what_the_turned_bar_gives_shifted`),
the frustum since this.

**Where it stops.** The check is in `where_a_closed_crossing_starts` alone.
`walk` still names a hyperbola's branch for a run with no ends by asking its
pickers, as it must to walk one branch; nothing here walks such a run. That
`where_a_closed_crossing_starts` takes no policy is a gap its doc records, and
it stands.

**What this leaves.** Nothing new. Q2588, already filed, asks whether a closed
crossing whose start is refused can make a tolerant fuse answer wrong.
