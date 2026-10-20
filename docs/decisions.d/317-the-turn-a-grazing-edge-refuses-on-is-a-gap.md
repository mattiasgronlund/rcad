## 317. The turn §244.3's grazing edge refuses on at `cx = 16` is a gap — 2026-09-28

*Queue row Q113, from §248.5's last bullet. Lane `lane-q113-seam-picker-orient3d`,
off `7de56c47`. §244.3's block edge along `x`, grazing a ball centred at
`x = 16`, refuses under `Policy::certified` on an `orient3d`. §248.3 read that
`orient3d` as a question against the plane of the ball's seam picker. **It is
not.** It is `Arrangement::turns_anticlockwise` in `fuse/arrange.rs`, asking
which way two crossings turn at the place below `x = 16`, and its two exact
rows are the block's own two normals. The sign it needs exists and is `2⁻⁵¹`
from zero, and the same three surfaces solved about the ball's centre reach it.
**A gap, not a limit** (§16.10). Pinned and measured in
`crates/rcad-topo/tests/a_block_edge_grazing_a_ball_at_cx_16.rs`. No `src/` was
changed.*

### 317.1 Where it refuses

Reproduced on the base with §244.3's shape (`mise exec -- just t -p rcad-topo
--test a_block_edge_grazing_a_ball_at_cx_16 --no-capture`, with a probe test
that printed every record). All three rules refuse with one record:

```
cx=16 certified Union: Undecidable
    orient3d Interval Undecidable { enclosure: 1.7763568394002505e-15 } [Constructed([[-1.7763568394002505e-15, 0.0], [1.0, 1.0], [-2.9802322387695313e-8, -2.9802322387695313e-8]]), Exact([0.0, 0.0, 1.0]), Exact([0.0, 1.0, 0.0])]
