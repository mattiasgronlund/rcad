#!/usr/bin/env python3
"""The PreToolUse hook's judgement: does this tool call write docs/queue.md?

Reads the hook payload on stdin and prints a deny decision, or nothing. Edit and
Write are refused on the file itself. A Bash command is refused when it *does*
something to the file — `sed -i` or `perl -i` on it, a redirect or `tee` into
it, `cp` or `mv` onto it, a script that opens it for writing — and never merely
because the path appears in it. On 2026-09-26 the patching hook refused a
read-only Python script for naming queue.md, and a guard that cannot tell
reading from writing gets routed round. `docs/decisions.md` §263.

git is left alone: `git checkout --theirs`, `restore` and every merge write the
file, and git is how a conflict is settled.

A command that runs the `pin` or `unpin` verb is not refused but asked about:
pinning a row is Mattias's call, and the ask puts it in front of him even in
auto mode (§430). He can run the verbs himself with a `!` command, which no
hook sees.
"""

import ast
import json
import os
import re
import shlex
import sys

QUEUE = "docs/queue.md"
SCRATCH = re.compile(r"^/(tmp|var/tmp)/|scratchpad|(^|/)target/")
INTERPRETERS = re.compile(r"^(python[0-9.]*|perl|ruby|node|bash|sh|zsh|dash)$")
WRAPPERS = {"sudo", "command", "nohup", "time", "nice", "exec", "builtin"}
SEPARATORS = {";", "&&", "||", "|", "&", "\n", "(", ")", ";;", "|&", "{", "}"}

REASON = (
    "docs/queue.md is changed through `just queue` now, which checks the row before it writes "
    "and holds it against the other dispatcher (docs/decisions.md §263): "
    "`just queue take <id> <branch>`, `add <tier> <model> <write set> <what>`, `done <id> <sha>`, "
    "`drop <id> §N`, `park <id> §N`, `after <id> <id>[,<id>]`, `tier <id> <tier>`, `tag <id> <class|small|->`, "
    "and `pick <id> ours|theirs` after a "
    "merge conflict. Reading it is fine: grep, git show, a read-only script. This call was refused because {why}."
)
PIN_REASON = (
    "Pinning a queue row is Mattias's call (docs/decisions.md §430): a pinned row goes first in the fill, "
    "and no dispatcher's verb moves it. This asks him because the command runs `{verb}`. A dispatcher "
    "takes a pinned row, closes it with `done`, or asks Mattias to unpin it."
)
PIN_VERBS = ("pin", "unpin")


def main():
    payload = json.load(sys.stdin)
    tool = payload.get("tool_name") or ""
    given = payload.get("tool_input") or {}
    verb = None
    if "command" in given and tool in ("", "Bash"):
        command = given.get("command") or ""
        why = writes(command, {}, [])
        verb = pin_verb_in(command, {})
    else:
        path = given.get("file_path") or given.get("notebook_path") or ""
        why = f"{tool or 'this tool'} writes {path}" if is_queue(path, []) else None
    if why:
        decide("deny", REASON.format(why=why))
    elif verb:
        decide("ask", PIN_REASON.format(verb=verb))


def decide(decision, reason):
    json.dump({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": decision,
        "permissionDecisionReason": reason,
    }}, sys.stdout)


def is_queue(path, cds):
    if not path or SCRATCH.search(path):
        return False
    if not path.startswith("/") and cds:
        path = cds[-1].rstrip("/") + "/" + path
    normal = os.path.normpath(path)
    return normal == QUEUE or normal.endswith("/" + QUEUE)


def writes(command, variables, cds):
    """Why this shell text writes the queue, or None."""
    code, bodies = split_heredocs(command)
    for words, redirects, heredocs in simple_commands(code, bodies, variables):
        why = command_writes(words, redirects, heredocs, variables, cds)
        if why:
            return why
    return None


