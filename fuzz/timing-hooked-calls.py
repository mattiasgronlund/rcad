#!/usr/bin/env python3
"""Fails when the timing crate reaches a memcmp-family hook from outside `is`.

docs/decisions.d/273 found five calls, in a fuzz target's own build, from
`rcad-fuzz-timing`'s code to `bcmp`: two in `read_back`, one in
`how_slow_the_name_says`, one in `core`'s `StrSearcher::new` (reached from
`split_once("us-")`), and one in [`SlowRun::is`]. Every one of those goes
through ASan's interceptor to libFuzzer's `__sanitizer_weak_hook_memcmp`,
which — mid-draw — hands the two sides to the `CMP` table libFuzzer's
mutations draw from whenever they come out unequal. So a `totals` file or a
`slowest/` name the timing crate is merely reading back can steer the
mutator, unless the only such call left is `SlowRun::is`'s own — which
compares two draws' bytes only after their hashes already matched, so it is
expected to run and expected to come out equal. `docs/decisions.d/273` has
the read-out this script replays; item 273.7 is what this script folds in.

Rust's fuzz build routes a call to a hooked function through the GOT rather
than a direct `call <name>`, even though the whole binary is statically
linked (ASan and libFuzzer are both built in): `call *SLOT(%rip)`, naming
the slot in a comment, or a load of the slot into a register followed later
by `call *%reg`. This finds every call or tail jump of either kind, in every
function whose *symbol* names `rcad_fuzz_timing` — which covers every
function the crate's own source defines, however the compiler inlined it —
and reads each hit's inline chain with `addr2line -i` to tell whether it
belongs to `SlowRun::is` or to something else.

A call that leaves a `rcad_fuzz_timing` function for one outside it — a
direct `call <name>` the compiler did not inline — is followed too, as far
as it goes: into `core`, `alloc`, `std`, or back into another crate's own
code, however many hops, as long as every hop is itself a direct call this
scan can read a name for. A path that ends at a hooked function counts as
an offender the same as a direct one, and prints as a chain, `crate
function -> ... -> hooked function`, rather than just a count.
`docs/decisions.d/273` §273.8 found one of these that this scan's old
filter — "the symbol names `rcad_fuzz_timing`" — could never have caught:
`how_slow_the_name_says`'s `split_once("us-")` reaching `bcmp` through
`core`'s `StrSearcher::new`.

A `call *SLOT(%rip)` whose slot is not on the hooked list is followed the
same way, through the same relocation table `hooked_slots` already reads
(`R_X86_64_RELATIVE`, from `readelf -rW`): resolved to an address, and when a
function symbol starts there, recorded as a `calls_out` edge exactly like a
direct `call <name>`, so the walk above carries on through it.
`docs/decisions.d/419` §419.5 found the wall this closes: `merge_out` reaches
`core`'s `StrSearcher::new` — which itself calls `bcmp`, `docs/decisions.d/273`
§273.3's fifth offender — through a GOT slot that *names* `StrSearcher::new`
by relocation rather than by a `call <name>` instruction, so the old scan had
no `calls_out` edge there for the walk to follow. A slot whose relocation
resolves to an address with no function symbol at all is not a call this scan
can name a target for either, the same as a `call *%reg` it never traced to a
slot: it falls into the "can leave" bucket below rather than being dropped.

**What this cannot see**: a call through a computed pointer — one where the
register a `call *%reg` uses was not last set by a direct, in-function load
of a hooked GOT slot, or a `call *SLOT(%rip)` whose relocation names no
function symbol at all. Most `jmp *%reg` of this kind is not that: it is the
tail of a relative jump table, the lowering of a match under `-fPIE` — `lea
table(%rip),%base; movslq (%base,%idx,4),%off; add %base,%off; jmp *%off` —
whose table holds offsets from its own base, so the jump can only land
inside the function it is in, never at an imported symbol. This scan
recognises that whole sequence, the load and the add of the same base
register and the jump through the result, and reports it apart, as a jump
that stays inside its own function. Everything else indirect that this scan
could not trace to a hooked-slot load — a bare `jmp *%reg` the sequence does
not match, every `call *%reg`, or a call through a memory operand (a vtable
slot, a stored function pointer) — stays in the "can leave" bucket, since it
might reach a target this scan did not name. `merge_out` alone made 12 such
calls before docs/decisions.d/273's fix (counted below, `--computed`), and
none of them can be ruled in or out by this scan; a `possibly reaches an
unrecorded target` line says how many a given binary has. Closing that would
need either a real points-to analysis of the binary or a runtime interposer
that walks the caller's stack on every ASan memcmp hook — both bigger than
this row. docs/decisions.d/273 says so and names the count for each target
this script has been run on; docs/decisions.d/419 names the two buckets'
counts for the five targets after the split added here (Q1444). The walk
out of the crate stops at the same wall: a computed call met in a function
it reaches, same as one met inside the crate, is counted in that bucket and
not followed either — nothing past it is silently dropped, but nothing past
it is named.

usage: timing-hooked-calls.py <binary> [<binary> ...]
exit 0 and a `clean` line per binary if none reaches a hook outside `is`;
exit 1 and the offending call(s) — address, kind, and inline chain — if any
binary has one, or if a load of a hooked slot could not be matched to any
call at all (the one case this script can tell you it might have missed a
call, rather than merely one it cannot see, per the computed-pointer
paragraph above).
"""
import bisect
import re
import subprocess
import sys
from collections import Counter

