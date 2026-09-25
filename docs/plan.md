# Plan state

`docs/roadmap.md` says what a tolerant B-Rep kernel needs. It has one commit in
this repository's whole history and it is meant to stay that way — §1 of
`docs/decisions.md` calls it the north star and not a description of this
codebase. This file is where the state goes instead.

**Read it with `just plan`**, which renders the roadmap's own headings with a
box beside each and this file's state in it. `just plan open` leaves out what is
finished, and that is the list lanes are cut from.

One line per item: the roadmap number, the state, then the evidence. There are
no titles here — they come from the roadmap at render time, so there is exactly
one copy of them. The item list is not here either: `check-plan` derives it from
the roadmap on both sides and fails when the two disagree.

The states are **done**, **partly**, **todo**, **unaudited** and **n/a**.
`check-plan` ticks a box for `done` and `n/a`, and holds a `done` to evidence
that is really in the tree. It cannot check that the evidence *proves* the item,
which is why each line names a module or a test rather than an opinion.

Audited 2026-09-15 against `main` at `6939147`. `docs/decisions.md` §106.

3        partly     crates/rcad-predicates/src/cascade.rs crates/rcad-topo/src/key.rs crates/rcad-topo/src/serialize.rs — cascade, intervals, arena, generational ids and serialization all stand; no autodiff or spatial-index crate, Newton and quadrature are hand-rolled
4.1      done       crates/rcad-geom/src/cone.rs crates/rcad-geom/src/bspline_curve.rs — analytic primitives stay exact and distinct, NURBS underneath
4.2      partly     crates/rcad-geom/src/de_boor.rs crates/rcad-geom/src/conic.rs — de Boor, knot insertion, splitting, Bezier round-trip; no degree elevation and no Oslo multiple insertion
4.3      todo       no fundamental forms, no curvature type, no Frenet frame, no continuity classifier; the `curvature` in the mesher is a deflection heuristic, not this
4.4      todo       no fitting, no interpolation, no arc-length parametrization anywhere
4.5      done       crates/rcad-geom/src/error.rs — bad input returns Err, broken invariants panic; poles and apexes threaded through every analytic surface
5.1      partly     crates/rcad-topo/src/model.rs — Vertex, Edge, Coedge, Wire, Face, Shell, Solid; no Compound, so the top level stops at one solid
5.2      done       crates/rcad-topo/src/model.rs — Orientation rides the coedge, not the face; the two-way group is exhaustive on purpose per decisions §3.7
5.3      done       crates/rcad-topo/src/model.rs crates/rcad-geom/src/walk.rs — the coedge owns orientation per face use
5.4      done       crates/rcad-topo/src/validate.rs — slop ordering, two opposite coedges per edge, wires close, dynamic range held
6.1      partly     crates/rcad-geom/src/plane.rs crates/rcad-geom/src/segment.rs — closed-form closest point only; no general Newton distance to a B-spline and no Extrema type at all
6.2      partly     crates/rcad-geom/src/three_surfaces.rs — a straight line against the five analytic surfaces, certified; no general curve, no Bezier clipping
6.3      partly     crates/rcad-geom/src/intersect.rs crates/rcad-geom/src/march.rs — every analytic pair classified, certified marching for the two degree-4 pairs, named refusals elsewhere; the freeform path does not exist and is deferred for want of a consumer, decisions §24.1; the marcher's step ceiling MOST_A_STEP_MAY_GROW_TO is still the caller's own ask times a million, a §16.10 gap that moves under unrelated work, decisions §125.6 and §125.7
6.4      done       crates/rcad-mesh/src/triangulate.rs — constrained Delaunay in uv, refined against a deflection, edge points shared between neighbours
7.1      done       crates/rcad-topo/src/fuse/mod.rs crates/rcad-topo/src/fuse/history.rs — one split, then a rule picks; history maps come out
7.2      done       crates/rcad-topo/src/fuse/mod.rs — Model::fuse takes slices of arguments and tools, not one of each
7.3      done       crates/rcad-topo/src/classify.rs — ray cast on exact predicates, ON decided by agreement
7.4      done       crates/rcad-topo/tests/degenerate_fuses.rs crates/rcad-topo/tests/degenerate_curved_fuses.rs — coplanar, tangent, sliver, contained, touching and self-intersecting, each named
8        todo       nothing exists; the hollow is the unit that starts it, decisions §104, and §104.2 says why no lane runs yet
9.1      done       crates/rcad-step/src/import.rs crates/rcad-step/src/write.rs — read and write, instancing preserved through the assembly occurrences
9.2      todo       import.rs says it in its own header: nothing is healed, no vertex merged, no gap sewn, no loop reordered
10.1     todo       fuse::Scratch holds plain Vecs on the global allocator, no bump arena; the parallel read benchmark does exist at crates/rcad/benches
11.1     partly     corpus/fetch.sh corpus/manifest.sha256 — 367 STEP files pinned from NIST and KiCad; none of the roadmap's other named assets
11.2     done       crates/rcad-topo/tests/fuses_that_must_agree.rs — volume identity, associativity, commutativity, idempotence, absorption, rigid-transform invariance
11.3     partly     fuzz/fuzz_targets — five targets, mutating models rather than generating them, with the nightly apparatus in the justfile; what they search is still being improved, one row per change in docs/queue.md, decisions §171.7 and §226
11.4     n/a        prose, not a thing that gets built