def split_heredocs(command):
    """The shell code, and each heredoc body in the order its `<<` appears."""
    code, bodies = [], []
    lines = command.split("\n")
    index = 0
    while index < len(lines):
        line = lines[index]
        code.append(line)
        index += 1
        for match in re.finditer(r"(?<!<)<<(-?)[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2", line):
            body = []
            while index < len(lines) and lines[index].strip() != match.group(3):
                body.append(lines[index])
                index += 1
            index += 1
            bodies.append("\n".join(body))
    return "\n".join(code), bodies


def simple_commands(code, bodies, variables):
    """Each simple command as (words, redirects, heredoc bodies), variables expanded."""
    code = code.replace("\\\n", " ")
    lexer = shlex.shlex(code, posix=True, punctuation_chars=True)
    lexer.whitespace = " \t\r"
    lexer.wordchars += "$:{}@%+,!^[]"
    try:
        tokens = list(lexer)
    except ValueError:
        tokens = code.split()
    bodies = list(bodies)
    words, redirects, heredocs = [], [], []
    position = 0
    while position < len(tokens):
        token = tokens[position]
        position += 1
        if token in SEPARATORS or (set(token) <= set("();|&\n") and token):
            if words or redirects:
                yield words, redirects, heredocs
            words, redirects, heredocs = [], [], []
            continue
        if token.startswith("<<") and set(token) <= set("<-"):
            if token == "<<<":
                position += 1
                continue
            position += 1
            if bodies:
                heredocs.append(bodies.pop(0))
            continue
        if ">" in token and set(token) <= set("<>&|("):
            target = tokens[position] if position < len(tokens) else ""
            position += 1
            if token == ">&" and (target.isdigit() or target == "-"):
                continue
            redirects.append(expand(target, variables))
            continue
        if set(token) <= set("<("):
            position += 1
            continue
        assignment = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$", token, re.S)
        if assignment and not words:
            variables[assignment.group(1)] = expand(assignment.group(2), variables)
            continue
        words.append(expand(token, variables))
    if words or redirects:
        yield words, redirects, heredocs


def expand(word, variables):
    for name, value in variables.items():
        word = word.replace("${" + name + "}", value).replace("$" + name, value)
    return word


def command_writes(words, redirects, heredocs, variables, cds):
    for target in redirects:
        if is_queue(target, cds):
            return f"a redirect writes {target}"
    words = unwrap(words)
    if not words:
        return None
    name, args = os.path.basename(words[0]), words[1:]
    if name == "cd" and args:
        cds.append(args[0])
        return None
    queue_args = [a for a in args if is_queue(a, cds)]
    if name == "tee" and queue_args:
        return f"tee writes {queue_args[0]}"
    if name in ("sed", "perl", "ruby") and queue_args and in_place(args):
        return f"{name} -i edits {queue_args[0]} in place"
    if name in ("awk", "gawk"):
        if queue_args and "inplace" in args:
            return f"awk -i inplace edits {queue_args[0]}"
        program = next((a for a in args if not a.startswith("-")), "")
        if re.search(r">>?\s*\"[^\"]*" + re.escape(QUEUE), program):
            return "an awk program prints into docs/queue.md"
    if name in ("cp", "install", "rsync", "ln") and args and is_queue(args[-1], cds):
        return f"{name} writes onto {args[-1]}"
    if name in ("mv", "rm", "truncate", "shred") and queue_args:
        return f"{name} changes {queue_args[0]}"
    if name == "dd" and any(a.startswith("of=") and is_queue(a[3:], cds) for a in args):
        return "dd writes docs/queue.md"
    if INTERPRETERS.match(name):
        return interpreter_writes(name, args, heredocs, variables, cds)
    return None


