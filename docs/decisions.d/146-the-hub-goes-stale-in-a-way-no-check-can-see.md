## 146. The hub goes stale in a way no check can see, fixed 2026-09-19

*Mattias ran `just decision 144.5` in the hub and was told the section does not
exist. It did: the hub's working tree was five days old. §60 parks the hub on a
detached HEAD for reasons that still hold, and §60.9 already said "Nothing moves
the hub's HEAD on its own" — what neither of them noticed is that a hub nobody
touches drifts with **no symptom at all**, and that `check-hub` passes it.*

### 146.1 The symptom, and why it reads as an answer

`just decision 144.5`, typed in `~/git/mattiasgronlund/rcad`, printed

    docs/decisions.md has no section 144.5

which is a true statement about the file in front of the reader and a false one
about this repository. The hub's `docs/decisions.md` was written 2026-09-14,
ended at §99, and was 27 000 lines against `main`'s 35 000. Nothing in the
message says which tree it read, so the answer is indistinguishable from the
section not existing.

This is §77.6's shape for the third time in one session: a command reporting a
property of the environment it ran in rather than of the thing it was asked
about. The other two were `check-plugins` passing on a runner that has no Claude
Code, and `check-wildcards` counting colour codes.

### 146.2 `check-hub` cannot see it, and the reason is structural

`check-hub` fails on exactly two conditions: a branch checked out in the hub,
and `git status --porcelain` coming back non-empty. A hub parked at an older
commit whose files match **that commit** has an empty status, so it reports
`the hub is parked and clean` and exits 0 — and `just gate` was green on it.

§60.9's staleness detection is real but is nested inside the dirty branch: it
tells stale from leftovers only once there is something in `status` to explain.
It was written for a hub that had files landing under it, which is a hub
somebody had worked in. The case where the drift leaves no trace was never in
scope, and it is the ordinary one, because the rule is that nobody works there.

A check that only fires when the fault is visible by other means is not the
check it looks like.

### 146.3 What was built

A hub that is behind is re-parked by the next `just` command typed in it.

`just` evaluates every top-level assignment before it runs a recipe, whether or
not the variable is used — measured on 1.42.4 with a throwaway justfile, both
for a recipe that uses the variable and one that does not, because the language
reference does not promise it. `just --list` the *flag* does not evaluate them,
but a bare `just` runs the `list` *recipe*, so it fires there too. That is the
only hook the file has, and `hub_aligned` is it. The value is unused and always
empty; the work is the side effect, and what it has to say goes to stderr, which
`just` passes through.

`tooling/hub-align/align.sh` does the work, and three quarters of it is the
refusals:

- **Not the hub, or a lone checkout** — exits within three git calls. Every lane
  pays that on every recipe, which is why it is three and not a `status`.
- **A branch checked out in the hub** — untouched. That is `check-hub`'s failure
  and it names the fix; detaching silently moves somebody off a branch.
- **Anything not already on `main`, byte for byte** — untouched, with a line
  saying so and pointing at `check-hub`. The re-park runs `clean -fd`, so a
  wrong "yes" here deletes the only copy of somebody's work (§60.10).

It always exits 0. It runs ahead of every recipe in the file, so a fault in it
must never be the reason `just test` did not run.

### 146.4 `main`, not `origin/main`, and the fetch that is not there

Mattias asked for alignment with `origin/main`. It aligns to local `main`, and
the difference is a network round trip on every single `just` invocation —
`just t` included. That is §144's finding paid in latency instead of tokens, and
it breaks the repo offline.

It costs nothing here: `main` is the branch this repo lands on locally, in
`wave-1`, and then pushes, so the local branch is never *behind* `origin/main`.
If it ever is, that is a push somebody owes, and a re-park quietly papering over
it would hide the thing worth knowing. One `git fetch` in the script is all it
would take to change this, and the argument above is what it would cost.

### 146.5 What the test had to be able to do

`check-hub-align-selftest` builds a repository and walks it through seven
states. The ones that matter are the four where the script must do **nothing**,
because a re-park deletes untracked files.

The fixture therefore writes *differing* bytes under a name `main` also carries
— the §60.10 case — and asserts they survive. Every other case in the file
passes a name-only comparison, so without that one the test proves nothing about
the dangerous path.

That claim was checked rather than asserted: with `unsafe="$path"` changed to
`unsafe=""`, the selftest fails with *"a hub holding uncommitted work was
re-parked"* and the output shows the fixture's work deleted. Reverted, it
passes. A guard nobody has watched fail is not known to be load-bearing.

### 146.6 What this does not close

The hub's `justfile` is stale too, so the first re-park happens under recipes
that predate the feature — the script says so when the justfile moves under a
run, and the answer is to run the command again. The live hub cannot bootstrap
itself for the same reason: its 2026-09-14 justfile has no `hub_aligned` in it,
so Mattias runs §60.9's published line by hand once, and it is self-maintaining
after that.

`check-hub` is still blind to a silently stale hub, and is left that way on
purpose: with the re-park running ahead of every recipe the state is now hard to
reach, and a check for a condition that fixes itself would fire on nothing. If
the hook is ever removed, this is the hole that comes back.

