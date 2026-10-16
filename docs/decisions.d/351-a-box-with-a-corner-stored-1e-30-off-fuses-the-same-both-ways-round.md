## 351. A box with a corner stored 1e-30 off fuses the same both ways round — 2026-09-28

*Queue row Q307, from §316.5. Lane `lane-q307-a-corner-stored-1e-30-off`, off
`665d42d2`. §316.5 named two fixes for a flat piece that the ray placed
`Outside` a solid whose face it lies on, and measured neither. **Both are
measured here, and the first is taken**: `settle_against_each` now asks whether
any piece, flat or curved, lies on a face of the solid before it fires the ray.
The two `[0, 2]³` boxes of §316.3 are one cube of volume 8 both ways round,
under union and intersection and both policies, and the box less its copy is
nothing both ways round. The replay's `a_solid_one_way_round_only` reads 0.
The second fix refuses every one of those fuses instead.*

### 351.1 The two fixes, measured

A throwaway probe test fused two boxes on the same six planes, one with one
stored corner moved, at 13 moves: §316.3's rows at unit size and at the seeds'
size `s`, the seeds' own three moves at size `s`, `(2, 0, 0)` moved by `1e-30`
and by `1e-300`, and `(2, 2, 2)` moved four ulps of `2` up in `y`. Each move
under both policies, all three rules and both orders is 156 fuses. A fuse
counts as wrong unless it answers with one six-faced solid (nothing, for a
difference) and the model validates. It was taken out before committing.

| | wrong of 156 | what the wrong ones gave |
|---|---|---|
| base `665d42d2` | 66 | `ResultIsNotManifold` one way round, answered the other, for every move but the two of `(2, 0, 0)` |
| fix 1: ask `coincident_face` first for a flat piece too | **0** | — and every log came back empty |
| fix 2: a `Zero` from the plane against a decided sign from a triangle is a disagreement | 132 | `Undecidable`, `a point in a piece that every solid could place`, **both** ways round, for all 11 moves that were wrong on the base, every rule |

Fix 2 fails exactly the way §316.5 feared. Every candidate of a piece lying on
the moved box's face lies on that face's plane, the tilted triangles give every
one a sign, and so every one is dropped. The fuzz corpus replay shows the same
thing from the other end:

| `every_seed_in_the_checked_in_corpus_is_fused_without_panicking` | `ok` | `refused` | `the_same_both_ways_round` | `a_solid_one_way_round_only` |
|---|---|---|---|---|
| base | 283 | 407 | 20 | 3 |
| fix 1 | 287 | 403 | 24 | 0 |
| fix 2 | 282 | 408 | 20 | 0 |

Fix 2 takes the counter to zero by refusing the three seeds' answers. Fix 1
takes it to zero by answering both ways round. With fix 1, every half of the
three seeds that decodes answers both ways round with the same cube,
`2e9c4138`'s first half included, which §316.1 found refusing both ways round.

Over the whole of `rcad-topo` (`mise exec -- just t -p rcad-topo
--no-fail-fast`), fix 1 failed one test outside the new file, the absorption
pin of 351.3. Fix 2 failed only the new file.

### 351.2 Which fix, and the family both come from

**Fix 1.** Both fixes come from one family: **the stored plane says where a
face is (§17.1), so where the ray's reading of a face and its plane can differ,
the plane decides.** The ray reads a face as triangles fanned from its stored
corners (`FaceView::for_each_triangle`). A valid solid may keep a stored corner
a hair off where its planes meet, because `Model::validate` accepts that
(§75.6), so the two readings can differ on a solid the caller was allowed to
hand over.

- Fix 1 reads the face from its planes. `coincident_face` proves two planes are
  one plane from their coefficients (`agreement_of_two_planes`), and
  `a_face_of_the_input_holds` winds the point round the **baked** corners, which
  `bake::derive_a_corner` solves from the planes. The stored corners are never
  read.
