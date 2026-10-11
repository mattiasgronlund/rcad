## 429. A long command survives a `/clear` only if no agent shell is its ancestor — 2026-10-11

*Measured by the session dispatch-2026-10-03-ec, after Mattias asked whether
the commands a `/clear` kills could be kept alive. Claude Code 2.1.281.*

### 429.1 What a `/clear` costs

A `/clear` leaves every running agent alive, workflow agents and plain
subagents alike, but kills the command each one has running at that moment. A
count over every rcad transcript on 2026-09-29 (328 `/clear`s since 09-17)
found 97 agent commands in flight at a `/clear`, and 76 came back "Exit code
137" in the same second: 42 of them `just gate`, 66 of them in workflow agents
and 10 in plain ones. A merger whose gate dies runs it again from the start. A
lander whose `just land` dies hands its row to the Opus merger and leaves its
tree locked with an unpushed `merge-q…` branch: six times on 10-06, 10-08 and
10-10 (Q777, Q1337+Q474, Q1591, Q1598, Q1645, Q1646).

Asking for no `/clear` while a gate runs has not held: two of those six came
two minutes after a handoff that said so in bold.

### 429.2 How Claude Code kills a command

Read from the 2.1.281 binary. The Bash tool starts its shell with
`detached: true`, so the shell leads a process group of its own. To stop it,
Claude Code sends SIGTERM to that whole group (`process.kill(-pid, …)`), then
lists every process with `ps -A -o pid= -o ppid=`, walks down from the shell's
pid, and sends the same signal to every descendant it finds. Five seconds
later it sends SIGKILL to the group. That is the pair the transcripts show:
"Recipe `land` was terminated by signal 15", then exit 137.

So a process escapes only if it is in another process group **and** no longer
a descendant of the shell when the kill comes.

### 429.3 What survives

Each child below waits 12 s and then writes a file; the command that started it
was stopped with TaskStop while it waited, and the files were read 15 s later.

| How the child was started | Its file |
|---|---|
| `cmd &` | not written |
| `nohup cmd &` | not written |
| the shell's own foreground | not written |
| Python `Popen(..., start_new_session=True)`, the parent still running | not written |
| `setsid -f cmd`, from a Python parent still running | **written** |
| `setsid -f cmd`, from the shell | **written** |

`setsid -f` forks, and the process in between exits, so the child is adopted by
init: in a session of its own and nobody's descendant.

TaskStop, not a `/clear`, stopped these: a session cannot `/clear` itself.
Both go through the kill in §429.2, and the signals a `/clear` leaves in the
transcripts are that kill's. The rows in §429.5 build tests that stop the
command the same way, and the first `/clear` after they land is the check on
the real thing.

### 429.4 Why `just land` and `just verify-lane` die though they detach

Both start their run in the background already, and say why: the run is to
outlive the agent that started it. `tooling/land/land.py` (the `Popen` before
`follow`) and `tooling/verify-lane/verify_lane.py` start it with
`start_new_session=True` and then wait on it in the same process. While that
process waits, the run is its child, so the walk in §429.2 finds it and kills
it. That is the fourth row of the table. Their `wait` verbs already follow a
run by its pid file, so a run that outlived its starter would be picked up by
the next `just land wait`; today there is never one to pick up.

`just gate` does not detach at all.

### 429.5 What is built

Two rows, `land.py` first, since a killed lander costs the most:

- **Q1670: `land.py` and `verify_lane.py` start their run with a double fork**, as
  `setsid -f` does, so the process that follows it holds no child. The test
  stops the follower as §429.2 does, its group and then every descendant, and
  shows that the run still finishes and that `wait` prints its result. Today's
  code fails it.
- **Q1671: `just gate` runs its steps in a run started the same way**, with its log
  and exit code in files under `target/`, and the call only follows it. A
  `just gate` in a tree whose gate is still running on the same commit follows
  that run rather than starting a second one, so an agent whose gate was killed
  gets the same gate back by running the same command. `gate-lane` execs
  `gate`, so lanes, verifiers and mergers all get it.

`just t` runs are short and stay as they are.

### 429.6 What it costs

