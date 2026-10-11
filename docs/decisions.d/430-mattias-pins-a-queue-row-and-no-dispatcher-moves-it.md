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
a running lane blocks is skipped with the usual line. A pinned row that waits
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

### 430.6 Decided while building it

The plan put a newly pinned row before the first tier 1 row that is not
pinned. It goes after the last pinned row instead. The two agree when the file
is in order. When a row has been ranked above the pins, which §430.2 lets
through, the first rule would put the new pin above that row and apart from
the other pins.
