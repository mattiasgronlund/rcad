# Where the STEP corpus comes from

The tests read a corpus of STEP files from the path in `RCAD_STEP_CORPUS`. The
corpus is not in git. This file says where every file in it came from, proves it
by re-downloading and comparing bytes, and gives the exact handles a CI fetch
script would name.

Established 2026-09-07 against the local corpus at
`/home/mattias/git/mattiasgronlund/rcad-step-corpus`. Every number below carries
the command that produced it.

## What the tests actually read

`crates/rcad-step/tests/every_file_in_the_corpus_reads.rs:175-180` walks the
corpus directory and keeps a file only when its extension is `stp` or `step`,
compared without regard to case. Nothing else is opened.

So the corpus directory holds more than the tests need:

```console
$ find $RCAD_STEP_CORPUS -type f | wc -l
520
$ find $RCAD_STEP_CORPUS -type f \( -iname '*.stp' -o -iname '*.step' \) | wc -l
367
```

The 153 files that are not STEP are CATPart, SLDPRT, Creo `.prt`, Inventor
`.ipt`, pdf, jpg, zip, xlsx and pptx. They arrive inside the NIST downloads and
no test opens them.

The 367 files that CI must reproduce:

```console
$ find $RCAD_STEP_CORPUS/nist -type f \( -iname '*.stp' -o -iname '*.step' \) | wc -l
44
$ find $RCAD_STEP_CORPUS/kicad -type f \( -iname '*.stp' -o -iname '*.step' \) | wc -l
323
$ find $RCAD_STEP_CORPUS -type f \( -iname '*.stp' -o -iname '*.step' \) -printf '%s\n' \
    | awk '{s+=$1} END {print s}'
139867388
```

139 867 388 bytes, which is 133.4 MiB. The NIST half is 58 708 181 bytes
(56.0 MiB) and the KiCad half is 81 159 207 bytes (77.4 MiB), by the same
command run on each subdirectory.

A published figure of 375 MB appears in `.github/workflows/ci.yml`. That is the
whole corpus directory on disk, companion formats included. It is not what CI
needs.

## The NIST half: 44 files

### Where it came from

The header of every one of these files says so:

```console
$ head -4 "$RCAD_STEP_CORPUS/nist/pmi-step-files/NIST-PMI-STEP-Files/nist_ctc_01_asme1_ap242-e1.stp"
ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('CTC-01 geometry with PMI representation and/or presentation','from the NIST MBE PMI Validation and Conformance Testing Project'),'2;1');
```

NIST publishes those files from the page "Download Free CAD Models, STEP Files,
and Test Results", at
<https://www.nist.gov/ctl/smart-connected-systems-division/smart-connected-manufacturing-systems-group/mbe-pmi-0>.

The corpus holds four subdirectories under `nist/`, and each one is an unpacked
NIST zip. Only two of the four contain any STEP file:

| corpus subdirectory | NIST zip | `.stp` files inside |
| --- | --- | --- |
| `nist/pmi-step-files/` | `NIST-PMI-STEP-Files.zip` | 33 |
| `nist/ftc-ctc/` | `NIST-FTC-CTC-PMI-CAD-models.zip` | 11 |
| `nist/htc/` | `NIST-HTC-PMI-v2.zip` | 0 |
| `nist/mtc-assembly/` | `NIST-MTC-Assembly.zip` | 0 |

CI needs the first two zips and neither of the other two.

### Proof

Both zips were downloaded fresh and every `.stp` inside compared with the local
copy. Not three files — all 44.

```console
$ curl -sSL -o NIST-PMI-STEP-Files.zip \
    "https://www.nist.gov/system/files/documents/noindex/2024/06/19/NIST-PMI-STEP-Files.zip"
$ curl -sSL -o NIST-FTC-CTC-PMI-CAD-models.zip \
    "https://www.nist.gov/system/files/documents/noindex/2024/05/07/NIST-FTC-CTC-PMI-CAD-models.zip"
$ unzip -q NIST-PMI-STEP-Files.zip -d x && unzip -q NIST-FTC-CTC-PMI-CAD-models.zip -d x
$ find x -type f -iname '*.stp' | wc -l
44
$ diff <(fresh sha256 list) <(local sha256 list)   # both sorted by path
$ echo $?
0
```

