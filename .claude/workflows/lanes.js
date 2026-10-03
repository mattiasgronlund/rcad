// Run lanes against one pinned base, check each writing lane, and merge the
// ones that pass onto main, one merge at a time.
//
// Two runs on 2026-09-19 and 2026-09-20 used scripts that were ~90% the same
// text, retyped the second time. This is that text, once. What it keeps from
// both is the part that turns CLAUDE.md's prose into something the script
// enforces: every lane reports a structured record, every number it reports
// carries the command that produced it, and a lane that did not start from the
// pinned base is not verified at all -- its measurements are about another tree.
//
// A dispatcher runs one workflow per queue row (docs/decisions.md §266): the
// lane, then its verifier, then a merger that lands it on main, closes the row
// and hands back about twenty lines. The merge used to be the dispatcher's, in
// its own context, and was most of what that context grew by. Several lanes in
// one run still work; their merges wait for one another. A run that merged then
// removes its own three worktrees (§315).
//
// Invoke it as the `lanes` workflow, which is `meta.name` below and not this
// file's name: Claude Code loads `.claude/workflows/*.js`, parses `meta`, and
// registers what `meta.name` says. Pass args:
//
//   { base: "<sha>",
//     lanes: [ { label, queueId, brief, writeSet, ... }, ... ] }
//
// Per lane: `label` (required, names the branch and the progress row),
// `queueId` (the docs/queue.md row this lane does; required when it writes,
// because its merger closes that row. A bundle of small rows (§352.4) gives
// them all, `Q1,Q2,Q3`, as `queue next` printed them, and its merger
// closes each), `brief` (required, what the lane is for
// -- the base and the switch below are appended), `writeSet` (the files it
// owns; absent or empty means read-only, which also means no verifier, no
// commit and no merge), `branch` (default `lane-<label>`, and it must not exist
// yet: the lane creates it and refuses to reset one), `model`, `sourcePaths`
// (what the verifier reverts to base to prove the tests fail without the
// change; defaults to the `crates/<c>/src` dirs in `writeSet`), `pinsOnly`
// (true for a lane whose write set has nothing to revert -- a verdict, a
// pinned test and a decisions section, no `src/` path anywhere in its work.
// The verifier then proves the merge on the new tests passing on the branch,
// not on a revert or a mutation. Never inferred from an empty `sourcePaths`:
// that stays the safe refusal, so a dispatcher who forgot to pass
// `sourcePaths` for a tooling lane still gets stopped, not waved through. Set
// `pinsOnly` and also declare a `src/` path, in `writeSet` or `sourcePaths`,
// and the lane is refused instead: there is something to revert, so it is not
// pins-only), `report`
// ({properties, required} merged into the lane's schema for findings that are
// this lane's own), and `verify: false` to skip the check, which leaves the
// merger's gate as the only one.
//
// The standing rules of each role live in `.claude/agents/lane.md`,
// `verifier.md` and `merger.md`, which `agentType` loads, and CLAUDE.md, which
// every role reads once. What stays here is what differs per lane.
export const meta = {
  name: 'lanes',
  description: 'Run rcad lanes in worktrees off one pinned base, verify each writing lane, and merge the ones that pass onto main one at a time',
  whenToUse: 'Dispatching one queue row, or several rows off one known-green base sha.',
  phases: [
    { title: 'Work', detail: 'one isolated worktree per lane' },
    { title: 'Verify', detail: 're-derive each writing lane\'s claims from its branch', model: 'sonnet' },
    { title: 'Merge', detail: 'one at a time: merge onto origin/main, close the row, gate, push', model: 'opus' },
    { title: 'Prune', detail: 'remove the worktrees a merged run leaves', model: 'haiku' },
  ],
}

const MERGE_ATTEMPTS = 3
const DOCS_CATCH_UPS = 5
const RESULT_LINES = 20

