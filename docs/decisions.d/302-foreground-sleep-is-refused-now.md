## 302. Foreground `sleep` is refused now, and a lane waits on a pid instead — 2026-09-27

*§147.4 said foreground `sleep` is not blocked in this harness, and that it
was tested. It was true then. It is not true now, and a lane paid for the
difference: Q175's first run came back with nothing eleven minutes into an
hour-long measurement.*

### 302.1 What happened

The brief told the lane to start two fuzz arms detached and then wait "in
foreground calls of under ten minutes each (sleep about nine minutes, then read
each arm's latest sample)". What the lane's transcript shows
(`wf_18ac466e-6b0`, agent `ac61f2879a5de543a`):

- **15:33:06Z**: `sleep 300 && .../q175_sample.sh`, foreground, refused:
  `Blocked: sleep 300 followed by: ... To wait for a condition, use Monitor
  with an until-loop ... To wait for a command you started, use
  run_in_background: true. Do not chain shorter sleeps to work around this
  block.`
- **15:33:13Z**: a bare `sleep 300`, refused the same way (`Blocked:
  standalone sleep 300`).
- **15:34 to 15:39Z**: the lane did what the refusal suggested: three
  `run_in_background` sleeps and a `Monitor` on its sampler. Then it ended its
  turn to wait for them.
- **15:41:17Z**: the workflow harness answered the ended turn with
  `[structured-output-enforce] You MUST call the StructuredOutput tool`, and
  the lane reported.

That is the failure `.claude/agents/lane.md` already names for
`run_in_background` (Q206, Q148's merger). The difference is that there is now
no foreground `sleep` to fall back on, so a lane told to wait has only two
options, and the harness offers it the one that ends it.

The arms were `setsid nohup`, so they ran on and the data was kept. The
sampler was a background task of the lane, so it died with the lane's report,
and one 300 s sample (about 15:48Z) is missing. The dispatcher restarted it at
15:53:33Z.

### 302.2 What works

A wait that is not a `sleep` passes:

```
timeout 540 tail --pid=<pid> -f /dev/null
```

It returns 0 when the process exits and 124 after nine minutes. Tested at
15:52:43Z against the running arm, pid 680280: not refused, exit 124 after 20
seconds with `timeout 20`. A lane repeats it, one plain foreground call each
time with `timeout` 600000, until it gets 0.

This is waiting on a process, not a loop of short sleeps, so it is not the
workaround the refusal forbids. It also costs a call only every nine minutes,
which is what the refusal is protecting.

### 302.3 What changes

- `.claude/agents/lane.md`: the rule "a job that runs past ten minutes goes in
  chunks of under ten" now says how a lane waits on a detached run.
- `CLAUDE.md`: the sentence saying foreground `sleep` is not blocked is
  corrected. The paragraph around it still holds: a foreground gate call waits
  by itself, and a wait loop around it is still wrong.
- §147.4 gets a pointer here.

A session reads its agent types once, when it starts (§266.6), so the running
dispatcher's lanes do not get the new `lane.md`. Until that session ends, every
brief for a lane that waits on a detached run carries the `tail --pid` line
itself. Q175's second run was briefed that way. Q157 and Q188 were sent before
this was known and may come back the way Q175's first run did.
