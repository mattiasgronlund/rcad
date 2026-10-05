## 407. The joiner joins one pipe however it is written — 2026-10-05

*Queue row Q527, the section Q354 (`84538a4e`) owed. That commit landed the
code; this records what it moved and backs its figures by rerunning the Q44
probe on this base.*

  407.1   What the joiner does now, for a pipe
  407.2   What it moves
  407.3   The Q44 probe, rerun on this base
  407.4   Cones: "still join by bits" is only part of the truth now

### 407.1 What the joiner does now, for a pipe

Three names, checked against the base with `just src` and unchanged since
Q354:

- `one_cone_or_pipe_the_same_way_round` (`fuse/assemble.rs:3320`) decides
  whether two curved faces of the fuse's own answer are pieces of one surface,
  used the same way round. For a pipe it compares `one_surface(&mine.surface,
  &theirs.surface)` — the **folded** key
  (`name_of_the_surface_either_way_round`), which is exact on a pipe's axis
  turned round and on its origin slid where that folds exactly. A cone still
  compares `name_of_surface_bits` — the raw, unfolded key — so two spellings of
  one cone are left apart here (§407.4).
- `rings_of_each_joined_face` (`fuse/assemble.rs:918`) builds the rings of each
  joined group. For a non-planar group it takes `members.iter().min_by_key(|face|
  name_of_surface_bits(&face.surface))` — the member whose surface sorts lowest
  by the raw bits — so the joined face's spelling does not depend on which
  solid was handed to `fuse` first.
- `turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way`
  (`fuse/assemble.rs`, below `rings_of_each_joined_face`) is Q354's rename of
  `read_every_side_off_the_spelling_its_face_is_joined_on`. It no longer
  rewrites `Edge::carried_by` (switching that off changed no answer: every
  rcad-topo test and all 168 Q44 probe fuses came out the same, per `84538a4e`'s
  message). What it still does: for every **closed** edge whose face was joined
  onto a different spelling than the one it was built on, read which way the
  new spelling stores the edge and which way the old one did
  (`a_closed_crossing_is_walked_the_way_it_is_stored`), and turn both sides
  round when they disagree. `a_circle_the_joined_spelling_names_instead_of_a_thin_pipe_is_turned_round`
  (`fuse/assemble.rs`, in `mod tests`) pins this on a hand-built assembly: a
  thin pipe crossing a wall spelled twice stores the circle between them
  opposite ways depending on whether it sorts between the two spellings or
  above both, and only the first case needs the turn.

### 407.2 What it moves

- **§224.2 item 1** said "the same bits and the same `used` orientation give
  Pos... a surface equal only up to how it is written gives Undecidable, and
  those faces are left apart" as the whole design. For a pipe that is no
  longer the rule: `one_cone_or_pipe_the_same_way_round` folds a pipe's
  spelling before comparing. **Closed for pipes by §407, 2026-10-05.**
- **§229.2** said "two curved faces with the same surface bits, used the same
  way round, are one face" as what `ff78ab00` built. That was the bit-exact
  rule for every curved kind. For a pipe it is now the folded rule.
  **Closed for pipes by §407, 2026-10-05.**
- **§231.5 second item** said "The joiner could join two spellings of one
  curved surface. It would have to pick one spelling by bits, and then turn
  round every closed side of the joined face read off the other one." That is
  exactly what `84538a4e` (readied by `68b81daf`, §231.3, and built on by
  Q354) does for a pipe. **Closed for pipes by §407, 2026-10-05.**
- **§325.5 row 2** said "Two spellings of one wall stay two faces... The two
  fixes on the table: the joiner could join pieces whose surfaces `intersect`
  calls one surface rather than only pieces with the same bits (§229.2), or
  `crossing_for` could let an edge between two spellings through." The first
  of those two is what landed. §325.4's own 2026-09-29 note already recorded
  this for its one case; **row 2 is closed for pipes by §407, 2026-10-05.**

Cones are untouched by any of this (§407.4): `one_cone_or_pipe_the_same_way_round`
still compares a cone's raw bits, so two spellings of one cone stay two faces
coming out of this function, as they did before Q354.

### 407.3 The Q44 probe, rerun on this base

The probe named in the row (`/tmp/.../scratchpad/q44_probe.rs`, 19919 bytes,
2026-09-28) still exists outside any tree. Copied into
`crates/rcad-topo/tests/q527_probe.rs` (gitignored by nothing — removed again
before this commit) and run with `mise exec -- cargo nextest run -p rcad-topo
q44_probe --no-capture` against this base (`a646a5f2`), writing
`target/q527/q44_probe_run.log` (gitignored, kept for the verifier to rerun
the same copy against):

- `q44_probe_every_second_fuse` prints 168 `PROBE` lines, matching `84538a4e`'s
  "all 168". Of those, the three spellings the row names — `Pipe Slid`,
  `Pipe Reversed` and `Pipe RoundBars` (`add_round_bar`) — are 28 lines each,
  84 in all. **All 84 answer (0 `REFUSED`) and all 84 validate**
  (`grep -c REFUSED` on the 84 is 0; none is missing `validate Ok("ok")`),
  matching the row's "0 to 28 of 28" for each spelling.
