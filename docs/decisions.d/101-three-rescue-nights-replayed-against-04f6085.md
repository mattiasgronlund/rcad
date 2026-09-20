## 101. Three rescue nights replayed against 04f6085, and nothing earned a seed

```text
find ~/rcad-fuzz-logs/artifacts-rescued-2026-09-11 \
     ~/rcad-fuzz-logs/artifacts-rescued-2026-09-13 \
     ~/rcad-fuzz-logs/artifacts-rescued-2026-09-14 -type f | wc -l
350
```

Three rescue directories, replayed against `04f6085` (origin/main, gate-green
there at 1877 tests). The honest answer is that none of the 350 is a live
kernel bug and none of them earns a seed — every one either fails to decode at
all, or replays into an outcome shape this rescue itself repeats several times
and `docs/decisions.md` §95 already names as common. `fuzz/corpus/` is
unchanged by this section.

### 101.1 What was where, and the one night that needed attributing first

`2026-09-11` and `2026-09-14` keep each artifact under its target's own
directory — `find ~/rcad-fuzz-logs/artifacts-rescued-2026-09-11 -maxdepth 2
-type d` names `two_solids_fused_by_a_stranger` (253), `where_two_surfaces_cross`
(3) and an empty `no_model_makes_the_writer_panic`; `2026-09-14` names
`where_two_surfaces_cross` (6) and three more empty ones. `2026-09-13` is flat —
88 files, no target directory — and `just fuzz-triage` takes the target off the
artifact's *parent directory name*, so pointed at that night directly it would
report all 88 as "did not come back clean" without ever running the decoder.

That night's own `fuzz-fuse.log` carries the answer. Its "Recommended
dictionary" (libFuzzer prints one per target, learned from that run's own
coverage) includes the byte string `\000\003\001\000\000%\000\000\000Y@\003`,
and every one of the 88 files opens with that same string or a one-byte
mutation of it (`400301...`, `a90301...`, `6f0301...`) — checked with `head -c
12 <file> | xxd -p` against all 88, no exception. `fuzz.log`,
`fuzz-reader.log` and `fuzz-writer.log` (the other three targets that ran that
night) report zero crash artifacts and finish "Done N runs in 7198 second(s)"
each, and the live `fuzz/artifacts/<target>/` directories for the other three
targets are empty — so nothing else that night could have produced these 88.
Decoding three samples, including two of the one-byte mutations, with `mise
exec -- cargo run -q -p rcad-fuzz-explain -- two_solids_fused_by_a_stranger
<file>` confirms it directly: all three decode into an `AHalf { .. }` pair and
replay. The 88 were copied into a per-target layout under this session's
scratchpad (`night13/artifacts/two_solids_fused_by_a_stranger/`) and triaged
from there — the recipe itself was not touched.

A stronger form of the same evidence sat in the tree already and did not need
running anything: 7 of the 88 — `158d358d..`, `5b8430a6..`, `66172e27..`,
`66d6743f..`, `8cc43081..`, `b0794805..`, `c29fad84..` — are byte-identical
(full trailing sha1) to files already checked into
`fuzz/corpus/two_solids_fused_by_a_stranger/`. A file the corpus regression
test replays as that target *is* that target; the dictionary match and the
decode were both corroborating what the corpus itself already said.

### 101.2 `two_solids_fused_by_a_stranger`: 247 of 253 do not even decode

```text
mise exec -- just fuzz-triage ~/rcad-fuzz-logs/2026-09-11 04f6085
...
247 of 256 input(s) did not come back clean on 04f6085
```

All 247 are in the fuse subdirectory (`grep -c "neither half of" triage.log` =
`grep -c "still fails on 04f6085: exit" triage.log` = 247, exact match), and
every one carries the identical message: `neither half of N bytes is a saved
Model`. That is `rcad-fuzz-explain`'s own decode step
(`crates/rcad-fuzz-explain/src/lib.rs:364`, `two_models_for_the_fuse`) failing
before it ever calls the fuse — both halves fail `postcard::from_bytes::<Model>`
and the function returns `Err` without attempting a replay.

