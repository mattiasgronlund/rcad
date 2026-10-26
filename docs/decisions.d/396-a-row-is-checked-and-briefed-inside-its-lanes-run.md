## 396. A row is checked and briefed inside its `lanes` run, not by the dispatcher — 2026-10-03

*A dispatcher writes about 20 000 output tokens before its first lane, and
thinking stays in context (§266.1). This section measures what those tokens
were spent on. Close to half before the first lane, and a third over a whole
session, go to two jobs it does again for every row: checking the row's write
set against the code, and writing the row's brief.
Both move into the `lanes` workflow as a first stage, the way merging moved
there in §266. Picking the rows, the `take`, and deciding a returned run stay
with the dispatcher.*

### 396.1 What was measured

Dispatcher transcripts under `~/.claude/projects/`, in the trees
`dispatch-next`, `dispatch-2026-10-03`, `dispatch-q655`, `dispatch-q792-q793`
and `ci-verdict-cache`. A session cut by `/clear` counts as two. Of these
segments, 69 sent a lane between 2026-09-25 and 2026-10-03. The nine that
started on or after 2026-10-01 are the ones below: they ran today's process
(one `lanes` run per row, §266; lanes writing their own rows, §384), and each
of them ends its window at a `Workflow` call on `lanes.js`. Of the older 60,
20 end at an `Agent` call instead, 17 of them `general-purpose`.

How a response is counted:

- One response is one `(requestId, message.id)`. Its records repeat the
  usage, so its output is taken once, as the largest `output_tokens` among
  them.
- Its output is split evenly between the tool calls in that response, and
  each call is put under what it was for. A response with no call counts
  as "other". Thinking text is stored empty, so *what* it thought cannot be
  read, only how much, and next to which calls.
- Each call's result is counted in characters, not tokens.

What each heading holds:

- **write the briefs**: the `Workflow` or `Agent` call itself, and reads of
  `lanes.js` and of `just recipe`;
- **check a row against the code**: `Read`, grep or `just src` into
  `crates/`, `just decision`, `just decisions-find`, `just claim-section`;
- **queue**: `just queue` reads and writes, grep or `Read` of
  `docs/queue.md`, and the commits of it;
- **a returned run**: run journals and transcripts, `git` on `lane-*` and
  `wf_*` branches, `just ff-main`;
- **slots and trees**: `just lanes`, `just status`, `git worktree`,
  `ListAgents`, fetching and checking the base;
- **start**: loading skills, reading the handoff and memory, `ToolSearch`;
- **other**: everything else.

Means per segment, over the nine:

| what for | output before the first lane | result chars before it | output, whole segment | result chars, whole segment |
|---|---:|---:|---:|---:|
| write the briefs | 5 183 | 15 642 | 10 247 | 21 061 |
| check a row against the code | 4 459 | 12 663 | 8 602 | 25 561 |
| queue | 4 396 | 14 258 | 13 164 | 30 126 |
| a returned run | 3 336 | 23 882 | 5 820 | 34 556 |
| other | 2 167 | 13 718 | 10 606 | 18 749 |
| slots and trees | 1 241 | 4 087 | 3 796 | 11 324 |
| start | 805 | 20 147 | 1 629 | 22 343 |
| **all** | **21 587** (median 19 076) | | **53 864** (median 53 675) | |

Over all 69 segments the order is the same at the top: write the briefs 29%,
queue 22%, check a row 16%, a returned run 10%, with a median of 24 877
output tokens before the first lane. All 69 segments record `"effort":"xhigh"`.

What the two moved jobs cost per lane: the nine made 34 `Workflow` calls
holding 36 lanes between them, and no `Agent` call. Divided out, a lane costs
about 2 560 output tokens to brief and 2 150 to check, plus about 5 300
characters of results to brief and 6 400 to check. That is paid at every
fill, not only the first. Before the first lane, the median `Workflow` call's
input alone was 5 352 characters.

