## 113. The refusal names the edge it gave up along, and §110.4 is withdrawn, built 2026-09-17

### 113.1 All 44 name a pair now

```text
RCAD_STEP_CORPUS=<corpus> mise exec -- cargo run -q -p rcad-step --release \
  --example near_parallel_which_pair
```

selects the same 44 files §107.1 counted. Run on `3b7fd249`, then again with
this section's change in:

| | before | after |
|---|---|---|
| `ThisEntityOfTheSolidWasRefused` — an `#id` a pair reads from | 0 | **44** |
| `TheSolidTheFileDescribesIsNotOne` — the whole-solid form | 44 | **0** |
| anything else | 0 | 0 |

A line that read `solid #15` and nothing else now reads

```text
  ThisEntityOfTheSolidWasRefused: solid #15, entity #315 (EDGE_CURVE)
  refusing pair: #302 (CYLINDRICAL_SURFACE) / #553 (SPHERICAL_SURFACE)
```

### 113.2 The motion §110.4 could not name is the mesher's own shift

§110.4 set two sentences against each other — `FaceView::surface` composes the
face's location, and `Import` stores every solid "with the file's own numbers at
the identity" — and said both could not be true of these files. **They are both
true, and they were never in tension.** `SolidBuilder::write_into` writes every
`FaceUse` and every `ShellUse` at `Isometry::identity()`, and
`Model::validate` builds its `SolidView` at the identity too, so the location
being composed *is* the identity and composing it is a no-op. A file with
several occurrences is refused by name (§42.8), so no occurrence transform is
ever read either.

**But the surfaces that reach the marcher are still not the stored ones, and
what moves them is downstream of both sentences.** `Model::validate` meshes
every curved face, and `the_frame_this_face_is_meshed_in`
(`crates/rcad-topo/src/mesh.rs`) shifts a face whose stored numbers are far from
the origin until they are small, works the geometry out there, and shifts back —
§39.7 item 2's fix for the bar a billion units up its own axis that went from
meshing at `1e-5` to refusing everything under `3.07e-4`. Its doc comment says
which faces it shifts: **"only a face at no placement at all"**, which is
"exactly the fused and the imported one". Every one of the 44 is an imported
face at no placement, so every one of them is shifted, by construction.

That is the translation §110.4 measured and could not account for.
`LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` refuses between a cylinder of
radius `1.5` about the axis through `(1.27, -1.27, 0)` and a sphere of radius
`1.5` at `(1.27, -5.07, 0)`; §110.4 found the file's own radius-`1.5` ball at
that same centre with `z = 10.0`. The file is the `Z10.0mm` variant, and the
shift is `z − 10`.

**So the correction runs the other way from the obvious one.** It is tempting
to say that because nothing in `rcad-step` transforms anything, a bit-identical
match exists for all 44 and the removed arm merely failed to find it. It does
not exist. `brep.rs` does hand the same `surface` binding to `add_face` and to
`face_surfaces`, and `write_into` stores it unchanged — so the *model's* face
and the reader's table agree bit for bit — but the marcher is not handed the
model's face. It is handed the mesher's shifted copy. Matching by value was
not one refactor away from silently finding nothing; it was already impossible
for every face the mesher shifts, which here is all of them.

**§110.4 is therefore a gap and not a limit (§16.10), and its impossibility
claim is withdrawn.** "No function of the reader's own table can turn this pair
into two `#id`s" is true of the table and false as a limit: the answer existed
the whole time, one frame up, as the key of the edge being walked. What was
missing was not arithmetic that could reach it but a field to put it in.

### 113.3 The message never carried the pair either

§110.4 also says the pair "is in the error, in coordinates, and a reader of the
message can find the two surfaces in the file by their radii and axes". That is
false of the message. `crates/rcad-geom/src/error.rs` formats
`CannotFollowACrossingWhereItsSurfacesAreTangent` as

```text
this crossing runs where its two surfaces are too nearly parallel to follow it, {at}
```

— `{at}` and nothing else. The `surfaces` field is never printed, and not one
of the 44 lines the before-run published carries a coordinate. The pair was
readable by a caller that matches the variant, and by no reader of the text.

**Of the two, this section makes the reader's half true by `#id` rather than by
coordinates**, and leaves the text of the geometry refusal alone on purpose:
§110.5 settled that a message carrying numbers is a different string per file,
which is exactly what `ProbeReport`'s grouping by reason cannot survive. So a
reader gets `entity #315 (EDGE_CURVE)` out of the refusal and the two faces out
of the file, while the surfaces stay a caller's business.

### 113.4 The edge is carried, not recovered

`rcad_geom::walk` is handed two surfaces and a run and never sees a model, so it
cannot name the edge; the caller that *is* walking an edge can. Two variants,
because there are two frames and pretending otherwise is what §110.4 got caught
by:

- `Error::ACrossingAlongThisEdgeCouldNotBeFollowed` carries an `EdgeKey`, and is
  what the sweep raises. Both doors into `walk` wrap with it — the ring walk in
  `validate.rs` and the mesher's in `mesh.rs`. **All 44 arrive through the
  mesher**, which is why wrapping only the ring walk changed none of them.
- `Error::ADraftEdgeHasACrossingThatCannotBeFollowed` carries the position the
  caller described that edge at. `SolidBuilder::build` sweeps a throwaway model
  and drops it, so a key into it is an address that stops existing; `build`
  turns the key back into the description's own index against the edge list
  `write_into` already builds.

**Only the two walls that are about following a crossing are wrapped** — the
tangency one and the stepping one. `walk` also refuses because it could not
place a point as close to the crossing as asked, because it could not name which
of two arcs this is, and because it could not decide; none of those is a
crossing that could not be followed. Wrapping every refusal the walk hands back
was tried first, and eight tests in four files went red reading those refusals
by the shape they have always had. The blast radius was the cheap half of that
signal; the name being a lie for them was the real one.

`rcad-step` then reads that index off `edge_names` exactly as the three
draft-edge refusals beside it do, and `which_entity_the_builder_meant` stops
answering `None`. Every *other* sweep refusal still answers `None` and is still
about the solid as a whole.

Neither variant's `Display` names its edge. An index into a description the
caller wrote tells that caller nothing it did not already have, and putting it
in the text would split one wall into as many `probe` reasons as there are
edges that hit it — §110.5's lesson, applied before it could be paid for twice.

### 113.5 Do not read the example's new verdict as a classification

With the pair in hand the example runs its tangency test for the first time,
and prints **"0 confirmed tangent, 44 transverse (a GAP)"**. That is not a
verdict about the 44. Every one of the pairs is **cylinder/sphere**, and
`evaluate_surfaces` handles plane/plane, plane/cylinder and cylinder/cylinder —
it prints "not one of plane/plane, plane/cylinder or cylinder/cylinder" against
every file and then calls it transverse anyway.

The one pair worked out by hand says the opposite. The LED file's cylinder and
sphere have the **same radius**, `1.5`, and the sphere's centre `(1.27, -5.07,
0)` lies on the cylinder's axis — the axis through `(1.27, -1.27, 0)` pointing
along `−y`. A sphere of a cylinder's radius centred on its axis touches it all
the way round: it is the end cap of a capsule, and it is tangent along a whole
circle, which is the shape `march`'s doc comment names as having no crossing to
follow.

So §109.1's hole is still open and this section does not close it: what changed
is that the pair is now readable at all. A cylinder/sphere case in
`evaluate_surfaces` is what closes it, and that file is not this lane's.

### 113.6 What is left

The two surfaces the refusal carries are in the **mesher's shifted frame** and
nothing on the variant says so. A caller that matches
`CannotFollowACrossingWhereItsSurfacesAreTangent` to read its pair gets numbers
that are not its model's, which is the trap §110.4 walked into with the field
rather than the text. Either the raise site moves them back or the doc comment
says which frame they are in; `march.rs` and `mesh.rs` belong to different
lanes, so this is recorded rather than done.

### 113.7 §112 landed beside this and moved the set, and where the rescued go next

§112 and this section were built at the same time and merged together, so the
44 of §113.1 was already historical when it was written down. §112 took the
cancelling subtraction out of `settled`, and measured on the merged tree:

```text
mise exec -- just t every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following
```

**24 corpus files still stop at the tangency wall**, all of them under
`kicad/`, and all 24 name an `EDGE_CURVE` with exactly two faces walking it.
The other twenty get past the marcher now. §113.1's before-and-after is what
this change did to the 44 that existed when it ran, and is not a count of
anything today.

**The rescued twenty do not import — they stop at the next wall, and it is a
third bucket.** `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`, one of the
twenty, now refuses with

```text
TheSolidTheFileDescribesIsNotOne { solid: #15, why: CannotMeshAFace {
  face: …, why: LoopsTouch { corner: 7, side: 4 } } }
```

which is neither §112's arithmetic nor §113's naming: it is the mesher's own,
it names a `FaceKey` of a model the caller never sees, and so it publishes the
whole solid for the same reason the tangency wall used to. Nobody is chasing it
yet and this is the record that it is there.

**That is also why this section's test names no file.** It named that one, and
half a day later the file it named was rescued out from under it — the
assertion still right, the fixture gone. It asserts over whichever files stop
at the wall now, with a floor of one so that the day the last is rescued it
goes red rather than green over an empty set. A floor at 24 would go red for
the good news instead, which is the other way to get this wrong.

