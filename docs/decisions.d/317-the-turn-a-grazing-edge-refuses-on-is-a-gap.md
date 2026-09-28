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
