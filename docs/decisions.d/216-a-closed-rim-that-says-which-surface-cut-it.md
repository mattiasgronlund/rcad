## 216. A closed rim that says which surface cut it, and two floors that were not where they looked — 2026-09-23

§0's *"the run does not say which plane cut the cone"*, half closed: the
vocabulary exists and the public builder's wrong answer is gone, and the fuse
still does not write it. Beside it, the two things §0 listed as also open turned
out to be one thing wearing two names, and neither is the gap §215.3 called it.
Three writing lanes off `dc90dd7a`, each verified by a Sonnet reader that reran
the claims.

### 216.1 The wrong answer was live, through the public API, at 1.45 mm

§215.1 wrote its guess down as a limit and said "why it is unreached is not
answered here". It asked the wrong population. §16.10 says a *"nothing reaches
this"* claim is about the **public API** and not today's callers, and the public
API reaches it in four faces.

A frustum whose wall is split into two bands by the leaning plane `x + 4z = 27`,
built through `SolidBuilder`, validates and meshes. Both bands sit on the one
cone, so the pair either side of the rim between them names no crossing, which is
exactly the run `what_a_run_between_these_two_lies_on`'s same-surface arm
answers — and it answers the circle through the rim's own corner square to the
axis. Measured, `just t --test merged -E 'test(/^a_leaning_cut_round_one_cone::/)' -- --no-capture`: the
rim comes back as **89 mesh points level with its own corner**, the furthest of
them **1.454760496437048** from the plane that was meant to cut it, on a 100 mm
solid. Not a refusal, not a rounding: a different curve, which is §4.1's worst
direction.

So it was never a limit waiting on the fuse. It was a defect anyone could reach,
and §215.1's "measured unreached today" was true only of the fuse — which is the
half it measured.

### 216.2 A fourth `EdgeRun`, and (d)'s literal wording would have been wrong

`EdgeRun::RightRoundWhereItCrosses { and: Surface }`: right round the closed
curve where the face crosses `and`, for when the faces either side are the same
surface and so name nothing. `and` does the same job it does in
`PartOfWhereItCrosses` and there is no `picked_by`, because the edge is the whole
of the curve rather than a piece of it.

**§214.4's (d) said to store it as `PartOfWhereItCrosses`, and that would have
broken six sites.** `EdgeView::is_closed` is the one copy of that rule and
answers `PartOfWhereItCrosses => Ok(false)`; a closed rim stored that way reads
as *not closed* at every one of its askers. A fourth kind that says it closes is
one word on that one line instead, and `tests/a_fourth_kind_of_run.rs` — which
exists because a bare `==` once let a new kind arrive at six sites with no
compile error — went red twice during the work and is what forced every site to
be named.

**§214.4's (b) turned out true, and nothing in the mesher changed.** `walk`'s
`the_two_surfaces_it_runs_between` already substitutes a run's second surface for
a pair that is one surface twice, so the rim's pair becomes `[Cone, Plane]` and
`where_a_plane_cuts_a_cone` answers the ellipse with no new arm anywhere: **1**
mesh point level with the corner, **0** off the cutting plane, same command as
above.

Two faults the suite caught and the lane fixed:

- **A new variant goes on the end.** Put third, it shifted postcard's variant
  indices and silently changed the meaning of every saved byte string carrying
  an edge run; the checked-in writer-fuzz seed `two_cubes_that_write` stopped
  decoding at all (`decoded: 0`).
- `EdgeRun::transformed` would have left the new variant's surface where it was.

**What is not done: the fuse does not write it.** The trace, which the brief
asked for either way: in `fuse/assemble.rs` the `Edge` that matters takes
`carried_by` straight from `FaceMeet::carried_by`, and a `FaceMeet` is the pair
of faces that *met* — so for a tool plane cutting a cone face the cutting plane
**is** in scope at `Edge::how_it_runs`. What is not in scope there is the
answer's own face pairing, which is what says whether the two faces either side
of the finished rim are the one cone. Writing the new variant unconditionally for
every closed curved run would therefore also rewrite every round bar's cap
circle, where the pair in the answer really is `[Cylinder, Plane]` and the
substitution would then read `and` in place of the face's own surface. That is a
wrong answer traded for a wrong answer, and it was not taken. The next lane's
question is narrow and is written down in §0: at `how_it_runs`, can the answer's
pairing be known, or does the arm need telling by the caller that already knows
the two output faces are one surface?

The ledger arm `check-wildcards` counted is `Run::of` in
`tests/fuses_that_must_agree.rs`, outside every lane's write set and owned at the
merge: the variant is named beside the other three rather than the ledger bumped
from 98 to 99, which is what §40 asks.

### 216.3 §215.3's second gap is a limit, and the conjunct was never the floor

§215.3 marked one step as inference and it was right about the *site*: probed,
`cut.kept` reads **`Pos`** on the floor answers. In the `Union` answer of the
side-6 corner-up cube centred on the pointed cone's apex and turned 0.7, the
`Surface::Cone` arm of `is_a_family_this_follows` was reached **39 642** times
with `kept == Pos` and 15 930 times with `Neg`, and the family gate then refused
a plane face whose cuts include a cone exactly 39 642 times — the two counts
match to the call. The same function's closing `Err` printed **zero** times
over the whole run.

**And it makes no difference.** Deleting the `cut.kept == Decision::Neg`
conjunct: **30 101 → 30 101**. Deleting it *and* letting a straight side read
`Beside(Pos)` against a cone: 30 101 again, with all 39 642 refusals relocating
to `where_a_side_lies` — not to its arc arm's `Lies::Across`, which is never
reached for a cone in this run, since the face the ellipse rings now takes
`what_a_flat_face_of_chords_and_arcs_holds` first, but to its **straight**-side
fallback instead, every one with both ends read `Pos`. There is a third blocker
behind that one.

