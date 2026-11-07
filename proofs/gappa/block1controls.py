"""Write block1.g's and lemma.g's controls, the variants of §464.4 3 and 4 and
§464.5 that must fail. Each changes one thing in a script that proves: a
smaller bound, to see that it did not prove its goal from hypotheses that
contradict each other, or a hypothesis dropped, to see that the proof needs it.

Run as: python3 block1controls.py, beside block1.g and lemma.g.
"""


def sub(src, old, new):
    assert src.count(old) == 1, (old, src[:80])
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


B = "B = 7b-53 + 7b-103;"
block1 = open("block1.g").read()

# §464.4 4: fails at 6u, and at 7u + 46u^2, one u^2 under the smallest B proved.
write("c_block1_6u.g", sub(block1, B, "B = 6b-53;"))
write("c_block1_46.g", sub(block1, B, "B = 7b-53 + 23b-105;"))

# §464.4 3: without the nine products normal, and what lemma.g gives from
# them, the goal fails.
src = sub(
    block1,
    "  # §12.2's first condition: product_is_normal on the nine products.\n"
    "  /\\ |bdxcdy| >= 1b-1022 /\\ |cdxbdy| >= 1b-1022 /\\ |cdxady| >= 1b-1022\n"
    "  /\\ |adxcdy| >= 1b-1022 /\\ |adxbdy| >= 1b-1022 /\\ |bdxady| >= 1b-1022\n"
    "  /\\ |t1| >= 1b-1022 /\\ |t2| >= 1b-1022 /\\ |t3| >= 1b-1022\n",
    "",
)
bound = " in [-4503599627370497b-105, 4503599627370497b-105]\n"
for rounded, exact in (
    ("bdxcdy", "bdx * cdy"),
    ("cdxbdy", "cdx * bdy"),
    ("cdxady", "cdx * ady"),
    ("adxcdy", "adx * cdy"),
    ("adxbdy", "adx * bdy"),
    ("bdxady", "bdx * ady"),
    ("t1", "U1"),
    ("t2", "U2"),
    ("t3", "U3"),
):
    src = sub(src, f"  /\\ {rounded} -/ {exact}{bound}", "")
write("c_block1_nonormal.g", src)

# §464.4 3: without eb normal, and what lemma.g gives from it, the goal fails.
src = sub(block1, "  # §12.2's second condition: the bound is normal, which decide checks.\n  /\\ |eb| >= 1b-1022\n", "")
src = sub(src, f"  /\\ eb -/ B * P{bound}", "")
write("c_block1_noeb.g", src)

lemma = open("lemma.g").read()
interval = "[-4503599627370497b-105, 4503599627370497b-105]"

# §464.5: the lemma fails at u, where u + 2u^2 proves.
write("c_lemma_u.g", sub(lemma, interval, "[-1b-53, 1b-53]"))

# §464.5: and with the rounded value only at least the smallest subnormal.
write("c_lemma_subnormal.g", sub(lemma, "|p| >= 1b-1022", "|p| >= 1b-1074"))