- **Whether those 84 match the one-spelling answer**: comparing each line's
  `then <rule> with <second>: <answer>` tail against the `Pipe One` line for
  the same `lower_first`, `rule` and `second` (`target/q527_compare2.sh`), 67
  of 84 match the `Pipe One` line **character for character**; the other 17
  report the same face count, the same curved-face count and a volume within
  about `6e-14` of the `Pipe One` line's volume (for example `Reversed`'s
  `Union with FarCube` gives `58.20344246266877` where `One` gives
  `58.20344246266879`), both well inside the published error bound on the same
  line. That is floating-point noise from a different order of construction,
  not a different answer — the row's "84 of 84 match" is read here as "within
  the bound `measure` publishes," which is 84 of 84, not as bit-identical,
  which is 67 of 84. The two Q44 probe test functions themselves,
  `q44_probe_which_face_and_where` and `q44_probe_every_second_fuse`, both
  pass outright (`cargo nextest`'s own verdict, not a figure this probe
  prints).

### 407.4 Cones: "still join by bits" is only part of the truth now

The row says "cones still join by bits until Q514," and that is still true of
`one_cone_or_pipe_the_same_way_round` itself — it is the one function the row
names, and its cone arm is untouched. But it is not the only place two
spellings of a cone can end up one face, and reading past it long enough to
check shows a second path that already folds a cone's spelling, for a
narrower case than the joiner's.

Q511's `a_frustum_banded_in_both_spellings_of_its_cone_builds_and_measures`
and `that_frustum_united_with_a_block_clear_of_it_keeps_its_volume`
(`crates/rcad-topo/tests/a_cone_face_against_its_own_reversed_spelling.rs`)
build one solid — a frustum from `z = 2` to `z = 6` — whose wall is **one
input solid's own two faces**: a band about `+z` from `z = 2` to the rim at
`z = 4`, and a band about `−z` from the rim to `z = 6`, split by a closed edge
the builder writes directly. United with a block standing clear of it, the
result keeps the rim's wall as **one face**, in the spelling that sorts
first. Q514's `two_frustums_stacked_on_one_cone_turned_round_unite_either_way_round`
(`crates/rcad-topo/tests/two_frustums_stacked_on_one_cone.rs`) builds the same
shape as **two input solids** — a lower frustum `z ∈ [2, 4]` about `+z` and an
upper one `z ∈ [4, 6]` about `−z`, touching at `z = 4` — and unites them.
There the wall comes back as **two faces**, one per spelling, unchanged since
before Q354.

Reading alone left it open which path joins Q511's bands and why it does not
reach Q514's pair, so both tests were run with a temporary `eprintln!` in
`one_cone_or_pipe_the_same_way_round`'s caller
(`edges_with_one_face_on_both_sides`, `fuse/assemble.rs`) and in
`one_curved_surface_used_one_way` (`fuse/bake.rs`), then reverted before this
commit (`git status`/`git diff` on `crates/` is clean).

- **Q511's bands never reach `one_cone_or_pipe_the_same_way_round` at all.**
  The print in `edges_with_one_face_on_both_sides` never fired for the
  frustum-and-block test. `one_curved_surface_used_one_way`'s print did,
  twice, for exactly the pair `Cone(axis −z)`/`Cone(axis +z)`, both `same=true`
  — once each way round, since `faces_baked_as_one` walks every face's every
  coedge.
- **Why**: `bake_one_solid` (`fuse/bake.rs`) runs on every input solid of a
  fuse, one solid at a time, before arrangement ever sees a crossing. A solid
  that is not wholly flat is grouped by `faces_baked_as_one`, which joins two
  faces of **that one solid** when an edge runs between them and
  `one_curved_surface_used_one_way` says they are one ball, pipe, cone or ring
  used the same way round. That function calls `one_surface` — the **folded**
  key, same as a pipe's arm in `one_cone_or_pipe_the_same_way_round` — not
  `name_of_surface_bits`. Its own doc comment says why a cone is safe to fold
  here: "a cone's [surface] is even in how far along the axis a point sits,
  because its surface is both nappes... so turning the axis round changes no
  sign." Q511's frustum is one input solid with its wall pre-split by the
  builder into the two nappes' spelling either side of the rim, so baking it
  alone finds the pair and joins it — before the block, or any crossing, is in
  the picture at all. This is what the row's own "cones still join by bits"
  does not cover: the bake step's curved-face join was already folded for
  cones, just never for a pair split across two different input solids.
- **Why Q514's pair is not joined.** `bake_one_solid` runs on the lower
  frustum and the upper frustum **separately**: each, baked alone, has exactly
  one wall face, so `faces_baked_as_one` has no second member to pair it with.
  Nothing folds a cone's spelling *across* solids at bake time. By the time
  the two baked solids are arranged and the fuse writes its answer, the only
  place left that could join the two wall faces is
  `one_cone_or_pipe_the_same_way_round` — the joiner this section is about —
  and its cone arm still compares raw bits, which these two spellings do not
  share. So the pair goes through unjoined, as the test's own doc comment
  says ("this file's face count is the one to move to three when a joiner
  reaches cones").

So: the bake step already folds a cone's two nappes **within one input
solid's own faces**; the joiner this section builds for pipes does not touch
cones at all, and is what would be needed to fold a cone's spelling **across**
solids, which is Q514's case and still refuses to answer with one wall. Taking
the joiner's cone arm from `name_of_surface_bits` to `one_surface` is the
smallest-reach candidate for Q514, and is untried — `faces_baked_as_one`'s own
doc comment (Q898) is the warning for it: a cone joined without
`what_each_edge_left_out_ran_along`'s corner handling got past
`AFaceMeetsItsNeighbourOnItsOwnSurface` only to be refused `Undecidable` at
every wall of `pieces_of_one_curved_surface.rs`.