- Fix 2 notices the two readings differ and gives the candidate up. For a piece
  lying on the face every candidate differs, so it can only refuse.

Neither is a tolerance. Every step of fix 1 is a sign or an exact comparison
of coefficients.

Three other fixes were considered and not measured:

- **Fan the ray's triangles from the baked corners.** That is `classify.rs` and
  `FaceView`, outside this row, and it changes point classification for every
  caller rather than for the fuse. With fix 1 the fuse no longer asks the ray
  about a face the piece lies on, so the fuse does not need it.
- **Refuse, or move, a stored corner off its planes when the solid is built.**
  §75.6 settled that `validate` accepts it, which sets what the fuse must
  handle. Changing that is a decision about the data model, not this row.
- **Start the fan somewhere else.** Which corner starts a wire is the caller's.
  The `(2, 2, 2)` move shows that a corner which starts no wire still refuses:
  it is the far end of the diagonal each of its three faces is fanned along,
  so both triangles of each face tilt.

**What changed in code.** `settle_against_each` drops the `in_plane.is_none()`
guard, so coincidence is asked first for every piece. Its `OnBoundary` and
`Undecidable` arm no longer asks `coincident_face` again. That call could only
repeat the answer the first one had already given, and write its log records
twice. `the_ray_and_the_stored_planes_disagree_about` is unchanged, and its
doc no longer claims the ray says `OnBoundary` wherever it cannot tell. It now
says why a `Zero` is not counted there.

### 351.3 What else moved

Every tally line of `fuses_that_must_agree.rs` was compared, base against fix
1 (`mise exec -- just t -p rcad-topo --test fuses_that_must_agree
--no-capture`, one run each):

| property | base | fix 1 |
|---|---|---|
| absorption over a ball | 67 refused of 134 fuses, 0 of 67 compared | 25 refused, 42 of 67 compared |
| `A \ A = ∅`, turned solid and a copy of it | 6 not a solid | 0 |
| `A ∪ A ∪ A ∪ …`, a turned solid and copies of it | 121 fuses, 5 not a solid | 131 fuses, 0 |
| commutativity, turned pairs | 26 not a solid, 8 disagreed | 24 not a solid, **10 disagreed** |

Every other line was the same.

**Absorption over a ball is the pin that moved.** Its second fuse is handed
half a ball whose flat face lies in the block's floor. That fuse refused on
every draw. Now 42 answer, and all 42 are the block. The pin that nothing was
compared became a floor at `A_THINNER_LAW_STILL_COMPARES_THIS_MUCH` (a half,
which a halving of 63% crosses). The predicate the other 25 name,
`a point in a piece that every solid could place`, is pinned beside the floor.
Twelve values of `RCAD_SOAK_SEED` all gave the same 42, because that seed
reseeds only the measurement tests. Why the other 25 still refuse was not
traced. `A \ (A \ B)` over a ball did not move, and still refuses 67 of 67.

**The turned pairs' counted rate rose from 8 to 10, and nothing got worse.** A
temporary print of each configuration's two orders, with the old behaviour
behind a temporary switch, found one configuration that changed out of 67.
Under union and under intersection it went from not a solid both ways round to
answering one way round: the union answered with the second solid first, the
intersection with the first first. That is §316's pattern, where the sign of the
move picks which order refuses. So fix 1 made half of that pair right, and the
count of pairs that disagree grew by the two fuses it put right. The other half
was not traced.

§316.3's table is out of date in one row: at unit size a move of `1e-300`
answered both ways round when it was measured. Since §324 the ray's `orient3d`
decides a sign that small, and on `665d42d2` that move refused one way round
like the rest.

### 351.4 The pins

- `tests/three_flat_seeds_that_fuse_one_way_round.rs`, new. The reproducer is
  one cube of volume 8, six faces, under both rules, both orders and both
  policies. The model validates, and a certified log is empty. The box less
  its copy is nothing, both orders. The probe's other ten moves that were
  wrong on the base are one cube both ways round, certified. Each test fails
  on `665d42d2`.
