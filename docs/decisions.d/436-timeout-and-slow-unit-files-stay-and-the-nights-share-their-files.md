## 436. Timeout and slow-unit files stay, and the nights share their files — 2026-10-11

*Mattias, 2026-10-11: "What do we do with: the 13 092 timeout and 6 718
slow-unit files, which are carried forward and never replayed, and the 3.0 GB of
dated night directories." These are the two things §434.6 left out. The 3.0 GB
was §434.1's figure, and it was wrong: it is 3.7 GB (436.3).*

In short: the timeout and slow-unit files stay where they are. They are still
not replayed, and they are not folded into the corpus. They cost disk and
nothing else, and most of that disk was the nights copying them again, along
with everything else, every night. So the nights stop copying what they already
hold. A file a night holds that the night before it also held, with the same
bytes, becomes a hard link to that night's file. Every night keeps its whole
directory at the same path, so nothing that reads one changes. Q1753 builds it
for every night from now on. The nights already on disk were linked by hand
(436.4).

### 436.1 What the files are

A `timeout-` file is an input that ran past `-timeout=10` and was killed
(§135). A `slow-unit-` file is one that finished but took at least ten seconds,
and was the slowest yet in its fork job: `FuzzerLoop.cpp:453`, with
`report_slow_units` at its default of 10, since nothing here sets it. Both are
slow inputs, and neither is a fault (§135.4).

In 10-11's `artifacts/`:

| target | timeout | slow-unit |
|---|---|---|
| where_two_surfaces_cross | 12 581 | 6 344 |
| two_solids_fused_by_a_stranger | 389 | 259 |
| no_model_makes_the_writer_panic | 114 | 97 |
| where_a_cone_or_a_torus_crosses | 7 | 17 |
| no_step_file_makes_the_reader_panic | 1 | 1 |

The 13 092 timeouts hold 2.5 MB of bytes and take 51.6 MB of disk. The 6 718
slow-units hold 1.6 MB and take 26.5 MB. Every file takes at least one 4 KB
block, so the number of files is what costs disk, not their size. 10-11's night
added 1 156 timeouts and 482 slow-units, nearly all on
`where_two_surfaces_cross` (its `latest` line).

### 436.2 Why they stay

Three things could be done with them instead. None is better.

**Replay them each morning, as crash files are.** A replay can take up to ten
seconds per file, so 13 092 timeouts could take 36 hours. One that hangs stalls
the single replay process: the first run over 09-26, with its 3 072 timeouts,
sat for 21 CPU-minutes and was killed (the comment in `fuzz-triage`). And a slow
input has no panic to sort it into a group. `fuzz-triage` already decided this,
and nothing about it has changed.

