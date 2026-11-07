"""Write two controls of skin_noeb.g, which proves, as 464.12's why.py does for
sphere_noeb.g: c_skin_noeb_rel.g, whose only goal is eb's relative error, and
c_skin_noeb_why.g, which asks for P and eb as well. Each fails on eb's relative
error, which is how skin_noeb.g gets past eb's rounding and was not traced."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
GOAL = "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n"


def rewrite(src, dst, goal):
    text = open(src).read()
    assert text.count(GOAL) == 1, src
    open(dst, "w").write(text.replace(GOAL, goal))


rewrite("skin_noeb.g", "c_skin_noeb_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ P in ? /\\ eb in ?\n"
        "  /\\ eb -/ B * P in ?\n")
text = open("skin_noeb.g").read()
head, rest = text.split("->\n", 1)
_, hints = rest.split("}\n", 1)
open("c_skin_noeb_rel.g", "w").write(head + "->\n  eb -/ B * P in ?\n}\n" + hints)
