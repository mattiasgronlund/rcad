#!/usr/bin/env bash
#
# Rebuild the STEP corpus from upstream into a directory of your choosing.
#
#     corpus/fetch.sh /path/to/rcad-step-corpus
#
# `corpus/sources.md` is the provenance record: where every one of the 367
# files comes from, how the 39 pin headers were chosen, what the licences say,
# and the proof that a fresh download is byte-identical to the corpus this
# manifest was made from. This script is that record made executable.
#
# The two NIST zips are pinned by their own sha256, because the dated
# `nist.gov/system/files/...` URL is not itself a pin — NIST re-uploads a
# revised test case under a new date directory. A moved checksum stops the
# script rather than quietly changing what the tests test.
#
# The KiCad half is pinned by commit, and *not* by the checksum of the archive
# GitLab hands back: GitLab builds those tarballs on request and promises
# nothing about their bytes. What is checked instead is every extracted file,
# against `corpus/manifest.sha256`, whose digests cannot drift.
#
# `curl`, `tar`, `unzip` and `sha256sum`. No clone and no `git` binary; the
# repository is 7251 STEP files and we want 323 of them.
set -euo pipefail

nist_pmi_url="https://www.nist.gov/system/files/documents/noindex/2024/06/19/NIST-PMI-STEP-Files.zip"
nist_pmi_sha256="8fa78429e6d8d9b0d7681d223b6aa9ec98c3772185c55b1a0e3679b21c181911"

nist_ftc_url="https://www.nist.gov/system/files/documents/noindex/2024/05/07/NIST-FTC-CTC-PMI-CAD-models.zip"
nist_ftc_sha256="17e14dbf207f447f16e0182a7eda5254ee19afff591dd409b8b510103dd05d10"

kicad_repo="https://gitlab.com/kicad/libraries/kicad-packages3D"
kicad_commit="e62ed1fc7862da83f789bd562671b5e4b82afcdf"

# Taken whole. `Connector_PinHeader_2.54mm.3dshapes` is not here because we keep
# 39 of its 278 files, named one by one further down.
kicad_whole_directories=(
    Fuse.3dshapes
    LED_THT.3dshapes
    Button_Switch_THT.3dshapes
    Relay_THT.3dshapes
    Potentiometer_THT.3dshapes
)

# Every seventh name of the 278 in `Connector_PinHeader_2.54mm.3dshapes`, sorted
# the way `sort` does under `en_US.UTF-8`. That rule is the history of the
# choice, not a selector to lean on — one model added upstream shifts every pick
# after it — so the names are carried here and the rule stays in `sources.md`.
kicad_pin_headers=(
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
)

main() {
    if [ "$#" -ne 1 ]; then
        echo "usage: ${0##*/} <destination directory>" >&2
        exit 2
    fi
    destination=$1

    corpus_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
    manifest="$corpus_dir/manifest.sha256"
    [ -f "$manifest" ] || fail "no manifest beside this script at $manifest"

    refuse_to_clobber "$destination"

    parent=$(dirname -- "$destination")
    mkdir -p -- "$parent"
    work=$(mktemp -d -- "$parent/.rcad-corpus-fetch.XXXXXX")
    trap 'rm -rf -- "$work"' EXIT
    build="$work/corpus"

    fetch_nist "$work" "$build"
    fetch_kicad "$work" "$build"
    keep_only_step_files "$build"
    check_against_manifest "$build" "$manifest"

    mv -- "$build" "$destination"
    echo "corpus built at $destination"
}

# The destination is somebody's only copy of the corpus if they point this at
# one by mistake, and the corpus is not in git. So a directory with anything in
# it stops the script; only a missing or empty one is written to.
refuse_to_clobber() {
    local destination=$1
    if [ -e "$destination" ]; then
        [ -d "$destination" ] || fail "$destination exists and is not a directory"
        if [ -n "$(ls -A -- "$destination")" ]; then
            fail "$destination is not empty; remove it or name a different directory"
        fi
        rmdir -- "$destination"
    fi
}