const writes = (lane) => Array.isArray(lane.writeSet) && lane.writeSet.length > 0
const rowsOf = (lane) => {
  const ids = lane.queueId.split(',')
  return ids.length === 1 ? `row ${ids[0]}` : `rows ${ids.join(', ')}, a bundle of ${ids.length} small rows (§352.4)`
}
const branchOf = (lane) => lane.branch || `lane-${lane.label}`
const mergeBranchOf = (lane) => `merge-${lane.label}`

if (!args || typeof args !== 'object') throw new Error('lanes: pass args {base, lanes: [...]}')
const BASE = args.base
if (typeof BASE !== 'string' || BASE.length < 7) throw new Error('lanes: args.base must be a full sha of the tree the lanes start from')
const LANES = Array.isArray(args.lanes) ? args.lanes : []
if (!LANES.length) throw new Error('lanes: args.lanes is empty')
for (const lane of LANES) {
  if (!lane || !lane.label || !lane.brief) throw new Error('lanes: every lane needs a label and a brief')
  if (writes(lane) && !/^[TDQ][0-9]+(,[TDQ][0-9]+)*$/.test(lane.queueId || '')) {
    throw new Error(`lanes: ${lane.label} writes, so it needs the queueId of its docs/queue.md row, or Q1,Q2,... for a bundle; its merger closes them`)
  }
}

log(`${LANES.length} lane(s) off ${BASE.slice(0, 8)}: ${LANES.map((l) => `${l.label}${writes(l) ? ` (${l.queueId})` : ' (read-only)'}`).join(', ')}`)

// The script body runs top to bottom before the functions below it are
// reached, and only function declarations are hoisted, so the one piece of
// state the stages share is declared here.
let lastMerge = Promise.resolve()

const results = await pipeline(
  LANES,
  (lane) => agent(writes(lane) ? WRITING(lane) : READING(lane), {
    label: lane.label,
    phase: 'Work',
    isolation: 'worktree',
    agentType: 'lane',
    schema: reportSchema(lane),
    model: lane.model,
  }),
  (report, lane) => verified(report, lane),
  (prior, lane) => {
    if (prior.stop) return handedBack(lane, prior)
    if (!writes(lane)) return { lane: lane.label, answer: prior.report }
    return mergeAfterTheLast(lane, prior).then((merge) => {
      const row = merged(lane, prior, merge)
      return row.merged ? pruned(lane, row, treesOf(prior).concat(merge.worktree_path)) : row
    })
  },
)

const rows = results.map((r, i) => r || {
  row: LANES[i].queueId,
  branch: branchOf(LANES[i]),
  merged: false,
  lines: [`${LANES[i].label}: a stage threw, so nothing after it ran; the run's journal says which`],
})
log(`done: ${rows.filter((r) => r.merged).length} of ${LANES.length} lane(s) merged`)
return { base: BASE, lanes: rows }

// A lane that started somewhere else measured another tree; verifying its
// claims against this base would only tell us they disagree, which we know.
function verified(report, lane) {
  const out = { report, check: null, kind: null }
  if (!report) return stopped(lane, out, 'the lane returned nothing')
  if (typeof report.base_sha !== 'string' || report.base_sha.slice(0, 8) !== BASE.slice(0, 8)) {
    return stopped(lane, out, `the lane checked out ${report.base_sha}, not ${BASE.slice(0, 8)}`)
  }
  if (!writes(lane)) return out
  if (report.outcome === 'blocked') return stopped(lane, out, 'the lane is blocked')
  if (lane.verify === false) return out
  out.kind = namesTests(report) ? 'tests' : 'moved_text'
  return agent(VERIFY(lane, report, out.kind), {
    label: `verify:${lane.label}`,
    phase: 'Verify',
    isolation: 'worktree',
    agentType: 'verifier',
    schema: verdictSchema(out.kind),
    model: 'sonnet',
  }).then((check) => {
    out.check = check
    if (!check) return stopped(lane, out, 'the verifier returned nothing')
    if (check.verdict === 'reject') return stopped(lane, out, 'the verifier rejected it')
    const why = unproven(lane, out.kind, check)
    if (why.length) return stopped(lane, out, `the verifier said ${check.verdict}, but ${why.join('; ')}`)
    return out
  })
}