HOOKED_BASE = [
    "bcmp", "memcmp", "strncmp", "strcmp", "strncasecmp",
    "strcasecmp", "strstr", "strcasestr", "memmem",
]
HOOKED = set()
for base in HOOKED_BASE:
    HOOKED |= {
        base,
        "__interceptor_" + base,
        "___interceptor_" + base,
        "__interceptor_trampoline_" + base,
    }

CALLER_SAVED = {"rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"}

REG64 = {}
for full, aliases in {
    "rax": ["eax", "ax", "al", "ah"], "rbx": ["ebx", "bx", "bl", "bh"],
    "rcx": ["ecx", "cx", "cl", "ch"], "rdx": ["edx", "dx", "dl", "dh"],
    "rsi": ["esi", "si", "sil"], "rdi": ["edi", "di", "dil"],
    "rbp": ["ebp", "bp", "bpl"], "rsp": ["esp", "sp", "spl"],
}.items():
    REG64[full] = full
    for alias in aliases:
        REG64[alias] = full
for n in range(8, 16):
    for suffix in ["", "d", "w", "b"]:
        REG64[f"r{n}{suffix}"] = f"r{n}"


def main():
    binaries = sys.argv[1:]
    if not binaries:
        print(__doc__.strip().splitlines()[-4], file=sys.stderr)
        return 2

    failed = False
    for binary in binaries:
        (
            hits, unmatched, computed, table_jumps, calls_out, takes_address,
            all_functions,
        ) = scan(binary)
        timing_functions = {f for f in all_functions if "rcad_fuzz_timing" in f}
        timing_hits = [h for h in hits if "rcad_fuzz_timing" in h["function"]]
        timing_unmatched = [
            load for fn, load in unmatched if fn and "rcad_fuzz_timing" in fn
        ]
        reachable = reachable_from(timing_functions, calls_out)
        timing_computed = [c for c in computed if c[0] in reachable]
        timing_table_jumps = [
            c for c in table_jumps if "rcad_fuzz_timing" in c[0]
        ]

        offenders = [h for h in timing_hits if not belongs_to_is(binary, h)]

        hits_by_function = {}
        for hit in hits:
            hits_by_function.setdefault(hit["function"], []).append(hit)
        memo = {}
        chain_offenders = []
        for function in sorted(timing_functions):
            found = reaches_hook(
                function,
                timing_functions,
                hits_by_function,
                calls_out,
                binary,
                memo,
                set(),
            )
            if found is not None and found[1]:
                hit, chain = found
                chain_offenders.append((function, chain, hit))

        already_reported = {function for function, _, _ in chain_offenders}
        pointer_offenders, taken_in_reach, hooked_elsewhere = through_pointers(
            binary,
            timing_functions,
            already_reported,
            hits_by_function,
            calls_out,
            takes_address,
        )

        print(f"{binary}:")
        print(
            f"  calls from rcad_fuzz_timing to a hooked function: {len(timing_hits)}"
            f" ({len(offenders)} outside SlowRun::is)"
        )
        print(
            f"  loads of a hooked slot in rcad_fuzz_timing with no call matched: "
            f"{len(timing_unmatched)}"
        )
        print(
            f"  calls or tail jumps in rcad_fuzz_timing through a computed "
            f"pointer that can leave the function, unscanned: "
            f"{len(timing_computed)}"
        )
        print(
            f"  tail jumps in rcad_fuzz_timing through a relative jump "
            f"table, staying inside their own function: "
            f"{len(timing_table_jumps)}"
        )
        print(
            f"  calls from rcad_fuzz_timing reaching a hooked function "
            f"through another function outside the crate: {len(chain_offenders)}"
        )
        print(
            f"  functions whose address rcad_fuzz_timing or what it reaches "
            f"takes, followed as if called: {len(taken_in_reach)}"
        )
        print(
            f"  calls from rcad_fuzz_timing reaching a hooked function "
            f"through a function pointer: {len(pointer_offenders)}"
        )
        print(
            f"  functions whose address only code outside that reach takes, "
            f"reaching a hooked function, not followed: {len(hooked_elsewhere)}"
        )
        for function in hooked_elsewhere:
            print(f"    {demangle(function)}")

        if offenders or timing_unmatched or chain_offenders or pointer_offenders:
            failed = True
            for hit in offenders:
                print_hit(binary, hit, "reaches a hook outside SlowRun::is")
            for function, chain, hit in chain_offenders + pointer_offenders:
                print_chain_offender(function, chain, hit)
            for fn, load in [
                (f, l) for f, l in unmatched if f and "rcad_fuzz_timing" in f
            ]:
                address, slot, reg = load
                print(
                    f"  {address:x}  load of hooked slot {slot:x} into %{reg}, "
                    f"never matched to a call — this scan cannot say it was safe"
                )
                print(f"    in {demangle(fn)}")
        else:
            print("  clean")

    return 1 if failed else 0