**That is not a stale format.** `two_solids_fused_by_a_stranger` bytes carry
`FORMAT_VERSION` (`crates/rcad-topo/src/serialize.rs:133`) as the first thing
written, and a mismatch is refused with a named error
(`Error::WrongFormatVersion`, guarded by its own test,
`a_model_written_by_a_different_version_of_the_format_is_refused`). It last
moved 2 → 3 at `3f384cf`, 2026-09-01 — ten days before this rescue's earliest
night and untouched since (`git log -p --follow -- crates/rcad-topo/src/serialize.rs`,
current value confirmed `3` at `crates/rcad-topo/src/serialize.rs:133`). A
throwaway probe crate under this session's scratchpad, path-depending on this
tree's `rcad-topo` for `Model` and on the workspace's own `postcard`, decoded
both halves of every one of the 253 fuse artifacts and peeked the leading
varint with `postcard::take_from_bytes::<u32>` before attempting the full
decode. Of the 500 halves that fail, 453 fail with `postcard::Error::SerdeDeCustom`
— the wrapper `Model`'s `Deserialize` uses for its *own* checks
(`check_keys_resolve`, `check_nothing_is_used_more_than_once`,
`check_everything_has_an_owner`, `check_shape_is_possible`, all in
`serialize.rs`) — and of those, 342 carry a leading varint that reads exactly
`3`, the current version, not a stale one. The other 47 fail with a raw
`postcard` structural error (`DeserializeBadOption` 38, `DeserializeBadVarint`
7, `DeserializeUnexpectedEnd` 2) — bytes that are not shaped like postcard's
wire format at all, version aside — and 34 of those 47 *also* carry a leading
`3`: the corruption a mutation left behind reaches past the version byte
either way. 376 of the 500, across both groups, read a leading `3`.