function stopped(lane, out, why) {
  log(`${lane.label}: not merged -- ${why}`)
  out.stop = why
  return out
}

function namesTests(report) {
  return Array.isArray(report.tests_that_fail_without_the_change) && report.tests_that_fail_without_the_change.length > 0
}

// A `merge` that did not watch the tests fail is a `merge` on the verifier's
// word rather than on a demonstration, which is the whole reason there is a
// verifier. The script refuses it here rather than leave it to the merger to
// notice which field was false.
//
// A revert can't fail for every fix: Q160's floor only moved up, so putting
// the old, too-low floor back is a change a passing proptest run can't
// distinguish from the new one at most seeds. `reproduced_mutation` is the
// verifier's other way to demonstrate the same thing -- a mutation the fix
// catches and the base commit doesn't -- and `mutationProven` is the one place
// that decides whether a reported one counts.
//
// A third lane has nothing to revert at all: a verdict, a pinned test and a
// decisions section, with no `src/` path in its write set (Q126, Q127, Q203).
// Those came back "verifier said merge" and were merged by hand, because the
// script refused for want of a revert that does not exist. `lane.pinsOnly`
// says so explicitly, and only an explicit `true` gets past the check below --
// never an empty `sourcePathsOf(lane)` on its own, which stays the refusal a
// dispatcher who forgot `sourcePaths` for a tooling lane needs (§287).
//
// A fourth changes nothing that runs: comments, doc comments and test code
// (§375). There a revert takes out nothing the tests could see, so a lane
// whose lines-that-run print ends `lines that run: 0` merges on its tests
// passing, and one that names no test merges on that print alone. The print is
// the evidence, never the verifier's reading of the diff: a count it did not
// get from the command it was given is refused like a missing one.
function unproven(lane, kind, check) {
  const why = []
  if (check.files_outside_write_set && check.files_outside_write_set.length) why.push(`the branch touches ${check.files_outside_write_set.join(', ')}, outside the write set`)
  const runs = linesThatRun(check.lines_that_run)
  if (runs === null) why.push('the lines-that-run print the verifier reported does not end in its count, lines that run: N')
  if (squeezed(check.lines_that_run_command) !== squeezed(linesThatRunCommand(lane))) why.push('the verifier ran another command than the lines-that-run command it was given')
  if (kind === 'tests') {
    if (check.tests_pass_on_branch !== true) why.push('the tests do not pass on the branch')
    const sources = sourcePathsOf(lane)
    if (lane.pinsOnly === true && sources.length) why.push('the lane is marked pinsOnly, but it declares a source directory to revert -- there is something to revert, so pinsOnly does not apply')
    const pinsOnly = lane.pinsOnly === true && !sources.length
    if (!pinsOnly && runs !== 0) {
      if (check.tests_fail_without_fix !== true && !mutationProven(check.reproduced_mutation)) why.push('the tests were never seen to fail with the source reverted, and no reproduced mutation stands in for it')
      if (!sources.length) why.push('the lane declared no source directory to revert')
    }
  } else if (runs) {
    why.push(`the branch changes ${runs} lines that run and names no test that fails without them, starting with: ${check.lines_that_run.trim().split('\n')[0]}`)
  }
  return why
}

// The count the `lines-that-run` recipe ends its print with, or null when the
// reported print does not end in one -- the recipe failed, or the verifier
// left the line out.
function linesThatRun(print) {
  if (typeof print !== 'string') return null
  const last = print.trim().split('\n').pop().trim()
  const count = /^lines that run: ([0-9]+)$/.exec(last)
  return count ? Number(count[1]) : null
}