All 44 are byte-identical, and they sit at the same relative paths inside the
zip as they do under `nist/pmi-step-files/` and `nist/ftc-ctc/`. Nothing was
renamed, converted or re-exported when the corpus was built.

### The pin

The stable name NIST advertises is a slug that redirects:

```console
$ curl -sSI https://www.nist.gov/document/nist-pmi-step-files | grep -i ^location
location: https://www.nist.gov/system/files/documents/noindex/2024/06/19/NIST-PMI-STEP-Files.zip
```

The slug is **not** a pin — it points at whatever NIST last uploaded. The dated
URL behind it is closer to one, but NIST controls it and re-uploads under a new
date directory when a test case is revised (the HTC zip already sits under
`2025/09/03`). So the real pin is the checksum, and the URL is a hint about
where to look:

```console
$ sha256sum NIST-PMI-STEP-Files.zip NIST-FTC-CTC-PMI-CAD-models.zip
8fa78429e6d8d9b0d7681d223b6aa9ec98c3772185c55b1a0e3679b21c181911  NIST-PMI-STEP-Files.zip
17e14dbf207f447f16e0182a7eda5254ee19afff591dd409b8b510103dd05d10  NIST-FTC-CTC-PMI-CAD-models.zip
```

A CI script names those two URLs and those two checksums, and fails loudly if a
checksum moves. It must fail rather than carry on: a silent re-download of a
revised NIST test case would change what the tests are testing.

### Cost

```console
$ stat -c %s NIST-PMI-STEP-Files.zip NIST-FTC-CTC-PMI-CAD-models.zip
13976599
60294624
```

74 271 223 bytes, 70.8 MiB, in two requests. Of that, only 58 708 181 bytes of
STEP survives extraction — the FTC/CTC zip is 60 MB and yields 11 STEP files
totalling 4 433 063 bytes, because the rest is CATIA, Creo, NX, Inventor and
SolidWorks copies of the same parts. There is no smaller download that gets
those 11 files; NIST publishes one zip per test-case family, not per format.

### Licence

The NIST page says, in its own words:

> The test cases, CAD models, and STEP files can be used without any
> restrictions. Their use in other software or hardware products does not imply
> a recommendation or endorsement of those products by NIST.

Fetching them in CI raises nothing. Republishing them raises nothing either.
NIST asks for acknowledgement and forbids use of the NIST logo in promotional
material; neither bites here.

## The KiCad half: 323 files

### Where it came from

Again the file headers say it:

```console
$ head -4 "$RCAD_STEP_CORPUS/kicad/Fuse_0402_1005Metric.step"
ISO-10303-21;
HEADER;
/* Fuse_0402_1005Metric.step 3D STEP model for use in ECAD systems
 * Copyright (C) 2020, kicad StepUp
```

These are the official KiCad 3D model library, `kicad-packages3D`, whose home is
GitLab: <https://gitlab.com/kicad/libraries/kicad-packages3D>. The GitHub mirror
`KiCad/kicad-packages3D` is dead — its `master` is a commit from October 2020 —
so do not fetch from it.

The corpus flattens all 323 into one directory. Upstream they live in six
`.3dshapes` directories. Every one of the 323 names was matched against the
repository's file list at a pinned commit; all 323 matched exactly one upstream
path, so the flattening lost nothing and can be undone:

| upstream directory | files upstream | files we keep |
| --- | --- | --- |
| `Fuse.3dshapes` | 28 | 28 — all |
| `LED_THT.3dshapes` | 87 | 87 — all |
| `Button_Switch_THT.3dshapes` | 81 | 81 — all |
| `Relay_THT.3dshapes` | 64 | 64 — all |
| `Potentiometer_THT.3dshapes` | 24 | 24 — all |
| `Connector_PinHeader_2.54mm.3dshapes` | 278 | 39 — a sample |

