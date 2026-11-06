#!/usr/bin/env python3
"""The Gappa proofs of docs/decisions.d/464 §464.10, and the check that the Rust
they restate has not moved from under them.

    python3 proofs/bounds.py check          what `just check-proofs` runs
    python3 proofs/bounds.py prove [NAME]    what `just prove-bounds` runs

Each proof is `proofs/gappa/<name>.g`. Its controls are
`proofs/gappa/c_<name>_<what>.g`, each a script that must fail. Each proof
starts with one line per item of Rust it restates,

    # restates: crates/rcad-predicates/src/orient3d.rs on_floats at d9f931e8

or, for a dependency, `# restates: Cargo.lock nalgebra at c3b2160e`.

`check` reads the working tree only, with no nix and no git history, since
CI's checkout is one commit. `prove` needs both: nix for Gappa and Coq under
§464.4's pin, and the history to see that each item's code is still the code
at the commit its line names.
"""

import hashlib
import os
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
GAPPA_DIR = ROOT / "proofs" / "gappa"
NIX_FILE = ROOT / "proofs" / "gappalib18.nix"
NIXPKGS = "github:NixOS/nixpkgs/02c087f3fe673c9e317f206a627cb6b910336dbe"
NIX = ["nix", "--extra-experimental-features", "nix-command flakes"]
PRECISION = "-Eprecision=200"

RESTATES = re.compile(r"^# restates: (\S+) (\S+) at ([0-9a-f]{7,40})$")


def main(argv):
    if len(argv) >= 2 and argv[1] == "check":
        return check(argv[2:])
    if len(argv) >= 2 and argv[1] == "prove":
        return prove(argv[2:])
    if len(argv) == 4 and argv[1] == "extent":
        return extent(argv[2], argv[3])
    print(__doc__, file=sys.stderr)
    return 2


def extent(path, item):
    """What `check-proofs-selftest` holds to `just src`'s print of the item."""
    text = working_file(path)
    lines = None if text is None else rust_item(text, item)
    if lines is None:
        print(f"{path} {item} is not found once", file=sys.stderr)
        return 1
    print("\n".join(lines))
    return 0


def check(args):
    if args:
        print(f"check takes no arguments, not {' '.join(args)}", file=sys.stderr)
        return 2
    scripts, controls, failures = read_tree()
    for name in sorted(scripts):
        for why in stamp_differences(name, scripts[name], controls.get(name, [])):
            failures.append(f"{name}.g: {why}; run just prove-bounds {name}.g")
    for stamp in sorted(GAPPA_DIR.glob("*.stamp")):
        if stamp.stem not in scripts:
            failures.append(f"{stamp.name} has no {stamp.stem}.g beside it")
    if failures:
        print("check-proofs: the proofs in proofs/gappa/ no longer match their stamps:")
        for failure in failures:
            print(f"  {failure}")
        print("a stamp records what prove-bounds proved; docs/decisions.d/464 §464.10 3 d")
        return 1
    count = sum(len(v) for v in controls.values())
    print(f"check-proofs: {len(scripts)} proofs and {count} controls match their stamps")
    return 0


def read_tree():
    """The proofs by name, their controls by the proof they belong to, and what
    is wrong with the directory itself."""
    failures = []
    scripts = {p.stem: p for p in GAPPA_DIR.glob("*.g") if not p.name.startswith("c_")}
    if not scripts:
        failures.append(f"no proofs found in {GAPPA_DIR.relative_to(ROOT)}")
    controls = {}
    for control in sorted(GAPPA_DIR.glob("c_*.g")):
        owners = [s for s in scripts if control.stem.startswith(f"c_{s}_")]
        if not owners:
            failures.append(f"{control.name} names no proof: a control is c_<proof>_<what>.g")
            continue
        controls.setdefault(max(owners, key=len), []).append(control)
    return scripts, controls, failures


def stamp_differences(name, script, controls):
    stamp = script.with_suffix(".stamp")
    if not stamp.exists():
        return ["it has no stamp"]
    want = read_stamp(stamp)
    got, missing = stamp_lines(script, controls)
    if missing:
        return missing
    return [why for why in compare(want, got)]


def compare(want, got):
    for key in sorted(set(want) | set(got)):
        if key in ("hints", "nonzero"):
            continue
        if key not in want:
            yield f"its stamp has no line for {key}"
        elif key not in got:
            yield f"its stamp has a line for {key}, which it no longer has"
        elif want[key] != got[key]:
            yield f"{key} differs from its stamp"


