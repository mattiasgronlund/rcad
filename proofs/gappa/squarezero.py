"""Write 464.20's square.g in the zero branches: one script for each zero set
of the exact unit vector m with at most two components zero, crossed with
each zero set of the turn with at most two parts zero, and a control beside
each.

A zero m_i is the constructor's n_i = 0 exactly, so its m_i, g_i and h_i
drop out of the averaging over the m_i^2, and M2 is the sum over the m_i
left, still one. square.g puts no lower bound on a part of the turn, so a
zero part is inside its hypotheses already; writing it as the double 0 shows
it. Three or four parts zero never reach the turn's arithmetic or cannot
keep q.q near one (464.34's 1). The hypothesis on d is vector.g's 20u, which
464.36's 88 cases prove in every zero branch of the vector and the turn.

Each script keeps square.g's restates lines, so it restates what square.g does.

The names are sq_<zeros>.g and c_sq_<zeros>_half.g, <zeros> the zero
factors, m's first, or n for none; casesq.txt lists them.
Run as: python3 squarezero.py"""
import itertools
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

M = ("m0", "m1", "m2")
TURN = ("w", "v0", "v1", "v2")
GOAL = "/\\ OO - 1 in [-128b-53, 128b-53]"
HALF = "/\\ OO - 1 in [-1b-54, 1b-54]"


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def kept(names, zero):
    return [x for x in names if x not in zero]


def make(square, zero):
    zeros = [x for x in M + TURN if x in zero]
    idx = [i for i in range(3) if f"m{i}" not in zero]
    gone = [i for i in range(3) if f"m{i}" in zero]
    assert square.count("4b642567") == 5
    src = square
    src = sub(src, "@rnd = float<ieee_64, ne>;\n",
              f"# 464.36: with zero factors {', '.join(zeros) or 'none'}, each the double 0.\n"
              "@rnd = float<ieee_64, ne>;\n")
    src = sub(src, "n0 = rnd(m0); n1 = rnd(m1); n2 = rnd(m2);",
              " ".join(f"n{i} = 0;" if i in gone else f"n{i} = rnd(m{i});" for i in range(3)))
    src = sub(src, "M2 = m0 * m0 + m1 * m1 + m2 * m2;", "M2 = " + " + ".join(f"m{i} * m{i}" for i in idx) + ";")
    src = sub(src, "w = rnd(w_); v0 = rnd(v0_); v1 = rnd(v1_); v2 = rnd(v2_);",
              " ".join(f"{x} = 0;" if x in zero else f"{x} = rnd({x}_);" for x in TURN))
    src = sub(src, "g0 = m0 * m0; g1 = m1 * m1; g2 = m2 * m2;", " ".join(f"g{i} = m{i} * m{i};" for i in idx))
    src = sub(src, "h0 = n0 * n0 - m0 * m0; h1 = n1 * n1 - m1 * m1; h2 = n2 * n2 - m2 * m2;",
              " ".join(f"h{i} = n{i} * n{i} - m{i} * m{i};" for i in idx))
    src = sub(src, "  /\\ m0 in [-1, 1] /\\ m1 in [-1, 1] /\\ m2 in [-1, 1]\n",
              "  /\\ " + " /\\ ".join(f"m{i} in [-1, 1]" for i in idx) + "\n")
    src = sub(src, "  # (scaled_to_length_one_from_anywhere's doc), and none is zero.\n"
                   "  /\\ |m0| >= 1b-1012 /\\ |m1| >= 1b-1012 /\\ |m2| >= 1b-1012\n",
              "  # (scaled_to_length_one_from_anywhere's doc), and none is zero but "
              + (", ".join(f"m{i}" for i in gone) or "none") + ".\n"
              "  /\\ " + " /\\ ".join(f"|m{i}| >= 1b-1012" for i in idx) + "\n")
    src = sub(src, "  /\\ |w| <= 1b1024 /\\ |v0| <= 1b1024 /\\ |v1| <= 1b1024 /\\ |v2| <= 1b1024\n",
              "  /\\ " + " /\\ ".join(f"|{x}| <= 1b1024" for x in kept(TURN, zero)) + "\n")
    src = sub(src, "M2 + (NN - M2) -> (g0 + h0) + (g1 + h1) + (g2 + h2);",
              "M2 + (NN - M2) -> " + " + ".join(f"(g{i} + h{i})" for i in idx) + ";")
    for i in gone:
        for line in (f"(g{i} + h{i}) - g{i} -> h{i};\n", f"h{i} / g{i} -> (n{i} / m{i}) * (n{i} / m{i}) - 1;\n",
                     f"n{i} / m{i} -> 1 + (n{i} - m{i}) / m{i};\n"):
            src = sub(src, line, "")
    return src


def name_of(zero):
    return "sq_" + ("".join(x for x in M + TURN if x in zero) or "n")


def main():
    square = open("square.g").read()
    seen = []
    for k in range(3):
        for mz in itertools.combinations(M, k):
            for j in range(3):
                for tz in itertools.combinations(TURN, j):
                    zero = set(mz) | set(tz)
                    name = name_of(zero)
                    src = make(square, zero)
                    open(name + ".g", "w").write(src)
                    open("c_" + name + "_half.g", "w").write(sub(src, GOAL, HALF))
                    seen.append(name)
    assert len(set(seen)) == len(seen)
    open("casesq.txt", "w").write("\n".join(seen) + "\n")
    print(len(seen), "cases")


if __name__ == "__main__":
    main()