function squeezed(s) {
  return typeof s === 'string' ? s.replace(/\s+/g, ' ').trim() : null
}

function mutationProven(m) {
  return !!m && m.outcome_on_base === 'pass' && m.outcome_on_branch === 'fail'
}

// Defaulted rather than asked for, because the verifier's revert step only
// works on non-test source: reverting the tests too would put back the version
// that passes and prove nothing.
function sourcePathsOf(lane) {
  if (Array.isArray(lane.sourcePaths) && lane.sourcePaths.length) return lane.sourcePaths
  const dirs = []
  for (const f of lane.writeSet || []) {
    const cut = f.indexOf('/src/')
    if (cut < 0) continue
    const dir = f.slice(0, cut + 4)
    if (!dirs.includes(dir)) dirs.push(dir)
  }
  return dirs
}

// Merges inside one run wait for one another, so they never race each other;
// they race only other runs and other sessions, and `ff-main` and the ancestor
// check before the push settle that without a lock of their own (§266.4).
function mergeAfterTheLast(lane, prior) {
  const run = lastMerge.then(() => agent(MERGE(lane, prior), MERGER(lane)))
  lastMerge = run.catch(() => null)
  return run
}

function MERGER(lane) {
  return {
    label: `merge:${lane.label}`,
    phase: 'Merge',
    isolation: 'worktree',
    agentType: 'merger',
    schema: mergedSchema(),
    model: 'opus',
  }
}

// §304.4: a row can reach here with the verifier already having said `merge`
// or `fix_first` and still end up `merged: false` -- the merger returned
// nothing, or declined for a reason that is its own rather than the row's
// content. Q124 and Q128 sat held five and a half hours because that only
// showed up in the log, not in the row's own returned text, and no dispatcher
// noticed. `verified` below carries it into the line itself.
function merged(lane, prior, m) {
  const row = { row: lane.queueId, branch: branchOf(lane), merged: false, lines: [] }
  const verified = !!prior.check && (prior.check.verdict === 'merge' || prior.check.verdict === 'fix_first')
  if (!m) {
    row.lines.push(`${lane.queueId} ${branchOf(lane)}: ${verified ? 'verified, not merged' : 'not merged'} -- the merger returned nothing`)
    row.unlock = treesOf(prior)
    return row
  }
  row.merged = m.merged === true
  row.lines.push(row.merged
    ? `${lane.queueId} ${branchOf(lane)}: pushed ${m.merged_sha} to main, attempt ${m.push_attempts} of ${MERGE_ATTEMPTS}, catch-ups over docs ${m.docs_catch_ups} of ${DOCS_CATCH_UPS}`
    : `${lane.queueId} ${branchOf(lane)}: ${verified ? 'verified, not merged' : 'not merged'} -- ${m.not_merged_because || 'the merger gave no reason'}`)
  if (m.push_attempts > MERGE_ATTEMPTS) log(`${lane.label}: the merger reports ${m.push_attempts} push attempts, past the bound of ${MERGE_ATTEMPTS}`)
  if (m.docs_catch_ups > DOCS_CATCH_UPS) log(`${lane.label}: the merger reports ${m.docs_catch_ups} catch-ups over docs alone, past the bound of ${DOCS_CATCH_UPS}`)
  if (m.gate_pass_line) row.lines.push(`gate: ${m.gate_pass_line}`)
  if (m.ff_main) row.lines.push(`ff-main: ${m.ff_main}`)
  for (const r of m.rows_closed || []) row.lines.push(`closed ${r}`)
  for (const r of m.rows_added || []) row.lines.push(`added ${r}`)
  for (const f of m.fixes_applied || []) row.lines.push(`fixed ${f}`)
  for (const n of m.needs_judgement || []) row.lines.push(`for you: ${n}`)
  if (row.lines.length > RESULT_LINES) log(`${lane.label}: the merger's result is ${row.lines.length} lines, past the ${RESULT_LINES} asked for; all of them are returned`)
  return row
}