- `tests/every_model_the_fuse_fuzzer_kept_is_fused_again.rs`:
  `a_solid_one_way_round_only` is asserted to be 0. A new test replays the
  three seeds by name and asks every decoding half to answer both ways round
  alike. The module doc's "Both ways round" paragraph now names the right
  halves: the second halves of `2e9c4138` and `65452b22` (`0x49`, certified
  intersections) and the **first** half of `1422656e` (`0x94`, a tolerant
  union). It calls the three one defect, not §48.2 item 5's class to count.
- `tests/fuses_that_must_agree.rs`: the absorption pin, as in 351.3.

No pin moved in a file outside this row.

### 351.5 What this leaves

- **`settle_against_each`'s `in_plane` argument is unused now.** It is named
  `_in_plane`, because the one call that passes it is in `place_the_piece`,
  outside this row. Dropping it there and here is a two-line follow-up.
- **The 25 absorption refusals over a ball**, still
  `a point in a piece that every solid could place`, not traced.
- **One turned pair that answers one way round only** is new to the count
  (351.3). Its other order was not a solid on the base as well. Configuration
  52 is the one this row's lane traced, and 351.6 closes it.
- §316.6's side finding was §324's. It answers both ways round here too.

### 351.6 An input corner on three planes is named by them, and put where they put it

*Queue row Q711, from 351.5 and Q476. Lane
`lane-q711-a-comb-and-an-ell-turned-27-64`, off `c3c02942`.*

**The shape.** Configuration 52 of the turned swap test is a comb (3 teeth, 1
high) at the origin and an ell (4 by 3, a 1 by 1 bite, 3 high) at `(2, 0, 0)`,
turned about `(2, 1, 2)` by 27/64 of a turn. The comb's corner `(2, 2, 0)`
lies on the ell's floor-and-`x = 2` edge, and the two solids store those
planes with the same bits. Selection keeps the floor and the wall of whichever
came first. In one order `Corners::of` named the corner as the meet of three
planes from the ell's pieces and as `Vertex(8v1)` from the comb's `y = 2` wall,
4 ulps apart in `y`, and four edges got one face each: the union comb first
and the intersection ell first were `ResultIsNotManifold { count: 1 }`.

**The fix, in two halves, both in `assemble.rs`.**

- *The name.* An input corner whose `around` is exactly three planes is named
  `CornerName::Meet` of those planes, `which` zero, so the other solid's
  pieces that meet it there name it alike. The planes travel from
  `BakedCorner::around`, read off `Scratch`, as Q476's probe did.
- *The point.* A corner on three planes is put where the three alone put it:
  each turned to face the one way, sorted, solved by Cramer's rule. That is
  `arrange`'s own solve for a meet of three planes, so a meet arriving from a
  piece lands on the same bits. Q476 measured the name alone and found four
  turned pairs that then answered both ways round but differed bit for bit,
  because `settle_on_the_lower_rounding` picks among the spellings that
  arrived and which arrive depends on the order. Where Cramer's rule cannot
  bound the meet, the point that arrived is kept; an input corner there keeps
  `bake`'s solve, the one case left that can still round apart.

The reproducer is `tests/a_turned_tooth_corner_on_the_edge_of_an_ell.rs`:
union and intersection, both orders, one solid of volume 36 and 5, the same
bit for bit, validating. Both tests fail on `c3c02942`.

**What moved.** A temporary switch putting the old naming back gave base and
fix from one build, on the shipped corpus:

| | base | fix |
|---|---|---|
| commutativity, turned pairs: not a solid of 268 | 24 | 8 |
| disagreed about being a solid | 10 | 0 |
| comparisons reached of 134 | 101 | 114 |
| §17.7 headline, turned pairs: not a solid of 441 | 14 | 6 |
| §17.7 headline, turned pairs: refused of 441 | 72 | 72 |

