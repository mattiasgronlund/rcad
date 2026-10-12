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
// Since §383 and §384 the lane writes the rows it leaves, as `add` steps, and a
// `merge` on such a lane goes to a Haiku lander that runs `just land` once. The
// Opus merger takes a `fix_first`, a lane with no usable rows, a `merge` whose
// own gate did not read `gate: PASS` (§414), and every merge the recipe stopped
// on. Since §385 the verifier's fixed steps, and its re-runs of the lane's
// numbers, are `just verify-lane`'s, and the verifier keeps the reading.
//
// Since §416 a writing lane past its context budget does not carry on past it.
// It commits what it has, lets go of its branch and reports `relay` with a
// handoff, and a fresh lane carries the branch on from that commit, at most
// MAX_RELAYS times a row. The verifier and the merger see one branch and the
// last lane's report, which carries the whole row's account.
//
// Since §433.8 a lane whose row names the inputs that show its fault -- crash
// files, corpus files, a shape -- runs them on its base first, and when none
// shows it, commits nothing and reports `does_not_reproduce` with each run. That
// lane goes to its verifier even under `verify: false`, since the row would
// otherwise close on the lane's word alone, and the verifier runs the same
// commands on the base. On its `confirm` a merger closes the row done at the
// base; it never goes to the lander, since there is no branch for `just land`
// to merge. Anything but a `confirm` comes back not merged.
//
// Invoke it as the `lanes` workflow, which is `meta.name` below and not this
// file's name: Claude Code loads `.claude/workflows/*.js`, parses `meta`, and
// registers what `meta.name` says. Pass args:
//
//   { base: "<sha>",
//     lanes: [ { label, queueId, notes }, { label, queueId, brief, writeSet, ... }, ... ] }
//
// A lane comes in one of two forms (§396.4). The short form, `{ label,
// queueId, notes }`, is the usual one. The run first sends the row to `prep`,
// a read-only Sonnet agent (`.claude/agents/prep.md`) that reads it with `just
// queue show` and the code at the base, and answers `ok` or `needs`. On `ok`,
// and only once its write set and model are checked against the rows it
// copied, the lane's brief is built from the rows' text, prep's facts, the
// dispatcher's `notes` (optional: only what the row's text does not say) and
// the two notes every brief carries, and the lane starts with the write set,
// model, `sourcePaths` and marks prep answered and the `for_the_verifier`
// field a writing lane gets by default. On anything else -- `needs`, an answer
// that does not check, a prep that threw -- no lane starts, and the row comes
// back not merged with prep's answer in its lines. The full form below is for
// a row sent again with a finding in its brief, a `report` schema, or `verify:
// false`. A lane giving `notes` beside any of the full form's own fields is
// refused.
//
// `args.lanes` with more than one entry throws at the start unless
// `args.shareOneRun` is `true`: five times from 2026-10-03 to 2026-10-05 a
// dispatcher put a fill of several rows into one run this way, each time
// stopped and relaunched as one run per row (§230.4). A bundle of small rows
// (§352.4) is one `lanes` entry with several `queueId`s, so it still passes
// with no flag.
//
// Both forms: `label` (required and unique in a run, names the branch and the
// progress row), `queueId` (the docs/queue.md row this lane does; required for
// the short form, and for the full form when it writes, because its merger
// closes that row. A bundle of small rows (§352.4) gives them all,
// `Q1,Q2,Q3`, as `queue next` printed them, and its merger closes each), and
// `branch` (below). The full form: `brief` (required, what the lane is for
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
// pins-only), `sameBehaviour` (true for a lane whose row changes code that runs
// and keeps what it computes -- one of two copies dropped for the other, a body
// lifted into a function, a block moved. No test can fail without such a
// change, so the lane names the tests that reach it instead, and the verifier
// shows they pass with the source reverted and fail under a mutation of the
// changed lines (§399). Never inferred, for `pinsOnly`'s reason, and never set
// beside it), `report`
// ({properties, required} merged into the lane's schema for findings that are
// this lane's own), and `verify: false` to skip the check, which leaves the
// merger's gate as the only one.
//
// The standing rules of each role live in `.claude/agents/prep.md`, `lane.md`,
// `verifier.md` and `merger.md`, which `agentType` loads, and CLAUDE.md, which
// every role reads once. What stays here is what differs per lane.
export const meta = {
  name: 'lanes',
  description: 'Run rcad lanes in worktrees off one pinned base, verify each writing lane, and merge the ones that pass onto main one at a time',
  whenToUse: 'Dispatching one queue row, or several rows off one known-green base sha.',
  phases: [
    { title: 'Prep', detail: 'check and brief each short-form lane\'s row at the base', model: 'sonnet' },
    { title: 'Work', detail: 'one isolated worktree per lane' },
    { title: 'Verify', detail: 're-derive each writing lane\'s claims from its branch', model: 'sonnet' },
    { title: 'Merge', detail: 'one at a time: just land through a Haiku lander, or the Opus merger when it stops or the verifier asked for fixes', model: 'opus' },
    { title: 'Prune', detail: 'remove the worktrees a merged run leaves', model: 'haiku' },
  ],
}

const MERGE_ATTEMPTS = 3
const DOCS_CATCH_UPS = 5
const RESULT_LINES = 20
const MAX_RELAYS = 3

const writes = (lane) => Array.isArray(lane.writeSet) && lane.writeSet.length > 0
const rowsOf = (lane) => {
  const ids = lane.queueId.split(',')
  return ids.length === 1 ? `row ${ids[0]}` : `rows ${ids.join(', ')}, a bundle of ${ids.length} small rows (§352.4)`
}
const branchOf = (lane) => lane.branch || `lane-${lane.label}`
const mergeBranchOf = (lane) => `merge-${lane.label}`
const afterLandBranchOf = (lane) => `merge-${lane.label}-after-land`
const STEP_WORDS = { add: 5, after: 3, tag: 3 }
const QUEUE_IDS = /^[TDQ][0-9]+(,[TDQ][0-9]+)*$/
const FULL_FORM = ['brief', 'writeSet', 'model', 'sourcePaths', 'pinsOnly', 'sameBehaviour', 'report', 'verify']
const fullFields = (lane) => FULL_FORM.filter((k) => lane[k] !== undefined)

