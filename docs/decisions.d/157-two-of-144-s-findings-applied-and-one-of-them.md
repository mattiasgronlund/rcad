## 157. Two of §144's findings applied, and one of them is not a lane, measured 2026-09-19

*§144.7 said the batching rule asks for something that mostly cannot be done, and
called itself a `CLAUDE.md` correction rather than a lane. §144.5 listed lane and
worktree admin second by size and implied, with the other five rituals, that it
was scriptable. Both were taken up here. The first is applied as written. The
second turned out to be wrong in an interesting way: the admin this repo was
warned about has stopped, what replaced it is not scriptable, and the honest
deliverable is a corrected paragraph and no recipe at all.*

### 157.1 What was measured

`just spend 2026-09-12 2026-09-20 ritual`, then two throwaway probes over the
same transcripts, reusing `spend.load` so the response grouping is the one
§144.1 insists on — `(requestId, message.id)`, never per record.

The probes asked what is *inside* the `lane / worktree admin` line, which the
`ritual` report gives only as a total: which sub-shape each matched response
carried, whether admin responses come in runs or singly, how often a session asks
"who holds what" more than once and how long it waits, and whether the ask was
piped through `head` or `tail`.

Neither probe is kept. They are twenty lines each against a public function, and
`RITUALS` in `spend.py` is where a shape worth keeping would go.

### 157.2 The batching rule was aimed at a population it could not move

§144.7 is applied to `CLAUDE.md` as it was written, with one number refreshed:
the share of responses calling more than one tool is 12%, about a point up over
two weeks, which is nothing.

The paragraph used to quote 1 655 read-only calls that directly followed another
as "9% of the week's bill spent on the split and nothing else". That framing is
what the correction removes. The pairs are mostly **dependent** — grep for a
line then read at that offset, count then read the tail — and a dependent pair
cannot be issued in one call however firmly it is asked for. The instruction to
batch independent probes is kept, because it is still right; what is removed is
the implied size of the prize.

What the dependent pairs actually want is a one-call reader for the file, which
is §144.6's lane and landed as §153's `just recipe` while this was being written.
That is the same finding from the other end: where a reader exists the shape is
one call, and where it does not, no amount of batching advice makes it one.

### 157.3 Lane admin is the second ritual by size, and over half of it is asking

The line is 5.7% of the week, second of the six. It is **larger** than the 3.3%
`CLAUDE.md` quoted for the week of 2026-09-07, not smaller, and the composition
has inverted.

What the paragraph was written against has gone. It describes a session unlocking
fifteen worktrees in fifteen calls and another hand-rolling a `du`/`find`/`rm`
sweep, and concludes that nearly all of the cost "was a recipe nobody ran".
Consecutive runs of three or more admin responses are now under a fifth of the
line, and the three recipes it names — `clean-targets`, `prune-worktrees`,
`release-lanes` — account for 152 responses in the week. They are run. That
sentence is now false and is corrected.

What is left is 262 separate asks of "who holds what" across 86 sessions: `just
lanes`, `git worktree list`, `just status`. Over half the line's cost, and the
larger half. Sixty-seven of the 86 sessions ask twice or more; the median gap
between one ask and the next is six minutes.

### 157.4 The fixes considered for the asking, and why none is built

§16.10 wants the family named rather than a paragraph, so:

- **A recipe that collapses the sequence.** `just land <branch>` — lock the tree
  holding `main`, fast-forward, verify, push, unlock — is a real gap and this
  session paid six calls for exactly that sequence. But landing is not where the
  money is: lock, unlock, add and remove together are the smaller half, and a
  landing recipe would not remove one ask of "who holds what".
- **Stop the re-asks.** Thirty-six per cent of re-asks come within two minutes of
  the previous one, which is too soon for state to have changed. That is the
  clearly wasted part and it is **$5** of the week — below the cost of the lane
  that would chase it.
- **Carry the answer somewhere that persists.** The status line is the obvious
  candidate and was rejected: it is rendered for the reader of the terminal, and
  a session cannot read its own. Claiming otherwise would have been exactly the
  unearned assumption the core principles forbid.
- **Make the answer smaller.** Rejected as backwards. `just lanes` is already one
  screen; the problem is not its size.

What the measurement does support is one sentence, and it is now in `CLAUDE.md`:
**take `just lanes` whole.** Fifty-eight per cent of the asks cut it through
`head` or `tail`, and a truncated ask is re-asked 71% of the time against 62%
for a whole one. That difference is real but modest and the sample is small, so
it is written as advice with its own arithmetic beside it, not as a rule.

The rest is irreducible, and for a good reason: lanes genuinely are locked and
unlocked by other sessions while you work. A session that asks twice an hour is
not being wasteful. **Lane admin is the ritual line that should not have a lane**,
and §144.5's list is corrected to say so.

### 157.5 The numbers move under you, and the shares do not

Between the first `ritual` run of this session and a re-run twenty-five minutes
later, the window total went from $1,104 to $1,197 — nine hundred and eighty-six
responses, written by other sessions while this one measured. Every share held:
ritual 19.3% both times, this line 5.8% then 5.7%.

That is §144.1's reason for quoting no totals, demonstrated inside a single
session rather than argued. Quote shares; name the report; re-run it.

### 157.6 Three section numbers were taken while this one was being chosen

§144.5 says the renumber collision is "a read against a value another session is
about to write", and that no rule addressed to the person picking can work.
Choosing a number for this section hit it three times: §145, §146 and §147 were
each checked with `git log --all -S`, each came back taken, and `main` moved from
§144 to §156 in the ninety minutes it took to do the work above.

Two of those sections are §144's own lanes, taken by other sessions concurrently
and landed before this one: §147 is gate babysitting, §152 the question file,
§153 `just recipe`. Three of the five lanes §144 opened were done by someone else
while a fourth was being measured.

That is the strongest evidence yet for the recipe §144.5 asks for — a number
taken at the moment of asking rather than at the moment of committing — and it
remains unbuilt. It is the one piece of §144.5 nobody has taken.