The scripts were `bucket_output.py`, `first_call.py` and `effort.py` in
session `a00f0853`'s scratchpad under `/tmp`, which a reboot wipes. Q819
rebuilds the counting in `just spend` from the list above.

### 396.2 What is already filed against each share

- **A returned run**: Q813, whose section is claimed and not yet written.
  The dispatcher that gets a run back
  decides it and writes the decision to the queue, so a dispatcher started
  later does not read the run's journal again.
- **Start**, and the `lanes.js` reads inside "write the briefs": Q818 moves
  the parts of the `dispatch-lanes` skill a fill does not use into
  `reference.md`, and gives one full args example, so a dispatcher stops
  opening `lanes.js`. The settings trim offered to Mattias on 2026-10-03
  takes the skills and connectors rcad does not use off every session's
  start.
- **Queue**: nothing yet. Some of it goes with the check below (a row
  re-filed because the check found its write set wrong). The rest is `queue
  next`, the takes, and filing rows. Q819's report splits it if it stays the
  biggest share.

Left: write the briefs and check a row. Together they are 9 642 output tokens
and 28 305 result characters before the first lane, and 18 849 and 46 622
over a segment, a third of everything the dispatcher writes.

### 396.3 Why the check moves and does not stop

The check finds things. Since 2026-09-25, `origin/main` has 51 queue commits
whose subject re-files or replaces a row, against 128 that take one. About 28
of them fix a write set in the same commit as a take, with no run behind the
fix: the dispatcher's own check caught it. About 12 re-file a row after its
verifier or merger found a file outside its set: the check missed it. These
counts come from grepping commit subjects and are approximate, but they put
a wrong write set at something like one fill in five. Dropping the check would
turn each of those into a lane that stops or writes outside its set.

Under §158.6's test it is also a hunt. The dispatcher is looking for a file
the row does *not* name, so it cannot name the file before it looks. §158
already says a hunt goes to a lane at its first probe. Step 4 of
`dispatch-lanes` asked for this one inline anyway.

### 396.4 What is built

- **`lanes.js` takes a lane as `{ label, queueId, notes }`.** For such a lane
  the run starts with a stage before the lane, **prep**: a Sonnet agent of a
  new type, `.claude/agents/prep.md`. Like `lane.md` and `verifier.md`
  (§266.6), it holds the standing rules. They are the ones steps 4 and 5 of
  `dispatch-lanes` hold today:
  - checking the write set against the work;
  - naming the recipe that pins a count the row moves (`check-*`,
    `check-ignored`, `wildcard_arms`);
  - the model per row, and per bundle (§352.4);
  - `sourcePaths`;
  - `pinsOnly`, said explicitly and never inferred (§287);
  - the verbatim API signatures and exact-arithmetic facts the lane will
    need.
- **Prep reads; it writes nothing.** It reads the row or rows with `just
  queue show`, and the code at the run's base. It needs no tree of its own.
- **It answers `ok` or `needs`.**
  - `ok` comes with `writeSet`, `model`, `sourcePaths`, `pinsOnly` and the
    facts. `lanes.js` builds the brief from the row's text, those facts and
    the dispatcher's `notes`, and the run goes on as today: lane, verifier,
    merger.
  - `needs` names each missing file and why. The run then stops before any
    lane starts and comes back not merged. The dispatcher re-files the row
    (drop and add, then `queue next`) and decides it as Q813 says.
- **Prep never re-files a row itself.** Only `queue next` checks a write set
  against the rows already running; `take` does not (`take_step` in
  `tooling/queue/queue.py` checks holds, other copies and readiness, never
  write sets). And two preps in one fill could widen two rows onto one file
  at once.
- **The full form stays** for what a row cannot carry: a row sent again with
  a verifier's finding in its brief, a `report` schema, `verify: false`. A
  lane gives either the short form or the full one, never both.
