"""Write 464.23's five scripts, parabola_s1.g to parabola_s5.g, and their controls.
Each script is the same chain, the scalars of the parabola a plane along a ruling
cuts a cone in, over one range of the slope t; the ranges are cut where the share
G is taken over a different length and where Gappa needs a finer split. Each
control changes one thing in one script and must fail. The section's probes, which
ask for G's range with no goal and prove nothing, are not here."""

import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

HEAD = """# restates: crates/rcad-geom/src/walk.rs POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF at 7485253e
# restates: crates/rcad-geom/src/walk.rs what_a_plane_cuts_out_of_a_cone at 17306d5ee
# restates: crates/rcad-geom/src/walk.rs the_parabola_a_plane_along_a_ruling_cuts at 7485253e
# restates: Cargo.lock nalgebra at 7485253e
# The scalars of the parabola a plane along a ruling cuts a cone in, as
# where_a_plane_cuts_a_cone and the_parabola_a_plane_along_a_ruling_cuts work
# them out in f64, against the same scalars without rounding, whose names end
# in S. hs is the apex's true height over the plane, nN and wW the true lengths
# of the plane's normal n and of the cone's axis w, t the cone's slope. R, Nn,
# Ww and U are the exact sums walk.rs rounds once each. Bw is the f64 dot of
# the computed `and` with w, and Bws the true one, -|w| / sqrt(1 + t^2) on a
# parabola. G is what the scalars put into the point at s = r * KS along `and`,
# over |KS| or over |offS|; q is the foot's quotient.
"""

CHAIN = """@rnd = float<ieee_64, ne>;

t = rnd(t_);
Bw = rnd(Bw_);

Ts = 1 + t * t;
As = t / sqrt(Ts);
Bs = -1 / sqrt(Ts);

R = hs * nN;
Nn = nN * nN;
Ww = wW * wW;
U = As * (nN * wW);
Bws = Bs * wW;

reach = rnd(R); sqn = rnd(Nn); sqw = rnd(Ww); up = rnd(U);
q = rnd(reach / sqn);
nl = rnd(sqrt(sqn)); wl = rnd(sqrt(sqw));
h = rnd(reach / nl);
nw = rnd(nl * wl);
au = rnd(up / nw);
bu = rnd(Bw / wl);
tt = rnd(t * t);
t2 = rnd(1 + tt);
k1 = rnd(2 * t2);
k2 = rnd(k1 * h);
k3 = rnd(k2 * au);
K = rnd(k3 * bu);
hh = rnd(h * h);
c1 = rnd(t2 * au);
c2 = rnd(c1 * au);
br = rnd(1 - c2);
C = rnd(hh * br);
off = rnd(C / K);
mrk = rnd(1 / K);

qS = R / Nn;
nlS = sqrt(Nn); wlS = sqrt(Ww);
hS = R / nlS;
nwS = nlS * wlS;
auS = U / nwS;
buS = Bws / wlS;
t2S = 1 + t * t;
k1S = 2 * t2S;
k2S = k1S * hS;
k3S = k2S * auS;
KS = k3S * buS;
hhS = hS * hS;
c1S = t2S * auS;
c2S = c1S * auS;
brS = 1 - c2S;
CS = hhS * brS;
offS = CS / KS;
m = mrk * KS;
"""

# t at least 7/16: G is the scalars' share along `and` over |KS|.
BIG = """G = (off - offS) / KS + r * r * (m - 1);
"""

# t at most 7/16: the same share over |offS|, which is the larger there.
SMALL = """G = (off - offS) / offS + r * r * ((4 * t * t) / (1 - t * t)) * (m - 1);
"""

T = "  /\\ t in [TLO, THI]\n"
BW = "  /\\ (Bw - Bws) / wW in [-15b-54, 15b-54]\n"
RS = "  /\\ r in [-1099511627777b-40, 1099511627777b-40]\n"
GOAL = "  |G| <= 84b-53 /\\ |(q - qS) / qS| <= 25b-56 /\\ |(K - KS) / KS| <= 1b-44\n"