// The merger marked the lane's tree, the verifier's and its own with `just
// mark-finished`, so `prune-worktrees` removes them now rather than an hour
// from now (§315.2). It runs here and not in the merger because it keeps the
// tree it is run from, and the merger's own is one of the three; from here it
// runs in the dispatcher's tree, which it keeps for the same reason. The exact
// command is the one `.claude/settings.json` allows, so nothing has to ask.
//
// A throw is caught here, and only here, because by now the row has merged and
// a thrown stage would report it as not merged. A session started before
// `pruner.md` existed cannot resolve the type and throws (§266.6).
function pruned(lane, row, trees) {
  return agent(PRUNE(trees), {
    label: `prune:${lane.label}`,
    phase: 'Prune',
    agentType: 'pruner',
    schema: prunedSchema(),
    model: 'haiku',
  }).then((p) => {
    if (!p) row.lines.push('prune: the agent returned nothing, so the run\'s worktrees may still be on disk')
    else for (const l of p.lines) row.lines.push(`prune: ${l}`)
    return row
  }, (e) => {
    row.lines.push(`prune: did not run (${e && e.message}), so the run's worktrees are still on disk`)
    return row
  })
}

function handedBack(lane, prior) {
  const verified = !!prior.check && (prior.check.verdict === 'merge' || prior.check.verdict === 'fix_first')
  const row = { row: lane.queueId, branch: branchOf(lane), merged: false, lines: [`${lane.label}: ${verified ? 'verified, not merged' : 'not merged'} -- ${prior.stop}`] }
  if (prior.report) {
    row.lines.push(`lane said ${prior.report.outcome}: ${prior.report.summary}`)
    if (prior.report.remaining) row.lines.push(`remaining: ${prior.report.remaining}`)
  }
  if (prior.check) {
    row.lines.push(`verifier said ${prior.check.verdict}${prior.check.notes ? `: ${prior.check.notes}` : ''}`)
    for (const c of prior.check.unbacked_claims || []) row.lines.push(`unbacked: ${c}`)
    for (const c of prior.check.number_mismatches || []) row.lines.push(`number: ${c}`)
    for (const c of prior.check.moved_text_problems || []) row.lines.push(`moved text: ${c}`)
  }
  row.unlock = treesOf(prior)
  return row
}

function treesOf(prior) {
  return [prior.report && prior.report.worktree_path, prior.check && prior.check.worktree_path].filter(Boolean)
}

function START(lane) {
  return `
Base: main at ${BASE}. Your first two commands, in this order, and report both shas back:
  git log --oneline -1
  git switch -c ${branchOf(lane)} ${BASE}
If the switch refuses because ${branchOf(lane)} already exists, stop and report it as blocked. Never reach for \`checkout -B\` or \`switch -C\`: they reset a branch that another worktree may still have checked out, and that tree then shows a staged revert of everything since its commit.`
}

function WRITING(lane) {
  return `Queue ${rowsOf(lane)}. You own these files and no others: ${lane.writeSet.join(', ')}.
Your branch: ${branchOf(lane)}.

${lane.brief}
${START(lane)}`
}

function READING(lane) {
  return `This lane is read-only: you own no files. Your branch: ${branchOf(lane)} (a checkout convenience only).

${lane.brief}
${START(lane)}`
}

