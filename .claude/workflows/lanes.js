// Run a batch of lanes against one pinned base, and check the writing ones.
//
// Two runs on 2026-09-19 and 2026-09-20 used scripts that were ~90% the same
// text, retyped the second time. This is that text, once. What it keeps from
// both is the part that turns CLAUDE.md's prose into something the script
// enforces: every lane reports a structured record, every number it reports
// carries the command that produced it, and a lane that did not start from the
// pinned base is not verified at all -- its measurements are about another tree.
//
// Invoke it as the `lanes` workflow, which is `meta.name` below and not this
// file's name: Claude Code loads `.claude/workflows/*.js`, parses `meta`, and
// registers what `meta.name` says. Pass args:
//
//   { base: "<sha>",
//     lanes: [ { label, brief, writeSet, ... }, ... ] }
//
// Per lane: `label` (required, names the branch and the progress row), `brief`
// (required, what the lane is for -- the shared block below is appended),
// `writeSet` (the files it owns; absent or empty means read-only, which also
// means no verifier and no commit), `branch` (default `lane-<label>`, and it
// must not exist yet: the lane creates it and refuses to reset one), `model`,
// `sourcePaths` (what the verifier reverts to base to prove the tests fail
// without the change; defaults to the `crates/<c>/src` dirs in `writeSet`),
// `report` ({properties, required} merged into the lane's schema for findings
// that are this lane's own), and `verify: false` to skip the check.
export const meta = {
  name: 'lanes',
  description: 'Run rcad lanes in parallel worktrees off one pinned base, then independently verify each writing lane',
  whenToUse: 'Dispatching two or more rcad lanes at once against a known-green base sha.',
  phases: [
    { title: 'Work', detail: 'one isolated worktree per lane' },
    { title: 'Verify', detail: 're-derive each writing lane\'s claims from its branch', model: 'sonnet' },
  ],
}

if (!args || typeof args !== 'object') throw new Error('lanes: pass args {base, lanes: [...]}')
const BASE = args.base
if (typeof BASE !== 'string' || BASE.length < 7) throw new Error('lanes: args.base must be a full sha of the tree the lanes start from')
const LANES = Array.isArray(args.lanes) ? args.lanes : []
if (!LANES.length) throw new Error('lanes: args.lanes is empty')
for (const lane of LANES) {
  if (!lane || !lane.label || !lane.brief) throw new Error('lanes: every lane needs a label and a brief')
}

const writes = (lane) => Array.isArray(lane.writeSet) && lane.writeSet.length > 0
const branchOf = (lane) => lane.branch || `lane-${lane.label}`

