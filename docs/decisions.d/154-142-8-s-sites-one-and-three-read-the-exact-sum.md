## 154. §142.8's sites one and three read the exact sum the caller already made, fixed 2026-09-19

*The last two of §142's four sibling sites, both closed on `c899b491`. Each
recomputed in intervals a quantity the caller had summed exactly one question
earlier; each now reads that sum back. Gate green on the merged tree: 2069 run,
2069 passed, 1 skipped, 25 doc tests.*

### 154.1 The same bug, and not the same fix

**Site three** was what §142.8 said it was. `how_far_apart_two_loops_are` asks
`how_far_off_an_axis_enclosing` for `A`, when the caller has just asked
`how_far_off_an_axis_exactly` for the *sign* of that same `A` to decide the
centre is on the axis. It now calls the exact routine and reads the result back
through `Exact::enclosure`, falling back to the enclosure where the exact arm
declines. For a centre exactly on the axis `1e10` out, `A` is `[-0.0, 0.0]`
instead of an enclosure straddling zero, `D₋` becomes `W(R² − r²)`, and the
offset rounding of `1.9e-6` loses to a `reach` of `3.2e-5`.

**Site one was a different shape, and the brief was wrong about it.**
`how_far_off_an_axis_exactly` has nothing to do with site one: its quantity is
the plane-against-pipe discriminant `n·n·r² − (n·o − d)²`, not a distance off an
axis. That arithmetic did already exist verbatim — but as an anonymous closure
inside `where_a_plane_and_a_pipe_cross`'s `Question`, not as anything callable.
It is now `how_much_room_a_plane_leaves_in_a_pipe_enclosing` / `_exactly`, and
both the `Question` and `two_lines_along_a_pipe` point at it, so there is one
copy of the discriminant.

`two_lines_along_a_pipe` then takes `out = √(room/n·n)` rather than
`√(r² − (reading/‖n‖)²)`. Those are the same number; the second is written so
that it *has* to cancel, and for an axis `1e10` from the origin the reading is
`±9.54e-7` — a hundred times the pipe's own width — while the true separation is
ordinary. **No new exact arithmetic was written at either site.** The half of
the hypothesis that survived is that one; the half about which routine site one
wanted did not.

### 154.2 Three published rates moved, and nothing was asserting any of them

| family | want of a half-space | two curves |
|---|---|---|
| a hair from tangency | 65 (3.2%) → **55 (2.68%)** | 304 → 314 |
| turned, the headline | 1 (0.05%) → **0 (0.00%)** | 12 → 13 |
| turned quartic | 42 (2.05%) → **41 (2.00%)** | 225 → 226 |

Re-measured on the merged tree by this session, not taken from the lane's
report. Twelve pairs that refused are named now and nothing moved the other way.

**The figures live in prose and the constants beside them are upper bounds, so
nothing fails when a rate moves.** `A_QUARTIC_CROSSING_MAY_LACK_A_HALF_SPACE`'s
doc comment read `2.10%` while the tree measured `2.05%`, and §142.9 repeated
the `2.10%`: it had drifted once already and no one had cause to notice. The
doc comments now name the command that re-measures and say the figure is not
the bound.

This is §15.6's rule meeting its blind spot. A published *bound* has a test —
`share_that_lacked_a_half_space_is_within` — and the bound did not move. A
published *measurement* has nothing, and a measurement is what a reader takes
away.

### 154.3 Why the bound stays at half a percent

The turned family measures `0.00%` now, and the constant stays `0.005`.
`soak` seeds a deterministic `ChaCha`, so the rates reproduce to the pair — but
a draw of 2052 is a sample of a family and not the family, and zero of *these*
2052 is what one of these 2052 became. A rate this low is a pair or no pair.
A bound of zero would assert a structural fact the draw does not carry, and
would then break on the next change to a generator rather than on a regression.
`A_TURNED_CROSSING_MAY_REFUSE` may be zero because §118.11 removed the
arithmetic; site one's fix re-routed the arithmetic and did not remove it.

### 154.4 The write set had to break, and that is §148.5 again

`three_more_places_a_crossings_two_pieces_cannot_be_told_apart.rs` **is**
§142.8's measurement: it pins the two refusals as characterisation tests, so
`rcad-geom` could not be green while it stood. The lane broke its write set in
one isolated commit, kept every measured enclosure and every assertion that it
straddles zero, and flipped the two verdicts to "named" — the stronger claim,
because the interval route still cannot decide either pair and neither goes
through it any more.

That was the right call and the brief should have foreseen it: §148.5 already
says a lane's green gate and the merged tree's are different questions, and the
difference is whatever the write set kept it away from. **A lane that fixes a
site owns that site's characterisation test.** Two of the file's three sites now
read as closed and its name says "three more places"; renaming it would collide
with the site-two lane, so it waits for that one.

