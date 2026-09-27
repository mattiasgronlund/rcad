## 301. What pins a corner's room from below: no shape found needs it, so the function is pinned by hand — 2026-09-27

*Queue row Q225, from §288. Lane `lane-q225-a-corners-room-from-below`, off
`b6c52df2`. The room `a_corner_is_this_place` grows a corner by is
`what_writing_the_corner_down_cost`, and §288 pinned it from above (x1.5
fails, x1.25 passes). **Nothing pinned it from below.** With the room halved
or cut to a tenth, every workspace test passes. At zero, two fail (301.1).
**The room is wider than any certified shape measured needs.** The fold asks
how far a meet's box sticks out past the corner's *own* box, not how wide the
meet is. Where `bake` solved the corner from the same three planes, the two
boxes hold one point and differ only by the order the solve took its rows in.
A turned wedge's meet box 4.5 rooms wide sticks out a fiftieth of the room
(301.3). Every certified fold in the workspace sticks out under 0.055 of the
room. The only folds past a tenth are 0.49, in two tests that walk the fuzz
corpus and check no answer (301.2). **No shape was found that fills the room,
and "no shape can" is not shown** (301.4). So the room is pinned at the
function, by a unit test with places written by hand: a place reaching exactly
the room folds and one a step further does not. The room x0.5, x0.1 and x1.25
each fail it (301.5). No kernel code changed.*

  301.1   The room shrunk to half, a tenth and nothing
  301.2   How far the workspace's folds stick out
  301.3   Why a wide meet is not a meet that fills the room
  301.4   The families tried, and where a shape could still be
  301.5   The two new tests and their mutations
  301.6   The probe
  301.7   What this leaves
  301.8   Which draws the 0.49 fold belongs to, and whether shrinking the room changes their answer

### 301.1 The room shrunk to half, a tenth and nothing

Each run was a scratch edit to the one line where `a_corner_is_this_place`
builds `allowed`, in `crates/rcad-topo/src/fuse/arrange.rs`:
`what_writing_the_corner_down_cost(node.point.enclosure()) * k`. That is where
§288 scaled it for its x1.5 and x1000 runs. `bake.rs`'s own use is untouched.
Each edit was undone before the next. The command was
`mise exec -- just t --workspace`, on `b6c52df2` with nothing else changed.

| k | Result |
|---|---|
| 0.5 | 2548 run, 2548 passed |
| 0.1 | 2548 run, 2548 passed |
| 0 | 1541 of 2548 run before the stop, 2 failed |

At zero the two failures are:

- `two_places_of_one_meet_inside_a_corners_room_are_neither_folded_into_it`
  (§288.5). With no room the second place is no longer inside it, so the
  corner's own place folds. That is the count condition moving, not the
  containment.
- `how_often_a_certified_cut_of_a_turned_curved_pair_refuses`. The turned
  family refused 868 of 2304 faces, 37.67%, against the three tenths §36
  records. A rate test, so a coarse pin, and only at zero.

So between a tenth of the room and all of it, nothing in the workspace held
the room before this lane.

### 301.2 How far the workspace's folds stick out

A probe (301.6) logged every call to `a_corner_is_this_place` that got past
its first gate, where the meet's box was not bit for bit the corner's. For
each it logged how far the meet's box sticks out past the corner's box, as a
fraction of the room. That is the smallest `k` at which the containment
condition still holds. The command was
`Q225_PROBE=<file> mise exec -- just t --workspace`, 2548 passed.

- 87 309 such calls. 87 305 folded. The other 4 stuck out more than ten rooms:
  one in §253's test and three in the two tests that walk the corpus, where
  §253's draw is. That they are that draw was not checked.
- Of the folds: 19 930 stick out not at all, 33 086 under 0.01, 34 210 from
  0.01 to 0.05, 58 from 0.05 to 0.1, none from 0.1 to 0.3, and 21 from 0.3
  to 0.5.