if (!args || typeof args !== 'object') throw new Error('lanes: pass args {base, lanes: [...]}')
const BASE = args.base
if (typeof BASE !== 'string' || BASE.length < 7) throw new Error('lanes: args.base must be a full sha of the tree the lanes start from')
const LANES = Array.isArray(args.lanes) ? args.lanes : []
if (!LANES.length) throw new Error('lanes: args.lanes is empty')
if (LANES.length > 1 && args.shareOneRun !== true) throw new Error('lanes: args.lanes has more than one entry; launch one Workflow call per row, or set args.shareOneRun to true to share this run on purpose (§230.4)')
const labels = new Set()
for (const lane of LANES) {
  if (!lane || !lane.label) throw new Error('lanes: every lane needs a label')
  if (labels.has(lane.label)) throw new Error(`lanes: two lanes are labelled ${lane.label}; a label names one lane's branch and its row of the run`)
  labels.add(lane.label)
  const full = fullFields(lane)
  if (lane.notes !== undefined && full.length) throw new Error(`lanes: ${lane.label} gives fields of both forms, notes beside ${full.join(', ')}; send { label, queueId, notes } for prep to brief, or the full form with a brief of its own (§396.4)`)
  if (!full.length) {
    if (!QUEUE_IDS.test(lane.queueId || '')) throw new Error(`lanes: ${lane.label} is in the short form, so it needs the queueId of its docs/queue.md row, or Q1,Q2,... for a bundle, for prep to read`)
    if (lane.notes !== undefined && typeof lane.notes !== 'string') throw new Error(`lanes: ${lane.label} gives notes that are not text; they go into its brief as they stand`)
    continue
  }
  if (!lane.brief) throw new Error(`lanes: ${lane.label} needs a brief, or none of ${FULL_FORM.join(', ')} for prep to write one`)
  if (writes(lane) && !QUEUE_IDS.test(lane.queueId || '')) {
    throw new Error(`lanes: ${lane.label} writes, so it needs the queueId of its docs/queue.md row, or Q1,Q2,... for a bundle; its merger closes them`)
  }
  if (lane.pinsOnly === true && lane.sameBehaviour === true) {
    throw new Error(`lanes: ${lane.label} is marked both pinsOnly and sameBehaviour; a pins-only lane has nothing in src/ whose behaviour it keeps (§399)`)
  }
}

log(`${LANES.length} lane(s) off ${BASE.slice(0, 8)}: ${LANES.map((l) => `${l.label}${!fullFields(l).length ? ` (${l.queueId}, prep first)` : writes(l) ? ` (${l.queueId})` : ' (read-only)'}`).join(', ')}`)

// The script body runs top to bottom before the functions below it are
// reached, and only function declarations are hoisted, so the state the
// stages share is declared here. A stage after the first is handed the item
// as it was sent, so it reads the lane prep briefed through `laneOf`.
let lastMerge = Promise.resolve()
const relayed = new Map()
const preps = new Map()
const laneOf = (item) => preps.get(item.label).lane

