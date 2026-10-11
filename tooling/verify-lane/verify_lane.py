#!/usr/bin/env python3
"""`just verify-lane`: a verifier's fixed steps, run for it, with the reading left to it.

`docs/decisions.md` §383.4 item 4 is the spec and §385 the build. In order: unlock
the lane's tree, `git log` from the base to the branch, the files outside the
write set, `lines-that-run` before the checkout, the checkout, `gate-lane`,
which leaves out the slow tier (§397.4), one run of each named test that
reaches that tier, then `verifier.md` step 7's revert of the source paths, the
test run and the restore, and last the lane's numbers. For a lane marked
`sameBehaviour` the revert's tests must pass rather than fail (§399). It ends
with one line of JSON holding every `verdictSchema` field it could fill, `skipped` for what it
skipped and why, and `left_to_you` for what it could not decide.

**One run, detached, that the verifier waits on** (§385.2). The harness ends a
Bash call at 600 s, and a cold gate alone can take nine minutes, before the
revert's rebuild and every number's command. So `just verify-lane <input file>`
starts the run in a session of its own and follows its log for at most
`VERIFY_LANE_WAIT` seconds, 540 by default. A run still going says so on its
last line, and `just verify-lane wait` follows it again from where the last
call stopped.

Python rather than bash, as `tooling/land/land.py` is, because the run checks
the branch out under itself: python reads the whole file before it starts, so
the copy that judges the branch is the one started before the checkout.

The input is a file the verifier writes with the Write tool, as the lander's
steps are (§384.2): one word to a line, a `+` line between records, each
record's first line its name. `lanes.js` writes the text into the verifier's
brief, so no word ever passes through a shell.
"""

import fnmatch
import json
import os
import re
import signal
import subprocess
import sys
import time

JUST = os.environ.get("VERIFY_LANE_JUST", "just")
WAIT = float(os.environ.get("VERIFY_LANE_WAIT", "540"))
NUMBER_TIMEOUT = 600
STATE = os.path.join("target", "verify-lane")
SHOWN = 12
SHA = re.compile(r"^[0-9a-f]{7,40}$")
NEXTEST_NAME = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*$")
TEST_PATH = re.compile(r"^(crates/[^/]+/tests/|fuzz/corpus/)")
COLOUR = re.compile(r"\x1b\[[0-9;]*m")
WRITES = re.compile(
    r"\b(claim-section|mark-finished|prune-worktrees|release-lanes|ff-main|clean-targets)\b"
    r"|\bjust\s+land\b"
    r"|\bqueue\s+(take|add|done|drop|park|after|rank|tier|tag|pick|pin|unpin|apply)\b"
    r"|\bgit\s+(commit|push|switch|checkout|reset|stash|worktree|merge|rebase|restore|clean|am|cherry-pick|revert|tag)\b"
    r"|(^|[;&|(]\s*)rm\s")
RECORDS = {
    "base": (1, 1),
    "branch": (1, 1),
    "lane-tree": (1, 1),
    "write-set": (1, None),
    "source-paths": (0, None),
    "pins-only": (0, 0),
    "same-behaviour": (0, 0),
    "tests": (1, None),
    "number": (3, 3),
}
REQUIRED = ("base", "branch", "lane-tree", "write-set")
USAGE = ("usage: just verify-lane <input file>, then just verify-lane wait for as long as its last line "
         "says it is still running")


class Refusal(Exception):
    pass


def main(argv):
    try:
        if argv == ["wait"]:
            return follow(None)
        if len(argv) == 2 and argv[0] == "--detached-run":
            return detached_run(argv[1])
        if len(argv) == 1 and not argv[0].startswith("-"):
            return start(argv[0])
        raise Refusal(USAGE)
    except Refusal as refusal:
        print(f"verify-lane: {refusal}")
        return 2


