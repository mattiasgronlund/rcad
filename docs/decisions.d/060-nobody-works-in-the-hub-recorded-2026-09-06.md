## 60. Nobody works in the hub, recorded 2026-09-06

§59 belongs to `rcad-c4`, which had claimed it in a worktree rather than in this
file. That is the same collision §55's preamble opens with and §57.8 built a
check for, and it happened again while this section was being written — so this
one moved rather than argue about it. The two sessions agreed by message, which
is a record that dies with them; the number in the heading is the one that does
not.

### 60.1 The tree the record cannot describe

§55 settled that a worktree's git lock is this repository's statement of who owns
it, and §55.4 gave the reason: it is the only candidate that survives the session
that wrote it. The alarm here was raised by a session trying to use it for
exactly that — announcing a ten-minute build before starting it — and finding
that it could not.

`git worktree lock` refuses the main working tree. There is no flag. So the one
tree that the record structurally cannot describe is the tree every session lands
in by default, and `just lanes` was printing `free` beside it whoever was
building in it. That is not a gap in the tooling; it is `lanes` reporting the
only thing git will tell it, which happens to be indistinguishable from an
answer.

The fix chosen is not a second record beside the lock. It is to delete the case:
**nobody works in the hub.** A tree nobody works in needs no owner, and `free`
becomes true about it rather than a guess.

### 60.2 The demonstration

Three refusals, all measured on 2026-09-06 in a repository built for the purpose.
The first is why the hub cannot be owned; the second and third are why `main`
cannot stay in it.

| what was tried | what git said |
|---|---|
| `git worktree lock --reason "test" .`, and by absolute path | `fatal: The main working tree cannot be locked or unlocked` |
| `git fetch . lane:main` from a lane, `main` checked out in the hub | `fatal: refusing to fetch into branch 'refs/heads/main' checked out at .../main` |
| `git push . lane:main`, same shape | `! [remote rejected] lane -> main (branch is currently checked out)` |

### 60.3 Why `main` leaving the hub is the load-bearing half

"Sessions use worktrees" on its own does not survive contact with a merge. Rows
two and three above say a branch checked out in the hub cannot be moved from
anywhere else, so every merge has to be performed *in* the hub — which is the one
moment when two sessions most need to know who is there, in the one tree that
cannot say. A rule that holds until the interesting moment and then breaks is
worse than none, because it is believed.

Parked on a detached HEAD the hub holds no branch, `main` is checkable out in a
worktree like any lane, and the merge tree is locked and named like any other.
This was already the live state when the section was written — `main` was checked
out and locked in `rcad-worktrees/wave-1`, held by a peer session mid-merge, and
the hub was detached. Somebody had reached the same design and not written it
down, which is the failure mode §55 was itself about.

### 60.4 What changed

| # | change | what proves it |
|---|---|---|
| 1 | `just check-hub` — fails when the hub has a branch checked out or uncommitted work in it; in `default` | `just check-hub-selftest`, in `default`; walks one fixture hub through lone-checkout, on-a-branch, dirty and parked, and asserts the middle two fail |
| 2 | `just lanes` labels the first tree `hub`, leaves it out of the lane count, and warns when it is not parked | `check-lanes-selftest`, extended; its fixture hub sits on a branch, so the warning is exercised |
| 3 | `.claude/hooks/session-start-hub.sh` tells a session that started in the hub to call `EnterWorktree` | `.claude/hooks/check-session-start-hub.sh`, in `check-hooks`; five cases, four of which assert *silence* |
| 4 | `just clean-targets` asks git for the worktrees instead of guessing a path, and so reaches the sibling trees and the hub's own 14 GB | `check-clean-targets-selftest`, in `default`; asserts the hub's `target/` goes and that a locked tree's survives |
| 5 | CLAUDE.md gains "Nobody works in the hub"; `delegating` gains never-merge-in-the-hub and a reclaim-and-re-park close | prose |

### 60.5 A silent sweep that had been silent for a month

Item 4 was meant to be a widening and turned out to be a bug fix. `clean-targets`
built the string `$(dirname $(git rev-parse --show-toplevel))/rcad-worktrees`,
which is the right answer only when it is run from the hub. Run from a lane it
resolved to `rcad-worktrees/rcad-worktrees`, found nothing, and printed *"there is
nothing here to clean"* — an empty answer wearing the clothes of a clean one, the
same shape as §57.8's and §55's. The rule in this section makes every run a run
from a lane, so it would have been every run from here on.

The disk was at 92% when this was found.

### 60.6 The exemption, and why it is structural

