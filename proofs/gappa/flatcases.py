"""Write flat.g, SideOfAFlatSurfaceThroughAPoint::on_floats and
SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND line for line, and its cases
and controls beside it:

- flatv.g, the claim on value, the double on_floats returns, not on y;
- flat_z<set>.g and flatv_z<set>.g for each set of the six products that are
  zero, which product_is_normal lets through: <set> names the products, 0 to 2
  the facing times the point and 3 to 5 the facing times the through point;
- each of those again with _c<set> for each set of its rounded sums that may
  cancel to zero, <set> naming the product each such sum adds;
- c_<name>_half.g, each of those at B = u / 2, which must fail;
- c_flat_nonormal.g, flat.g without product_is_normal, flat_noeb.g, flat.g
  without eb normal, which proves (464.13's 7), and c_flat_z012345_noeb.g, every product zero without eb
  normal.

Run as: python3 flatcases.py; it writes beside itself.
"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

TERMS = range(6)
SIGN = "+++---"
FACTORS = [("f0", "p0"), ("f1", "p1"), ("f2", "p2"), ("f0", "q0"), ("f1", "q1"), ("f2", "q2")]
B_CODE = "B = 6b-53 + 1b-99;"
B_HALF = "B = 1b-54;"
LEMMA = "in [-4503599627370497b-105, 4503599627370497b-105]"


def product(k):
    f, x = FACTORS[k]
    return f"{f} * {x}"


def make(zero=(), cancel=(), on_value=False, b=B_CODE, normal=True, eb_normal=True):
    live = [k for k in TERMS if k not in zero]
    out = "value" if on_value else "y"
    used = sorted({name for k in live for name in FACTORS[k]})

    src = [
        "# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurfaceThroughAPoint::on_floats at 8c943dfd",
        "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND at 8c943dfd",
        "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at 8c943dfd",
        "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at 8c943dfd",
        "# restates: crates/rcad-predicates/src/cascade.rs decide at 8c943dfd",
        "# SideOfAFlatSurfaceThroughAPoint::on_floats and its error bound: the sign",
        "# of facing . point - facing . through.",
    ]
    if on_value:
        src.append("# The claim is on value, the double on_floats returns, not on y.")
    if zero:
        src.append(f"# The case where product{'s' if len(zero) > 1 else ''} "
                   f"{', '.join(str(k) for k in zero)} {'are' if len(zero) > 1 else 'is'} zero.")
    if cancel:
        src.append(f"# The sum{'s' if len(cancel) > 1 else ''} adding product"
                   f"{'s' if len(cancel) > 1 else ''} {', '.join(str(k) for k in cancel)} "
                   f"may cancel to zero, which rounds to itself.")
    src += [
        "@rnd = float<ieee_64, ne>;",
        "",
        "# The facing, the point and the through point, each an f64. A product that",
        "# is zero has the double 0 for its point's or its through point's",
        "# coordinate; where the facing's coordinate is the zero, both its products",
        "# are zero, and every number below is the same as with both of those",
        "# coordinates zero.",
        "f0 = rnd(f0_); f1 = rnd(f1_); f2 = rnd(f2_);",
        " ".join(f"p{i} = 0;" if i in zero else f"p{i} = rnd(p{i}_);" for i in range(3)),
        " ".join(f"q{i} = 0;" if i + 3 in zero else f"q{i} = rnd(q{i}_);" for i in range(3)),
        "",
        "# What on_floats computes, line for line. The fold starts at 0.0, and",
        "# adding 0.0 or a zero product to a double gives that double back",
        "# (addzero.g), so the zero products and the first addition are left out.",
    ]
    if live:
        src.append(" ".join(f"t{k} = rnd({product(k)});" for k in live))

    rounded = []
    if not live:
        src.append("y = 0;")
    else:
        first = live[0]
        running = f"t{first}" if SIGN[first] == "+" else f"-t{first}"
        for k in live[1:-1]:
            src.append(f"S{k} = {running} {SIGN[k]} t{k};")
            if k in cancel:
                src.append(f"s{k} = S{k};")
            else:
                src.append(f"s{k} = rnd(S{k});")
                rounded.append(k)
            running = f"s{k}"
        if len(live) == 1:
            src.append(f"y = {running};")
        else:
            src.append(f"y = {running} {SIGN[live[-1]]} t{live[-1]};")
    value_rounded = len(live) >= 2
    src.append("value = rnd(y);" if value_rounded else "value = y;")

    src += ["", "# The permanent folds the same products' magnitudes in the same order."]
    if not live:
        src.append("P = 0;")
    else:
        perm = f"|t{live[0]}|"
        for k in live[1:]:
            perm = f"rnd({perm} + |t{k}|)"
        src.append(f"P = {perm};")
    src += ["", b, "eb = rnd(B * P);", "", "# The true value of the same inputs."]
    src.append("Ex = " + " + ".join(product(k) for k in (0, 1, 2)) + " - "
               + " - ".join(product(k) for k in (3, 4, 5)) + ";")
    src += ["", "# The permanent with no rounding of its own."]
    src.append("Phat = " + (" + ".join(f"|t{k}|" for k in live) if live else "0") + ";")

    if live:
        src += ["", "# What each rounding multiplied its argument by."]
        src.append(" ".join(f"rp{k} = t{k} / ({product(k)});" for k in live))
        if rounded:
            src.append(" ".join(f"rs{k} = s{k} / S{k};" for k in rounded))
        if on_value and value_rounded:
            src.append("rv = value / y;")
        src += ["", f"# Each product's term, as {out} carries it, less the term itself: e is",
                "# that amount and W the same over the product as on_floats rounds it.",
                "# The through point's terms are subtracted, so their e is the other way",
                "# round."]
        for k in live:
            carry = [f"rs{j}" for j in rounded if j >= k]
            if on_value and value_rounded:
                carry.append("rv")
            src.append(f"W{k} = {' * '.join(carry) if carry else '1'} - 1 / rp{k};")
        src.append(" ".join(f"e{k} = t{k} * W{k};" if SIGN[k] == "+" else f"e{k} = -(t{k} * W{k});"
                            for k in live))
        src.append(" ".join(f"c{k} = |t{k}|;" for k in live))

    hyps = []
    finite = ["|f0| <= 1b1024", "|f1| <= 1b1024", "|f2| <= 1b1024"]
    coords = [f"|p{i}| <= 1b1024" for i in range(3) if i not in zero] + \
             [f"|q{i}| <= 1b1024" for i in range(3) if i + 3 not in zero]
    hyps.append(("Every input is a finite f64.", [finite, coords]))
    if live:
        hyps.append(("No factor of a product left is zero. For an f64 that is the same as\n"
                     "  # being at least the smallest subnormal.",
                     [[f"|{n}| >= 1b-1074" for n in used if n[0] == "f"],
                      [f"|{n}| >= 1b-1074" for n in used if n[0] != "f"]]))
        if rounded:
            hyps.append(("No rounded sum cancels to zero.", [[f"|S{k}| >= 1b-1074" for k in rounded]]))
        if normal:
            hyps.append(("product_is_normal on the products left.",
                         [[f"|t{k}| >= 1b-1022" for k in live]]))
    if eb_normal:
        hyps.append(("The bound is normal, which decide checks.", [["|eb| >= 1b-1022"]]))
    lemma = []
    if normal:
        lemma += [[f"t{k} -/ {product(k)} {LEMMA}"] for k in live]
    if eb_normal:
        lemma.append([f"eb -/ B * P {LEMMA}"])
    if lemma:
        hyps.append(("What lemma.g proves each of those gives.", lemma))
    if live:
        hyps.append(("x / |x| is in [-1, 1] for any x that is not zero.",
                     [[f"t{k} / |t{k}| in [-1, 1]" for k in live]]))
    if on_value and value_rounded:
        hyps.append(("y is not zero. Where it is, value is zero too, and the claim on y is\n"
                     "  # the claim on value.", [["|y| >= 1b-1074"]]))

    src += ["", "{"]
    first_line = True
    for comment, rows in hyps:
        src.append(f"  # {comment}")
        for row in rows:
            if not row:
                continue
            src.append(("     " if first_line else "  /\\ ") + " /\\ ".join(row))
            first_line = False
    src.append("->")
    if live:
        src += [f"  ({out} - Ex) / Phat in ? /\\ P -/ Phat in ?",
                f"  /\\ ({out} - Ex) / eb in [-1, 1]"]
    else:
        src.append(f"  eb in ? /\\ ({out} - Ex) / eb in [-1, 1]")
    src += ["}", ""]

    if live:
        src.append("# How far each rounding is from the identity.")
        for k in live:
            src.append(f"t{k} / ({product(k)}) -> 1 + (t{k} - {product(k)}) / ({product(k)});")
        for k in rounded:
            src.append(f"s{k} / S{k} -> 1 + (s{k} - S{k}) / S{k};")
        if on_value and value_rounded:
            src.append("value / y -> 1 + (value - y) / y;")
        src += [
            "",
            "# The error is the sum of the products' errors, each a fraction of its",
            "# own piece of the permanent; add_rr then averages them.",
            f"({out} - Ex) / Phat -> ((Phat + ({out} - Ex)) - Phat) / Phat;",
            f"Phat + ({out} - Ex) -> " + " + ".join(f"(c{k} + e{k})" for k in live) + ";",
        ]
        for k in live:
            src.append(f"(c{k} + e{k}) - c{k} -> e{k};")
        for k in live:
            src.append(f"e{k} / c{k} -> (t{k} / |t{k}|) * W{k};" if SIGN[k] == "+"
                       else f"e{k} / c{k} -> -((t{k} / |t{k}|) * W{k});")
        src += [
            "",
            "# From the permanent to the one on_floats rounds, and on to eb.",
            f"({out} - Ex) / eb -> (({out} - Ex) / Phat) * (Phat / P) * (P / eb);",
            "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
            "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        ]
    return "\n".join(src) + "\n"


def cancellable(live):
    """The sets of rounded sums that can cancel to zero together. A sum that
    cancels leaves the running sum zero, so the next rounded sum is the next
    product alone, which is not zero."""
    sums = live[1:-1]
    for n in range(len(sums) + 1):
        for c in itertools.combinations(sums, n):
            if all(sums.index(b) - sums.index(a) > 1 for a, b in zip(c, c[1:])):
                yield c


def write(name, text):
    open(name, "w").write(text)


def name_of(prefix, zero, cancel):
    tag = (f"_z{''.join(map(str, zero))}" if zero else "") + \
          (f"_c{''.join(map(str, cancel))}" if cancel else "")
    return prefix + tag


if __name__ == "__main__":
    proving = []
    for n in range(6):
        for zero in itertools.combinations(TERMS, n):
            live = [k for k in TERMS if k not in zero]
            for cancel in cancellable(live):
                for on_value in (False, True):
                    if on_value and len(live) < 2:
                        continue
                    name = name_of("flatv" if on_value else "flat", zero, cancel)
                    write(f"{name}.g", make(zero, cancel, on_value))
                    write(f"c_{name}_half.g", make(zero, cancel, on_value, b=B_HALF))
                    proving.append(name)
    # Every product zero: then eb is zero, so decide does not compare, and
    # Gappa proves the goal from the contradiction. Its control drops eb normal.
    write("flat_z012345.g", make(tuple(TERMS)))
    write("c_flat_z012345_noeb.g", make(tuple(TERMS), eb_normal=False))
    write("c_flat_nonormal.g", make(normal=False))
    write("flat_noeb.g", make(eb_normal=False))
    print(f"{len(proving)} scripts that must prove with a control at u / 2 each, and flat_z012345.g")
