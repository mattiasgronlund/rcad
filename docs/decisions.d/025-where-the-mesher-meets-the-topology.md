## 25. Where the mesher meets the topology

*Recorded 2026-09-01, before the lanes fan out. §15 settled the shared meanings
before M1 and M2.5, §23 before M2's lanes and §24 before M3's stages; this is
the same, for §13 rule 4's reason. Three quarters of M2 has landed. What is left
is the seam between `rcad-mesh` and `rcad-topo`, plus curved `measure`, and it
is about to be four people.*

### 25.1 The boundary is refined by whoever can see both sides of it

**Settled: `rcad-topo` refines every boundary. `rcad-mesh` refines only the
inside of a patch and never adds a point to a loop it was handed.**

§23.4: edge points are worked out once and the same 3D points go to both faces
along that edge, bit for bit. `rcad-mesh` is shown one face at a time, and its
`Cargo.toml` makes that a fact the compiler checks (§23.6), so it cannot know a
loop is shared. If it subdivided one, the two faces along an edge would each
refine in their own `(u, v)` and land on different 3D points — and not by a
rounding. A cylinder's inverse and a plane's are different functions of the same
edge, so the two sides need not even produce the same *number* of points, and
§23.4's oracle would fail by a count rather than by a distance.

Rejected: hand the mesher both faces at once. That is topology in the one crate
whose whole value is not having any, and it costs the oracle that needs no
solid — §13 rule 1's condition, and the reason `rcad-mesh` could be built beside
the rest instead of after it.

Rejected: let the mesher subdivide and have the caller weld coincident points
afterwards. Welding is a tolerance decision on constructed points, which is a
place to manufacture the undecidability §1.7 exists to measure — §15.5's mistake
arriving by a new road. An index cannot drift and costs nothing.

So the mesher may assume **a loop it is given is final**: every corner is a point
the caller means to see in the mesh, it hands every one of them back at the index
it came in at, and it adds nothing to any loop. `Patch`'s own doc already
promises the index half. Points it adds go strictly inside the patch and on the
end of the corner list.

One consequence, because it looks wrong otherwise. A refined boundary point is
**not** certified onto its surface and does not have to be: it comes out of an
evaluator and carries a deviation. `validate`'s "every corner of a curved face is
certified onto its surface" is about vertices stored in the model, not about
points in a mesh, and running the two together would be asking a picture to be as
true as the geometry it is a picture of.

### 25.2 Which of a surface's own two coordinates a point sits at

`Surface::point_at` has no inverse, and every lane below needs one.

```rust
impl Surface  { pub fn coordinates_of(&self, point: Point) -> (f64, f64) }
impl Sphere   { pub fn coordinates_of(&self, point: Point) -> (f64, f64) }
impl Cylinder { pub fn coordinates_of(&self, point: Point) -> (f64, f64) }
```

`Plane::plane_coordinates_of` already exists and keeps its name; the `Surface`
arm calls it.

**Bare numbers. No `Decision` and no `Result`.** It decides nothing and cannot
fail. §23.4 says `(u, v)` is a working space and never an answer, and a predicate
is for answers; handing one back here would invite a caller to take a decision on
projected coordinates, which is exactly what §15.5 forbids.

**It answers about the nearest point on the surface.** A point off the surface
gets its projection's coordinates, which is what `Plane::plane_coordinates_of`
already does, so this is one rule rather than two. The axis of a cylinder and the
centre of a sphere have no nearest point, and there the answer is `(0, 0)`,
stated rather than refused: a caller that reaches it is already broken, and a
number is cheaper than a branch nobody takes.

**What a round-trip test may assert:** for a point *on* the surface,
`point_at(coordinates_of(p))` is within a stated multiple of `radius · ε` of
`p` — the bound a named public constant beside the expression, with a test that
could fail, per §15.6 and its amendment. What it may **not** assert: bit equality
in either direction, or `coordinates_of(point_at(u, v)) == (u, v)`, which is
false for any `u` outside the branch below and undefined at a pole.

