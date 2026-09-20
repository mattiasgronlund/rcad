## 85. A box that rounded itself away, recorded 2026-09-10

The first full night §81 bought — 00:10 to 08:10 on 2026-09-10, over `879bd9f`,
which was `main` — found one crash, and it took 31 seconds. The other three
targets ran their whole windows clean: the reader 242 615 885 runs, the writer
845 845 895, the fuse 5 554 191.

The crash was in `where_two_surfaces_cross`, and what it says is that a point
the walk put **on** a crossing came back certified `Pos` against one of the two
surfaces that named it:

    point  Point([1631.9267094795373, 326904.9266930905, 16252928.89923649])
    bound  4.895112993500317e-10
    pair   Cylinder { origin (0,0,0), dir (0,0,-1), radius 326909.0 }
           Cylinder { origin (1023,0,16252927), dir (0.0018644…, 0.9999982…, 0),
                      radius 1.984375 }

A pipe of radius 326 909 crossed by one of radius 1.98, meeting sixteen million
units out along z. **Neither of the two things that were obviously wrong was
wrong**, and
§85.1 is the arithmetic for that. What was wrong is a third thing that was in
five production files at the time.

### 85.1 The walk did not lie, and neither did the predicate

There were two candidates and it is worth writing down that both were checked
before either was believed, because the fix for one is nothing like the fix for
the other.

**Did the walk understate its deviation?** No. Over exact rationals on the `f64`
values as stored, the point is `4.311427e-10` off the thin pipe's skin, against
a published deviation of `4.895112993500317e-10`. It used 0.881 of its own bound
and stayed inside it.

**Did `side_of` certify a sign it had not earned?** No — not for the box it was
handed. This pair splits exactly, because the thin pipe's axis direction has a
zero z-component:

    radial² − r² = wz² + (wx·dy − wy·dx)² / |d|² − r²      (w = point − origin)

`wz` comes only from the box's z side and the rest only from x and y, so the
lowest value over an axis-aligned box is the sum of two independent minima and
no search is needed. Over the box as handed over, that lowest value is
**+1.147117e-09**. The skin is not in that box. `Pos` was correct.

### 85.2 The box that was asked for and the box that arrived

Every one of these sites built its box the same way:

    Interval::new(coordinate - reach, coordinate + reach)

`coordinate - reach` and `coordinate + reach` round to nearest, and **nearest
rounds inwards**: each end lands up to half an ulp short of where it was asked
for. Where `reach` is itself under half an ulp of `coordinate`, both ends land
back on `coordinate` and the side stops being a side.

That is not a corner case out at the edge of the format. It is what happens to a
**small crossing far from the origin**, which is an ordinary thing for a CAD
kernel to be asked about. Here, per side:

| side | value | one ulp | half-width asked | half-width got |
|---|---|---|---|---|
| x | 1 631.93 | 2.27e-13 | 4.8951e-10 | 4.8954e-10 |
| y | 326 904.93 | 5.82e-11 | 4.8951e-10 | 4.6566e-10 |
| z | 16 252 928.90 | 1.86e-09 | 4.8951e-10 | **0** |

One ulp at z is nearly four times the whole bound, so the z side collapsed to a
point. The y side was rounded from 8.41 ulps down to 8. Only x came out intact,
and it came out a shade *wider* than asked — nearest rounding goes both ways,
which is why nothing about this is visible in a spot check.

Give the same point a half-width of `bound` on every side, with no rounding, and
the lowest `radial² − r²` in it is **−7.123016e-10**. The skin does pass through
the box the harness meant to build, and `Undecidable` is the only answer that
box can carry. The assertion was right, the predicate was right, and the box
between them was three-quarters of the size it claimed.

### 85.3 `Interval::around`, and why the rounding is `inari`'s job

`Interval::around(centre, reach)` is `Interval::exactly(centre)` plus
`[−reach, reach]`, added through `inari`, whose arithmetic is outward-rounded by
contract. §2.4 already put the directed rounding behind that one wrapper; this
is the first thing in the tree that asks it for the **construction** of a box
rather than for arithmetic over one, and the reason the bug existed is that
building a box looked like a job for two subtractions rather than a job for
level 2.