`check-hub` passes when the repository has exactly one worktree, because then
there is no hub and no lane, only a checkout, and no second session to collide
with. That is a fresh clone, and it is the shape a CI checkout has. CI runs
`check`, `lint`, `test` and `deny` rather than `default`, so the check does not
fire there today and the exemption is not load-bearing for the build — it is
what would keep it green if `default` ever moved into CI.

It is deliberately not an environment variable: an exemption that has to be set
is an exemption that gets set, whereas this one lapses by itself the moment a
second tree exists, which is the moment ownership starts being a question.

### 60.7 What this does not close

**Nothing stops a session working in the hub.** The session-start hook is a
notice and `check-hub` is a check; both can be ignored until the next `just
default`. The mechanical version is a `PreToolUse` hook that refuses `Edit` and
`Write` there — `no-in-place-rust-patching.sh` is the precedent — and it is
deliberately not built: §13 wants the consumer named first, and it would also
refuse Mattias's own `!` commands in the directory he starts in, which is a cost
worth paying only once the cheap version is shown to leak.

**Nothing re-parks the hub either.** After a merge the hub sits on whatever
commit it was detached at, and it is a person or the closing gate that moves it
forward. That is the safe direction to fail — a stale hub is harmless where a hub
holding `main` is not — but §55.6's "an abandoned lock is rubbish only a person
clears" now has a sibling.

### 60.8 Untracked counts, and `fuzz/corpus/` is not ignored

`rcad-c4` asked the two questions this section had answered only by implication,
and both deserve the record rather than the implication.

**Untracked-and-unignored is uncommitted work.** A check that looked only at
tracked modifications would have passed the hub all evening while it held the
single most valuable thing at risk: 1542 corpus entries a fuse campaign had
earned and left there. When the campaign moved to a worktree it started from the
tracked 1088, so those 1542 existed in exactly one place, and that place was the
tree nobody owns. They are now committed — replaying takes the corpus from
`cov: 13342 ft: 54505` to `cov: 14123 ft: 65660`, so they were coverage and not
noise. A rule that ignored untracked files would have been silent about precisely
the harm it exists to prevent.

**`fuzz/corpus/` stays tracked.** The suggestion was to ignore it so that a fuzz
run stops dirtying whatever tree it is in. That is backwards: 3999 corpus entries
are committed on purpose, `fuzz/.gitignore` names `artifacts/`, `coverage/` and
`target/` and pointedly not `corpus/`, and a seed corpus that is not shipped is a
fuzzer that starts from nothing on every machine. The dirt a campaign leaves is
not the problem; leaving it in the hub is. The remedy is the rule, not an
exemption from it — and the exemption would have thrown away tonight's 1542.

**What no check here can see.** The collision that sent this section from §59 to
§60 was caught by one session telling another, not by `check-decisions`, which
reads one file and so fires when the second branch *merges* rather than when the
number is taken. §59.5 records that from the other side. It is the same fact as
this section's: two sessions cannot see each other's work in progress, and the
answer is a record that outlives the session — a lock, a heading, a commit — not
a smarter check. Firing at publication is the right time; nothing above claims
otherwise.

### 60.9 A stale hub is not leftovers, and the message has to know which

§60.7 called an un-re-parked hub harmless. It is harmless to the *repository* and
it was not harmless to this check, which `rcad-c4` spotted before it bit anyone:
a hub parked at an older commit shows every path that has landed since as
untracked, indistinguishable by `status` from work somebody abandoned there. The
symptom is one thing and the two causes want opposite fixes, so the message that
says "commit this from a worktree of your own" sends a reader to duplicate a
merge that has already happened.

They are told apart by asking git rather than by guessing: if nothing is modified
and **every** untracked path is already in `ls-tree main`, the hub is behind, not
dirty. `check-hub` then names the command that fixes it and says that nothing
moves the hub's HEAD on its own. `check-hub-selftest` builds that state — a file
committed on `main` from a lane, present on disk in a hub whose HEAD predates it
— and asserts both that the stale message appears and that the leftovers message
does not.

**The first remedy this section printed did not work.** It was
`git -C <hub> checkout --detach main`, on the reasoning that files whose content
already matches the target commit cannot be at risk. Git does not reason that
way. It refuses on the paths alone:

    error: The following untracked working tree files would be overwritten by checkout:
            fuzz/corpus/aaa
    Please move or remove them before you switch branches.

Measured both ways in a throwaway repository: byte-identical content aborts, and
differing content aborts with the same message, so the two are indistinguishable
from the output. `git clean -fd` first, then the checkout, exits 0, restores
every file byte-identical, and leaves `git status` empty. The published line is
therefore `git -C <hub> clean -fd && git -C <hub> checkout --detach main`, and
`clean` without `-x` leaves ignored files alone, so a campaign's crash artifacts
survive it.