const results = await pipeline(
  LANES,
  (lane) => prepared(lane),
  (prep) => prep.stop ? prep : worked(prep.lane),
  (report, item) => notPrepared(item) || verified(report, laneOf(item)),
  (prior, item) => {
    const lane = laneOf(item)
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

// §396.4: checking a row against the code and writing its brief were close to
// half of what a dispatcher wrote before its first lane, and a third over a
// session (§396.1). Prep does both inside the run, in the dispatcher's tree
// and with no tree of its own, since it writes nothing. A full-form lane skips it. A prep that throws is a
// stop like any other: a `claude` process started before `prep.md` existed
// cannot resolve the type.
function prepared(lane) {
  if (fullFields(lane).length) return kept(lane, { lane, answer: null, stop: null })
  return agent(PREP(lane), {
    label: `prep:${lane.label}`,
    phase: 'Prep',
    agentType: 'prep',
    schema: prepSchema(),
    model: 'sonnet',
  }).then((answer) => {
    const stop = prepRefused(lane, answer)
    return kept(lane, stop ? { lane, answer, stop } : { lane: briefed(lane, answer), answer, stop: null })
  }, (e) => kept(lane, { lane, answer: null, stop: `prep threw (${e && e.message}); a session begun before its tree held .claude/agents/prep.md cannot resolve the type, so restart it, or send the row in the full form` }))
}

function kept(lane, prep) {
  preps.set(lane.label, prep)
  return prep
}

// lanes.js cannot read docs/queue.md, so prep copies each row's id, model and
// write set as `just queue show` prints them, and its answer is held to that
// copy: the ids to the ones sent, the write set to the union of the rows' and
// the model to theirs. Prep never widens a write set, because only `queue
// next` checks one against the rows already running (§396.4).
function prepRefused(lane, answer) {
  if (!answer) return 'prep returned nothing'
  if (answer.answer !== 'ok') return `prep answered ${answer.answer}, so no lane starts`
  if (Array.isArray(answer.needs) && answer.needs.length) return `prep answered ok and still named ${answer.needs.length} file(s) the row needs, so no lane starts`
  if (answer.pinsOnly === true && answer.sameBehaviour === true) return 'prep marked the row both pinsOnly and sameBehaviour; a pins-only lane has nothing in src/ whose behaviour it keeps (§399)'
  const rows = Array.isArray(answer.rows) ? answer.rows : []
  const ids = rows.map((r) => r && r.id)
  if (!sameSet(ids, lane.queueId.split(','))) return `prep reported the rows ${ids.join(',') || 'none'}, not ${lane.queueId}`
  if (!sameSet(answer.writeSet, writeSetOf(rows))) return `prep answered the write set ${JSON.stringify(answer.writeSet)}, not ${JSON.stringify(writeSetOf(rows))}, the union of its rows'; it never widens or narrows one (§396.4)`
  if (answer.model !== modelOf(rows)) return `prep answered the model ${answer.model}, not ${modelOf(rows)}, which its rows' model column gives (§352.4)`
  return null
}

function sameSet(a, b) {
  if (!Array.isArray(a) || !Array.isArray(b)) return false
  const x = [...new Set(a)].sort()
  const y = [...new Set(b)].sort()
  return x.length === y.length && x.every((v, i) => v === y[i])
}

function writeSetOf(rows) {
  const files = []
  for (const r of rows) {
    for (const f of (r && typeof r.write_set === 'string' ? r.write_set : '').split(',')) {
      const file = f.trim()
      if (file && file !== '-' && !files.includes(file)) files.push(file)
    }
  }
  return files
}

function modelOf(rows) {
  return rows.some((r) => r && r.model === 'opus') ? 'opus' : 'sonnet'
}

function briefed(lane, answer) {
  const ready = {
    label: lane.label,
    queueId: lane.queueId,
    brief: SHORT_BRIEF(lane, answer),
    writeSet: answer.writeSet,
    model: answer.model,
    sourcePaths: Array.isArray(answer.sourcePaths) ? answer.sourcePaths : [],
    pinsOnly: answer.pinsOnly === true,
    sameBehaviour: answer.sameBehaviour === true,
  }
  if (lane.branch) ready.branch = lane.branch
  if (answer.writeSet.length) ready.report = forTheVerifier()
  return ready
}

// §416: the median lane peaked at 203k with its first commit at the very end,
// so a lane stopped at the budget warning handed on nothing, and briefs told
// lanes to carry on past it instead. Now the lane commits and hands its branch
// to a fresh one. A second lane re-read 14-26k when the first had committed
// and 60-105k when it had not, which is why the handoff is a commit and not
// only prose. A row that asks for more than MAX_RELAYS is handed back.
async function worked(lane) {
  let report = await laneAgent(lane, writes(lane) ? WRITING(lane) : READING(lane), lane.label, 0)
  const trees = []
  let before = null
  let n = 0
  while (report && writes(lane) && report.outcome === 'relay' && n < MAX_RELAYS) {
    const why = relayRefused(report)
    if (why) {
      relayed.set(lane.label, { trees, before, refused: why })
      return report
    }
    n++
    log(`${lane.label}: relay ${n} of ${MAX_RELAYS}, from ${report.landed_sha}`)
    trees.push(report.worktree_path)
    before = report
    report = await laneAgent(lane, RELAY(lane, before, n), `relay ${n}:${lane.label}`, n)
    if (report && report.outcome !== 'blocked' && !sameSha(report.continued_from, before.landed_sha)) {
      relayed.set(lane.label, { trees, before, refused: `relay ${n} started at ${report.continued_from}, not at ${before.landed_sha} where the lane before it stopped` })
      return report
    }
  }
  const more = !!report && writes(lane) && report.outcome === 'relay'
  relayed.set(lane.label, { trees, before, refused: more ? `the lane asked for relay ${n + 1}, past the ${MAX_RELAYS} a row gets` : null })
  return report
}

function laneAgent(lane, prompt, label, relay) {
  return agent(prompt, {
    label,
    phase: 'Work',
    isolation: 'worktree',
    agentType: 'lane',
    schema: reportSchema(lane, relay),
    model: lane.model,
  })
}

function relayRefused(report) {
  if (typeof report.handoff !== 'string' || !report.handoff.trim()) return 'the lane asked for a relay and wrote no handoff'
  if (typeof report.worktree_path !== 'string' || !report.worktree_path) return 'the lane asked for a relay and named no worktree to take its branch from'
  return null
}

function sameSha(a, b) {
  return typeof a === 'string' && typeof b === 'string' && a.length >= 7 && b.length >= 7 && (a.startsWith(b) || b.startsWith(a))
}

// A row prep stopped started no lane, so there is nothing to verify; it goes
// straight to `handedBack`, which lists the files prep said it needs.
function notPrepared(item) {
  const prep = preps.get(item.label)
  if (!prep.stop) return null
  return stopped(prep.lane, { report: null, check: null, kind: null, relayTrees: [], relayedFrom: null, prep: prep.answer }, prep.stop)
}

// A lane that started somewhere else measured another tree; verifying its
// claims against this base would only tell us they disagree, which we know.
function verified(report, lane) {
  const relay = relayed.get(lane.label) || { trees: [], before: null, refused: null }
  const out = { report, check: null, kind: null, relayTrees: relay.trees, relayedFrom: relay.before }
  if (!report) return stopped(lane, out, 'the lane returned nothing')
  if (relay.refused) return stopped(lane, out, relay.refused)
  if (typeof report.base_sha !== 'string' || report.base_sha.slice(0, 8) !== BASE.slice(0, 8)) {
    return stopped(lane, out, `the lane checked out ${report.base_sha}, not ${BASE.slice(0, 8)}`)
  }
  if (!writes(lane)) return out
  if (report.outcome === 'blocked') return stopped(lane, out, 'the lane is blocked')
  if (report.outcome === 'does_not_reproduce') return gone(lane, report, out)
  if (lane.verify === false) return out
  out.kind = lane.sameBehaviour === true ? 'same_behaviour' : namesTests(report) ? 'tests' : 'moved_text'
  if (out.kind === 'same_behaviour') {
    if (namesTests(report)) return stopped(lane, out, 'the lane is marked sameBehaviour and names a test that fails without its change, which says the behaviour changed')
    if (!reachesTests(report).length) return stopped(lane, out, 'the lane is marked sameBehaviour and names no test that reaches its change')
  }
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

// §433.8: a lane whose row's fault no longer shows made no commit, so there is
// nothing to revert, no gate of its own to read and no branch to merge. Its
// verifier runs the lane's commands on the base again, and a `confirm` is held
// to the runs it reports, as `unproven` holds a `merge`.
function gone(lane, report, out) {
  out.kind = 'does_not_reproduce'
  if (!sameSha(report.landed_sha, BASE)) return stopped(lane, out, `the lane reported does_not_reproduce and landed ${report.landed_sha}, not the base ${BASE.slice(0, 8)}; a lane whose fault does not show commits nothing (§433.8)`)
  if (!Array.isArray(report.base_runs) || !report.base_runs.length) return stopped(lane, out, 'the lane reported does_not_reproduce with no base_runs, so its verifier has nothing to run again (§433.8)')
  return agent(GONE(lane, report), {
    label: `verify:${lane.label}`,
    phase: 'Verify',
    isolation: 'worktree',
    agentType: 'verifier',
    schema: goneSchema(),
    model: 'sonnet',
  }).then((check) => {
    out.check = check
    if (!check) return stopped(lane, out, 'the verifier returned nothing')
    if (check.verdict !== 'confirm') return stopped(lane, out, check.still_shown_by ? `the verifier saw the fault still show: ${check.still_shown_by}` : `the verifier refused it: ${unconfirmed(check) || 'it gave no reason'}`)
    const why = unconfirmed(check)
    if (why) return stopped(lane, out, `the verifier said confirm, but ${why}`)
    return out
  })
}

function unconfirmed(check) {
  if (check.branch_at_base !== true) return 'the lane branch is not at the base, so the lane committed something'
  if (!Array.isArray(check.inputs_left_out) || check.inputs_left_out.length) return `the lane ran none of these inputs the row names: ${(check.inputs_left_out || []).join(', ') || 'the verifier did not say which'}`
  if (!Array.isArray(check.runs) || !check.runs.length) return 'the verifier reported no runs'
  const shown = check.runs.find((run) => !run || run.shows_fault !== false)
  if (shown) return `its run of ${shown && shown.command} did not say the fault is gone`
  return null
}

function stopped(lane, out, why) {
  log(`${lane.label}: not merged -- ${why}`)
  out.stop = why
  return out
}

function namesTests(report) {
  return Array.isArray(report.tests_that_fail_without_the_change) && report.tests_that_fail_without_the_change.length > 0
}

function reachesTests(report) {
  return Array.isArray(report.tests_that_reach_the_change) ? report.tests_that_reach_the_change : []
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
//
// A fifth changes what runs and keeps what it computes: one of two copies
// dropped for the other, a body lifted into a function (Q601, Q854). No test
// can fail without it, so it names none and got the moved-text steps, which
// refuse any line that runs; Q854's dispatcher did that row in its own tree.
// `lane.sameBehaviour` is the row's claim, set by the dispatcher and never
// inferred, for `pinsOnly`'s reason. The tests the lane names reach the change:
// they pass on the branch, pass with the source reverted, and one fails under
// a mutation of the changed lines (§399).
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
  } else if (kind === 'same_behaviour') {
    if (check.tests_pass_on_branch !== true) why.push('the tests do not pass on the branch')
    if (runs !== 0) {
      if (!sourcePathsOf(lane).length) why.push('the lane is marked sameBehaviour and declared no source directory to revert')
      if (check.tests_pass_without_change !== true) why.push('the tests that reach the change were never seen to pass with the source reverted')
      if (!mutationCaught(check.mutation_caught)) why.push('no mutation of the changed lines was seen to fail a test that reaches them')
    }
  } else if (runs) {
    why.push(`the branch changes ${runs} lines that run and names no test that fails without them (a row that keeps what they compute goes again marked sameBehaviour, §399), starting with: ${check.lines_that_run.trim().split('\n')[0]}`)
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

// Passing on the branch and passing with the source reverted hold just as well
// for tests that never run the changed lines. A mutation of those lines that
// one of them catches is what shows they do (§399).
function mutationCaught(m) {
  return !!m && m.outcome_on_branch === 'fail'
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
// Landers in different runs take one, and share one gate (§397.5); the Opus
// merger takes none.
function mergeAfterTheLast(lane, prior) {
  const run = lastMerge.then(() => merging(lane, prior))
  lastMerge = run.catch(() => null)
  return run
}

// §383.4 item 3 and §383.5: a verifier's `merge` on a lane that wrote its rows
// goes to the Haiku lander, which runs `just land` once. A `fix_first`, a lane
// whose rows `just land` cannot take, a `merge` whose own gate_summary did not
// read `gate: PASS` (§414, so a verifier's red gate or an unrun one never
// reaches the lander's gate too), and every merge the recipe stopped on go to
// the Opus merger, with the stop in its brief. A lander that throws is a stop
// too, never the end of the row: a `claude` process started before
// `lander.md` existed cannot resolve the type (§384.3). One that died after its
// push has landed the lane already, and its detached run can still be landing
// it, which is why the merger's brief has it wait with `just land wait-for`,
// then check `origin/main` for the lane's tip, before it merges anything.
function merging(lane, prior) {
  if (prior.kind === 'does_not_reproduce') return closedAtBase(lane, prior)
  if (!landable(prior)) return byMerger(lane, prior, null)
  return agent(LAND(lane, prior), LANDER(lane)).then(
    (land) => land && land.merged === true ? landedBy(land) : byMerger(lane, prior, landStop(land)),
    (e) => byMerger(lane, prior, `the lander threw before it reported (${e && e.message}), so nothing says whether it pushed`),
  )
}

function landable(prior) {
  return !!prior.check && prior.check.verdict === 'merge' && stepsProblem(prior.report.rows) === null && gatePassed(prior.check)
}

// The two lines `_gate-summary` prints are exactly `gate: PASS` and `gate:
// FAIL  first failure: <step> (exit N)` (`just recipe _gate-summary`); a
// served gate prints `gate: PASS` too. `gate_summary`'s own schema description
// is "the summary block, verbatim, or why you could not run it" -- a verifier
// that could not run the gate holds neither line, so it reads as not passed
// here and the row goes to the Opus merger same as an outright FAIL (§414).
function gatePassed(check) {
  return typeof check.gate_summary === 'string' && check.gate_summary.split('\n').some((line) => line === 'gate: PASS')
}

// Null when every step is one `just land` takes: `add <tier> <model> <write
// set> <what>`, `after @N <ids>` or `tag @N <tag>`, each word a line of the
// steps file the lander writes, so no word may be empty, hold a newline, or be
// the `+` that separates steps there.
function stepsProblem(rows) {
  if (!Array.isArray(rows)) return 'the lane reported no rows'
  for (const step of rows) {
    if (!Array.isArray(step) || !step.every(oneLine)) {
      return `the step ${JSON.stringify(step)} is not a list of words`
    }
    if (STEP_WORDS[step[0]] !== step.length) return `the step ${JSON.stringify(step)} is not add <tier> <model> <write set> <what>, after @N <ids>, or tag @N <tag>`
  }
  return null
}

function landedBy(land) {
  land.via = 'merged by just land, which the lander ran'
  return land
}

function landStop(land) {
  if (!land) return 'the lander returned nothing'
  return `just land stopped at ${land.stop || 'no named stop'}: ${land.not_merged_because || 'it gave no reason'}`
}

function byMerger(lane, prior, stop) {
  if (stop) log(`${lane.label}: just land did not merge it, so the Opus merger takes it -- ${stop}`)
  return agent(MERGE(lane, prior, stop), MERGER(lane)).then((m) => {
    if (m && stop) m.via = `the Opus merger took it after ${stop}`
    return m
  })
}

// §433.8: the lane made no commit, so the lander, whose `just land` merges a
// branch, never sees the row. A merger closes it done at the base in one queue
// commit, gated and pushed as a merge is.
function closedAtBase(lane, prior) {
  return agent(CLOSE(lane, prior), CLOSER(lane)).then((m) => {
    if (m) m.via = m.merged === true
      ? 'closed done at the base with nothing merged, its fault no longer showing (§433.8)'
      : 'a close at the base that its verifier confirmed and its merger did not push (§433.8)'
    return m
  })
}

function LANDER(lane) {
  return {
    label: `land:${lane.label}`,
    phase: 'Merge',
    isolation: 'worktree',
    agentType: 'lander',
    schema: landedSchema(),
    model: 'haiku',
  }
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

function CLOSER(lane) {
  return {
    label: `close:${lane.label}`,
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
  if (m.via) row.lines.push(m.via)
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
  const handoff = [prior.report, prior.relayedFrom].find((r) => r && typeof r.handoff === 'string' && r.handoff.trim())
  if (handoff) row.lines.push(`handoff, on ${branchOf(lane)} at ${handoff.landed_sha}: ${handoff.handoff}`)
  if (prior.check) {
    row.lines.push(`verifier said ${prior.check.verdict}${prior.check.notes ? `: ${prior.check.notes}` : ''}`)
    for (const c of prior.check.unbacked_claims || []) row.lines.push(`unbacked: ${c}`)
    for (const c of prior.check.number_mismatches || []) row.lines.push(`number: ${c}`)
    for (const c of prior.check.moved_text_problems || []) row.lines.push(`moved text: ${c}`)
  }
  for (const n of (prior.prep && prior.prep.needs) || []) row.lines.push(`needs ${n.file}: ${n.why}`)
  row.unlock = treesOf(prior)
  return row
}

function treesOf(prior) {
  return (prior.relayTrees || []).concat([prior.report && prior.report.worktree_path, prior.check && prior.check.worktree_path]).filter(Boolean)
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
${START(lane)}
${handOver()}`
}

function READING(lane) {
  return `This lane is read-only: you own no files. Your branch: ${branchOf(lane)} (a checkout convenience only).

${lane.brief}
${START(lane)}`
}

// The lane before has let go of the branch with `git switch --detach`, as
// lane.md tells it to; the `-C` fallback covers one that did not.
function RELAY(lane, before, n) {
  const branch = branchOf(lane)
  return `Queue ${rowsOf(lane)}. You own these files and no others: ${lane.writeSet.join(', ')}.
Your branch: ${branch}, which you carry on: you are relay ${n} of at most ${MAX_RELAYS} (§416).

A lane before you did part of this row, passed its context budget, committed its work to ${branch} at ${before.landed_sha} and handed it to you. Its handoff, which is where you start:

${before.handoff}

The row's brief, as that lane had it:

${lane.brief}

Base of the branch: main at ${BASE}. Your first commands, in this order:
  git log --oneline -1
  git switch ${branch}
  git rev-parse HEAD
  git merge-base --is-ancestor ${BASE} HEAD
  git worktree unlock ${before.worktree_path}
Report what git rev-parse HEAD printed as continued_from, and ${BASE} as base_sha once the ancestor check exits 0. If the switch refuses because ${branch} is checked out in ${before.worktree_path}, run git -C ${before.worktree_path} switch --detach and then the switch again. If it still refuses, if HEAD is not ${before.landed_sha}, or if the ancestor check fails, stop and report it as blocked. Never reach for \`checkout -B\` or \`switch -C\`.

Your report is the row's report, not only your part of it. Carry into it, beside your own, everything of the lane before you that still holds: each number (run it again if a commit since could move it), each test in tests_that_fail_without_the_change, each step in rows, each file in files_changed, its decisions_section, out_of_write_set_needs and remaining.
${handOver()}

The lane before you reported (JSON): ${JSON.stringify(before)}`
}

// Said in the brief as well as in lane.md: a `claude` process keeps the agent
// types it started with through a `/clear`, so a dispatcher cleared after §416
// sends lanes whose lane.md has no relay, from briefs that no longer tell them
// to carry on past the warning (Q1422).
function handOver() {
  return 'When "Lane budget passed" comes and your row is not done, hand over (§416), even if this brief says to carry on past it: commit everything by explicit path, run git switch --detach, and report outcome relay, with landed_sha your last commit and a handoff saying what is done by commit, what is next in order, the places as file, function and lines, what you measured with its command, and what you tried that did not work.'
}

// Prep runs in the dispatcher's tree, whose HEAD carries the `take` commit
// past the base and can carry more, so it is pointed at the base itself.
function PREP(lane) {
  return `Prepare queue ${rowsOf(lane)} for its lane, by prep.md's rules, and answer ok or needs.
Base: main at ${BASE}. Read the code there, with git show ${BASE}:<path> and git grep -n '<pattern>' ${BASE}, not at your tree's HEAD.
${lane.notes ? `The dispatcher's notes, which go into the lane's brief as they stand, after your facts: ${lane.notes}` : 'The dispatcher sent no notes.'}`
}

// What a dispatcher wrote by hand for every row until §396.4: the row, its
// mark and why, what was found at the base, its own notes, and the two notes
// every brief it wrote carried word for word.
function SHORT_BRIEF(lane, answer) {
  const bundle = lane.queueId.includes(',')
  const facts = Array.isArray(answer.facts) && answer.facts.length
    ? `\n\nWhat prep found at the base:\n${answer.facts.map((f) => `- ${f}`).join('\n')}`
    : ''
  const notes = lane.notes ? `\n\nFrom the dispatcher: ${lane.notes}` : ''
  return `Do ${bundle ? 'each row' : 'the row'}. ${bundle ? 'Their lines' : 'Its line'}, as just queue show prints ${bundle ? 'them' : 'it'}:

${answer.rows.map((r) => r.line).join('\n')}${marksOf(answer)}${facts}${notes}

${rowsNote()}

${testsNote()}
`
}

function marksOf(answer) {
  const why = answer.marks_why ? `: ${answer.marks_why}` : ''
  if (answer.pinsOnly === true) return `\n\nThe row is marked pinsOnly (§287)${why}`
  if (answer.sameBehaviour === true) return `\n\nThe row is marked sameBehaviour (§399)${why}`
  return ''
}

function rowsNote() {
  return 'Rows: no `tag` step when the add text already starts with the class; `add` tags it from the text, `queue apply` refuses the repeat, and `just land` stops (Q1047). The class must be the text\'s first word exactly, `refuses/B` and not `refuses/B:`, or `add` does not read it (Q1505). A row\'s text ends with its §N and nothing after it, not even a full stop, or check-queue refuses it and `just land` stops.'
}

function testsNote() {
  return 'Test names in tests_that_fail_without_the_change are bare nextest names matching ^name(::name)*$, never `<binary> <name>` with a space (Q1347). A tooling lane whose test is a `check-*-selftest` recipe names it there too, by its recipe name, not by a nextest path (§287).'
}

function reportSchema(lane, relay) {
  const core = {
    start_sha: { type: 'string', description: 'what git log -1 said BEFORE your switch' },
    base_sha: { type: 'string', description: 'what you switched to' },
    branch: { type: 'string' },
    worktree_path: { type: 'string' },
    summary: { type: 'string', description: 'plain words, at most 10 lines' },
    numbers: { type: 'array', items: numberSchema(), description: 'every figure you report, with its command' },
    out_of_write_set_needs: { type: 'array', items: { type: 'string' } },
    remaining: { type: 'string', description: 'what is left, including what you learned about work you were told not to attempt; each defect with its class (wrong, crash or refuses, then /A to /D, §352.3), each defect you guess at and did not run with no class (§360.5), a fault from a fuzz crash file graded C or D only after its shape was run again with ordinary numbers and no class until then (§440.2), and each item of about ten minutes whose place is known marked small' },
  }
  const required = ['start_sha', 'base_sha', 'branch', 'worktree_path', 'outcome', 'summary', 'numbers', 'remaining']
  if (writes(lane)) {
    core.outcome = { type: 'string', enum: ['fixed', 'limit_demonstrated', 'partial', 'blocked', 'does_not_reproduce', 'relay'] }
    core.base_runs = {
      type: 'array',
      description: 'only with outcome does_not_reproduce, lane.md and §433.8: one run on the base for each input the row names that shows its fault -- crash file, corpus file or shape',
      items: {
        type: 'object',
        required: ['input', 'command', 'printed', 'shows_fault'],
        properties: {
          input: { type: 'string', description: 'the crash file, corpus file or shape, as the row names it' },
          command: { type: 'string', description: 'the exact command you ran, in the foreground' },
          probe: { type: 'string', description: 'for a shape, the path and the whole text of the probe test you wrote, ran and removed; empty otherwise' },
          printed: { type: 'string', description: 'the lines of its print that say whether the fault shows, verbatim' },
          shows_fault: { type: 'boolean' },
        },
      },
    }
    core.handoff = { type: 'string', description: 'only with outcome relay, lane.md and §416: what is done, by commit; what is next, in order; the places, as file, function and lines; what you measured, with its command; and what you tried that did not work' }
    core.landed_sha = { type: 'string', description: 'your last commit' }
    core.decisions_section = { type: 'string', description: 'the section number you claimed with just claim-section, if any' }
    core.files_changed = { type: 'array', items: { type: 'string' } }
    core.tests_that_fail_without_the_change = { type: 'array', items: { type: 'string' }, description: 'empty for a lane that changed only .md files and comments (§375)' }
    core.rows = rowsSchema()
    required.push('landed_sha', 'files_changed', 'tests_that_fail_without_the_change', 'rows')
    if (lane.sameBehaviour === true) {
      core.tests_that_reach_the_change = { type: 'array', items: { type: 'string' }, description: 'the tests that run the lines you changed and pass before and after; your row keeps the behaviour, so no test fails without it and tests_that_fail_without_the_change stays empty (§399)' }
      required.push('tests_that_reach_the_change')
    }
    if (relay > 0) {
      core.continued_from = { type: 'string', description: 'what git rev-parse HEAD printed straight after your git switch onto the branch' }
      required.push('continued_from')
    }
  } else {
    core.outcome = { type: 'string', enum: ['answered', 'partial', 'blocked'] }
    core.rows = rowsSchema()
    required.push('rows')
  }
  const extra = lane.report || {}
  return {
    type: 'object',
    required: required.concat(Array.isArray(extra.required) ? extra.required : []),
    properties: Object.assign(core, extra.properties || {}),
  }
}

// A read-only lane can turn up the same rows a writing one does -- the
// nightly triage row (§418.3) hands back one `add` per fault and found no
// field for them until Q1754, and its rows went into `remaining` as pasted
// JSON a dispatcher then filed by hand.
function rowsSchema() {
  return {
    type: 'array',
    items: { type: 'array', items: { type: 'string' } },
    description: 'the just queue apply steps that add the rows your work leaves, each a list of words: ["add", "<tier>", "<model>", "<write set>", "<what, ending in §N>"], ["tag", "@1", "<class>|small"], ["after", "@1", "<id>[,<id>]"]; @1, @2 are this list\'s own adds, since ids come from a shared counter; empty when you leave nothing to add (§383.4, lane.md)',
  }
}

// Every writing brief a dispatcher sent before §396.4 set this field: a
// verifier reverting a src/ directory to the base takes the #[cfg(test)] pins
// in it back out too, so the lane says how to lay them back.
function forTheVerifier() {
  return {
    properties: {
      for_the_verifier: { type: 'string', description: 'How to lay back every #[cfg(test)] test you add or change in src/ when the source is reverted to the base, without `git checkout <base> -- <src dir>`: the commands, and what each such test prints then. Write `none` when every test you added or changed is in a tests/ directory.' },
    },
    required: ['for_the_verifier'],
  }
}

function prepSchema() {
  return {
    type: 'object',
    required: ['answer', 'rows', 'writeSet', 'model', 'sourcePaths', 'pinsOnly', 'sameBehaviour', 'marks_why', 'facts', 'needs'],
    properties: {
      answer: { type: 'string', enum: ['ok', 'needs'], description: 'ok when a lane can start from the rows as they stand; needs when the work touches a file their write sets do not name' },
      rows: {
        type: 'array',
        description: 'each row, as mise exec -- just queue show <id> prints it',
        items: {
          type: 'object',
          required: ['id', 'model', 'write_set', 'line'],
          properties: {
            id: { type: 'string', description: 'the row\'s id, as Q840' },
            model: { type: 'string', description: 'the row\'s model column, exactly as printed' },
            write_set: { type: 'string', description: 'the row\'s write-set column, exactly as printed: paths joined by commas, or - for none' },
            line: { type: 'string', description: 'the row\'s whole line, exactly as printed; it is the lane\'s brief' },
          },
        },
      },
      writeSet: { type: 'array', items: { type: 'string' }, description: 'the union of the rows\' write sets, leaving out -, never widened or narrowed (§396.4)' },
      model: { type: 'string', enum: ['opus', 'sonnet'], description: 'opus if any row\'s model column says opus, else sonnet (§352.4)' },
      sourcePaths: { type: 'array', items: { type: 'string' }, description: 'what the verifier reverts to the base; empty to default to the crates/<c>/src directories in the write set' },
      pinsOnly: { type: 'boolean', description: 'true only for a row that pins a test and writes a decisions section, with no src/ path in its work; said, never inferred (§287)' },
      sameBehaviour: { type: 'boolean', description: 'true only for a row that changes lines that run and keeps what they compute; said, never inferred, and never beside pinsOnly (§399)' },
      marks_why: { type: 'string', description: 'why the mark you set holds; empty when you set neither' },
      facts: { type: 'array', items: { type: 'string' }, description: 'what you found at the base that the lane needs, one bullet each, each with the file and line it is from' },
      needs: {
        type: 'array',
        description: 'each file the work touches that the rows do not name, and why; empty with ok',
        items: {
          type: 'object',
          required: ['file', 'why'],
          properties: { file: { type: 'string' }, why: { type: 'string' } },
        },
      },
    },
  }
}

// The lane's tree and the verifier's own tree are two different worktrees the
// harness made. Naming only the lane's path here, with the schema saying
// "your own worktree", is what made a verifier echo the lane's path back
// (§263.4): it was never told its own tree has a path at all, let alone how
// to find it.
//
// Since §385 the verifier's fixed steps are `just verify-lane`'s, and the rest
// of this prompt is for what it leaves. Its input travels as the lander's steps
// do (§384.2), as a file the verifier writes with the Write tool, one word to a
// line: the Bash guard refuses `$( )`, variables and loops, and a number's
// command holds quotes and `$`.
function VERIFY(lane, r, kind) {
  const revert = sourcePathsOf(lane).join(', ') || '(none declared; say so and skip the revert)'
  const pinsOnlyNote = lane.pinsOnly === true
    ? '\nThis lane is marked pinsOnly: a verdict, a pinned test and a decisions section, nothing in src/ to revert. Skip the revert step. Step 3 already proved the tests pass; do not run them again to watch that (§370) -- take tests_pass_on_branch from its gate summary the way step 6 says, then report tests_fail_without_fix: false with no reproduced_mutation and say in notes that pinsOnly is why -- that is the expected shape here, not a defect.'
    : ''
  const input = verifyInput(lane, r, kind)
  const unsent = input.unsent.length
    ? `\nThese could not go in the file, since a word of each is empty, a lone +, or more than one line, so check them by hand: ${JSON.stringify(input.unsent)}`
    : ''
  const steps = { tests: 'tests', same_behaviour: 'same behaviour', moved_text: 'moved text' }[kind]
  return `Check lane ${lane.label}, queue ${rowsOf(lane)}, by the ${steps} steps.
Base: ${BASE}
Branch: ${branchOf(lane)}
Write set: ${lane.writeSet.join(', ')}
The lane's worktree, to unlock once you are done with it -- this is not your own tree, and not what you report as worktree_path: ${r.worktree_path}
Source paths to revert: ${revert}${pinsOnlyNote}

Run just verify-lane first: it does verifier.md's fixed steps for you (§385). First run git rev-parse --show-toplevel. Then, with the Write tool, write the lines between the two dashed lines below to target/verify-lane.txt under the path it printed, exactly as they stand: one word to a line, a + line between records, nothing added and nothing left out.
----------
${input.text}
----------
Then run this, as one plain Bash call in the foreground with timeout 600000:
  mise exec -- just verify-lane target/verify-lane.txt
While its last line says the run is still going, run this the same way, as often as it takes:
  mise exec -- just verify-lane wait
Its last line is then one line of JSON. Take every field it holds as it stands and do not run its steps again: skipped says what it skipped and why, for your notes, and left_to_you is what it could not decide, which is yours, beside the reading verifier.md names.${unsent}
If your tree's just has no verify-lane recipe, do verifier.md's steps by hand.

Lines that run -- just verify-lane runs this once, before it checks the branch out, and its JSON holds the command as lines_that_run_command and all it prints as lines_that_run. By hand, in step 1, before you check the branch out, run it once in your own tree, exactly as written, and report it the same way. It prints at most eleven lines, and the last is always lines that run: N:
  ${linesThatRunCommand(lane)}
When N is 0, nothing that runs changed (§375): skip the revert below, say so in notes, and report tests_fail_without_fix: false with no reproduced_mutation if your steps are the tests ones, or no tests_pass_without_change and no mutation_caught if they are the same behaviour ones.
${kind === 'same_behaviour' ? sameBehaviourProof() : revertProof()}
Your own worktree is a different tree the harness made for this check. Run git rev-parse --show-toplevel in it and report exactly that output as worktree_path in your verdict.

The lane claims (JSON): ${JSON.stringify(r)}`
}

function revertProof() {
  return `Revert them and watch the tests fail, then restore them -- that is the default proof for the fail half, and report it as tests_fail_without_fix. Step 3 already proved the tests pass; do not run them again to watch that (§370) -- take tests_pass_on_branch from its gate summary the way step 6 says.
Only when the revert itself cannot fail -- a fix that just moves a floor or a bound, where a value on the wrong side of the old one can still pass a proptest run at most seeds -- report a reproduced_mutation instead: make a small, uncommitted change to the source that reintroduces the kind of bug the fix is meant to catch, run one command unchanged against the base commit and against this branch, and give the exact scratch change, the command, and both outcomes ('pass' or 'fail'). Say in notes why the revert could not fail here. Do not reach for a mutation when a revert would do.`
}

function sameBehaviourProof() {
  return `This lane is marked sameBehaviour (§399): its row changes lines that run and keeps what they compute, so no test can fail without it, and the tests it names are the ones that reach the change. Step 3 already proved the tests pass; do not run them again to watch that (§370) -- take tests_pass_on_branch from its gate summary the way step 6 says.
With the source reverted the tests must pass, not fail: just verify-lane reverts them, runs the tests, and reports tests_pass_without_change: true only when every one ran and passed. A test that fails there says the base answered differently, so the behaviour changed, and that is a reject.
The mutation is yours, verifier.md's same behaviour step 8: with the tree at the branch tip, change one line the diff adds so that it still builds and computes a wrong answer, on the path the lane's tests take, run one of them against it with mise exec -- just t <name>, put the line back, and check that git status --porcelain prints nothing. Report the change, the command and the outcome as mutation_caught. It must fail, on an assertion or a panic and not a build error: a test that passes on the branch, with the source reverted and under the mutation too never ran the lines that changed.
Then read the lane's argument that the old and the new give the same answer on every input, case by case, against the diff. The tests show it only for the inputs they feed; a case the argument leaves out, or one where the two differ, is a reject.`
}

// The records `tooling/verify-lane/verify_lane.py` reads, each its name and
// then its words, joined one to a line with a `+` line between records. A
// number with a word no line can carry is left out and named in the prompt.
function verifyInput(lane, r, kind) {
  const records = [
    ['base', BASE],
    ['branch', branchOf(lane)],
    ['lane-tree', r.worktree_path],
    ['write-set'].concat(lane.writeSet),
    ['source-paths'].concat(sourcePathsOf(lane)),
  ]
  if (lane.pinsOnly === true) records.push(['pins-only'])
  if (lane.sameBehaviour === true) records.push(['same-behaviour'])
  const unsent = []
  const tests = kind === 'tests' ? r.tests_that_fail_without_the_change : kind === 'same_behaviour' ? reachesTests(r) : []
  if (tests.some(oneLine)) records.push(['tests'].concat(tests.filter(oneLine)))
  unsent.push(...tests.filter((t) => !oneLine(t)).map((t) => ({ test: t })))
  for (const n of r.numbers || []) {
    const words = [n.what, n.value, n.command]
    if (words.every(oneLine)) records.push(['number'].concat(words))
    else unsent.push(n)
  }
  return { text: records.map((record) => record.join('\n')).join('\n+\n'), unsent }
}

// A word one line of a file can carry, in the lander's steps and the
// verifier's input alike: not empty, not the `+` that separates, one line.
function oneLine(w) {
  return typeof w === 'string' && w.trim() !== '' && w !== '+' && !w.includes('\n')
}

// A recipe, so the verifier types and reports one short line (§382). It runs
// before the verifier checks the branch out, so the recipe that judges the
// branch is never a copy the branch changed.
function linesThatRunCommand(lane) {
  return `mise exec -- just lines-that-run ${BASE} ${branchOf(lane)}`
}

// §433.8: no change to revert and no gate to read, so none of `just
// verify-lane`'s steps apply. The verifier runs what the lane ran, on the base,
// in a tree of its own.
function GONE(lane, r) {
  const ids = lane.queueId.split(',')
  return `Check lane ${lane.label}, queue ${rowsOf(lane)}, by the does not reproduce steps (§433.8). The lane reported does_not_reproduce: it says it ran every input the row names on the base, saw none show the fault, and committed nothing.
Base: ${BASE}
Branch: ${branchOf(lane)}
The lane's worktree, to unlock first -- this is not your own tree, and not what you report as worktree_path: ${r.worktree_path}
Do not run just verify-lane: there is no change to revert and no gate to read.
Check that git rev-parse ${branchOf(lane)} prints ${BASE}, and report branch_at_base from it; when it does not, the lane committed something, and that is a refuse.
Read the row with ${ids.map((id) => `mise exec -- just queue show ${id}`).join(', then ')}. Every crash file, corpus file or shape it names that shows its fault must have a run in the lane's base_runs; one with none goes in inputs_left_out, and that is a refuse.
Then, in your own tree: git checkout --detach ${BASE}
Run each command in base_runs as it stands, one plain Bash call each, in the foreground with timeout 600000. For a run with a probe, write the probe test at its path with the Write tool, exactly as base_runs gives it, run the command, then delete the file, so that git status --porcelain prints nothing. Report each run's command, the lines of its print that say whether the fault shows, verbatim, and shows_fault.
Confirm only when every input has a run and none shows the fault. Otherwise refuse, and name in still_shown_by the command whose print still shows it.
Your own worktree is a different tree the harness made for this check. Run git rev-parse --show-toplevel in it and report exactly that output as worktree_path.

The lane claims (JSON): ${JSON.stringify(r)}`
}

function MERGE(lane, prior, stop) {
  const r = prior.report
  const branch = stop ? afterLandBranchOf(lane) : mergeBranchOf(lane)
  const verifier = prior.check
    ? `the verifier's, ${prior.check.worktree_path}`
    : 'none: the dispatcher set verify: false for this lane, so your gate is the only check'
  const landed = stop
    ? `just land ran first and did not merge it: ${stop}
So your branch is ${branch}, not merge-${lane.label}, which the lander may have made. The lander's run can outlive the lander (§397.5), so straight after your switch, before you merge anything, run this as one plain Bash call with timeout 600000, and again for as long as its last line says still running:
  mise exec -- just land wait-for ${branchOf(lane)}
Then run these two:
  git fetch origin
  git merge-base --is-ancestor ${branchOf(lane)} origin/main
Exit 0 from the second means the lander's run pushed before it stopped or died: merge nothing, and finish by merger.md's paragraph on just land.`
    : 'The lander did not run: the verifier asked for fixes, the lane wrote no rows just land takes, or the gate the verifier ran did not read gate: PASS.'
  return `Merge queue ${rowsOf(lane)}, lane ${lane.label}, onto main, or say why not.
Lane branch: ${branchOf(lane)}, which it reports ending at ${r.landed_sha}.
The lane's write set: ${lane.writeSet.join(', ')}
Trees to unlock: the lane's, ${r.worktree_path}; ${relayedFrom(prior)}${verifier}
Only if your push went through, after unlocking your own tree: mise exec -- just mark-finished ${treesOf(prior).join(' ')} <your own tree>
Your first commands, in this order:
  git log --oneline -1
  git fetch origin
  git switch -c ${branch} origin/main
If the switch refuses because ${branch} already exists, stop and report it as not merged. Never reach for \`checkout -B\` or \`switch -C\`.
${landed}
Push attempts: at most ${MERGE_ATTEMPTS}. A catch-up over docs alone costs no attempt; at most ${DOCS_CATCH_UPS} of those, reported as docs_catch_ups.
The lane's rows are its add steps for your queue apply, as merger.md step 3 says.

The lane's report (JSON): ${JSON.stringify(r)}

The verifier's verdict (JSON): ${JSON.stringify(prior.check)}`
}

function relayedFrom(prior) {
  const trees = prior.relayTrees || []
  return trees.length ? `the lanes it was relayed from, ${trees.join(', ')}; ` : ''
}

// The rows close at the base, not at a lane tip: the lane made no commit, and
// the base is where its verifier ran the row's inputs and saw none show the
// fault (§433.8). `queue done` takes any sha that names a commit.
function CLOSE(lane, prior) {
  const r = prior.report
  const ids = lane.queueId.split(',')
  const close = ids.length > 1
    ? `mise exec -- just queue apply ${ids.map((id) => `done ${id} ${BASE}`).join(' + ')}`
    : `mise exec -- just queue done ${lane.queueId} ${BASE}`
  return `Close queue ${rowsOf(lane)}, lane ${lane.label}, done at the base, by merger.md's section on a row whose fault no longer shows (§433.8), or say why not. Nothing merges: the lane made no commit, and its verifier ran the row's inputs on the base and saw none show the fault.
Base: main at ${BASE}.
Trees to unlock: the lane's, ${r.worktree_path}; ${relayedFrom(prior)}the verifier's, ${prior.check.worktree_path}
Only if your push went through, after unlocking your own tree: mise exec -- just mark-finished ${treesOf(prior).join(' ')} <your own tree>
Your first commands, in this order:
  git log --oneline -1
  git fetch origin
  git switch -c close-${lane.label} origin/main
If the switch refuses because close-${lane.label} already exists, stop and report it as not merged. Never reach for \`checkout -B\` or \`switch -C\`.
The close, as one command:
  ${close}
Then gate and send it on as merger.md's step 4 says, and finish by its step 5. File no rows: the lane left none.
Push attempts: at most ${MERGE_ATTEMPTS}. A catch-up over docs alone costs no attempt; at most ${DOCS_CATCH_UPS} of those, reported as docs_catch_ups.

The lane's report (JSON): ${JSON.stringify(r)}

The verifier's check (JSON): ${JSON.stringify(prior.check)}`
}

// Every argument the lander types is literal text, since the Bash guard of a
// worktree-isolated agent refuses `$( )`, variables and loops, and the command
// must start with the exact text `.claude/settings.json` allows. The steps are
// the one argument a shell would have to quote: a row's text holds quotes,
// backticks and `§`. So they travel as a file the lander writes with the Write
// tool, one word to a line, which no shell ever reads (§384.2).
function LAND(lane, prior) {
  const r = prior.report
  const write = r.rows.length
    ? `First run git rev-parse --show-toplevel. Then, with the Write tool, write the lines between the two dashed lines below to target/land-steps.txt under the path it printed, exactly as they stand: one word to a line, a + line between steps, nothing added and nothing left out.
----------
${r.rows.map((step) => step.join('\n')).join('\n+\n')}
----------
`
    : 'The lane left no rows to add, so there is no steps file to write.\n'
  return `Land queue ${rowsOf(lane)}, lane ${lane.label}, with just land, and hand back the JSON it ends with.
${write}Then run this, once, as one plain Bash call in the foreground with timeout 600000:
  ${landCommand(lane, prior)}
While its last line says land: still running, run this the same way, as often as it takes, and nothing else:
  mise exec -- just land wait
Its last line is then one line of JSON. Report every field of it exactly as it stands.`
}

function landCommand(lane, prior) {
  const r = prior.report
  return `mise exec -- just land ${r.rows.length ? '--steps target/land-steps.txt ' : ''}${lane.queueId} ${branchOf(lane)} ${r.landed_sha} ${mergeBranchOf(lane)} ${treesOf(prior).join(' ')}`
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
  } else if (kind === 'same_behaviour') {
    properties.tests_pass_on_branch = { type: 'boolean', description: "from step 3's gate summary (served or fresh), not a separate run -- §370" }
    properties.tests_pass_without_change = { type: 'boolean', description: 'true when every test that reaches the change ran and passed with the source paths put back to the base; false when one failed there, which says the behaviour changed (§399)' }
    properties.mutation_caught = {
      type: 'object',
      description: 'A small, uncommitted change to a line the diff adds, so that it still builds and computes a wrong answer, and one of the lane\'s tests run against it on the branch. It must fail, on an assertion or a panic and not a build error: passing with and without the change holds just as well for a test that never runs the changed lines (§399). Leave it out only when lines that run is 0.',
      properties: {
        scratch_change: { type: 'string', description: 'the change, as a diff or an exact description' },
        command: { type: 'string', description: 'the exact test command, run with the change in place' },
        outcome_on_branch: { type: 'string', enum: ['pass', 'fail'], description: 'what the command produced on this branch with the change in place -- expected \'fail\'' },
      },
      required: ['scratch_change', 'command', 'outcome_on_branch'],
    }
    required.push('tests_pass_on_branch')
  } else {
    properties.paragraphs_checked = { type: 'integer', description: 'how many removed or moved paragraphs you traced' }
    properties.moved_text_problems = { type: 'array', items: { type: 'string' }, description: 'a paragraph that landed nowhere, twice, or with its meaning changed' }
    required.push('paragraphs_checked', 'moved_text_problems')
  }
  return { type: 'object', required, properties }
}

function goneSchema() {
  return {
    type: 'object',
    required: ['worktree_path', 'branch_at_base', 'runs', 'inputs_left_out', 'verdict', 'still_shown_by'],
    properties: {
      worktree_path: { type: 'string', description: 'the output of git rev-parse --show-toplevel run in your OWN tree, not the lane\'s worktree path given in the prompt' },
      branch_at_base: { type: 'boolean', description: 'true only when git rev-parse of the lane branch printed the base' },
      runs: {
        type: 'array',
        description: 'each command of the lane\'s base_runs, run as it stands with your tree at the base',
        items: {
          type: 'object',
          required: ['command', 'printed', 'shows_fault'],
          properties: {
            command: { type: 'string' },
            printed: { type: 'string', description: 'the lines of its print that say whether the fault shows, verbatim' },
            shows_fault: { type: 'boolean' },
          },
        },
      },
      inputs_left_out: { type: 'array', items: { type: 'string' }, description: 'each crash file, corpus file or shape the row names that the lane\'s base_runs has no run of' },
      verdict: { type: 'string', enum: ['confirm', 'refuse'] },
      still_shown_by: { type: 'string', description: 'with refuse, the command whose print still shows the fault; empty otherwise' },
      notes: { type: 'string', description: 'at most five lines, for the dispatcher: what no other field says' },
    },
  }
}

function numberSchema() {
  return {
    type: 'object',
    required: ['what', 'value', 'command'],
    properties: {
      what: { type: 'string' },
      value: { type: 'string', description: 'text the command prints, exactly as it prints it: the verifier\'s just verify-lane looks for it in that print (§383.4, §385)' },
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

// `just land`'s JSON is `mergedSchema`'s fields and the stop it named, so the
// lander's report goes through `merged()` exactly as a merger's does.
function landedSchema() {
  const schema = mergedSchema()
  schema.properties.stop = { type: 'string', description: 'the stop just land named, as its JSON has it; empty when it merged' }
  schema.required = schema.required.concat('stop')
  return schema
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