// Defaulted rather than asked for, because the verifier's revert step only
// works on non-test source: reverting the tests too would put back the version
// that passes and prove nothing.
const sourcePathsOf = (lane) => {
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

const COMMON = (lane) => `
Base: main at ${BASE}. Your first two commands, in this order, and report both shas back:
  git log --oneline -1
  git switch -c ${branchOf(lane)} ${BASE}
A worktree-isolated lane branches from the session's HEAD, not from origin/main, so the switch above is what puts you on the base -- do not skip it and do not assume you are already there.
If the switch refuses because ${branchOf(lane)} already exists, stop and report it as blocked. Never reach for \`checkout -B\` or \`switch -C\`: they reset a branch that another worktree may still have checked out, and that tree then shows a staged revert of everything since its commit.
Every cargo and just call needs a \`mise exec --\` prefix (the compile cache is wired through mise's env). Run single tests with \`mise exec -- just t <args>\`, never bare cargo nextest.
Read then Edit; never patch a tracked file through a shell heredoc.
The Bash guard in a worktree-isolated session refuses any command with a path or argument it cannot resolve before the run -- no variables, no \`$( )\`, no loop over a runtime value, and no \`mise exec\` inside a loop or compound. Write out paths literally, one plain command per call; when you genuinely need a loop, write a script to \`target/\` (gitignored) and run it as \`./target/<name>.sh\`.
Budget ~45 minutes. If the work is bigger, commit what is done, report what is left, and stop.
Never restate a number you did not run yourself; every number you report carries the command that produced it.
Do not merge anything into main and do not push.`

const WRITING = (lane) => `You own these files and no others: ${lane.writeSet.join(', ')}. Anything outside them, stop and report it in out_of_write_set_needs rather than editing it.
Stage with explicit \`git add <path>\`, never -A. Commit as you go.
Your branch: ${branchOf(lane)}.

${lane.brief}
${COMMON(lane)}`

const READING = (lane) => `You write no tracked file and make no commit. Your branch: ${branchOf(lane)} (a checkout convenience only).

${lane.brief}
${COMMON(lane)}`

const NUMBER = {
  type: 'object',
  required: ['what', 'value', 'command'],
  properties: {
    what: { type: 'string' },
    value: { type: 'string' },
    command: { type: 'string', description: 'the exact command you ran to get this value' },
  },
}

const reportSchema = (lane) => {
  const core = {
    start_sha: { type: 'string', description: 'what git log -1 said BEFORE your switch' },
    base_sha: { type: 'string', description: 'what you switched to' },
    branch: { type: 'string' },
    worktree_path: { type: 'string' },
    summary: { type: 'string', description: 'plain words, at most 10 lines' },
    numbers: { type: 'array', items: NUMBER, description: 'every figure you report, with its command' },
    out_of_write_set_needs: { type: 'array', items: { type: 'string' } },
    remaining: { type: 'string', description: 'what is left, including what you learned about work you were told not to attempt' },
  }
  const required = ['start_sha', 'base_sha', 'branch', 'outcome', 'summary', 'numbers', 'remaining']
  if (writes(lane)) {
    core.outcome = { type: 'string', enum: ['fixed', 'limit_demonstrated', 'partial', 'blocked'] }
    core.landed_sha = { type: 'string', description: 'your last commit' }
    core.decisions_section = { type: 'string', description: 'the section number you claimed with just claim-section, if any' }
    core.files_changed = { type: 'array', items: { type: 'string' } }
    core.tests_that_fail_without_the_change = { type: 'array', items: { type: 'string' } }
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

const VERIFY = (lane, r) => `You are checking another lane's claims; do not trust its report. Write no tracked file, make no commit.

The lane claims (JSON): ${JSON.stringify(r)}

1. \`git log --oneline ${BASE}..${branchOf(lane)}\` -- confirm the branch exists and sits on ${BASE}. Then \`git diff --name-only ${BASE}..${branchOf(lane)}\` and flag every file outside the lane's write set: ${lane.writeSet.join(', ')}.
2. \`git checkout --detach ${branchOf(lane)}\` and run the claimed tests with \`mise exec -- just t <name>\` -- they must pass.
3. Put the non-test source back to base -- ${sourcePathsOf(lane).map((p) => `\`git checkout ${BASE} -- ${p}\``).join(', ') || '(the lane declared no source dir; say so and skip this step)'} -- and rerun the same tests. They must FAIL, and record the exact failure: a test that passes here proves nothing about the change. Then put the branch's source back the same way before going on.
4. \`mise exec -- just gate quiet\` on the branch tip, including any docs commit -- one plain Bash call, timeout 600000, no redirect and no pipe through tail (a pipe hands you tail's exit code, not the gate's). Record the summary block verbatim.
5. Re-run every command in the lane's numbers list and compare the values you get.
6. Read the lane's new decisions section if it claimed one. Per §16.10 every "cannot", "unreachable" or "limit" claim owes a demonstration and not a paragraph -- say which ones have one. If the lane demonstrated a limit rather than fixing the thing, check that demonstration hardest: does it name the family of fixes it considered and say why that family is the only one?
Every cargo/just call needs a \`mise exec --\` prefix. The Bash guard refuses runtime-computed paths; write them out literally.`

const VERDICT = {
  type: 'object',
  required: ['branch_ok', 'tests_pass_on_branch', 'tests_fail_without_fix', 'gate_summary', 'number_mismatches', 'unbacked_claims', 'verdict'],
  properties: {
    branch_ok: { type: 'boolean', description: 'exists, sits on the pinned base' },
    files_outside_write_set: { type: 'array', items: { type: 'string' } },
    tests_pass_on_branch: { type: 'boolean' },
    tests_fail_without_fix: { type: 'boolean', description: 'false if you could not make them fail with the source reverted' },
    gate_summary: { type: 'string', description: 'the summary block, verbatim, or why you could not run it' },
    number_mismatches: { type: 'array', items: { type: 'string' } },
    unbacked_claims: { type: 'array', items: { type: 'string' } },
    verdict: { type: 'string', enum: ['merge', 'fix_first', 'reject'] },
    notes: { type: 'string' },
  },
}

log(`${LANES.length} lane(s) off ${BASE.slice(0, 8)}: ${LANES.map((l) => `${l.label}${writes(l) ? '' : ' (read-only)'}`).join(', ')}`)

const results = await pipeline(
  LANES,
  (lane) => agent(writes(lane) ? WRITING(lane) : READING(lane), {
    label: lane.label,
    phase: 'Work',
    isolation: 'worktree',
    schema: reportSchema(lane),
    model: lane.model,
  }),
  (report, lane) => {
    if (!report) return { lane: lane.label, report: null, check: null, skipped: 'lane returned nothing' }
    // A lane that started somewhere else measured another tree; verifying its
    // claims against this base would only tell us they disagree, which we know.
    if (typeof report.base_sha !== 'string' || report.base_sha.slice(0, 8) !== BASE.slice(0, 8)) {
      log(`${lane.label}: checked out ${report.base_sha}, not ${BASE.slice(0, 8)} -- not verifying`)
      return { lane: lane.label, report, check: null, skipped: 'wrong base' }
    }
    if (!writes(lane)) return { lane: lane.label, report, check: null, skipped: 'read-only lane' }
    if (lane.verify === false) return { lane: lane.label, report, check: null, skipped: 'verify disabled for this lane' }
    if (report.outcome === 'blocked') return { lane: lane.label, report, check: null, skipped: 'lane blocked' }
    if (!report.tests_that_fail_without_the_change || !report.tests_that_fail_without_the_change.length) {
      return { lane: lane.label, report, check: null, skipped: 'lane named no test that fails without the change' }
    }
    return agent(VERIFY(lane, report), {
      label: `verify:${lane.label}`,
      phase: 'Verify',
      isolation: 'worktree',
      schema: VERDICT,
      model: 'sonnet',
    }).then((check) => {
      // A `merge` that did not watch the tests fail is a `merge` on the
      // verifier's word rather than on a demonstration, which is the whole
      // reason there is a verifier. Say so here; do not leave it to whoever
      // reads the JSON to notice which field was false.
      const unproven = []
      if (check && check.verdict === 'merge') {
        if (check.tests_fail_without_fix !== true) unproven.push('the tests were never seen to fail with the source reverted')
        if (!sourcePathsOf(lane).length) unproven.push('the lane declared no source directory to revert')
        if (check.files_outside_write_set && check.files_outside_write_set.length) unproven.push(`the branch touches ${check.files_outside_write_set.join(', ')}, outside the write set`)
      }
      if (unproven.length) log(`${lane.label}: verifier said merge, but ${unproven.join('; ')}`)
      return { lane: lane.label, report, check, unproven: unproven.length ? unproven : undefined }
    })
  },
)

const checked = results.filter((r) => r && r.check)
const clean = checked.filter((r) => r.check.verdict === 'merge' && !r.unproven)
log(`done: ${checked.length} of ${LANES.length} lane(s) verified, ${clean.length} of those came back merge with nothing unproven`)
for (const r of results) {
  if (r && r.skipped) log(`${r.lane}: not verified -- ${r.skipped}`)
}

return { base: BASE, lanes: results }
