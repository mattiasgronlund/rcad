## 150. Two pipes that cross in two runs answer one way round and refuse the other, found 2026-09-19

*§148's sweep turned up one pair it neither fixed nor broke: both pipes one
wide, the second axis `1e16` out along the shared direction and nudged two `f64`
steps along `z`. It answers `Curves(2)` asked one way round and
`CannotNameWhichPieceOfACrossingThisIs` the other. Identical before §148 and
after, so it is not that fix's. Same shape as §140 — an order-dependent answer —
and **nothing else about it has been measured.***

### 150.1 The observation

`WAY = [0.3, 0.5, 0.7]` normalised, both radii `1.0`, the first axis at the
origin, the second at that direction times `1e16` rounded componentwise onto the
grid and then nudged `+2` ulps along `z`. Asked with the first pipe written
first it is named in two straight runs; asked the other way round it refuses.

Verified on the merged tree by this session, over the full eighty-five-pair
sweep: **it is the only one of the eighty-five that answers differently
depending on the order.** That it reads the same on the pre-fix tree is the
lane's measurement, not re-run here.

### 150.2 It is a different question from §145, and the lane's reason for that was wrong

The lane put it down to the two pipes being "a hair off touching, `δ ≈ r₁+r₂ =
2`". The verdict is right and **the number this section first carried was
wrong**: it said `1.6583123951777`, and the axes are `1.7575736716416024`
apart, against a touching distance of `2`. §156.2 has the correction and how it
was caught. Either way it is not a hair — the pair crosses properly, with room,
and whatever refuses one way round is not a near-tangency being lost.

What does separate it from §145 is the plain step: at this pair the step across
does **not** cancel to zero, so §148's arithmetic is not what it turns on. Both
facts are in the sweep — the cancelling rows are the nudge-0 ones, and this is a
nudge-`+2` row.

So the honest statement is that it is order-dependent, it is not §145's family,
and the reason for it is unknown. §140's diagnosis path — find which predicate
sees a different sign with the arguments the other way round — is the obvious
place to start, and nobody has walked it.

