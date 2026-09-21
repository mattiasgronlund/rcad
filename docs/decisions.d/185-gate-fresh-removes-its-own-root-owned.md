## 185. `gate-fresh` removes its own root-owned leftover — 2026-09-21

*§180.4 gave the container a `chown -R` before it exits, so a normal run hands
the tree back host-owned and the trap's plain `rm -rf` clears it. That leaves
one case open, which §180.4's own text names and does not close: a container
that dies before reaching its `chown` — killed, OOM, a docker error — still
writes root-owned files, and the trap could only print the `docker run … rm
-rf` command that would remove them, never run it. That leaked roughly 50 GB
of clones under `/tmp/tmp.*/rcad` before this was noticed. The trap now runs
that command itself.*

**What leaked, and why the printed command was not enough.** The container is
root and `target/` is root-owned the moment anything is compiled into it; the
host user's `chmod -R u+w` cannot grant itself a permission it does not have,
so the plain `rm -rf "$work"` failed exactly the way §180.4 documented, and
the trap's answer was a message on stderr naming the tree and the command that
clears it — never a command run. A message is not a cleanup: nothing acted on
it unless a person read that one run's log and typed the command by hand, and
most runs of a background recipe are never read that closely. Every died
container left its tree exactly where the trap complained about it.

**What the trap does now.** `cleanup()` tries `rm -rf "$work"` as before; only
when that fails does it reach for docker — the same daemon and the same image
the recipe already required and already pulled, so this costs no extra
minutes — `docker run --rm -v "$work:/w" '{{ image }}' rm -rf /w/rcad`, root
inside the throwaway container same as the gate's own, then retries the plain
`rm -rf`. Only if that second attempt also fails does it fall back to the old
behaviour: name the tree and the command, on stderr, and leave. The exit code
is `$rc`, captured before any of this runs, and nothing in the fallback path
touches it — a clean-up that cannot finish must still not fail the gate, same
as §180.4.

**The test.** `check-gate-fresh-cleanup-selftest`, wired into `gate_steps`
beside `check-gate-fresh-selftest`. Real root is not available to a selftest,
so a marker file stands in for a root-owned leftover: the fake `docker` planted
on `PATH` for the main run drops a `.root-owned` file inside the mounted tree
instead of exiting cleanly, and a fake `rm`, also on `PATH`, refuses to remove
anything that still carries that marker — the same shape of refusal the host's
real `rm` hits against a file it does not own, even after `chmod -R u+w` has
already run. The fallback `docker run … rm -rf /w/rcad` call is told apart from
the main run by its own `-v ...:/w` mount, and it clears the marker with the
real `rm`, exactly as a container really would, being root inside too.
Everything the fake `rm` does not recognise passes straight through to the
real one, so the shim cannot break anything outside the one case it models.

Two arms: the fallback docker succeeds (tree gone, exit code is the died
container's own status, not swallowed or overwritten) and the fallback docker
also fails (tree still there, the "left behind" message printed, exit code
still the same). Run against the pre-fix `cleanup()` first, arm one failed with
"the fallback docker was never called" — and the run left a real directory
behind under `/tmp`, the same leak this section is about, which was then
cleaned up by hand. Run against the fix, both arms pass.

`check-gate-runs-every-check`'s count moved 42 → 43 for the new `check-*`
recipe, in the same commit.

Commands this section was written from: `just decision 180`, `180.4`; `just
recipe gate-fresh`, `check-gate-fresh-selftest`,
`check-gate-fresh-cleanup-selftest`, `check-gate-runs-every-check`.