**The seam.** `u` comes back on one `atan2` branch, `(-π, π]`, chosen once and
said out loud. A point exactly on a cylinder's seam — `y = 0`, `x > 0` in
`Cylinder::basis` — comes back `u = 0` and never `2π`, because
`atan2(+0.0, positive)` is `+0.0`. So a point on the seam has one answer here and
not two.

**A wire that goes all the way round is the caller's problem and not the
surface's.** Whoever walks a loop is the one that knows it is a loop, so the
unwrap lives in `rcad-topo`: after taking each corner's `u`, add or subtract a
turn so that no two corners in a row jump by more than half a turn. A ball's face
then comes out as the rectangle §25.6 needs, with its two seam sides at `u = 0`
and `u = 2π` instead of both at zero.

Rejected: have `coordinates_of` take the previous `u` and carry on from it. That
makes a surface's answer depend on the order it was asked in, which is the shape
of bug §19.1 and §23.8 keep finding.

### 25.3 The mesher reports a length and never judges it

```rust
pub fn triangulate(domain: &TrimmedDomain) -> Result<Patch, CannotTriangulate>
impl Patch { pub fn deviation(&self) -> Approximation }
```

The signature does not change. What changes is that a `Patch` carries a number.

**A length, in the model's own unit, in an `Approximation`** — §15.2 and §23.5 —
with `Cause::FlatStandIn`, which §25.8 takes here. Never in `(u, v)`: a `u` on a
pipe of radius 1000 and a `u` on one of radius 0.001 are not the same length, and
§23.4 says `(u, v)` is never an answer.

**One number per patch**, the furthest any point of any triangle can be from the
surface. Not one per triangle: §5 records provenance per decision and not per
coordinate, and §24.13 says the same about the evaluator for the same reason.

**`triangulate` does not take a `&Policy`, and that is the decision rather than
an omission.** §12.5's convention gives a policy and a log to an operation that
can make a non-certified *decision*, and this one makes none — it refines until
its own bound is under the `Deflection` it was handed. Whether that is good
enough is a judgement, it belongs to the caller, and the caller is in `rcad-topo`
where the policy already is. A mesher that judged would mean someone who asked
for a rougher picture had silently asked the kernel to certify less, which is the
conflation §23.5 exists to stop.

**There is no best-effort answer.** `triangulate` meets the deflection or
refuses. A patch that quietly missed it and reported the miss would need every
caller to compare two numbers, and none would. The refusal is about the request —
this deflection needs more triangles than this domain can hold — and never about
the surface; naming it is the lane's.

Then what is the number for, if a successful call always comes in under the
deflection? Because it is usually far under, and because the caller has to put a
real figure on the face it built (§15.2) rather than the ceiling it asked for.

### 25.4 A volume error is not a length, and `Measurement` says so in three units

```rust
pub struct Measurement { /* volume, volume_error, surface_area, area_error,
                            centroid, centroid_wobble */ }

impl Measurement {
    pub fn volume_error(self) -> f64;     // cubed unit
    pub fn area_error(self) -> f64;       // squared unit
    pub fn centroid_wobble(self) -> f64;  // length
}
```

**Only `centroid_wobble` is judged against `Policy::max_wobble`.** It is the one
length, and §15.2 says a magnitude is how far a *point* could be out of place —
the centroid is a point and this is how far it could be out of place. The other
two are reported and never judged: a budget in lengths says nothing true about a
cubic unit, and cramming one into an `Approximation` is §12.4's mistake, which
§23.5 refused in advance.

The signature the lane lands:

```rust
pub fn measure(
    self,
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Measurement>
```

A `Deflection` because the answer is only as good as the mesh it is summed over,
and a deflection read from anywhere but the call site is §2.6's footgun. A
`Policy` and a log because it can now make a non-certified decision, which is
§12.5's rule.

**For a wholly flat solid the deflection is ignored**, the exact fan is summed
exactly as it is today, and all three errors come back exactly `0.0`. That is a
claim with a test behind it, and it is what keeps M0's numbers bit for bit.