def stamp_lines(script, controls):
    """What a stamp of this script would hold, but the counts only a proof
    gives, and which restated items could not be found."""
    lines = {"script": sha256(script.read_bytes()), "pin": pin()}
    missing = []
    restated = restates(script)
    if not restated:
        missing.append("it names no Rust it restates: a proof starts with # restates: lines")
    for path, item, _commit in restated:
        text = working_file(path)
        code = None if text is None else item_code(path, text, item)
        if code is None:
            missing.append(f"{path} {item} is no longer found")
            continue
        lines[f"restates {path} {item}"] = sha256(code.encode())
    for control in controls:
        lines[f"control {control.name}"] = sha256(control.read_bytes())
    return lines, missing


def restates(script):
    found = []
    for line in script.read_text(encoding="utf-8").split("\n"):
        if not line.startswith("#"):
            break
        m = RESTATES.match(line)
        if m:
            found.append(m.groups())
        elif line.startswith("# restates"):
            raise SystemExit(f"{script.name}: a restates line it cannot read: {line}")
    return found


def working_file(path):
    full = ROOT / path
    return full.read_text(encoding="utf-8") if full.is_file() else None


def read_stamp(stamp):
    lines = {}
    for line in stamp.read_text(encoding="utf-8").split("\n"):
        if line.strip():
            key, _, value = line.rpartition(" ")
            lines[key] = value
    return lines


def pin():
    """Gappa, Coq and flocq come from one nixpkgs commit, gappalib from the nix
    file beside this script, and every run is at one precision. A change to
    any of them is a different pin."""
    nix = sha256(NIX_FILE.read_bytes()) if NIX_FILE.exists() else "missing"
    return f"{NIXPKGS}+gappalib18.nix:{nix}+{PRECISION}"


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def item_code(path, text, item):
    """The item's code as §464.10 2 counts it, or None where it is not found
    once: `just src`'s extent of it, with every `//` line and blank line taken
    out and every run of white space made one space. A dependency named in
    Cargo.lock is its [[package]] entry there."""
    extent = package_entry(text, item) if path == "Cargo.lock" else rust_item(text, item)
    if extent is None:
        return None
    kept = []
    for line in extent:
        bare = line.strip()
        if bare and not bare.startswith("//"):
            kept.append(re.sub(r"\s+", " ", line))
    return "\n".join(kept) + "\n"


LEAD = (
    r"^[ \t]*(pub(\([^)]*\))?[ \t]+)?(default[ \t]+)?((const|async|unsafe)[ \t]+)*"
    r'(extern[ \t]+"[^"]*"[ \t]+)?'
)
KINDS = "fn|struct|enum|trait|type|union|mod|static"


def rust_item(text, item):
    """`just src`'s extent of the one item of this name in the file, read off
    rustfmt's layout as that recipe reads it: back to the blank line above
    the definition, forward to the first `}` at its own indentation.

    `Owner::name` is the one `name` inside the file's `impl` blocks for
    `Owner`, for a name the file defines more than once, as plane.rs does
    `on_floats`. `just src` takes no owner, so for those the selftest holds
    the extent to `just src`'s in a copy where the other definitions are
    renamed."""
    owner, _, name = item.rpartition("::")
    pattern = re.compile(
        LEAD + rf"(({KINDS})[ \t]+{name}\b|const[ \t]+{name}[ \t]*:|macro_rules![ \t]*{name}\b)"
    )
    lines = text.split("\n")
    inside = impl_lines(lines, owner) if owner else range(len(lines))
    hits = [i for i in inside if pattern.search(lines[i])]
    if len(hits) != 1:
        return None
    at = hits[0]
    definition = lines[at]
    pad = definition[: len(definition) - len(definition.lstrip(" "))]
    start = at
    while start > 0:
        above = lines[start - 1].strip()
        if above == "" or above.endswith("{") or above.startswith("}"):
            break
        start -= 1
    trimmed = definition.rstrip()
    if trimmed.endswith(";") or ("{" in definition and trimmed.endswith("}")):
        return lines[start : at + 1]
    for end in range(at + 1, len(lines)):
        if lines[end].startswith(pad + "}"):
            return lines[start : end + 1]
    return None


def impl_lines(lines, owner):
    """The lines inside the file's top-level `impl` blocks for `owner`, its
    own and each trait's, but not a block inside a module such as `tests`."""
    opens = re.compile(rf"^impl(<.*?>)?[ \t]+((\S.*[ \t])?for[ \t]+)?{owner}\b")
    inside = []
    for at, line in enumerate(lines):
        if not opens.match(line) or line.rstrip().endswith("}"):
            continue
        end = next(i for i in range(at + 1, len(lines)) if lines[i].startswith("}"))
        inside += range(at + 1, end)
    return inside


def package_entry(text, name):
    entries = [e for e in text.split("\n\n") if e.strip().startswith("[[package]]")]
    found = [e for e in entries if f'\nname = "{name}"\n' in e + "\n"]
    if len(found) != 1:
        return None
    return found[0].strip("\n").split("\n")


