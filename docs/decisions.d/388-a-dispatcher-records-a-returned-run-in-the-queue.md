## 388. A dispatcher records a returned run in the queue — 2026-10-11

*Mattias asked, on 2026-10-03, why Q650's run sat as a `/tmp` handoff a reboot
would wipe while `just status`, `just queue` and `just lanes` could show
nothing of it. Built in the session that filed and then took Q813.*

§138.3 says end finished, or end written down. For a dispatcher closing out a
run, "written down" is the queue first, not the handoff. The queue is read by
every session that opens this tree — a lane, a verifier, a merger, the next
dispatcher — and a row in it survives a `/clear` and a reboot alike. A handoff
file does neither: Q650's run returned `merged: false` with a verifier's
finding and two unfiled rows, and the dispatcher wrote all three into a `/tmp`
file and stopped, leaving `just status`, `just queue` and `just lanes` to show
a row that still read `lane:` with no agent behind it.

So a dispatcher amends §138.3 like this: decide a returned run's row — take it
again, park it, or drop it — and file the rows its journal names, in the one
`just queue apply` that closes it out, before writing anything else. The three
recipes above are the record a session without this one's context can read
back whole; a handoff is for what they cannot hold; `dispatch-lanes` step 6
says the mechanics.

A corollary for the context budget hook, which is `guardrails`' and lives in
the `claude-questions` repo, outside this one: it should stop telling every
dispatcher nearing budget to call the `handoff` skill, and instead ask first
whether every returned run is closed out this way. A dispatcher with nothing
left that `just status`, `just queue` and `just lanes` cannot show has nothing
for a handoff file to hold — its whole close-out is the one message in
`dispatch-lanes` step 6's new paragraph, telling Mattias to `/clear` in the
same `claude` process and run `/dispatch-lanes`. That wording is not built
here: `claude-questions` is a separate repository and this row touches nothing
outside `rcad`. It is left for Mattias to carry over.
