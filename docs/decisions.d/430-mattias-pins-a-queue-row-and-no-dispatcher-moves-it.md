## 430. Mattias pins a queue row, and no dispatcher moves it — 2026-10-11

*Asked for by Mattias on 2026-10-11: "add the ability for me to override the
priority of issues. I.e. not allowing the dispatcher to re-prioritize them",
and then to put Q813, Q817, Q818 and Q840 first in tier 1.*

### 430.1 Why a row needs a mark

On 2026-10-08, 47a96c89 moved Q1310, Q1312 and Q1349 to the head of tier 1 at
Mattias's word. 38 minutes later a dispatcher in another tree read §307.1,
found them misfiled by it, and moved them back to tier 2.2 (0bab4895). They
were put back in ea512ca9. The reason lived only in a commit message: a row had
nowhere to say "placed here on purpose".

Left in place, the head of tier 1 was still not "next". Tier 1 goes in class
order (§352.3), so a row with no class there is taken after every ready row
that has one. So the fill needs an order above tier and class, and the verbs
need to know which rows are in it.

### 430.2 The mark is the first word of the description

`pinned/<n>`, n a number from 1, goes in front of the description, and a class
or `small` may follow it (§352.2). It goes in the row and not in a list kept
beside the queue, so the row keeps its class and `small`, and the one parse
that every reader shares (`_queue-rows`, §230.2) carries it, with no second
file to keep in step.

**The order is n, not the line.** The merge driver keeps one side's line order
within a block, so a merge can undo an order the lines carry. That is why
tiers 1 and 5 go by class and not by line (§352.3), and it holds here too. It
also means `rank` cannot disturb the pinned order, so ranking a row that is not
pinned above one that is goes through.

### 430.3 The fill

`queue next` takes the ready pinned rows before anything else, in the order of
n. That includes the full bundle of small rows, which otherwise takes the first
slot (§352.4). Then it walks the tiers and classes as before. A pinned row that
a running lane blocks is skipped with the usual line, and since §430.7 the rows
below it stay out of its write set. A pinned row that waits
on another is not ready, so a chain such as Q813 and Q817, then Q818, then Q840
goes out one row at a time as each lands. `ready` and the plain listing show
the same order.

### 430.4 What the verbs do with a pinned row

- `pin <id> [<id>...]` puts the mark on each open row in the order given, n one
  past the highest pin an open row carries. It moves the row to tier 1 if it is
  elsewhere, and puts it just after the pinned rows there, so the file reads in
  the order the fill takes. It refuses a closed row; a parked row, which the
  fill never takes; a row already pinned, which is unpinned first to re-order;
  and a row whose class is C or D, which names tier 5 (§403).
- `unpin <id> [<id>...]` takes the mark off, and the row stays where it is in
  tier 1.
- `take`, `done` and `pick` work on a pinned row as on any other. A closed row
  keeps its mark, which is the record that it was pinned.
- `drop`, `park`, `after`, `rank` of the pinned row itself, `tier` and `tag`
  refuse an open pinned row. The refusal says Mattias placed it and names the
  three ways on: take it, close it with `done`, or ask him to `unpin` it.
  *§430.8, 2026-10-20: `tag` sets and takes off `small` on a pinned row with no
  class, which moves it nowhere; a class it still refuses.*
- `add` refuses a description that starts with `pinned/`.

`check-queue` holds the shape. A pin is `pinned/` and a number from 1, an open
pinned row is tier 1, no two open rows share a number, and the class checks
read the word after a pin. A pinned row sits in tier 1 whatever §307.1 says its
kind would be; the mark is what says it is there on purpose.

### 430.5 Only Mattias pins

The queue write guard (§263) answers `ask` for any Bash command that runs `pin`
or `unpin`. The verb is the word after `queue`, or after `queue.py`, and in an
`apply` batch every word after a `+` as well, inside `bash -c` too. Claude
Code's permission-modes page says a hook's ask is shown to the user even in
auto mode ("auto mode still shows you those prompts"), so a dispatcher that
tries is put in front of Mattias, who can say yes. He can also run the verbs
himself as a `!` command, which no hook sees. `just land` lets a lane run only
`add`, `after` and `tag` (`STEP_ARGS` in `tooling/land/land.py`), so no lane
pins through it. `just verify-lane` runs a lane's number commands through
`bash -c`, where no hook sees them, so it counts `pin` and `unpin` among the
queue verbs that write and runs neither.

What the guard does not see: a command built while it runs, such as Python
calling `subprocess` with a list it computes. The write guard has the same
limit. Both are there to stop an accident, not a session set on getting round
them. What an ask does in a background subagent the docs do not say, and it is
not measured here: no lane has a reason to run these verbs.

*Seen on 2026-10-11, at the first real pin.* A top-level session in auto mode
ran `mise exec -- just queue apply pin Q813 Q817 Q818 Q840`. Its transcript
holds a `hook_success` record for `PreToolUse:Bash` on that call at 09:26:12Z,
whose output is `permissionDecision: ask` with this section's reason. The
call's result came back at 09:40:14Z, 14 minutes later, with the four rows
pinned 1 to 4. A command a hook asks about runs only after a yes, so the
prompt reached Mattias and he said yes.

### 430.6 Decided while building it

The plan put a newly pinned row before the first tier 1 row that is not
pinned. It goes after the last pinned row instead. The two agree when the file
is in order. When a row has been ranked above the pins, which §430.2 lets
through, the first rule would put the new pin above that row and apart from
the other pins.