- **Steps 4 and 5 of `dispatch-lanes` shrink.** Take the rows `queue next`
  gives, send each as `{ label, queueId }`, and add `notes` only for what
  the row's text does not say. Step 4's rules move into `prep.md` rather
  than being repeated.

**2026-10-04, Q837:** §399 adds a second mark beside `pinsOnly`,
`sameBehaviour`, for a row that changes lines that run and keeps what they
compute. Prep answers it too, said explicitly and never inferred, for the same
reason, and never both marks on one row.

**2026-10-11, Q840:** built, as this section says, with these details:

- **`.claude/agents/prep.md`**: a Sonnet type with `Read`, `Grep`, `Glob`
  and `Bash`, and no `Edit` or `Write`. It holds the rules of the list above,
  `sameBehaviour` beside `pinsOnly`, and the facts rules that were step 5's:
  `claim-section` before a section, how far two fuzz runs drift, and nothing
  about the context hook for a writing row. It reads the code at the run's
  base with `git show` and `git grep`, because it runs in the dispatcher's
  tree, whose `HEAD` carries the `take`. A bundle goes to one prep.
- **The short form in `lanes.js`.** A lane with none of the full form's
  fields is sent to prep first. Prep copies each row's id, model and write
  set as `just queue show` prints them, since `lanes.js` cannot read the
  queue file. Its `ok` is then held to that copy. The ids must be the ones
  sent. The write set must be the union of the rows' sets, never wider or
  narrower. The model must be `opus` if any row says so, else `sonnet`. The
  answer must not carry both marks. Anything else stops the row before any
  lane starts: `needs`, an answer that fails a check, or a prep that throws.
  The row comes back not merged, with a `needs <file>: <why>` line for each
  file prep named.
- **What a short-form brief carries**: the rows' lines, the mark and why,
  prep's facts and the dispatcher's `notes`. It ends with the two notes every
  brief carried word for word, one on `tag` steps and a row's last word, one
  on bare test names. A writing short-form lane is also asked for a
  `for_the_verifier` field by default, saying how to lay back the
  `#[cfg(test)]` pins it adds in `src/`. A full-form brief gets none of the
  three; its dispatcher still types them.
- **Refused at the run's start**: a lane with no label, two lanes with one
  label, a lane giving `notes` beside a full-form field, and a short-form lane
  with no `queueId`.
- **`check-lanes-workflow`** gains arms 23 to 27 and sixteen cuts. Prep is
  sent with its type, and `prep.md` makes it a Sonnet agent with no tool that
  writes. It runs before the lane. A `needs` starts no lane and comes back. A short-form lane takes
  its write set, model and marks from prep's checked answer. Both forms are
  refused. The two notes and the default field are on a short-form brief.
  Each arm fails with the base's `lanes.js`.
- **`dispatch-lanes` shrinks.** Step 4 keeps the bundle line and points to
  `prep.md`. Step 5 sends `{ label, queueId }`, and keeps the full form for a
  row sent again with a finding, a `report` schema, or `verify: false`.

Nobody has run prep yet. A `claude` process reads its agent types when it
starts, and the session that built it began before `prep.md` existed. The
first short-form run must come from a dispatcher restarted after this lands.
Until then the full form works as before.

### 396.5 What it should save, what it costs, and the test

**Saved.** Before the first lane, the two shares should fall from 9 642
output tokens and 28 305 result characters to the `Workflow` call with ids
and notes, plus a `needs` return about one fill in five. Q818 removes the
`lanes.js` reads on its own, so a comparison must count that once and not
credit it to both.

**Cost.** One Sonnet agent per row, and its minutes before the lane starts.
A lookup lane climbs from about 19 000 to 53 000 tokens to answer one
question (§158.5). A prep is the same shape, and its price for this job is
not measured. Its reads are billed at Sonnet's rate in a short context. The
dispatcher's were billed at Opus's rate at over 100 000 tokens of context,
and then read again on every later call of a session that runs for hours.

