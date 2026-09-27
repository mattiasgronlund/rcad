## 286. A corner with fewer than three surfaces round it is not asked about — 2026-09-27

*Queue row Q128, from §253.6. Lane `lane-q128-thin-corner-2`, off `ff78f99f`.
It restarts `lane-q128-thin-corner`, which the weekly usage limit killed with
only a probe written; the probe is used below and is not in the branch.
`place_a_corner` hands a corner with one or two distinct surfaces round it
straight out of the model. The question was whether it should grow that corner
and ask its surfaces about it, as `derive_a_corner` does for three or more.
**No.** Asking the way `derive_a_corner` asks would refuse a block
`Model::validate` accepts, and in the whole workspace it would refuse nothing
else. No code changed. The reason is pinned in
`crates/rcad-topo/tests/a_corner_with_fewer_than_three_surfaces_round_it.rs`,
and the `bake.rs` header, which said such a corner was already certified on its
surface, is corrected.*

### 286.1 What asking could do, and the answers weighed

`derive_a_corner` solves a new point from three surfaces and then asks every
surface round the corner about that point, grown by
`what_writing_the_corner_down_cost`. A sign is a refusal:
`Invalid::VertexOffItsFacePlane` or `Invalid::VertexOffItsFaceSurface`. The
check is there because the solved point is new. The fourth surface never saw it.

With one or two surfaces there is nothing to solve. Two surfaces meet along a
curve, and the whole curve satisfies both (§31.3). So the point handed on is
the stored corner, and it stays the stored corner whatever an ask says. **An ask
here cannot move a point. It can only refuse.** That makes the family of
answers the choice of how far off its surfaces a stored corner may sit before
the fuse refuses it:

- **(a) Any distance: do not ask.** This is the code today.
- **(b) The corner's own rounding**: grow by `what_writing_the_corner_down_cost`,
  `MEASURING_SLACK × Plane::SIGNED_DISTANCE_ROUNDOFF × max(size, 1)`, which is
  `64ε × max(size, 1)`. This is what §253.6 asked about.
- **(c) What `Model::validate` allows**: for a flat face, `1e-9 × size` of
  modelling tolerance plus `64ε × size` (`allowed_wobble`); for a curved face,
  `16ε × size` or the slop recorded on the corner, whichever is wider
  (`not_provably_off`). In both, the size is the largest coordinate of the model
  and not of the corner.
- **(d) Solve the corner from something else.** There is nothing else. Fewer
  than three distinct surfaces is the definition of this branch, and the stored
  corner is the one number left.

The stored corner is the same point `validate` asks the same surfaces about. So
every choice of allowance is one of two kinds. Either it is narrower than (c)
somewhere, and there it refuses a solid `validate` accepts. §75.6 says the fuse
must be able to bake such a solid. Or it is at least as wide as (c)
everywhere, and then it refuses only solids `validate` refuses. (b) is the
first kind, and 286.3 is a block it refuses. (c) is the second kind.

**(c) was not built.** It changes an answer only for an input `validate`
rejects, and `Model::fuse`'s `# Panics` says such an input is not checked, in
any build. It would be `validate`'s own question, asked again at the door with
the same allowance. Whether a broken input with a corner like this can come out
of a fuse as a wrong `Ok` is a different question, and it was not measured
(286.4).

### 286.2 What the workspace hands on unasked, measured

The killed lane's probe went into `place_a_corner`, just after the
`distinct.len() >= 3` branch, behind `RCAD_Q128_PROBE`. For every corner with
fewer than three distinct surfaces, it appended one line to a file. The line held
the surface kinds, the policy, the test's name, and each surface's
`side_of_surface` answer on the stored corner and on the stored corner grown by
`what_writing_the_corner_down_cost`. The core of it:

```rust
let room = what_writing_the_corner_down_cost(where_the_model_puts_it.enclosure());
let grown = grown_by(where_the_model_puts_it, room);
let bare: Vec<Decision> = distinct.iter().map(|s| side_of_surface(s, where_the_model_puts_it)).collect();
let asked: Vec<Decision> = distinct.iter().map(|s| side_of_surface(s, grown)).collect();
let bare_off = bare.iter().any(|d| matches!(d, Decision::Pos | Decision::Neg));
let asked_off = asked.iter().any(|d| matches!(d, Decision::Pos | Decision::Neg));
// one line per corner: kinds, bare_off, asked_off, *policy == Policy::certified(),
// and the test name read from the argument after `--exact`
```

`RCAD_Q128_PROBE=<file> mise exec -- just t --workspace` on `ff78f99f` with the
probe:

