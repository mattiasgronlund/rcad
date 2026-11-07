"""Write block 1 with two or more of the nine differences adx to cdz zero.

With no arguments, write dz_<names>.g for every set of two or more whose y is
not zero by its shape, and print the names. With arguments, each a set
written as adx_bdx, write that set alone. Beside each, write the controls
c_dz_<names>_6u.g, where it fails there, and c_dz_<names>_5u.g, the script at
B = 6u and 5u, and,
where a minor rounds a double less a zero, dzq_<names>.g: the script with
its q hypotheses taken out and only its |q| goals left.
"""
import itertools
import sys

DIFFERENCES = ["adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz"]
# The six products: name, the two differences it multiplies, the minor it is in.
PRODUCT = {
    1: ("bdxcdy", "bdx", "cdy", 1),
    2: ("cdxbdy", "cdx", "bdy", 1),
    3: ("cdxady", "cdx", "ady", 2),
    4: ("adxcdy", "adx", "cdy", 2),
    5: ("adxbdy", "adx", "bdy", 3),
    6: ("bdxady", "bdx", "ady", 3),
}
MINOR = {1: "first", 2: "second", 3: "third"}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}
# The z difference each term multiplies its minor by.
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"


def main():
    if sys.argv[1:]:
        sets = [arg.split("_") for arg in sys.argv[1:]]
    else:
        sets = [
            list(zero)
            for n in range(2, 10)
            for zero in itertools.combinations(DIFFERENCES, n)
            if zero_terms(set(zero)) != {1, 2, 3}
        ]
    for zero in sets:
        name = "dz_" + "_".join(zero)
        src = make(set(zero))
        write(f"{name}.g", src)
        if longest_path(set(zero)) > 5:
            write(f"c_{name}_6u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 6b-53;"))
        write(f"c_{name}_5u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 5b-53;"))
        if "  /\\ q" in src:
            write(f"dzq_{'_'.join(zero)}.g", q_goals_alone(src))
        print(name)


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def longest_path(zero):
    """The roundings on the longest path through a term of y that is not zero.
    A set with 5 proves at 6u, so its 6u script is no control."""
    dead = dead_products(zero)
    terms = zero_terms(zero)
    longest = 0
    for k, (_, _, _, m) in PRODUCT.items():
        if k in dead or m in terms:
            continue
        other = 2 * m if k == 2 * m - 1 else 2 * m - 1
        n = 3 + 1 + (other not in dead) + 1
        if m in (1, 2) and (3 - m) not in terms:
            n += 1
        longest = max(longest, n)
    return longest


def dead_products(zero):
    return {k for k, (_, f, g, _) in PRODUCT.items() if f in zero or g in zero}


def zero_terms(zero):
    """The terms that are zero by their shape: a z difference zero, or both
    products of the minor zero."""
    dead = dead_products(zero)
    return {m for m in (1, 2, 3) if Z_OF[m] in zero or {2 * m - 1, 2 * m} <= dead}


