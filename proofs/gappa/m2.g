# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# m2.g: one component and skin alone, to see why y / P12 is not bounded when
# each piece K h^2 over h^2 reaches -1 (464.15 4).
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
{ |hx| in [1b-1022, 1b1024] /\ |kx| in [1b-1022, 1b1024] /\ skin in [1b-1022, 1b1024]
  /\ |qx| >= 1b-1074 /\ |sx| >= 1b-1022
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  -> y / P12 in ? /\ Yx / P3x in ? /\ fHx / cHx in ? /\ ((P12 + y) - P12) / P12 in ? }
qx / Qx -> 1 + (qx - Qx) / Qx;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
y / P12 -> ((P12 + y) - P12) / P12;
P12 + y -> (P3x + Yx) + (SG - skin);
P3x + Yx -> (cHx + fHx) + (cKx + fKx) + (cXx + fXx);
(cHx + fHx) - cHx -> fHx;
(cKx + fKx) - cKx -> fKx;
(cXx + fXx) - cXx -> fXx;
fHx / cHx -> Kx * (hx / |hx|) * (hx / |hx|);
fKx / cKx -> Kx * (kx / |kx|) * (kx / |kx|);
fXx / cXx -> -(Kx * (hx / |hx|) * (kx / |kx|));
((SG - skin) - SG) / SG -> -(skin / SG);
skin / SG -> 1;
Yx / P3x -> ((P3x + Yx) - P3x) / P3x;
