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
    take     <id>  <time>  <worktree>  <branch>  <write set>
    release  <id>  <time>  <worktree>  <how>

A row is held from its `take` until the `release` that `done`, `drop` or `park`
writes. Two worktrees taking one row is refused at the second `take`, before
either has committed the file, because the hold is written under the lock.
"""

import fcntl
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rows import Queue, highest_q  # noqa: E402

ID = re.compile(r"^[TDQ][0-9]+$")
BRANCH = re.compile(r"^[A-Za-z0-9._/-]+$")
SHA = re.compile(r"^[0-9a-f]{7,40}$")
SECTION = re.compile(r"^§?([0-9]+(\.[0-9]+)?)$")

USAGE = """\
just queue                                 ready rows by tier, then running, waiting, parked
just queue ready                           the ready rows alone
just queue next [N]                        what N free slots take; N defaults to the free slots on this machine
just queue running                         what holds a slot on this machine, one line each
just queue show <id>                       one row, its hold, and its state in other worktrees
just queue take <id> <branch>              dispatch a ready row to <branch>: state lane:<branch>, held until done/drop/park
just queue add <tier> <model> <write set> <what ending in §N>    a new todo row at the foot of its tier; prints the id
just queue done <id> <merge sha>           state done:<sha>, moved under ## Closed
just queue drop <id> <§N>                  state dropped:§N, moved under ## Closed
just queue park <id> <§N>                  state parked:§N, left in its tier
just queue after <id> <id>[,<id>...]       state after:<ids>; each id must exist in some copy, branch or claim
just queue pick <id> ours|theirs           keep one side of a row the merge driver left in conflict"""


class Refused(Exception):
    pass


def main(argv):
    if not argv:
        print(USAGE, file=sys.stderr)
        return 2
    verb, args = argv[0], argv[1:]
    here = Here(os.environ.get("QUEUE_FILE", "docs/queue.md"))
    verbs = {
        "take": take, "add": add, "done": done, "drop": drop, "park": park,
        "after": after, "pick": pick, "show": show, "running": running,
        "elsewhere": elsewhere,
    }
    if verb not in verbs:
        print(f"queue: '{verb}' is not a verb\n{USAGE}", file=sys.stderr)
        return 2
    try:
        verbs[verb](here, args)
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
                held[event[1]] = {"time": event[2], "tree": event[3], "branch": event[4], "write_set": event[5]}
            elif event[0] == "release":
                held.pop(event[1], None)
        return held

    def added(self):
        return {event[1]: event for event in self.events() if event[0] == "add"}


def now():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def take(here, args):
    need(args, 2, "take <id> <branch>")
    row_id, branch = args
    if not BRANCH.match(branch):
        raise Refused(f"'{branch}' cannot be a branch in lane:<branch>; letters, digits and ._/- only")
    with Claims(here.claims) as claims:
        queue = open_row(here, row_id)
        row = queue.find(row_id)
        hold = claims.holds().get(row_id)
        if hold and hold["tree"] != here.top:
            raise Refused(f"{row_id} is held by {hold['branch']} from {hold['tree']} since {hold['time']}")
        for tree, text in here.other_copies(lanes_too=False):
            theirs = Queue(text).find(row_id) if row_id in text else None
            if theirs and theirs.state.startswith("lane:") and theirs.state != row.state:
                raise Refused(f"{row_id} is {theirs.state} in {tree.path}'s copy, which has not reached this one")
        # A lane: row this worktree holds, or dispatched by hand here, may be
        # taken again for a new branch: that is a failed lane re-dispatched.
        if not row.state.startswith("lane:"):
            refuse_unless_ready(queue, row)
        queue.replace(row, row.with_state(f"lane:{branch}"))
        checked(here, queue)
        claims.append("take", row_id, now(), here.top, branch, row.write_set)
        write(here, queue)
    print(f"{row_id}: {row.state} -> lane:{branch}, held for {branch} until done, drop or park")


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


def add(here, args):
    if len(args) < 4:
        raise Refused("usage: add <tier> <model> <write set> <what, ending in §N>")
    tier, model, write_set = args[:3]
    what = " ".join(args[3:])
    with Claims(here.claims) as claims:
        queue = here.read()
        refuse_conflicted(here, queue)
        block = queue.block_of_key(tier)
        if block is None:
            raise Refused(f"there is no '## Tier {tier}' block to add a row to")
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
        checked(here, queue)
        claims.append("add", row_id, now(), here.top, text)
        write(here, queue)
    print(row_id)


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


def done(here, args):
    need(args, 2, "done <id> <merge sha>")
    row_id, sha = args
    if not SHA.match(sha):
        raise Refused(f"'{sha}' is not a sha: 7 to 40 lowercase hex digits")
    if subprocess.run(["git", "rev-parse", "--verify", "--quiet", f"{sha}^{{commit}}"],
                      capture_output=True).returncode != 0:
        raise Refused(f"{sha} is not a commit in this repository")
    close(here, row_id, f"done:{sha}", "done")


def drop(here, args):
    need(args, 2, "drop <id> <§N>")
    close(here, args[0], f"dropped:{section(args[1])}", "drop")


def close(here, row_id, state, how):
    with Claims(here.claims) as claims:
        queue = open_row(here, row_id)
        row = queue.find(row_id)
        if row.is_closed():
            raise Refused(f"{row_id} is closed already: {row.state}")
        closed = queue.block_of_key("closed")
        if closed is None:
            raise Refused("there is no '## Closed' block to move the row under")
        queue.remove(row)
        queue.append_to_block(closed, row.with_state(state))
        checked(here, queue)
        claims.append("release", row_id, now(), here.top, how)
        write(here, queue)
    print(f"{row_id}: {row.state} -> {state}, moved under ## Closed")


def park(here, args):
    need(args, 2, "park <id> <§N>")
    row_id, state = args[0], f"parked:{section(args[1])}"
    with Claims(here.claims) as claims:
        queue = open_row(here, row_id)
        row = queue.find(row_id)
        if row.is_closed():
            raise Refused(f"{row_id} is closed: {row.state}")
        queue.replace(row, row.with_state(state))
        checked(here, queue)
        claims.append("release", row_id, now(), here.top, "park")
        write(here, queue)
    print(f"{row_id}: {row.state} -> {state}")


def after(here, args):
    if len(args) < 2:
        raise Refused("usage: after <id> <id>[,<id>...]")
    row_id = args[0]
    waits = [w for arg in args[1:] for w in arg.split(",") if w]
    with Claims(here.claims) as claims:
        queue = open_row(here, row_id)
        row = queue.find(row_id)
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
        checked(here, queue)
        write(here, queue)
    print(f"{row_id}: {row.state} -> {state}")


def exists_elsewhere(here, claims, row_id):
    if row_id in claims.added():
        return True
    pattern = re.compile(rf"^\s*{re.escape(row_id)}\s", re.M)
    if any(pattern.search(text) for _, text in here.other_copies(lanes_too=True)):
        return True
    return any(pattern.search(text) for text in branch_copies(here.relative))


def pick(here, args):
    need(args, 2, "pick <id> ours|theirs")
    row_id, side = args
    if side not in ("ours", "theirs"):
        raise Refused(f"'{side}' is not ours or theirs")
    with Claims(here.claims):
        queue = here.read()
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
                    queue.lines = lines[:start] + (ours if side == "ours" else theirs) + lines[index + 1:]
                    queue.parse()
                    checked(here, queue)
                    write(here, queue)
                    print(f"{row_id}: kept {side}")
                    return
                start = None
        raise Refused(f"no conflict in {here.relative} holds {row_id}")


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


def running(here, args):
    """What holds a slot on this machine: every lane tree an agent holds, and every hold with no tree yet.

    `--count` prints the number alone, for `queue next`.
    """
    trees = here.trees()
    counted = [(tree.branch, tree.path) for tree in trees if tree.is_lane]
    branches = {branch for branch, _ in counted}
    for row_id, hold in sorted(Claims(here.claims).holds().items()):
        if hold["branch"] not in branches:
            counted.append((hold["branch"], f"{row_id} taken from {hold['tree']}, no tree yet"))
    if args == ["--count"]:
        print(len(counted))
        return
    for branch, where in counted:
        print(f"{branch}  {where}")
    print(f"{len(counted)} running")


def elsewhere(here, args):
    """Rows another dispatcher holds, as `_queue-rows` records, for the fill to steer round.

    Each is held by a `take` from another worktree, or is `lane:` in another
    dispatcher's copy, and is neither running nor closed in this one.
    """
    queue = here.read()
    found = {}
    for row_id, hold in Claims(here.claims).holds().items():
        if hold["tree"] != here.top:
            found[row_id] = (f"lane:{hold['branch']}", hold["write_set"], hold["tree"])
    for tree, text in here.other_copies(lanes_too=False):
        for row in Queue(text).rows:
            if row.state.startswith("lane:"):
                found.setdefault(row.id, (row.state, row.write_set, tree.path))
    sep = "\037"
    for row_id, (state, write_set, where) in sorted(found.items()):
        mine = queue.find(row_id)
        if mine and (mine.is_closed() or mine.state.startswith("lane:")):
            continue
        raw = f"{row_id}  -  {state}  -  {write_set}  (held in {where})"
        print(sep.join(["R", "elsewhere", "", "6", row_id, "-", state, "-", write_set, f"(held in {where})", raw]))


def open_row(here, row_id):
    if not ID.match(row_id):
        raise Refused(f"'{row_id}' is not an id: T, D or Q and then a number")
    queue = here.read()
    refuse_conflicted(here, queue)
    if queue.find(row_id) is None:
        raise Refused(f"{row_id} is not a row in {here.relative}")
    return queue


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