# 44 STEP files, in two zips, of which one is 60 MB and yields eleven of them.
# The rest of that zip is CATIA, Creo, NX, Inventor and SolidWorks copies of the
# same parts, and NIST publishes one zip per test-case family rather than one
# per format, so there is no smaller download that reaches those eleven.
fetch_nist() {
    local work=$1 build=$2

    download "$nist_pmi_url" "$work/NIST-PMI-STEP-Files.zip"
    check_one_file "$work/NIST-PMI-STEP-Files.zip" "$nist_pmi_sha256"
    mkdir -p -- "$build/nist/pmi-step-files"
    unzip -q -- "$work/NIST-PMI-STEP-Files.zip" -d "$build/nist/pmi-step-files"

    download "$nist_ftc_url" "$work/NIST-FTC-CTC-PMI-CAD-models.zip"
    check_one_file "$work/NIST-FTC-CTC-PMI-CAD-models.zip" "$nist_ftc_sha256"
    mkdir -p -- "$build/nist/ftc-ctc"
    unzip -q -- "$work/NIST-FTC-CTC-PMI-CAD-models.zip" -d "$build/nist/ftc-ctc"
}

# 323 STEP files, flattened into one `kicad/` directory the way the corpus this
# manifest was made from has them. All 323 names are unique across the six
# upstream directories, so nothing collides and nothing is lost.
fetch_kicad() {
    local work=$1 build=$2
    mkdir -p -- "$build/kicad"

    local directory
    for directory in "${kicad_whole_directories[@]}"; do
        download "$kicad_repo/-/archive/$kicad_commit/x.tar.gz?path=$directory" \
            "$work/$directory.tar.gz"
        # `kicad-packages3D-<commit>-<directory>/<directory>/<file>`, so two
        # components come off and the files land flat.
        tar -xzf "$work/$directory.tar.gz" -C "$build/kicad" --strip-components=2
    done

    local urls=()
    local name
    for name in "${kicad_pin_headers[@]}"; do
        urls+=("$kicad_repo/-/raw/$kicad_commit/Connector_PinHeader_2.54mm.3dshapes/$name")
    done
    echo "fetching ${#urls[@]} pin headers"
    curl --fail --location --silent --show-error --retry 3 --retry-delay 2 \
        --output-dir "$build/kicad" --remote-name-all "${urls[@]}"
}

# The NIST zips carry 153 companion CATPart, SLDPRT, `.prt`, `.ipt`, pdf, jpg,
# zip, xlsx and pptx files, and the KiCad archives carry a CREDITS.md apiece.
# No test opens any of them: the walk in `every_file_in_the_corpus_reads.rs`
# keeps `.stp` and `.step` and nothing else. Dropping them is what makes the
# built corpus exactly the 367 files the manifest names.
keep_only_step_files() {
    local build=$1
    find "$build" -type f ! -iname '*.stp' ! -iname '*.step' -delete
    find "$build" -mindepth 1 -type d -empty -delete
}

# `just check-corpus` is the real oracle and says which files are missing,
# altered or unnamed. This is the gate that stops a bad fetch from ever being
# moved into place, so it only has to answer yes or no.
check_against_manifest() {
    local build=$1 manifest=$2
    echo "checking $(wc -l <"$manifest") files against ${manifest##*/}"
    (cd -- "$build" && sha256sum --check --quiet -- "$manifest")

    local built
    built=$(find "$build" -type f | wc -l)
    local named
    named=$(wc -l <"$manifest")
    [ "$built" -eq "$named" ] ||
        fail "built $built files but $manifest names $named"
}

download() {
    local url=$1 target=$2
    echo "fetching ${target##*/}"
    curl --fail --location --silent --show-error --retry 3 --retry-delay 2 \
        --output "$target" -- "$url"
}

check_one_file() {
    local path=$1 want=$2
    local got
    got=$(sha256sum -- "$path" | cut -d' ' -f1)
    if [ "$got" != "$want" ]; then
        fail "$(
            echo "$path does not have the checksum this script pins:"
            echo "  want $want"
            echo "  got  $got"
            echo "Upstream moved. Do not edit the checksum to match — work out what"
            echo "changed, and update corpus/manifest.sha256 and corpus/sources.md"
            echo "in the same commit if the new bytes are what we want."
        )"
    fi
}

fail() {
    echo "$*" >&2
    exit 1
}

main "$@"
