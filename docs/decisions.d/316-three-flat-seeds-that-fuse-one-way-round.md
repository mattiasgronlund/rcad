## 316. Three flat fuzz seeds that fuse one way round are a defect in selection, not a limit — 2026-09-28

*Q7's lane, on base `7de56c47` (after Q170's §312). Decided from the shapes.
This section writes no code and pins nothing: the brief pins only an expected
limit, and this is not one.*

### 316.1 What the seeds hold, and what each gives today

Every half below decodes to one model with two cubes of side
`s = 2.3611832414348226e21` at the origin. Both cubes sit on the same six axis
planes, bit for bit. The second cube differs only in one or two **stored**
corners, moved by an amount no plane can see.

| seed | half, header | rule, policy | stored corner(s) of solid 2 that are off | solid 1 then 2 | solid 2 then 1 |
|---|---|---|---|---|---|
| `2e9c4138…` | second, `0x49` | intersection, certified | `(0, 1.358e-312, 0)` and `(s, 4.02e-87, 0)` | `ResultIsNotManifold { count: 1 }` | one cube, 6 faces |
| `65452b22…` | second, `0x49` | intersection, certified | `(0, 2.716e-312, 0)` | `ResultIsNotManifold { count: 1 }` | one cube, 6 faces |
| `1422656e…` | **first**, `0x94` | **union, tolerant** | `(0, 0, 3.2379e-319)` | one cube, 6 faces | `ResultIsNotManifold { count: 3 }` |

The replay's `a_solid_one_way_round_only` reads 3 on this base, and these are
the three halves it counts. Two things differ from what
`every_model_the_fuse_fuzzer_kept_is_fused_again.rs` says about them:

- **`1422656e`'s counted half is its first, not its second.** Its second half,
  whose only moved corner is `(s, 4.02e-87, 0)`, answers the same cube both
  ways round today. §316.3 predicts that: the moved corner does not start the
  wire of the face it tilts. So the file's sentence was probably wrong when it
  was written, and Q170 did not move it. Nobody replayed `0c1346f9` to prove it.
- **`2e9c4138`'s first half refuses both ways round**: `count: 3` for a union
  and `count: 1` for an intersection, under either policy. Both orders refuse,
  so no counter sees it. This lane did not read that half's shape.

### 316.2 Which answer is right

Two cubes on the same six planes are one cube. The planes decide where the
solid is and a stored corner is only a cached copy (§17.1). The fuse bakes
every corner from its three planes (`bake::derive_a_corner`), so the moved
corner comes out as exactly `(0, 0, 0)`. `Model::validate` accepts the move,
and that sets what the fuse has to handle (§75.6). So the union and the
intersection are both that cube: one solid, six faces. The order that answers
hands back exactly that, with every corner at `0` or `s`.

**The refusal is wrong.** `ResultIsNotManifold` says the answer cannot be a
solid, and it is one. The answer exists and the other order reached it, so by
§16.10 this is a gap and not a limit.

### 316.3 Why the other order refuses

This was traced with temporary prints in `select::choose` and
`assemble::darts_along_each_edge`, in a scratch copy of the tree that was not
committed.

**The smallest reproducer is at unit size.** Take two boxes `[0, 2]³`, built
face by face with `SolidBuilder`, in a `Scale::millimetres(100.0)` model.
Store the second box's corner `(0, 0, 0)` as `(0, 1e-30, 0)`. A certified
intersection refuses `count: 1` with the clean box first and answers one cube
with the moved box first. A certified union is the other way round.

**What selection does.** The `y = 0` face of each box lies on the `y = 0` face
of the other. The moved box's piece of that face is placed `On` the clean box,
facing the same way. That is right, and "keep once" keeps it only when its box
comes first. The clean box's piece is placed **`Outside`** the moved box. That
is wrong: the piece lies on the moved box's face. Now each rule follows:

- **Intersection, clean box first.** An intersection drops an `Outside` piece,
  and "keep once" drops the second of two `On` pieces. So neither copy of
  `y = 0` is kept, and four edges end up with one face each.
- **Union, moved box first.** A union keeps an `Outside` piece, so both copies
  stay, and four edges end up with three faces each.

The order that answers does so only because there the wrong `Outside` and the
right "keep once" happen to decide the same thing.

**Where the `Outside` comes from.** `settle_against_each` (`fuse/select.rs`)
asks whether two faces coincide before firing the ray only for a **curved**
piece. A flat piece's point goes to `classify_uncertain_point`. That call
splits each face into triangles from its **stored** corners, all fanned from
the first corner of the outer wire (`FaceView::for_each_triangle`). The moved
box's `y = 0` wire starts at the moved corner, so both of its triangles tilt by
`1e-30`. The piece's point `(0.5, 0, 0.5)` lies exactly on the stored plane
`−y = 0`, yet it is certified as strictly outside both triangles.

`the_ray_and_the_stored_planes_disagree_about` exists to catch exactly this: a
ray that reads triangles and a stored plane that say different things. But it
returns `false` whenever the plane puts the point at `Decision::Zero`. Its doc
says that case is *"left to the ray's own answer, which already says
`OnBoundary` or `Undecidable` wherever it cannot tell."* Here the ray says
neither. It says `Outside`, certified, about a point that the plane (which is
what decides) puts on the face. **That sentence is the defect.**

