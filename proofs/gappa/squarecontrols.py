"""Write vector.g's and square.g's controls, and square.g's runs on how far
the turn may sit from length one. Each changes one thing: a smaller bound, to
see whether the script proved its goal from hypotheses that contradict each
other; a hypothesis dropped, to see whether the proof needs it; or the turn's
q.q held to another width, to see where the 64 stops holding."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


N4 = "  /\\ N4 in [9007199254740980b-53, 9007199254741004b-53]\n"

vector = open("vector.g").read()
VGOAL = "/\\ out - Ex in [-20b-53, 20b-53]"
# 19.5u, below the 18.6 q.q + 1 = 19.6u D reaches.
write("c_vector_195.g", sub(vector, VGOAL, "/\\ out - Ex in [-39b-54, 39b-54]"))
# Without |q.q - 1| <= 6 epsilon.
write("c_vector_non4.g", sub(vector, N4, ""))
# Without the nine products normal, and what lemma.g gives from them.
start = vector.index("  # The nine products are normal")
end = vector.index("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
end += len("  /\\ m0 -/ M in [-4503599627370497b-105, 4503599627370497b-105]\n")
block = vector[start:end]
assert block.count(">= 1b-1022") == 9 and block.count(" -/ ") == 9, block
write("c_vector_nonormal.g", vector[:start] + vector[end:])

square = open("square.g").read()
SGOAL = "/\\ OO - 1 in [-128b-53, 128b-53]"
# 119u, below the 2u + 48u + 2 sqrt(3) 20u = 119.28u the sum reaches.
write("c_square_119.g", sub(square, SGOAL, "/\\ OO - 1 in [-119b-53, 119b-53]"))
# Without |q.q - 1| <= 6 epsilon: the share the doc measures.
write("c_square_non4.g", sub(square, N4, ""))
# Without what vector.g proves of the turn's rounding.
D = ("  # What vector.g, vector1.g and vector2.g prove of each turned component.\n"
     "  /\\ d0 in [-20b-53, 20b-53] /\\ d1 in [-20b-53, 20b-53]\n"
     "  /\\ d2 in [-20b-53, 20b-53]\n")
write("c_square_nod.g", sub(square, D, ""))
# q.q held to 14u and to 15u of one, 7 and 7.5 epsilon, and to exactly one,
# which is what the doc's count of two roundings assumes.
write("square_14.g", sub(square, N4, "  /\\ N4 in [9007199254740978b-53, 9007199254741006b-53]\n"))
write("c_square_15.g", sub(square, N4, "  /\\ N4 in [9007199254740977b-53, 9007199254741007b-53]\n"))
write("square_one.g", sub(square, N4, "  /\\ N4 in [1, 1]\n"))
