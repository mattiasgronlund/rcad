#!/usr/bin/env python3
"""One row of docs/queue.md changed at a time, checked before it is written.

`just queue <verb>` runs this. Every verb takes the lock on the claims file in
the git common directory, reads this worktree's copy of the queue, makes one
change to one row, runs `just check-queue` on the result, and writes it only if
that passes. `docs/decisions.md` §263 is the argument; §159 is where the claims
file and its lock come from.

The claims file is shared by every worktree of this clone and carried by no
branch. It is a log, one line per event, tab-separated:

    add      <id>  <time>  <worktree>  <row>
    take     <id>  <time>  <worktree>  <branch>  <write set>  <tier>
    release  <id>  <time>  <worktree>  <how>

A row is held from its `take` until the `release` that `done`, `drop` or `park`
writes. The tier is the row's when it was taken, which the file cannot say once
`tier` or `tag` has moved the held row; a `take` line of five fields after its
verb records none. Two worktrees taking one row is refused at the second `take`, before
either has committed the file, because the hold is written under the lock.

`just queue apply <verb> <args...> + <verb> <args...> ...` runs several write
verbs as one batch: every step lands on an in-memory copy of the queue, in
order, each seeing the ones before it, and only once every step has succeeded
is `check-queue` run, are the claims events appended, and is the file written
-- once, not once per step (§309). A step whose verb refuses aborts the whole
batch, named by its position and its own text, before anything is written.
`@1`, `@2`, ... stand for the ids the batch's first, second, ... `add` hands
out, since an `after` step often needs to name a row an `add` just added.
"""

import fcntl
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rows import CLASS, SMALL, TIERS_IN_ORDER, Queue, highest_q, tier_of_class  # noqa: E402

ID = re.compile(r"^[TDQ][0-9]+$")
BRANCH = re.compile(r"^[A-Za-z0-9._/-]+$")
SHA = re.compile(r"^[0-9a-f]{7,40}$")
SECTION = re.compile(r"^§?([0-9]+(\.[0-9]+)?)$")
AT_ID = re.compile(r"^@([0-9]+)$")

USAGE = """\
just queue                                 ready rows by tier, then running, waiting, parked
just queue ready                           the ready rows alone
just queue blocked                         each ready row whose write set meets a running lane's, as next skips it
just queue next [N]                        what N free slots take; N defaults to the free slots on this machine
just queue running                         what holds a slot on this machine, one line each
just queue show <id>                       one row, its hold, and its state in other worktrees
just queue take <id> <branch>              dispatch a ready row to <branch>: state lane:<branch>, held until done/drop/park
just queue add <tier> <model> <write set> <what ending in §N>    a new todo row at the foot of its tier; prints the id
just queue tier <id> <tier>                move an open row to the foot of another tier, writing that tier's heading if missing;
                                            a row with a class goes only to the tier its class names
just queue tag <id> <class>|small|-        tag an open row: a class (wrong, crash or refuses, then /A to /D) ranks a tier 1
                                            or tier 5 row and moves it to tier 1 for A and B, tier 5 for C and D; small lets
                                            a lane take it with other small rows; - removes it
just queue done <id> <merge sha>           state done:<sha>, moved under ## Closed
just queue drop <id> <§N>                  state dropped:§N, moved under ## Closed
just queue park <id> <§N>                  state parked:§N, left in its tier
just queue after <id> <id>[,<id>...]       state after:<ids>; each id must exist in some copy, branch or claim
just queue rank <id> before <id>           move <id> to sit just before the other id, within their shared tier
just queue pick <id> ours|theirs           keep one side of a row the merge driver left in conflict
just queue apply <verb> <args> [+ <verb> <args>]...   run several of the verbs above as one all-or-nothing
                                            batch, under one hold of the claims lock; @1, @2, ... are the
                                            ids the batch's own adds hand out"""


class Refused(Exception):
    pass


def main(argv):
    if not argv:
        print(USAGE, file=sys.stderr)
        return 2
    verb, args = argv[0], argv[1:]
    here = Here(os.environ.get("QUEUE_FILE", "docs/queue.md"))
    readers = {"show": show, "running": running, "elsewhere": elsewhere}
    if verb in readers:
        readers[verb](here, args)
        return 0
    # A plain verb is a batch of one step, run through `apply` itself (§309.3.5):
    # one path writes the file, whether the call names one verb or several.
    if verb != "apply" and verb not in STEP_FUNCS:
        print(f"queue: '{verb}' is not a verb\n{USAGE}", file=sys.stderr)
        return 2
    try:
        apply(here, args if verb == "apply" else [verb, *args])
    except Refused as refusal:
        print(f"queue {verb}: {refusal}", file=sys.stderr)
        return 1
    return 0