### 430.7 A skipped pin keeps the rows below it out of its write set

*Asked for by Q2328, 2026-10-19.* §430.3 skipped a pinned row that a running
lane blocks and gave the slot to the next row that fit, with no regard for
what that row held. A pin with a wide write set then waited for ever. Q2328's
text tells it of Q729, pinned/1 on all of `crates/rcad-topo/tests/`: on
2026-10-18 it was skipped four fills running, each for a lane on one test file
there, and each fill took another row on a file there (Q2129, Q2316, Q2198,
then Q2317). The directory was never clear when a slot came free.

**The rule.** When `queue next` skips a pinned row, every row it reaches after
that row and whose write set meets the pinned row's is skipped too, with
`skip <id> (tier <t>): its <entry> meets <pin>'s <entry>, pinned and skipped
above`. No new lane takes a file the pin needs, so the lanes in its way drain
and the pin is taken by the first fill after the last of them lands. Rows that
meet no skipped pin are taken as before. The fixture `pinwait` in
`check-queue-selftest` is Q729's case in small, and on the base it took a later
pin, a full bundle, a wrong/A row and a shared-file row, each on a file the pin
names.

What was decided while building it:

- **Every row after it, a later pin included.** A later pin that meets an
  earlier skipped one waits for it, since Mattias's n says which goes first. So
  does each small row of the full bundle, which is chosen after the pins.
- **A skipped pin counts as a holder of a `queue_shared_files` file** (§424.4).
  Two holders may share such a file, so one row may still take it beside the
  pin, which then comes in as the second; the next row on the file is skipped,
  naming that row and the pin. A row taken above counts the same way, and the
  line has the same shape.
- **A pin skipped for any reason holds**, a running lane, a row held elsewhere,
  a row taken above it or a pin skipped above it. Each of these lasts until a
  lane lands, so any row taken meanwhile into the pin's files only adds one more
  lane to wait for.
- **A pin that waits holds nothing.** It is not ready, so the fill does not
  reach it, and a chain of pins (§430.3) does not freeze the files of the later
  links while the first runs.
- **`blocked` records the pin too**, since Q2394. Q2328 left it naming the
  rows a running lane holds back alone, because the queue page asserts each
  line it reads has one of the shapes in `tooling/queue-page/render.py`, which
  lay outside Q2328's write set. Q2394 changed both together: `blocked` takes
  a pinned row a running lane or a row held elsewhere holds back as a holder,
  and prints the `pinned and skipped above` line for each later ready row that
  meets it, in the fill order, and the page reads that line (`PINNED_LINE`)
  as a row the fill will not take.

The cost is slots left empty while the lanes in the pin's way drain, where
before they went to rows the pin then waited behind. That is what a pin asks
for; `unpin` is the way out when it is not.

### 430.8 A pinned row can be tagged small

*Mattias, 2026-10-20: "If I pin a row, that row will always execute in it's
own lane, even if it is very small. That seems stupid. Many of the tier 2.2
fixes are small, but if I would like to prioritise some of them they will
always cost a full lane which they should not."*

**What the fill did already.** A pin whose next word is `small` goes out as a
bundle. `take_at` in the `queue` recipe takes it with the next ready small
rows in the walk, up to `queue_bundle`, and the walk puts the later pins
first. Three pins went out that way, each tagged `small` before it was
pinned: Q1931 (pinned/2) in one lane with Q1918, Q1925, Q1927 and Q1942
(taken in 704063f0), Q1946 (pinned/4) with Q1945 (0fb5fb99), and Q2395
(pinned/5) with Q2467 (fe0319c4). The six other rows pinned beside Q2395 in
98249cbd had no `small`, and each was taken into a slot of its own.

**What was missing.** `small` had to be on the row before the pin. §430.4 had
`tag` refuse an open pinned row, so a row found to be small once it was pinned
kept a lane to itself, whoever found it. And few rows carry the mark: on
2026-10-20, 1 of the 75 open rows in tier 2.2 did, and 4 of the 69 in tier 2.

**The rule.** `tag <id> small` and `tag <id> -` work on an open pinned row that
has no class. The word goes after the pin, `pinned/<n> small`, which is where
the fill and `check-queue` already read it. The pin keeps its number and its
line, so the row is taken when it would have been; what changes is that up to
four ready small rows go in its lane. A class is still refused on a pinned row,
set or replaced: a class says how far up the fill a row goes, and the pin is
Mattias's answer to that. A defect is never small (§352.2), so a pinned row
with a class has no reason to take `small` either.

**The cost.** The pin lands with the rows that rode with it, so later than it
would alone: §352.4 sizes a full bundle at about 45 minutes. Pins come first in
the walk, so a set of small pins goes out together before any small row that
is not pinned joins them.

**To use it.** Tag before the pin or after it, in one batch or two: `just queue
apply pin Q1 Q2 + tag Q1 small + tag Q2 small`. A session that pins at
Mattias's word, or a dispatcher about to take a pinned row alone, tags it
`small` when it is small by §352.4: it needs no trace, its place is known, and
the work takes about ten minutes.

The test is the pin arm of `check-queue-api-selftest`. `tag small` and `tag -`
on a pinned row go through and leave the pin and its place, and `tag small` on
a pinned `wrong/A` row is refused. On the base the first of these was refused.