The convexity argument does not survive either. An ellipse's inside is convex and
its outside is not, so *"any number of convex pieces meet in one"* says nothing
about the leftover — and the leftover really can be two pieces: a 2×12×5 slab the
pointed cone's radius-3 circle runs into at one long side and out at the other,
built through the public API and pinned as the counterexample.

So **§215.3's second bullet is wrong** where it says "Both are gaps under §16.10,
not limits and not defects", and its *"worth 9 326 → about 23 094"* is not a cost
that conjunct can pay. The demonstration §215.3 offered — the same plane, cut by
the same cone, followed in `Intersection` and refused in `Union` — showed that
the two differ, which is not the same as showing the outside is followable. The
conjunct stays, the limit is written at the site with all three blockers and the
measurement, and no test fails without the change because the change is a doc
comment. The lane said so in those words rather than inventing one, which is
§2 read honestly rather than satisfied.

### 216.4 The lens's wall is the cone-face arm, and the census's 30 is a namesake

The brief for this one said `select.rs` could not wind a point round a piece of a
flat face. **The premise was wrong and the measurement says so.** `select.rs`
finds four points inside the bar's flat rectangular bottom end — a plain
rectangle — and `the_piece_holds` answers `Some(true)` for all four. What fails
is the step after: classifying any of them against the lens returns
`CannotFollowAFacesTrimming` naming the lens's **cone** face, so every candidate
is dropped and the refusal that comes out names the search that succeeded.

The wall is `is_a_family_this_follows` again, and this time it is §215.3's
*other* gap: the outer match has arms for `Plane` and `Sphere` only, so a face
whose own surface is a cone is refused whatever cuts it, and
`what_a_face_cut_by_part_of_a_crossing_holds` asks that question **before** it
reads the point at all. §167's family is a plane face and a sphere face; a strip
of cone between two lids is in neither.

**The census's 30 is a different failure with the same name.** Neither the
diagnosis nor the refusal-name split that was built and backed out moved a single
row of `how_often_a_fuse_with_a_cone_refuses` or
`what_the_answered_block_fuses_actually_say`. So the block family's 30 *a point
inside a piece of a face* really are pieces no candidate was inside, and closing
the lens's wall will not move them — §215.1's 30 was not the entry this lane was
sent to, and the two only ever shared a string.

That string is itself worth splitting: `place_the_piece` raises one refusal name
for two different searches, so the fuse writes down the one that worked. Splitting
it needs three pins updated in two files no lane owned, and their doc prose
corrected with them, so it is written into a doc comment and left for whoever
owns those files.

### 216.5 Both floors are one gate, and widening it alone will move nothing

Two lanes arrived at `is_a_family_this_follows` from opposite ends and found the
same shape behind it: a gate that refuses before any arithmetic runs, with
`where_a_side_lies` sitting behind it having no case for a curved face's sides at
all. That is the warning this section exists to leave, because it is the one
thing a next lane would otherwise pay to rediscover: **the plane arm was widened
and moved 0**, and the cone arm has every reason to do the same.

What is measured for the cone arm: in the `Intersection` answer of the same case
the gate refused `face=cone, cuts=[plane, plane, plane]` **37 108** times and
refused nothing else at all — all 12 541 plane-face calls there passed — so it is
the sole cause on that answer and the cleanest thing to measure against. What is
argued for it, by the other lane and not run: a cone face cut by exactly two
planes, each crossing the cone in a closed curve, one kept cap-side and one
band-side, is connected — such a plane meets every ruling once, the two cut
planes meet in one line, that line meets the cone in at most two points, so the
rulings where the strip is non-empty are bounded by at most two zeros on the
circle of rulings and form one arc. Mixed cuts with three or more planes are not
settled and should stay refused, and the nappe guard `what_a_curved_face_holds`
already has is needed with it, or the region in front of all cuts includes the
far nappe and the answer would be **wrong** rather than refused.

Measure the gate's cost past the gate before building any of it.

### 216.6 What this cost, and what the verifiers found

`just gate quiet` on the merged code of all three lanes plus the ledger arm,
read before this section was written: **PASS, 2309 tests run, 2309 passed, 6
skipped**, 3 more in the second binary, 25 doc tests across 9 crates — six tests
more than the base's 2303. The base `dc90dd7a` was gated green, read and not
forecast, before the lanes were cut.

Two of the three lanes ended `limit_demonstrated` rather than `fixed`, and both
earned it by measurement rather than by paragraph, which is §16.10's bar met
twice in one batch. The one that ended `partial` was partial for a single red
step in a file it did not own.

**A probe number dies with its revert, and one verifier said so.** The
`a-point-inside-a-piece-of-a-face` verifier flagged that six of that lane's
figures came from an `eprintln` probe and a refusal-name split both reverted
before landing, so they cannot be reproduced from the tree as it stands; it then
corroborated the central claim by reading `holds.rs` itself and confirmed the
point is never read before the gate refuses. That is the right call and it is
worth keeping: a landed test outranks a probe number, and §215.3's rule of
carrying the reproducer in the report is what keeps a probe usable at all. Both
lanes did carry theirs.

Left open, and each is a lane: **the fuse writing the new variant** (§216.2's
narrow question); **the cone arm of `is_a_family_this_follows`** with §216.5's
warning measured first; the refusal-name split of §216.4; §215.2's other two
walls; `measure.rs`'s cone-tip bound (§208.6), §212.6's island gap, §124, §136.4,
§125.6, all untouched.
