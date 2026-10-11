#!/usr/bin/env python3
"""`just land`: one clean merge of a verified lane onto main, or a named stop.

`docs/decisions.md` §383.4 item 2 is the spec and §384 the build: `merger.md`'s steps 1, 3, 4 and 5
as they stand, with no judgement anywhere. The lane wrote its rows as `add`
steps, its verifier said `merge`, and this does the rest: fetch, branch from
`origin/main`, unlock, check the sha, merge, close the row and add the lane's
rows in one `queue apply`, `check-queue`, commit, gate, and the push loop with
§367's free catch-ups over docs, then `ff-main`, unlock and `mark-finished`.

**Anything that wants judgement is a stop, named.** The sha differs from the
lane's report, a merge conflicts, §0 names the row, `queue apply` or
`check-queue` refuses, the gate fails, or the attempts run out. `lanes.js`
hands a stop to the Opus merger with the stop line in its brief (§383.5), so
this never settles anything itself.

It ends with one line of JSON holding `mergedSchema`'s fields from `lanes.js`,
and a `stop` naming the stop, empty when it merged.

**Landers share one gate** (§397.5). `just land <arguments>` starts one run in
a session of its own and follows its log for at most `LAND_WAIT` seconds, 540
by default, and `just land wait` follows it again, as `just verify-lane` does
(§385.2). The run writes its lane's entry to a spool in the git common
directory and waits on a `flock` there. Holding it, a run whose lane an earlier
holder landed reports that merge and ends. Any other run lands its own lane,
then every lane waiting whose run is alive, oldest first, each with its own
merge commit and queue apply, and gates them once. A red gate with other lanes
in it goes back to `origin/main` and lands its own lane alone. A run outlives
its lander, so the Opus merger runs `just land wait-for <lane branch>` from its
own tree, which waits until no run is landing that lane.

Python rather than bash because the tree it runs from is switched under it:
python reads the whole file before it runs, bash reads a script as it goes.
"""

import fcntl
import glob
import json
import os
import re
import signal
import subprocess
import sys
import time

MERGE_ATTEMPTS = 3
DOCS_CATCH_UPS = 5
JUST = os.environ.get("LAND_JUST", "just")
WAIT = float(os.environ.get("LAND_WAIT", "540"))
STATE = os.path.join("target", "land")
ROW = re.compile(r"^[TDQ][0-9]+$")
SHA = re.compile(r"^[0-9a-f]{7,40}$")
STEP_ARGS = {"add": 4, "after": 2, "tag": 2}
USAGE = ("usage: just land [--steps <file>] <row>[,<row>...] <lane branch> <lane sha> "
         "<merge branch> [<tree to unlock and mark finished>...], then just land wait for as long as "
         "its last line says it is still running")


class Stop(Exception):
    def __init__(self, name, why):
        super().__init__(why)
        self.name = name
        self.why = why


def main(argv):
    if argv == ["wait"]:
        top = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip()
        return follow(os.path.join(top, STATE))
    if len(argv) == 2 and argv[0] == "wait-for":
        return wait_for(argv[1])
    if argv == ["--detached-run"]:
        return detached_run()
    return start(argv)


def new_report():
    return {
        "start_sha": "",
        "worktree_path": "",
        "merge_branch": "",
        "merged": False,
        "merged_sha": "",
        "gate_pass_line": "",
        "push_attempts": 0,
        "docs_catch_ups": 0,
        "ff_main": "",
        "fixes_applied": [],
        "rows_closed": [],
        "rows_added": [],
        "unlocked": [],
        "not_merged_because": "",
        "needs_judgement": [],
        "stop": "",
    }


