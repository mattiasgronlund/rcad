## 159. `just claim-section` — a section number taken when asked for, built 2026-09-19

### 159.1 What was wrong

§144.5 is the renumber problem: a section is appended at the tail by whoever
finishes first, and two sessions reach for the same next number. The check the
memory note prescribed — `git log --all -S'## N. '` — was right every time it
ran and lost anyway, because it reads a value another session is about to write,
and nothing holds the answer for the hour it takes to write a section. §144.5
lost that race while being written, §152.7 lost it while fixing §144.5, and
§157.6 lost it three times running. `check-decisions` catches the collision, but
only at the gate, after the section and every citation of it have been written
against the wrong number.

§144.5 said what was needed: the number taken at the moment of asking rather
than at the moment of committing.

### 159.2 What was built

`just claim-section 'what it is about'` prints one number. Under a `flock`, it
takes one past the highest of:

1. every `## N.` on every local branch and remote-tracking branch;
2. every `## N.` in every worktree's copy of the file, **committed or not** — a
   lane writes its section long before it commits it;
3. every number handed out before,

and writes the new number to the claims file before printing it. The claims file
is `rcad-section-claims` in the git common directory, so every worktree of this
clone shares it and no branch carries it. Each line says the number, the time,
the worktree that asked, and what for.

On the real repository it takes 1.1 s across 183 branches: blobs are
deduplicated before being read, since most branches share the same copy of the
file.

This section's number came from the recipe.

### 159.3 What it does not do, and why that is fine

**A claim that is never used leaves a gap.** `check-decisions` asks for unique
and ascending numbers, not consecutive ones, and a gap has never cost anyone
anything. Freeing claims would bring back the race it closes.

**It covers one machine.** A section written somewhere else is seen once it is
on a branch here — the same reach the `git log --all` check had, so nothing is
lost. Every session on record here ran on this machine.

**Nothing forces a session to use it.** `CLAUDE.md` names it beside
`decisions-find`, and the memory note that prescribed the hand check now points
here. A hook that refused a new `## N.` nobody claimed was considered and not
built: the claims file does not travel with a branch, so on CI and on a fresh
clone it would refuse every section.

### 159.4 The test

`check-claim-section-selftest`, in `just gate`, builds a throwaway repository
with a section in each of the three places, one at a time, so that each one is
the highest when it is tested. Then it runs eight claims at once and asks for
eight different numbers.

Each part was checked by breaking it on a scratch copy of the `justfile` and
watching the selftest fail:

- no lock: the eight concurrent claims all got the same number;
- no branch scan: the first claim got 3, not 6;
- no worktree scan: the second got 7, not 10;
- previous claims not read: the claim from the other worktree got 10, not 11.

---