function reportSchema(lane) {
  const core = {
    start_sha: { type: 'string', description: 'what git log -1 said BEFORE your switch' },
    base_sha: { type: 'string', description: 'what you switched to' },
    branch: { type: 'string' },
    worktree_path: { type: 'string' },
    summary: { type: 'string', description: 'plain words, at most 10 lines' },
    numbers: { type: 'array', items: numberSchema(), description: 'every figure you report, with its command' },
    out_of_write_set_needs: { type: 'array', items: { type: 'string' } },
    remaining: { type: 'string', description: 'what is left, including what you learned about work you were told not to attempt; each defect with its class (wrong, crash or refuses, then /A to /D, §352.3), each defect you guess at and did not run with no class (§360.5), and each item of about ten minutes whose place is known marked small' },
  }
  const required = ['start_sha', 'base_sha', 'branch', 'worktree_path', 'outcome', 'summary', 'numbers', 'remaining']
  if (writes(lane)) {
    core.outcome = { type: 'string', enum: ['fixed', 'limit_demonstrated', 'partial', 'blocked'] }
    core.landed_sha = { type: 'string', description: 'your last commit' }
    core.decisions_section = { type: 'string', description: 'the section number you claimed with just claim-section, if any' }
    core.files_changed = { type: 'array', items: { type: 'string' } }
    core.tests_that_fail_without_the_change = { type: 'array', items: { type: 'string' }, description: 'empty for a lane that changed only .md files and comments (§375)' }
    required.push('landed_sha', 'files_changed', 'tests_that_fail_without_the_change')
  } else {
    core.outcome = { type: 'string', enum: ['answered', 'partial', 'blocked'] }
  }
  const extra = lane.report || {}
  return {
    type: 'object',
    required: required.concat(Array.isArray(extra.required) ? extra.required : []),
    properties: Object.assign(core, extra.properties || {}),
  }
}

// The lane's tree and the verifier's own tree are two different worktrees the
// harness made. Naming only the lane's path here, with the schema saying
// "your own worktree", is what made a verifier echo the lane's path back
// (§263.4): it was never told its own tree has a path at all, let alone how
// to find it.
function VERIFY(lane, r, kind) {
  const revert = sourcePathsOf(lane).join(', ') || '(none declared; say so and skip the revert)'
  const pinsOnlyNote = lane.pinsOnly === true
    ? '\nThis lane is marked pinsOnly: a verdict, a pinned test and a decisions section, nothing in src/ to revert. Skip the revert step. Step 3 already proved the tests pass; do not run them again to watch that (§370) -- take tests_pass_on_branch from its gate summary the way step 6 says, then report tests_fail_without_fix: false with no reproduced_mutation and say in notes that pinsOnly is why -- that is the expected shape here, not a defect.'
    : ''
  return `Check lane ${lane.label}, queue ${rowsOf(lane)}, by the ${kind === 'tests' ? 'tests' : 'moved text'} steps.
Base: ${BASE}
Branch: ${branchOf(lane)}
Write set: ${lane.writeSet.join(', ')}
The lane's worktree, to unlock once you are done with it -- this is not your own tree, and not what you report as worktree_path: ${r.worktree_path}
Source paths to revert: ${revert}${pinsOnlyNote}
Lines that run -- in step 1, before you check the branch out, run this once in your own tree, exactly as written; report it as lines_that_run_command and all it prints, verbatim, as lines_that_run. It prints at most eleven lines, and the last is always lines that run: N:
  ${linesThatRunCommand(lane)}
When N is 0, nothing that runs changed (§375): skip the revert below, say so in notes, and report tests_fail_without_fix: false with no reproduced_mutation if your steps are the tests ones.
Revert them and watch the tests fail, then restore them -- that is the default proof for the fail half, and report it as tests_fail_without_fix. Step 3 already proved the tests pass; do not run them again to watch that (§370) -- take tests_pass_on_branch from its gate summary the way step 6 says.
Only when the revert itself cannot fail -- a fix that just moves a floor or a bound, where a value on the wrong side of the old one can still pass a proptest run at most seeds -- report a reproduced_mutation instead: make a small, uncommitted change to the source that reintroduces the kind of bug the fix is meant to catch, run one command unchanged against the base commit and against this branch, and give the exact scratch change, the command, and both outcomes ('pass' or 'fail'). Say in notes why the revert could not fail here. Do not reach for a mutation when a revert would do.
Your own worktree is a different tree the harness made for this check. Run git rev-parse --show-toplevel in it and report exactly that output as worktree_path in your verdict.

The lane claims (JSON): ${JSON.stringify(r)}`
}