**Which moves reach it.** This is measured on the reproducer's two boxes. Each
cell gives the certified intersection, with the clean box first and then the
moved box first:

| moved corner, move | `[0, 2]³`, size 100 | `[0, s]³`, size `100 · s` |
|---|---|---|
| `(0,0,0)`, `y + 1e-30` | refuses 1, answers | refuses 1, answers |
| `(0,0,0)`, `z + 1e-17` | refuses 1, answers | refuses 1, answers |
| `(0,0,0)`, `y − 1e-17` | answers, refuses 3 | answers, refuses 3 |
| `(0,0,0)`, `y + 1e-300` or `+ 1.358e-312` | answers both | refuses 1, answers |
| `(hi,0,0)`, `y + 1e-30`, `z + 1e-17` or `y − 1e-17` | answers both | answers both |
| `(hi,hi,hi)`, any move above | answers both | answers both |
| `[−1, 1]³` and `[1, 3]³`, the same three corners, any move above | answers both | — |

In every row that refuses, the union refuses in the other order, with
`count: 3` where the intersection had `count: 1` and the other way round. The
sign of the move picks which order refuses, because `−y` makes the clean piece
`Inside` rather than `Outside`.

Two things limit which moves get through:

- A move smaller than half an ulp of a coordinate that is not zero is lost when
  the box is built. So the seeds' moves (`1e-312`, `1e-87`, `1e-319`) can only
  survive on a zero coordinate.
- A moved corner that does not start the wire tilts only one triangle. The
  first point the ray tries sits on the diagonal that triangle shares, which
  reads `OnBoundary`, so the answer comes out right.

At unit size a `1e-300` move answers both ways. That fits the ray's `orient3d`
being unable to decide a sign that small, so it falls through to the
coincidence check. This was not traced.

**2026-09-29: that row is out of date.** Since §324's second power of two,
`orient3d` decides a sign this small, and on `665d42d2` the same unit-size
`1e-300` move refused one way round like the rest of the table, rather than
answering both ways. §351.3 measured it, Q314 was dropped as moot once §351.3
pinned it, and `three_flat_seeds_that_fuse_one_way_round.rs`'s
`every_move_of_a_stored_corner_that_refused_one_way_round_is_one_cube_both_ways`
carries the row now, with the same two rows' worth of moves this table lists at
unit size and at the seeds' own size. The table above stays as the history of
what was measured before §324, not as the current shape of the code.

### 316.4 Against §48.2 item 5

§48.2 item 5 does not describe an expected refusal. It records that the laws
suite cannot **see** an asymmetric non-manifold answer. The laws suite's
`both_agree_about_whether_there_is_an_answer` calls such an answer
order-dependence, and says the rate it counts *"is zero on a fixed kernel"*.
These seeds show that symptom.

**They are like the configuration item 5's pinned test holds, not the same
one.** `turning_a_pair_makes_what_the_two_share_depend_on_which_one_comes_first`
pins two input vertices at one corner that a turn leaves a last bit apart, so
that neither the names nor `exactly_at` join them (roadmap §7.4). Here the
corners do join: every edge the prints showed runs from the single corner at
`(0, 0, 0)`. What goes wrong is that one face is read as lying off the other
face.

### 316.5 The row this hands on

**Fix:** in `fuse/select.rs`, a flat piece's point that lies exactly on a
stored face plane of the solid it is placed against must not be called
`Inside` or `Outside` by triangles fanned from stored corners. There are two
fixes and neither has been measured:

- Ask `coincident_face` first for a flat piece, as `settle_against_each`
  already does for a curved one. It winds the point round the **baked**
  corners, which are where the planes meet.
- Have the cross-check count a `Zero` from the plane against a sign from a
  triangle as a disagreement. The risk: every point of a coincident piece lies
  on the plane, so this could drop all of them and refuse
  `PLACED_AGAINST_EVERY_SOLID` instead.

The test to write first is the unit reproducer above. Both rules in both
orders must give one cube of volume 8. After the fix the replay's
`a_solid_one_way_round_only` should read 0. The flat families' counted rate in
`fuses_that_must_agree.rs` may move too.

The same row owns correcting the replay's module doc
(`every_model_the_fuse_fuzzer_kept_is_fused_again.rs`, the "Both ways round"
paragraph). It calls all three halves second halves and certified
intersections, and it files them as a class to count rather than as a defect.

### 316.6 A side finding, not traced

The same two unit boxes with corner `(2, 0, 0)` stored as `(2, 1e-300, 0)`
refuse `Undecidable` in both orders under both rules. The record names
`orient3d` at `Level::Interval` on `(0,0,0)`, `(2, 1e-300, 0)`, `(2, 0, 2)`,
`(0.5, −0, 0.5)`. That determinant is exactly zero, because the fourth point
is a quarter of the third, so exact arithmetic could decide it. Both orders
refuse, so no both-ways counter sees it. This lane did not investigate it.