def make(zero):
    names = [d for d in DIFFERENCES if d in zero]
    dead = dead_products(zero)
    terms = zero_terms(zero)
    # A minor that is not zero but has one product zero rounds a double less
    # a zero, or a zero less a double.
    half = [m for m in (1, 2, 3) if m not in terms and len({2 * m - 1, 2 * m} & dead) == 1]
    # Its q rounds a double plus a zero, whether its term is zero or not.
    half_q = [m for m in (1, 2, 3) if len({2 * m - 1, 2 * m} & dead) == 1]
    src = open("block1.g").read()
    src = sub(
        src,
        "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
        "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
        f"# Block 1 where {' and '.join(names)} are zero. A zero rounds to itself,\n"
        "# and so does a product with a zero factor.\n",
    )
    for axis in "xyz":
        points = [p for p in "abc" if f"{p}d{axis}" in zero]
        if points:
            src = sub(src, f"d{axis} = rnd(d{axis}_);", f"d{axis} = {points[0]}{axis};")
            for p in points[1:]:
                src = sub(src, f"{p}{axis} = rnd({p}{axis}_);", f"{p}{axis} = {points[0]}{axis};")
    for name in names:
        exact = name[0].upper() + name[1:]
        src = sub(src, f"{name} = rnd({exact});", f"{name} = {exact};")
        src = drop_def(src, f"r{name} = {name} / {exact};")
        src = drop_conj(src, f"|{name}| >= 1b-1074")
        src = sub(src, f"{name} / {exact} -> 1 + ({name} - {exact}) / {exact};\n", "")
        if name in Z_OF.values():
            src = drop_conj(src, f"{name} / |{name}| in [-1, 1]")
    for k in sorted(dead):
        prod, f, g, _ = PRODUCT[k]
        factors = f"{f} * {g}"
        src = sub(src, f"{prod} = rnd({factors});", f"{prod} = {factors};")
        src = drop_def(src, f"rp{k} = {prod} / ({factors});")
        src = drop_conj(src, f"|{prod}| >= 1b-1022")
        src = sub(src, f"  /\\ {prod} -/ {factors} {BOUND}", "")
        src = drop_conj(src, f"{prod} / |{prod}| in [-1, 1]")
        src = sub(src, f"{prod} / ({factors}) -> 1 + ({prod} - {factors}) / ({factors});\n", "")
    for m in sorted(terms):
        minor = MINOR[m]
        if {2 * m - 1, 2 * m} <= dead:
            src = sub(src, f"{minor} = rnd(M{m});", f"{minor} = M{m};")
            src = sub(src, f"q{m} = rnd({NOTE[m]});", f"q{m} = {NOTE[m]};")
        src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
        src = sub(src, f"r{m} = rnd(R{m});", f"r{m} = R{m};")
        src = drop_def(src, f"rm{m} = {minor} / M{m};")
        src = drop_def(src, f"rt{m} = t{m} / U{m};")
        src = drop_conj(src, f"|{minor}| >= 1b-1074")
        src = drop_conj(src, f"|t{m}| >= 1b-1022")
        src = drop_conj(src, f"|r{m}| >= 1b-1022")
        src = sub(src, f"  /\\ t{m} -/ U{m} {BOUND}", "")
        src = sub(src, f"  /\\ r{m} -/ R{m} {BOUND}", "")
        src = sub(src, f"{minor} / M{m} -> 1 + ({minor} - M{m}) / M{m};\n", "")
        src = sub(src, f"t{m} / U{m} -> 1 + (t{m} - U{m}) / U{m};\n", "")
    alive = [m for m in (1, 2, 3) if m not in terms]
    src = sub(
        src,
        "  # The permanent's own three products are normal too. The comment above\n"
        "  # stays_accurate argues it from the nine.\n",
        f"  # Each of {', '.join(f'r{m}' for m in alive)} is normal, because its t is and its minor is\n"
        f"  # at most its q. Each of {', '.join(f'r{m}' for m in sorted(terms))} is zero.\n",
    )
    src = sub(
        src,
        "  # No factor of the nine products is zero. For an f64 that is the same as\n"
        "  # being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {' and '.join(names)}. For an\n"
        "  # f64 that is the same as being at least the smallest subnormal.\n",
    )
    src = sub(
        src,
        "  # §12.2's first condition: product_is_normal on the nine products.\n",
        "  # §12.2's first condition: product_is_normal on the nine products. Those\n"
        "  # on a zero pass, and say nothing.\n",
    )
    tail = ""
    if {1, 2} <= terms:
        # Both of t1 and t2 are zero, so s12 is too and nothing rounds; and
        # both of r1 and r2, so their sum in P rounds nothing either.
        src = sub(src, "s12 = rnd(S);", "s12 = S;")
        src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P = rnd((r1 + r2) + r3);")
        src = sub(src, "rs = s12 / S;\n", "")
        src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
        src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
    elif terms & {1, 2}:
        (m,) = terms & {1, 2}
        other = 3 - m
        tail += f"\n# t{m} is zero, so t1 + t2 is the double t{other} and rounds to itself.\nt1 + t2 -> t{other};\n"
    if half_q:
        last = f"  /\\ eb -/ B * P {BOUND}"
        qs = " and ".join(f"q{m}" for m in half_q)
        src = sub(
            src,
            last,
            last + f"  # {qs} each round a double plus a zero, so each is at least 2^-1022\n"
            "  # as that double is, and lemma.g gives the same of it. The goals\n"
            "  # check the first part.\n"
            + "".join(f"  /\\ q{m} -/ ({NOTE[m]}) {BOUND}" for m in half_q),
        )
        goal = "  (y - Ex) / P6 in ? /\\ P -/ Phat in ?\n"
        src = sub(src, goal, goal + "".join(f"  /\\ |q{m}| >= 1b-1022\n" for m in half_q))
    if half:
        hints = []
        for m in half:
            left, right = PRODUCT[2 * m - 1][0], PRODUCT[2 * m][0]
            gone = 2 * m - 1 if 2 * m - 1 in dead else 2 * m
            hints.append(f"{left} - {right} -> -{right};\n" if gone == 2 * m - 1 else f"{left} - {right} -> {left};\n")
        mins = " and ".join(MINOR[m] for m in half)
        tail += (
            f"\n# {mins} each round a double less a zero, or a zero less a double,\n"
            "# and so round to themselves.\n" + "".join(hints)
        )
    if {1, 2} <= terms:
        # Gappa finds no REL(P, Phat) here: two of the three parts it would
        # average are zero. So ask for the ratio as an expression, and carry
        # r1 and r2 as amounts against Phat, as make_two_zero.py does.
        src = sub(src, "/\\ P -/ Phat in ?\n", "/\\ (P - Phat) / Phat in ?\n")
        src = sub(
            src,
            "Phat = (|bdxcdy| + |cdxbdy|) * |adz|\n"
            "     + (|cdxady| + |adxcdy|) * |bdz|\n"
            "     + (|adxbdy| + |bdxady|) * |cdz|;\n",
            "Phat1 = (|bdxcdy| + |cdxbdy|) * |adz|;\n"
            "Phat2 = (|cdxady| + |adxcdy|) * |bdz|;\n"
            "Phat3 = (|adxbdy| + |bdxady|) * |cdz|;\n"
            "Phat = Phat1 + Phat2 + Phat3;\n",
        )
        x = "((r1 + r2) + r3)"
        tail += (
            "\n# r1 and r2 are zero, and so are Phat1 and Phat2. Carry each as an\n"
            "# amount against Phat, which Phat3 keeps away from zero.\n"
            f"(P - Phat) / Phat -> ((P - {x}) / {x}) * (1 + ({x} - Phat) / Phat) + ({x} - Phat) / Phat;\n"
            f"({x} - Phat) / Phat -> (r1 - Phat1) / Phat + (r2 - Phat2) / Phat"
            " + ((r3 - Phat3) / Phat3) * (Phat3 / Phat);\n"
            "Phat3 / Phat -> 1 / (1 + (Phat1 + Phat2) / Phat3);\n"
        )
    # A term whose product is zero carries no error: it is zero in y and in
    # Ex alike, and stays in P6 only as the zero it is.
    for k in sorted(dead | {k for m in terms for k in (2 * m - 1, 2 * m)}):
        src = sub(src, line_starting(src, f"W{k} = ") + "\n", "")
        line = line_starting(src, "", f"e{k} = ")
        part = line[line.index(f"e{k} = "):]
        src = drop_def(src, part[: part.index(";") + 1])
        src = sub(src, f"(c{k} + e{k}) - c{k} -> e{k};\n", "")
        src = sub(src, line_starting(src, f"e{k} / c{k} -> ") + "\n", "")
        src = sub(src, f"(c{k} + e{k})", f"c{k}")
    return src + tail