def start(argv):
    """Checks the arguments and the tree, then starts the run and follows it.

    A stop here comes before there is any run, so this prints the JSON itself
    and unlocks its own tree, as a stop in the run does. A tree that has
    started a run already is refused whatever the arguments, since unlocking
    it would free a tree a run is still at work in.
    """
    report = new_report()
    try:
        top = output("git", "rev-parse", "--show-toplevel")
        report["worktree_path"] = top
        state = os.path.join(top, STATE)
        if os.path.exists(state):
            print(f"land: {state} is there already, so this tree has started a run; "
                  f"`mise exec -- just land wait` follows it or prints its result again")
            return 2
        try:
            args = parsed(argv)
            report["merge_branch"] = args["merge_branch"]
            args["steps"] = read_steps(args["steps"])
            args["start_sha"] = report["start_sha"] = output("git", "rev-parse", "HEAD")
            refuse_unless_a_clean_linked_tree()
        except Stop:
            unlock(top, report)
            raise
    except Stop as stop:
        return ended(report, stop)
    os.makedirs(state)
    write(os.path.join(state, "printed"), "0")
    write_json(os.path.join(state, "args.json"), args)
    log_path = os.path.join(state, "log")
    write(log_path, "")
    start_detached([sys.executable, os.path.abspath(__file__), "--detached-run"], top, log_path,
                    os.path.join(state, "pid"))
    return follow(state)


def ended(report, stop):
    if stop is not None:
        report["stop"] = stop.name
        report["not_merged_because"] = stop.why
        print(f"land: stopped at {stop.name}: {stop.why}", flush=True)
    print(json.dumps(report, ensure_ascii=False), flush=True)
    return 0 if report["merged"] else 1


def parsed(argv):
    argv = list(argv)
    steps = None
    if argv[:1] == ["--steps"]:
        if len(argv) < 2:
            raise Stop("usage", USAGE)
        steps = argv[1]
        argv = argv[2:]
    if len(argv) < 4:
        raise Stop("usage", USAGE)
    rows, branch, sha, merge_branch, *trees = argv
    rows = rows.split(",")
    for row in rows:
        if not ROW.match(row):
            raise Stop("usage", f"'{row}' is not a row id: T, D or Q and then a number. {USAGE}")
    if not SHA.match(sha):
        raise Stop("usage", f"'{sha}' is not a sha: 7 to 40 lowercase hex digits. {USAGE}")
    return {"steps": steps, "rows": rows, "branch": branch, "sha": sha,
            "merge_branch": merge_branch, "trees": trees}


def read_steps(path):
    """The lane's steps, one word to a line and a `+` line between steps, as the lander wrote them.

    One word to a line is so that no word is ever quoted: a row's text holds
    quotes, backticks and `§`, and the lander copies it into a file rather than
    into a shell command (§384.2).
    """
    if path is None:
        return []
    try:
        with open(path, encoding="utf-8") as f:
            lines = [line.rstrip() for line in f.read().split("\n")]
    except OSError as error:
        raise Stop("steps", f"cannot read the steps file {path}: {error}")
    while lines and lines[-1] == "":
        lines.pop()
    if not lines:
        return []
    steps = [[]]
    for line in lines:
        if line == "+":
            steps.append([])
        else:
            steps[-1].append(line)
    for number, step in enumerate(steps, start=1):
        text = " ".join(step)
        if not step or "" in step:
            raise Stop("steps", f"step {number} of {path} is empty or holds a blank line: '{text}'")
        verb = step[0]
        if verb not in STEP_ARGS:
            raise Stop("steps", f"step {number} of {path} is '{text}': a lane's steps are add, after "
                                f"and tag, and just land writes the done steps itself")
        if len(step) - 1 != STEP_ARGS[verb]:
            raise Stop("steps", f"step {number} of {path} is '{text}': {verb} takes "
                                f"{STEP_ARGS[verb]} words after it, one to a line, and it has {len(step) - 1}")
    return steps


def refuse_unless_a_clean_linked_tree():
    own = output("git", "rev-parse", "--absolute-git-dir")
    common = output("git", "rev-parse", "--path-format=absolute", "--git-common-dir")
    if own == common:
        raise Stop("tree", "this is the repository's main working tree, the hub, where nothing happens "
                           "(§60); run just land from a worktree of its own")
    status = output("git", "status", "--porcelain")
    if status:
        raise Stop("tree", f"this tree has uncommitted changes, and just land switches it: {status}")