class Here:
    """This worktree, its copy of the queue, and the clone it belongs to."""

    def __init__(self, queue_file):
        self.top = git("rev-parse", "--show-toplevel").strip()
        self.common = git("rev-parse", "--path-format=absolute", "--git-common-dir").strip()
        self.file = os.path.abspath(queue_file)
        self.relative = os.path.relpath(self.file, self.top)
        self.claims = os.environ.get("QUEUE_CLAIMS") or os.path.join(self.common, "rcad-queue-claims")

    def read(self):
        if not os.path.isfile(self.file):
            raise Refused(f"{self.relative}: no such file")
        with open(self.file, encoding="utf-8") as f:
            return Queue(f.read())

    def trees(self):
        return worktrees()

    def other_copies(self, lanes_too):
        """Every other worktree's copy of the queue, committed or not.

        Only a dispatcher writes the file (§230.2), so a lane's copy is its
        base's copy and says nothing a dispatcher's does not. `lanes_too` is for
        the questions where a stale copy cannot mislead: whether an id exists.
        """
        for tree in self.trees():
            if tree.path == self.top:
                continue
            if not lanes_too and tree.is_lane:
                continue
            path = os.path.join(tree.path, self.relative)
            if os.path.isfile(path):
                with open(path, encoding="utf-8", errors="replace") as f:
                    yield tree, f.read()


def git(*args, cwd=None, check=True, stdin=None):
    done_ = subprocess.run(["git", *args], cwd=cwd, input=stdin, capture_output=True, text=True)
    if check and done_.returncode != 0:
        raise Refused(f"git {' '.join(args)} failed: {done_.stderr.strip()}")
    return done_.stdout


class Tree:
    def __init__(self, path):
        self.path = path
        self.branch = "(detached)"
        self.head = None
        self.locked = None
        self.is_lane = False


def worktrees():
    """`git worktree list`, with every agent tree a session did not sit in marked as a lane.

    The worktree hook puts a session's own tree in `<hub>-worktrees/<name>` and a
    subagent's in `<session tree>-worktrees/<name>`, and locks both with
    `agent <name> since <time>` (§55). So a lane is a tree locked by an agent
    that is not directly under the hub's directory.
    """
    trees = []
    for line in git("worktree", "list", "--porcelain").splitlines():
        if line.startswith("worktree "):
            trees.append(Tree(line[len("worktree "):]))
        elif line.startswith("HEAD "):
            trees[-1].head = line[len("HEAD "):]
        elif line.startswith("branch refs/heads/"):
            trees[-1].branch = line[len("branch refs/heads/"):]
        elif line == "locked":
            trees[-1].locked = ""
        elif line.startswith("locked "):
            trees[-1].locked = line[len("locked "):]
    assert trees, "git worktree list printed no tree, and a repository always has one"
    sessions = trees[0].path + "-worktrees"
    for tree in trees[1:]:
        agent = tree.locked is not None and tree.locked.startswith("agent ")
        tree.is_lane = agent and os.path.dirname(tree.path) != sessions
    return trees


class Claims:
    """The claims file, read and appended under its lock."""

    def __init__(self, path):
        self.path = path

    def __enter__(self):
        self.lock = open(self.path + ".lock", "a")
        fcntl.flock(self.lock, fcntl.LOCK_EX)
        return self

    def __exit__(self, *_):
        self.lock.close()

    def events(self):
        if not os.path.exists(self.path):
            return []
        with open(self.path, encoding="utf-8") as f:
            return [line.rstrip("\n").split("\t") for line in f if line.strip()]

    def append(self, *fields):
        assert all("\t" not in field and "\n" not in field for field in fields), fields
        with open(self.path, "a", encoding="utf-8") as f:
            f.write("\t".join(fields) + "\n")

    def holds(self):
        held = {}
        for event in self.events():
            if event[0] == "take":
                held[event[1]] = {"time": event[2], "tree": event[3], "branch": event[4], "write_set": event[5],
                                  "tier": event[6] if len(event) > 6 else None}
            elif event[0] == "release":
                held.pop(event[1], None)
        return held

    def added(self):
        return {event[1]: event for event in self.events() if event[0] == "add"}


def now():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


class Batch:
    """The ids this batch's own `add` steps have handed out, for `@1`, `@2`, ... to resolve against."""

    def __init__(self):
        self.added_ids = []


