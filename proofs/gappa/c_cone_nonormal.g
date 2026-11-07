# restates: crates/rcad-geom/src/cone.rs on_floats at e59fab75
# restates: crates/rcad-geom/src/cone.rs cross_product_halves_if_normal at e59fab75
# restates: crates/rcad-geom/src/cone.rs cross_product_halves at e59fab75
# restates: crates/rcad-geom/src/cone.rs SIDE_OF_CONE_ERROR_BOUND at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs decide at e59fab75
# HowFarFromTheSkin::on_floats and SIDE_OF_CONE_ERROR_BOUND, rcad-geom's cone.
# A control: without product_is_normal and the lemma lines it gives.
@rnd = float<ieee_64, ne>;

# The point, the apex, the axis direction and the slope, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
sl = rnd(sl_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
hx = rnd(fy * dz); kx = rnd(fz * dy);
hy = rnd(fz * dx); ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = rnd(fy * dx);
Qx = hx - kx; Qy = hy - ky; Qz = hz - kz;
qx = rnd(Qx); qy = rnd(Qy); qz = rnd(Qz);

# up_halves, and up, summed left to right as Rust adds them.
ux = rnd(fx * dx); uy = rnd(fy * dy); uz = rnd(fz * dz);
U01 = ux + uy; u01 = rnd(U01);
U = u01 + uz; up = rnd(U);

sx = rnd(qx * qx); sy = rnd(qy * qy); sz = rnd(qz * qz);
ss = rnd(sl * sl);
w = rnd(up * up);
along = rnd(ss * w);
S01 = sx + sy; s01 = rnd(S01);
S012 = s01 + sz; s012 = rnd(S012);
y = s012 - along;
value = rnd(y);

# The permanent: each half's reach squared, then the slope squared times the
# reach of up squared.
nx = rnd(|hx| + |kx|); ny = rnd(|hy| + |ky|); nz = rnd(|hz| + |kz|);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
R01 = rnd(|ux| + |uy|); R = rnd(R01 + |uz|);
R2 = rnd(R * R);
A = rnd(ss * R2);
P = rnd(rnd(rnd(vx + vy) + vz) + A);

B = 12b-53 + 3b-101;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Dt = Fx * dx + Fy * dy + Fz * dz;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - sl * sl * (Dt * Dt);

# The permanent with no rounding of its own.
G = |ux| + |uy| + |uz|;
Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
     + (|hz| + |kz|) * (|hz| + |kz|) + ss * (G * G);

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rqx = qx / Qx; rqy = qy / Qy; rqz = qz / Qz;
rsx = sx / (qx * qx); rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs01 = s01 / S01; rs012 = s012 / S012;
rux = ux / (fx * dx); ruy = uy / (fy * dy); ruz = uz / (fz * dz);
ru01 = u01 / U01; rU = up / U;
rss = ss / (sl * sl); rw = w / (up * up); ral = along / (ss * w);

# Each component's square, as y carries it, is (h - k)^2 times K; the true
# one is (h a - k b)^2, a and b undoing the roundings of h and k.
Kx = rqx * rqx * rsx * rs01 * rs012;
Ky = rqy * rqy * rsy * rs01 * rs012;
Kz = rqz * rqz * rsz * rs012;
ax = 1 / (rfy * rhx); bx = 1 / (rfz * rkx);
ay = 1 / (rfz * rhy); by = 1 / (rfx * rky);
az = 1 / (rfx * rhz); bz = 1 / (rfy * rkz);

# So each component's error is three terms, one for each piece of its
# reach squared: h^2, k^2 and 2 |h| |k|. W is each term's error over its
# piece.
WHx = Kx - ax * ax; WKx = Kx - bx * bx; WXx = Kx - ax * bx;
WHy = Ky - ay * ay; WKy = Ky - by * by; WXy = Ky - ay * by;
WHz = Kz - az * az; WKz = Kz - bz * bz; WXz = Kz - az * bz;
cHx = |hx| * |hx|; cKx = |kx| * |kx|; cXx = 2 * |hx| * |kx|;
cHy = |hy| * |hy|; cKy = |ky| * |ky|; cXy = 2 * |hy| * |ky|;
cHz = |hz| * |hz|; cKz = |kz| * |kz|; cXz = 2 * |hz| * |kz|;
eHx = hx * hx * WHx; eKx = kx * kx * WKx; eXx = -(2 * hx * kx * WXx);
eHy = hy * hy * WHy; eKy = ky * ky * WKy; eXy = -(2 * hy * ky * WXy);
eHz = hz * hz * WHz; eKz = kz * kz * WKz; eXz = -(2 * hz * kz * WXz);

# The axial term, as y carries it, is ss (gx ux + gy uy + gz uz)^2 times KA,
# g undoing nothing and folding in the roundings of the two sums; the true
# one is sl^2 (Dt)^2 = ss (ex ux + ey uy + ez uz)^2 / rss, e undoing the
# roundings of f and of the product. So its error is six terms, one for each
# piece of ss G^2: ss u^2 for each of the three, and 2 ss |u| |u'| for each
# pair. W is each term's error over its piece.
KA = rw * ral;
gx = ru01 * rU; gy = ru01 * rU; gz = rU;
ex = 1 / (rfx * rux); ey = 1 / (rfy * ruy); ez = 1 / (rfz * ruz);
WAxx = KA * gx * gx - ex * ex / rss;
WAyy = KA * gy * gy - ey * ey / rss;
WAzz = KA * gz * gz - ez * ez / rss;
WAxy = KA * gx * gy - ex * ey / rss;
WAxz = KA * gx * gz - ex * ez / rss;
WAyz = KA * gy * gz - ey * ez / rss;
cAxx = ss * |ux| * |ux|; cAyy = ss * |uy| * |uy|; cAzz = ss * |uz| * |uz|;
cAxy = 2 * ss * |ux| * |uy|; cAxz = 2 * ss * |ux| * |uz|; cAyz = 2 * ss * |uy| * |uz|;
eAxx = -(ss * ux * ux * WAxx); eAyy = -(ss * uy * uy * WAyy); eAzz = -(ss * uz * uz * WAzz);
eAxy = -(2 * ss * ux * uy * WAxy); eAxz = -(2 * ss * ux * uz * WAxz);
eAyz = -(2 * ss * uy * uz * WAyz);

# The same permanent, as four blocks of the fifteen pieces the error is
# averaged over: each component's reach squared, and the axial term.
P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;
SA = cAxx + cAyy + cAzz + cAxy + cAxz + cAyz;
P15 = P3x + P3y + P3z + SA;

# Each component's square as y carries it, and the axial term as Ex has it.
Yx = Kx * Qx * Qx; Yy = Ky * Qy * Qy; Yz = Kz * Qz * Qz;
At = sl * sl * (Dt * Dt);

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |sl| <= 1b1024
  # No factor is zero, no component cancels to zero and nor does up. For an
  # f64 that is the same as being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |sl| >= 1b-1074
  /\ |qx| >= 1b-1074 /\ |qy| >= 1b-1074 /\ |qz| >= 1b-1074
  /\ |up| >= 1b-1074
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  /\ hy / |hy| in [-1, 1] /\ ky / |ky| in [-1, 1]
  /\ hz / |hz| in [-1, 1] /\ kz / |kz| in [-1, 1]
  /\ ux / |ux| in [-1, 1] /\ uy / |uy| in [-1, 1] /\ uz / |uz| in [-1, 1]
->
  (y - Ex) / P15 in ? /\ P -/ Phat in ?
  /\ (y - Ex) / eb in [-1, 1]
}