Rejected: keep `measure()` and add a second `measure_meshed(&Mesh)`. Two
functions answering "how big is this" is two answers to one question, and §23.8
is four instances of what happens when one of them stops being right.

**Not taken at this gate**, which is a change from the plan that opened it.
Adding the fields now would put three of them and one argument into a public type
with nothing producing any of them, tested only by asserting the zero they were
initialised with — §12.6's first gap exactly, the mistake §13 was written after.
`Measurement` and `measure` live in one file that one lane owns; the shape above
is the contract, and that lane lands it in one go.

**Amended 2026-09-01, while implementing. "Judged" cannot mean "refused on", and
the arithmetic is what says so.** `Policy::max_wobble` is `64·ε·working_size` —
`1.4e-12` for a 100 mm model — and `0.0` under `Policy::certified`. A mesh of a
ball of radius three at a deflection of `0.1` leaves a centroid wobble of about
`3e-1`, and the wobble falls roughly in step with the deflection, so meeting that
budget would need a deflection near `5e-13`. A ball asked for one comes back
`MoreMeshPointsThanASolidWillHold`. So
`policy.wobble_is_acceptable(centroid_wobble)` is `false` for **every** curved
solid at **every** deflection that can be met, and a `measure` that refused on it
would have replaced §25.5's blanket refusal with another wearing a hat — the lane
would have delivered nothing.

It is also the conflation §23.5 forbids, arriving from the other side: the
*deflection* would be deciding what the *policy* certifies, so a caller who asked
for a rougher picture would be refused an answer rather than given a rougher one.
§23.5's own division of labour is the one that holds — the deflection says how
fine, and the policy decides what happens when the deviation cannot be *proven*
under it, which is the floor `SolidView::mesh` already asks it about.

So what the sentence above means is that `centroid_wobble` is the one of the
three that is *commensurable* with `Policy::max_wobble`, and comparing them is
the caller's to do on a number handed back for exactly that. `measure` passes
`policy` and the log through to `mesh` and refuses nothing of its own.
`judging_a_meshed_centroid_against_the_policy_would_refuse_every_ball` is the
demonstration rather than the paragraph, per `CLAUDE.md`.

### 25.5 Until then, `measure` refuses a curved solid

§24.4 states as a premise that `is_wholly_flat` gates `measure`. **It does not,
and never did.** `classify` and `fuse` ask it; `measure` was left out when §23.8
item 4 added the shared check, and nothing in the suite asked.

What happened on a round bar was neither a refusal nor an answer. Its caps fan
into no triangles at all — a one-corner wire has no polygon — and its wall into
two triangles with no area, so `six_volumes` comes out a clean `0.0` and
`measure` dies on its own `assert!(six_volumes > 0.0)`. A ball does the same. The
assert is right that a solid enclosing nothing is a broken model; it is wrong
about which of the two it was looking at.

**Settled: `measure` asks `is_wholly_flat` first and returns
`Error::MeasureNeedsFlatFaces`**, alongside `Error::FuseNeedsFlatFaces` and for
the same reason — the solid is sound and the operation is what is not ready for
it. Deliberately not `Error::Undecidable`: nothing here is in doubt. Asked over
every face before anything is summed, so which answer comes back cannot depend on
the order the faces were reached.

Landed at this gate, with a test. It comes back out when §25.4's lane lands.

### 25.6 A pole is a side of the rectangle, not a repeated corner

`Loop::through` refuses fewer than three corners and `triangulate` refuses two
corners in the same place, so a ball's face — a rectangle in `(u, v)` whose top
and bottom edges are each one point in 3D — has to be handed over in some shape
that survives both.

**Settled: the caller hands over the rectangle, and a pole is a *side* of it
whose corners share one 3D point.**

A ball's one face is four wire steps: down the seam, across the south pole, up
the seam, back across the north pole. In `(u, v)` that is four corners —
`(0, π/2)`, `(0, −π/2)`, `(2π, −π/2)`, `(2π, π/2)` — three or more of them, all
different, so `Loop::through` and `triangulate` take it as it stands. The two
`u`s differ because §25.2's unwrap made them differ.