def q_goals_alone(src):
    """The script with its q hypotheses out and its |q| goals alone left."""
    lines = src.split("\n")
    lines = [l for l in lines if not l.startswith("  /\\ q")]
    start = lines.index("->")
    end = lines.index("}", start)
    goals = [l for l in lines[start + 1 : end] if l.startswith("  /\\ |q")]
    goals[0] = "  " + goals[0][len("  /\\ ") :]
    return "\n".join(lines[: start + 1] + goals + lines[end:])


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def drop_conj(src, item):
    """Take one conjunct out of a hypothesis line, and the line with it when
    nothing is left on it."""
    if src.count("\n  /\\ " + item + "\n") == 1:
        return src.replace("\n  /\\ " + item + "\n", "\n")
    if src.count(item + " /\\ ") == 1:
        return src.replace(item + " /\\ ", "")
    if src.count(" /\\ " + item + "\n") == 1:
        return src.replace(" /\\ " + item + "\n", "\n")
    return sub(src, "  /\\ " + item + "\n", "")


def drop_def(src, item):
    """Take one definition out of a line of definitions."""
    if src.count(item + " ") == 1:
        return src.replace(item + " ", "")
    if src.count(" " + item + "\n") == 1:
        return src.replace(" " + item + "\n", "\n")
    return sub(src, item + "\n", "")


def line_starting(src, start, containing=""):
    (line,) = [l for l in src.split("\n") if l.startswith(start) and containing in l]
    return line


main()
