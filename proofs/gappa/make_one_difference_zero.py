"""Write block 1 with one of the nine differences adx to cdz zero, as zd_<name>.g."""
import sys

# The six products: name, factors, the minor they are in.
PRODUCT = {
    1: ("bdxcdy", "bdx * cdy", 1),
    2: ("cdxbdy", "cdx * bdy", 1),
    3: ("cdxady", "cdx * ady", 2),
    4: ("adxcdy", "adx * cdy", 2),
    5: ("adxbdy", "adx * bdy", 3),
    6: ("bdxady", "bdx * ady", 3),
}
MINOR = {1: "first", 2: "second", 3: "third"}
# Each minor's two products, and what its q rounds.
MINUS = {1: ("bdxcdy", "cdxbdy"), 2: ("cdxady", "adxcdy"), 3: ("adxbdy", "bdxady")}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}
# The z difference each term multiplies its minor by.
TERM_OF = {"adz": 1, "bdz": 2, "cdz": 3}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def drop_conj(src, item):
    """Take one conjunct out of a hypothesis line."""
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


def make(name):
    point, axis = name[0], name[2]
    exact = name[0].upper() + name[1:]
    src = open("block1.g").read()
    src = sub(
        src,
        "# Orient3d::on_floats and ORIENT3D_ERROR_BOUND,\n"
        "# crates/rcad-predicates/src/orient3d.rs at d9f931e8.\n",
        f"# Block 1 where {name} is zero: d{axis} is the same double as {point}{axis}. A\n"
        "# zero rounds to itself, and so does a product with a zero factor.\n",
    )
    src = sub(src, f"d{axis} = rnd(d{axis}_);", f"d{axis} = {point}{axis};")
    src = sub(src, f"{name} = rnd({exact});", f"{name} = {exact};")
    src = drop_def(src, f"r{name} = {name} / {exact};")
    src = drop_conj(src, f"|{name}| >= 1b-1074")
    src = sub(src, f"{name} / {exact} -> 1 + ({name} - {exact}) / {exact};\n", "")
    if name in TERM_OF:
        m = TERM_OF[name]
        gone = [2 * m - 1, 2 * m]
        minor = MINOR[m]
        src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
        src = sub(src, f"r{m} = rnd(R{m});", f"r{m} = R{m};")
        src = drop_def(src, f"rm{m} = {minor} / M{m};")
        src = drop_def(src, f"rt{m} = t{m} / U{m};")
        src = drop_conj(src, f"|{minor}| >= 1b-1074")
        src = drop_conj(src, f"|t{m}| >= 1b-1022")
        src = drop_conj(src, f"|r{m}| >= 1b-1022")
        src = sub(src, f"  /\\ t{m} -/ U{m} {BOUND}", "")
        src = sub(src, f"  /\\ r{m} -/ R{m} {BOUND}", "")
        src = drop_conj(src, f"{name} / |{name}| in [-1, 1]")
        src = sub(src, f"{minor} / M{m} -> 1 + ({minor} - M{m}) / M{m};\n", "")
        src = sub(src, f"t{m} / U{m} -> 1 + (t{m} - U{m}) / U{m};\n", "")
        others = [MINOR[k] for k in (1, 2, 3) if k != m]
        src = sub(
            src,
            "  # The permanent's own three products are normal too. The comment above\n"
            "  # stays_accurate argues it from the nine.\n",
            f"  # The other two of the permanent's products are normal because |{others[0]}|\n"
            f"  # and |{others[1]}| are at most their q. r{m} is zero.\n",
        )
        src = sub(
            src,
            "  # §12.2's first condition: product_is_normal on the nine products.\n",
            "  # §12.2's first condition: product_is_normal on the nine products. The\n"
            f"  # check on {name} and {minor} passes on the zero, and says nothing.\n",
        )
        if m == 3:
            tail = ""
        else:
            other = 3 - m
            tail = (
                f"\n# t{m} is zero, so t1 + t2 is the double t{other} and rounds to itself.\n"
                f"t1 + t2 -> t{other};\n"
            )
    else:
        gone = [k for k, (_, f, _) in PRODUCT.items() if name in f]
        for k in gone:
            prod, factors, _ = PRODUCT[k]
            src = sub(src, f"{prod} = rnd({factors});", f"{prod} = {factors};")
            src = drop_def(src, f"rp{k} = {prod} / ({factors});")
            src = drop_conj(src, f"|{prod}| >= 1b-1022")
            src = sub(src, f"  /\\ {prod} -/ {factors} {BOUND}", "")
            src = drop_conj(src, f"{prod} / |{prod}| in [-1, 1]")
            src = sub(src, f"{prod} / ({factors}) -> 1 + ({prod} - {factors}) / ({factors});\n", "")
        names = " and ".join(PRODUCT[k][0] for k in gone)
        src = sub(
            src,
            "  # §12.2's first condition: product_is_normal on the nine products.\n",
            "  # §12.2's first condition: product_is_normal on the nine products. The\n"
            f"  # checks on {names} pass on the zero, and say nothing.\n",
        )
        minors = [PRODUCT[k][2] for k in gone]
        hints = []
        for k in gone:
            prod, _, m = PRODUCT[k]
            left, right = MINUS[m]
            hints.append(f"{left} - {right} -> -{right};\n" if prod == left else f"{left} - {right} -> {left};\n")
        qs = [f"q{m}" for m in minors]
        last = f"  /\\ eb -/ B * P {BOUND}"
        src = sub(
            src,
            last,
            last + f"  # {qs[0]} and {qs[1]} each round a double plus a zero, so each is at least\n"
            "  # 2^-1022 as that double is, and lemma.g gives the same of them. The\n"
            "  # goals check the first part.\n"
            + "".join(f"  /\\ q{m} -/ ({NOTE[m]}) {BOUND}" for m in minors),
        )
        goal = "  (y - Ex) / P6 in ? /\\ P -/ Phat in ?\n"
        src = sub(src, goal, goal + "".join(f"  /\\ |q{m}| >= 1b-1022\n" for m in minors))
        mins = " and ".join(MINOR[m] for m in minors)
        tail = (
            f"\n# {names} are zero, so {mins} each round a double less a zero, or a\n"
            "# zero less a double, and so round to themselves.\n" + "".join(hints)
        )
    src = sub(
        src,
        "  # No factor of the nine products is zero. For an f64 that is the same as\n"
        "  # being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {name}. For an f64 that is\n"
        "  # the same as being at least the smallest subnormal.\n",
    )
    # A term whose product is zero carries no error: it is zero in y and in
    # Ex alike, and stays in P6 only as the zero it is.
    for k in gone:
        src = sub(src, line_starting(src, f"W{k} = ") + "\n", "")
        line = line_starting(src, "", f"e{k} = ")
        part = line[line.index(f"e{k} = "):]
        src = drop_def(src, part[: part.index(";") + 1])
        src = sub(src, f"(c{k} + e{k}) - c{k} -> e{k};\n", "")
        src = sub(src, line_starting(src, f"e{k} / c{k} -> ") + "\n", "")
        src = sub(src, f"(c{k} + e{k})", f"c{k}")
    return src + tail


def line_starting(src, start, containing=""):
    (line,) = [l for l in src.split("\n") if l.startswith(start) and containing in l]
    return line


for name in sys.argv[1:]:
    open(f"zd_{name}.g", "w").write(make(name))
