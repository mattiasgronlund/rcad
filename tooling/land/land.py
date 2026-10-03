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

Python rather than bash because the tree it runs from is switched under it:
python reads the whole file before it runs, bash reads a script as it goes.
"""

import glob
import json
import os
import re
import subprocess
import sys

MERGE_ATTEMPTS = 3
DOCS_CATCH_UPS = 5
JUST = os.environ.get("LAND_JUST", "just")
ROW = re.compile(r"^[TDQ][0-9]+$")
SHA = re.compile(r"^[0-9a-f]{7,40}$")
STEP_ARGS = {"add": 4, "after": 2, "tag": 2}
USAGE = ("usage: just land [--steps <file>] <row>[,<row>...] <lane branch> <lane sha> "
         "<merge branch> [<tree to unlock and mark finished>...]")


class Stop(Exception):
    def __init__(self, name, why):
        super().__init__(why)
        self.name = name
        self.why = why


def main(argv):
    report = {
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
    try:
        land(argv, report)
    except Stop as stop:
        report["stop"] = stop.name
        report["not_merged_because"] = stop.why
        print(f"land: stopped at {stop.name}: {stop.why}", flush=True)
    print(json.dumps(report, ensure_ascii=False), flush=True)
    return 0 if report["merged"] else 1


def land(argv, report):
    """Unlocks its own tree on every stop as well, last of the unlocks, as `merger.md` step 5 does.

    A stop goes to the Opus merger, which works in a tree of its own, and this
    tree left locked would hold a slot that `just queue next` counts (§263.4).
    """
    top = output("git", "rev-parse", "--show-toplevel")
    report["worktree_path"] = top
    try:
        args = parsed(argv)
        report["merge_branch"] = args["merge_branch"]
        steps = read_steps(args["steps"])
        report["start_sha"] = output("git", "rev-parse", "HEAD")
        refuse_unless_a_clean_linked_tree()
        merge_and_push(args, steps, report)
    finally:
        unlock(top, report)
    finish(args["trees"] + [top], report)


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


def merge_and_push(args, steps, report):
    command("fetch", "git", "fetch", "origin")
    command("switch", "git", "switch", "-c", args["merge_branch"], "origin/main")
    for tree in args["trees"]:
        unlock(tree, report)
    tip = lane_tip(args["branch"], args["sha"])
    merge("conflict", "git", "merge", "--no-ff", "--no-commit", args["branch"])
    refuse_if_section_0_names(args["rows"])
    apply_queue(args["rows"], tip, steps, report)
    command("check-queue", JUST, "check-queue")
    command("commit", "git", "add", "docs/queue.md")
    message = f"Merge {args['branch']}: {', '.join(args['rows'])} done, by just land"
    command("commit", "git", "commit", "-q", "-m", message)
    left = output("git", "status", "--short")
    if left:
        raise Stop("commit", f"git status --short prints something after the merge commit: {left}")
    report["gate_pass_line"] = gate()
    push(report)


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


def apply_queue(rows, tip, steps, report):
    short = output("git", "rev-parse", "--short=8", tip)
    batch = [["done", row, short] for row in rows] + steps
    words = []
    for step in batch:
        words += (["+"] if words else []) + step
    rc, said, out = run_split(JUST, "queue", "apply", *words)
    if rc != 0:
        raise Stop("queue-apply", f"just queue apply refused the batch: {last_line(said)}")
    report["rows_closed"] = [f"{row} done:{short}" for row in rows]
    ids = [line for line in out.splitlines() if ROW.match(line.strip())]
    adds = [step for step in steps if step[0] == "add"]
    if len(ids) != len(adds):
        report["needs_judgement"].append(f"queue apply printed {len(ids)} new ids for {len(adds)} add steps")
    for row_id, (_, tier, model, _write_set, what) in zip(ids, adds):
        report["rows_added"].append(f"{row_id.strip()} tier {tier} {model}: {what[:80]}")


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


def finish(trees, report):
    """`ff-main`, then `mark-finished` on the lane's, the verifier's and this tree, once the push went through."""
    rc, said = run(JUST, "ff-main")
    report["ff_main"] = last_line(said)
    if rc != 0:
        report["needs_judgement"].append(f"ff-main refused, and the tree it would move is not this one: {last_line(said)}")
    rc, said = run(JUST, "mark-finished", *trees)
    if rc != 0:
        report["needs_judgement"].append(f"mark-finished refused, so the run's trees wait out their hour: {last_line(said)}")


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


def last_line(said):
    lines = [line for line in said.splitlines() if line.strip()]
    return lines[-1].strip() if lines else "(it printed nothing)"


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