- The 21 are all 0.4923, in `a_fused_draw_hands_its_provenance_back_through_the_door`
  and `every_seed_in_the_checked_in_corpus_is_fused_without_panicking`. The
  corner is at the origin with a subnormal `z`, such as `2.98e-315`, and the
  meet's box is `[0, 6.996e-15]` in `z`. Neither test checks what the fuse
  answers, so a fold lost there fails nothing. Which draws these are was not
  found: the corner's bytes occur in hundreds of corpus files.
- The largest outside those two is 0.0547, in
  `a_wedge_cut_from_a_ball_by_two_knives_through_its_polar_axis_is_answered_or_refused_by_name`.
  The turned-pair suites sit between 0.02 and 0.035.

### 301.3 Why a wide meet is not a meet that fills the room

The row asked for a corner whose meet box reaches most of the room. **Reaching
is easy; sticking out is what the room is spent on.** The containment
condition asks whether every point of the meet's box lies inside the corner's
box grown by the room. So what counts is how far the meet's box sticks out
past the corner's box, not how wide it is. And the first gate needs the two
boxes to touch, so a meet box can never stick out further than its own width.

At a corner with three planes round it, `bake` solves the corner from those
three planes, and `place_the_meets` solves the meet from the same three.
Before the solve, `three_planes_faced_the_one_way` turns some planes round, so
the rows go in a different order and round a little differently. Both boxes
still hold the one true point, and they differ by that rounding alone.

The demonstration is a wedge: a triangle climbing at a slope from 1/16 to
1/512, extruded by one, turned about three awkward axes by six angles, united
with a cube well clear of it under `Policy::certified()`. Its sharp corners
are three planes crossing at a shallow angle. From the probe, per slope, over
every corner of every turn:

| Slope | Widest meet box, in rooms | Furthest it sticks out, in rooms |
|---|---|---|
| 1/16 | 0.35 | 0.019 |
| 1/64 | 0.62 | 0.023 |
| 1/128 | 1.14 | 0.016 |
| 1/256 | 2.27 | 0.011 |
| 1/512 | 4.47 | 0.016 |

Every one of those folds. One union refused at the base, slope 1/512 turned
2.5 about `(1, 2, 3)`, and it was not traced. The same 90 unions with the room
scaled, refusal counts from the same probe build reading the scale from an
environment variable: 1 at x1, x0.5, x0.1, x0.03 and x0.02; 5 at x0.01; 33 at
x0.001 and at zero. So a wedge pins the room at about a hundredth, however
wide its meets.

### 301.4 The families tried, and where a shape could still be

A meet sticks out far only where the corner's box was worked out from other
numbers than the meet's. That happens three ways:

- **A corner with fewer than three surfaces round it**, which `bake` reads out
  of the model and moves (§286). A face's crossings through it then come from
  one surface and from cuts.
- **A corner with four or more surfaces round it**, which `bake` solves from
  the best-conditioned triple while the face asks about its own.
- **A cut through the corner.** Under `Policy::certified` a cut counts as
  through the corner only when `side_of_surface` proves it, and that needs the
  corner to be a point, not a box. If the corner is also exactly on the face's
  other two surfaces, the meet is exactly the corner and the solve names it
  (§131), so nothing sticks out.

So a shape that fills the room needs the corner off one of its own surfaces by
the model's word, as the first two ways allow, and a meet box wide enough to
reach it. Tried here:

- **Thin turned wedges**, the third way's opposite: three planes, one triple,
  the widest meets. 216 certified unions over two runs, slopes 1 to 1/512. At
  most 0.031 of the room.
- **A wall through a bar's seam corner**, the first way plus a cut. The bar's
  bottom corner `(1, 0, 0)` has the cap and the pipe round it. A block's wall
  from `(1, 0)` climbing at a slope from 1 to `0.3 × 2⁻¹³` passes through it
  exactly, because the wall's offset is its normal's `x`. 84 unions, both
  policies. Every call the probe logged was at the two seam corners, so the
  fold was reached there. The meet's box was at most 0.15 of the room wide at
  any slope, however near the wall came to grazing the pipe, and it stuck out
  at most 0.078. Every union then refused elsewhere: the block's edge lies on
  the pipe, which is a knife edge and not this question.
