"""Write the two controls again with more goals, to name where each breaks or
how it gets through: c_sphere_nonormal_why.g asks for what the rounding of the
x square and of the radius's square multiplied them by; c_sphere_noeb_why.g
asks for P and eb and, so that it fails as a control must, eb's relative
error, which c_sphere_noeb_rel.g asks for alone. sphere_noeb.g itself proves."""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
GOAL = "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n"


def rewrite(src, dst, goal):
    text = open(src).read()
    assert text.count(GOAL) == 1, src
    open(dst, "w").write(text.replace(GOAL, goal))


rewrite("c_sphere_nonormal.g", "c_sphere_nonormal_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ rpx in ? /\\ rpr in ?\n")
rewrite("sphere_noeb.g", "c_sphere_noeb_why.g",
        "  (y - Ex) / Phat in ? /\\ P -/ Phat in ? /\\ P in ? /\\ eb in ?\n"
        "  /\\ eb -/ B * P in ?\n")
text = open("sphere_noeb.g").read()
head, rest = text.split("->\n", 1)
_, hints = rest.split("}\n", 1)
open("c_sphere_noeb_rel.g", "w").write(head + "->\n  eb -/ B * P in ?\n}\n" + hints)
