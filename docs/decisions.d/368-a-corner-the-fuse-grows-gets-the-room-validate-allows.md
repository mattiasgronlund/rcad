## 368. A corner the fuse solves again gets the room `validate` allows it — 2026-09-29

*Q414, which replaced Q27. Lane `lane-q414-a-corner-given-validates-room`, off
`7fc73410`. Every figure below was run in this lane, at that base plus its own
commit, with the command named beside it, except where 368.3 says Q705 ran it.
Q705 merged that branch onto `6273ce5e` as
`lane-q705-a-corner-given-validates-room` and flipped the three pins of 368.3.*

`derive_a_corner` in `crates/rcad-topo/src/fuse/bake.rs` asks two questions
of a corner with three or more surfaces round it. The selector,
`the_place_the_model_points_at`, picks which place three surfaces meet at is
this corner by growing the stored corner and asking which place it overlaps.
The re-check then grows the solved corner and asks every surface round it
whether it is provably off. **Both used to grow the corner by its own
rounding alone**, `what_writing_the_corner_down_cost`, `64ε × max(1, size of
the corner)`. `Model::validate` lets a flat face's corner sit off its plane by
a modelling tolerance on top of a rounding, both at the solid's size. So the
fuse refused solids `validate` accepts, and §75.6's rule is that it must not.

**Now both grow the corner by two things added together:**

- **The room `Model::validate_solid` grants a corner of its solid off that
  kind of surface.** Off a plane that is `allowed_wobble`,
  `min(default_linear_tolerance, 1e-9 × L) + 64ε × L`; off a curved surface it
  is `what_writing_a_point_down_costs`, `16ε × L`. `L` is the solid's largest
  coordinate as §363 measures it. The re-check grows the corner by the room
  for the surface it is asking; the selector by the widest room among the
  three surfaces it is choosing with.
- **The corner's own rounding**, as before. §363 measures `L` where the
  corners are stored or where the placement puts them, whichever is smaller,
  and the fuse meets the corner where the placement puts it. A solid moved far
  from where it was built has planes rounded at the far size, and this half is
  what pays for that.

  368.1   Why it is `validate`'s figure and not a copy of it
  368.2   What it lets through, and what it still refuses
  368.3   The three pins it retired
  368.4   What this leaves

### 368.1 Why it is `validate`'s figure and not a copy of it

The room is read from `validate.rs` by
`Model::room_a_corner_of_this_solid_gets`, which calls `allowed_wobble` and
`what_writing_a_point_down_costs` themselves, over the same scope
`validate_solid` builds. The fuse bakes each input solid with that solid's
room. §75.6 already said why a copy is wrong: the fuse and the sweep have to
agree, and making that true by writing the formula twice is a coincidence that
drifts. The commit that first widened the selector (`9f500c24`) read its
constant from `validate.rs` for the same reason and still took only half the
figure.

**It is `validate_solid`'s size and not `validate`'s.** `validate` measures
every corner against the whole model's largest coordinate, so a solid far out
widens the room every other solid's corners get. `validate_solid`'s own doc
comment says that accepting a corner because of a neighbour is the worse way
to be wrong, and that a fuse already judges its answer that way. So the fuse
takes a corner as far off its plane as its own solid allows. A model
`validate` passes only because of a far solid can still be refused, and that
is `validate_solid`'s answer for that solid too.

**The size is the solid's and not the corner's, and a test says so.**
`a_corner_at_the_origin_gets_the_room_its_solid_gets_and_not_the_room_its_own_size_would`
moves the block's corner at the origin `1.5e-9` off its front plane. At the
block's size, two, the room off a plane is `min(1e-7, 2e-9) + 128ε`, just over
`2e-9`; at size one it is `1e-9 + 64ε` (worked from the formula; the test
asserts the miss lies between the two). With `L` capped at one in `room_a_corner_of_this_solid_gets`, that
test fails and the other four it was run with pass (`just t -p rcad-topo
--no-fail-fast -E 'test(/^a_corner_with_fewer_than_three_surfaces_round_it::/) |
test(/big_solid_is_baked|grazing_planes/)'`: 5 run, 4 passed, 1 failed). The
cap was removed before commit.

### 368.2 What it lets through, and what it still refuses