// A recipe, so the verifier types and reports one short line (§382). It runs
// before the verifier checks the branch out, so the recipe that judges the
// branch is never a copy the branch changed.
function linesThatRunCommand(lane) {
  return `mise exec -- just lines-that-run ${BASE} ${branchOf(lane)}`
}

function MERGE(lane, prior) {
  const r = prior.report
  const verifier = prior.check
    ? `the verifier's, ${prior.check.worktree_path}`
    : 'none: the dispatcher set verify: false for this lane, so your gate is the only check'
  return `Merge queue ${rowsOf(lane)}, lane ${lane.label}, onto main, or say why not.
Lane branch: ${branchOf(lane)}, which it reports ending at ${r.landed_sha}.
The lane's write set: ${lane.writeSet.join(', ')}
Trees to unlock: the lane's, ${r.worktree_path}; ${verifier}
Only if your push went through, after unlocking your own tree: mise exec -- just mark-finished ${treesOf(prior).join(' ')} <your own tree>
Your first commands, in this order:
  git log --oneline -1
  git fetch origin
  git switch -c ${mergeBranchOf(lane)} origin/main
If the switch refuses because ${mergeBranchOf(lane)} already exists, stop and report it as not merged. Never reach for \`checkout -B\` or \`switch -C\`.
Push attempts: at most ${MERGE_ATTEMPTS}. A catch-up over docs alone costs no attempt; at most ${DOCS_CATCH_UPS} of those, reported as docs_catch_ups.

The lane's report (JSON): ${JSON.stringify(r)}

The verifier's verdict (JSON): ${JSON.stringify(prior.check)}`
}

function PRUNE(trees) {
  return `Run this, once, and nothing else:
  mise exec -- just prune-worktrees
Then report, verbatim, every line of its output that names one of these worktrees, and its last line:
${trees.map((t) => `  ${t}`).join('\n')}`
}

function verdictSchema(kind) {
  const properties = {
    worktree_path: { type: 'string', description: 'the output of git rev-parse --show-toplevel run in your OWN tree, not the lane\'s worktree path given in the prompt' },
    branch_ok: { type: 'boolean', description: 'exists, sits on the pinned base' },
    files_outside_write_set: { type: 'array', items: { type: 'string' } },
    gate_summary: { type: 'string', description: 'the summary block, verbatim, or why you could not run it' },
    number_mismatches: { type: 'array', items: { type: 'string' } },
    unbacked_claims: { type: 'array', items: { type: 'string' } },
    fixes: { type: 'array', items: { type: 'string' }, description: 'each fix fix_first asks for: the file, the change, and why' },
    lines_that_run_command: { type: 'string', description: 'the lines-that-run command from your brief, exactly as you ran it' },
    lines_that_run: { type: 'string', description: 'everything that command printed, verbatim, down to its last line, lines that run: N -- never your own reading of the diff' },
    verdict: { type: 'string', enum: ['merge', 'fix_first', 'reject'] },
    notes: { type: 'string', description: 'at most five lines, for the merger: what it must act on and what no other field says -- never an account of the steps you ran, which the other fields already hold' },
  }
  const required = ['worktree_path', 'branch_ok', 'files_outside_write_set', 'gate_summary', 'number_mismatches', 'unbacked_claims', 'fixes', 'lines_that_run_command', 'lines_that_run', 'verdict']
  if (kind === 'tests') {
    properties.tests_pass_on_branch = { type: 'boolean', description: "from step 3's gate summary (served or fresh), not a separate run -- §370" }
    properties.tests_fail_without_fix = { type: 'boolean', description: 'false if you could not make them fail with the source reverted' }
    properties.reproduced_mutation = {
      type: 'object',
      description: 'Fill this in only when tests_fail_without_fix is false because the revert itself cannot fail -- a fix that only moves a floor or a bound, where a value on the old, wrong side of it can still pass a proptest run. Say why in notes. It stands in for the revert: a mutation the fix catches and the base commit does not.',
      properties: {
        scratch_change: { type: 'string', description: 'the small, uncommitted change to the source that reintroduces the kind of bug the fix is meant to catch, as a diff or an exact description' },
        command: { type: 'string', description: 'the exact test command, run unchanged against both versions below' },
        outcome_on_base: { type: 'string', enum: ['pass', 'fail'], description: 'what the command produced with the scratch change applied on top of the base commit -- expected \'pass\', since the base could not catch it' },
        outcome_on_branch: { type: 'string', enum: ['pass', 'fail'], description: 'what the command produced with the same scratch change applied on top of this branch -- expected \'fail\', since the fix catches it' },
      },
      required: ['scratch_change', 'command', 'outcome_on_base', 'outcome_on_branch'],
    }
    required.push('tests_pass_on_branch', 'tests_fail_without_fix')
  } else {
    properties.paragraphs_checked = { type: 'integer', description: 'how many removed or moved paragraphs you traced' }
    properties.moved_text_problems = { type: 'array', items: { type: 'string' }, description: 'a paragraph that landed nowhere, twice, or with its meaning changed' }
    required.push('paragraphs_checked', 'moved_text_problems')
  }
  return { type: 'object', required, properties }
}