- **Not tried: a corner with four planes round it**, such as a pyramid's apex
  built through `SolidBuilder` and turned. That is the second way and the one
  most likely to fill the room. It is left for a row (301.7).

**This is not a demonstration that no shape can.** It measures one family of
each of two of the three ways, and it names the third. The workspace's own
0.49 (301.2) shows that folds past a tenth exist; what is not known is whether
one exists under a policy whose answer a test reads.

### 301.5 The two new tests and their mutations

- **`a_place_as_far_off_a_corner_as_its_room_is_folded_into_it_and_one_a_step_further_is_not`**,
  in `arrange.rs`'s `tests` module. It bakes the unit cube, draws the bottom
  face up to its corners, and asks about the corner `(0, 0, 0)` on the walls
  `x = 0` and `y = 0`. The places are written by hand: the box `[−r, r]³`,
  with `r` the room, `2⁻⁴⁶`, folds; the box one `f64` step wider does not.
  That is the doc comment's number, held from both sides.
- **`a_thin_turned_wedge_whose_corners_are_meets_wider_than_the_room_is_still_united`**,
  in `crates/rcad-topo/tests/a_meet_that_fills_a_corners_room.rs`. 72
  certified unions, slopes 1/16 to 1/256, every one answered with two solids
  and an empty log. It is the shape-level demonstration of 301.3, and it does
  **not** pin the room at a half or a tenth.

Each mutation was a scratch edit to the same line as 301.1, undone with
`git checkout -- crates/rcad-topo/src/fuse/arrange.rs`.

| k | The unit test | The wedge test | §288's two |
|---|---|---|---|
| 0.5 | fails: the room's edge no longer folds | not run | pass |
| 0.1 | fails: the room's edge no longer folds | passes | pass |
| 0.01 | not run | fails: slope 1/16 turned 1.1 about `(1, 2, 3)` refused | not run |
| 1.25 | fails: one step past now folds | not run | pass |

The unit test's command was `mise exec -- just t -p rcad-topo --lib -E
'test(/a_place_as_far_off_a_corner|two_places_of_one_meet_inside|a_corner_is_its_own_place/)'
--no-fail-fast`, on the branch with the unit test committed. The wedge
test's was `mise exec -- just t -p rcad-topo --test
a_meet_that_fills_a_corners_room`. There is no source change, so the
mutations are the proof. A revert would take the unit test out with the file.

**The unit test pins a number, not a need.** It says the fold's allowance is
exactly `what_writing_the_corner_down_cost`, which is what the doc comment and
§253.4 claim. If a shape that fills the room is found, a test on it should
stand beside this one.

### 301.6 The probe

This replaced the last expression of the closure in `a_corner_is_this_place`,
and was taken back out before either test was committed:

```rust
let scale = std::env::var("Q225_SCALE").ok()
    .and_then(|text| text.parse::<f64>().ok()).unwrap_or(1.0);
let allowed = super::bake::grown_by(node.point,
    super::bake::what_writing_the_corner_down_cost(node.point.enclosure()) * scale);