def apply(here, args):
    steps = split_steps(args)
    with Claims(here.claims) as claims:
        queue = here.read()
        batch = Batch()
        messages = []
        events = []
        for number, (verb, raw_args) in enumerate(steps, start=1):
            step_args = substitute_ids(raw_args, batch.added_ids)
            try:
                queue, message, event = STEP_FUNCS[verb](queue, claims, here, step_args, batch)
            except Refused as refusal:
                if len(steps) == 1:
                    raise
                text = " ".join([verb, *raw_args])
                raise Refused(f"step {number} ({text}) is refused: {refusal}")
            messages.append(message)
            if event is not None:
                events.append(event)
        checked(here, queue)
        for event in events:
            claims.append(*event)
        write(here, queue)
    for message in messages:
        print(message)


def split_steps(args):
    """`args` cut on a standalone `+`; each piece must start with a verb `apply` knows."""
    steps, current = [], []
    for token in args:
        if token == "+":
            steps.append(current)
            current = []
        else:
            current.append(token)
    steps.append(current)
    if len(steps) == 1 and not steps[0]:
        raise Refused("usage: apply <verb> <args...> [+ <verb> <args...> ...]")
    parsed = []
    for step in steps:
        if not step:
            raise Refused("a step between two '+'s is empty")
        verb, step_args = step[0], step[1:]
        if verb not in STEP_FUNCS:
            raise Refused(f"'{verb}' is not a verb apply knows: {', '.join(STEP_FUNCS)}")
        parsed.append((verb, step_args))
    return parsed


def substitute_ids(args, added_ids):
    """`@1`, `@2`, ... replaced by the id the batch's first, second, ... `add` handed out."""
    def one(part):
        match = AT_ID.match(part)
        if not match:
            return part
        n = int(match.group(1))
        if n < 1 or n > len(added_ids):
            raise Refused(f"'{part}' names the batch's {n}th add, and only {len(added_ids)} have run so far")
        return added_ids[n - 1]
    return [",".join(one(part) for part in token.split(",")) for token in args]


def require_row(queue, here, row_id):
    if not ID.match(row_id):
        raise Refused(f"'{row_id}' is not an id: T, D or Q and then a number")
    row = queue.find(row_id)
    if row is None:
        raise Refused(f"{row_id} is not a row in {here.relative}")
    return row


def take_step(queue, claims, here, args, batch):
    need(args, 2, "take <id> <branch>")
    row_id, branch = args
    if not BRANCH.match(branch):
        raise Refused(f"'{branch}' cannot be a branch in lane:<branch>; letters, digits and ._/- only")
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    hold = claims.holds().get(row_id)
    if hold and hold["tree"] != here.top:
        raise Refused(f"{row_id} is held by {hold['branch']} from {hold['tree']} since {hold['time']}")
    history = None
    for tree, text in here.other_copies(lanes_too=False):
        theirs = Queue(text).find(row_id) if row_id in text else None
        if theirs and theirs.state.startswith("lane:") and theirs.state != row.state:
            if history is None:
                history = row_states_in_main_history(here, row_id)
            if theirs.state not in history:
                raise Refused(
                    f"{row_id} is {theirs.state} in {tree.path}'s copy, a branch main has never shown "
                    f"for it, so this looks like a take racing this one"
                )
    # A lane: row this worktree holds, or dispatched by hand here, may be
    # taken again for a new branch: that is a failed lane re-dispatched.
    if not row.state.startswith("lane:"):
        refuse_unless_ready(queue, row)
    queue.replace(row, row.with_state(f"lane:{branch}"))
    message = f"{row_id}: {row.state} -> lane:{branch}, held for {branch} until done, drop or park"
    return queue, message, ("take", row_id, now(), here.top, branch, row.write_set, row.tier)


def refuse_unless_ready(queue, row):
    if row.state == "todo":
        return
    if row.state.startswith("after:"):
        waiting = []
        for other in row.state[len("after:"):].split(","):
            found = queue.find(other)
            if found is None or not found.state.startswith("done:"):
                waiting.append(f"{other} ({found.state if found else 'not in this copy'})")
        if waiting:
            raise Refused(f"{row.id} is not ready: it waits on {', '.join(waiting)}")
        return
    raise Refused(f"{row.id} is not ready: its state is {row.state}")


def main_ref():
    """`origin/main` when this clone has one, else `main`, or `None` with neither."""
    for candidate in ("origin/main", "main"):
        found = subprocess.run(["git", "rev-parse", "--verify", "--quiet", candidate], capture_output=True)
        if found.returncode == 0:
            return candidate
    return None