**Fold them into the corpus, as §434 does with crash files that pass.** Every
night replays its corpus before it fuzzes, and the slowest inputs in the corpus
are what eat the night: §135.1 found that 99% of `where_two_surfaces_cross`'s
inputs took at most 7% of its night. A timeout file that the fold's `-merge=1
-timeout=10` reads to the end now runs in under ten seconds, but only just. A
slow-unit took ten seconds or more. Folding either one puts back the inputs
`-timeout=10` was set to keep out.

**Delete them once the night has counted them.** No, for two reasons.

- The night's "new tonight" count compares the names in `fuzz/artifacts/`
  against the names it held when the night began (`artifact-names-before`).
  `fuzz-yield`'s count, which the queue page shows, compares each night's names
  against the night before. On 10-11's night, 48 names the tree already held
  were written again:

  | target | timeout | slow-unit |
  |---|---|---|
  | where_two_surfaces_cross | 21 of 11 436 | 10 of 5 869 |
  | no_model_makes_the_writer_panic | 10 of 107 | 3 of 92 |
  | two_solids_fused_by_a_stranger | 3 of 385 | 1 of 257 |
  | where_a_cone_or_a_torus_crosses | 0 of 7 | 0 of 17 |
  | no_step_file_makes_the_reader_panic | 0 of 1 | 0 of 1 |

  "Written again" means the name is in 10-11's `artifact-names-before` and its
  file in the nightly tree changed after the night began, at 22:13:42Z on
  10-10. libFuzzer rewrites a name it has written before (Q51), and nothing
  else writes into the tree's `fuzz/artifacts/`. With those files deleted, each
  of the 48 would have counted as new. On `where_two_surfaces_cross` that is 31
  more on top of 1 620. On the writer it is 13 more on top of 12: its 7 new
  timeouts would have read 17. So deleting the files needs a list of names kept
  in their place, read by the night's count and by `fuzz-yield`.
- The bytes have been used. Q188, Q239 and Q270 rebuilt a corpus out of 09-26's
  timeout files, to measure what slow seeds cost at the start of a fork-mode
  run (§275.5, §297). `fuzz-promote` takes a slow-unit file and adds it to the
  tracked corpus under its own name.

Their only cost is disk, and 436.3 shows that most of it was the nights copying
them. Once the nights share their files (436.4), they cost one copy in the
nightly tree and one copy shared by all the nights. Each copy grows by about
1 640 files a night, which is 6.7 MB, or 2.4 GB a year. That is not worth
building a list of names to save.

### 436.3 What the nights cost

Measured on 10-11, before the nights were linked (436.4).

- `du -sh ~/rcad-fuzz-logs` gives 3.7 GB. The 3.0 GB in Mattias's question came
  from §434.1, which got it from one `du` given two `artifacts/` directories and
  then the whole tree. `du` counts a file only once across all its arguments,
  so the tree's line left those two directories out. Give `du` one argument at
  a time. §434.1 was corrected in 8b96c38d.
- Nearly all of it is the nights' `artifacts/`: 3.5 GB, summed over the 21
  nights that have one. Each night copies the whole of the tree's
  `fuzz/artifacts/` into a directory of its own, so every night is a full copy
  of the one before it, plus that night's new files.
- Across all nights there are 99 169 distinct names, and every one of them is
  in 10-11's copy. One copy of each takes 408 MB. So 3.1 GB of the 3.5 GB is
  copies of files that 10-11's directory also holds. All of 09-27's names are
  in 10-03's copy too: 09-28 has no directory, and 09-29 to 10-02 have a build
  log and no `artifacts/`.
- No file in 10-11's copy shares an inode with the tree. Both are on one
  filesystem.
- The growth speeds up. 10-11's copy is 408 MB, 18 MB more than 10-10's, and
  every night adds a new whole copy. At that rate, the 280 GB free on `/` at
  19:30Z on 10-11 would run out in about 160 nights. Lanes fill the same disk, and a full
  disk shows up as an LLVM IO error that only Mattias can clear. Q1731 makes
  each copy smaller, since most crash files will leave the tree, but each night
  would still add a whole copy of a pile that grows every night.

### 436.4 The nights share their files

**The rule.** When the nearest earlier night holds a file with the same name and
the same bytes as a file in tonight's `artifacts/`, tonight's file is a hard
link to that night's file. A file the earlier night does not hold is copied from
the tree, as it is now.

- **What does not change.** Every night's `artifacts/<target>/` still holds
  exactly the names the tree held, with the same bytes, at the same path.
  `fuzz-yield`, the queue page, `fuzz-triage`, `fuzz-promote`, §434.3's sixth
  rule, and every row and section that cites a night's file by its path read
  it as before. Deleting one night removes only that night's names. The bytes
  stay for every other night that holds them.
- **Never a link into the tree.** libFuzzer rewrites a file it has written
  before by opening it with `fopen(…, "wb")` (`FuzzerIO.cpp:82`). That empties
  the file, and then the bytes are written again. A fork worker killed between
  the two leaves the file short. A night linked to the tree would be cut short
  along with it, and so would every later night linked to that one.
- **Links between nights are safe because nothing writes into a night's
  `artifacts/` after its night.** Checked on 10-11: in the justfile, only
  `copy_out` writes there, and only during the night. `fuzz-triage` writes its
  files next to `artifacts/`, not inside it. `tooling/fuzz-yield` and
  `tooling/queue-page` only read. A change that starts writing into a night's
  `artifacts/` breaks this rule.
- **The same bytes, checked, not assumed from the name.** The copier runs every
  five minutes while the targets write, so it can copy a file that libFuzzer
  has only half written, and `cp -n` then keeps the short copy for the rest of
  the night. None has happened yet: all 99 169 files, in 10-11's copy and in the
  tree, hash to the SHA-1 in their names. But today a short copy lasts one
  night, because the next night copies afresh from the tree. A link would carry
  it into every night after. So a file is linked only after its bytes are
  checked to be the same as the tree's.
- **Done after the night's last `copy_out`.** By then every target has stopped,
  so nothing is writing into the tree. A night killed before that step is a
  full copy, the same as every night is today.
- **`du` reads differently after this.** `du -s` of one night counts every file
  it holds, shared or not. The real total is `du -s ~/rcad-fuzz-logs` given on
  its own. What one night adds is its new files and its directory entries: the
  entries in 10-11's `artifacts/` take 8.3 MB.
- **Built into `fuzz-overnight`, right after the night's last `copy_out`**
  (Q1753, first `3c48eb5f` then `cbff930b`). `cbff930b` replaced the first
  lane's shell loop — which forked `cmp`, `mktemp`, `ln` and `mv` once per
  file, and was correct but had not finished linking `cp -R` copies of two
  real nights (86 838 and 91 080 files) after 220 s — with one Python process
  that walks `$night/artifacts/` itself: `os.path.isfile` for `[ -f ]`,
  `filecmp.cmp(..., shallow=False)` for `cmp -s`, `os.link` then
  `os.replace` for `ln -f` then `mv -f`, none of them forking a process.
  Timed on the same two `cp -R` copies, placed under `target/` and removed
  once measured, the one-pass walk took 1.1-1.3 s. `check-fuzz-overnight-selftest`'s
  `thirty`, `thirty_one` and `thirty_two` pin the four rules above.

**The nights already on disk were linked by hand on 2026-10-11**, at 19:51Z,
outside the night's hours. This covered only the dated nights' `artifacts/`:
not the nightly tree, not the corpus, and not the rescued or experiment
directories. `hardlink -n -t` (util-linux 2.39.3) over the 21 nights was still
comparing after 13 CPU-minutes, so it was stopped and nothing was linked with
it. Instead, each file was linked to the earliest night's file with the same
target and name, but only after checking that the earliest file hashed to its
name and that the two files' bytes were equal. 753 669 files were linked, none
differed, and the run took 48 s. `du -s ~/rcad-fuzz-logs` went from 3 704 MB to
752 MB. Afterwards every night held the same names as before, and all 852 838
files hashed to their names, on 99 169 inodes.

### 436.5 What was weighed and not taken

- **Delete old nights after some number of days.** Every name in an old night
  is in 10-11's copy, so until Q1731 lands, deleting old nights loses no file.
  But 40 lines in `docs/queue.md` and 23 sections cite a night's file by its
  path (`git grep 'rcad-fuzz-logs/2026-'`). `fuzz-yield`'s count for a night needs the night before it. And once
  Q1731 lands, a crash file removed from the tree is kept only in the nights
  (§434.3, rule 6). With links, an old night costs only its directory entries,
  so deleting it saves almost nothing.
- **One shared store, plus a list of names for each night.** It takes the same
  disk as links, and every reader and every citation would have to move to a
  new path.
- **Link each night straight to the tree (`cp -Rl`).** One flag to change, and
  wrong: see 436.4's second bullet.

### 436.6 What this leaves

- **Q1753 is done**, by `3c48eb5f` and then `cbff930b` (436.4): `fuzz-overnight`
  links by name and hash, comparing against the tree, and `check-fuzz-overnight-selftest`'s
  `thirty`, `thirty_one` and `thirty_two` pin the four rules. For scale: hand-linking
  by name over all 21 nights took 48 s (436.4's last paragraph); the built
  recipe's own one-pass walk of the two heaviest single nights (86 838 and
  91 080 files) took 1.1-1.3 s.
- **Nothing for the timeout and slow-unit files** (436.2).
