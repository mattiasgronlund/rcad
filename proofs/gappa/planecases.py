"""Write plane.g's cases with one product below the normals, which
product_is_normal refuses: planes<k><sign>.g for product k in 0, 1, 2 and
its sign, "" positive or "n" negative, and each again at B = 3u as
c_planes<k><sign>_3u.g. 464.8 quotes planes0.g, the one for n0 * p0
positive, whole.

Each file starts with plane.g's restates lines, HEADER, which 464.8 does not
quote: proofs/bounds.py reads them."""

HEADER = (
    "# restates: crates/rcad-geom/src/plane.rs SideOfAFlatSurface::on_floats at affe0f01\n"
    "# restates: crates/rcad-predicates/src/point.rs exact_coordinates at affe0f01\n"
    "# restates: crates/rcad-geom/src/plane.rs SIDE_OF_PLANE_ERROR_BOUND at affe0f01\n"
    "# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at affe0f01\n"
    "# restates: crates/rcad-predicates/src/cascade.rs decide at affe0f01\n"
)
R = {0: "rs01 * rs012", 1: "rs01 * rs012", 2: "rs012"}


def make(k, neg, b="B = 4b-53 + 1b-101;"):
    others = [i for i in (0, 1, 2) if i != k]
    abs_t = {i: f"|t{i}|" for i in others}
    abs_t[k] = "-t%d" % k if neg else "t%d" % k
    sign_word = "negative" if neg else "positive"
    lines = [
        f"# plane.g where n{k} * p{k} may be below the normals, which product_is_normal",
        f"# refuses: n{k} * p{k} is under 2^-1022 and {sign_word}, and the other two",
        "# products are normal.",
        "@rnd = float<ieee_64, ne>;",
        "",
        "n0 = rnd(n0_); n1 = rnd(n1_); n2 = rnd(n2_);",
        "p0 = rnd(p0_); p1 = rnd(p1_); p2 = rnd(p2_);",
        "off = rnd(off_);",
        "",
        f"X{k} = n{k} * p{k};",
        "; ".join(f"t{i} = rnd(X{k})" if i == k else f"t{i} = rnd(n{i} * p{i})" for i in (0, 1, 2)) + ";",
        "S01 = t0 + t1;",
        "s01 = rnd(S01);",
        "S012 = s01 + t2;",
        "s012 = rnd(S012);",
        "y = s012 - off;",
        "value = rnd(y);",
        "",
        f"Phat = {abs_t[0]} + {abs_t[1]} + {abs_t[2]} + |off|;",
        f"P = rnd(rnd(rnd({abs_t[0]} + {abs_t[1]}) + {abs_t[2]}) + |off|);",
        "",
        b,
        "eb = rnd(B * P);",
        "",
        "Ex = " + " + ".join(f"X{k}" if i == k else f"n{i} * p{i}" for i in (0, 1, 2)) + " - off;",
        "",
        "; ".join(f"c{i} = -X{k}" if (i == k and neg) else f"c{i} = X{k}" if i == k else f"c{i} = |t{i}|" for i in (0, 1, 2)) + ";",
        f"P3 = {abs_t[0]} + {abs_t[1]} + {abs_t[2]};",
        f"# The three terms' part of the permanent with X{k} in place of t{k}.",
        "P3x = c0 + c1 + c2;",
        "",
        "; ".join(f"rp{i} = t{i} / (n{i} * p{i})" for i in others) + ";",
        "rs01 = s01 / S01; rs012 = s012 / S012;",
        "",
        f"# Each term's error as a fraction of the term, for X{k} and not t{k}, and",
        f"# what rounding X{k} cost, as an amount.",
    ]
    for i in (0, 1, 2):
        lines.append(f"W{i} = {R[i]} - 1;" if i == k else f"W{i} = {R[i]} - 1 / rp{i};")
    lines.append("; ".join(f"e{i} = X{k} * W{i}" if i == k else f"e{i} = t{i} * W{i}" for i in (0, 1, 2)) + ";")
    lines.append(f"d{k} = (t{k} - X{k}) * ({R[k]});")
    lines += [
        "",
        "{",
        "     |n0| <= 1b1024 /\\ |n1| <= 1b1024 /\\ |n2| <= 1b1024",
        "  /\\ |p0| <= 1b1024 /\\ |p1| <= 1b1024 /\\ |p2| <= 1b1024",
        "  /\\ |off| <= 1b1024",
        "  /\\ " + " /\\ ".join(f"|n{i}| >= 1b-1074" for i in others),
        "  /\\ " + " /\\ ".join(f"|p{i}| >= 1b-1074" for i in others),
        "  /\\ |S01| >= 1b-1074 /\\ |S012| >= 1b-1074",
        f"  # The case: X{k} is {sign_word} and below the normals, the other two",
        "  # products are normal. A product of two doubles that are not zero is at",
        "  # least 2^-2148.",
        f"  /\\ X{k} in " + ("[-1b-1022, -1b-2148]" if neg else "[1b-2148, 1b-1022]"),
        "  /\\ " + " /\\ ".join(f"|t{i}| >= 1b-1022" for i in others),
        "  # The bound is normal, which decide checks.",
        "  /\\ |eb| >= 1b-1022",
    ]
    for i in others:
        lines.append(f"  /\\ t{i} -/ n{i} * p{i} in [-4503599627370497b-105, 4503599627370497b-105]")
    lines += [
        "  /\\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]",
        "  /\\ " + " /\\ ".join(f"t{i} / |t{i}| in [-1, 1]" for i in others),
        "->",
        f"  (e0 + e1 + e2) / P3x in ? /\\ d{k} in ? /\\ P in ?",
        "  /\\ (y - Ex) / eb in [-1, 1]",
        "}",
        "",
    ]
    for i in others:
        lines.append(f"t{i} / (n{i} * p{i}) -> 1 + (t{i} - n{i} * p{i}) / (n{i} * p{i});")
    lines += [
        "s01 / S01 -> 1 + (s01 - S01) / S01;",
        "s012 / S012 -> 1 + (s012 - S012) / S012;",
        "",
        f"# The relative part is averaged over P3x as in plane.g; what rounding X{k}",
        "# cost is an amount, and eb being normal keeps it small against eb.",
        f"(y - Ex) / eb -> ((e0 + e1 + e2) / P3x) * (P3x / P) * (P / eb) + d{k} / eb;",
        "(e0 + e1 + e2) / P3x -> ((P3x + (e0 + e1 + e2)) - P3x) / P3x;",
        "P3x + (e0 + e1 + e2) -> (c0 + e0) + (c1 + e1) + (c2 + e2);",
        "(c0 + e0) - c0 -> e0;",
        "(c1 + e1) - c1 -> e1;",
        "(c2 + e2) - c2 -> e2;",
    ]
    for i in (0, 1, 2):
        if i == k:
            lines.append(f"e{i} / c{i} -> " + (f"-W{i};" if neg else f"W{i};"))
        else:
            lines.append(f"e{i} / c{i} -> (t{i} / |t{i}|) * W{i};")
    lines += [
        f"P3x / P -> (P3 / Phat) * (Phat / P) + " + (f"(t{k} - X{k}) / P;" if neg else f"(X{k} - t{k}) / P;"),
        "P3 / Phat -> 1 / (1 + |off| / P3);",
        "Phat / P -> 1 / (1 + (P - Phat) / Phat);",
        "P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));",
        "P -> eb / (B * (1 + (eb - B * P) / (B * P)));",
    ]
    return HEADER + "\n".join(lines) + "\n"


for k in (0, 1, 2):
    for neg in (False, True):
        name = f"planes{k}{'n' if neg else ''}"
        open(f"{name}.g", "w").write(make(k, neg))
        open(f"c_{name}_3u.g", "w").write(make(k, neg, "B = 3b-53;"))