**Test.** Run Q819's `just spend start` over the first ten dispatcher
segments after the row lands. If "write the briefs" and "check a row
against the code" together have not fallen by half, add a dated note here
that says so, and why.

**2026-10-26, Q1700:** Yes for three of the four numbers; the fourth fell by
only a third, and the gap is in how the counting sorts calls, not in what
prep does.

By today, 135 of the 139 dispatcher segments that sent a lane since Q840
landed (2026-10-11T12:26:45Z) had sent at least one short-form lane through
prep. The ten earliest, by when each first did so, are sessions `aec37d77`,
`8a5234d9`, `42a1f000`, `70df2354`, `61c90cad`, `599295de`, `abdb89fa`,
`9500e8ec`, `a6e49354` and `a01f9d4d`, each ended within a day of Q840
landing. `just spend`'s `lane` argument takes one substring, not a list, so
no single `just spend start <since> <until>` call names exactly these ten;
the comparison instead called `spend.load()` and `spend.report_start()` —
the functions the recipe itself runs — on the window's responses filtered to
those ten session ids, the same counting §396.1 and Q819 used, over a chosen
set of sessions rather than a chosen date range. A follow-up row gives the
recipe a session-id list so this does not need a scratch script next time.

Before the first lane: write the briefs fell from 5 183 to 2 005 output
tokens and from 15 642 to 6 606 result characters; check a row against the
code fell from 4 459 to 986 tokens and from 12 663 to 6 009 characters.
Together, 9 642 to 2 991 tokens (a fall of 69%) and 28 305 to 12 615
characters (55%) — both past half.

Over the whole segment: write the briefs fell from 10 247 to 6 662 tokens
and from 21 061 to 17 966 characters; check a row from 8 602 to 2 650 tokens
and from 25 561 to 13 720 characters. Together, 18 849 to 9 312 tokens
(51%, at half) but only 46 622 to 31 686 characters (32%, short of half).

The number that did not halve is result characters, check a row against the
code, over the whole segment. `purpose_of` sorts every `just decision`, `just
decisions-find`, `just src` and every read under `crates/` or
`docs/decisions.d/` into that bucket by the file or command alone, whether it
is the per-fill write-set check prep now owns or something else the
dispatcher still does by hand later in the same segment: deciding a returned
lane, filing a new row, chasing a fuzz finding, reading a section to write a
brief's facts for an exception row. Of the 13 720 characters the bucket holds
over a whole segment in the new sample, 7 711 (56%) land after the first lane
is sent — in this sample, a 14 467-character `git show` of an old decisions
file for session `9500e8ec`'s dated note, and 8 482- and 5 629-character runs
of `just decision` reading sections 352.3, 360.5, 418.3 and 437.1-437.9 for
session `a6e49354`. None of that moved to prep, because prep runs once per
fill, before a lane starts; the rest of a long dispatcher segment goes on
reading the record exactly as it did before Q840.

### 396.6 What was not chosen

- **The lane checks its own write set and stops if it is wrong.** The
  dispatcher would still write every brief. And each stop costs a lane's
  start plus a returned run the dispatcher has to decide, about one fill in
  five.
- **One Sonnet prep lane per fill that hands the args back.** The args would
  come back through the dispatcher's context and go out again as the
  `Workflow` call's input, 5 352 characters at the median.
- **The session that files a row checks it then.** It mostly does already: a
  lane files its own rows (§384). But a write set goes stale between filing
  and taking, and the 28 fixes in §396.3 are what was missed at filing or
  what moved since.
- **A lower reasoning effort**: §396.7.

### 396.7 The effort level is Mattias's, and nothing here measures it

Every one of the 69 segments ran at `xhigh`. With no segment at another level
to compare against, nothing here can say what a lower level would save, or
what it would cost in rows checked badly. The level is a setting, not a
change to this repository. One dispatcher session at `high`, read with Q819's
report against the medians in §396.1, is the measurement that would answer it.