def start_detached(cmd, cwd, log_path, pid_path):
    """Starts `cmd` with a double fork, as `setsid -f` does, so nothing here ends up its parent (§429.5).

    The process in between forks the run, writes the run's own pid to
    `pid_path`, and exits; `waitpid` blocks until it has, which is also until
    the pid file is on disk, so the caller never has to sleep and hope it is
    there yet. The run calls `setsid` itself, so it leads its own session and
    process group, and `cmd`'s first word is executed in place of it, not
    read again later: the lane's checkout cannot change what runs detached.
    """
    middle = os.fork()
    if middle == 0:
        run_pid = os.fork()
        if run_pid == 0:
            os.chdir(cwd)
            os.setsid()
            devnull = os.open(os.devnull, os.O_RDONLY)
            os.dup2(devnull, 0)
            os.close(devnull)
            logfd = os.open(log_path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
            os.dup2(logfd, 1)
            os.dup2(logfd, 2)
            os.close(logfd)
            os.execv(cmd[0], cmd)
            os._exit(127)  # os.execv only returns on failure
        write(pid_path, f"{run_pid}\n")
        os._exit(0)
    os.waitpid(middle, 0)


def follow(state):
    """Print what the run logged since the last call, until it ends or this call's time is up.

    Exits with the run's own code once it has ended, its JSON the last line,
    and 3 while it is still going. Holds no handle to the run: a run outlives
    whatever started it, so the only trace of it here is its pid file.
    """
    try:
        with open(os.path.join(state, "pid"), encoding="utf-8") as f:
            pid = int(f.read())
    except (OSError, ValueError):
        print(f"land: no run has started in this tree. {USAGE}")
        return 2
    ended_file = os.path.join(state, "exit")
    deadline = time.monotonic() + WAIT
    printed = False
    while True:
        done = os.path.exists(ended_file)
        printed = print_new(state) or printed
        if done:
            if not printed:
                print(last_line(read(os.path.join(state, "log"))))
            return int(read(ended_file).strip() or 1)
        if not running(pid):
            if os.path.exists(ended_file):
                continue
            print(f"land: the run (pid {pid}) ended without finishing; {os.path.join(state, 'log')} "
                  f"holds what it ran")
            return 1
        if time.monotonic() >= deadline:
            branch = read_json(os.path.join(state, "args.json"))["branch"]
            print(f"land: still running (pid {pid}). Run `mise exec -- just land wait` to keep waiting; "
                  f"from another tree, run `mise exec -- just land wait-for {branch}` until it says no run is "
                  f"landing it, then fetch origin before you read origin/main")
            return 3
        time.sleep(0.2)


def print_new(state):
    """Prints the whole lines the log gained since the last print, and says whether there were any."""
    at_path = os.path.join(state, "printed")
    try:
        at = int(read(at_path) or 0)
    except ValueError:
        at = 0
    with open(os.path.join(state, "log"), "rb") as f:
        f.seek(at)
        gained = f.read()
    cut = gained.rfind(b"\n") + 1
    if cut == 0:
        return False
    sys.stdout.write(gained[:cut].decode("utf-8", errors="replace"))
    sys.stdout.flush()
    write(at_path, str(at + cut))
    return True


def running(pid):
    try:
        return stat_fields(pid)[0] != "Z"
    except OSError:
        return False


def wait_for(branch):
    """Waits at most `LAND_WAIT` seconds for no run to be landing `branch`, for the Opus merger in another tree.

    A detached run outlives the lander that started it, so `lanes.js` can hand
    the lane to the Opus merger while the run, or a holder that took its
    entry, may still push it (§397.5). So it waits while a live run's entry
    names the lane, and while anyone holds the lock: a holder can be landing an
    entry whose run died after the holder listed it, and which holder took
    which lanes is not to be read without the lock.
    """
    spool = Spool()
    lock = os.open(spool.lock, os.O_RDWR | os.O_CREAT, 0o644)
    deadline = time.monotonic() + WAIT
    while True:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            why = "a holder has the land lock"
        else:
            landing = [entry for entry in spool.everyone() if entry["branch"] == branch and alive(entry)]
            fcntl.flock(lock, fcntl.LOCK_UN)
            if not landing:
                print(f"land: no run is landing {branch}; fetch origin before you read origin/main")
                return 0
            why = f"the run in {landing[0]['tree']} (pid {landing[0]['pid']}) waits to land it"
        if time.monotonic() >= deadline:
            print(f"land: still running: {why}. Run `mise exec -- just land wait-for {branch}` to keep waiting")
            return 3
        time.sleep(0.5)


def detached_run():
    """The run itself, in a session of its own, writing the log `follow` prints.

    Its last line is the JSON, and the `exit` file it writes after that is what
    tells `follow` it is over.
    """
    signal.signal(signal.SIGHUP, signal.SIG_IGN)
    top = output("git", "rev-parse", "--show-toplevel")
    state = os.path.join(top, STATE)
    args = read_json(os.path.join(state, "args.json"))
    report = new_report()
    report.update(worktree_path=top, merge_branch=args["merge_branch"], start_sha=args["start_sha"])
    stop = None
    try:
        land(args, report)
        assert report["merged"], "land returned with no stop and no merge"
    except Stop as raised:
        stop = raised
    code = ended(report, stop)
    write(os.path.join(state, "exit"), f"{code}\n")
    return code


def land(args, report):
    """Waits for the land lock, then lands this lane, alone or with the lanes waiting, or reports that an earlier holder did.

    The lock is never released by hand: it goes when the process does, so a
    run killed anywhere never holds it. Every spool entry and result file is
    removed under the lock, so a holder never reads one half gone.
    """
    top = report["worktree_path"]
    spool = Spool()
    own = spool.add(args, top)
    say("land: waiting for the land lock")
    lock = os.open(spool.lock, os.O_RDWR | os.O_CREAT, 0o644)
    fcntl.flock(lock, fcntl.LOCK_EX)
    say("land: holding the land lock")
    try:
        result = spool.result(own)
        others = []
        try:
            if result is None:
                others = hold_batch(own, report, spool)
            else:
                say(f"land: an earlier run landed {own['branch']}, in {result['merged_sha']}")
                report.update(result)
        finally:
            unlock(top, report)
        if result is None:
            finish(own, others, report, spool)
        else:
            report["needs_judgement"] += mark_finished([top])
    finally:
        spool.forget(own)


def hold_batch(own, report, spool):
    """Lands this run's own lane, then every lane waiting whose run is alive, oldest first, and gates them once.

    Its own lane goes first, so a stop of its own comes before any other lane's
    queue apply, and leaves none to throw away. Its own lane and verifier trees
    are unlocked straight after the switch, as `merger.md` step 1 does, and the
    other lanes' only once their merge is pushed. Returns the other lanes it
    landed.
    """
    command("fetch", "git", "fetch", "origin")
    command("switch", "git", "switch", "-c", own["merge_branch"], "origin/main")
    for tree in own["trees"]:
        unlock(tree, report)
    landing = land_or_reset(own)
    others = []
    for entry in spool.waiting(own):
        try:
            entry["landing"] = land_or_reset(entry)
        except Stop as stop:
            say(f"land: {entry['branch']} stopped at {stop.name}, so it is left for its own run: {stop.why}")
            continue
        others.append(entry)
    try:
        gate_and_push(report)
    except Stop as stop:
        if stop.name != "gate" or not others:
            raise
        say(f"land: the gate failed with {len(others) + 1} lanes merged, so {own['branch']} lands alone "
            f"and the others are left for their own runs: {stop.why}")
        command("fetch", "git", "fetch", "origin")
        command("reset", "git", "reset", "-q", "--hard", "origin/main")
        landing = land_or_reset(own)
        others = []
        gate_and_push(report)
    report["rows_closed"] = landing["rows_closed"]
    report["rows_added"] = landing["rows_added"]
    report["needs_judgement"] += landing["needs_judgement"]
    return others


def land_or_reset(entry):
    """`land_one`, with a stop putting the branch back where it was, mid-merge or not."""
    before = output("git", "rev-parse", "HEAD")
    try:
        return land_one(entry)
    except Stop:
        command("reset", "git", "reset", "-q", "--hard", before)
        raise


def land_one(entry):
    """One lane's merge commit: the sha check, the merge, the §0 check, its queue apply, `check-queue` and the commit."""
    landing = {"rows_closed": [], "rows_added": [], "unlocked": [], "needs_judgement": []}
    tip = lane_tip(entry["branch"], entry["sha"])
    merge("conflict", "git", "merge", "--no-ff", "--no-commit", entry["branch"])
    refuse_if_section_0_names(entry["rows"])
    apply_queue(entry["rows"], tip, entry["steps"], landing)
    command("check-queue", JUST, "check-queue")
    command("commit", "git", "add", "docs/queue.md")
    message = f"Merge {entry['branch']}: {', '.join(entry['rows'])} done, by just land"
    command("commit", "git", "commit", "-q", "-m", message)
    left = output("git", "status", "--short")
    if left:
        raise Stop("commit", f"git status --short prints something after the merge commit: {left}")
    return landing


def lane_tip(branch, reported):
    rc, tip = run("git", "rev-parse", "--verify", "--quiet", f"{branch}^{{commit}}")
    if rc != 0:
        raise Stop("sha-differs", f"{branch} is not a branch here, so it cannot end at {reported}")
    if not tip.startswith(reported):
        raise Stop("sha-differs", f"{branch} ends at {tip}, and the lane reported {reported}")
    if run("git", "merge-base", "--is-ancestor", tip, "origin/main")[0] == 0:
        raise Stop("already-on-main", f"origin/main already holds {tip}, so a merge landed it before this one")
    return tip


def merge(stop, *cmd):
    rc, said = run(*cmd)
    if rc == 0:
        return
    conflicted = output("git", "diff", "--name-only", "--diff-filter=U")
    run("git", "merge", "--abort")
    raise Stop(stop, f"{' '.join(cmd)} failed, conflicting in: {conflicted or '(no file named)'}; {last_line(said)}")


def refuse_if_section_0_names(rows):
    found = sorted(glob.glob("docs/decisions.d/000-*.md"))
    if len(found) != 1:
        raise Stop("section-0", f"found {len(found)} files for §0 under docs/decisions.d/, not one, "
                                f"so it cannot tell whether §0 names the row")
    with open(found[0], encoding="utf-8") as f:
        text = f.read()
    named = [row for row in rows if re.search(rf"(?<![A-Za-z0-9]){row}(?![0-9])", text)]
    if named:
        raise Stop("section-0", f"§0 names {', '.join(named)}, so this merge edits §0 in the same commit "
                                f"(§172.5), and that wants a reader")


def apply_queue(rows, tip, steps, landing):
    short = output("git", "rev-parse", "--short=8", tip)
    batch = [["done", row, short] for row in rows] + steps
    words = []
    for step in batch:
        words += (["+"] if words else []) + step
    rc, said, out = run_split(JUST, "queue", "apply", *words)
    if rc != 0:
        raise Stop("queue-apply", f"just queue apply refused the batch: {last_line(said)}")
    landing["rows_closed"] = [f"{row} done:{short}" for row in rows]
    ids = [line for line in out.splitlines() if ROW.match(line.strip())]
    adds = [step for step in steps if step[0] == "add"]
    if len(ids) != len(adds):
        landing["needs_judgement"].append(f"queue apply printed {len(ids)} new ids for {len(adds)} add steps")
    for row_id, (_, tier, model, _write_set, what) in zip(ids, adds):
        landing["rows_added"].append(f"{row_id.strip()} tier {tier} {model}: {what[:80]}")


def gate_and_push(report):
    report["push_attempts"] = 0
    report["docs_catch_ups"] = 0
    report["gate_pass_line"] = gate()
    push(report)


def gate():
    rc, said = run(JUST, "gate", "quiet")
    lines = said.splitlines()
    if rc != 0:
        failed = [line for line in lines if line.startswith("gate: FAIL")]
        raise Stop("gate", (failed or [last_line(said)])[0])
    passed = [line for line in lines if line.startswith("gate: PASS")]
    if not passed:
        raise Stop("gate", "the gate exited 0 and printed no 'gate: PASS' line")
    return passed[0]


def push(report):
    """`merger.md` step 4: the ancestor check, the push, and a catch-up when another merge got there first."""
    report["push_attempts"] = 1
    while True:
        command("fetch", "git", "fetch", "origin")
        if run("git", "merge-base", "--is-ancestor", "origin/main", "HEAD")[0] == 0:
            rc, said = run("git", "push", "origin", "HEAD:main")
            if rc == 0:
                report["merged"] = True
                report["merged_sha"] = output("git", "rev-parse", "HEAD")
                return
            command("fetch", "git", "fetch", "origin")
            if run("git", "merge-base", "--is-ancestor", "origin/main", "HEAD")[0] == 0:
                raise Stop("push", f"the push was refused with origin/main still behind this branch, "
                                   f"so no other merge explains it: {last_line(said)}")
        gained = output("git", "diff", "--name-only", "HEAD...origin/main").splitlines()
        if all(path.startswith("docs/") or path.endswith(".md") for path in gained):
            report["docs_catch_ups"] += 1
        else:
            report["push_attempts"] += 1
        if report["docs_catch_ups"] > DOCS_CATCH_UPS or report["push_attempts"] > MERGE_ATTEMPTS:
            report["docs_catch_ups"] = min(report["docs_catch_ups"], DOCS_CATCH_UPS)
            report["push_attempts"] = min(report["push_attempts"], MERGE_ATTEMPTS)
            raise Stop("attempts", f"origin/main moved past this branch every time: {MERGE_ATTEMPTS} push "
                                   f"attempts and {DOCS_CATCH_UPS} catch-ups over docs alone are the bound (§367)")
        merge("conflict", "git", "merge", "--no-edit", "origin/main")
        command("check-queue", JUST, "check-queue")
        report["gate_pass_line"] = gate()


def finish(own, others, report, spool):
    """`ff-main`, then every lane's trees unlocked and marked finished, and last a result file for each other lane.

    The results go last so that each holds every field. A holder that dies
    before writing them leaves those lanes on `main` with no result, so each of
    their runs stops at already-on-main, which goes to the Opus merger (§384.3).
    """
    shared = []
    rc, said = run(JUST, "ff-main")
    report["ff_main"] = last_line(said)
    if rc != 0:
        shared.append(f"ff-main refused, and the tree it would move is not this one: {last_line(said)}")
    for entry in others:
        for tree in entry["trees"]:
            unlock(tree, entry["landing"])
    shared += mark_finished(own["trees"] + [tree for entry in others for tree in entry["trees"]] + [own["tree"]])
    report["needs_judgement"] += shared
    for entry in others:
        landing = entry["landing"]
        spool.leave_result(entry, {
            "merge_branch": report["merge_branch"],
            "merged": True,
            "merged_sha": report["merged_sha"],
            "gate_pass_line": report["gate_pass_line"],
            "push_attempts": report["push_attempts"],
            "docs_catch_ups": report["docs_catch_ups"],
            "ff_main": report["ff_main"],
            "rows_closed": landing["rows_closed"],
            "rows_added": landing["rows_added"],
            "unlocked": landing["unlocked"],
            "needs_judgement": landing["needs_judgement"] + shared,
        })


def mark_finished(trees):
    rc, said = run(JUST, "mark-finished", *trees)
    if rc != 0:
        return [f"mark-finished refused, so the run's trees wait out their hour: {last_line(said)}"]
    return []


class Spool:
    """The lanes waiting to land, one JSON file each, and the result a holder leaves for each lane it landed.

    Both sit in the git common directory, beside the lock, so every tree of the
    repository sees the same ones. A file's name starts with the time it was
    written, so sorting the names puts the oldest first.
    """

    def __init__(self):
        common = output("git", "rev-parse", "--path-format=absolute", "--git-common-dir")
        where = os.path.join(common, "rcad-land")
        self.lock = os.path.join(where, "lock")
        self.entries = os.path.join(where, "spool")
        self.results = os.path.join(where, "results")
        os.makedirs(self.entries, exist_ok=True)
        os.makedirs(self.results, exist_ok=True)

    def add(self, args, top):
        pid = os.getpid()
        entry = dict(args, id=f"{time.time_ns():020d}-{pid}", pid=pid, pid_start=stat_fields(pid)[19], tree=top)
        write_json(self.entry_path(entry), entry)
        return entry

    def waiting(self, own):
        """The other entries to land, oldest first: each one's run alive, and no earlier holder's result for it.

        An entry whose run is gone is dropped, with any result it never read.
        `lanes.js` has handed that lane to the Opus merger, and taking it as well
        would land it twice (§397.5 item 7).
        """
        everyone = self.everyone()
        assert own["id"] in [entry["id"] for entry in everyone], f"this run's own entry {own['id']} is not in {self.entries}"
        found = []
        for entry in everyone:
            if entry["id"] == own["id"]:
                continue
            if not alive(entry):
                say(f"land: the run that waited to land {entry['branch']}, from {entry['tree']}, is gone, so its "
                    f"entry is dropped and the lane left to the Opus merger")
                self.forget(entry)
            elif not os.path.exists(self.result_path(entry)):
                found.append(entry)
        return found

    def everyone(self):
        """Every entry, oldest first. Read it holding the lock, since a holder removes entries."""
        names = sorted(name for name in os.listdir(self.entries) if name.endswith(".json"))
        return [read_json(os.path.join(self.entries, name)) for name in names]

    def result(self, entry):
        path = self.result_path(entry)
        return read_json(path) if os.path.exists(path) else None

    def leave_result(self, entry, result):
        write_json(self.result_path(entry), result)

    def forget(self, entry):
        os.remove(self.entry_path(entry))
        if os.path.exists(self.result_path(entry)):
            os.remove(self.result_path(entry))

    def entry_path(self, entry):
        return os.path.join(self.entries, f"{entry['id']}.json")

    def result_path(self, entry):
        return os.path.join(self.results, f"{entry['id']}.json")


def alive(entry):
    """Whether the run that wrote `entry` is still going: its pid is there, not a zombie, and started when the entry says."""
    try:
        fields = stat_fields(entry["pid"])
    except OSError:
        return False
    return fields[0] != "Z" and fields[19] == entry["pid_start"]


def stat_fields(pid):
    """`/proc/<pid>/stat` from field 3, the state, on: field 22, the time the process started, is at 19."""
    with open(f"/proc/{pid}/stat", encoding="utf-8") as f:
        return f.read().rsplit(")", 1)[1].split()


def unlock(tree, report):
    rc, said = run("git", "worktree", "unlock", tree)
    if rc == 0 or "is not locked" in said:
        report["unlocked"].append(tree)
    else:
        report["needs_judgement"].append(f"could not unlock {tree}: {last_line(said)}")


def command(stop, *cmd):
    rc, said = run(*cmd)
    if rc != 0:
        raise Stop(stop, f"{' '.join(cmd)} failed: {last_line(said)}")


def output(*cmd):
    rc, said, out = run_split(*cmd)
    if rc != 0:
        raise Stop("git", f"{' '.join(cmd)} failed: {last_line(said)}")
    return out.strip()


def run(*cmd):
    rc, said, _ = run_split(*cmd)
    return rc, said


def run_split(*cmd):
    print("+ " + " ".join(cmd), flush=True)
    result = subprocess.run(cmd, capture_output=True, text=True)
    said = (result.stdout + result.stderr).rstrip()
    if said:
        print(said, flush=True)
    return result.returncode, said, result.stdout


def say(line):
    """Prints a line to the detached run's log, timestamped so a reader of the log alone can tell when it waited (§411.4).

    The lines `follow` and `ended` print directly — `land: still running ...`,
    `land: stopped at ...`, the JSON — go through plain `print`, not `say`, and
    stay bare: `lanes.js` and `lander.md` match them as they stand.
    """
    stamp = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    print(f"{stamp} {line}", flush=True)


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def write(path, text):
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def read_json(path):
    return json.loads(read(path))


def write_json(path, data):
    """Writes through a file beside it and a rename, so a reader never sees half of it."""
    write(path + ".tmp", json.dumps(data, ensure_ascii=False))
    os.replace(path + ".tmp", path)


def last_line(said):
    lines = [line for line in said.splitlines() if line.strip()]
    return lines[-1].strip() if lines else "(it printed nothing)"


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