def row_states_in_main_history(here, row_id):
    """Every state `row_id` has held across main's own history of this queue file (§230.2).

    A `lane:<branch>` another copy still shows for this row is a stale copy
    catching up, not a racing take, when main's history already carried that
    exact state for this row at some commit — `take_step` lets the take go
    then, and refuses only a branch main has never shown (Q1436).
    """
    ref = main_ref()
    if ref is None:
        return set()
    shas = git("log", "--format=%H", ref, "--", here.relative, cwd=here.top).split()
    states = set()
    for sha in shas:
        text = git("show", f"{sha}:{here.relative}", cwd=here.top, check=False)
        if row_id in text:
            found = Queue(text).find(row_id)
            if found:
                states.add(found.state)
    return states


def add_step(queue, claims, here, args, batch):
    if len(args) < 4:
        raise Refused("usage: add <tier> <model> <write set> <what, ending in §N>")
    tier, model, write_set = args[:3]
    what = " ".join(args[3:])
    refuse_unless_a_tier(tier)
    refuse_class_with_punctuation(what)
    refuse_conflicted(here, queue)
    block = queue.block_for_tier(tier)
    if block is None:
        raise Refused(f"there is no '## Tier {tier}' block, and no later heading to write one above")
    top = highest_q(queue.text())
    for _, text in here.other_copies(lanes_too=True):
        top = max(top, highest_q(text))
    for text in branch_copies(here.relative):
        top = max(top, highest_q(text))
    for claimed in claims.added():
        top = max(top, int(claimed[1:]))
    row_id = f"Q{top + 1}"
    text = f"{row_id:<4} {tier:<4} todo    {model:<7} {write_set}   {what}"
    queue.append_to_block(block, text)
    batch.added_ids.append(row_id)
    return queue, row_id, ("add", row_id, now(), here.top, text)


CLASS_PREFIX = re.compile(CLASS.pattern[:-1])


def refuse_class_with_punctuation(what):
    """Refuse a description whose first word is a class (§352) with
    punctuation stuck on after it, such as `refuses/C:` or `wrong/B,`.

    `Row.tag` only ever reads a first word that matches `CLASS` exactly, so a
    first word like `refuses/C:` goes in with no tag at all: Q1305's lane
    wrote two rows that way, check-queue refused them, and `just land` stopped
    at its queue apply (§352.3). A first word that is exactly a class, with
    nothing after it, is not refused: it tags the row, as today.
    """
    first = what.split(None, 1)[0] if what else ""
    if CLASS.match(first):
        return
    stuck = CLASS_PREFIX.match(first)
    if stuck and not re.search(r"[A-Za-z0-9]", first[stuck.end():]):
        raise Refused(
            f"'{first}' is a class with punctuation after it; write exactly '{stuck.group(0)}', with nothing after it"
        )


def branch_copies(relative):
    """The queue on every local and remote-tracking branch, each distinct blob once.

    Two `git cat-file --batch` calls rather than one `git show` per ref: there
    are about two hundred refs and most carry the same blob.
    """
    refs = git("for-each-ref", "--format=%(refname)", "refs/heads", "refs/remotes").split()
    if not refs:
        return []
    checked_refs = git("cat-file", "--batch-check=%(objectname) %(objecttype)",
                       stdin="".join(f"{ref}:{relative}\n" for ref in refs))
    blobs = sorted({line.split()[0] for line in checked_refs.splitlines() if line.endswith(" blob")})
    if not blobs:
        return []
    raw = subprocess.run(["git", "cat-file", "--batch"], input=("\n".join(blobs) + "\n").encode(),
                         capture_output=True, check=True).stdout
    texts = []
    position = 0
    for _ in blobs:
        header_end = raw.index(b"\n", position)
        size = int(raw[position:header_end].split()[2])
        start = header_end + 1
        texts.append(raw[start:start + size].decode("utf-8", errors="replace"))
        position = start + size + 1
    return texts


def done_step(queue, claims, here, args, batch):
    need(args, 2, "done <id> <merge sha>")
    row_id, sha = args
    if not SHA.match(sha):
        raise Refused(f"'{sha}' is not a sha: 7 to 40 lowercase hex digits")
    if subprocess.run(["git", "rev-parse", "--verify", "--quiet", f"{sha}^{{commit}}"],
                      capture_output=True).returncode != 0:
        raise Refused(f"{sha} is not a commit in this repository")
    return close_step(queue, here, row_id, f"done:{sha}", "done")


def drop_step(queue, claims, here, args, batch):
    need(args, 2, "drop <id> <§N>")
    return close_step(queue, here, args[0], f"dropped:{section(args[1])}", "drop")


