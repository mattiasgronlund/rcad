"""Write two controls again with more goals, to name where each breaks or how
it gets through: c_flat_nonormal_why.g asks for what the rounding of the first
product multiplied it by; c_flat_noeb_why.g asks for P and eb, and
c_flat_noeb_rel.g for eb's relative error alone. flat_noeb.g itself proves, so
the _why control also asks for eb's relative error, which fails."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
GOAL = "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n"


def rewrite(src, dst, goal):
    text = open(src).read()
    assert text.count(GOAL) == 1, src
    open(dst, "w").write(text.replace(GOAL, goal))


rewrite("c_flat_nonormal.g", "c_flat_nonormal_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ rp0 in ?\n")
rewrite("flat_noeb.g", "c_flat_noeb_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ P in ? /\\ eb in ?\n"
        "  /\\ eb -/ B * P in ?\n")
text = open("flat_noeb.g").read()
head, rest = text.split("->\n", 1)
_, hints = rest.split("}\n", 1)
open("c_flat_noeb_rel.g", "w").write(head + "->\n  eb -/ B * P in ?\n}\n" + hints)