The repository holds 7251 `.step` files across 105 `.3dshapes` directories at
that commit, so our 323 are about 4% of it.

### How the 39 pin headers were chosen

This was the part most likely to be unrecoverable, and it is not. Sort the 278
names in `Connector_PinHeader_2.54mm.3dshapes` the way `sort` does under
`en_US.UTF-8`, then take every seventh — lines 7, 14, 21 and so on to 273. That
gives exactly our 39, with no file over and none missing.

```console
$ LC_ALL=en_US.UTF-8 sort upstream-278-names | awk 'NR%7==0' | LC_ALL=C sort > candidate
$ LC_ALL=C sort our-39-names > ours
$ cmp candidate ours && echo same
same
```

The locale matters. Under `LC_ALL=C` — which is git's own byte order for tree
entries, and what Python's `sorted()` gives — the same rule picks a different
39. Whoever built the corpus listed the directory in a normal desktop locale.

That rule is how the choice was made, not a selector to lean on. It reads the
whole directory, so one model added upstream shifts every pick after it. A fetch
script should carry the 39 names, listed below, and use the rule only as the
explanation of where they came from.

```
PinHeader_1x03_P2.54mm_Horizontal.step
PinHeader_1x04_P2.54mm_Vertical.step
PinHeader_1x06_P2.54mm_Vertical_SMD_Pin1Right.step
PinHeader_1x08_P2.54mm_Vertical_SMD_Pin1Left.step
PinHeader_1x10_P2.54mm_Horizontal.step
PinHeader_1x11_P2.54mm_Vertical.step
PinHeader_1x13_P2.54mm_Vertical_SMD_Pin1Right.step
PinHeader_1x15_P2.54mm_Vertical_SMD_Pin1Left.step
PinHeader_1x17_P2.54mm_Horizontal.step
PinHeader_1x18_P2.54mm_Vertical.step
PinHeader_1x20_P2.54mm_Vertical_SMD_Pin1Right.step
PinHeader_1x22_P2.54mm_Vertical_SMD_Pin1Left.step
PinHeader_1x24_P2.54mm_Horizontal.step
PinHeader_1x25_P2.54mm_Vertical.step
PinHeader_1x27_P2.54mm_Vertical_SMD_Pin1Right.step
PinHeader_1x29_P2.54mm_Vertical_SMD_Pin1Left.step
PinHeader_1x31_P2.54mm_Horizontal.step
PinHeader_1x32_P2.54mm_Vertical.step
PinHeader_1x34_P2.54mm_Vertical_SMD_Pin1Right.step
PinHeader_1x36_P2.54mm_Vertical_SMD_Pin1Left.step
PinHeader_1x38_P2.54mm_Horizontal.step
PinHeader_1x39_P2.54mm_Vertical.step
PinHeader_2x01_P2.54mm_Vertical.step
PinHeader_2x04_P2.54mm_Horizontal.step
PinHeader_2x06_P2.54mm_Vertical_SMD.step
PinHeader_2x08_P2.54mm_Vertical.step
PinHeader_2x11_P2.54mm_Horizontal.step
PinHeader_2x13_P2.54mm_Vertical_SMD.step
PinHeader_2x15_P2.54mm_Vertical.step
PinHeader_2x18_P2.54mm_Horizontal.step
PinHeader_2x20_P2.54mm_Vertical_SMD.step
PinHeader_2x22_P2.54mm_Vertical.step
PinHeader_2x25_P2.54mm_Horizontal.step
PinHeader_2x27_P2.54mm_Vertical_SMD.step
PinHeader_2x29_P2.54mm_Vertical.step
PinHeader_2x32_P2.54mm_Horizontal.step
PinHeader_2x34_P2.54mm_Vertical_SMD.step
PinHeader_2x36_P2.54mm_Vertical.step
PinHeader_2x39_P2.54mm_Horizontal.step
```

