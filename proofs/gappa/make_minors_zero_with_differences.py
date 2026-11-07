"""Write block 1 with one or more differences zero and one or two minors
cancelling to zero.

Run beside the dz_<differences>.g that make_differences_zero.py writes. Each
argument is a set of differences and minors such as adx_bdz_first or
adz_first_second: the differences in the order DIFFERENCES gives them, then
the minors. With no arguments, write every set past 464.26's one difference
with one minor that is a branch, and print the names, one line each.

A branch, where every minor named has both its products free of the zero
differences, is dm_<set>.g, with the controls c_dm_<set>_6u.g and
c_dm_<set>_5u.g, and, where a minor rounds a double less a zero,
dmq_<set>.g: the script with its q hypotheses taken out and only its |q| goals
left. Written with --impossible, each argument is instead a set of
differences and one minor with one product free of them and one not, and the
script is dmi_<set>.g, which asks only that the minor is at least 2^-1022,
with its control c_dmi_<set>_nonormal.g, which drops the check on the free
product. The 28 sets in PROVES_AT_6U prove at 6u, so their 6u script is no
control (464.43).
"""
import itertools
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))

DIFFERENCES = ["adx", "ady", "adz", "bdx", "bdy", "bdz", "cdx", "cdy", "cdz"]
PRODUCT = {
    1: ("bdxcdy", "bdx", "cdy"),
    2: ("cdxbdy", "cdx", "bdy"),
    3: ("cdxady", "cdx", "ady"),
    4: ("adxcdy", "adx", "cdy"),
    5: ("adxbdy", "adx", "bdy"),
    6: ("bdxady", "bdx", "ady"),
}
MINOR = {"first": 1, "second": 2, "third": 3}
NAME = {1: "first", 2: "second", 3: "third"}
NOTE = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|", 3: "|adxbdy| + |bdxady|"}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"
PROVES_AT_6U = {
    "adx_adz_first",
    "adx_bdz_first",
    "adx_cdz_first",
    "ady_adz_first",
    "ady_bdz_first",
    "ady_cdz_first",
    "adz_bdx_second",
    "adz_bdy_second",
    "adz_cdx_third",
    "adz_cdy_third",
    "bdx_bdz_second",
    "bdx_cdz_second",
    "bdy_bdz_second",
    "bdy_cdz_second",
    "bdz_cdx_third",
    "bdz_cdy_third",
    "adx_adz_bdz_first",
    "adx_adz_cdz_first",
    "ady_adz_bdz_first",
    "ady_adz_cdz_first",
    "adz_bdx_bdz_second",
    "adz_bdy_bdz_second",
    "adz_cdx_cdz_third",
    "adz_cdy_cdz_third",
    "bdx_bdz_cdz_second",
    "bdy_bdz_cdz_second",
    "bdz_cdx_cdz_third",
    "bdz_cdy_cdz_third",
}


def main():
    args = sys.argv[1:]
    impossible = args[:1] == ["--impossible"]
    if impossible:
        args = args[1:]
    if args:
        sets = [split(arg) for arg in args]
    else:
        sets = every_impossible() if impossible else every_branch()
    for zero, minors in sets:
        name = "_".join(zero + [NAME[m] for m in minors])
        src = open(f"dz_{'_'.join(zero)}.g").read()
        if impossible:
            (m,) = minors
            write(f"dmi_{name}.g", cannot_cancel(src, set(zero), m))
            write(f"c_dmi_{name}_nonormal.g",cannot_cancel(src, set(zero), m, control=True))
        else:
            src = cancel(src, zero, minors)
            write(f"dm_{name}.g", src)
            if name not in PROVES_AT_6U:
                write(f"c_dm_{name}_6u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 6b-53;"))
            write(f"c_dm_{name}_5u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 5b-53;"))
            if "  /\\ q" in src:
                write(f"dmq_{name}.g", q_goals_alone(src))
        print(name)


def split(arg):
    words = arg.split("_")
    return [w for w in words if w in DIFFERENCES], [MINOR[w] for w in words if w in MINOR]


def dead_products(zero):
    return {k for k, (_, f, g) in PRODUCT.items() if f in zero or g in zero}


def shape_zero_terms(zero):
    dead = dead_products(zero)
    return {m for m in (1, 2, 3) if Z_OF[m] in zero or {2 * m - 1, 2 * m} <= dead}


