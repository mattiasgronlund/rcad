## 244. Whether a closed crossing folds two nodes that are apart — 2026-09-25

*Queue row Q57, from §232.3. `round_a_closed_crossing` answers `Zero` when
the sweep from the walked start is proved zero and the two nodes' boxes
overlap, and `merge_points_that_are_one_point` folds on that `Zero`. Lane
`lane-q57-closed-crossing-folds`, off `60668c15`, read-only on `crates/`.
**In progress**: this commit is the half that is argued from the code; the
witness search follows.*

  244.1   Which proved zeros reach the arm

### 244.1 Which proved zeros reach the arm

`which_comes_first_round_a_crossing` has no level 1, so a `Zero` comes from
one of two places in `decide`:

- **The exact level**, which runs only when `from`, `one` and `other` are all
  `PredicatePoint::Exact`. Two exact points overlap only when every coordinate
  is equal, and then `one == other` has already answered at the top of
  `round_a_closed_crossing`. So an exact-level zero never reaches the
  overlap arm with two different nodes.
- **The interval level**, when `n · ((one − from) × (other − from))` comes out
  as exactly `[0, 0]`. `inari` rounds outward, so that needs every product in
  the sum to be a point interval, which in practice means coordinates of no
  width that cancel: `one`, `other` and `from` sharing a coordinate exactly,
  as the half ball of `round_a_closed_crossing`'s own doc comment does
  (`x = 0, z = 0.5`).

`from` is always the walked triple as an exact point — `how_a_crossing_runs`
builds `PredicatePoint::from(from.coordinates())` whatever the surfaces
certify — so a wrong fold needs two nodes on the crossing, a shared exact
coordinate with `from`, and boxes that touch.