def scan(binary):
    """Every call/tail-jump to a hooked function, every unmatched load, every
    computed-pointer call or tail jump that can leave its function, every
    tail jump recognised as a relative jump table that cannot, every direct
    call or tail jump's own target function (the call graph a transitive
    walk follows), and every function seen as a disassembly header,
    anywhere in `binary`."""
    names_at = {}
    for line in run(["nm", binary]).splitlines():
        parts = line.split()
        if len(parts) == 3:
            names_at.setdefault(int(parts[0], 16), set()).add(parts[2])
    hooked_addresses = {a for a, names in names_at.items() if names & HOOKED}

    slot_target = {}
    for line in run(["readelf", "-rW", binary]).splitlines():
        parts = line.split()
        if len(parts) >= 4 and parts[2] == "R_X86_64_RELATIVE":
            slot_target[int(parts[0], 16)] = int(parts[3], 16)
    hooked_slots = {
        slot: target for slot, target in slot_target.items()
        if target in hooked_addresses
    }

    dis = run(["objdump", "-d", "--no-show-raw-insn", binary])

    hits = []
    unmatched_loads = []
    computed_calls = []
    table_jumps = []
    calls_out = {}
    references = []
    all_functions = set()
    function_at = {}
    function = None
    regs = {}
    open_loads = {}
    for line in dis.splitlines():
        header = re.match(r"^([0-9a-f]+) <(.*)>:$", line)
        if header:
            for load in open_loads.values():
                unmatched_loads.append((function, load))
            function, regs, open_loads = header.group(2), {}, {}
            all_functions.add(function)
            function_at[int(header.group(1), 16)] = function
            continue
        insn = re.match(r"^\s+([0-9a-f]+):\s+(\S+)\s*(.*)$", line)
        if not insn:
            continue
        address = int(insn.group(1), 16)
        mnemonic, rest = insn.group(2), insn.group(3)
        comment = None
        if "#" in rest:
            rest, comment = rest.split("#", 1)
            rest = rest.strip()
        operands = split_operands(rest)

        if mnemonic in ("call", "jmp"):
            target, kind, table_jump = None, None, False
            direct = re.match(r"^([0-9a-f]+) <([^>]+)>", rest)
            if direct:
                callee = direct.group(2).split("+")[0]
                calls_out.setdefault(function, []).append((address, callee))
            if direct and int(direct.group(1), 16) in hooked_addresses:
                target, kind = int(direct.group(1), 16), "direct"
            elif rest.startswith("*") and "(%rip)" in rest and comment:
                slot = int(comment.split()[0], 16)
                if slot in hooked_slots:
                    target, kind = hooked_slots[slot], f"GOT slot {slot:x}"
                else:
                    resolved = slot_target.get(slot)
                    callees = (
                        names_at.get(resolved, set())
                        if resolved is not None else set()
                    )
                    if callees:
                        for callee in sorted(callees):
                            calls_out.setdefault(function, []).append(
                                (address, callee)
                            )
                    else:
                        computed_calls.append((function, address, rest))
            elif re.match(r"^\*%\w+$", rest):
                reg = REG64.get(rest[2:])
                state = regs.get(reg)
                if state and state[0] == "slot":
                    target = hooked_slots[state[1]]
                    kind = f"%{reg} loaded at {state[2]:x} from GOT slot {state[1]:x}"
                    open_loads.pop(reg, None)
                # A relative jump table's tail: `lea table(%rip),%base;
                # movslq (%base,%idx,4),%off; add %base,%off; jmp *%off`.
                # The table holds offsets from `%base`, itself a fixed
                # address this function's own code computed, so the sum can
                # only land inside this function — never at an imported
                # symbol. Recognising the whole sequence, not just the
                # mnemonic, is the point: §273.9 found that a `jmp *%reg`
                # cannot be told apart from a genuine cross-function one by
                # mnemonic alone. Only `jmp` earns this; `call *%reg` never
                # does, cautious-side, even when the same registers line up.
                elif mnemonic == "jmp" and state and state[0] == "table_target":
                    table_jump = True
            # Anything else indirect — a register this scan could not trace
            # back to a hooked-slot load or a relative jump table, or a call
            # through a memory operand (a vtable slot, a stored function
            # pointer) — is a target this scan cannot name at all, not one
            # it named and found harmless. `merge_out` alone had 12 of
            # these before §273's fix (docs/decisions.d/273 §273.3, using a
            # different scan that also counts this way); see the module
            # docstring.
            if target is None and not table_jump and rest.startswith("*") and not (
                    "(%rip)" in rest and comment):
                computed_calls.append((function, address, rest))
            elif table_jump:
                table_jumps.append((function, address, rest))
            if target is not None:
                hits.append({
                    "function": function, "address": address,
                    "mnemonic": mnemonic, "kind": kind,
                    "target": sorted(names_at.get(target, [])),
                })
            if mnemonic == "call":
                for reg in CALLER_SAVED:
                    if reg in open_loads:
                        unmatched_loads.append((function, open_loads.pop(reg)))
                    regs[reg] = ("call", address)
            continue

        if comment and "(%rip)" in rest:
            referenced = int(comment.split()[0], 16)
            rip_operand = next(
                i for i, operand in enumerate(operands) if "(%rip)" in operand
            )
            stores_to_it = len(operands) > 1 and rip_operand == len(operands) - 1
            is_hooked_slot_load = mnemonic == "mov" and referenced in hooked_slots
            if not stores_to_it and not is_hooked_slot_load:
                references.append((function, address, referenced))

        if not operands:
            continue
        destination = operands[-1]
        reg = REG64.get(destination[1:]) if destination.startswith("%") else None
        if reg is None or mnemonic.startswith(("cmp", "test", "push", "bt")):
            continue
        if reg in open_loads:
            unmatched_loads.append((function, open_loads.pop(reg)))
        if (mnemonic == "mov" and len(operands) == 2 and "(%rip)" in operands[0]
                and comment and int(comment.split()[0], 16) in hooked_slots):
            slot = int(comment.split()[0], 16)
            regs[reg] = ("slot", slot, address)
            open_loads[reg] = (address, slot, reg)
        elif mnemonic == "mov" and operands[0].startswith("$"):
            regs[reg] = ("const", int(operands[0][1:], 16), address)
        # The three steps of a relative jump table's tail, tracked per
        # register so the `jmp *%reg` above can tell a real one from a
        # register that merely ends up holding some other computed value:
        # a rip-relative `lea` (the table's own fixed base address), a
        # `movslq` of a 4-byte table entry off that base (the offset), and
        # an `add` of the same base back into the offset (the target).
        elif mnemonic == "lea" and len(operands) == 2 and "(%rip)" in operands[0]:
            regs[reg] = ("table_base", address)
        elif (mnemonic == "movslq" and len(operands) == 2
                and regs.get(base_register(operands[0]), (None,))[0] == "table_base"):
            regs[reg] = ("table_offset", base_register(operands[0]), address)
        elif (mnemonic == "add" and len(operands) == 2
                and operands[0].startswith("%")
                and regs.get(reg, (None,))[0] == "table_offset"
                and regs[reg][1] == REG64.get(operands[0][1:])
                and regs.get(regs[reg][1], (None,))[0] == "table_base"):
            regs[reg] = ("table_target", address)
        else:
            regs[reg] = ("other", mnemonic, address)
    for load in open_loads.values():
        unmatched_loads.append((function, load))

    takes_address = functions_whose_address_is_taken(
        binary, references, slot_target, names_at, function_at
    )
    return (
        hits, unmatched_loads, computed_calls, table_jumps, calls_out,
        takes_address, all_functions,
    )