# How far each rounding is from the identity.
fx / Fx -> 1 + (fx - Fx) / Fx;
fy / Fy -> 1 + (fy - Fy) / Fy;
fz / Fz -> 1 + (fz - Fz) / Fz;
hx / (fy * dz) -> 1 + (hx - fy * dz) / (fy * dz);
kx / (fz * dy) -> 1 + (kx - fz * dy) / (fz * dy);
hy / (fz * dx) -> 1 + (hy - fz * dx) / (fz * dx);
ky / (fx * dz) -> 1 + (ky - fx * dz) / (fx * dz);
hz / (fx * dy) -> 1 + (hz - fx * dy) / (fx * dy);
kz / (fy * dx) -> 1 + (kz - fy * dx) / (fy * dx);
qx / Qx -> 1 + (qx - Qx) / Qx;
qy / Qy -> 1 + (qy - Qy) / Qy;
qz / Qz -> 1 + (qz - Qz) / Qz;
sx / (qx * qx) -> 1 + (sx - qx * qx) / (qx * qx);
sy / (qy * qy) -> 1 + (sy - qy * qy) / (qy * qy);
sz / (qz * qz) -> 1 + (sz - qz * qz) / (qz * qz);
s01 / S01 -> 1 + (s01 - S01) / S01;
s012 / S012 -> 1 + (s012 - S012) / S012;
ux / (fx * dx) -> 1 + (ux - fx * dx) / (fx * dx);
uy / (fy * dy) -> 1 + (uy - fy * dy) / (fy * dy);
uz / (fz * dz) -> 1 + (uz - fz * dz) / (fz * dz);
u01 / U01 -> 1 + (u01 - U01) / U01;
up / U -> 1 + (up - U) / U;
ss / (sl * sl) -> 1 + (ss - sl * sl) / (sl * sl);
w / (up * up) -> 1 + (w - up * up) / (up * up);
along / (ss * w) -> 1 + (along - ss * w) / (ss * w);

