# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# One component of cyl.g alone: its square against the true one, over its
# reach squared, to time Gappa on three pieces.
@rnd = float<ieee_64, ne>;
py = rnd(py_); pz = rnd(pz_);
oy = rnd(oy_); oz = rnd(oz_);
dy = rnd(dy_); dz = rnd(dz_);
Fy = py - oy; Fz = pz - oz;
fy = rnd(Fy); fz = rnd(Fz);
hx = rnd(fy * dz); kx = rnd(fz * dy);
Qx = hx - kx; qx = rnd(Qx);
sx = rnd(qx * qx);
Cx = Fy * dz - Fz * dy;
rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rqx = qx / Qx; rsx = sx / (qx * qx);
Kx = rqx * rqx * rsx;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
WHx = Kx - ax * ax; WKx = Kx - bx * bx; WXx = Kx - ax * bx;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
eHx = hx * hx * WHx; eKx = kx * kx * WKx; eXx = -(2 * hx * kx * WXx);
P3 = cHx + cKx + cXx;
{
     |py| <= 1b1024 /\ |pz| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |qx| >= 1b-1074
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |sx| >= 1b-1022
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
->
  (sx - Cx * Cx) / P3 in ?
}
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
qx / Qx -> 1 + (qx - Qx) / Qx;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
(sx - Cx * Cx) / P3 -> ((P3 + (sx - Cx * Cx)) - P3) / P3;
P3 + (sx - Cx * Cx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
eHx / cHx -> (hx / |hx|) * (hx / |hx|) * WHx;
eKx / cKx -> (kx / |kx|) * (kx / |kx|) * WKx;
eXx / cXx -> -((hx / |hx|) * (kx / |kx|) * WXx);