def functions_whose_address_is_taken(
    binary, references, slot_target, names_at, function_at
):
    """For every function, the functions whose address it can put in a
    register: `{function: [(address, taken function), ...]}`.

    A reference into code takes the address of the function it lands in. A
    reference into data — a `lea` of a vtable, a load of a GOT slot or of a
    static holding a pointer — can hand the code any pointer the data object
    there holds, and through any data object one of those points at in turn,
    so it takes every function those relocations name. A data object runs
    from the address referenced to the next address anything names: a
    symbol, a section's start or end, a relocation's target, or another
    reference. Where that runs past the real object's end, it takes more
    functions than the code can reach, never fewer.
    """
    code, boundaries = [], set()
    for line in run(["readelf", "-SW", binary]).splitlines():
        section = re.match(r"^\s*\[\s*\d+\]\s+(.*)$", line)
        if not section:
            continue
        fields = section.group(1).split()
        if len(fields) not in (9, 10) or not re.fullmatch(r"[0-9a-f]{16}", fields[2]):
            continue
        start, size = int(fields[2], 16), int(fields[4], 16)
        if start == 0:
            continue
        boundaries |= {start, start + size}
        if len(fields) == 10 and "X" in fields[6]:
            code.append((start, start + size))
    boundaries |= set(names_at) | set(slot_target.values())
    boundaries |= {referenced for _, _, referenced in references}
    boundaries = sorted(boundaries)
    slots = sorted(slot_target)
    function_starts = sorted(function_at)

    def in_code(address):
        return any(start <= address < end for start, end in code)

    def function_containing(address):
        index = bisect.bisect_right(function_starts, address) - 1
        return function_at[function_starts[index]] if index >= 0 else None

    behind = {}

    def functions_behind(referenced):
        if referenced in behind:
            return behind[referenced]
        found, seen, pending = set(), {referenced}, [referenced]
        while pending:
            address = pending.pop()
            if in_code(address):
                function = function_containing(address)
                if function is not None:
                    found.add(function)
                continue
            end_index = bisect.bisect_right(boundaries, address)
            end = boundaries[end_index] if end_index < len(boundaries) else address + 8
            first = bisect.bisect_left(slots, address)
            last = bisect.bisect_left(slots, end)
            for slot in slots[first:last]:
                pointer = slot_target[slot]
                if pointer not in seen:
                    seen.add(pointer)
                    pending.append(pointer)
        behind[referenced] = found
        return found

    takes_address = {}
    for function, address, referenced in references:
        for taken in sorted(functions_behind(referenced)):
            takes_address.setdefault(function, []).append((address, taken))
    return takes_address


