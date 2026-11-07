"""Write stage.g's and stage_e0.g's two share edges, their controls, and the
runs that show what the scripts do not use and why they are cut as they are.
An edge takes share at 0 or at 1, the two values the f64 divide can give
outside the range the main scripts take. Each control changes one thing: a
smaller bound, to see whether the script proved its goal from hypotheses that
contradict each other; or a hypothesis dropped or weakened, to see whether the
proof needs it."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def write(name, src):
    open(name, "w").write(src)


SHARE = "  /\\ share in [1b-1074, 9007199254740991b-53]\n"
SPLIT = "# Where 1 - share starts to be exact.\n$ share in (0.5);\n"

stage = open("stage.g").read()
small = open("stage_e0.g").read()

# The edges. With share at 0 or at 1, one of the two incoming errors has no
# weight, so XE / cE needs no average, and there is no half to split at.
AVERAGE = stage[stage.index("# XE / cE through the average"):stage.index("# XM a share at a time")]
DIRECT = ("# share is 0 or 1, so XE / cE needs no average.\n"
          "XE / cE -> (rest * ((d0 - t0) / E) + share * ((d1 - t1) / E)) / (1 + 9b-53);\n\n")
for name, at in (("stage_0.g", "[0, 0]"), ("stage_1.g", "[1, 1]")):
    text = sub(stage, SHARE, "  /\\ share in " + at + "\n")
    text = sub(text, AVERAGE, DIRECT)
    text = sub(text, " /\\ XE / V in ?", "")
    write(name, sub(text, SPLIT, ""))
# The share-at-1 edge, stage_e0_1.g, is proved by Gappa and refused by coqc
# (464.22 7), so it is not written: prove-bounds stamps only what coqc accepts.
# stage_e0_1_coq.g, below, is the edge with the hint that makes coqc refuse
# it dropped (464.40).
for name, at in (("stage_e0_0.g", "[0, 0]"),):
    text = sub(small, SHARE, "  /\\ share in " + at + "\n")
    write(name, sub(text, SPLIT, ""))

# 8.5u for the M part, below the 8.5u and a hair XMM reaches.
text = sub(stage, "cM = 9b-53 * M;", "cM = 17b-54 * M;")
write("c_stage_85.g", sub(text, "XM / cM -> XMM / 9b-53;", "XM / cM -> XMM / 17b-54;"))
# M allowed below the normal range.
write("c_stage_m.g", sub(stage, "/\\ M in [1b-1022, 1b1024]", "/\\ M in [1b-1074, 1b1024]"))
# Without the incoming error's size against M.
write("c_stage_noe.g", sub(stage, "  /\\ E / M in [1b-60, 1b-40]\n", ""))
# Without share's range: what the case split gives.
write("c_stage_noshare.g", sub(stage, SHARE, ""))
# Without M at least the true running values.
write("c_stage_not.g", sub(stage, "  /\\ t0 / M in [-1, 1] /\\ t1 / M in [-1, 1]\n", ""))
# Without the incoming errors at most E.
write("c_stage_nod.g", sub(stage, "  /\\ (d0 - t0) / E in [-1, 1] /\\ (d1 - t1) / E in [-1, 1]\n", ""))
E0_GOAL = "/\\ (out - Ex) / M in [-9b-53, 9b-53]"
# 8.375u for the small-E case, below the 8.5u it proves.
write("c_stage_e0_8375.g", sub(small, E0_GOAL, "/\\ (out - Ex) / M in [-67b-57, 67b-57]"))
# M allowed below the normal range.
write("c_stage_e0_m.g", sub(small, "/\\ M in [1b-1022, 1b1024]", "/\\ M in [1b-1074, 1b1024]"))
# Without the incoming errors at most 2^-60 M.
write("c_stage_e0_nod.g", sub(small, "  /\\ (d0 - t0) / M in [-1b-60, 1b-60] /\\ (d1 - t1) / M in [-1b-60, 1b-60]\n",
                        ""))

# The runs. Without the parameter between the two knots, and without the
# knots' difference positive and in range: neither is used by Gappa, and both
# enter by hand.
write("stage_noa.g", sub(stage, "  /\\ A in [0, 1]\n", ""))
write("stage_noknots.g", sub(stage, "  /\\ above - below in [1b-1074, 1b1024]\n", ""))
# The small-E case at 8.5u, which it reaches.
write("stage_e0_85.g", sub(small, E0_GOAL, "/\\ (out - Ex) / M in [-17b-54, 17b-54]"))
# The share-at-1 edge without the hint z0 / M -> rest * (d0 / M), and its
# control at 7u under the 7u and 2^-60 the edge reaches.
HINT = "z0 / M -> rest * (d0 / M);\n"
WHY = ("# Not z0 / M -> rest * (d0 / M), which stage_e0.g has. rest is 0 here, and\n"
       "# with that hint Gappa 1.8.0 proves P0 = z0 by saying rest has 0 bits,\n"
       "# FLT(rest, 0), which -Bcoq writes as it is and coqc refuses, since\n"
       "# gappalib's FLT takes a positive count of bits (464.40).\n")
edge = sub(sub(small, SHARE, "  /\\ share in [1, 1]\n"), SPLIT, "")
coq = sub(edge, HINT, WHY)
write("stage_e0_1_coq.g", coq)
write("c_stage_e0_1_coq_7.g", sub(coq, E0_GOAL, "/\\ (out - Ex) / M in [-7b-53, 7b-53]"))
# stage.g with E allowed to be zero, which is why stage_e0.g exists.
write("c_stage_e_from_0.g", sub(stage, "/\\ E / M in [1b-60, 1b-40]", "/\\ E / M in [0, 1b-40]"))
# stage.g averaging over the weights themselves, which is why it takes twice
# them.
text = sub(stage, "cR = 2 * rest * E; cS = 2 * share * E;", "cR = rest * E; cS = share * E;")
text = sub(text, "XE / cE -> (XE / V) * (2 * (rest + share) / (1 + 9b-53));",
           "XE / cE -> (XE / V) * ((rest + share) / (1 + 9b-53));")
text = sub(text, "rest * (d0 - t0) / cR -> ((d0 - t0) / E) / 2;", "rest * (d0 - t0) / cR -> (d0 - t0) / E;")
write("c_stage_plain.g",
      sub(text, "share * (d1 - t1) / cS -> ((d1 - t1) / E) / 2;", "share * (d1 - t1) / cS -> (d1 - t1) / E;"))
# stage_e0.g with share over all of [0, 1], which is why share is cut in three.
write("c_stage_e0_whole.g", sub(small, SHARE, "  /\\ share in [0, 1]\n"))