def every_branch():
    """Every set of differences and minors past 464.26 that is a branch: y not
    zero by the differences' shape, each minor named with both its products
    free of them, and some term left that is neither zero by its shape nor
    cancelled."""
    out = []
    for n in range(1, 10):
        for zero in itertools.combinations(DIFFERENCES, n):
            dead = dead_products(set(zero))
            shape = shape_zero_terms(set(zero))
            if shape == {1, 2, 3}:
                continue
            for c in (1, 2):
                if n == 1 and c == 1:
                    continue
                for minors in itertools.combinations((1, 2, 3), c):
                    if any({2 * m - 1, 2 * m} & dead for m in minors):
                        continue
                    if shape | set(minors) == {1, 2, 3}:
                        continue
                    out.append((list(zero), list(minors)))
    return out


def every_impossible():
    """Every set of two or more differences whose y is not zero by its shape,
    with each minor that has one product free of them and one not: that
    minor cannot cancel."""
    out = []
    for n in range(2, 10):
        for zero in itertools.combinations(DIFFERENCES, n):
            dead = dead_products(set(zero))
            if shape_zero_terms(set(zero)) == {1, 2, 3}:
                continue
            for m in (1, 2, 3):
                if len({2 * m - 1, 2 * m} & dead) == 1:
                    out.append((list(zero), [m]))
    return out


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def cannot_cancel(src, zero, m, control=False):
    """dz_<differences>.g asking only that the minor is at least 2^-1022.

    One product of the minor is zero, so the minor rounds the other one, a
    double that product_is_normal has checked; the hint that says so is in
    dz_<differences>.g already. The control drops that check, and must fail.
    """
    minor = NAME[m]
    left, right = PRODUCT[2 * m - 1][0], PRODUCT[2 * m][0]
    gone = 2 * m - 1 if set(PRODUCT[2 * m - 1][1:]) & zero else 2 * m
    hint = f"{left} - {right} -> -{right};\n" if gone == 2 * m - 1 else f"{left} - {right} -> {left};\n"
    if m in shape_zero_terms(zero):
        # dz_<differences>.g asks nothing of a minor whose term its z
        # difference zeroes, and gives no hint for it.
        assert hint not in src
        src += f"\n# {minor} rounds a double less a zero, or a zero less a double,\n# and so rounds to itself.\n" + hint
    else:
        assert hint in src
        src = drop_conj(src, f"|{minor}| >= 1b-1074")
    lines = src.split("\n")
    start = lines.index("->")
    end = lines.index("}", start)
    lines = lines[: start + 1] + [f"  |{minor}| >= 1b-1022"] + lines[end:]
    src = "\n".join(lines)
    if control:
        (alive,) = [PRODUCT[k][0] for k in (2 * m - 1, 2 * m) if not set(PRODUCT[k][1:]) & zero]
        src = drop_conj(src, f"|{alive}| >= 1b-1022")
    return src