def reachable_from(starts, calls_out):
    """Every function a function in `starts` reaches, directly or
    transitively, by a direct call or tail jump — `starts` included."""
    seen = set(starts)
    pending = list(starts)
    while pending:
        function = pending.pop()
        for _, callee in calls_out.get(function, []):
            if callee not in seen:
                seen.add(callee)
                pending.append(callee)
    return seen


def belongs_to_is(binary, hit):
    """True when `hit`'s inline chain passes through `SlowRun::is` itself.

    `addr2line -f -i` lists one (function, call-site file:line) pair per
    level of inlining, innermost first — except that its first pair keeps
    `-f`'s own habit of naming the *outer* concrete symbol next to the
    innermost source line, so the call site genuinely inside `is` shows up
    as a middle pair, not the first one: `is` calls `bytes == other` at
    line 776, `core` inlines a generic `eq` into that, and `eq` is what
    finally calls `bcmp`. A call `is` makes can itself be inlined anywhere
    `is` is inlined — into `offer`, into `stop_the_clock`, wherever — so
    this checks every function name the chain names, not the symbol the
    disassembly found the instruction under.
    """
    chain = addr2line_chain(binary, hit["address"])
    return "is" in chain[0::2]


def reaches_hook(
    function, timing_functions, hits_by_function, calls_out, binary, memo, on_stack
):
    """`None`, or the `(hit, chain)` by which `function` reaches a hook
    outside `timing_functions` — every function whose symbol names
    `rcad_fuzz_timing` — not excused as `SlowRun::is`'s own: the first one
    found by following `function`'s own direct calls out, however far,
    into `core`, `alloc`, `std`, or back into another crate's own code
    (`chain` the callees between `function` and the hit, nearest first). A
    hit inside `timing_functions` itself is never this call's to report:
    `main`'s own `offenders` already covers every one of those, on every
    function in `timing_functions` in turn, so a chain that merely lands in
    a second crate function would double it rather than add to it. A
    computed call cannot be followed past, so a hook reachable only
    through one stays unseen here, same as it always has inside the crate.
    """
    if function in memo:
        return memo[function]
    if function in on_stack:
        return None
    on_stack.add(function)
    result = None
    if function not in timing_functions:
        for hit in hits_by_function.get(function, []):
            if not belongs_to_is(binary, hit):
                result = (hit, [])
                break
    if result is None:
        for _, callee in calls_out.get(function, []):
            found = reaches_hook(
                callee,
                timing_functions,
                hits_by_function,
                calls_out,
                binary,
                memo,
                on_stack,
            )
            if found is not None:
                hit, chain = found
                result = (hit, [callee] + chain)
                break
    on_stack.discard(function)
    memo[function] = result
    return result