One name in the corpus looks out of place next to the `SW_*` and `PinHeader_*`
files: `Nidec_Copal_SH-7010C.step`. It is one of the 81 files in
`Button_Switch_THT.3dshapes`, so it came in with that whole directory and is not
a hand-added extra.

### Proof

All 323, not three. The six directories were downloaded as archives at the
pinned commit and every file compared:

```console
$ SHA=e62ed1fc7862da83f789bd562671b5e4b82afcdf
$ curl -sSL -o Fuse.tar.gz \
    "https://gitlab.com/kicad/libraries/kicad-packages3D/-/archive/$SHA/x.tar.gz?path=Fuse.3dshapes"
  ... and the same for the other five ...
$ compare each of the 323 by sha256 against $RCAD_STEP_CORPUS/kicad/
identical: 323   different: 0   not-found-upstream: 0
```

A single file, shown in full so the shape of the check is visible:

```console
$ curl -sSL -o Fuse_0402_1005Metric.step \
    "https://gitlab.com/kicad/libraries/kicad-packages3D/-/raw/master/Fuse.3dshapes/Fuse_0402_1005Metric.step"
$ sha256sum Fuse_0402_1005Metric.step "$RCAD_STEP_CORPUS/kicad/Fuse_0402_1005Metric.step"
c18cb5f8977e59e7f62917f145c9ee7efeeaf96ed4a69b0ffb965911153f2608  Fuse_0402_1005Metric.step
c18cb5f8977e59e7f62917f145c9ee7efeeaf96ed4a69b0ffb965911153f2608  .../kicad/Fuse_0402_1005Metric.step
```

Line endings, headers and export timestamps are all identical. Nothing was
touched on the way in.

### The pin

Commit `e62ed1fc7862da83f789bd562671b5e4b82afcdf` on
`gitlab.com/kicad/libraries/kicad-packages3D`. That is what a CI script names.

The release tag `10.0.6` also carries the same bytes for the three files spot
checked there, so a tag would work as a pin too. Prefer the commit: a tag is a
name a maintainer can move, and a commit sha is not.

```console
$ curl -sSL "https://gitlab.com/kicad/libraries/kicad-packages3D/-/raw/10.0.6/Fuse.3dshapes/Fuse_0402_1005Metric.step" | sha256sum
c18cb5f8977e59e7f62917f145c9ee7efeeaf96ed4a69b0ffb965911153f2608  -
```

Do not pin the archive tarball's own checksum. GitLab builds those on request.
Two downloads of the same archive seconds apart did match
(`87b1ff48e736f1fcf6a9dc0ee8f8943cf83fbdbee0ec1a10c4face70e3b83d04` both times),
but that is one trial and probably a cache hit, and it is not a promise GitLab
makes. Check the extracted files, whose checksums cannot drift.

### Cost

The cheapest measured route is five whole-directory archives plus the 39 pin
headers one at a time:

```console
five whole-directory archives          7 387 162 bytes   7.0 MiB
39 pin header files, one request each 42 630 749 bytes  40.7 MiB
                                      ----------------
                                      50 017 911 bytes  47.7 MiB
```

The 39 requests took 27 seconds on a home connection.

Taking the sixth directory as an archive too is one request instead of 39 but
costs more, because it drags in all 278 pin headers:

```console
all six archives                      62 766 789 bytes  59.9 MiB
```

Neither route needs a clone. There is no 5 GB checkout here and no `git` binary
required; `curl` and `tar` are enough. A shallow or sparse clone was not
measured, because it is not the cheapest option and nothing depends on it.

Both halves together: **124 289 134 bytes, 118.5 MiB, in 46 requests** — five
archives, 39 single files, two zips — to
reconstruct 367 files and 133.4 MiB on disk. Compressed archives and zips are
why the download is smaller than the result.

### Licence

`LICENSE.md` in the repository: Creative Commons CC-BY-SA 4.0, with an exception
that waives article 3 for electronic designs made using the library. In KiCad's
own words:

> if you wish to redistribute the KiCad libraries, or parts thereof (including
> in modified form) as a collection then the exception above does not apply.
> Redistributed library collections must be shared under the same license
> agreement. Under these circumstances, the libraries must also retain
> attribution information, including the license documents which are distributed
> with the library files.

Downloading in CI is not redistribution. The files go from GitLab to a runner
and into that repository's own Actions cache; nobody else is handed a copy. The
CC-BY-SA obligations attach to handing the collection on, and none of that
happens.

The exception is worth reading carefully anyway, because it does **not** cover
us. It waives article 3 for *designs made with* the library. We are not making a
design; we are reading the files as test input. That is plain use of the
Licensed Material, which CC-BY-SA allows without condition until you
redistribute.

Note also that CC-BY-SA is not a software licence and the `cargo-deny` rule
against LGPL runtime dependencies does not reach it. That rule is about code
linked into the kernel. These are test inputs, downloaded at test time, absent
from every shipped artefact.

## What the fetch script needs

Route (a) — fetch from upstream, pinned, behind the Actions cache — works. Both
halves are re-fetchable and both were reproduced byte for byte.

The handles, in one place:

- `https://www.nist.gov/system/files/documents/noindex/2024/06/19/NIST-PMI-STEP-Files.zip`,
  sha256 `8fa78429e6d8d9b0d7681d223b6aa9ec98c3772185c55b1a0e3679b21c181911`
- `https://www.nist.gov/system/files/documents/noindex/2024/05/07/NIST-FTC-CTC-PMI-CAD-models.zip`,
  sha256 `17e14dbf207f447f16e0182a7eda5254ee19afff591dd409b8b510103dd05d10`
- `gitlab.com/kicad/libraries/kicad-packages3D` at commit
  `e62ed1fc7862da83f789bd562671b5e4b82afcdf`
- five whole directories: `Fuse.3dshapes`, `LED_THT.3dshapes`,
  `Button_Switch_THT.3dshapes`, `Relay_THT.3dshapes`,
  `Potentiometer_THT.3dshapes`
- the 39 named files from `Connector_PinHeader_2.54mm.3dshapes`, listed above

The script should carry a checksum for each of the 367 files and check every one
after extraction, not just the two zip checksums. The per-file lists were
generated here; each is a sorted `sha256  path` file, and to let a later lane
tell whether it has rebuilt the same list, here are their digests:

```console
$ sha256sum kicad-manifest.txt
0d71bcf10f846a361a545c2ecb97bf3099fbd05505a0446a5209199aaf0b2bd3
$ sha256sum nist-manifest.txt
e1cec62d5b981006b76742c4aaaefb35696bae5e83cbb99ddbdb6112c844560d
```

The KiCad list holds lines like

```
865172cb0d7899e6403a722e24f8a1b57d60c7d96387c2dfcd7d97cfe75ad4e9  Button_Switch_THT.3dshapes/SW_CK_JS202011AQN_DPDT_Angled.step
```

sorted by path under `LC_ALL=C`, one line per file, 323 lines. The NIST list is
the same shape with paths relative to the zip root, 44 lines, for example

```
bd5a73c58880ba9093f8dc653c87f4224d6a795459e7fb64f6fdc22ba76a02d3  NIST-FTC-CTC-PMI-CAD-models/CTC Definitions/nist_ctc_01_asme1_rd.stp
```

Two shapes for the fetched tree are possible, and the tests do not care which:
the walk in `every_file_in_the_corpus_reads.rs` recurses, and all 323 KiCad
names are unique across the six upstream directories, so flat and nested both
work. Flat reproduces today's corpus exactly.

## What the fallback would have cost

Route (b), republishing the 367 files once as a release asset or a sibling
repository, was the named fallback. It is not needed, and it is worse. It would
be redistribution of a modified KiCad library collection — a hand-picked 4%
subset with the directory structure removed — so CC-BY-SA 4.0 would apply in
full: the asset would have to be shared under CC-BY-SA, carry attribution, and
ship `LICENSE.md` alongside. The NIST half would raise nothing. Route (a) avoids
that question entirely by never handing the files on.