**§136.8's block is cut.** `[0, 2]³` at a 100 mm scale with its top front left
corner stored at `(0, 1e-9, 2)`. The test's own line, `just t -p rcad-topo
--no-capture -E
'test(a_block_with_a_corner_off_one_of_its_three_planes_by_less_than_validate_allows_is_cut)'`:
the miss is `1e-9`, `validate` allows `2.0000284217094305e-9`, and the
rounding alone is `2.842170943040401e-14`. The pin
`a_block_with_a_corner_off_one_of_its_three_planes_by_less_than_validate_allows_is_refused`
is now `..._is_cut`: the difference with the unit cube from `(-0.5, -0.5,
-0.5)` is one solid of volume `8 − 0.125` and it validates. The fuse moves the
corner onto `(0, 0, 2)`, where its three planes cross, by no more than
`validate` let it sit off them.

**§136.4's corner is baked.** bake.rs's ignored repro is
`a_corner_at_the_origin_of_a_big_solid_is_baked_for_a_miss_validate_allows`
now and runs. A chamfer pinned `1e5` away misses the origin by `1.7352e-13`
(§136.4's measurement; the test asserts the miss is past the corner's own
rounding and inside the room `validate_solid` grants a cube of side `1e5`).
The modelling half alone, `min(1e-7, 1e-4)`, is far wider than that miss, so
this case no longer turns on the size being threaded down, which is what
§136.4 named; the size test in 368.1 is the one that does. `ignored_tests` in the `justfile`
goes from 10 to 9.

**§70.3's corner is still refused.**
`a_corner_near_three_grazing_planes_but_far_from_where_they_cross_is_refused`
now passes the room a solid of size two gets, about `2e-9`, and asserts it is
smaller than the `1.414` the stored corner sits from where the three grazing
planes cross. The answer is still `Error::Undecidable`: nothing placed the
corner. The room is a modelling tolerance, `1e-9` of the solid's size at most,
and §70.3's corner is a whole unit off.

Which tests fail without the change was run as a mutation, validate's two
rooms multiplied by zero in `room_a_corner_of_this_solid_gets`, which is the
code at the base: `just t -p rcad-topo --no-fail-fast -E
'test(/^a_corner_with_fewer_than_three_surfaces_round_it::/) |
test(/^a_corner_its_own_surfaces_place_somewhere_else::/) |
test(/big_solid_is_baked|grazing_planes|a_fourth_plane/)'` gives 10 run, 7
passed, 3 failed. The three are the §136.8 test, the §136.4 test and the
size test above.

### 368.3 The three pins it retired

Three pins failed with the change, in files Q414's lane did not own; Q705
flipped them. In Q414's lane `just t --workspace --no-fail-fast` gave 2824
run, 2821 passed, 3 failed, 9 skipped. The steps the gate runs after its tests
passed on that commit: `cargo test --workspace --doc`, `just
check-release-build` (3 run, 3 passed) and `just check-fuzz-unit` (exit 0).

**`crates/rcad-step/tests/a_fuse_read_back_from_rounded_text.rs`.**
`a_turned_cube_read_back_from_thirteen_digits_cannot_be_united_with_nothing`
pinned this gap as it stood: a turned cube read back from thirteen-digit text,
which `validate` accepts, refused `Undecidable` at "where three surfaces
meet". Its doc comment said the day this is fixed it fails, and it did: the
union answers one solid (`expected a rounded corner to stop the fuse, got
Ok(1)`). It is `a_turned_cube_read_back_from_thirteen_digits_is_united_with_nothing`
now, and asserts one solid, a sound model, and the volume as built to within
the file's own rounding bound, `how_far_rounding_can_move_a_volume`.