def through_pointers(
    binary, timing_functions, already_reported, hits_by_function, calls_out,
    takes_address,
):
    """The chains by which a `rcad_fuzz_timing` function reaches a hook
    outside `SlowRun::is` once taking a function's address counts as
    calling it; the functions whose address that reach takes; and the
    functions that reach a hook whose address only code outside that reach
    takes.

    A call through a computed pointer can only land on a function whose
    address some code took. Counting every address the reach takes as a
    call from the function that took it follows each computed call in the
    reach to everything it could land on, unless the pointer came into the
    reach from outside it: a hook or callback another part of the program
    stored in memory the reach reads. The third result names every such
    function that reaches a hook, so that assumption is checked per binary
    rather than taken on trust.

    A timing function in `already_reported` reached a hook by direct calls
    alone, and `main` already prints its chain.
    """
    edges = {}
    for outgoing in (calls_out, takes_address):
        for function, outs in outgoing.items():
            edges.setdefault(function, set()).update(
                callee for _, callee in outs
            )
    called = {
        (function, callee) for function, outs in calls_out.items()
        for _, callee in outs
    }
    reach = reachable_from(timing_functions, {
        function: [(None, callee) for callee in callees]
        for function, callees in edges.items()
    })
    taken_in_reach = {
        taken for function in reach
        for _, taken in takes_address.get(function, [])
    }

    def first_offending_hit(function):
        if function in timing_functions:
            return None
        hits = hits_by_function.get(function, [])
        if function in reach:
            hits = [hit for hit in hits if not belongs_to_is(binary, hit)]
        return hits[0] if hits else None

    hooked = {
        function for function in hits_by_function
        if first_offending_hit(function) is not None
    }
    callers = {}
    for function, callees in edges.items():
        for callee in callees:
            callers.setdefault(callee, set()).add(function)
    reaches_a_hook = set(hooked)
    pending = list(hooked)
    while pending:
        function = pending.pop()
        for caller in callers.get(function, ()):
            if caller not in reaches_a_hook:
                reaches_a_hook.add(caller)
                pending.append(caller)

    offenders = []
    for start in sorted(timing_functions - already_reported):
        path = shortest_path_to_a_hook(start, edges, reaches_a_hook, hooked)
        if path is None:
            continue
        marked = [
            callee if (caller, callee) in called else "&" + callee
            for caller, callee in zip(path, path[1:])
        ]
        offenders.append((start, marked, first_offending_hit(path[-1])))

    taken_anywhere = {
        taken for outs in takes_address.values() for _, taken in outs
    }
    hooked_elsewhere = sorted(
        (taken_anywhere - taken_in_reach - reach) & reaches_a_hook
    )
    return offenders, taken_in_reach, hooked_elsewhere