def cancel(src, zero, minors):
    """dz_<differences>.g with each named minor's two products the same
    double, as 464.7 writes a minor at zero."""
    shape = shape_zero_terms(set(zero))
    differences = " and ".join(zero)
    cancelled = " and ".join(NAME[m] for m in minors)
    src = sub(
        src,
        f"# Block 1 where {differences} are zero. A zero rounds to itself,\n"
        "# and so does a product with a zero factor.\n",
        f"# Block 1 where {differences} are zero and {cancelled} {'cancels' if len(minors) == 1 else 'cancel'} to zero:\n"
        "# the two products of each are one double. A zero rounds to itself, and\n"
        "# so does a product with a zero factor.\n",
    )
    src = sub(
        src,
        f"  # No factor of the nine products is zero but {differences}. For an\n"
        "  # f64 that is the same as being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {differences} and\n"
        f"  # {cancelled}. For an f64 that is the same as being at least the\n"
        "  # smallest subnormal.\n",
    )
    alive = [m for m in minors if m not in shape]
    for m in minors:
        minor = NAME[m]
        left, f, g = PRODUCT[2 * m]
        src = sub(src, f"{left} = rnd({f} * {g});", f"{left} = {PRODUCT[2 * m - 1][0]};")
        src = sub(src, f"{minor} = rnd(M{m});", f"{minor} = M{m};")
    # A term zero by its shape already is zero in y and in Ex alike, and
    # dz_<differences>.g carries it so: only its products and its minor change.
    assert len(alive) <= 1, (zero, minors)
    if not alive:
        return src
    (m,) = alive
    minor = NAME[m]
    src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
    src = sub(src, f"rm{m} = {minor} / M{m};", f"rm{m} = 1;")
    src = sub(src, f"rt{m} = t{m} / U{m};", f"rt{m} = 1;")
    src = drop_conj(src, f"|{minor}| >= 1b-1074")
    src = drop_conj(src, f"|t{m}| >= 1b-1022")
    src = drop_conj(src, f"|r{m}| >= 1b-1022")
    start = src.index("  # Each of r")
    end = src.index("\n", src.index("\n", start) + 1) + 1
    zeroed = sorted(shape)
    said = f"{' and '.join(f'r{k}' for k in zeroed)} {'is' if len(zeroed) == 1 else 'are'} zero, and nothing" if zeroed else "Nothing"
    src = sub(
        src,
        src[start:end],
        "  # Each r asked to be normal here is, because its t is and its minor is\n"
        f"  # at most its q. {said} checks r{m}, since {minor} is zero.\n",
    )
    src = sub(src, f"  /\\ t{m} -/ U{m} {BOUND}", "")
    src = sub(src, f"  /\\ r{m} -/ R{m} {BOUND}", "")
    src = sub(src, f"{minor} / M{m} -> 1 + ({minor} - M{m}) / M{m};\n", "")
    src = sub(src, f"t{m} / U{m} -> 1 + (t{m} - U{m}) / U{m};\n", "")
    if m == 3:
        return src
    other = 3 - m
    if other in shape:
        # t1 and t2 are both zero, so s12 is too and nothing rounds.
        src = sub(src, "s12 = rnd(S);", "s12 = S;")
        src = sub(src, "rs = s12 / S;\n", "")
        src = src.replace(" * rs - 1", " - 1")
        src = sub(src, "  # The first two terms do not cancel to zero.\n  /\\ |t1 + t2| >= 1b-1074\n", "")
        src = sub(src, "s12 / S -> 1 + (s12 - S) / S;\n", "")
        src = sub(src, f"\n# t{other} is zero, so t1 + t2 is the double t{m} and rounds to itself.\nt1 + t2 -> t{m};\n", "")
        return both_first_terms_zero(src, m, other)
    return src + f"\n# t{m} is zero, so t1 + t2 is the double t{other} and rounds to itself.\nt1 + t2 -> t{other};\n"


def both_first_terms_zero(src, m, other):
    """t{m} cancels and t{other} is zero by its shape, as 464.26's
    make_minor_zero_too.py writes it for one difference."""
    note = NOTE[m]
    src = sub(src, "/\\ P -/ Phat in ?\n", "/\\ (P - Phat) / Phat in ?\n")
    src = sub(src, "P = rnd(rnd(r1 + r2) + r3);", "P12 = rnd(r1 + r2);\nP = rnd(P12 + r3);")
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
    x = "(P12 + r3)"
    return src + (
        f"\n# r{m} may be below the normals and r{other} is zero, so their errors are\n"
        "# amounts and not fractions. Carry each as an amount against Phat, which\n"
        "# Phat3 keeps away from zero.\n"
        f"(P - Phat) / Phat -> ((P - {x}) / {x}) * (1 + ({x} - Phat) / Phat) + ({x} - Phat) / Phat;\n"
        f"({x} - Phat) / Phat -> (P12 - (r1 + r2)) / Phat + (r{other} - Phat{other}) / Phat"
        f" + (r{m} - R{m}) / Phat + ((q{m} - ({note})) / ({note})) * (Phat{m} / Phat)"
        " + ((r3 - Phat3) / Phat3) * (Phat3 / Phat);\n"
        f"Phat{m} / Phat -> 1 / (1 + (Phat{other} + Phat3) / Phat{m});\n"
        "Phat3 / Phat -> 1 / (1 + (Phat1 + Phat2) / Phat3);\n"
        "Phat3 -> r3 / (1 + (r3 - Phat3) / Phat3);\n"
        f"{x} -> P / (1 + (P - {x}) / {x});\n"
    )


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


main()