It is not a widening. A reach that lands on the grid comes back exactly as
asked — `around(4.0, 0.25)` is `[3.75, 4.25]` — and a reach of zero comes back
as the point. Only the ends that would have rounded inwards move, and they move
by one ulp.

**It refuses a centre that is not finite rather than handing back
`Interval::ENTIRE`.** Every call site already refused one, by way of
`Interval::new` seeing `inf - reach` and `inf + reach` come back equal and
erroring; `exactly` maps a non-finite value to `ENTIRE`, so composing the two
naively would have turned five refusals into five callers carrying on while
knowing nothing. `ENTIRE` is sound as an enclosure and is not sound as an
answer — the same trap `Interval::exactly`'s own doc comment describes.

### 85.4 Five production sites, and why only a test ever fired

The instance was in a fuzz harness. The class was not:

| file | what its box is |
|---|---|
| `rcad-geom/src/walk.rs` | a walked point, asked against the half-spaces that trim its run |
| `rcad-geom/src/moved_corner.rs` | where a corner lands after a move |
| `rcad-topo/src/fuse/select.rs` | a candidate put back onto the surface it belongs to |
| `rcad-topo/src/fuse/arrange.rs` | where a closed crossing starts |
| `rcad-topo/src/validate.rs` | a corner being checked against a surface it is on |

`walk.rs` is the one that matters, and it is the same geometry as the crash:
`somewhere_within` there takes **the same walked point and the same published
deviation**, and hands the box to `every_half_space_holds`. A side that has
collapsed there does not raise anything — it certifies which side of a trimming
half-space a boundary point falls, and the run is cut accordingly.

Nothing in the tree has caught the other four, and the reason is worth keeping
because it is not "they are safe" — no search was run at them, and what follows
says why one would have to be built rather than waited for. Their reach is a
roundoff constant times the largest coordinate
involved, so it is a handful of ulps of the coordinate by construction, and
inward rounding shaves a fraction off it instead of the whole of it. A fraction
of an enclosure that is claimed to contain the true point is still not sound —
a true point within half an ulp of the box's edge is outside the box that was
built — it is just far harder to reach. The fuzzer reached the easy one in half
a minute because a walk's deviation is a share of the **crossing's** size and
says nothing about where the crossing sits (`fuzzing/mod.rs`'s `HOW_FINE`), so a
small crossing a long way out separates the two scales as far as you like.

Three test harnesses carried it too — `tests/fuzzing/mod.rs`,
`every_point_along_a_crossing_is_on_both_surfaces.rs`,
`the_degeneracies_a_crossing_has_to_answer.rs` — and so did the walked-edge
check inside `mesh.rs`'s test module, which asserts the same property as the
fuzz target and would have failed the same way given the same pair.

### 85.5 What is checked, and what is not

The crashing seed is promoted into `fuzz/corpus/where_two_surfaces_cross/` as
`crash-a2acceace…`, so `every_seed_in_the_checked_in_corpus_names_its_crossing_honestly`
replays it on every `just gate` — the corpus goes from 824 seeds to 825, and from
743 that decode to a pair to 744. That is the end-to-end regression and it fails
without this change.

The file is named here because the name is the only thing that ties it back to
this section. A seed is named for its contents, the night that found it leaves a
copy in `fuzz/artifacts/` that `.gitignore` hides, and §85 and §88 landed a seed
each against the same target a day apart — so a reader holding a `crash-…` file
has no way to tell which finding it belongs to except `git log --diff-filter=A`.
On 2026-09-11 two sessions in one evening reconstructed it that way, and one of
them first read this seed as an untriaged crash.

`Interval::around`'s own unit tests carry the arithmetic: the collapse at
16 252 928.899, the three ways there is no box to hand back, and the two cases
that must not widen.

**Nothing checks that a box is built with `around` rather than with `new`.**
Both are public and `new` is the right call wherever the two ends are known
separately, which is most of its uses, so a `check-*` arm would be a grep with
a list of exceptions. What is here instead is that every place that builds a box
round a point now calls `around` — ten call sites across nine files, five of
them production — and the ones that are a box round a walked point say in a
comment why they are not `new`.

**This is not §78's widening.** §78 gave a stored corner the room that writing
it down cost — a question about how big the box should be. This is about a box
of an agreed size arriving smaller than agreed. The two are independent, and a
tree with §78's room and this rounding still hands over a box that does not
hold its point.
---