**Identity is the caller's table and never the mesher's index.** Two corners at
different `(u, v)` may be one 3D point, and both pole corners of that rectangle
are. The topo layer maps each `Patch` corner to a point index of its own and maps
every pole corner of a face to the same one, so §23.4's oracle — every mesh edge
used by exactly two triangles, opposite ways round — is a statement about indices
and comes out right. A triangle whose three corners do not map to three different
points is **dropped**: it has no area and covers nothing, and dropping it is what
turns the rectangle's top row into the fan a sphere's pole actually has.

What the mesher may refuse is what it refuses today and nothing new: fewer than
three corners, two corners in the same place, a boundary that crosses itself, a
corner of one loop sitting on another loop's side. It is **not** entitled to
notice that two different `(u, v)` are one point, because it cannot see 3D at all.

Rejected: a pole as one repeated corner, with the mesher taught to allow it. That
asks a crate with no topology to tell which repeats are a pole from which are the
broken input `CornersCoincide` exists to catch — the same overloading §23.3
turned down for zero-length edges, and it makes that check unaskable the same way.

### 25.7 What a mesh of a solid is

§23.6 settled the two levels: `rcad-mesh` returns topology-free triangles and
`rcad-topo` pairs them with the key it asked about. This names the upper one.

`crates/rcad-topo/src/mesh.rs`:

```rust
pub struct Mesh { /* … */ }
impl Mesh {
    pub fn points(&self) -> &[Point];
    pub fn triangles(&self) -> &[MeshTriangle];
    pub fn deviation(&self) -> Approximation;
}

pub struct MeshTriangle { /* … */ }
impl MeshTriangle {
    pub fn corners(self) -> [u32; 3];
    pub fn face(self) -> FaceKey;
}

impl SolidView<'_> {
    pub fn mesh(
        self,
        deflection: Deflection,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Mesh>;
}
```

**One point table for the whole solid, and that is what watertightness *is*
here.** An edge's points appear in it once, and both faces along that edge name
the same index, so §23.4's oracle is a check on indices with no distance in it
anywhere. §23.4 called the claim combinatorial rather than metric; this is the
type that makes it so.

`mesh` takes a `&Policy` and a log where `triangulate` takes neither (§25.3). The
judging happens once, here, where the policy is.

**The order is fixed** — faces in stored order, and within a face whatever
`triangulate` gives back for a domain built the same way — because §9.2's
benchmark compares a serial run against a parallel one bit for bit, and every sum
built on a shuffled mesh would come out different.

The facade exports `Mesh` and `MeshTriangle`. §24.15's argument for the
intersector applies here for less work: the only topology in the signature is a
`FaceKey`, which is public already.

### 25.8 What landed at this gate

Each has a test that fails without it.

1. **`Cause::FlatStandIn`** (§25.3), taken here rather than in a lane. §15.9's
   precedent is exact: `Cause::ConstructedPoint` was taken before M1's worktree
   opened so that the freeze on `rcad-predicates` would be real rather than
   aspirational, and it cost a day. **`rcad-predicates` is frozen from here until
   the lanes close.** The variant goes **last** in the enum, because `postcard`
   writes an enum as its number: one inserted in the middle makes every
   `Approximation` already on disk read back blaming something else, silently,
   because the bytes still parse and the magnitude is still right. There is a
   test on the numbering.
2. **`measure` refuses a curved solid** (§25.5).
3. **`SolidView::bounding_box` is now `box_around_its_corners`** (§25.9).
4. **`Deflection` is exported from the facade.** Nothing out there takes one yet,
   and it goes out anyway: every signature this section writes down takes one,
   and three lanes would otherwise each add the same line to the one file they
   share. §15.9's table makes the facade's re-export list the shared, append-only
   thing precisely so that a fan does not contend over it, and pre-loading the
   one name all of them need is what a gate is for.