def close_step(queue, here, row_id, state, how):
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    if row.is_closed():
        raise Refused(f"{row_id} is closed already: {row.state}")
    closed = queue.block_of_key("closed")
    if closed is None:
        raise Refused("there is no '## Closed' block to move the row under")
    queue.remove(row)
    queue.append_to_block(closed, row.with_state(state))
    message = f"{row_id}: {row.state} -> {state}, moved under ## Closed"
    return queue, message, ("release", row_id, now(), here.top, how)


def park_step(queue, claims, here, args, batch):
    need(args, 2, "park <id> <§N>")
    row_id, state = args[0], f"parked:{section(args[1])}"
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    if row.is_closed():
        raise Refused(f"{row_id} is closed: {row.state}")
    queue.replace(row, row.with_state(state))
    message = f"{row_id}: {row.state} -> {state}"
    return queue, message, ("release", row_id, now(), here.top, "park")


def after_step(queue, claims, here, args, batch):
    if len(args) < 2:
        raise Refused("usage: after <id> <id>[,<id>...]")
    row_id = args[0]
    waits = [w for arg in args[1:] for w in arg.split(",") if w]
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    if row.is_closed() or row.state.startswith("lane:"):
        raise Refused(f"{row_id} is {row.state}, and only a row nobody holds can wait")
    for other in waits:
        if not ID.match(other):
            raise Refused(f"'{other}' is not an id")
        if other == row_id:
            raise Refused(f"{row_id} cannot wait on itself")
        if queue.find(other) is None and not exists_elsewhere(here, claims, other):
            raise Refused(f"{other} exists nowhere: not in this copy, not in any worktree's copy, "
                          f"not on any branch, and never handed out by `just queue add`")
    state = "after:" + ",".join(waits)
    queue.replace(row, row.with_state(state))
    message = f"{row_id}: {row.state} -> {state}"
    return queue, message, None


def exists_elsewhere(here, claims, row_id):
    if row_id in claims.added():
        return True
    pattern = re.compile(rf"^\s*{re.escape(row_id)}\s", re.M)
    if any(pattern.search(text) for _, text in here.other_copies(lanes_too=True)):
        return True
    return any(pattern.search(text) for text in branch_copies(here.relative))


def rank_step(queue, claims, here, args, batch):
    """Move a row to sit just before another row in the same tier (§263.9).

    `add` always appends at its tier's foot, and the write guard refuses
    reordering rows by hand, so this is the only way to promote a row within
    its tier. It never changes a row's tier field, only its place in the file.
    """
    need(args, 3, "rank <id> before <id>")
    row_id, keyword, other_id = args
    if keyword != "before":
        raise Refused("usage: rank <id> before <id>")
    if row_id == other_id:
        raise Refused(f"{row_id} cannot rank before itself")
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    other = queue.find(other_id)
    if other is None:
        raise Refused(f"{other_id} is not a row in {here.relative}")
    if row.is_closed() or other.is_closed():
        raise Refused("a closed row sits under ## Closed and is never ranked")
    if row.block != other.block:
        raise Refused(f"{row_id} is tier {row.tier}, {other_id} is tier {other.tier}; "
                      f"rank only moves a row within its own tier")
    queue.move_before(row, other)
    message = f"{row_id}: ranked before {other_id}"
    return queue, message, None


def tier_step(queue, claims, here, args, batch):
    """Move an open row to the foot of another tier, writing that tier's heading if the file has none (§307.5).

    The write guard refuses the hand edit, and a heading a hand edit would have
    added is refused with it, so this is how a row reaches a tier nobody has
    filed in yet. The row keeps its state and its hold; only its tier field and
    its place change, and `rank` then places it within the tier. A row with a
    class goes only to the tier its class names (§403).
    """
    need(args, 2, "tier <id> <tier>")
    row_id, to = args
    refuse_unless_a_tier(to)
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    if row.is_closed():
        raise Refused(f"{row_id} is closed: {row.state}, and a closed row sits under ## Closed in no tier")
    if row.tier == to:
        raise Refused(f"{row_id} is tier {to} already")
    if row.tag is not None and CLASS.match(row.tag) and to != tier_of_class(row.tag):
        raise Refused(f"{row_id} carries the class {row.tag}, which puts it in tier {tier_of_class(row.tag)}; "
                      f"remove it first with `tag {row_id} -`")
    move_to_tier(queue, row, to)
    message = f"{row_id}: tier {row.tier} -> {to}, at the foot of ## Tier {to}"
    return queue, message, None


def refuse_unless_a_tier(tier):
    if tier not in TIERS_IN_ORDER:
        raise Refused(f"'{tier}' is not a tier: {', '.join(TIERS_IN_ORDER)}")


