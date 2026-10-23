## 310. A turned pyramid's apex, four planes round it, fills the room — 2026-09-27

*Queue row Q276, from §301.7's first bullet. Lane `lane-q276-four-planes-round-an-apex`,
off `a35aa8ba` (docs only; no kernel code changed). §301.4 named a corner with
four planes round it as the family most likely to fill the fold room and left
it untried. Q240 built it: a pyramid's apex, its four side planes turned so
none of the four line up with the triple `bake` solves the corner from. This
section is that row's record, in
`crates/rcad-topo/tests/a_corner_with_four_planes_round_it.rs` (commits
`034111e9` and `4b6e3e2e`).*

  310.1   Why an apex with four planes round it can fill the room
  310.2   The fold-by-room table
  310.3   The gap at a fold of 1/64 and below
  310.4   The room x0.5 across rcad-topo
  310.5   What this leaves

### 310.1 Why an apex with four planes round it can fill the room

`bake` solves a corner with four planes round it from its best-conditioned
triple. `place_the_meets` asks each side face where its *own* triple meets:
the face and its two neighbours. At a pyramid's apex the four triples are the
four side faces' own, so one face's triple is the one `bake` used and the
other three ask about a different triple than the corner's box came from.

The test bends the pyramid's base at one corner by a fraction `fold`, so the
two side faces meeting over that long base edge fold by about `fold` too.
Both of those faces' triples are then three planes nearly through one line,
so their solve gives a wide box, while `bake` picks one of the other two
(better-conditioned) triples and keeps a narrow one. So those two faces' meets
stick out past the apex's box, by roughly the inverse of the fold — the
mechanism §301.4 named and did not build. (Citation to `034111e9`'s module
doc comment; not re-measured here.)

### 310.2 The fold-by-room table

`a_corner_with_four_planes_round_it.rs`'s module doc comment carries the
measurement, over 108 certified unions per fold (a pyramid with a cube clear
of it, at heights 1/4, 1 and 4, each side plane written through the apex or
through a base corner, turned 18 ways):

| Fold | Furthest an apex meet sticks out, in rooms | Answered | Room ×0.5 | Room ×1000 |
|---|---|---|---|---|
| 1 | 0.12 | 108 | 108 | 108 |
| 1/4 | 0.24 | 103 | 103 | 103 |
| 1/16 | 1.17 | 99 | 31 | 103 |
| 1/64 | 4.7 | 0 | 0 | 107 |
| 1/256 | 21 | 0 | 0 | 105 |
| 1/4096 | 290 | 0 | 0 | 97 |

The apex's own box stayed under 0.18 rooms wide at every fold. (Citation to
`034111e9`'s module doc comment; a number lifted from a test's module comment
is a citation, not a measurement, and it is not re-measured here.)

**So a four-plane corner does fill the room**, which none of §301's shapes
did (§301.4). At this section's base, `a35aa8ba`, the workspace's two
committed tests pinned the two ends of what the table shows: at a fold of
1/16 the union still answered although 31 of the 36 turned cases would
refuse with the room halved
(`a_pyramid_whose_two_faces_fold_by_a_sixteenth_is_united_though_its_apex_meets_stick_out_past_half_the_room`),
and at a fold of 1/64 every one of the 36 was refused as it stood
(`a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_refused_because_its_apex_meets_stick_out_past_the_room`).
`mise exec -- just t -p rcad-topo --test merged -E 'test(/^a_corner_with_four_planes_round_it::/)'`
(log kept at `target/q276/test-a_corner_with_four_planes_round_it.log`), 2
tests run, 2 passed — a re-measurement, not a citation. **Both tests have
since moved.** §356 turned the sixty-fourth's test round: it unites rather
than refuses, under its new name
`a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_united_though_its_apex_meets_stick_out_past_the_room`
(310.3). The sixteenth's test kept its name and still passes, but §356.5
forecast that it no longer pins the room at a half, and §301.10 (Q495)
confirms that by running the room ×0.5 across the whole of `rcad-topo`: the
sixteenth's test is not among the four that fail.

### 310.3 The gap at a fold of 1/64 and below

At folds 1/64, 1/256 and 1/4096 none of the 108 unions per fold answers under
`Policy::certified()`, while the room ×1000 answers 107, 105 and 97 of them.
**The answer exists, so this is a gap, not a limit** (§16.10): what stops them
answering at the room as it is, is that the apex's box and a folding face's meet box
come from two different triples of the corner's own four planes, not that no
representable answer exists. The committed test
(`a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_refused_because_its_apex_meets_stick_out_past_the_room`)
pins today's refusal at 1/64 by asserting every one of the 36 turned cases is
refused with a record naming the predicate `"order along a crossing"`; closing
the gap turns that test round (`4b6e3e2e`'s panic message says so directly:
"the gap is closed, so turn this test round"). Neither figure in this
paragraph was re-measured here; both are cited to `034111e9`.

### 310.4 The room x0.5 across rcad-topo

*This section was written from a citation to Q240's commits `034111e9` and
`4b6e3e2e` ("only two of rcad-topo's tests fail"), never re-measured, because
the row that wrote it had no `crates/` in its write set (§301.4's wording,
above, is this section's own history and is kept for that reason). §356
changed the room build the scratch edit touches into a branch, and §356.5
forecast that the count had dropped to one. Q495 is the first row whose write
set includes `crates/rcad-topo/src/fuse/arrange.rs`, and ran the edit.*

With the fold room scaled by a half across the whole of `rcad-topo` — not
just the property suites — **four tests fail, not two or one**:
`a_place_as_far_off_a_corner_as_its_room_is_folded_into_it_and_one_a_step_further_is_not`
(§301.5) and three more hand-written unit tests beside it in `arrange.rs`'s
`tests` module. **Neither pyramid fold test is among them.** The
sixteenth-fold pin
(`a_pyramid_whose_two_faces_fold_by_a_sixteenth_is_united_though_its_apex_meets_stick_out_past_half_the_room`)
passes at ×0.5, confirming §356.5's forecast, and so does the sixty-fourth's
(310.2) and the wedge's (§301.5, §301.9). The full count, the five tests that
fail at ×0.1 instead, the three that fail at ×1.25, and the command, are
§301.10's.

### 310.5 What this leaves

- **The gap at a fold of 1/64 and below** (310.3) is Q272's: close it in
  `arrange.rs` or `bake.rs`, or say why the family of fixes considered cannot,
  and turn the sixty-fourth pyramid's test round.
- **The refusals away from the apex's own fold** — at a fold of 1/4 (5 of 108)
  and, with the room ×1000, at every fold from 1/4 to 1/4096 — were not traced
  and are Q273's: say for each whether it is a limit or a gap.
- **The base corners**, each with three planes round it, stuck out up to 1.25
  rooms at a fold of 1/256 under Q240's second, uncommitted probe run. That
  contradicts §301.3's picture of a three-plane corner's box and meet
  differing only by rounding, and is Q274's to re-measure and reconcile.

No new row is added for any of these three; they are already filed.
