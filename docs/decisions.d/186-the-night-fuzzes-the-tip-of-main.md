## 186. The night fuzzes the tip of `main`, not the last green — 2026-09-21

§90 moved the overnight fuzz onto **the newest commit on `main` that had gone
green**, and §173.9 let a recorded local `just gate` count as green once CI could
no longer answer. Mattias has now asked for the night to fuzz **the newest commit
on `main`**, full stop. `fuzz-watermark` takes the tip of `origin/main`, fetched
fresh, and keeps the two conditions that were never about greenness: a clean tree
and forward only.

### 186.1 What waiting for green cost, on the night that asked

The night of 2026-09-21 started at 00:10 and fuzzed `ae124a29`. By then `main` had
moved on to `c53043a8`, and `origin/main` had been there since 23:13
(`git reflog refs/remotes/origin/main`). Between the two sat `ed450e9a`, §182's
fix for §181's pipe-on-a-fat-pipe family.

The watermark passed it over because the green record said nothing newer than
`ae124a29`, recorded 19:47Z. `c53043a8`'s gate was not written down until 12:17Z
the next day. So the night spent eight hours on a tree without the fix and found
`where_two_surfaces_cross/crash-ff66695`: the same family, a thin pipe offset by
5e153 from a fat one. `fuzz-triage` replays it on `59cabf30` and it returns two
curves. The one crash the night found was a crash already fixed.

Under this rule the night would have stood on `c53043a8`.

### 186.2 Why §90's reason for green does not hold

§90.2 argued that green is what §81.4 wanted: nothing fuzzes a tree nobody has
read end to end. The objection behind §81.4 was a revert. A crash found on a
commit that later gets reverted would be noise.

That noise already has a filter, and it is not the watermark. **`fuzz-triage`
replays every finding against `main` the next morning** and prints `still fails
on main: yes | no`. A finding on a broken tip that was since reverted reads `no`,
which is the same thing the morning of 2026-09-21 read for a finding on a stale
green. Waiting for green does not remove that line of work. It only makes sure
the night always runs somewhat behind.

And green has turned out to be the unreliable input. §173.9 was needed because
CI stopped answering. This section is needed because a local gate answers late,
or not at all for a commit nobody gated before bed. Each supplier of "green" has
failed by lagging, and a lagging watermark fails silently, since staying put is
how `fuzz-watermark` is built to fail.

What the tip can cost that green could not is **a tip that does not build.**
Then `fuzz-overnight` writes `the fuzz targets did not build, so nothing was
fuzzed` and fails, and the morning's summary says so. That is a loud, rare
failure in place of a silent, common one.

### 186.3 What changed

- `fuzz-watermark` fetches `origin`, and prints `origin/main` when `HEAD` is a
  strict ancestor of it. It stays put and says why on a dirty tree, an origin it
  cannot reach, a tree already on the tip, or a tip that is not ahead. It no
  longer reads `gh` or `rcad-greens`.
- `_record-green` and `rcad-greens` stay, because `just status`'s green row still
  reads them (§173.10).
- `check-fuzz-overnight-selftest` no longer relies on a `gh` stub that answers
  nothing to keep the gate's own tree from moving. That guard was hermetic only
  as long as the watermark needed an answer from `gh` before it moved. The tip
  needs no such answer, so a lane's gate would have been checked out onto
  `origin/main`. The arms that run in the gate's tree now set
  `RCAD_FUZZ_ALREADY_ADVANCED=1`, and the HEAD assertion after arm three still
  holds them to it. The arm that advances runs in its own fixture repository,
  with a bare origin.

### 186.4 What is checked

`check-fuzz-watermark-selftest`, against a fixture repository with a local bare
origin:

1. With a recorded green one behind the tip, it picks the tip. This is the arm
   that fails without this section. Run against `origin/main`'s justfile at
   `29ab552e`, the old recipe picks the recorded green, and the new one picks the
   tip.
2. After another clone pushes, it picks the new tip, so it fetches rather than
   trusting the last fetch.
3. On the tip, it stays and says `already on the tip of main`.
4. Ahead of the tip, or on a commit off `main`, it stays and says the tip is not
   ahead of it.
5. With origin unreachable, it stays, says so, and exits 0.
6. With uncommitted work, it stays and says so.

The seventh arm is `_record-green`'s and is unchanged.

### 186.5 When it takes effect

§180.1's fixed point applies. The timer's tree reads `fuzz-watermark` from the
commit it already stands on, so the first night after this lands still runs the
green rule. That night advances as long as some green commit is ahead of it.
The merge of this section is gated, so it is, and the night re-executes the
checked-out recipe. The watermark does not run again that night, since a night
advances once. Every night after follows the tip.