def tag_step(queue, claims, here, args, batch):
    """Set, replace or remove the tag an open row's description starts with (§352).

    A class says how bad a defect is and how likely a user's model meets it,
    and the fill takes tiers 1 and 5 in class order. The class also names the
    row's tier, A and B tier 1 and C and D tier 5, so setting one moves the row
    there (§403.4). `small` lets the fill hand a lane several such rows at once.
    A row carries one tag at most, since a defect needs a trace first and so is
    never small.
    """
    need(args, 2, "tag <id> <class>|small|-")
    row_id, tag = args
    if tag == "-":
        tag = None
    elif not (CLASS.match(tag) or tag == SMALL):
        raise Refused(f"'{tag}' is not a tag: a class (wrong, crash or refuses, then /A, /B, /C or /D), small, or -")
    refuse_conflicted(here, queue)
    row = require_row(queue, here, row_id)
    if row.is_closed():
        raise Refused(f"{row_id} is closed: {row.state}")
    is_class = tag is not None and CLASS.match(tag) is not None
    if is_class and row.tier not in ("1", "5"):
        raise Refused(f"{row_id} is tier {row.tier}, and a class ranks tier 1 and tier 5 rows only")
    if tag == row.tag:
        raise Refused(f"{row_id} is tagged {tag or 'nothing'} already")
    queue.replace(row, row.with_tag(tag))
    message = f"{row_id}: tag {row.tag or '-'} -> {tag or '-'}"
    if is_class and tier_of_class(tag) != row.tier:
        to = tier_of_class(tag)
        move_to_tier(queue, queue.find(row_id), to)
        message += f", tier {row.tier} -> {to}, at the foot of ## Tier {to}"
    return queue, message, None


def move_to_tier(queue, row, to):
    """`row` at the foot of `## Tier <to>`, its tier field set to `to`.

    Writing a missing heading can put five lines above the row, so the row is
    found again by its id before it is taken out.
    """
    block = queue.block_for_tier(to)
    if block is None:
        raise Refused(f"there is no '## Tier {to}' block, and no later heading to write one above")
    moved = queue.find(row.id)
    queue.remove(moved)
    queue.append_to_block(block, moved.with_tier(to))


def pick_step(queue, claims, here, args, batch):
    need(args, 2, "pick <id> ours|theirs")
    row_id, side = args
    if side not in ("ours", "theirs"):
        raise Refused(f"'{side}' is not ours or theirs")
    lines = queue.lines
    start = None
    for index, line in enumerate(lines):
        if line.startswith("<<<<<<<"):
            start = index
        elif line.startswith(">>>>>>>") and start is not None:
            block = lines[start:index + 1]
            middle = next(i for i, text in enumerate(block) if text.startswith("======="))
            ours, theirs = block[1:middle], block[middle + 1:-1]
            if any(text.split()[:1] == [row_id] for text in ours + theirs):
                kept = ours if side == "ours" else theirs
                queue.lines = lines[:start] + kept + lines[index + 1:]
                queue.parse()
                for text in kept:
                    put_in_its_block(queue, queue.find(text.split()[0]))
                message = f"{row_id}: kept {side}"
                return queue, message, None
            start = None
    raise Refused(f"no conflict in {here.relative} holds {row_id}")


def put_in_its_block(queue, row):
    """The markers stand where the frame side had the row; the side kept may belong elsewhere."""
    home = "closed" if row.is_closed() else row.tier
    if row.block[0] == home:
        return
    block = queue.block_of_key(home)
    if block is None:
        raise Refused(f"{row.id} belongs under the {home} heading, and the file has no block there")
    queue.remove(row)
    queue.append_to_block(block, row.text)


STEP_FUNCS = {
    "take": take_step, "add": add_step, "done": done_step, "drop": drop_step, "park": park_step,
    "after": after_step, "rank": rank_step, "tier": tier_step, "tag": tag_step, "pick": pick_step,
}


def show(here, args):
    need(args, 1, "show <id>")
    row_id = args[0]
    queue = here.read()
    row = queue.find(row_id)
    claims = Claims(here.claims)
    if row:
        print(f"{here.relative}, under {'## Closed' if row.block[0] == 'closed' else '## Tier ' + row.block[0]}:")
        print(f"    {row.text}")
    else:
        print(f"{row_id} is not a row in this copy")
        if row_id in claims.added():
            event = claims.added()[row_id]
            print(f"handed out by `just queue add` in {event[3]} at {event[2]}:\n    {event[4]}")
    hold = claims.holds().get(row_id)
    print(f"held by {hold['branch']} from {hold['tree']} since {hold['time']}" if hold else "no hold")
    for tree, text in here.other_copies(lanes_too=False):
        theirs = Queue(text).find(row_id) if row_id in text else None
        if theirs and (row is None or theirs.state != row.state):
            print(f"{theirs.state} in {tree.path}")