HYP = """
{
     hs in [1, 2]
  /\\ nN in [1, 2] /\\ wW in [1, 2]
""" + T + BW + RS + """->
""" + GOAL + """}

reach ~ R; sqn ~ Nn; sqw ~ Ww; up ~ U; Bw ~ Bws; q ~ qS;
nl ~ nlS; wl ~ wlS; h ~ hS; nw ~ nwS; au ~ auS; bu ~ buS;
tt ~ t * t; t2 ~ t2S; k1 ~ k1S; k2 ~ k2S; k3 ~ k3S; K ~ KS;
hh ~ hhS; c1 ~ c1S; c2 ~ c2S;

hS -> hs;
auS -> As;
buS -> Bs;
c2S -> t * t;
KS -> -2 * hs * t;

(C - CS) / hhS -> ((C - hh * br) / (hh * br)) * ((hh * br) / hhS) + ((hh - hhS) / hhS) * br + (br - brS);
br - brS -> (br - (1 - c2)) - (c2 - c2S);
c2 - c2S -> ((c2 - c2S) / c2S) * (t * t);
m - 1 -> ((mrk - 1 / K) / (1 / K)) * (KS / K) - (K - KS) / K;
KS / K -> 1 / (1 + (K - KS) / KS);
(K - KS) / K -> ((K - KS) / KS) / (1 + (K - KS) / KS);
"""

BIG_HINTS = """G -> (off - C / K) / KS + (C - CS) / (K * KS) + r * r * (((mrk - 1 / K) / (1 / K)) * (KS / K))
     - ((1 - t * t) / (4 * t * t) + r * r) * ((K - KS) / K);
(off - C / K) / KS -> ((off - C / K) / (C / K)) * ((C / K) / KS);
(C / K) / KS -> (C - CS) / (K * KS) + ((1 - t * t) / (4 * t * t)) * (KS / K);
(C - CS) / (K * KS) -> ((C - CS) / hhS) * ((KS / K) / (4 * t * t));
"""

SMALL_HINTS = """G -> ((off - C / K) / (C / K)) * (1 + (C - CS) / CS) * (KS / K) + ((C - CS) / CS) * (KS / K)
     + r * r * ((4 * t * t) / (1 - t * t)) * (((mrk - 1 / K) / (1 / K)) * (KS / K))
     - (1 + r * r * ((4 * t * t) / (1 - t * t))) * ((K - KS) / K);
(C - CS) / CS -> ((C - hh * br) / (hh * br)) * (1 + (hh - hhS) / hhS) * (1 + (br - brS) / brS)
     + ((hh - hhS) / hhS) * (1 + (br - brS) / brS) + (br - brS) / brS;
(br - brS) / brS -> ((br - (1 - c2)) / (1 - c2)) * (1 - ((c2 - c2S) / c2S) * ((t * t) / (1 - t * t)))
     - ((c2 - c2S) / c2S) * ((t * t) / (1 - t * t));
"""


def sub(src, old, new):
    assert src.count(old) == 1, old
    return src.replace(old, new)


def script(lo, hi, small, split):
    body, hints = (SMALL, SMALL_HINTS) if small else (BIG, BIG_HINTS)
    text = HEAD + CHAIN + body + HYP + hints
    text = sub(text, "TLO", lo)
    text = sub(text, "THI", hi)
    return text + ("\n$ t in " + split + ";\n" if split else "")


def write(name, text):
    open(name, "w").write(text)


SCRIPTS = {
    "parabola_s1.g": script("1b-500", "0.25", True, None),
    "parabola_s2.g": script("0.25", "0.4375", True, "16"),
    "parabola_s3.g": script("0.4375", "1", False, "32"),
    "parabola_s4.g": script("1", "2", False, "8"),
    "parabola_s5.g": script("2", "5", False, "8"),
}
for name, text in SCRIPTS.items():
    write(name, text)

# The controls. A goal under what s4 reaches, to see that its hypotheses do
# not contradict each other; then a hypothesis dropped or widened, to see that
# the proof needs it. The small shape's goal control, s1.g at 40u, ran past
# one.sh's 600 s, so it is not here: 464.23's 6 gives a point that meets every
# hypothesis instead.
G84 = "|G| <= 84b-53"
write("c_parabola_s4_40.g", sub(SCRIPTS["parabola_s4.g"], G84, "|G| <= 40b-53"))
write("c_parabola_s1_nobw.g", sub(SCRIPTS["parabola_s1.g"], BW, ""))
write("c_parabola_s1_nor.g", sub(SCRIPTS["parabola_s1.g"], RS, ""))
write("c_parabola_s5_to8.g", sub(SCRIPTS["parabola_s5.g"], "t in [2, 5]", "t in [2, 8]"))