A run that no agent is waiting for still runs to its end. When an agent is
stopped on purpose (TaskStop on a run, a lane abandoned) its gate goes on for
up to the nine minutes `gate`'s `[doc]` names, on cores another lane could
use. The auto-mode classifier refuses a `kill`, so ending one early is
Mattias's. A gate that ends by itself costs at most that much, and it costs
less than a gate killed and run again.

### 429.7 What Q1670 built, and where §429.2 and §429.3 disagree

Q1670 is the double fork, as `start_detached()` in both `land.py` and
`verify_lane.py`: fork once for a process in between, that process forks the
run, writes the run's own pid to the pid file, and exits right away; the
caller's `os.waitpid` on it blocks until it has. The run is reparented to
init while that process is exiting, before the lander or the verifier ever
calls `follow()`. There is no moment to catch here: by the time anything
could look, it has already happened.

The killed-follower test, added in an earlier run on this row, did not show
the fix doing anything, because of how it was written, not because the fix
was wrong. It found the follower's descendants *after* sending it SIGTERM —
the order §429.2 gives for Claude Code's own kill — by forking `ps -A` and
`awk`. On this machine that fork is slower than the kernel reparenting an
orphaned child once its parent dies, so by the time the scan ran, even the
old single-forked run already showed ppid 1, the same as the new
double-forked one. The test passed on both the old code and the new, which
told nobody anything.

The fix was to scan before the kill, not after. `stop_like_a_clear()` in both
selftests now calls `descendants_of()` first, sends SIGTERM to the follower's
group and to every pid the scan found, waits, then sends SIGKILL to both. The
old code's run is a direct child of the follower for as long as the follower
is alive, so the scan finds it no matter when it is run — nothing races here
either, because the thing being looked for does not go away on its own. The
new code's run has already been reparented to init before the follower calls
`follow()`, so the scan never finds it, at any point before or after the
kill. Checked both ways: with the old `land.py` and the old `verify_lane.py`
swapped back in, each selftest's killed-follower arm fails, with `follow()`'s
own line `the run (pid …) ended without finishing`; with the fix back in,
both pass.

That leaves §429.2 and §429.3 disagreeing, and nothing here settles it. Done
in §429.2's order, the scan lost to the reparenting every time, for the old
code as well as the new; yet §429.3's table, taken against the real kill,
found the old shape's child killed. Either Claude Code lists the processes
before its SIGTERM takes effect, not after as §429.2 reads the binary, or its
walk is fast enough to win where a shell's lost. Neither is measured. The
test does not hang on which: a run that is not the follower's descendant
before any signal never becomes one, so a scan taken first, which finds every
descendant there is at that moment, misses the run exactly when Claude Code's
walk would, whenever that walk runs.

The first real `/clear` after this lands is still the check §429.3 names:
whether a `just land` or `just verify-lane` run outlives it. Nothing here
changes what that check looks for, only what the selftest can tell about it
ahead of time.

### 429.8 What Q1671 built: a gate that outlives its call

`just gate` now runs its steps in a run of their own, and the call only
follows it. The call takes a lock in `<gate_log>.run/`, decides whether to
follow a run, refuse, or start one, refuses a dirty tree as before (§415), and
then starts the run as `( setsid bash <copy of the recipe's script> & )`. The
subshell exits as soon as it has written the run's pid, so the run belongs to
init and leads a session of its own before the call starts following it. The
run is the old recipe from the emptying of the log onward, unchanged. Its
stdout and stderr go to `<gate_log>.run/out`, and the last thing it does is
write its exit code to `<gate_log>.run/exit`. The call prints `out` as it
grows, checking every 0.2 s, and exits with the code in `exit`.

What a reader sees is the same: `quiet`'s one line a step, the failing step's
tail and the summary block, `loud`'s stream, the exit code, the served
verdicts, the record, the audit and `fresh`. That holds because the run is the
old code, and `check-gate-summary-selftest` still passes. One thing did change:
the run's stderr now reaches the caller on stdout, folded into the one file. A
refusal still goes to stderr, since the call prints it before any run starts.