def branch_tip(branch):
    """The commit a branch points at, or None if no such branch exists here."""
    found = subprocess.run(["git", "rev-parse", "--verify", "--quiet", f"refs/heads/{branch}"],
                           capture_output=True, text=True)
    return found.stdout.strip() if found.returncode == 0 else None


def is_ancestor(maybe_ancestor, of):
    return subprocess.run(["git", "merge-base", "--is-ancestor", maybe_ancestor, of],
                          capture_output=True).returncode == 0


def already_landed(sha):
    for ref in ("origin/main", "main"):
        exists = subprocess.run(["git", "rev-parse", "--verify", "--quiet", ref],
                                capture_output=True).returncode == 0
        if exists and is_ancestor(sha, ref):
            return True
    return False


def has_a_tree(branch, branches, trees, claimed, left):
    """Whether some currently locked tree is this row's, once its own branch's tree is gone (§263.4, §299).

    A detached tree whose own reflog says it left this branch is this row's,
    whether or not the branch's tip is on main (`left`, §299.6). A verifier's
    tree is detached (§266.2), so it carries no branch to match; a
    merger's carries `merge-<label>` instead of `lane-<label>` (§266.3). The
    verifier's tree is checked out from the lane branch's tip and only ever
    gains commits on top of it, so that tip stays an ancestor of its HEAD for as
    long as the verifier runs. A tip already landed on main is excluded: a
    landed branch still held is a release nobody wrote (a stale claim, §263.4),
    not a live run, and attributing it to some unrelated detached tree would
    hide that stale hold instead of counting it.

    `claimed` collects the path of every detached tree an ancestry match has
    already used, so two rows checked in the same `running` call never both
    claim the one tree.

    Returns `None`, rather than `False`, when the branch has been deleted
    outright: `git rev-parse` can no longer read a tip to test ancestry
    against at all, which a merely-unstarted or merely-gone-worktree row never
    hits (§299.3). `running` resolves that case itself, from what is left of
    the detached trees once every row with a live branch has claimed its own.
    """
    if branch in branches:
        return True
    if branch.startswith("lane-") and f"merge-{branch[len('lane-'):]}" in branches:
        return True
    if branch in left:
        return True
    tip = branch_tip(branch)
    if tip is None:
        return None
    if already_landed(tip):
        return False
    for tree in trees:
        if tree.is_lane and tree.branch == "(detached)" and tree.path not in claimed \
                and is_ancestor(tip, tree.head):
            claimed.add(tree.path)
            return True
    return False


def running(here, args):
    """What holds a slot on this machine: every lane tree an agent holds, and every hold with no tree yet.

    A row whose lane branch is deleted outright (§299.3, Q234) leaves
    `has_a_tree` nothing to test ancestry against, so those rows are set aside
    and matched against whatever detached tree is left once every other row
    has claimed its own -- one row to one tree, so a row that really has none
    left still gets its fallback line, and two such rows never collapse onto
    one slot.

    A bundle of small rows is several holds on one branch, and one lane (§352),
    so holds are counted per branch: the rows sharing one are named together.

    `--count` prints the number alone, for `queue next`.
    """
    trees = here.trees()
    counted = [(tree.branch, tree.path) for tree in trees if tree.is_lane]
    branches = {branch for branch, _ in counted}
    by_branch = {}
    for row_id, hold in sorted(Claims(here.claims).holds().items()):
        by_branch.setdefault(hold["branch"], ([], hold))[0].append(row_id)
    left = branches_left(trees, by_branch)
    claimed = set(left.values())
    branch_gone = []
    for branch, (row_ids, hold) in by_branch.items():
        ids = ",".join(row_ids)
        found = has_a_tree(branch, branches, trees, claimed, left)
        if found is None:
            branch_gone.append((ids, hold))
        elif not found:
            counted.append((branch, f"{ids} taken from {hold['tree']}, no tree yet"))
    spare = sum(1 for tree in trees
                if tree.is_lane and tree.branch == "(detached)" and tree.path not in claimed)
    for ids, hold in branch_gone[spare:]:
        counted.append((hold["branch"], f"{ids} taken from {hold['tree']}, no tree yet"))
    if args == ["--count"]:
        print(len(counted))
        return
    for branch, where in counted:
        print(f"{branch}  {where}")
    print(f"{len(counted)} running")