def unwrap(words):
    while words:
        head = os.path.basename(words[0])
        if head in WRAPPERS:
            words = words[1:]
        elif head == "env":
            words = [w for w in words[1:] if not re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", w)]
        elif head == "mise" and "--" in words:
            words = words[words.index("--") + 1:]
        else:
            return words
    return words


def in_place(args):
    for arg in args:
        if arg == "--in-place" or arg.startswith("--in-place="):
            return True
        if re.match(r"^-[A-Za-z]*i", arg) and not arg.startswith("--"):
            return True
    return False


def interpreter_writes(name, args, heredocs, variables, cds):
    scripts = list(heredocs)
    argv = []
    flag = {"python": "-c", "bash": "-c", "sh": "-c", "zsh": "-c", "dash": "-c"}.get(
        re.sub(r"[0-9.]+$", "", name), "-e")
    for position, arg in enumerate(args):
        if arg == flag and position + 1 < len(args):
            scripts.append(args[position + 1])
            argv = args[position + 2:]
            break
        if arg == "-":
            argv = args[position + 1:]
            break
    queue_in_argv = any(is_queue(a, cds) for a in argv)
    for script in scripts:
        if name.startswith("python"):
            why = python_writes(script, queue_in_argv, cds)
        elif name in ("bash", "sh", "zsh", "dash"):
            why = writes(script, dict(variables), list(cds))
        else:
            why = other_script_writes(name, script, queue_in_argv)
        if why:
            return why
    return None


ARGV = "\0argv"


def python_writes(source, queue_in_argv, cds):
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return None
    names = {}
    for _ in range(2):
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
                text = path_text(node.value, names)
                if text is not None:
                    names[node.targets[0].id] = text

    def queue(expr):
        text = path_text(expr, names)
        if text == ARGV:
            return queue_in_argv
        return text is not None and is_queue(text.replace("?", "x"), cds)

    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        func = node.func
        called = func.id if isinstance(func, ast.Name) else func.attr if isinstance(func, ast.Attribute) else ""
        owner = func.value.id if isinstance(func, ast.Attribute) and isinstance(func.value, ast.Name) else ""
        args = node.args
        keywords = {k.arg: k.value for k in node.keywords if k.arg}
        if called == "open" and owner in ("", "io", "codecs") and args and queue(args[0]):
            mode = args[1] if len(args) > 1 else keywords.get("mode")
            if mode is not None and not (isinstance(mode, ast.Constant) and not set(str(mode.value)) & set("wax+")):
                return "a Python script opens docs/queue.md for writing"
        if called == "open" and owner == "os" and args and queue(args[0]):
            flags = ast.unparse(args[1]) if len(args) > 1 else ""
            if re.search(r"O_(WRONLY|RDWR|APPEND|CREAT|TRUNC)", flags):
                return "a Python script opens docs/queue.md for writing"
        if isinstance(func, ast.Attribute) and func.attr in ("write_text", "write_bytes", "touch", "unlink") \
                and queue(func.value):
            return f"a Python script calls {func.attr} on docs/queue.md"
        if isinstance(func, ast.Attribute) and func.attr == "open" and owner not in ("os", "io", "codecs") \
                and queue(func.value):
            mode = args[0] if args else keywords.get("mode")
            if mode is not None and not (isinstance(mode, ast.Constant) and not set(str(mode.value)) & set("wax+")):
                return "a Python script opens docs/queue.md for writing"
        if isinstance(func, ast.Attribute) and func.attr in ("rename", "replace") \
                and path_text(func.value, names) is not None and args and (queue(args[0]) or queue(func.value)):
            return f"a Python script moves a file onto or off docs/queue.md with {func.attr}"
        if owner in ("shutil", "os") and called in ("copy", "copy2", "copyfile", "move", "replace", "rename") \
                and len(args) > 1 and (queue(args[1]) or (called in ("move", "replace", "rename") and queue(args[0]))):
            return f"a Python script writes docs/queue.md with {owner}.{called}"
        if owner == "os" and called in ("remove", "unlink", "truncate") and args and queue(args[0]):
            return f"a Python script calls os.{called} on docs/queue.md"
        if called in ("input", "FileInput") and owner == "fileinput" and "inplace" in keywords \
                and any(queue(a) for a in args + [keywords.get("files")] if a is not None):
            return "a Python script edits docs/queue.md in place with fileinput"
        if owner == "os" and called == "system" and args and isinstance(args[0], ast.Constant):
            why = writes(str(args[0].value), {}, list(cds))
            if why:
                return why
        if owner == "subprocess" and args and isinstance(args[0], (ast.List, ast.Tuple)) \
                and all(isinstance(e, ast.Constant) for e in args[0].elts):
            why = writes(" ".join(shlex.quote(str(e.value)) for e in args[0].elts), {}, list(cds))
            if why:
                return why
    return None


def path_text(expr, names):
    """The path an expression spells, as far as it can be read; `?` for a part it cannot."""
    if isinstance(expr, ast.Constant) and isinstance(expr.value, str):
        return expr.value
    if isinstance(expr, ast.JoinedStr):
        return "".join(v.value if isinstance(v, ast.Constant) else "?" for v in expr.values)
    if isinstance(expr, ast.Name):
        return names.get(expr.id)
    if isinstance(expr, ast.BinOp) and isinstance(expr.op, (ast.Div, ast.Add)):
        left, right = path_text(expr.left, names), path_text(expr.right, names)
        if right is None:
            return None
        joiner = "/" if isinstance(expr.op, ast.Div) else ""
        return (left if left is not None else "?") + joiner + right
    if isinstance(expr, ast.Subscript) and ast.unparse(expr.value) == "sys.argv":
        return ARGV
    if isinstance(expr, ast.Call):
        func = expr.func
        called = func.id if isinstance(func, ast.Name) else func.attr if isinstance(func, ast.Attribute) else ""
        if called in ("Path", "PurePath", "PosixPath", "join") and expr.args:
            parts = [path_text(a, names) for a in expr.args]
            if parts[-1] is None:
                return None
            return "/".join(p if p is not None else "?" for p in parts)
        if called in ("resolve", "absolute", "expanduser", "abspath", "realpath", "normpath") \
                and isinstance(func, ast.Attribute):
            return path_text(expr.args[0] if expr.args else func.value, names)
    return None


def other_script_writes(name, script, queue_in_argv):
    if QUEUE not in script and not queue_in_argv:
        return None
    idioms = {
        "perl": r"open\s*\(?\s*[^,]+,\s*['\"]\s*(\+<|\+?>)",
        "ruby": r"File\.(write|open\s*\([^)]*['\"][wa+])",
        "node": r"(writeFile|appendFile|createWriteStream|copyFile|rename|truncate)(Sync)?\s*\(",
    }
    if re.search(idioms.get(name, r"$^"), script):
        return f"a {name} script names docs/queue.md and writes a file"
    return None


def pin_verb_in(command, variables):
    """`pin` or `unpin` if this shell text runs one as a queue verb, else None (§430)."""
    code, bodies = split_heredocs(command)
    for words, _, heredocs in simple_commands(code, bodies, variables):
        verb = pin_verb_of(unwrap(words), heredocs, variables)
        if verb:
            return verb
    return None


def pin_verb_of(words, heredocs, variables):
    """The verb is the word after `queue`, or after `queue.py`; after `apply`, also every word after a `+`."""
    if not words:
        return None
    name, args = os.path.basename(words[0]), words[1:]
    if name in ("bash", "sh", "zsh", "dash"):
        scripts = list(heredocs)
        if "-c" in args[:-1]:
            scripts.append(args[args.index("-c") + 1])
        return next((verb for verb in (pin_verb_in(script, dict(variables)) for script in scripts) if verb), None)
    if name == "just" and "queue" in args:
        steps = args[args.index("queue") + 1:]
    elif re.match(r"^python[0-9.]*$", name) and any(os.path.basename(arg) == "queue.py" for arg in args):
        steps = args[[os.path.basename(arg) for arg in args].index("queue.py") + 1:]
    else:
        return None
    verbs = steps[:1]
    if verbs == ["apply"]:
        verbs = steps[1:2] + [steps[index + 1] for index, word in enumerate(steps[:-1]) if word == "+"]
    return next((verb for verb in verbs if verb in PIN_VERBS), None)


if __name__ == "__main__":
    main()
