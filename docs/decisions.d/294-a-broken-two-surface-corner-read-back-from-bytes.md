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
  tried with that block are `Undecidable` on "a point inside a piece of a
  face". The sound frustum is `Undecidable` on that same predicate against a
  block over the rim by its seam, and so is every moved corner there, under
  both policies
  (`a_frustum_whose_top_corner_is_moved_in_its_file_reads_back_and_does_not_validate`,
  `a_frustum_whose_top_corner_is_moved_gives_what_the_bar_gives_and_one_refusal_more`).