def prove(args):
    scripts, controls, failures = read_tree()
    if failures:
        for failure in failures:
            print(f"prove-bounds: {failure}", file=sys.stderr)
        return 1
    wanted = [a[: -len(".g")] if a.endswith(".g") else a for a in args] or sorted(scripts)
    unknown = [w for w in wanted if w not in scripts]
    if unknown:
        print(f"prove-bounds: no proof named {', '.join(unknown)} in proofs/gappa/", file=sys.stderr)
        return 2
    coq = CoqPaths()
    failed = []
    for name in wanted:
        print(f"== {name}.g")
        if not prove_one(name, scripts[name], sorted(controls.get(name, [])), coq):
            failed.append(name)
    if failed:
        print(f"prove-bounds: not stamped: {', '.join(f + '.g' for f in failed)}")
        return 1
    print(f"prove-bounds: stamped {len(wanted)}")
    return 0


def prove_one(name, script, controls, coq):
    moved = code_moved_since_reading(script)
    if moved:
        for why in moved:
            print(f"  refused: {why}")
        print("  read the script against the code as it is now, then move its")
        print("  restates line to a commit after that reading (§464.10 3 c)")
        return False
    lines, missing = stamp_lines(script, controls)
    if missing:
        for why in missing:
            print(f"  refused: {why}")
        return False
    proved = gappa([PRECISION, str(script)])
    if proved.returncode != 0:
        print(f"  gappa did not prove it, exit {proved.returncode}:")
        print(indent(proved.stderr, 4))
        return False
    for control in controls:
        run = gappa([PRECISION, str(control)])
        if run.returncode != 1 or "some properties were not satisfied" not in run.stderr:
            print(f"  control {control.name} did not fail on its goal, exit {run.returncode}:")
            print(indent(run.stderr, 4))
            return False
        print(f"  control {control.name} fails, as it must")
    with tempfile.TemporaryDirectory(prefix=f"prove-{name}-") as work:
        work = pathlib.Path(work)
        written = gappa([PRECISION, "-Bcoq", str(script)])
        if written.returncode != 0:
            print(f"  gappa -Bcoq failed, exit {written.returncode}:")
            print(indent(written.stderr, 4))
            return False
        counts = field_and_coqc(work, name, written.stdout, coq)
    if counts is None:
        return False
    lines["hints"], lines["nonzero"] = (str(c) for c in counts)
    body = "".join(f"{key} {lines[key]}\n" for key in sorted(lines))
    script.with_suffix(".stamp").write_text(body, encoding="utf-8")
    print(f"  proved; coqc accepts it standing on {counts[0]} hints and {counts[1]} nonzero facts")
    return True


def code_moved_since_reading(script):
    moved = []
    for path, item, commit in restates(script):
        then = git_show(commit, path)
        now = working_file(path)
        if then is None:
            moved.append(f"{path} does not exist at {commit}")
            continue
        old = item_code(path, then, item)
        new = None if now is None else item_code(path, now, item)
        if old is None:
            moved.append(f"{path} {item} is not found once at {commit}")
        elif new != old:
            moved.append(f"{path} {item}'s code is not what it was at {commit}")
    return moved


def git_show(commit, path):
    run = subprocess.run(
        ["git", "-C", str(ROOT), "show", f"{commit}:{path}"], capture_output=True, text=True
    )
    return run.stdout if run.returncode == 0 else None


def gappa(args):
    return subprocess.run(
        NIX + ["shell", f"{NIXPKGS}#gappa", "--command", "gappa"] + args,
        capture_output=True,
        text=True,
    )


class CoqPaths:
    """Where gappalib 1.8.0 and flocq live in the nix store, built once a run."""

    def __init__(self):
        gappalib = nix_out(["build", "--impure", "--no-link", "--print-out-paths", "-f", str(NIX_FILE)])
        flocq = nix_out(["build", "--no-link", "--print-out-paths", f"{NIXPKGS}#coqPackages_8_20.flocq"])
        self.coqpath = ":".join(f"{p}/lib/coq/8.20/user-contrib" for p in (gappalib, flocq))

    def coqc(self, file):
        return subprocess.run(
            NIX + ["shell", f"{NIXPKGS}#coqPackages_8_20.coq", "--command", "coqc", str(file)],
            capture_output=True,
            text=True,
            cwd=file.parent,
            env={**os.environ, "COQPATH": self.coqpath},
        )


def nix_out(args):
    run = subprocess.run(NIX + args, capture_output=True, text=True)
    if run.returncode != 0:
        raise SystemExit(f"nix {' '.join(args)} failed:\n{run.stderr}")
    return run.stdout.strip().split("\n")[-1]


def indent(text, n):
    return "\n".join(" " * n + line for line in text.strip().split("\n")[-40:])


