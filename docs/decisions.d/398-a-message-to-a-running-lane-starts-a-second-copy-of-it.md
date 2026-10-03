## 398. A message to a running lane starts a second copy of it — 2026-10-04

*Queue row Q856, from Q437's lane report. Dispatcher session `rcad-94`.*

**Q437's second writer was the dispatcher's own `SendMessage`.** At 21:46:16Z
the dispatcher sent Q437's lane (run `wf_2154d68c-293`, agent
`af05cd4d6154540fe`) a correction to its brief. The lane was mid-task. The tool
answered "Resuming agent af05cd4" and started a second copy of the lane from
its transcript, beside the workflow's own, in the same worktree and on the same
branch. Neither copy knew of the other. One ran `git checkout -- bake.rs` over
the other's scratch edit. The other committed `4c472a5d`, whose message cites
a mutation run on that scratch edit, and its gate was killed partway (Q857).
The copy handed its report back to the dispatcher, outside the run, so the
run's verifier never saw it.

**The check that shows it.** The lane's transcript,
`subagents/workflows/wf_2154d68c-293/agent-af05cd4d6154540fe.jsonl` under the
dispatcher's session, forks at the parent of the correction: one child is the
correction at 21:46:16Z, the other an attachment at 21:46:20Z, and from there
every record has two children. Finding the message in that file once proves
nothing, since both copies write to it. `ListAgents` does not list a
workflow's agents, which is the sign that a send cannot reach the live one.

**So the workflow already starts one agent per lane tree**, and Q856's fix,
making it do so, has nothing to change. Q856 is dropped on this section.

**The rule: a dispatcher never messages an agent a `lanes` run is running.**
A brief found wrong while its lane runs is corrected after the run returns:
the verifier and merger judge what the lane did, and the row is sent again if
it has to be. If the lane must not finish, stop the run with `TaskStop` and
send the row again. `dispatch-lanes` step 5 says so. The wrong brief here
quoted §136.8 unread. It said the picker's room is not widened toward
`validate`'s allowance, where §136.8 and §368 say it was. Reading a section
before quoting it is what would have made the message unnecessary.

**What it cost.** Nothing that reached `main`: the verifier found the branch
held exactly the three files the lane claimed, and Q437 merged as `bfc39286`.
Both copies measured the same premise gone (§301.9).