The same file's header gives each family's counts, which are printed and not
asserted. Q705 ran each family's test, `just t -p rcad-step --no-capture -E
'test(a_turned_pair_read_back_from_rounded_text)'` and the same for
`a_skew_pair_…`, `a_lattice_pair_…` and `a_block_with_a_bar_through_it_…`,
with the change and under the mutation of 368.2, which is the base's fuse:

| family | exact text | 11 digits | 12 digits | 13 digits |
|---|---|---|---|---|
| turned, before | 109 of 144 | 0 | 0 | 0 |
| turned, after | 109 of 144 | 107 | 105 | 111 |
| skew, before | 122 of 144 | 122 | 122 | 45 |
| skew, after | 122 of 144 | 122 | 122 | 122 |

Decided fuses, on the file's fixed seed. With the change the turned family's
rounding still introduces 8, 10 and 4 refusals and rescues 6 at each digit
count, and no answer is wrong. The header's claim that every one of the 109 is
refused held at the base and the change made it stale. Its skew row said 77 of
119 at 13 digits; the base gives 77 of 122, so the 119 was stale before this
change. The lattice and bar-through-a-block rows move no real and decide 144
at every digit count, before and after.

**`crates/rcad-topo/tests/a_corner_its_own_surfaces_place_somewhere_else.rs`**
pinned the rounding-only room from both sides.

- `a_corner_a_tenth_of_a_nanometre_off_its_own_top_is_refused`: a box of side
  two with a corner `1e-10` off its top. Its header said `validate` accepts it,
  and that the fuse "silently" replacing the stored corner with where its
  planes cross is what it guards against. §75.6's rule and §136.8 overrule
  that: the corner is inside the room `validate` grants, a face *is* its plane
  (§17.1), and the corner moves by less than that room. It is
  `a_corner_a_tenth_of_a_nanometre_off_its_own_top_is_moved_onto_where_its_planes_cross`
  now. It asks the intersection and not the union, because the union swallows
  the corner inside the cube and the intersection, `[0, 1]³`, keeps it, and it
  asserts the answer's corner is `(1, 1, 1)` bit for bit.
- `a_corner_twice_the_rounding_room_off_its_own_top_is_refused`: `2.84e-14` off
  is inside the new room too. Its line, "the room is a rounding and not a
  licence to move a corner", is what this section changes. It is
  `a_corner_twice_the_rounding_room_off_its_own_top_is_baked` now.
- **The far side of the new room cannot be pinned through the fuse.** A corner
  past it is past what `validate` allows, and `SolidBuilder::build` sweeps the
  solid and refuses it as `VertexOffItsFacePlane` first.
  `a_corner_just_past_the_fuses_room_is_refused_before_the_fuse_sees_it` shows
  that, with a corner a rounding past the fuse's room. It passes with or
  without the change: it pins the claim, not the change.

Under the mutation of 368.2 the three flipped pins fail, and so do the three
tests 368.2 names: `just t -p rcad-topo -p rcad-step --no-fail-fast -E
'test(/^a_corner_its_own_surfaces_place_somewhere_else::/) |
test(a_turned_cube_read_back_from_thirteen_digits_is_united_with_nothing) |
test(/^a_corner_with_fewer_than_three_surfaces_round_it::/) |
test(/bake::tests::/)'` gives 20 run, 14 passed, 6 failed. The header's
`allowed_wobble` figure was also wrong, a tenth of a nanometre where the
formula gives `1e-9` at size one, and Q705 corrected it.

### 368.4 What this leaves

- **`a_corner_is_this_place` in `arrange.rs` still folds a meet into a corner
  only within the corner's own rounding**, or when the three are planes bake
  solved the corner among (§356). Its doc said that rounding is "the allowance
  `bake.rs` uses to let a meet stand in for a stored corner", which stopped
  being true here. Q705 corrected that doc comment and
  `bake_placed_the_corner_on_the_three_planes`'s, and left the code as it was.
  Its room should grow to `bake.rs`'s, as a row of its own. The case it would
  miss: a corner with four or more surfaces
  round it, one of them curved, where a plane misses the solved corner by more
  than the rounding and within `allowed_wobble`. A meet of a triple holding the
  curved surface and that plane can then land outside the rounding, stays a
  node of its own, and is left to `merge_points_that_are_one_point`. Nobody has
  built one, so `refuses/C` at most. Before this change the re-check refused
  such a corner outright, so it is not a new refusal.
- **`the_half_space_both_ends_are_on` in `assemble.rs` need not move.** Its
  room is how far the next fuse's re-solved box can land from a stored end of
  this fuse's answer. The answer's corners are meets written down, within a
  rounding of their surfaces, and the next fuse re-solves them from the same
  surfaces. The selector's room decides whether a box is taken, not where it
  lands. Worked, not run.
- **The re-check can still refuse a solid `validate` accepts.** The stored
  corner is within the room of each plane, and the solved corner within the
  room of the stored one along each axis, so a fourth plane can miss the solved
  corner by up to about twice the room. A corner with four planes each missed
  by nearly the whole tolerance in opposing directions would be refused
  `VertexOffItsFacePlane`. Not built, `refuses/C` at most.
- **A curved surface's room leaves out the slop recorded on a corner.**
  `validate` grows a corner of a curved face by that slop or by
  `what_writing_a_point_down_costs`, whichever is wider (`not_provably_off`);
  the fuse grows by the second alone, as it grew by the rounding alone before.
  `refuses/C` at most.
- **§286.1's answer (c) is now the room `derive_a_corner` asks with.** Asking
  a corner with fewer than three surfaces the same way would refuse only a
  solid `validate` refuses. It is still not built, for §286.1's reason: it is
  `validate`'s own question asked again at the door. The `bake.rs` header and
  the test file's header say so.