DEFINITION = re.compile(r"^Definition (\w+) := (.*)\.( \(\*.*\*\))?$")
HINT = re.compile(r"^Hypothesis (a\d+) : (.*)\.$")
NOT_ARITHMETIC = ("BND", "makepairF", "Float2", "REL", "ABS")

CLOSE_NZ = """Ltac close_nz :=
  first [ assumption
        | apply Coq.Reals.Rbasic_fun.Rabs_no_R0; assumption
        | match goal with H : ?X <> 0%R |- ?E <> 0%R =>
            let Hz := fresh in
            intro Hz; apply H; transitivity E; [ring | exact Hz]
          end ]."""


def field_and_coqc(work, name, coq_proof, coq):
    """§464.10 4's transform, with its table read off Coq rather than by hand.

    Each number Gappa's proof defines is taken to be nonzero, as a section
    hypothesis `nz_X` declared just after its definition. A first `coqc` asks
    `field` to prove each hint from those; a hint it takes becomes a Lemma, one
    it does not stays a Hypothesis. A second `coqc` checks the result, and its
    `Print Assumptions` of the last lemma names what the proof stands on: the
    hints kept and the nonzero facts used. A fact no proof uses is not listed,
    so declaring one for every number states nothing the count includes.
    """
    lines = coq_proof.rstrip("\n").split("\n")
    last = [m.group(1) for m in (re.match(r"^Lemma (\w+) :", l) for l in lines) if m]
    if not last or "End Generated_by_Gappa." not in lines:
        print("  gappa -Bcoq wrote no section of lemmas this transform knows")
        return None
    hints = [m.group(1) for m in (HINT.match(l) for l in lines) if m]

    trial = work / f"{name}_field.v"
    trial.write_text(transformed(lines, taken=set(), trial=True, last=last[-1]), encoding="utf-8")
    run = coq.coqc(trial)
    if run.returncode != 0:
        print("  coqc refused the field trial:")
        print(indent(run.stdout + run.stderr, 4))
        return None
    taken = set(re.findall(r"^TAKEN (a\d+)$", run.stdout, flags=re.M))

    proved = work / f"{name}_proved.v"
    proved.write_text(transformed(lines, taken=taken, trial=False, last=last[-1]), encoding="utf-8")
    run = coq.coqc(proved)
    if run.returncode != 0:
        print("  coqc refused the proof:")
        print(indent(run.stdout + run.stderr, 4))
        return None
    if "Section Variables:" not in run.stdout:
        print("  coqc printed no Section Variables, so what the proof stands on is unread")
        return None
    stands_on = run.stdout.split("Section Variables:", 1)[1].split("Axioms:", 1)[0]
    kept = sorted(set(re.findall(r"^(a\d+)\s+:", stands_on, flags=re.M)))
    nonzero = sorted(set(re.findall(r"^(nz_\w+)\s+:", stands_on, flags=re.M)))
    if set(kept) - (set(hints) - taken):
        print(f"  the proof stands on hints field took: {', '.join(sorted(set(kept) & taken))}")
        return None
    print(f"  field took {len(taken)} of {len(hints)} hints")
    return len(kept), len(nonzero)


def transformed(lines, taken, trial, last):
    out = ["From Coq Require Import Reals Field."]
    defined = []
    for line in lines:
        d = DEFINITION.match(line)
        h = HINT.match(line)
        if line == "End Generated_by_Gappa." and not trial:
            out.append(f"Print Assumptions {last}.")
        if h and (trial or h.group(1) not in taken):
            out.append(line)
            if trial:
                out += [
                    f"Goal {h.group(2)}.",
                    "Proof.",
                    f"  tryif solve [ {field_proof(defined)} ]",
                    f'  then idtac "TAKEN {h.group(1)}" else idtac "KEPT {h.group(1)}".',
                    "Abort.",
                ]
            continue
        if h:
            out += [f"Lemma {h.group(1)} : {h.group(2)}.", "Proof.", f"  {field_proof(defined)}.", "Qed."]
            continue
        out.append(line)
        if line == "Section Generated_by_Gappa.":
            out.append(CLOSE_NZ)
        if d:
            defined.append((d.group(1), d.group(2)))
            if d.group(1).startswith("_") and not d.group(2).startswith(NOT_ARITHMETIC):
                out.append(f"Hypothesis nz_{d.group(1)} : {d.group(1)} <> 0%R.")
    return "\n".join(out) + "\n"


def field_proof(defined):
    arithmetic = [
        n for n, body in defined if "rounding_float" not in body and not body.startswith(NOT_ARITHMETIC)
    ]
    unfold = f"try unfold {', '.join(reversed(arithmetic))}; " if arithmetic else ""
    return f"{unfold}try unfold Float1; field; repeat split; close_nz"


if __name__ == "__main__":
    sys.exit(main(sys.argv))