Not taken, and named here so that nobody takes them inside a lane: the
`Measurement` fields and `measure`'s new arguments (§25.4), and `Mesh` (§25.7).
Both would be public shapes with nothing producing them.

### 25.9 What the gate found that the plan did not

**`bounding_box` was not one, and had not been since a curved face became
legal.** §23.8 recorded four defects that were each a helper meaning one thing
until a new kind of input arrived; this is the fifth, and it was still in the
tree because §23.8 stopped at the ones that had already fired.

A round bar's two corners are both on its seam, so the box round them is a
**line**, while the bar bulges a whole radius out of it on every side. Nothing
crashed and nothing refused: a caller asking how big a bar is got a confident
wrong answer, which §4.1 ranks below the crash. A ball is worse — its one face
has two corners, two corners make no triangle, and the answer is `None`, which
the doc said meant "no faces at all". A ball has one.

Renamed `box_around_its_corners`, which cannot be misread, exactly as
`distance_between_its_ends` was (§23.8 item 3). A real box round a curved solid
needs the mesh and belongs to the lane that builds one. `classify` is the only
caller inside the kernel and it asks `is_wholly_flat` before it reads a box, so
nothing changes for it.

Also corrected: `rcad-mesh`'s crate doc still said the triangulator was not there
yet. It shipped in the same milestone that wrote the sentence.

### 25.10 Who owns what, and in what order

| Lane | What it is | Files it may touch | Opens |
|---|---|---|---|
| **A. Which coordinates a point sits at** | §25.2 | `rcad-geom/src/surface.rs`, `sphere.rs`, `cylinder.rs`, `lib.rs`, its own new test file | now |
| **B. Refinement, and the number for how far it strays** | §25.3 | everything under `crates/rcad-mesh/` | now |
| **C. The mesh of a solid** | §25.1, §25.6, §25.7 | `rcad-topo/src/mesh.rs` (new), `view.rs`, `lib.rs`, `rcad-topo/Cargo.toml`, `rcad-topo/tests/meshing_a_curved_solid.rs` (new) | when A lands |
| **D. Curved `measure`, and the two gaps behind it** | §25.4, §25.5, §23.9's first two | `rcad-topo/src/measure.rs`, `validate.rs`, `error.rs`, `rcad-topo/tests/measuring.rs`, `a_solid_with_a_curved_face.rs`, `rcad/src/lib.rs` | when C lands |
| `rcad-predicates` | nobody | frozen (§25.8) | — |

**Not a four-way fan, and the ordering is a dependency rather than a preference.**
A and B are in different crates and neither consumes the other: B refines inside
a patch, which needs `point_at` and a curvature bound and never the inverse. C
needs A's inverse to put a shared edge point into each face's own `(u, v)`, and
it is written against B's contract rather than B's code, so it needs B landed only
to run green. D needs C's `Mesh`.

That puts two lanes inside `rcad-topo`, which §13 rule 6 permits — it caps work
in flight in one crate at two or three — and they are in sequence rather than
side by side because the dependency is real. §24.12 made the same call for the
same reason.

`crates/rcad/src/lib.rs` is shared, re-export lists only, as §15.9's table has
it, **with one exception written down here so it is not a surprise**: the worked
example in it calls `measure()`, and §25.4 changes that signature. That edit
belongs to D. C hands its two re-export lines to D rather than editing the file.

§13 rule 5 wants the lanes' generators to differ so that two suites cannot go
green together for the same wrong reason. A draws points on spheres and pipes, B
draws polygons in a flat domain, and those two are clear of everything. **C and D
share the bar and the ball**, because those are the only two curved solids that
exist, and that is worth naming rather than glossing. What separates them is the
oracle: C's is combinatorial — every mesh edge used by exactly two triangles, and
χ of the mesh equal to χ of the solid — and D's is arithmetic, a bar's volume
against `πr²h`. No one wrong mesh satisfies both.

M2's closing gate writes **§26**; the number is reserved here so the lanes do not
collide over it.

---