// … `folds` is the old expression over this `allowed` …
if let Some(path) = std::env::var_os("Q225_PROBE").filter(|_| node.point != place) {
    let corner = node.point.enclosure();
    let full = super::bake::what_writing_the_corner_down_cost(corner);
    let out = (0..3).map(|axis| (meet[axis].hi() - corner[axis].hi())
        .max(corner[axis].lo() - meet[axis].lo()).max(0.0)).fold(0.0, f64::max) / full;
    // also the meet's and corner's widths over `full`, the nearest other
    // place's gap over `full`, the test's thread name, the corner and the place,
    // appended as one line to `path`
}
folds
```

The probe test files wrote a `MARK` line into the same file before each
fuse, so each logged line could be read back against its shape. The workspace
run and the wedge and bar probes were read with short Python scripts in the
lane's scratchpad.

### 301.7 What this leaves

- **A corner with four planes round it** (301.4) is the family most likely to
  fill the room and was not built. A pyramid's apex through `SolidBuilder`,
  turned so the planes round, where `bake` solves another triple than the face
  asks.
- **The corpus folds at 0.49** are the only folds past a tenth. Which draws
  they are, what policy they ran under, and what they answer with the room at
  x0.45, is not known. No test reads their answer.
- **The rate test failing at zero** is the only shape-level pin below the
  wedge's hundredth, and a rate test is coarse.

### 301.8 Which draws the 0.49 fold belongs to, and whether shrinking the room changes their answer

*Queue row Q241.* §301.6's probe was reapplied to `a_corner_is_this_place` (the
same scratch edit, taken back out before this was committed — `git diff
6f3c62e0 -- crates/rcad-topo/src/fuse/arrange.rs` is empty, checked at the
commit that carries this section) plus a `MARK` line, and a scratch test
walked `fuzz/corpus/two_solids_fused_by_a_stranger` itself, replaying each
half's header, split, rule and policy exactly as
`crates/rcad-topo/tests/fuzzing/fuse.rs`'s `how_one_model_is_fused_noting_which_way_the_second_call_went`
does, and calling `Model::fuse` once forwards and — for a two-sided union or
intersection — once swapped, the way that harness's "both ways round" check
does. The command was
`Q225_PROBE=<file> mise exec -- just t -p rcad-topo --test q241_scratch_probe --no-capture`,
run once with `Q225_SCALE` unset (x1, as shipped) and once with
`Q225_SCALE=0.45`.

**Three draws carry the fold**, all `first` half, header `0x94` — which is
`Policy::tolerant` (`header & 0x80 = 0x80`) and `Rule::Union`
(`(0x94 >> 4) % 3 = 0`) — and all two-sided (one solid on each side):

| seed | forwards | swapped |
|---|---|---|
| `76405eecf690b34fabb09db4a396aff25fb7c542` | `Err(Undecidable)`, "a point in a piece that every solid could place" | `Err(Undecidable)`, same predicate |
| `8e8cff1316a6c48db5d7941fc91cd081ad950ccc` | `Err(CannotTellTheCellsOfAFaceApart { face: FaceSlot(6v1) })` | `Err(CannotTellTheCellsOfAFaceApart { face: FaceSlot(8v1) })` |
| `a63af238a6bc4459c21a055015ea5e495e702e46` | `Err(CannotTellTheCellsOfAFaceApart { face: FaceSlot(6v1) })` | `Err(CannotTellTheCellsOfAFaceApart { face: FaceSlot(8v1) })` |

No draw answers `Ok`, so there is no face count or volume to report for any of
the three.

**None of the three changes between x1 and x0.45.** For every one of the six
calls above (three draws, forwards and swapped), the `Err` — variant, payload
and, for the two `Undecidable` answers, the whole `ProvenanceLog` of records
behind it — is **byte-identical** between the two runs, compared line for line
by a scratch script over the probe's own output. Shrinking the room to
`x0.45` does take the fold away (`folds=false` where it was `true`, `out=0.492`
against the room now `0.45`, from the same probe line), so the draws that
carried the 0.49 are confirmed to be the right three and the two scales really
do take different paths through `a_corner_is_this_place` for them; the
downstream answer is simply insensitive to which path was taken, on all three.

**So there is nothing to check the x1 answer against.** The row's next step —
weighing the x1 answer against something that does not go through the fold —
only applies to a draw whose answer *moves*, and none of these three move.
This is not the same as the fold being harmless in general: it says the room
between x0.45 and x1 is not what decides these three draws' answers, not that
no draws are decided by it (301.7's untried four-plane family is still open).

The scratch probe test (`q241_scratch_probe.rs`) and its two log files are not
committed — the probe edit to `arrange.rs` came back out and the test file was
deleted before this commit, per the row's instruction.
