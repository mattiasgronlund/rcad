"""Write block 1 with one difference zero and one minor cancelling to zero.

Run beside the dz_<difference>.g that make_differences_zero.py writes for one
difference. Each argument is a pair such as adx_first. Where a product of the
minor has the difference as a factor, the minor is that product's partner and
cannot be zero, and the script written is dmi_<pair>.g, which asks only that
the minor is at least 2^-1022. Otherwise it is dm_<pair>.g, with the controls
c_dm_<pair>_6u.g and c_dm_<pair>_5u.g, and, where a minor rounds a double less
a zero, dmq_<pair>.g: the script with its q hypotheses taken out and only its
|q| goals left.
"""
import sys

PRODUCT = {
    1: ("bdxcdy", "bdx", "cdy"),
    2: ("cdxbdy", "cdx", "bdy"),
    3: ("cdxady", "cdx", "ady"),
    4: ("adxcdy", "adx", "cdy"),
    5: ("adxbdy", "adx", "bdy"),
    6: ("bdxady", "bdx", "ady"),
}
MINOR = {"first": 1, "second": 2, "third": 3}
Z_OF = {1: "adz", 2: "bdz", 3: "cdz"}
BOUND = "in [-4503599627370497b-105, 4503599627370497b-105]\n"
# These four prove at 6u, so their 6u script is no control (464.26).
PROVES_AT_6U = {"adx_first", "ady_first", "bdx_second", "bdy_second"}


def main():
    for pair in sys.argv[1:]:
        difference, minor = pair.split("_")
        m = MINOR[minor]
        src = open(f"dz_{difference}.g").read()
        if difference in PRODUCT[2 * m - 1][1:] + PRODUCT[2 * m][1:]:
            write(f"dmi_{pair}.g", cannot_cancel(src, difference, minor, m))
            write(f"c_dmi_{pair}_nonormal.g", cannot_cancel(src, difference, minor, m, control=True))
        else:
            src = cancel(src, difference, minor, m)
            write(f"dm_{pair}.g", src)
            if pair not in PROVES_AT_6U:
                write(f"c_dm_{pair}_6u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 6b-53;"))
            write(f"c_dm_{pair}_5u.g", sub(src, "B = 7b-53 + 7b-103;", "B = 5b-53;"))
            if "  /\\ q" in src:
                write(f"dmq_{pair}.g", q_goals_alone(src))
        print(pair)


def write(path, text):
    with open(path, "w") as f:
        f.write(text)


def cannot_cancel(src, difference, minor, m, control=False):
    """dz_<difference>.g asking only that the minor is at least 2^-1022.

    One product of the minor is zero, so the minor rounds the other one, a
    double that product_is_normal has checked; the hint that says so is in
    dz_<difference>.g already. The control drops that check, and must fail.
    """
    src = drop_conj(src, f"|{minor}| >= 1b-1074")
    lines = src.split("\n")
    start = lines.index("->")
    end = lines.index("}", start)
    lines = lines[: start + 1] + [f"  |{minor}| >= 1b-1022"] + lines[end:]
    src = "\n".join(lines)
    if control:
        (alive,) = [PRODUCT[k][0] for k in (2 * m - 1, 2 * m) if difference not in PRODUCT[k][1:]]
        src = drop_conj(src, f"|{alive}| >= 1b-1022")
    return src


def cancel(src, difference, minor, m):
    """dz_<difference>.g with the minor's two products the same double, as
    464.7 writes a minor at zero."""
    left, f, g = PRODUCT[2 * m]
    src = sub(src, f"{left} = rnd({f} * {g});", f"{left} = {PRODUCT[2 * m - 1][0]};")
    src = sub(src, f"{minor} = rnd(M{m});", f"{minor} = M{m};")
    src = sub(
        src,
        f"# Block 1 where {difference} are zero. A zero rounds to itself,\n"
        "# and so does a product with a zero factor.\n",
        f"# Block 1 where {difference} is zero and {minor} cancels to zero: its two\n"
        "# products are one double. A zero rounds to itself, and so does a\n"
        "# product with a zero factor.\n",
    )
    src = sub(
        src,
        f"  # No factor of the nine products is zero but {difference}. For an\n"
        "  # f64 that is the same as being at least the smallest subnormal.\n",
        f"  # No factor of the nine products is zero but {difference} and {minor}. For\n"
        "  # an f64 that is the same as being at least the smallest subnormal.\n",
    )
    if Z_OF[m] == difference:
        # The term is zero by its shape already, in y and in Ex alike, and
        # dz_<difference>.g carries it so.
        return src
    src = sub(src, f"t{m} = rnd(U{m});", f"t{m} = U{m};")
    src = sub(src, f"rm{m} = {minor} / M{m};", f"rm{m} = 1;")
    src = sub(src, f"rt{m} = t{m} / U{m};", f"rt{m} = 1;")
    src = drop_conj(src, f"|{minor}| >= 1b-1074")
    src = drop_conj(src, f"|t{m}| >= 1b-1022")
    src = drop_conj(src, f"|r{m}| >= 1b-1022")
    start = src.index("  # Each of r")
    end = src.index("\n", src.index("\n", start) + 1) + 1
    zeroed = [k for k, z in Z_OF.items() if z == difference]
    said = f"r{zeroed[0]} is zero, and nothing" if zeroed else "Nothing"
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
    if Z_OF[other] == difference:
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
    """t{m} cancels and t{other} is zero by its shape, as in dz_adz_bdz.

    Gappa then finds no REL(P, Phat): two of the three parts it would average
    may be off the normals. So ask for the ratio as an expression, and carry
    r1 and r2 as amounts against Phat, which Phat3 keeps away from zero, as
    make_two_zero.py does. r{other} is exactly zero, and r{m} may be anything
    from zero up, so the sum P12 rounds is r{m} and is kept as the code rounds
    it. Phat{other} is zero, so no hint divides by it.
    """
    note = {1: "|bdxcdy| + |cdxbdy|", 2: "|cdxady| + |adxcdy|"}[m]
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
