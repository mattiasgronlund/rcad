## 472. A row whose word names a STEP file or a fuzz target hangs on the row whose work filed it — 2026-10-28

*Mattias, 2026-10-28, on Q2939's page: "How do I understand where Q2939
actually came from ... It seems to have come from Q2929 ... And it seems to have
come from Q2922!" Told why, and offered two fixes — reword Q2937, Q2938 and
Q2939 to `from/Q2929`, and have a row's page show both parents where its word
and its commit disagree — he answered: "(1) will just fix these rows, can we
make sure that we can resolve the information automatically for all rows? (2)
Yes that woud be nice."*

This amends §433.2's count. It also writes down a rule that Q1707 built for
§433.6, which until now only the code and the `lineage` recipe stated: an input
named in a row's text does not take the row from a parent that is already in
that input's tree.

### 472.1 How the link was lost

`lane.md` tells a lane to start each row it files, where a fuzz target or a
corpus file shows the fault, with `from/fuzz:<target>` or `from/step:<file>`,
"not `from/<your row>`, even when your row's work turned it up (§433.2)".
`merger.md` says the same. `lineage` took a row's word over the commit that
added it. So a row that the work on a fault filed never hung on that work.

Q2922's lane merge (a56a04fa) filed Q2929 with
`from/step:LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step`. Then 2f2ffdc5 closed
Q2929 and filed Q2937, Q2938 and Q2939, all naming that same file. Q2939's page
said it came from the file. Q2929's page said it led to nothing.

The words follow §433.2's first rule, and they are right. What lost the link was
`lineage`'s reading of them: it dropped §433.2's second rule, "A row that fixing
a fault leads to stays `from/<the fault's row>`". Rewording rows by hand fixes
only the rows someone happens to notice.

### 472.2 The rule

A row whose word names an input X hangs on the row R that its commit gives —
the lane's row on a lane merge, else the one row the commit closed, else
§432's text and message rules — **where R is in X's tree**: R hangs on X, or
somewhere below it. Otherwise it stays on X.

The reason is the one §433.6's reading already gives: a row in X's tree was
filed by the work on a fault X showed.

Where R is in another input's tree, the word wins. That is §433.2's Q68 case: a
fix lets a second input get further, and that input shows a new fault. Q2928,
Q2929, Q2930 and Q2931 are such rows. Q2922's merge filed them, but each names a
different LED file from Q2922's, so each stays on its own file.

Whether R is in X's tree depends on R's own parent. So rows are settled oldest
first, and every parent is settled before its children are. They are settled
twice, once without §433.6's text links and once with them, because R can be in
X's tree through a text link alone.

### 472.3 What an input counts

§433.2 asks each input how many faults it showed. A row moved below X is still
one of X's faults. So the source table now counts **the faults X shows**: the
rows hung on X, plus the rows hung below X whose word names X, less re-files. It
used to count only the rows hung on X itself ("faults found directly"), which
would have dropped every moved row from its input's count.

A re-file is the fault of the dropped row it re-files, and is counted there
once.

### 472.4 Both parents on the page

Wherever a row's word and its commit name different parents, `lineage` keeps
the parent it did not take beside the one it did (`ParentInfo.beside`). `just
lineage main <id>` prints both:

- `Q2929: parent step:LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step, from word,
  led-to; the commit gives parent Q2922, from the commit, led-to`
- `Q2939: parent Q2929, from the commit, led-to, in the tree of
  step:LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step, which its word names`

The row's page says the same in its "Came from" line.

### 472.5 Measured

On `main` at ba1bfd99, 2026-10-28 21:00Z. Main's `lineage.py` and this one were
run over the per-commit summaries the queue page keeps
(`~/rcad-queue-page/.lineage-cache.json`), using a scratch script.

- **16 rows change parent with text links, and 14 without them.** Q2922 and
  Q2923 move to Q2911, by the commit message. Q2926 moves to Q2922; Q2937,
  Q2938 and Q2939 to Q2929; Q2940 and Q2941 to Q2931; Q2948 to Q2928; Q2254 to
  Q1780; Q2339 to Q2335; Q2343 to Q2336; Q2349 to Q2337. Q1780 moves to Q1772
  as a re-file. Two rows move only with text links: Q2006 moves to Q1479 as a
  re-file, and Q2131 moves to Q2130. Each was read by hand, and each is a
  follow-up or a re-file of the row it now hangs on.
- **No tree changes size.** Every input's whole tree is the same size as
  before, for all 11 inputs without text links and all 18 with them.
- **Fault counts move for two inputs, by one each, and both were counting a
  fault twice.** two_solids_fused_by_a_stranger goes from 17 to 16 with text
  links, and from 9 to 8 without. Q1780 re-files Q1772 (c91bd88b, "Q1780 files
  Q1772 again by its crash files"). where_a_cone_or_a_torus_crosses goes from 3
  to 2: Q2006 re-files Q1479 (8895e807, "re-file Q1479 as Q2006").
- **36 rows have a word naming an input.** 16 hang on their commit's row. 14
  stay on the input because their commit's row is in another tree. 6 stay
  because their commit gives no row (§472.6).
- **90 of the 1224 rows read from a word** now show their commit's row beside
  the parent.

### 472.6 Where the commit gives no row

A row keeps its word's input when its commit gives no row. That happens when
the commit closed no row and is not a lane merge, or when it closed several and
nothing ties the new row to one of them.

At ba1bfd99 this is 6 rows, and each was read. Q1752 came from a fuzz night's
triage. Q1781, Q1782 and Q2007 are new faults that a night showed, filed beside
re-files. Q2911 came from §469's docs commit, which closed no row. Five of the
six have no row behind them, so the input is their right parent. Q2005 is the
exception: it re-files Q1781, and its own `was/Q1781` says so, but that is its
second word (§472.7).

### 472.7 What this leaves

- **A row with two words loses the second.** Q2005 says
  `from/fuzz:where_two_surfaces_cross was/Q1781`. Q1780 and Q2254 also put a
  `was/` after their `from/fuzz:`, and `extract_word` reads only the first word.
  A grep of `docs/queue.md` at ba1bfd99 finds 36 rows with two came-from words
  side by side somewhere in the line. That count is a grep, not a reading of
  each row. §432.2 gives a row one word. Whether a row can carry a re-file and a
  source at once is Mattias's to settle, and nothing is filed for it yet.
- `lane.md` and `merger.md` are unchanged. A lane still writes the input,
  because that is what §433.2's first rule asks for, and `lineage` now finds
  the row the input led to.