```

The call site was found with a backtrace, not read off the record. A copy of the
tree in the scratchpad had `must_decide` in `fuse/mod.rs` print
`Backtrace::force_capture()` whenever an `orient3d` refused; that copy was never
in this worktree. The first refusal's stack, innermost first:

```
must_decide              fuse/mod.rs
turns_anticlockwise      fuse/arrange.rs:2333
half_of_the_turn         fuse/arrange.rs:2265
sorted_anticlockwise     fuse/arrange.rs:2205
walk_the_cycles          fuse/arrange.rs:2131
Arrangement::of          fuse/arrange.rs:838
split_a_face             fuse/arrange.rs:114
split_both_groups        fuse/mod.rs:421
```

`turns_anticlockwise` works out §17.4's determinant of three gradients at a
node: the face's surface, then the partner of each of the two crossings. Here
the face is the ball's, so the rows are:

- `gradient_at(ball, place)`: the place's box minus the centre, `(16, 0, 0)`.
  The place below `16` is the box `[15.999999999999998, 16]`, so this row's `x`
  is `[−2⁻⁴⁹, 0]`.
- `(0, 0, 1)`: the block's top, `z = −2⁻²⁵`.
- `(0, 1, 0)`: the block's side, `y = 1`.

The two exact rows lie in `x = 0` because the edge runs along `x`. That is
where §248.3's reading came from, since the seam's picker is a half-space on
`x = 0` too. The seam's picker is not asked anywhere in the stack.

### 317.2 The sign exists

The determinant of the rows `(x, 1, z)`, `(0, 0, 1)` and `(0, 1, 0)` is `−x`.
At the true place below `16`, `x` is the place minus the centre, `−2⁻⁵¹`, so
the turn is `+2⁻⁵¹`. At the place above it is `−2⁻⁵¹`. Neither is zero. The box
row `[−2⁻⁴⁹, 0]` holds both `−2⁻⁵¹` and `0`, and that is the only reason the
interval level cannot tell.

`the_sign_that_turn_needs_is_there_when_the_ball_is_solved_about_its_own_centre`
measures it through the public API. It solves the block's edge against the ball
twice with `meet_of_three_surfaces`, once with the ball centred at `(16, 0, 0)`
and once with it at the origin, and asks `orient3d_of_points` the turn's question
with the rows in the order the record has them:

| ball's centre | place | gradient's `x` | turn |
|---|---|---|---|
| `(16, 0, 0)` | `[15.999999999999998, 16]` | `[−2⁻⁴⁹, 0]` | `Undecidable`, `1.78e-15` wide |
| `(16, 0, 0)` | `[16, 16.000000000000004]` | `[0, 2⁻⁴⁸]` | `Undecidable`, `3.55e-15` wide |
| origin | `(−2⁻⁵¹, 1, −2⁻²⁵)`, one `f64` triple | `−2⁻⁵¹` | `Pos` |
| origin | `(2⁻⁵¹, 1, −2⁻²⁵)`, one `f64` triple | `2⁻⁵¹` | `Neg` |

The two are one problem. Both planes' normals have no `x` part, so moving the
ball `16` along `x` leaves the planes' four numbers exactly as they were. The
test asserts that, and that each place about the origin lies inside exactly one
of the two places about `16`. The turn reads only the place minus the centre,
so it is the same number in both rows of a pair.

### 317.3 Why that makes it a gap

§16.10 asks whether the answer exists in the numbers given, and whether some
representable question reaches it. Both hold:

- **The answer exists.** The ball's centre, its radius `1 + 2⁻⁵¹`, and the
  planes `y = 1` and `z = −2⁻²⁵` are all `f64`. The turn at each place is
  `±2⁻⁵¹`. It does not need the place itself to be an `f64`, and at `16 ± 2⁻⁵¹`
  it is not one.
- **A representable question reaches it.** The same meet solved about the
  ball's centre is an exact shift of this one, and there the two places are
  `f64` triples and the turn is decided.

What loses the sign is the order of the arithmetic. The place is rounded out
to a box around `16`, which is `2⁻⁴⁹` wide, and only then is the centre taken
off. The difference the turn needs is a quarter of that width.

### 317.4 The question a fix should ask

For a fix lane. Not built here; this row writes no `src/`.

**Call site:** `Arrangement::turns_anticlockwise`, `fuse/arrange.rs`, the
`gradient_at(&self.surface, at)` row. The same function takes both partners'
gradients the same way, and on a plane face the ball is a partner. The first
row is always the face's own surface, and this shape's tolerant run records the
same turn with the ball's row second or third, so the plane faces' arrangements
ask it too.

**The question it should ask:** the gradient of a round surface at a meet
should come from the meet as it sits relative to the surface's own centre, and
not from the world box with the centre subtracted afterwards. Two ways to get
that:

1. **Solve the meet about the centre.** Measured in 317.2. The leg it rests on:
   the shift has to be exact. It is here because the planes do not move under a
   shift along `x`. In general a plane's offset moves by `n · c`, which can round,
   and a sign proved about a rounded shift is about a different problem. The fix
   has to check the shift is exact, or fall back to what it does now.
2. **Read the sign off which root the place is.** Not measured. For a quadric
   `Q` and two planes whose line runs along `d = n₁ × n₂`, the turn's
   determinant is `∇Q(p) · d`. That is the slope of the quadratic
   `q(s) = Q(p₀ + s d)` at the root, and at a simple root it is `±√disc`: positive
   at the larger root when the leading coefficient is positive. So where the
   discriminant is proved positive, the sign is the root's index times the sign
   of the leading coefficient, with no box in it. Its legs, none checked here:
   two separate places from `meet_of_three_surfaces` mean it proved the
   discriminant positive; `gradient_at` is a positive multiple of `∇Q` for each
   round surface (for a ball, `p − c` is `∇Q / 2`); the order of
   `meet_of_three_surfaces`' places along `d` can be recovered by the caller;
   and the rows' order in `turns_anticlockwise` changes the sign only the way a
   determinant's row swap does.

**The test to flip:** `a_certified_fuse_refuses_on_the_turn_at_the_place_below_16`
in the new file. It pins the refusal by its record and goes red when the turn is
answered. 317.5 says what it should expect next.

### 317.5 What the fuse does once the turn is answered

Measured on the same scratchpad copy, with a probe that swapped in the true
place for the two boxes: `x = −2⁻⁵¹` for `[−2⁻⁴⁹, 0]` and `2⁻⁵¹` for
`[0, 2⁻⁴⁸]`. It ran with `Q113_FORCE=1` on §244.3's shape at `cx = 16`.

- Under `Policy::certified`, all three rules refuse on **`order along a
  crossing`** about the two places, `5.329070518200751e-15` wide. That is
  §248.3's refusal on the edge along `z`, so the two shapes then agree.
- Under `Policy::tolerant(25.0)`, all three rules return
  `CannotTellTheCellsOfAFaceApart`, the same as the shape returns at `cx = 0`
  with no probe.
- The new file's pin fails under that probe, on `Union`, with the
  `order along a crossing` record. So the pin is not vacuous.

The edge along `z` has the same gap. Under `Policy::tolerant(25.0)` at `h = 16`,
`a_block_edge_grazing_a_ball.rs`'s shape records `orient3d` refusals resolved to
`Zero`. Their ball row is `z ∈ [−2⁻⁴⁹, 0]`, and their exact rows are
`(1, 0, 0)` and `(0, −1, 0)`. The certified fuse there never reaches the turn,
because `order along a crossing` refuses first. So that file's choice of edge
still does what §248.3 wanted, for a reason other than the one it gave.

### 317.6 What this leaves

- `crates/rcad-topo/src/fuse/arrange.rs`, `turns_anticlockwise`: the gap in
  317.4, as a row for a fix lane.
- `crates/rcad-topo/tests/a_block_edge_grazing_a_ball.rs`: its module doc repeats
  §248.3's reading, that the edge along `x` refuses "on the plane that seam is
  picked out by". It should name `turns_anticlockwise`. That file was outside
  this lane's write set.
- §244.3's shape at `cx = 0` refuses under `Policy::certified` on `order along
  a crossing` at `Interval`, with an enclosure of `0.0`, about two **exact**
  places, `(±2⁻⁵¹, 1, −2⁻²⁵)`. The numerical-refusal skill says that an
  `Interval` stop on exact input is suspect, since level 3 should have taken it.
  Seen in the same probe run, and not looked at.

### 317.7 What closed it — 2026-10-05

*Q914, lane `lane-q914-a-turn-read-off-its-root` off `96681fd6`, after Q699
and Q902 (§406.3).*

**Measured on the base first**, with a probe that printed every record. At
`cx = 16` under `Policy::certified`, every rule with the ball first refused on
317.1's turn, `orient3d`, `2⁻⁴⁹` wide; with the block first, on `order along
a crossing` between the two places along the block's edge, `5.33e-15` wide.
Under `Policy::tolerant(25.0)` every rule, either solid first, was
`ResultIsNotManifold { count: 3 }`. The edge along `z` of
`a_block_edge_grazing_a_ball.rs` at `h = 16` refused under `Policy::certified`
on `order along a crossing` with either solid first; a print in the order
showed every such question asked round a circle the block's faces cut the
ball in, never along the edge. Its tolerant run folds the two meets, records
`orient3d` resolved to `Zero` at the fold, and refuses `a point in a piece that
every solid could place`, as that file's doc says.

**The turn: 317.4's second way, built by §406.3 and opened to a curved face.**
`the_turn_where_two_planes_cross_a_quadric` stopped where the curved surface is
the face. §406.3's argument does not use which of the three the face lies on:
the determinant is a fact about three gradients at one point, it is
`∇Q · (n₁ × n₂)` up to the row the curved one is in (the first row, for the
ball's own face, with no turn of sign), and the sign is proved by two exact
signs of `Q` along the planes' line, at the true meet; the node's box only
picks which of the line's two meets that is. What a curved face adds is that
the turn is taken about its own gradient, which must not be nothing at the
node. It is not wherever the route answers: it answers only where the sign
along the line changes once across a side of the box, which is a simple root,
where `∇Q · (n₁ × n₂)` is not nothing. A cone's apex, where `∇Q` is nothing,
is a double root of every line through it, or the line lies along a ruling, so
the route stays silent there. And still only at a node that stands for one
meet (§406.3). The stop is gone, and
`the_turn_on_a_balls_face_where_a_block_edge_grazes_it_is_read_off_the_root`
in `fuse/arrange.rs` checks the two signs at `16` against the same shape at
`0`, where the places are exact and the determinant decides.

**The order along the edge: a gap, decided by the quadric at the shared end.**
The two places' boxes along the edge's `x` are `[15.999999999999998, 16]` and
`[16, 16.000000000000004]`, each already between two neighbouring `f64`, so
`MeetOfThree::boxed_tight` cannot narrow them apart. But the edge's point at
`16` is `2⁻¹⁰³` inside the ball, proved by the exact sign §406.3 reads along
the line, so neither place is at `16`, and the one whose box ends there comes
first along `x`. `Arrangement::as_the_quadric_says` reads that for two nodes
on a straight crossing whose boxes share one end along its axis, where a node
whose three surfaces are the face's plane, the partner plane and a quadric
stands for one meet and the line's point at that end is off the quadric
(`MeetOfThree::the_line_is_off_the_quadric_at`). The row named another way, by
which root each place is: `meet_of_three_surfaces` numbers its two places by
the step along `(through + s·direction) / minor`, and `minor`'s sign is picked
by a ranking in `f64`, so which way `which: 0` lies along the edge is one more
leg to prove, and the shared end needs none of it. It reads the nodes' own
boxes; two meets whose boxes overlap by more than one end are not reached by
it, and none has been seen.

**What each fuse gives now.** At `cx = 16`, every rule, either solid first,
under either policy, refuses `Undecidable` with one record, `a point inside a
piece of a face`, whose inputs are the two places. No fuse answers, so nothing
is held to a volume. `every_fuse_refuses_on_a_point_inside_a_piece_between_the_two_places`
pins all twelve; it was `a_certified_fuse_refuses_on_the_turn_at_the_place_below_16`.
The edge along `z` at `h = 16` is unchanged under either policy: its order is
asked round the circles, which the shared-end rule does not reach.

**What this leaves.**

- At `cx = 16`, `a point inside a piece of a face` between the two places,
  every fuse. §365.3's rule places the same slivers on the edge along `z` at
  `h = 0`; why it does not place these is not traced. Refuses/B.
- On the edge along `z` at `h = 16`, under `Policy::certified`, `order along a
  crossing` between the two meets round each circle the block's two faces cut
  the ball in, with either solid first. Q699 orders two such meets where the
  walked start lies between them; here it does not. Refuses/B.
- Q909 traces the tolerant results, which moved: at `cx = 16` every fuse is now
  the certified refusal above, not `ResultIsNotManifold { count: 3 }`.

> **2026-10-10 note (Q1073): the sliver between the two places is placed, and
> every fuse now refuses on a limit under the wrong name.** The first item
> above is closed. The two places `16 ∓ 2⁻⁵¹` come out one corner,
> `(16, 1, −2⁻²⁵)`, so both arcs of the ball's circle through them run from
> that corner back to it, and `Edges::of` named both as one closed edge. §365.3's
> side rule then matched the ball's sliver to the block's top along the long
> arc, inside the block, and to its wall across its true side, outside it, and
> `select` refused the sliver. `the_edge_each_side_of_every_piece_is`
> (`fuse/assemble.rs`) now names such a side, whose two ends have two names but
> one corner, by those two names and which way it runs between them
> (`which_arc_of_a_circle_walked_this_way`), so the two arcs are two names. That
> only splits names `Edges::of` joined, so the rule can lose a match and never
> gain one. `fuse::select`'s
> `the_sliver_of_a_ball_between_two_places_one_corner_apart_is_placed_by_its_side_on_the_wall`
> fails without it.
>
> Every fuse then refuses while the answer is put together, the same under
> `Policy::certified` and `Policy::tolerant(25.0)`. The union, either solid
> first, and the ball less the block are `ResultIsNotManifold { count: 4 }`: the
> answer's own naming makes the two arcs one closed edge again, with four faces
> along it. The intersection, either solid first, and the block less the ball
> are `CannotMeshAFace` with `CornersCoincide { position: 5 }`, from the mesh in
> `nest_the_shells`: a face's loop runs into the one corner, round a closed edge,
> and out again. No fuse answers, so nothing is held to a volume.
> `every_fuse_refuses_on_the_two_places_f64_holds_as_one_corner` pins all
> twelve, in place of
> `every_fuse_refuses_on_a_point_inside_a_piece_between_the_two_places`.
>
> **This is a limit refused under the wrong name.** The answer needs a corner at
> each place, and `f64`'s spacing at `16` is `2⁻⁴⁸`, so it holds neither. The two
> would make an edge `2⁻⁵⁰` long in a shape whose coordinates run to `18`, a
> spread near `2⁵⁴` against the supported `1e11`, and scaling the shape by a
> power of two moves both lengths alike. The refusal it should give is
> `Invalid::DynamicRangeTooWide`. The fuse's `check_dynamic_range` does not give
> it: it skips an edge whose two ends are one corner, and it runs after
> `group_into_shells` and `nest_the_shells`, where both refusals come from. A
> row asks for the right name where a kept side's two ends have two names and
> one corner, and for what a tolerant policy should give there.

> **2026-10-20 note (Q2193): every fuse at `cx = 16` is refused by name before
> the answer is put together.** `Corners::of_both_ends` (`fuse/assemble.rs`)
> refuses a side on a curved surface whose two ends of two names land on one
> corner, `Error::TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint` at
> `(16, 1, −2⁻²⁵)`, all twelve fuses under either policy
> (`every_fuse_refuses_on_the_two_places_f64_holds_as_one_corner`). The name
> is not `Invalid::DynamicRangeTooWide`; the variant's doc says why it does not
> claim a limit, and queue row Q1915 still asks for that name.