def branches_left(trees, held):
    """Each held branch a detached lane tree was last on, with that tree's path (§299.6).

    A read-only lane that builds an old commit leaves its branch for a detached
    HEAD, and its branch tip stays at its base, already on main, so
    `has_a_tree` reads the hold as stale and its tree as some other row's. The
    tree's own HEAD reflog still names the branch: `git switch --detach` writes
    `checkout: moving from <branch> to <commit>`, and a move between two
    detached commits names the commit it left by its full hash. That is the
    tree's own record, not a guess from ancestry, so it is no unrelated tree
    (§299.2). The trees it names are claimed before any ancestry match runs. A
    reflog that cannot be read names nothing, and the row is counted twice: the
    cheaper mistake of §263.4.
    """
    left = {}
    for tree in trees:
        if not (tree.is_lane and tree.branch == "(detached)"):
            continue
        log = subprocess.run(["git", "-C", tree.path, "reflog", "show", "--format=%gs", "HEAD"],
                             capture_output=True, text=True)
        for subject in log.stdout.splitlines() if log.returncode == 0 else []:
            moved = re.match(r"checkout: moving from (\S+) to ", subject)
            if moved and not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", moved.group(1)):
                if moved.group(1) in held:
                    left[moved.group(1)] = tree.path
                break
    return left


def elsewhere(here, args):
    """Rows another dispatcher holds or has closed, for the fill to steer round.

    A held row is `lane:` in another worktree's hold or another dispatcher's
    copy, and is neither running nor closed in this one: the fill must not
    offer it, and must treat its write set as busy. A closed row is `done:` or
    `dropped:` in another copy — main's own worktree among them — while this
    copy has not caught up: the fill must not offer it either, but its write
    set is free, since the work behind it is finished (§263.9).
    """
    queue = here.read()
    held = {}
    for row_id, hold in Claims(here.claims).holds().items():
        if hold["tree"] != here.top:
            held[row_id] = (f"lane:{hold['branch']}", hold["write_set"], hold["tree"])
    closed = {}
    for tree, text in here.other_copies(lanes_too=False):
        for row in Queue(text).rows:
            if row.state.startswith("lane:"):
                held.setdefault(row.id, (row.state, row.write_set, tree.path))
            elif row.is_closed():
                closed.setdefault(row.id, (row.state, tree.path))
    sep = "\037"
    for row_id, (state, write_set, where) in sorted(held.items()):
        mine = queue.find(row_id)
        if mine and (mine.is_closed() or mine.state.startswith("lane:")):
            continue
        raw = f"{row_id}  -  {state}  -  {write_set}  (held in {where})"
        print(sep.join(["R", "elsewhere", "", "6", row_id, "-", state, "-", write_set, f"(held in {where})", raw]))
    for row_id, (state, where) in sorted(closed.items()):
        mine = queue.find(row_id)
        if mine is None or mine.is_closed():
            continue
        raw = f"{row_id}  -  {state}  -  -  (closed in {where})"
        print(sep.join(["R", "closedelsewhere", "", "6", row_id, "-", state, "-", "-", f"(closed in {where})", raw]))


def refuse_conflicted(here, queue):
    if queue.conflicted:
        raise Refused(f"{here.relative} still holds a merge conflict at line {queue.conflicted[0] + 1}; "
                      f"settle each row in it with `just queue pick <id> ours|theirs` first")


def checked(here, queue):
    """Refuse unless the file as it would be passes `just check-queue`: one set of rules, not two."""
    directory = os.path.dirname(here.file)
    fd, candidate = tempfile.mkstemp(prefix=".queue-", suffix=".md", dir=directory)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(queue.text())
        result = subprocess.run(
            [os.environ.get("QUEUE_JUST", "just"), "--justfile", os.environ["QUEUE_JUSTFILE"],
             "--working-directory", os.getcwd(), "check-queue", candidate],
            capture_output=True, text=True)
    finally:
        os.unlink(candidate)
    if result.returncode != 0:
        said = (result.stdout + result.stderr).replace(candidate, f"{here.relative} (as it would be)")
        raise Refused(f"nothing written, because check-queue refuses the result:\n{said.rstrip()}")


def write(here, queue):
    directory = os.path.dirname(here.file)
    fd, staged = tempfile.mkstemp(prefix=".queue-", suffix=".md", dir=directory)
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        f.write(queue.text())
    os.chmod(staged, os.stat(here.file).st_mode & 0o777)
    os.replace(staged, here.file)


def section(arg):
    match = SECTION.match(arg)
    if not match:
        raise Refused(f"'{arg}' is not a section: §N or §N.M")
    return "§" + match.group(1)


def need(args, n, usage):
    if len(args) != n:
        raise Refused(f"usage: {usage}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
