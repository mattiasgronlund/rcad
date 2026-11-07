"""Write o2s.g's other cases: o2sn.g (G negative), o2f.g and o2fn.g (the same
with forwards below the normals instead), and o2tiny.g (both products under
2^-1000)."""


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


base = open("o2s.g").read()

# G negative: |G| is -G and |backwards| is -backwards.
src = base
src = sub(src, "acy * bcx is under 2^-1022 and positive", "acy * bcx is under 2^-1022 and negative")
src = sub(src, "# backwards is not negative, as G is not, so |backwards| is backwards.\nPhat = |forwards| + backwards;",
          "# backwards is not positive, as G is not, so |backwards| is -backwards.\nPhat = |forwards| - backwards;")
src = sub(src, "c2 = G;", "c2 = -G;")
src = sub(src, "G is positive and below", "G is negative and above")
src = sub(src, "G in [1b-2148, 1b-1022]", "G in [-1b-1022, -1b-2148]")
src = sub(src, "e2 / c2 -> -W2;", "e2 / c2 -> W2;")
src = sub(src, "Phat2 / P -> Phat / P + (G - backwards) / P;", "Phat2 / P -> Phat / P + (backwards - G) / P;")
open("o2sn.g", "w").write(src)

# forwards below the normals instead of backwards.
for sign in ("", "n"):
    src = open(f"o2s{sign}.g").read()
    src = sub(src, "# o2.g where backwards may be below", "# o2.g where forwards may be below")
    if sign:
        src = sub(src, "acy * bcx is under 2^-1022 and negative, and acx * bcy is at",
                  "acx * bcy is under 2^-1022 and negative, and acy * bcx is at")
    else:
        src = sub(src, "acy * bcx is under 2^-1022 and positive, and acx * bcy is at",
                  "acx * bcy is under 2^-1022 and positive, and acy * bcx is at")
    src = sub(src, "F = acx * bcy;\nG = acy * bcx;\nforwards = rnd(F);\nbackwards = rnd(G);",
              "G = acx * bcy;\nF = acy * bcx;\nforwards = rnd(G);\nbackwards = rnd(F);")
    if sign:
        src = sub(src, "# backwards is not positive, as G is not, so |backwards| is -backwards.\nPhat = |forwards| - backwards;",
                  "# forwards is not positive, as G is not, so |forwards| is -forwards.\nPhat = -forwards + |backwards|;")
    else:
        src = sub(src, "# backwards is not negative, as G is not, so |backwards| is backwards.\nPhat = |forwards| + backwards;",
                  "# forwards is not negative, as G is not, so |forwards| is forwards.\nPhat = forwards + |backwards|;")
    src = sub(src, "rp1 = forwards / F;", "rp1 = backwards / F;")
    src = sub(src, "c1 = |forwards|;", "c1 = |backwards|;")
    src = sub(src, "W1 = 1 - 1 / (racx * rbcy * rp1);\nW2 = 1 - 1 / (racy * rbcx);",
              "W1 = 1 - 1 / (racy * rbcx * rp1);\nW2 = 1 - 1 / (racx * rbcy);")
    # y - Ex = (G - Acx Bcy) + (forwards - G) - (backwards - Acy Bcx).
    src = sub(src, "e1 = forwards * W1; e2 = -(G * W2);\nd2 = -(backwards - G);",
              "e1 = -(backwards * W1); e2 = G * W2;\nd2 = forwards - G;")
    src = sub(src, "/\\ forwards / |forwards| in [-1, 1]", "/\\ backwards / |backwards| in [-1, 1]")
    src = sub(src, "forwards / F -> 1 + (forwards - F) / F;", "backwards / F -> 1 + (backwards - F) / F;")
    src = sub(src, "e1 / c1 -> (forwards / |forwards|) * W1;", "e1 / c1 -> -((backwards / |backwards|) * W1);")
    if sign:
        src = sub(src, "e2 / c2 -> W2;", "e2 / c2 -> -W2;")
        src = sub(src, "Phat2 / P -> Phat / P + (backwards - G) / P;", "Phat2 / P -> Phat / P + (forwards - G) / P;")
    else:
        src = sub(src, "e2 / c2 -> -W2;", "e2 / c2 -> W2;")
        src = sub(src, "Phat2 / P -> Phat / P + (G - backwards) / P;", "Phat2 / P -> Phat / P + (G - forwards) / P;")
    open(f"o2f{sign}.g", "w").write(src)

# Both products under 2^-1000: then eb is not normal, so decide does not
# compare. Gappa proves the goal from the contradiction.
src = open("o2.g").read()
src = sub(src, "# o2.g\n", "# o2.g\n") if "# o2.g\n" in src else src
src = sub(src, "  # Both products are normal. on_floats does not check this.\n  /\\ |forwards| >= 1b-1022 /\\ |backwards| >= 1b-1022\n",
          "  # The case: both products are under 2^-1000.\n  /\\ |acx * bcy| <= 1b-1000 /\\ |acy * bcx| <= 1b-1000\n")
src = sub(src, "  # What lemma.g proves each of those three gives.\n"
          "  /\\ forwards -/ acx * bcy in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ backwards -/ acy * bcx in [-4503599627370497b-105, 4503599627370497b-105]\n"
          "  /\\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]\n", "")
src = sub(src, "  (y - Ex) / Phat in ? /\\ P -/ Phat in ?\n  /\\ (y - Ex) / eb in [-1, 1]", "  eb in ? /\\ (y - Ex) / eb in [-1, 1]")
open("o2tiny.g", "w").write(src)