def shortest_path_to_a_hook(start, edges, reaches_a_hook, hooked):
    """The shortest list of functions from `start` to one in `hooked`,
    through `reaches_a_hook` only, `start` first; `None` when there is
    none."""
    parent = {start: None}
    queue = [start]
    for function in queue:
        for callee in sorted(edges.get(function, ())):
            if callee in parent or callee not in reaches_a_hook:
                continue
            parent[callee] = function
            if callee in hooked:
                path = [callee]
                while parent[path[-1]] is not None:
                    path.append(parent[path[-1]])
                return path[::-1]
            queue.append(callee)
    return None


def addr2line_chain(binary, address):
    text = run(["addr2line", "-i", "-f", "-C", "-e", binary, f"0x{address:x}"])
    return text.splitlines()


def print_hit(binary, hit, why):
    print(f"  {hit['address']:x}  {hit['mnemonic']} -> {hit['target']}  "
          f"via {hit['kind']}  {why}")
    print(f"    in {demangle(hit['function'])}")
    for frame in paired_chain(binary, hit["address"]):
        print(f"      {frame}")


def print_chain_offender(function, chain, hit):
    print(f"  {hit['address']:x}  {hit['mnemonic']} -> {hit['target']}  "
          f"via {hit['kind']}  reaches a hook outside SlowRun::is through:")
    names = [function] + chain + list(hit["target"])
    print(f"    {' -> '.join(demangle_step(name) for name in names)}")


def demangle_step(name):
    """`demangle`, keeping the `&` a chain puts in front of a function whose
    address was taken rather than called."""
    if name.startswith("&"):
        return "&" + demangle(name[1:])
    return demangle(name)


def paired_chain(binary, address):
    lines = addr2line_chain(binary, address)
    return [
        f"{lines[i + 1]}  ({lines[i]})" for i in range(0, len(lines) - 1, 2)
    ]


def split_operands(text):
    operands, depth, current = [], 0, ""
    for char in text:
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        if char == "," and depth == 0:
            operands.append(current.strip())
            current = ""
        else:
            current += char
    if current.strip():
        operands.append(current.strip())
    return operands


def base_register(operand):
    """The base register of an AT&T memory operand, e.g. `rax` out of
    `(%rax,%r12,4)` or `8(%rax)` — the first register AT&T syntax always
    places right after the `(`. `None` for an operand with no parentheses at
    all, such as a plain register or an immediate."""
    match = re.search(r"\(%(\w+)", operand)
    return REG64.get(match.group(1)) if match else None


def demangle(name):
    text = run(["c++filt", name]).strip()
    return text if text else name


def run(argv):
    return subprocess.run(argv, check=True, capture_output=True, text=True).stdout


if __name__ == "__main__":
    sys.exit(main())