def start(path):
    read_input(path)
    top = toplevel()
    refuse_unless_a_clean_linked_tree()
    state = os.path.join(top, STATE)
    if os.path.exists(state):
        print(f"verify-lane: {state} is there already, so this tree has started a run; "
              f"`mise exec -- just verify-lane wait` follows it or prints its result again")
        return 2
    os.makedirs(state)
    with open(os.path.join(state, "printed"), "w", encoding="utf-8") as f:
        f.write("0")
    with open(os.path.join(state, "log"), "w", encoding="utf-8") as log:
        child = subprocess.Popen(
            [sys.executable, os.path.abspath(__file__), "--detached-run", os.path.abspath(path)],
            cwd=top, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    with open(os.path.join(state, "pid"), "w", encoding="utf-8") as f:
        f.write(f"{child.pid}\n")
    return follow(child)


def follow(child):
    """Print what the run logged since the last call, until it ends or this call's time is up.

    Exits 0 once the run has ended, with its JSON as the last line, and 3 while
    it is still going.
    """
    state = os.path.join(toplevel(), STATE)
    try:
        with open(os.path.join(state, "pid"), encoding="utf-8") as f:
            pid = int(f.read())
    except (OSError, ValueError):
        print(f"verify-lane: no run has started in this tree. {USAGE}")
        return 2
    ended = os.path.join(state, "exit")
    deadline = time.monotonic() + WAIT
    printed = False
    while True:
        done = os.path.exists(ended)
        printed = print_new(state) or printed
        if done:
            if not printed:
                print(last_line(read(os.path.join(state, "log"))))
            return int(read(ended).strip() or 1)
        if not alive(pid, child):
            if os.path.exists(ended):
                continue
            print(f"verify-lane: the run (pid {pid}) ended without finishing; "
                  f"{os.path.join(state, 'full.log')} holds what it ran")
            return 1
        if time.monotonic() >= deadline:
            print(f"verify-lane: still running (pid {pid}). Run `mise exec -- just verify-lane wait` to keep "
                  f"waiting; with no verify-lane recipe in this tree, wait with "
                  f"`timeout 540 tail --pid={pid} -f /dev/null` and Read {os.path.join(state, 'log')}")
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
    with open(at_path, "w", encoding="utf-8") as f:
        f.write(str(at + cut))
    return True


def alive(pid, child):
    if child is not None:
        return child.poll() is None
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8") as f:
            return f.read().rsplit(")", 1)[1].split()[0] != "Z"
    except (OSError, IndexError):
        return False


def detached_run(path):
    """The run itself, in a session of its own, writing the log `follow` prints.

    Its last line is the JSON, whatever happened, and the `exit` file it writes
    after that is what tells `follow` it is over.
    """
    signal.signal(signal.SIGHUP, signal.SIG_IGN)
    state = os.path.join(toplevel(), STATE)
    report = {"worktree_path": toplevel()}
    code = 1
    try:
        verify(read_input(path), report, os.path.join(state, "full.log"))
        code = 0
    except Exception as error:  # noqa: BLE001 -- whatever broke, the verifier gets the JSON and the reason
        say(f"verify-lane: stopped by {type(error).__name__}: {error}")
        left(report, f"the recipe stopped by {type(error).__name__}: {error}; every step after the last one "
                     f"it printed is yours")
    for own in ("skipped", "left_to_you"):
        if own in report:
            report[own] = report.pop(own)
    say("=== verify-lane: done ===")
    print(json.dumps(report, ensure_ascii=False), flush=True)
    with open(os.path.join(state, "exit"), "w", encoding="utf-8") as f:
        f.write(f"{code}\n")
    return code


def verify(lane, report, full_log):
    base, branch = lane["base"], lane["branch"]
    run = Runner(full_log)
    say(f"verify-lane: {branch} against the base {base[:12]}, in {report['worktree_path']}")
    unlock(run, lane["lane-tree"], report)
    tip = branch_on_base(run, base, branch, report)
    if tip is None:
        return
    files_outside(run, lane, tip, report)
    count = lines_that_run(run, base, branch, report)
    rc, said = run.git("checkout", "--detach", tip)
    if rc != 0:
        left(report, f"git checkout --detach {tip} failed, so nothing after it ran: {last_line(said)}")
        return
    say(f"checked out {tip}")
    passed, left_out_slow = gate(run, report)
    if lane["tests"]:
        kinds = {name: test_kind(run, name) for name in lane["tests"]}
        tests_pass(run, kinds, passed, left_out_slow, report)
        revert(run, lane, kinds, count, tip, report)
    numbers(run, lane["numbers"], report)


def unlock(run, tree, report):
    rc, said = run.git("worktree", "unlock", tree)
    if rc == 0:
        say(f"unlocked the lane's tree, {tree}")
    elif "is not locked" in said:
        say(f"the lane's tree, {tree}, was not locked")
    else:
        left(report, f"could not unlock the lane's tree, {tree}: {last_line(said)}")


def branch_on_base(run, base, branch, report):
    """The branch's tip when the branch exists and sits on the base, else None and the reason."""
    rc, tip = run.git("rev-parse", "--verify", "--quiet", f"{branch}^{{commit}}")
    if rc != 0:
        report["branch_ok"] = False
        left(report, f"{branch} is not a branch here, so nothing after this ran")
        return None
    report["branch_ok"] = run.git("merge-base", "--is-ancestor", base, tip)[0] == 0
    _, log = run.git("log", "--oneline", f"{base}..{tip}")
    commits = log.splitlines()
    say(f"git log --oneline {base[:12]}..{branch}: {len(commits)} commit(s)")
    for line in commits[:SHOWN]:
        say(f"  {line}")
    if len(commits) > SHOWN:
        say(f"  ... and {len(commits) - SHOWN} more")
    if not report["branch_ok"]:
        left(report, f"{branch} does not sit on the base {base}, so its claims are about another tree "
                     f"and nothing after this ran")
        return None
    return tip


def files_outside(run, lane, tip, report):
    _, names = run.git("diff", "--name-only", "--no-renames", lane["base"], tip)
    outside = [name for name in names.splitlines() if name and not in_write_set(name, lane["write-set"])]
    report["files_outside_write_set"] = outside
    say(f"files outside the write set: {', '.join(outside) if outside else 'none'}")


def in_write_set(path, entries):
    """Whether `path` sits under one of `entries`.

    An entry can name one item of a file, `crates/x.rs#some_fn`; only the
    part before `#` is a path, so that is what is matched. Whether the diff
    actually stays inside `some_fn` is not this function's to see — it reads
    paths, not items — so that reading is left to the verifier.
    """
    for entry in entries:
        trimmed = entry.split("#", 1)[0].rstrip("/")
        if path == trimmed or path.startswith(trimmed + "/"):
            return True
        if any(c in trimmed for c in "*?[") and (fnmatch.fnmatch(path, trimmed) or fnmatch.fnmatch(path, trimmed + "/*")):
            return True
    return False


def lines_that_run(run, base, branch, report):
    """`verifier.md` step 1's print, taken before the checkout so the recipe that judges is not the branch's.

    Returns the count it ends with, or None when it ends in none.
    """
    report["lines_that_run_command"] = f"mise exec -- just lines-that-run {base} {branch}"
    _, said = run(JUST, "lines-that-run", base, branch)
    report["lines_that_run"] = said
    say(f"{report['lines_that_run_command']}:")
    for line in said.splitlines():
        say(f"  {line}")
    found = re.fullmatch(r"lines that run: ([0-9]+)", last_line(said))
    if not found:
        left(report, "the lines-that-run print does not end in its count, lines that run: N, so no revert ran")
        return None
    return int(found.group(1))


def gate(run, report):
    """`gate-lane` on the branch tip, or `gate quiet` where the branch has no `gate-lane`.

    The second is a lane carried on from a base older than §397.4. Relays all
    but the one line each passing step prints, and returns whether the gate
    passed and whether it left the slow tier out.
    """
    left_out_slow = run(JUST, "--show", "gate-lane")[0] == 0
    rc, said = run(JUST, "gate-lane") if left_out_slow else run(JUST, "gate", "quiet")
    lines = said.splitlines()
    for line in lines:
        if not re.fullmatch(r"· \S+ \([0-9.]+ s\)", line):
            say(line)
    at = next((i for i, line in enumerate(lines) if line == "=== gate summary ==="), None)
    if at is None:
        left(report, f"the gate exited {rc} and printed no summary block; its output is in the log above")
        return False, left_out_slow
    report["gate_summary"] = "\n".join(lines[at:])
    return rc == 0 and "gate: PASS" in lines[at:], left_out_slow


def test_kind(run, name):
    """'recipe' for a recipe of this justfile, 'nextest' for a name `just t` can filter on, else None."""
    if run(JUST, "--show", name)[0] == 0:
        return "recipe"
    if NEXTEST_NAME.match(name):
        return "nextest"
    return None


def tests_pass(run, kinds, passed, left_out_slow, report):
    """`verifier.md` step 6: the gate is the proof for a test name and for a recipe it runs.

    Two things the gate never ran run here, once each: a named recipe outside
    `gate_steps`, and, when the gate left the slow tier out, a named test whose
    name is part of a slow test's. `just t` filters on part of a name, so the
    second would select a slow test.
    """
    unknown = [name for name, kind in kinds.items() if kind is None]
    for name in unknown:
        left(report, f"'{name}' is neither a recipe nor a test name `just t` can filter on, so the recipe "
                     f"ran nothing for it")
    if not passed:
        left(report, "the gate did not pass, so whether the named tests pass is yours to read from its summary")
        return
    _, steps = run(JUST, "--evaluate", "gate_steps")
    outside_gate = [name for name, kind in kinds.items() if kind == "recipe" and name not in steps.split()]
    slow = []
    if left_out_slow:
        _, tier = run(JUST, "--evaluate", "slow_tests")
        slow_names = [token.rsplit("/", 1)[-1] for token in tier.split()]
        slow = [name for name, kind in kinds.items()
                if kind == "nextest" and any(name in slow_name for slow_name in slow_names)]
    failed = []
    for name in outside_gate:
        rc, said = run(JUST, name)
        say(f"{name}, which the gate does not run: {'passed' if rc == 0 else f'FAILED, exit {rc}'} on the branch")
        if rc != 0:
            failed.append(name)
            tail(said)
    for name in slow:
        rc, said = run(JUST, "t", name)
        say(f"{name}, which reaches the slow tier gate-lane leaves out: "
            f"{'passed' if rc == 0 else f'FAILED, exit {rc}'} on the branch")
        if rc != 0:
            failed.append(name)
            tail(said)
    if failed:
        report["tests_pass_on_branch"] = False
    elif not unknown:
        report["tests_pass_on_branch"] = True
        ran_once = outside_gate + slow
        say("tests_pass_on_branch: true, from the gate's PASS line" +
            (f" and {', '.join(ran_once)} run once" if ran_once else ""))


def revert(run, lane, kinds, count, tip, report):
    """`verifier.md` step 7: the source paths back to the base, the named tests run, and the branch's put back."""
    sources = lane["source-paths"]
    if lane["pins-only"]:
        skip(report, "the revert: the lane is pinsOnly, so there is nothing to revert")
        report["tests_fail_without_fix"] = False
        return
    if count is None:
        return
    if count == 0:
        skip(report, "the revert: lines that run: 0, so a revert takes out nothing a test could see (§375)")
        if not lane["same-behaviour"]:
            report["tests_fail_without_fix"] = False
        return
    if not sources:
        left(report, "the lane declared no source paths and is not pinsOnly, so there was nothing to revert; "
                     "lanes.js refuses that as the lane's gap")
        return
    tests_inside = test_code_changed(run, lane["base"], tip, sources)
    if tests_inside:
        left(report, f"the source paths hold test code the branch changed, so a revert would take the tests out "
                     f"too: revert by hand, keeping the new tests (verifier.md step 7). The first: "
                     f"{'; '.join(tests_inside[:3])}")
        return
    runnable = [name for name, kind in kinds.items() if kind is not None]
    if not runnable:
        return
    rc, said = run.git("restore", f"--source={lane['base']}", "--worktree", "--", *sources)
    if rc != 0:
        left(report, f"the revert of {' '.join(sources)} failed, so no test ran against it: {last_line(said)}")
        return
    say(f"reverted {' '.join(sources)} to the base")
    outcomes = {}
    try:
        for name in runnable:
            outcomes[name] = reverted_outcome(run, name, kinds[name])
    finally:
        put_back(run, sources, tip, report)
    if lane["same-behaviour"]:
        same_answers(kinds, outcomes, report)
        return
    if len(runnable) == len(kinds) and all(o == "failed" for o in outcomes.values()):
        report["tests_fail_without_fix"] = True
    elif all(o == "passed" for o in outcomes.values()):
        report["tests_fail_without_fix"] = False
    else:
        left(report, "the tests with the source reverted were not all seen to fail: "
                     + "; ".join(f"{name} {outcome}" for name, outcome in outcomes.items())
                     + ". A test the revert took out, or one that did not build, is not a failure")


def same_answers(kinds, outcomes, report):
    """A `sameBehaviour` lane's revert: the base computed what the branch does, so every test must pass on it (§399)."""
    if len(outcomes) == len(kinds) and all(o == "passed" for o in outcomes.values()):
        report["tests_pass_without_change"] = True
        return
    failed = any(o == "failed" for o in outcomes.values())
    if failed:
        report["tests_pass_without_change"] = False
    left(report, "the tests with the source reverted were not all seen to pass: "
                 + "; ".join(f"{name} {outcome}" for name, outcome in outcomes.items())
                 + (". One that failed says the base answered differently, so the behaviour changed" if failed
                    else ". A test the revert took out, or one that did not build, is not a pass"))


def reverted_outcome(run, name, kind):
    """'failed' only for a test that ran and failed: Q231's verifier read "no tests to run" as a failure."""
    if kind == "recipe":
        if run(JUST, "--show", name)[0] != 0:
            outcome, rc, said = "was taken out by the revert", None, ""
        else:
            rc, said = run(JUST, name)
            outcome = "passed" if rc == 0 else "failed"
    else:
        rc, said = run(JUST, "t", name)
        outcome = {0: "passed", 100: "failed", 4: "ran no test", 101: "did not build"}.get(rc, f"ended with exit {rc}")
    say(f"{name}, with the source reverted: {outcome}" + (f" (exit {rc})" if rc is not None else ""))
    if outcome != "passed":
        tail(said)
    return outcome


def put_back(run, sources, tip, report):
    rc, said = run.git("restore", f"--source={tip}", "--worktree", "--", *sources)
    _, status = run.git("status", "--porcelain")
    _, head = run.git("rev-parse", "HEAD")
    if rc != 0 or status or head != tip:
        say(f"!!! the tree is not back at {tip}: {last_line(said)} {status}")
        left(report, f"the restore did not put the tree back at the branch tip {tip}; git status --porcelain "
                     f"says: {status or '(nothing)'}, HEAD is {head}")
    else:
        say(f"put {' '.join(sources)} back as the branch has them")


def test_code_changed(run, base, tip, sources):
    """The changed lines under `sources` that `lines-that-run` would skip as test code.

    The same three rules as that recipe's awk: a file under `crates/*/tests/`
    or `fuzz/corpus/`, the items after a column-0 `#[cfg(test)]` in a `.rs`
    file, and the bodies of the justfile's `check-*` recipes.
    """
    _, diff = run.git("diff", "--no-color", "--no-ext-diff", "-M", "--src-prefix=a/", "--dst-prefix=b/",
                      "-U1000000", base, tip, "--", *sources)
    found = []
    side = None
    for raw in diff.split("\n"):
        if raw.startswith("diff --git "):
            words = raw.split()
            a, b = words[2][2:], words[3][2:]
            kind = "rust" if b.endswith(".rs") else "just" if b == "justfile" or b.endswith("/justfile") else None
            side = Side(b, bool(TEST_PATH.match(a) and TEST_PATH.match(b)), kind)
            continue
        if side is None:
            continue
        if raw.startswith("@@"):
            side.hunk = True
            continue
        if not side.hunk or not raw or raw.startswith("\\"):
            continue
        mark, code = raw[0], raw[1:]
        test = [False, False]
        for s in (0, 1):
            if mark != ("-" if s else "+"):
                test[s] = side.test(s, code)
        if mark != " " and (side.whole or test[mark == "+"]):
            found.append(f"{side.name}: {mark}{code.strip()}")
    return found


class Side:
    """One file of the diff, read as `lines-that-run` reads it: once as the base has it, once as the branch has it."""

    def __init__(self, name, whole, kind):
        self.name = name
        self.whole = whole
        self.kind = kind
        self.hunk = False
        self.pending = [False, False]
        self.in_test = [False, False]
        self.in_check = [False, False]

    def test(self, s, code):
        if self.kind == "rust":
            return self.rust_test(s, code)
        if self.kind == "just":
            return self.recipe_test(s, code)
        return False

    def rust_test(self, s, code):
        if self.in_test[s]:
            if code.startswith("}"):
                self.in_test[s] = False
            return True
        if re.match(r"([ \t]|$)", code):
            return False
        if re.fullmatch(r"#\[cfg\(test\)\][ \t]*", code):
            self.pending[s] = True
            return False
        if re.match(r"(//|#\[)", code):
            return False
        if self.pending[s]:
            self.pending[s] = False
            self.in_test[s] = bool(re.search(r"\{[ \t]*$", code))
            return True
        return False

    def recipe_test(self, s, code):
        if re.match(r"([ \t]|$)", code):
            return self.in_check[s]
        self.in_check[s] = bool(re.match(r"@?check-[A-Za-z0-9_-]+[^:]*:([^=]|$)", code))
        return False


def numbers(run, lane_numbers, report):
    """Each number's command, run on the branch tip, and whether its `value` is in what it printed.

    A value the print does not hold goes in `number_mismatches` for the
    verifier to read. A command that writes to the repository or the queue is
    not run at all.
    """
    report["number_mismatches"] = []
    for i, (what, value, command) in enumerate(lane_numbers, start=1):
        writes = WRITES.search(command)
        if writes:
            say(f"number {i}, {what}: not run, since `{command}` writes ({writes.group(0).strip()})")
            left(report, f"number {i}, {what}: not run, since its command writes; re-run it by hand only if "
                         f"that is safe: {command}")
            continue
        rc, said = run("bash", "-c", command, timeout=NUMBER_TIMEOUT)
        if rc is None:
            say(f"number {i}, {what}: `{command}` did not end within {NUMBER_TIMEOUT} s")
            left(report, f"number {i}, {what}: its command did not end within {NUMBER_TIMEOUT} s: {command}")
            continue
        plain = COLOUR.sub("", said)
        if value in plain or squeezed(value) in squeezed(plain):
            say(f"number {i}, {what}: '{value}' is in what `{command}` printed (exit {rc})")
            continue
        say(f"number {i}, {what}: '{value}' is NOT in what `{command}` printed (exit {rc}); its last lines:")
        tail(plain, 5)
        report["number_mismatches"].append(
            f"{what}: '{value}' is not in what `{command}` printed (exit {rc}); its last line: {last_line(plain)}")


def squeezed(text):
    return " ".join(text.split())


class Runner:
    """Runs a command in a session of its own, keeps everything it printed in `full.log`, and hands back its output.

    A session of its own so that a number's command can be killed whole at its
    timeout, children and all.
    """

    def __init__(self, full_log):
        self.full_log = full_log

    def __call__(self, *cmd, timeout=None):
        proc = subprocess.Popen(cmd, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                text=True, errors="replace", start_new_session=True)
        try:
            said, _ = proc.communicate(timeout=timeout)
            rc = proc.returncode
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            said, _ = proc.communicate()
            rc = None
        with open(self.full_log, "a", encoding="utf-8") as f:
            f.write(f"+ {' '.join(cmd)}\n{said}\n[exit {rc}]\n")
        return rc, said.rstrip("\n")

    def git(self, *args):
        rc, said = self("git", *args)
        return rc, said.strip()


def read_input(path):
    """The verifier's input file: records split by `+` lines, each a name and then its words, one to a line."""
    try:
        lines = [line.rstrip() for line in read(path).split("\n")]
    except OSError as error:
        raise Refusal(f"cannot read the input file {path}: {error}")
    while lines and lines[-1] == "":
        lines.pop()
    records = [[]]
    for line in lines:
        if line == "+":
            records.append([])
        else:
            records[-1].append(line)
    lane = {"source-paths": [], "pins-only": False, "same-behaviour": False, "tests": [], "numbers": []}
    seen = set()
    for number, record in enumerate(records, start=1):
        if not record or "" in record:
            raise Refusal(f"record {number} of {path} is empty or holds a blank line: '{' '.join(record)}'")
        name, words = record[0], record[1:]
        if name not in RECORDS:
            raise Refusal(f"record {number} of {path} is '{name}', and the records are {', '.join(RECORDS)}")
        least, most = RECORDS[name]
        if len(words) < least or (most is not None and len(words) > most):
            wanted = f"{least}" if least == most else f"at least {least}"
            raise Refusal(f"record {number} of {path}, {name}, has {len(words)} words after its name, one to a "
                          f"line, and takes {wanted}")
        if name == "number":
            lane["numbers"].append(words)
            continue
        if name in seen:
            raise Refusal(f"{path} has the record {name} twice")
        seen.add(name)
        if name in ("pins-only", "same-behaviour"):
            lane[name] = True
        elif most == 1:
            lane[name] = words[0]
        else:
            lane[name] = words
    missing = [name for name in REQUIRED if name not in seen]
    if missing:
        raise Refusal(f"{path} has no {', '.join(missing)} record. {USAGE}")
    if lane["pins-only"] and lane["same-behaviour"]:
        raise Refusal(f"{path} marks the lane both pinsOnly and sameBehaviour, and lanes.js refuses that (§399)")
    if not SHA.match(lane["base"]):
        raise Refusal(f"the base '{lane['base']}' in {path} is not a sha: 7 to 40 lowercase hex digits")
    return lane


def refuse_unless_a_clean_linked_tree():
    rc, own = git_quiet("rev-parse", "--absolute-git-dir")
    _, common = git_quiet("rev-parse", "--path-format=absolute", "--git-common-dir")
    if rc != 0:
        raise Refusal(f"this is not a git tree: {own}")
    if own == common:
        raise Refusal("this is the repository's main working tree, the hub, where nothing happens (§60); "
                      "run just verify-lane from a worktree of its own")
    _, status = git_quiet("status", "--porcelain")
    if status:
        raise Refusal(f"this tree has uncommitted changes, and just verify-lane checks the branch out: {status}")


def toplevel():
    rc, top = git_quiet("rev-parse", "--show-toplevel")
    if rc != 0:
        raise Refusal(f"this is not a git tree: {top}")
    return top


def git_quiet(*args):
    result = subprocess.run(["git", *args], capture_output=True, text=True)
    return result.returncode, (result.stdout + result.stderr).strip()


def say(line):
    print(line, flush=True)


def tail(said, lines=SHOWN):
    for line in [line for line in said.splitlines() if line.strip()][-lines:]:
        say(f"  | {line}")


def skip(report, why):
    report.setdefault("skipped", []).append(why)
    say(f"skipped {why}")


def left(report, what):
    report.setdefault("left_to_you", []).append(what)
    say(f"left to you: {what}")


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


def last_line(said):
    lines = [line for line in said.splitlines() if line.strip()]
    return lines[-1].strip() if lines else "(it printed nothing)"


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