# The error is the sum of four blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. In blocks so that no hint's identity spans all fifteen
# pieces, as cyl.g learned.
(y - Ex) / P15 -> ((P15 + (y - Ex)) - P15) / P15;
P15 + (y - Ex) -> (P3x + (Yx - Cx * Cx)) + (P3y + (Yy - Cy * Cy)) + (P3z + (Yz - Cz * Cz)) + (SA + (At - along));
P3x + (Yx - Cx * Cx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
P3y + (Yy - Cy * Cy) -> (cHy + eHy) + (cKy + eKy) + (cXy + eXy);
P3z + (Yz - Cz * Cz) -> (cHz + eHz) + (cKz + eKz) + (cXz + eXz);
SA + (At - along) -> (cAxx + eAxx) + (cAyy + eAyy) + (cAzz + eAzz) + (cAxy + eAxy) + (cAxz + eAxz) + (cAyz + eAyz);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
(cHy + eHy) - cHy -> eHy;
(cKy + eKy) - cKy -> eKy;
(cXy + eXy) - cXy -> eXy;
(cHz + eHz) - cHz -> eHz;
(cKz + eKz) - cKz -> eKz;
(cXz + eXz) - cXz -> eXz;
(cAxx + eAxx) - cAxx -> eAxx;
(cAyy + eAyy) - cAyy -> eAyy;
(cAzz + eAzz) - cAzz -> eAzz;
(cAxy + eAxy) - cAxy -> eAxy;
(cAxz + eAxz) - cAxz -> eAxz;
(cAyz + eAyz) - cAyz -> eAyz;
eHx / cHx -> (hx / |hx|) * (hx / |hx|) * WHx;
eKx / cKx -> (kx / |kx|) * (kx / |kx|) * WKx;
eXx / cXx -> -((hx / |hx|) * (kx / |kx|) * WXx);
eHy / cHy -> (hy / |hy|) * (hy / |hy|) * WHy;
eKy / cKy -> (ky / |ky|) * (ky / |ky|) * WKy;
eXy / cXy -> -((hy / |hy|) * (ky / |ky|) * WXy);
eHz / cHz -> (hz / |hz|) * (hz / |hz|) * WHz;
eKz / cKz -> (kz / |kz|) * (kz / |kz|) * WKz;
eXz / cXz -> -((hz / |hz|) * (kz / |kz|) * WXz);
eAxx / cAxx -> -((ux / |ux|) * (ux / |ux|) * WAxx);
eAyy / cAyy -> -((uy / |uy|) * (uy / |uy|) * WAyy);
eAzz / cAzz -> -((uz / |uz|) * (uz / |uz|) * WAzz);
eAxy / cAxy -> -((ux / |ux|) * (uy / |uy|) * WAxy);
eAxz / cAxz -> -((ux / |ux|) * (uz / |uz|) * WAxz);
eAyz / cAyz -> -((uy / |uy|) * (uz / |uz|) * WAyz);

# From the fifteen pieces to the permanent on_floats rounds, and on to eb.
(y - Ex) / eb -> ((y - Ex) / P15) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));