function numberSchema() {
  return {
    type: 'object',
    required: ['what', 'value', 'command'],
    properties: {
      what: { type: 'string' },
      value: { type: 'string' },
      command: { type: 'string', description: 'the exact command you ran to get this value' },
    },
  }
}

function mergedSchema() {
  return {
    type: 'object',
    required: ['start_sha', 'worktree_path', 'merge_branch', 'merged', 'push_attempts', 'docs_catch_ups', 'rows_closed', 'rows_added', 'unlocked', 'needs_judgement'],
    properties: {
      start_sha: { type: 'string', description: 'what git log -1 said before your switch' },
      worktree_path: { type: 'string', description: 'the output of git rev-parse --show-toplevel run in your OWN tree' },
      merge_branch: { type: 'string' },
      merged: { type: 'boolean', description: 'true only when your push to origin/main went through' },
      merged_sha: { type: 'string', description: 'the sha you pushed to origin/main' },
      gate_pass_line: { type: 'string', description: 'the PASS line of the gate on the tree you pushed, verbatim' },
      push_attempts: { type: 'integer', minimum: 0 },
      docs_catch_ups: { type: 'integer', minimum: 0, description: 'tries after a catch-up whose diff was all docs/ or .md, which push_attempts does not count (§367)' },
      ff_main: { type: 'string', description: 'the last line just ff-main printed' },
      fixes_applied: { type: 'array', items: { type: 'string' }, description: 'each verifier fix you made, one line each' },
      rows_closed: { type: 'array', items: { type: 'string' }, description: 'e.g. "Q142 done:<sha>"' },
      rows_added: { type: 'array', items: { type: 'string' }, description: 'e.g. "Q160 tier 1 sonnet: <what, in a few words>"' },
      unlocked: { type: 'array', items: { type: 'string' } },
      not_merged_because: { type: 'string' },
      needs_judgement: { type: 'array', items: { type: 'string' }, description: 'only what the dispatcher must decide, one line each' },
    },
  }
}

function prunedSchema() {
  return {
    type: 'object',
    required: ['lines'],
    properties: {
      lines: { type: 'array', items: { type: 'string' }, description: 'each output line that names one of the worktrees, then the last line, all verbatim' },
    },
  }
}
