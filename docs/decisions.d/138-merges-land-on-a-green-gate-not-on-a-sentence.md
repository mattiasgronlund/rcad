## 138. Merges land on a green gate, not on a sentence, and a session ends finished or written down, decided 2026-09-19

Mattias, in his own words on 2026-09-19: he has never blocked a merge, so
holding one for his decision is wasteful; a merged commit could still be backed
out, and that has never been needed either. His trust in the recommendations is
high enough that asking is the cost, not deciding. And a stretch of work should
be run to a state where it is either complete or written down and postponed,
rather than parked.

Three rules follow. They are working practice, not kernel design, and they are
recorded here for the same reason §60 is: a habit that lives only in a session's
memory dies with the session, and the four sessions that share this repo would
each have to be told again.

### 138.1 A green gate is the authorisation to merge

`just gate` on the merged tree is what clears a branch to land. No sentence from
Mattias is needed and none should be waited for.

The arithmetic is not close. The wait has never once changed an outcome — no
merge has been blocked — so its expected value as a check is zero, while its
cost is a round trip that lands, by construction, at the *end* of a session,
when the prompt cache has cooled and the whole conversation is re-processed to
read one word. That is the most expensive turn a session has, spent on its least
work.

What still holds is the gate itself, and the two traps around it that are about
evidence rather than permission: gate the tree you actually merged, because
`main` moves under a lane while it runs, and never write down a gate you have
not read — §127.1's "green on this tree" was written while the run was still
going, and the run failed.

**Backing out is the safety net, and it is cheap.** A bad merge is a revert on a
branch whose whole history is on disk. That is why this trade is one-sided: the
thing being protected against costs a commit to undo, and the protection costs a
cold-cache turn every time it is applied.

### 138.2 Take the recommendation instead of offering it

Where the habit was a `Q`-labelled question carrying a recommendation and a
narrowing, take the recommendation, say which way you went and why, and carry
on. A question whose answer you can already predict is not a question; it is a
paragraph of narration with a stop in the middle.

`open-questions.md` keeps the ones that are genuinely open — where a wrong
answer is expensive to undo, or where the shape of what Mattias wants is not
derivable from the record. The `asking-questions` grammar, the status line and
`just question-check` are unchanged for those; nothing about §100 is repealed.
So is the `## Waiting on you` block, for the turns that still end with one.

**A decision taken alone is still written down.** The record is what made the
asking safe to drop — the commit message for a small one, a `docs/decisions.md`
section for one with an argument behind it, the handoff for one that is still
running. Deciding without asking and deciding without recording are different
things, and only the first is what was granted here.

### 138.3 End finished, or end written down

Drive the work to completion, or write it into the handoff as postponed with
what is done and what remains, and end there. Do not idle on a question.

This is the same cost as §138.1 seen from the other end. A half-finished lane
that nobody wrote down is reconstructed at full context price by whoever picks
it up, and a session parked on a question pays that price to ask it. The
`handoff` skill already produces the artefact; the rule is that producing it is
part of stopping, not an option beside it.

### 138.4 What this does not touch

A peer's relayed word is still not Mattias's — but that now matters only where a
decision genuinely is his, which after §138.1 is a much smaller set. The half
that survives intact is the other one: a session that was **refused permission**
for an action may not hand it to a peer to perform. Being the hands for another
session's refusal is not a decision to take, at any level of trust, and §138
grants nothing there.

Nor does any of this loosen the gate, the branch-per-change rule for test
claims (§8.3), or `Model::validate` at the boundaries. Autonomy was granted over
*asking*, not over *checking*.