Over the 67 pairs, counting a rule both ways round as one: ten that answered
one way round only (#6, #17, #41, #52 and #55, union and intersection each)
and five that answered neither way (#7 both rules, #24, #28 and #50 unions)
answer both ways now, bit for bit. Two unions that answered both ways, #37 and
#44, answer neither now. Each is two solids touching along an edge, which the
same pair unturned refuses `count: 4` both ways round, and turned they now
refuse the same. #19 and #49 unions were not a solid both ways before and
still are, `count: 4`. **Turning a pair no longer changes whether those
answer.**

Elsewhere in `rcad-topo` (`mise exec -- just t -p rcad-topo --no-fail-fast`):

- `how_often_one_pair_of_turned_boxes_leaves_such_an_edge`: 104 of 624
  `count: 1` went to 0, all 624 answered. The cube and the taller box turned
  by a sixty-fourth now unite to the taller box under both policies.
- `turning_a_pair_makes_what_the_two_share_depend_on_which_one_comes_first`
  flipped: the turned frame and comb share the same two solids both ways
  round, bit for bit. It is renamed
  `turning_a_frame_and_a_comb_shares_the_same_two_solids_whichever_comes_first`,
  and, as its doc asked, `both_agree_about_whether_there_is_an_answer` fails
  on the turned family now instead of counting. The swap property drew no
  disagreement over 979 turned pairs at `RCAD_SOAK_FACTOR=20`, and no
  property of `fuses_that_must_agree.rs` failed for it at a factor of 5.
- `the_turned_pair_whose_union_lost_a_face_one_way_round.rs`: the ell and
  comb turned half a turn no longer unite. The union is
  `ResultIsNotManifold { count: 4 }` both ways round, which is what the same
  pair unturned gives. Its two union tests fail, and that file was outside
  this row's write set.

**The band.** The turned not-a-solid rate fell from 2.45% to 0.78%: 726 of
92 844 fuses, `RCAD_SOAK_SEED` 11 to 22 at `RCAD_SOAK_FACTOR=20`, 48 to 78 a
seed. Q915 forecast 0.81%. `A_TURNED_PAIR_IS_NOT_A_SOLID` is re-derived by
the convention to 0.52%–2.3%. §303.4 still cites the old band.

**Q915's two cubes answer.** Two cubes of side 2, the second moved by
`(2, 0.5, 0.25)`, both turned 0.7 about the axis through `(1, 0, 0)` along
`(1, 2, 3)`: the union is one solid of volume 16.000000000000004 both ways
round and validates, where the switch gives `ResultIsNotManifold { count: 1 }`
both ways. That was a scratch test and is not pinned here.

### 351.7 What this leaves

- `the_turned_pair_whose_union_lost_a_face_one_way_round.rs` needs its union
  tests to pin the `count: 4` both ways round, and its ring test a pair whose
  union still repeats a corner, or none.
- Q915's own pins and the questions it asks beyond the union.
- §303.4's sentence on the turned band, and §242's citation of
  `a_box_and_a_taller_one_in_the_same_place_are_refused_once_they_are_turned`,
  now `a_box_and_a_taller_one_in_the_same_place_unite_to_the_taller_box_turned_too`.
- An input corner on three planes whose meet Cramer's rule cannot bound keeps
  `bake`'s point, and can still round apart from a meet of the same planes.
  How often a fuse reaches that was not measured.

### 351.8 Landed, and the turned pair's file pinned again

*Queue row Q925. Lane `lane-q925-land-a-comb-and-an-ell-turned-27-64`, Q711's
branch merged onto `5d28a70c`, which holds Q609's and Q668's changes. The
first item of 351.7 is done here, and the third.*

**The ell and the comb.** `the_turned_pair_whose_union_lost_a_face_one_way_round.rs`
pins §149's pair's union as `ResultIsNotManifold { count: 4 }` both ways
round, upright and turned half a turn about `(0, 3, 2)`, in one test.
Upright, the comb's right tooth sits flat against the ell and its left tooth
meets the ell along one edge only, from the origin to `(0, 0, 1)`. Why the
turned pair answered before 351.6, with a ring through the origin twice, was
not traced. The intersection test is kept as it was.

**The ring through one corner twice.** No other test reached a fused answer
with a ring through one corner twice. `fuses_that_must_agree.rs`'s own ring
test writes its ring by hand, and the ball against a prism's edge that Q686
is about makes its wire through one corner twice inside the fuse and refuses
at meshing, so no answer reaches the caller. So a scratch test,
since deleted, searched the suite's families: 2 000 draws each under proptest's
`TestRunner::deterministic`, every rule both ways round.

| family | fuses | answered | with a ring through one corner twice |
|---|---|---|---|
| lattice | 12 000 | 11 750 | 26 |
| turned | 12 000 | 9 848 | 8 |
| skew | 12 000 | 9 978 | 4 |

All 16 it printed were unions. One of the turned draws is an ell five by four
with a three-by-one bite, three high, and a frame four by four, one high, at
`(-3, -3, 0)`. The ell covers the frame's far corner, so the frame's top face
is left a C whose two ends touch at `(0, 0, 1)`, a corner of its hole. Its
union has that ring upright and turned by the file's half turn, the same both
ways round, and its intersection has none.
`an_ell_and_a_frame_unite_to_a_face_whose_ring_goes_through_one_corner_twice`
pins that, in place of
`the_union_of_this_pair_has_a_ring_that_goes_through_one_corner_twice`. It
fails on `5d28a70c`'s `src/`, which the lane expected it not to: the
verifier's revert gave `ResultIsNotManifold { count: 1 }` for the union
second against first.

So §151.5's "the only ring-with-a-repeated-corner anyone has produced" is
answered: the lattice family draws them upright, about one answered fuse in
450 there, and the smallest-rotation rule is asked of real answers and not
only of the ring written by hand.

**The citations.** §303.4 gives the turned band since 351.6, and §242 names
the box and the taller one's test by its new name.

### 351.9 What this leaves

- `fuzzing/comparison.rs`'s doc on `turned_to_its_smallest` still says a
  turned pair makes a ring through one corner twice at the point the turn
  holds still. 351.8 shows it is the frame's C, upright or turned. The file was
  outside this row's, and Q936 fixes the doc.
- 351.7's last item, how often an input corner keeps `bake`'s point, is
  counted in 351.10.
- Q915's union answers since 351.6, by that section's scratch test. Q915 was
  dropped and re-filed as Q926, which is open for its own pins.

### 351.10 How often an input corner keeps bake's point

*Queue row Q927.* At `7ad57a1f`, a probe outside `assemble.rs` made the same
calls `where_three_planes_meet_whichever_face_asks` does (each plane faced one
way, `faced.sort_by(Surface::how_it_sorts_against)`, `Plane::meet_of_three`,
then the point kept only when its uncertainty is finite) over every vertex of
each solid the turned and the skew pair families (351.8's scratch test) drew,
40 000 draws of each family at `RCAD_SOAK_FACTOR` 20, restricted to a vertex on
exactly three distinct face planes — chosen by the probe itself, not by
`input_corners_on_exactly_three_planes`, since that function is what the row
says not to go through. 1 040 084 turned-family corners and 1 039 796
skew-family corners lay on exactly three planes, and **none** left `Plane::meet_of_three`
unable to bound the meet: every one of those 2 079 880 corners' three planes
met with a finite uncertainty. So `bake`'s point was never kept for this
family, and the either-way-round question 351.9 raised has no instance in it.

Zero is what this family should give: a cube's corner is three planes at
right angles, and the turned and skew pairs are boxes, so every input corner
on exactly three planes is a right-angled corner like a cube's, the easiest
case for Cramer's rule to bound. The question stays open for a family whose
input corners meet three planes at some other angle.