The files sit beside `gate_log`. Every selftest sets a `gate_log` of its own,
so its runs never meet the real gate's. The run is known by its pid and its
start, which is the boot id plus the start time from `/proc/<pid>/stat`. So a
pid used again after a reboot, or after the pids wrap, is not taken for the
run, and a zombie counts as ended.

**The same gate.** Two calls are the same gate when they agree on six things:
the commit, the steps, the steps that can be served, `reuse` or `fresh`,
`quiet` or `loud`, and how many steps run at once. A call that follows a run
takes the run's answer as its own, so the run has to be answering the same
question:

- A run on another commit read another tree.
- A `gate-lane` run leaves `test-slow` out. A merger's whole gate that
  followed it would put code on `main` that the slow tier never read.
- A `reuse` run serves steps that `fresh` asked to have run.
- The run prints in one form only, so `quiet` cannot follow `loud`.
- `1` is asked for to rule out steps getting in each other's way (§372.8),
  and a run of all of them at once cannot answer that.

A call that is not the same gate as a live run exits 2. It names that run's
pid, its start, and the six lines that make it that gate. It does not wait:
two gates in one tree would share `target/`, the cargo lock and the step logs,
and nothing asks for that. The check for a run to follow comes before the
dirty-tree refusal, since following reads nothing of the tree, and a run can
write into the tree, as a failing proptest does.

**A run that died.** Say a run's pid is gone and it wrote no exit code,
because it was killed outright or the machine rebooted. The next call of the
same gate prints whatever output the run left, then `just gate: the run (pid
N) ended without finishing; <gate_log> holds what it ran`, writes `lost`, and
exits 1. A third call finds `lost` and starts a new run, which passes if its
steps do. A dead run of another gate is not reported: the call just starts
afresh.

**A Ctrl-C.** A Ctrl-C in a terminal reaches the call but not the run, which
is in a session of its own. So the call passes it on as SIGTERM to the run's
whole process group, writes `stopped` so that the next call starts afresh
instead of reporting a dead run, and exits 130.

The call sends SIGTERM and not SIGINT because the run is a background job of
the call's shell, and a background job of a script starts with SIGINT
ignored. Measured with `target/probe-env.sh`:

- `/proc/<pid>/status` showed `SigIgn: 0000000000000006`, which is SIGINT and
  SIGQUIT, for a plain `sleep 3 &` started from a script.
- A bash started that way did not run its own `trap … INT`.
- Under `env --default-signal=INT`, the same bash did run it.

The same is true of every step of the old gate, since each one was a
background job of the recipe's shell. Whether a Ctrl-C stopped all of the
steps before this change was not run. SIGTERM is not trapped, because SIGTERM
is what §429.2's kill sends.

**The test.** `check-gate-verdicts-selftest` has three arms for this. They use
a step that waits for as long as a file is there (`_gate-selftest-sleep` with
`GATE_SELFTEST_HOLD`). Each arm starts the gate in a group of its own, with
SIGINT at its default, as a terminal or Claude Code would start it.

- **Stopped like a `/clear`.** The arm stops the call as `stop_like_a_clear()`
  does, finding the descendants before the first signal (§429.7), then runs
  the same gate again. It checks that the second call follows the run, and
  that a gate of other steps and a `fresh` gate both refuse beside it. Once
  the step is let go, it checks that the follower prints the step's line and
  the summary block and exits 0, and that the step ran only once.
- **Killed outright.** The arm kills the run outright, then checks the report
  of the second call and the new run of the third.
- **A Ctrl-C.** The arm sends SIGINT to the call's group and checks that the
  run is gone.

Two checks show that the arms catch what they are for:

- With the base's `gate` recipe put back and the arms kept, the first arm
  fails with `the same gate run again on the same commit did not follow the
  run whose call was stopped`.
- With the follower's `trap … INT` line taken out, the third arm fails with
  `a Ctrl-C to the gate's call left its run going`.

**§429.2 and §429.3.** Nothing measured here disagrees with them. The first
arm agrees with §429.3's `setsid -f` row. Because it finds the descendants
before the kill, it says nothing about the order §429.7 left open.
