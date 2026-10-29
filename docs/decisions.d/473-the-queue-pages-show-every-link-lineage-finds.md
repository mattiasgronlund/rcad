## 473. The queue pages show every link lineage finds; one parent is only for counting — 2026-10-29

*Mattias, 2026-10-29, after asking where Q2959 came from and being given §472
in more detail: "I would like to have as much information as possible linked
in the web-pages allowing me to understand the data. Not a lot of limits saying
that "this aspect can not be of interest". We should make sure that Came from,
include the filename at the end, and if I click that one, there should be a
web-page showing all rows whos work has been related to that file."*

This builds on §444's row pages and §472's two parents. It changes no count.

### 473.1 What one parent hid

§433.2 gives a row one parent, so that each fuzz target and STEP file counts
each fault once. The row pages showed that parent and nothing else. Five kinds
of link never reached a page:

- **The other side of a reading not taken.** §472.4 shows a row's commit row
  beside the input its word names, but only on the row's own page. Q2938's lane
  merge, 1c03f2ea, filed Q2958 and Q2959, whose words name LED files. Q2938's
  page named neither.
- **A commit message's body.** 1c03f2ea's body says "Q2949 answered by Q2938:
  its slot 7 refusal no longer shows, and what follows it is filed as Q2959".
  Q2959's page did not show it.
- **A second word.** Q2005 says `from/fuzz:where_two_surfaces_cross
  was/Q1781`. `lineage` reads only the first word (§472.7), and Q1781's page
  did not name Q2005.
- **Files named in text.** §433.6 reads a source from text only for rows filed
  before Q1703, and only where the text names one input. Q2959 also names
  `LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step`, beside its word's file.
- **Files had no pages.** A chain ended on a file's name, with nothing to
  click.

### 473.2 The rule

The one parent stays. Every count and every tree (§433, §472.3) is taken over
it, as before. The pages show every other link beside it, each saying what kind
of link it is, and none of them moves a count. A link the counting cannot use
is still one a reader can follow.

### 473.3 A row's page

- **Came from** ends on the fuzz target or STEP file where the chain reaches
  one. The input is linked to its page and shows what it counts: "the STEP file
  X: 3 faults it shows, 4 rows in its tree". A step whose row carries more than
  one came-from word shows the later ones, linked, and says that `lineage` reads
  only the first.
- **Also named by**: each row with a reading that names this row and that
  `lineage` did not take. That can be the commit's row under a word that won,
  the parent a row has without its text's source (§433.6), or a later word.
- **Named in the text of**: each row whose line names this row, in main's copy
  or in the copy its adding commit held. An `after:` names rows too.
- **Fuzz targets and STEP files it names**: every input its description names:
  a crash file, a seed, a corpus file, or a fuzz target named in passing.
- **Its commits**: the commit that added the row and the one that closed it.
  Each shows its sha, day and subject, and every paragraph of its message that
  names the row.

### 473.4 An input's page

Each input gets `inputs/fuzz-<target>.html` or `inputs/step-<file>.html`. A
`/` in a corpus file's tail (§433.1) is written `~`.

- **What it is**: the target's `fuzz/fuzz_targets/<target>.rs`, or the corpus
  paths `corpus/manifest.sha256` lists under that name. Where no commit on main
  ever held it, the page says so, since a row's word can name any file.
- **Counted**: as `just lineage` counts it.
- **Came from it**: its tree.
- **Tied to a row in its tree**: rows hung elsewhere that a reading not taken
  ties to a row in its tree. These are the rows Mattias asked for that the tree
  does not hold. Q2958 and Q2959 are on the O6.35 Z3.0 LED file's page, because
  Q2938's merge filed them and Q2938 is in that file's tree.
- **Named by**: every other row whose text or later word names it.

### 473.5 Decided without asking

- **"Tied to" takes any reading not taken**, not only the commit's row beside
  a word. Mattias asked for every row related to the file. A later word, or the
  parent a row has without its text, relates a row as much as a commit does.
  The selftest's commit H pins this: Q53 is tied by its second word alone, and
  a rule that took only the commit's row drops it.
- **A link to an input page asserts that the page exists.** Every input a page
  can name is in `Related.inputs`, so a missing page is a bug in `Related`, not
  a reason to print plain text.
- **Inputs are read from a row's description alone**, as `lineage` reads them,
  so a `done:<sha>` is never read as a fuzz seed's hash. Rows are read from the
  whole line, because an `after:` names rows.
- **A fuzz target named in passing counts as named.** §433.6 refuses it as a
  source, since Q58 names the writer target and is no fault of it. But the row's
  work touches that target, so the target's page lists the row under "Named by".
- **A commit paragraph names a row where `ROW_REF` finds its id.** For a Q id
  that is the test `lineage.token_present` makes, with one pattern compiled once.

### 473.6 Measured

Rendered from main at a8ad8d319, its last commit to `docs/queue.md`, on
2026-10-29. The counts come from a scratch script over the rendered pages and
over `Related`.

- **2895 row pages and 22 input pages**: 5 fuzz targets and 17 STEP files.
  Four of the 22 have no tree. Only rows that name them reach them.
- **88 rows are named by 127 others** in a reading `lineage` did not take.
- **35 rows carry a later word.** §472.7's grep found 36 lines with two words
  side by side at ba1bfd99. This count reads each row's word field.
- 192 rows name an input in their text. 1915 rows are named in another row's
  line. 1774 have a paragraph of a commit message that names them. 365 have a
  chain that ends on an input page.
- Q2959's chain ends on the Z15.0 LED file, with "3 faults it shows, 4 rows in
  its tree", and its commits show 1c03f2ea's Q2949 paragraph. Q2938's "Also
  named by" holds Q2958 and Q2959, which the page before this named nowhere.
  Q1781's holds Q2005. The O6.35 Z3.0 file's "Tied to a row in its tree" holds
  Q2958 and Q2959.
- **What this adds costs 0.7 to 0.9 s a render.** This was measured twice
  against bc3876ca0's row pages on the same caches, and `render.py`'s docstring
  has the parts.

### 473.7 What this leaves

- Whether a row may carry two words is still Mattias's to settle (§472.7). The
  pages show the later words. Nothing counts them.