So the 247 are fuzzer mutations of real, current-format seeds, most of them
landing on a leading version byte a mutation left untouched while corrupting
the arena underneath it — exactly the failure mode `Model`'s own consistency
checks exist to refuse, and 90% of the time do. `Err` here is `rcad-fuzz-explain`'s
crate doc comment working as documented (*"`Err` is only ever 'these bytes are
not a case of this target at all'"*, `lib.rs:78`) for the right reason: not
because the format moved out from under them, but because most of what a
byte-level mutation produces is not a self-consistent `Model`, then or now.
Nothing decoded for the corpus regression test to gain either
(`every_model_the_fuse_fuzzer_kept_is_fused_again.rs` only counts what
decodes). They earn nothing.

The remaining 6 of 253 decode and replay clean — 0 of them panic. §101.3 covers
what they, and the 88 from `2026-09-13`, actually reached.

### 101.3 The 94 that do decode, and why they are duplicates in kind

`grep -c "outcome: panicked" triage-*.log` is `0` across all three nights'
logs — no artifact in this rescue reproduces a live kernel bug. What the 94
decoding `two_solids_fused_by_a_stranger` artifacts (6 from `2026-09-11`, 88
from `2026-09-13`, all confirmed clean by `mise exec -- just fuzz-triage
<night> 04f6085` reporting `0 of 88` and `6` of the `9` clean entries for
`2026-09-11`) reach breaks into four shapes:

- **82** `outcome: succeeded — nothing was fused: no half held a solid to
  fuse` (3 from `2026-09-11`, 79 from `2026-09-13`, all with byte-identical
  outcome text).
- **5** `outcome: succeeded —` an ordinary completed fuse — 1 `Difference made
  1 solids` (`2026-09-11`) and 4 `Union made 1 solids` (`2026-09-13`).
  `one_sided` is the *common* draw over the checked-in corpus (`one_sided: 569`
  of `682` fuses, §95.6), not the rare one, and both rules are already named
  in its control-seed table (§95's four control seeds).
- **4** `outcome: refused — rcad_topo::Error::CannotMeshAFace { .. why:
  CoordinateOutOfRange { .. why: TooSmall } }` (1 from `2026-09-11`, 3 from
  `2026-09-13`) — the same refusal, recurring inside this one rescue batch
  alone.
- **3** `outcome: refused — rcad_topo::Error::Undecidable { .. predicate:
  "where three surfaces meet" .. }` (1 from `2026-09-11`, 2 from
  `2026-09-13`) — §95's own doc comment already names `Undecidable` raised by
  "where three surfaces meet" as one of "the interesting half" of the
  checked-in corpus's refusals.

Nothing here is a shape the corpus does not already reach, and the two
refusal kinds are shown recurring inside a rescue of only 94 decoding
artifacts — not evidence of rarity.

### 101.4 `where_two_surfaces_cross`: six of nine were already checked in

```text
mise exec -- just fuzz-triage ~/rcad-fuzz-logs/2026-09-14 04f6085
...
2 of 6 input(s) did not come back clean on 04f6085
```

9 artifacts total (3 from `2026-09-11`, 6 from `2026-09-14`). Comparing every
rescued file's trailing sha1 against every `fuzz/corpus/*/` file's trailing
sha1 — `find ~/rcad-fuzz-logs -type f \( -name 'crash-*' -o -name
'slow-unit-*' \) -printf '%f\n' | sed 's/^.*-//' | sort -u` against `find
fuzz/corpus -type f -printf '%f\n' | sed 's/^.*-//' | sort -u`, then `comm
-12` — finds 6 exact matches in this target: `crash-3f9c76a3..`,
`slow-unit-6a164a91..` and `crash-a2acceace0..` from `2026-09-11` (all 3 of
that night's artifacts), plus `crash-cba396f8..`, `crash-d04e0041..` and
`slow-unit-b9068e9e..` from `2026-09-14` (3 of that night's 6, not the 2 an
earlier pass here reported — `d04e0041..` is a bare-hash corpus filename
under the same underscored directory name that hid the 7 fuse matches; see
below for why). The 3 remaining `2026-09-14` files all decode and
replay clean — `outcome: succeeded — SurfaceMeet::Curves, N of them` in every
case, the dominant shape already in an 831-file corpus.

Across all 350 rescued files that same `comm` finds 13 exact matches — the 6
here plus the 7 fuse matches of §101.1 — and 337 whose trailing sha1 matches
no corpus file. **This lane's own brief had it right at "337 sha1s appear in
no file under `fuzz/corpus/`"**; an earlier pass here reported 345 and 5
because its corpus side was built with `find fuzz/corpus -type f | sed
's/^.*-//'` — `sed` operating on the full path rather than the basename. A
corpus filename that is a bare sha1 under an underscored directory name (no
dash anywhere, e.g. `fuzz/corpus/two_solids_fused_by_a_stranger/158d358d..`)
has nothing for `s/^.*-//` to cut, so the whole path passed through unchanged
and never matched a bare 40-character hash from the rescued side; only the
`crash-`/`slow-unit-`-prefixed corpus files (which do carry a dash) were ever
being compared. `-printf '%f\n'` before the `sed`, taking the basename first,
is the fix, and it is what the two `find` commands above do.

### 101.5 A panic that belongs to the explain tool, not the kernel

`slow-unit-b9068e9ed0f147c54aec5b6b35af2f152303b761` — already checked into
`fuzz/corpus/where_two_surfaces_cross/` — makes `rcad-fuzz-explain` itself
panic, exit 101, both under `cargo run` and the release binary:

```text
thread 'main' panicked at crates/rcad-fuzz-explain/src/../../rcad-geom/tests/fuzzing/mod.rs:726:9:
9.332114885078756e-187 is past what one byte of exponent reaches
```

Line 726 is `written_as_numbers`, a helper that packs a float into the
`bytes_for` reconstruction line the explain tool prints so a case can be
pasted into a test — it is not reached by replaying the case itself, and its
own doc comment documents this exact panic: *"If a number cannot be written
down that way, which means a seed was asked for that no decoder could have
produced."* `grep -n "bytes_for\|written_as_numbers" \
crates/rcad-geom/tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`
finds neither name — the gate-covered corpus regression test never calls this
helper, so it does not hit this panic, and `just gate` is unaffected. This is
a known limit of the diagnostic tool's own repro-line writer, on a seed
already in the corpus; nothing to promote and nothing in `crates/` to fix from
this lane.

(One other `2026-09-14` file, `slow-unit-7e62ebe6..`, was killed mid-replay in
this session at ~3 minutes under a debug build — not a hang: the release
binary (`cargo build -q -p rcad-fuzz-explain --release`, then run directly)
replays it in under a second, `outcome: succeeded`, exit 0.)

### 101.6 Why zero, and not "the two that looked most interesting"

Every artifact in this rescue lands in one of three buckets, and none of the
three earns a seed:

- **Does not decode** (247) — fuzzer mutations of real, current-format seeds
  that a mutation broke, refused by `Model`'s own consistency checks or by
  `postcard`'s wire format itself (§101.2's evidence). Nothing for the corpus
  regression test to gain, since it only counts what decodes.
- **Decodes, replays clean, new (not already in the corpus)** (90: 87 of the
  94 `two_solids_fused_by_a_stranger` artifacts of §101.3, plus 3
  `where_two_surfaces_cross` from `2026-09-14`) — every outcome shape recurs
  inside this rescue batch alone, or is named in §95 as already the corpus's
  common case.
- **Already checked in** (13: 7 `two_solids_fused_by_a_stranger`, all from
  `2026-09-13`, §101.1; 6 `where_two_surfaces_cross` — 3 from `2026-09-11`,
  3 from `2026-09-14`, §101.4) — byte-identical to a file under
  `fuzz/corpus/` already.

`fuzz/corpus/` §95.1 measured 1751 fuse seeds and this file's own corpus
regression test reports `ok: 265, refused: 417` over `682` fuses already —
hundreds of completions and refusals a commit can already break. Adding more
of the same four shapes because they came from a rescue rather than because
they reach something new is exactly the cost §95's neighbour section warns
against: "a corpus that grows by one night of one bug's duplicates costs every
later run." So `fuzz/corpus/` is unchanged by this lane.

