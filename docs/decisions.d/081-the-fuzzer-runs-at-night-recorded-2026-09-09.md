## 81. The fuzzer runs at night, recorded 2026-09-09

`just fuzz-overnight` fuzzes all four targets in turn inside a fixed window, and
a systemd user timer runs it from 00:10 to 08:10. It is the first thing in this
repository that runs without somebody starting it.

What it adds is **hours, and not a second opinion.** §29.9 is the reason there
is anything to add: `cargo-fuzz` builds with a sanitizer that wants a nightly
compiler and `rust-toolchain.toml` pins a stable one, so the gate does not fuzz
and never has. CI runs `just gate` on every push and has been green since
`34266469454` (§77.6), and none of those 1811 tests is one of these four
targets. The night is the only place they run at length, on the same machine and
reported to the same person.

### 81.1 Why the night's corpus lives outside the tree

`fuzz/corpus/` is tracked, 30 MB across the four targets, and the tracked corpus
is the point: §29.9's 562 minimised inputs include 32 written by hand and named
for the configuration each one is. Eight hours a night would bury those under
thousands nobody chose. Measured rather than guessed — the reader target wrote
294 new inputs in its first sixty seconds against a fresh directory.

So the tracked corpus is handed to libFuzzer as the **second** corpus argument,
which makes it a read-only seed, and the night's own findings go to the first:
`~/rcad-fuzz-corpus/<target>`, outside every worktree. Promotion into
`fuzz/corpus/` stays a thing somebody decided.

Order is the only thing that says which directory is which, so
`check-fuzz-overnight-selftest` asserts the order rather than trusting it.

### 81.2 The window is a deadline, not a budget

Each target gets an equal share of *whatever is left when its turn comes*,
computed against the clock each time round, rather than a quarter of the window
computed once. A slow build, or a target that stops early, is paid for out of
the same window instead of added to the end of it.

The failure this prevents is silent and would have run until lunch: four targets
each handed the full window is a recipe that passes every other assertion and
takes four times as long as it was told to. The selftest runs a four-second
window against a stub `cargo` that sleeps out exactly the slice it is granted,
so the clock being divided is a real clock, and it fails that mutation.

There is no minimum slice. A floor of "long enough to be worth starting" is a
number nobody can defend — an 18 MB corpus takes seconds just to load, so the
honest floor is per-target — and it would make the window untestable at any size
a test can afford to wait for. A target is skipped when there is no time left,
and said to be skipped.

### 81.3 A crash does not end the night

`cargo fuzz` exits non-zero and writes the input to `fuzz/artifacts/<target>/`,
which is ignored (`fuzz/.gitignore:1`) and therefore survives a `git clean`
without dirtying anything. A crash in the reader is no reason to cost the fuse
its hours, so each target is reported and the run carries on, failing at the end
if anything failed.

### 81.4 Not in the hub, and not silently stale

The recipe refuses to run in the hub, using the same parse `check-hub` does.
§60's rule is that nobody works in the hub because `git worktree lock` refuses
there, and an unattended eight-hour job is the one kind of work that most needs
an owner and can least be asked for one: `just lanes` would print `free` all
night beside a tree in use. The timer gets a worktree of its own, locked, with
the reason `overnight fuzzing` — a lock held by a machine rather than a session.
§76.4 says most worktrees carry no lock and names the ones that should; this is
one of them.

**The lock is what protects it, and the reason string protects nothing.**
`prune-worktrees` and `clean-targets` skip any locked tree without reading why
it is locked, so those are already safe. What the reason is for is the sweep
that is done by hand: on 2026-09-09 a session cleared three stale locks and had
to ask the other sessions one at a time whose was whose, because a lock date
does not prove its owner is gone. A reason without the `agent ` prefix says *no
session will ever answer for this one* — to a reader, and to nothing else.
Making the prefix load-bearing in code would be a change to `prune-worktrees`,
which is §76's business and not this section's; recorded here as the gap it is,
rather than claimed as a property it has.

**Amended by §90 on 2026-09-11: the night now advances itself, onto the last
commit CI went green on.** What this paragraph said, and why it was wrong, kept
here because the reasoning is most of what §90 argues against:

> Nothing advances that worktree to `main`, and nothing should: an unattended
> `git checkout` racing the lanes that merge into `main` is a write nobody
> reviewed. The cost is a timer that can sit on one commit for a month, fuzzing
> something nobody ships — §62's shape exactly, a run that completes and was
> settled before it started. So the commit and its age go in the summary line,
> where the staleness is visible instead of assumed. Advancing it is a person's
> `git -C ../rcad-nightly-fuzz checkout --detach main`.

The objection was sound and the remedy was not. "A write nobody reviewed" is
answered by a green gate, which §77 had already put on every push to `main` and
which this section did not think to ask for. And the cost it accepted came due
on the third night, not in a month: the visible staleness it relied on was the
commit's *age*, which cannot see the failure it exists to see. A commit one day
old can be sixty-eight commits behind, and on 2026-09-11 one was.

The hand-advance stays exactly as written — §90 only ever moves forward, so a
tree somebody put on a commit deliberately is never dragged off it.

### 81.5 systemd rather than crontab

`tooling/systemd/rcad-fuzz.timer` and `.service`, installed by copying them into
`~/.config/systemd/user/`. They are **in the tree**, and the reason is this
section: it described two files no reader of the repo could open, which is the
same shape as the sentence §81.4 had to withdraw. `%h` for the home directory is
what makes them trackable, and it costs nothing — the worktree layout below it
is this project's convention either way.

`OnCalendar=*-*-* 00:10:00`, and `Persistent=false` because a missed midnight
must not fire at ten in the morning and fuzz until six. `TimeoutStartSec=29400`
is the outer backstop that makes 08:10 true even if the recipe misbehaves; a
crontab line cannot stop itself and would need a second entry to do it.

**That backstop was written as `RuntimeMaxSec=` and did nothing.** With
`Type=oneshot` the whole run counts as *starting*, so `RuntimeMaxSec=` is
ignored; `TimeoutStartSec=` is the knob that bounds `ExecStart`, and its default
is infinity. Nothing in a night's output would have shown the difference —
`fuzz-overnight` stops itself, so the backstop is only ever load-bearing on the
night the thing it backs up is broken. What caught it was
`systemd-analyze --user verify`, in one line, before the file was ever
installed. Verify the units; do not read them.

The installed unit says it a second way, and this is the demonstration rather
than the argument — `systemctl --user show rcad-fuzz.service`, with the fix in
place:

```
TimeoutStartUSec=8h 10min
RuntimeMaxUSec=infinity
```

8h 10min is the backstop doing its job. `infinity` on the line beside it is what
the original spelling would have left behind, on a unit that loads without
complaint and runs without one.

There is deliberately **no `check-*` arm** running that verify, and the reason
is §77's. The gate runs on a GitHub runner, `systemd-analyze` is not something
this repo installs or pins, and the two ways to write such an arm are both the
failure §77 named: fail where the tool is missing and the gate breaks for a
property of the machine, or skip there and the arm never runs on the only
machine that runs it every push. So this is a two-file, run-it-yourself check,
named here so the next person editing a unit knows the command exists.

The unit calls `mise` by absolute path. cron and systemd both start without a
profile, so `just`, `cargo-fuzz` and the `RUSTC_WRAPPER=kache` that `mise.toml`
sets are all absent from the environment, and cargo fails outright on a wrapper
it cannot find (§8.1). That is the failure this would most likely have had, and
it would have arrived as a silent non-zero exit at ten past midnight.

---

