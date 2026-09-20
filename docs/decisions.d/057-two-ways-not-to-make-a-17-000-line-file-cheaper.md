## 57. Two ways not to make a 17 000-line file cheaper, recorded 2026-09-06

Two candidates were put up for the cost of reading `docs/decisions.md`: point
rust-analyzer at the tree, or split the file. Both were measured and both are
declined, and the first was declined for a reason worth writing down — it was
aimed at the wrong file.

### 57.1 Where the reading actually goes

Over the 64 transcripts in this project's history, `Read` delivered 2.59 M
characters. By file type:

    .md                64.8%   1.68 M
    .rs                32.8%   0.85 M
    everything else     2.4%   0.06 M

`decisions.md` alone is **0.76 M — 29% of every character `Read` has ever
delivered here**, more than all Rust source in every session combined.
`roadmap.md` is another 0.32 M. Bash delivered 3.90 M over the same span, half
again more than `Read`, so file reading is not even the largest channel.

### 57.2 The 17 000-line file is Markdown

rust-analyzer helps with `decisions.md` by exactly zero, because it does not
read Markdown. The two questions never met: the file that prompted the question
is not a file the tool can open. That is the whole of the answer to the first
candidate, and everything in §57.3 is about the other third.

### 57.3 What rust-analyzer costs and buys on the Rust third

It was driven over stdio against this workspace rather than guessed at:

    cachePriming done                        16.3s
    workspace/symbol usable                  16.4s after a cold launch
    hover on a fn signature                  325 chars
    rust-analyzer RSS                        2.02 GB

Sixteen seconds is not the objection. The objection is the shape of the answers.
`documentSymbol` on `arrange.rs`, the largest file in the tree, needs no index at
all — it is a parse — and returns **78.3 kchars of JSON** for 275 symbols. The
same outline by grep is **2 972 chars** for the 53 top-level items, so for the
query most often wanted the protocol costs 26× the tokens, and a formatter to
fix that is work done to arrive back where grep already was.

What is left is the part text search genuinely cannot do: the type of an
expression, and a definition chased through re-exports. That is real, and it is
narrow, and it would need a JSON-RPC driver plus either a 16-second cold start
per query or a resident 2 GB process beside three other live sessions. It is
deferred under §13 until a named consumer appears; the facade re-export work is
the first plausible one.

### 57.4 The split, and why the recipes already beat it

`decisions.md` is 1.105 Mchars, 54 sections, 452 subsections, median section
17.2 kchars. The case for splitting is that a section is large. It is not:

    just decisions-toc   4 354 chars   all 54 sections
    just decision 41     4 853 chars   §41's prose and its subsection list
    just decision 41.6   3 519 chars
    just decision 8        252 chars

§41 is 79.5 kchars and costs 4.9 to open, because `just decision N` hands back
the section's own prose and then the *list* of its subsections rather than their
text. A file per section cannot beat that, because a file per section still
delivers the section. The effect is already in the transcripts:

    2026-08-30   16 calls   242.0 kchars    15.1 kchars/call
    2026-09-05   16 calls   121.2 kchars     7.6 kchars/call
    2026-09-06    8 calls    16.4 kchars     2.1 kchars/call

The honest argument *for* a split is conflicts — 98 commits touch this file, ten
to thirty a day, at up to +6 900 lines in one day, from as many as four
concurrent sessions all appending at the tail. That pressure is real enough that
§55 opens by explaining a reserved number. But a split only half-answers it: it
removes the textual conflict at the tail and leaves the number collision exactly
where it was. This section is itself the proof: it was written as §56 and had to
be renumbered, because another lane had taken 56 while it was being written.

### 57.5 What was built instead

One gap survived the measurement. Grep answers in line numbers, and a line
number is the one address this file does not take — CLAUDE.md says never to
guess a line range — so every hit cost a walk back up to the heading above it,
and nothing automated that walk. `just decisions-find` closes it by answering in
section numbers, which is what `just decision` wants next.

Its two refusals are deliberate. **The match is case-insensitive**, because of
886 grep commands measured across one day, 97 matched nothing at all, and a
search that misses `Refusal` for a typed `refusal` fails in exactly that silent
way. **No match, and the empty pattern, fail rather than printing nothing**, for
the reason `decision` fails on a number that does not exist: an empty answer
reads as "the decisions say nothing about this", and that is a claim a typo can
forge.

### 57.6 A claim this section does not make

Nothing here says the Rust third is cheap. It is 0.85 Mchars, 33% of everything
`Read` has delivered, and 0.31 M of that arrived through calls that passed no
`limit` at all. What is settled is only that rust-analyzer is the wrong
instrument for it — 26× the tokens on the common query. A `just` recipe that
addresses a source file by symbol rather than by line range is the same fix
`decision` was, and is not refused here. It is §57.7.

### 57.7 `just src`, and what validated it

`just src Decision` prints that enum with its doc comment and attributes, from
wherever in the workspace it is defined, and `just src new crates/rcad-geom`
narrows a name that is defined 43 times. **One definition prints the item; more
than one prints the places**, because printing every `fn new` in the workspace
is the cost the recipe exists to avoid.

The extent is read off rustfmt's layout rather than parsed: back to the blank
line above the definition, forward to the first `}` at the definition's own
indentation. That is a real assumption and it is why `just fmt` matters here,
so **it fails and says so when that brace is not there**, rather than printing
to the end of the file. An item that runs on for 7 000 lines is the failure the
recipe is against; producing one silently would be worse than not having it.

Across the 32 top-level functions of `arrange.rs` — 330 509 characters, the
largest file in the tree — every extraction ends on a brace at column 0, and the
mean extraction is 1 961 characters against the 6 541 of the largest. Across 149
items in four more large files, none was cut wrong; the single item the check
flagged was the check's own bug, since a tuple struct has no braces to balance.

The selftest earned its place immediately. With one file to search, `grep -n`
omits the filename, so `just src shared src/b.rs` — the narrowing case, the
whole point of the second argument — parsed the line number as the path and died
on a file called `1`. It is `-nH` now. A fixture repo is what caught it, because
the unfiltered searches that ran first all had more than one file and all
passed.

### 57.8 Two sections with one number, now caught

§57.4 said a split would not have fixed the number collision. It did not say
anything was catching it, and nothing was: `check-decisions` asked only whether
a heading had the right *shape*, so two `## 56.` headings passed it cleanly, and
`decision` stopped at the first of them. The second section was addressed by
nothing at all, silently. That is the same failure as a search that matches
nothing — an empty answer that reads like a clean one — and it is the third time
this file has recorded that shape.

It is not a typo's failure, which is why it is worth a check rather than care.
Sections are appended at the tail by whoever finishes first, ten to thirty times
a day, from as many as four concurrent sessions; two of them reaching for the
same next number is the ordinary case. It cost one manual renumber to find, at
merge time, by reading. `check-decisions` now names both offending lines and
fails.

The fixtures are duplicates rather than the real file, because the real file has
no duplicate today and pinning it there would prove nothing. Before the check,
the heading-shape grep found nothing wrong with a two-line file containing `## 1.`
twice; that is what the check-in demonstration ran.

