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
  (351.3). Its other order was not a solid on the base as well.
- §316.6's side finding was §324's. It answers both ways round here too.
