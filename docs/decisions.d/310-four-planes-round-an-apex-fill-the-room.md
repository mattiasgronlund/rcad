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
did (§301.4). The workspace's two committed tests pin the two ends of what
the table shows: at a fold of 1/16 the union still answers although 31 of the
36 turned cases would refuse with the room halved
(`a_pyramid_whose_two_faces_fold_by_a_sixteenth_is_united_though_its_apex_meets_stick_out_past_half_the_room`),
and at a fold of 1/64 every one of the 36 is refused as it stands
(`a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_refused_because_its_apex_meets_stick_out_past_the_room`).
Both tests pass at this section's base, `a35aa8ba`: `mise exec -- just t -p
rcad-topo --test a_corner_with_four_planes_round_it` (log kept at
`target/q276/test-a_corner_with_four_planes_round_it.log`), 2 tests run, 2
passed — a re-measurement, not a citation.

### 310.3 The gap at a fold of 1/64 and below

At folds 1/64, 1/256 and 1/4096 none of the 108 unions per fold answers under
`Policy::certified()`, while the room ×1000 answers 107, 105 and 97 of them.
**The answer exists, so this is a gap, not a limit** (§16.10): what stops the
110 answering the room x1 is that the apex's box and a folding face's meet box
come from two different triples of the corner's own four planes, not that no
representable answer exists. The committed test
(`a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_refused_because_its_apex_meets_stick_out_past_the_room`)
pins today's refusal at 1/64 by asserting every one of the 36 turned cases is
refused with a record naming the predicate `"order along a crossing"`; closing
the gap turns that test round (`4b6e3e2e`'s panic message says so directly:
"the gap is closed, so turn this test round"). Neither figure in this
paragraph was re-measured here; both are cited to `034111e9`.

### 310.4 The room x0.5 across rcad-topo

With the fold room scaled by a half, only two of rcad-topo's tests fail:
the sixteenth-fold pyramid pin
(`a_pyramid_whose_two_faces_fold_by_a_sixteenth_is_united_though_its_apex_meets_stick_out_past_half_the_room`,
310.2's table already names its 31-of-36 refusal count) and the hand-written
room-boundary pin
(`a_place_as_far_off_a_corner_as_its_room_is_folded_into_it_and_one_a_step_further_is_not`,
§301.5). 1054 of 1056 pass, over one run of the property suites. **This
figure is cited to Q240's commits `034111e9` and `4b6e3e2e`, and is not
re-measured here**: reaching it needs the same scratch edit to
`a_corner_is_this_place` that §301.6's probe used (an `Q225_SCALE`
environment variable read inside `arrange.rs`), and this row's write set does
not include `crates/`, so no such edit was made.

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
