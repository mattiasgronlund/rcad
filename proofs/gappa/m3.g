# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# m3.g: m2.g with every piece doubled, so each ratio is at most about a half
# and y / P12 is bounded (464.15 4).
@rnd = float<ieee_64, ne>;
hx =rnd(hx_); kx = rnd(kx_); skin = rnd(sk_);
Qx = hx - kx; qx = rnd(Qx); sx = rnd(qx * qx);
rqx = qx / Qx; rsx = sx / (qx * qx);
Kx = rqx * rqx * rsx;
Yx = Kx * Qx * Qx;
y = sx - skin;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
P3x = cHx + cKx + cXx;
SG = skin * 1;
P12 = P3x + SG;
fHx = Kx * hx * hx; fKx = Kx * kx * kx; fXx = -(2 * Kx * hx * kx);
dHx = 2 * cHx; dKx = 2 * cKx; dXx = 2 * cXx;
P3dx = dHx + dKx + dXx; SGd = 2 * SG;
P12d = P3dx + SGd;
{ |hx| in [1b-1022, 1b1024] /\ |kx| in [1b-1022, 1b1024] /\ skin in [1b-1022, 1b1024]
  /\ |qx| >= 1b-1074 /\ |sx| >= 1b-1022
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  -> y / P12 in ? /\ ((P3dx + Yx) - P3dx) / P3dx in ? /\ fHx / dHx in ? }
qx / Qx -> 1 + (qx - Qx) / Qx;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
y / P12 -> 2 * (((P12d + y) - P12d) / P12d);
P12d + y -> (P3dx + Yx) + (SGd - skin);
P3dx + Yx -> (dHx + fHx) + (dKx + fKx) + (dXx + fXx);
(dHx + fHx) - dHx -> fHx;
(dKx + fKx) - dKx -> fKx;
(dXx + fXx) - dXx -> fXx;
fHx / dHx -> Kx * (hx / |hx|) * (hx / |hx|) / 2;
fKx / dKx -> Kx * (kx / |kx|) * (kx / |kx|) / 2;
fXx / dXx -> -(Kx * (hx / |hx|) * (kx / |kx|)) / 2;
((SGd - skin) - SGd) / SGd -> -(skin / SGd);
skin / SGd -> 1 / 2;