So the files are deleted either way. "Re-parking makes them tracked rather than
deleted" was argued between the two sessions and never reached this file, which
is the only reason it is not a correction to a published paragraph. What survives
of it is the *timing*: deleting at re-park time is safe because by then the
content is on `main`, where deleting before the merge would have left
`lane-hub-rule` as the only copy.

**The selftest runs the line the check prints**, greps it back out of the output,
executes it, and asserts the hub lands on `main`, that `status` comes back empty,
that the removed file is restored, and that `check-hub` then passes. A remedy
nobody executes is a claim without a test, which is §15.6's rule pointed at a
string rather than at a bound. Restoring the old one-command version fails it
with *"the remedy the check prints does not run"*.

Found by `rcad-c4` running it rather than reading it — twice over, since §60.7's
"harmless" is what pointed them at the case to begin with. Both defects were in
prose that had been reviewed and neither was in code that had been tested, which
is the argument for the paragraph above.

### 60.10 The same name is not the same file

§60.9's test for "the hub is only behind" was **every untracked path is already
in `ls-tree main`**. That compares names. It stood in front of a `clean -fd` that
deletes those same paths, so a file in the hub whose name is on `main` but whose
bytes are not was told it was safe and then removed.

Demonstrated in a throwaway repository rather than argued. A hub parked one
commit back, holding `notes.txt`:

    hub blob:  3efd841a4fa917602af4c8547eb46d369b94a4d1
    main blob: 56985387e75729cd805934b2322e7d79ff9bd594

`check-hub` answered *"Every one of those is already committed on `main`, so
nothing here is lost work"* and printed its remedy. Running that remedy: the file
afterwards holds `main`'s version, and the hub's content is reachable from no ref
at all. It was never in a commit; there was nothing to recover it from.

**It could not have bitten on 2026-09-06, and that is luck rather than design.**
The 1542 were libfuzzer corpus entries, which are named by the hash of their
contents, so for them a matching name *did* imply matching bytes. `check-hub`
does not know that, and nothing restricts it to `fuzz/corpus/`. A hub parked
behind `main` with a half-written `docs/notes.md` in it, while a lane lands a
different `docs/notes.md`, is the same shape with a person's work in it.

The check now hashes each path and compares against `main:<path>`. It enumerates
with `ls-files --others --exclude-standard -z` rather than reading `status`,
which reports an untracked directory as a single entry and quotes awkward names.
The one thing `clean -fd` removes that this does not hash is an empty directory,
which holds nothing to lose.

**The old fixture could not fail.** It wrote `landed` with the same content on
both sides, so a name-only comparison and a content comparison agreed on it —
the test passed for a reason weaker than the one it was believed to prove. The
new case writes different bytes under the same name and asserts the "already
committed" sentence does *not* appear. Put the name-only comparison back and it
fails with *"content that is on no branch was called committed, and the remedy
deletes it"*.

**Two more things measured during the re-park, both about the same remedy.**

`checkout --detach main` does **not** restore a tracked file deleted in the
working tree when that file is identical in the old commit and the new one.
Checkout writes only what differs between the two, so the deletion survives it —
and checkout *prints* `D <path>` while doing nothing about it, which is the one
line a reader would scan for reassurance. A separate `git restore -- <path>` is
required. This bit a real file: a `crash-*` sweep had deleted
`fuzz/corpus/no_model_makes_the_writer_panic/crash-a54b24c9…`, a tracked seed
from `ce2dbf2` that is named like an artifact but is not one, and the re-park's
own assertion is what caught that the checkout had left it deleted.

`git clean -fd` will not remove a nested linked worktree — measured on **git
2.43.0**, with `.git` confirmed to be a file rather than a directory, `clean -nd`
listing it as no candidate at all. `git clean -ffd` removes it, taking
uncommitted work with it and leaving the registration `prunable`, which `just
lanes` then reports as `missing`. So the protection is a property of `clean`
stopped at one `-f`, not of `clean`; the published remedy must never gain a
second. The version is part of the claim because `rust-toolchain.toml` pins the
compiler and `mise.toml` pins the tooling, and nothing here pins git — §15.6 for
why a bound without its conditions is not a bound, and `dc650e7` for what
happens to a "the guard holds" sentence that travels without them.

**A worktree made from the hub lands inside the hub.** `git worktree add
rcad-worktrees/<name>` run with the hub as the working directory creates
`<hub>/rcad-worktrees/<name>`, not the sibling directory of the same name. An
empty `rcad-worktrees/` had been sitting in the hub since 2026-09-06 08:42 from
exactly that slip, and it went silently into the re-park's delete set — noticed
only because a dry run counted 1543 where 1542 had been verified. `rcad-c4` then
made the identical mistake while opening the lane for this section, which is the
evidence that it is easy rather than careless. Absolute paths in `worktree add`.

