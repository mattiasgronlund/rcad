## 110. The tangency refusal names its pair and its raise site, built 2026-09-17

§109.5 asked for three things. Two are built and the third is measured and
refused: the `#id` half cannot be reached from `rcad-step`, and this section is
the demonstration of that rather than a paragraph about it (§16.10).

### 110.1 One variant became two, and the argument for splitting

`Error::CannotFollowACrossingWhereItsSurfacesAreTangent` was a unit variant with
six raise sites. It now carries the two `Surface`s the marcher gave up between
and a `WhereFollowingACrossingGaveUp` saying which question gave up, and the
shortening budget has moved out to a variant of its own,
`Error::CannotFollowACrossingBecauseNoStepStaysOnIt`.

**Site tag or split variant is a judgment about meaning, not about tidiness.**
Four of the sites ask the same question in different words — can the two normals
still tell us anything — and differ only in which arithmetic noticed first.
Those keep one variant with a tag, because a caller that wants "is this pair too
nearly parallel" gets one name to match on and the tag when it wants more. The
fifth, `one_step` running out of shortenings, asks nothing about the normals at
all: it asks whether any piece short enough to be part of the curve could be
found. A caller matching "tangent" on that is being told something false, which
is exactly what §109.6 found the corpus census doing. So it is a variant.

The fifth site was the honest test of the split: two balls half a radius apart,
stepped **against** the way their crossing circle runs, refuse with the stepping
name and no tangency anywhere in the picture
(`a_step_asked_the_wrong_way_round_a_circle_is_refused_as_stepping_and_not_tangency`).
**`march` never asks that**, because it takes its direction from
`which_way_the_crossing_runs` at the point it is standing on, so that test
demonstrates the path is live and nothing about how a real walk reaches it. Two
ways of reaching it through the sagitta tests were tried first and neither
works: on unit balls and on balls a million across, the midpoint settles onto
both surfaces exactly, the sagitta is `0.0`, and `0.0 <= room * 0.75` holds
however small the room is made. **Whether a real walk can exhaust the budget is
therefore open**, and nothing here claims it either way.

The site enum is five questions and not five line numbers. A line number goes
stale on the next edit, and what a reader of a refusal wants is what was being
asked.

### 110.2 The site tag falsified §109.6 on its first run

§109.6 took `march` apart one call at a time on a ball one rounding of radius
bigger than its pipe and concluded the wall is **`one_step` exhausting its fifty
shortenings**. With the site tag in, that same test reports
`SettlingAPointOntoBothSurfaces`.

`one_step` does refuse, and it refuses on its **first trial**, from the `?` on
`settled(surfaces, here + tangent * step)`. It never reaches a second
shortening, let alone fifty. The seed itself settles without complaint — the
test still asserts that — and what collapses is the settling of the stepped-out
point, where `1 − c·c` is exactly zero.

So §109.6's own first gap-shaped hypothesis, recorded there as untested — that
`settled`'s Newton correction `(s₁ − s₂c)/(1 − c²)` reaches zero at a lean of
`2.5e-8` while `|u×v|²` is still good at `4.9e-11` — **is what is happening**.
§109.6's headline mechanism is withdrawn for the second time in two lanes, and
the conclusion it shares with its predecessor still stands: the wall is eight
powers of ten above the cross-of-normals one and nowhere near it.

`the_stepping_gives_up_on_a_ball_a_rounding_bigger_than_its_pipe_and_not_the_normals`
is renamed `the_settling_and_not_the_stepping_gives_up_on_a_ball_a_rounding_bigger_than_its_pipe`,
because its old name was the claim.

**A lane briefed at `one_step` should be briefed at `settled` instead.** That is
the only place the reachable wall has been seen.

### 110.3 All 44 corpus files are one site, and it is the settling

Run on 2026-09-17 with the geometry change in and `rcad-step` untouched:

```text
RCAD_STEP_CORPUS=<corpus> mise exec -- cargo run -q -p rcad-step --release \
  --example near_parallel_which_pair
```

selects the same 44 files §107.1 counted, and every one of them prints

```text
  TheSolidTheFileDescribesIsNotOne: solid #15, why: geometry: this crossing runs
  where its two surfaces are too nearly parallel to follow it, asked while
  settling a point onto both of them
```

**44 of 44 at one site**, and it is neither of the two §109.5 expected to stop
being counted as tangency. The corpus and the unit test agree, which they could
not do before the tag existed.

### 110.4 The `#id` half does not reach, and here is the measurement

The third bullet of §109.5 — an arm that stops `brep.rs:727` falling back to the
whole solid — was built and then taken out again, because it names nothing.

The plan was the obvious one: `describe_the_face` already keeps every face's
`Surface` beside the `#id` it was read from (`face_names` / `face_surfaces`), so
match the refusal's pair against that table. Built, and instrumented, it scores
**0 matches** for both surfaces of every one of the 44 — not one match and not
several, none. The table was widened from one `Walk` to the whole file first, on
the theory that `Model::validate` sweeps solids built earlier; that changed
nothing.

What the instrumentation printed is a translation:

- one file refuses between a pipe at `(0, 0, 1)` and a ball at `(0, 0, 11.3)`,
  both radius `0.2`, while its own faces sit on pipes and balls of radius `0.2`
  at `(23.3, ±4.5, ·)` and `(-0.7, ±4.5, ·)` — the same shapes, moved in `x`
  and `y`;
- another refuses about a ball at `(1.27, -5.07, 0.0)` of radius `1.5` whose
  only radius-`1.5` face ball is the same centre at `z = 10.0`.

So the surfaces the sweep is holding and the surfaces the reader wrote down are
a rigid motion apart. `FaceView::surface` composes the face's location, and
`Import`'s own doc comment says every solid is stored "with the file's own
numbers at the identity" — those two sentences cannot both be true of these
files. **Which one is wrong is not settled here**, and guessing it was what this
lane ran out of budget for. What is settled is that no function of the reader's
own table can turn this pair into two `#id`s.

**The fix is `rcad-topo`'s**, which no lane owned here either: either
`the_two_surfaces_along_this_edge` hands the marcher the face keys alongside the
surfaces, or the refusal carries the keys and `rcad-step` maps keys to `#id`s it
already has. Both are small; neither is in this lane's files.

Until then, `Refusal::TheSolidTheFileDescribesIsNotOne { solid: #15 }` is still
what the 44 publish. **The pair is no longer unreadable, though** — it is in the
error, in coordinates, and a reader of the message can find the two surfaces in
the file by their radii and axes. What is missing is only the `#id`.

**This section's two claims are withdrawn — §113.** The impossibility claim is
wrong, and the motion it could not name is the mesher's own shift; the sentence
about a reader of the message is wrong about the message, which prints no
coordinate at all.

### 110.5 What a message that grew a tail costs

The refusal's `Display` now ends with which question gave up, so `probe`'s
deduplicated reason strings changed. `near_parallel_census.rs` and
`near_parallel_which_pair.rs` both selected their 44 files by comparing that
string **whole**, and would have silently selected nothing. Both now match on a
prefix, and both carry the second wall's prefix beside it so a file that stops
on the stepping budget lands in the same bucket.

The message deliberately carries the *site* and not the surfaces' numbers. A
message carrying coordinates is a different string per file, and `ProbeReport`'s
whole job is to group files by reason — one file per reason is not a grouping.

