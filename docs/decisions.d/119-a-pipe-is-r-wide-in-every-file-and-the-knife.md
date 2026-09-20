## 119. A pipe is `r` wide in every file, and the knife edge was a rounding, finished 2026-09-17

*§118.11 changed what a pipe means and wrote down what was left to do. Its own
account of what was left is wrong, and that is the first thing this section is
for. Six lanes finished the convention across five files; what fell out when they
did is the rest of it, and it is more than a tidy-up — a family that used to
refuse decides now, a knife edge three sections argued about turns out to have
been a rounding, and one pair of pipes was answering confidently and wrongly.*

Gated and pushed: **`70f72302` on `main`**, `just gate` PASS, **1968 tests run,
1968 passed, 0 skipped**, plus 3 release-only tests and 24 doc tests. The gate
was run in `wave-1` on the merged tree after the fast-forward, and that tree is
byte-identical to the one the lanes were merged into (`9ff76e58`). No lane ever
saw all six changes at once, so **1968 is the only number to quote** — the
largest any single lane reported was 1962.

### 119.1 A convention change needs a sweep for the convention, not for the symbol

§118.11's closing paragraph named one thing as outstanding: the leaning-pipes row
of `intersect.rs`. What was actually still written in the old convention, found by
building the list from the files rather than from that paragraph:

- plane and pipe (`room` certified a pipe `r/√W` wide), ball and pipe, two pipes
  side by side, cone and pipe, the torus picker — that is, **`intersect.rs`
  throughout**, not one row;
- `march.rs:662` and `:763`, the marcher's *certified* distance;
- `three_surfaces.rs:699` and `:1494`, the discriminant — which was *deliberately*
  matched to the old predicate and cited §27.1 for doing so.

**The sweep that missed them searched for `ρ`.** That found every paragraph of
prose and none of the arithmetic, because the arithmetic spells the same thing
`‖w‖`, `length()`, or `W`. Two sites (`two_lines_along_a_pipe`,
`where_a_ball_rests_against_a_pipe`) alias the direction into a local first and
no grep for either spelling reaches them — the same hole §118.11 already recorded
about its own write-set table, one file along.

So the rule, which is §16.10's shape applied to a sweep: **a claim that a
convention has been finished is a claim about what cannot be found, and it is
owed the same demonstration as any other.** Name the families of spelling the
convention can wear, and search for each; a single symbol is one family.

**A doc comment that argues for the old convention is worse than one that merely
states it.** `three_surfaces.rs` cited §27.1 for deliberately writing a different
pipe from `intersect.rs`'s. That was correct and well-reasoned until the
predicate moved underneath it, and then it was an argument for a defect.

### 119.2 The six lanes

All six merged with no conflicts.

| Branch | Tip | What |
|---|---|---|
| `lane-pipes-are-their-own-width` | `2cf7d1de` | `intersect.rs`, converted atomically |
| `lane-leaning-crossing-red-test` | `254ca86b` | the red test, no `src/` access |
| `lane-stale-certified-radius` | `e2dc5820` | the claims outside `intersect.rs` |
| `lane-three-surfaces-pipe-convention` | `0d6e430d` | the discriminant |
| `lane-marcher-measures-the-right-pipe` | `4845af85` | the marcher |
| `lane-settle-the-turned-pair` | `c98af79f` | the knife edge and the refusal rate |

`intersect.rs` had to move in one commit and not five: §118.11 says why —
`certified_radii_enclosing`'s `enclosing` arm pairs with
`how_two_radii_and_a_gap_compare_exactly`'s `exactly` arm inside one `Question`,
and converting one without the other lets level 3 certify a sign the enclosure
does not bracket.

### 119.3 A ball resting inside a pipe was never on a knife edge

`is_on_a_knife_edge` had three arms and now has two. The ball-and-pipe arm — a
ball resting all the way round the inside of a pipe exactly as wide as it is — was
there because the old question carried the stored direction's squared length on
the ball's side, and `‖w‖²R² − r²` is that direction's **rounding wearing a
discriminant's clothes**.

The demonstration, and it is a demonstration and not a paragraph: the pre-§118.11
rule was rebuilt in `rcad_predicates::expansion` and asked of all 63 turns of one
resting ball. **Two circles for 48 of them, nothing for 15, one circle for none** —
the same geometry, three answers, and the truthful one never given. The new
answer holds for a reason a turn cannot reach: a rotation does not change a scalar
radius, and the ball's middle and the axis' origin are the same point, so
`R² − r²` is exactly zero after every turn as well as before.

One trap that lane recorded: its first attempt used `Vector::length_squared`,
which rounds three times, and got wrong signs out of it. **The exactness is
load-bearing in the demonstration, not only in the kernel.**

### 119.4 The turned quartic family stops refusing, and a property retires with it

`A_TURNED_QUARTIC_CROSSING_MAY_REFUSE` goes **`0.002` → `0.0`**, earned by soak
rather than by argument: 409 604 turned quartic pairs on each of two seeds, not
one sign refusal between them. The constant's own doc carries the arithmetic —
with `ρᵢ = rᵢ` the `√Wᵢ` leave the comparison altogether, so the expansion whose
spread was past the window a slide has to land in is not built at all.