```
     Summary [ 115.920s] 2535 tests run: 2535 passed (3 slow), 12 skipped
corners handed on unasked (one line per corner per bake)   45 856
  under Policy::certified                                   45 290
  under Policy::tolerant                                       566
  stored corner proved off a surface, bare (certified)         305
  stored corner proved off a surface, grown                      0
```

- **Growing the corner and asking, as (b) does, refuses nothing in the
  workspace.** No corner under either policy is proved off a surface once it is
  grown.
- **305 certified corners are proved off a surface before they are grown.**
  They come from ten tests: 168 on a plane and a cone, 84 on two planes, 36
  on a cone alone and 17 on a ball and a plane. All 84 two-plane lines come
  from `every_kicad_file_that_imports_round_trips_to_the_same_solid`, so
  imported flat parts do carry corners with only two planes round them.
- The figures in the table are lines of the log, one per corner each time a
  solid is baked, and not distinct corners.

### 286.3 A block `validate` accepts that (b) refuses

`a_block_with_a_corner_off_its_edge_by_less_than_validate_allows_is_cut_and_not_refused`
builds `[0, 2]³` face by face with `SolidBuilder`. Its top front edge runs from
`(0, 0, 2)` through an extra corner at `(1, 1e-9, 2)` to `(2, 0, 2)`. That corner
has only the top face and the front face round it, so it goes down this branch.
It sits `1e-9` off the front face's plane.

- `validate` (inside `SolidBuilder::build`) accepts the block. Worked out from
  `allowed_wobble`, not measured: at a largest coordinate of two and a 100 mm
  scale it is `min(1e-7, 2e-9) + 2.84e-14`.
- Grown by the `2.84e-14` that (b) grows it by, the corner is still proved off
  the front plane. The test asserts that first, so a pass cannot come from the
  question having become agreeable.
- The certified difference with the unit cube `[1.5, 2.5]³` is one solid of
  volume `8 − 1/8`, to within `1e-8`, and the model validates afterwards.

With (b) patched into `place_a_corner`:

```rust
let asked_with_room = grown_by(
    where_the_model_puts_it,
    what_writing_the_corner_down_cost(where_the_model_puts_it.enclosure()),
);
for surface in &distinct {
    match side_of_surface(surface, asked_with_room) {
        Decision::Zero | Decision::Undecidable { .. } => (),
        Decision::Neg | Decision::Pos => {
            return Err(match surface {
                Surface::Plane(flat) => Invalid::VertexOffItsFacePlane {
                    distance: flat.signed_distance_to(middle_of(where_the_model_puts_it)).abs(),
                },
                _ => Invalid::VertexOffItsFaceSurface,
            }
            .into());
        }
        other => panic!("{other:?}"),
    }
}
```

the test fails on the fuse:
`Invalid(VertexOffItsFacePlane { distance: 1e-9 })`. **So asking moves a
result, and it moves a right answer to a refusal.** Nothing else in the
workspace moves. `mise exec -- just t --workspace --no-fail-fast` with the
patch in reported
`2536 tests run: 2535 passed (3 slow), 1 failed, 12 skipped`, and the one
failure was this test. That matches 286.2's count of zero corners proved off
once grown.

The corner is `3.5e4` times the room (b) grows it by, and `validate`'s allowance
here is mostly the modelling tolerance. A flat corner is the widest case for
that reason. On a curved face, (b)'s room `64ε × size` is wider than
`validate`'s `16ε × size` when the corner is the model's largest coordinate. It
is narrower when the corner sits more than four times closer to the origin than
the model's far end, or when the corner carries slop from an earlier fuse. That
is §136.4's shape, and no curved case was built here.

### 286.4 What this leaves

- **A corner with three or more surfaces round it is refused for the same
  modelling tolerance.** `the_place_the_model_points_at` grows the stored corner
  by only the rounding, so a three-plane corner `1e-9 × size` off one of its
  planes places nothing, and the fuse refuses `Undecidable`. That is §136.4's
  gap, widened from the plane's rounding to the modelling tolerance
  `validate` also grants. It was not measured here.
- **A broken input.** `Deserialize` does not call `validate` (`serialize.rs`),
  so a model whose two-surface corner is far off its surfaces can reach a fuse
  through the public API. What the fuse then does, whether a wrong `Ok`, a
  `TheFuseWroteABrokenAnswer` blaming the fuse, or a panic, was not measured.
  That measurement is what would argue for (c).
- **The header was wrong, and is corrected.** `bake.rs` said
  `check_curved_face` had already certified the stored corner `Decision::Zero`
  on its surface. `validate` asks for "not provably off within an allowance",
  not for `Zero`, and 305 certified corners in 286.2 are not `Zero`. Nothing
  runs `validate` before `bake` either (§75.6).