`every_turned_quartic_crossing_that_refuses_was_exactly_tangent_before_the_turn`
is **retired rather than re-aimed**, and the reason is the honest one: what it
measured is gone, so its own vacuity guard (`refused > 0`) is what failed. The
only family in that file still refusing for want of a sign already has a property
asking exactly this question of exactly those 217 refusals, and pointing a second
one at it would be one property written twice (§13 rule 5).
`DRAWS_A_KNIFE_EDGE_NOW_TAKES` went with it.

**Its name still stands in two places in this file**, at §29.5 and §32.1, and both
now carry a note saying so. They are left as the records they were rather than
rewritten: the sentences are true about the day they describe. A name in a
historical section is a citation, not a promise that `cargo test` can reach it.

`TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED` is **51**, down from 65 — verified on the
merged tree, where it is asserted and green. That is the same figure the lane
measured alone, so nothing else in the unit moves that watch.

The wildcard ledger is **87, not the 86 the lane was briefed with**. Both arms the
merge removed were *composite*, not direct — the direct half never moved from 83 —
and F's own removal of the knife-edge arm puts one composite point back.
`wildcard_arms := "87"` in the justfile, and `check-wildcards` is green.

### 119.5 Two pipes overlapping by `1.1e-16` were answered `Ok(Nothing)`

The severity finding of the unit, reported by the lane that wrote the red test.
On the base, a pair of pipes that genuinely overlap by `1.1e-16` came back
**`Ok(Nothing)`: confidently wrong, no refusal, and no provenance to find it
by.** §4.1 ranks that below crashing, and it is the whole argument for the
convention change stated as one case — the old predicate was not timid, it was
certifying a sign it had not earned, because `2a² − 1` for a 45° axis is not
zero.

### 119.6 §29's geometric claim is spent, and the `Exact` test is not

§118.11 anticipated this and §16.10 requires it: when the ground under a "cannot
be done" moves, the claim is owed a re-examination rather than an inheritance.

§29.2 records that the exact level runs out for two leaning pipes and nowhere
else in the file, because their three comparisons are lengths in three different
metrics — `rᵢ/√Wᵢ` against `|T|/√K` — and clearing the roots costs eight degrees
and builds an expansion five hundred binary orders wide. **That is no longer what
the kernel asks.** `how_two_radii_and_a_gap_compare_exactly` is now degree six
against degree six with no `Wᵢ` in it, and the restriction to pairs whose stored
directions have the same squared length is gone with it.

What does **not** change is `Exact`. The expansion is still five hundred orders
wide and still does not square where it stands; `the_question_two_leaning_pipes
_ask_is_too_wide_to_square_where_it_stands` passes untouched, because it was
always a self-contained test of `Exact` that happened to be built out of pipes.
Its doc comment said otherwise in the present tense and now says this. The name
is kept deliberately: it says where the numbers came from, and renaming it would
make §29.6's and §30.5's records name a test that does not exist — the very
failure §119.4 had to leave a note about.

One row of §30.5's refusal table — *"two leaning pipes' comparison is too wide to
square where it stands"*, booked there as a gap — is a refusal this kernel no
longer produces, because it no longer builds the comparison. The row stays as the
measurement of M3 it was; what closed it is here rather than there.

### 119.7 Which numbers here were re-derived, and which were taken on a lane's word

Git cannot tell one author from another here, so the split is written down.

**Verified on the merged tree by the session that wrote this section:** the gate
(PASS, 1968, exit code captured separately from the log); `main` fast-forwarded
with `HEAD^{tree}` equal to the gated tree; `TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED
= 51`; `A_TURNED_QUARTIC_CROSSING_MAY_REFUSE = 0.0`; `wildcard_arms = 87` with
`check-wildcards` green; `is_on_a_knife_edge` carrying two arms and a doc comment
explaining the third; §119 free on every branch (`git log --all -S`).

**Lane reports, not re-derived — re-measure before quoting:** turned-quartic
refusals 14 → 0 on a branch carrying only `intersect.rs`; the `1.1e-16` pair
above; `moved_corner.rs`'s pipe arm at three units with the published
`90·f64::EPSILON·L` unchanged because a plane still governs at four
(`CORNER_AFTER_MOVING_ROUNDOFF` deliberately untouched); 160 of 3040 readings
moved in the corner instrument with the worst case unmoved at `0.0407`, the worst
reading not being on a pipe; the discriminant's published degree fourteen
unchanged; and the 48/15/0 turn census of §119.3.

**Two fourteens in this unit are not the same fourteen.** The turned-quartic
refusals fell 14 → 0 on a branch carrying only the `intersect.rs` change; the
discriminant's degree fourteen is a degree and did not move at all. Two
independent facts that share a numeral, recorded here because a later reader will
otherwise join them.

One thing a lane looked for and **did not** find, said plainly rather than
rounded up: it could not build a case where the *old* certified enclosure
excludes the truth. The defect above is a wrong certified sign, which is enough.

### 119.8 What this unit leaves

- §113.7's `LoopsTouch`, where the files §115 rescued stop next.
- §111.3's missing exact-expansion tier.
- §115.4's one file, and §116's unattempted half.
- **§120**, opened by this unit's measurement and scheduled after it.

