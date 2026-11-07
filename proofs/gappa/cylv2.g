# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves_if_normal at e59fab75
# restates: crates/rcad-geom/src/cylinder.rs cross_product_halves at e59fab75
# restates: crates/rcad-geom/src/cylinder.rs SIDE_OF_CYLINDER_ERROR_BOUND at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs product_is_normal at e59fab75
# restates: crates/rcad-predicates/src/cascade.rs decide at e59fab75
# HowFarFromTheSkin::on_floats and SIDE_OF_CYLINDER_ERROR_BOUND, rcad-geom's cylinder.
# The claim is on value, the double on_floats returns, not on y.
@rnd = float<ieee_64, ne>;

# The point, the axis origin, the axis direction and the radius, each an f64.
px = rnd(px_); py = rnd(py_); pz = rnd(pz_);
ox = rnd(ox_); oy = rnd(oy_); oz = rnd(oz_);
dx = rnd(dx_); dy = rnd(dy_); dz = rnd(dz_);
r = rnd(r_);

# What on_floats computes, line for line.
Fx = px - ox; Fy = py - oy; Fz = pz - oz;
fx = rnd(Fx); fy = rnd(Fy); fz = rnd(Fz);

# cross_product_halves: h is each component's left half, k its right half.
hx = rnd(fy * dz); kx = rnd(fz * dy);
hy = rnd(fz * dx); ky = rnd(fx * dz);
hz = rnd(fx * dy); kz = rnd(fy * dx);
Qx = hx - kx; Qy = hy - ky; Qz = hz - kz;
qx = rnd(Qx); qy = rnd(Qy); qz = rnd(Qz);

# direction_squared, left to right as Rust adds it.
tx = rnd(dx * dx); ty = rnd(dy * dy); tz = rnd(dz * dz);
T01 = tx + ty; t01 = rnd(T01);
T = t01 + tz; D = rnd(T);

rr = rnd(r * r);
skin = rnd(rr * D);
sx = rnd(qx * qx); sy = rnd(qy * qy); sz = rnd(qz * qz);
S01 = sx + sy; s01 = rnd(S01);
S012 = s01 + sz; s012 = rnd(S012);
y = s012 - skin;
value = rnd(y);

# The permanent: each half's reach squared, then the radius term.
nx = rnd(|hx| + |kx|); ny = rnd(|hy| + |ky|); nz = rnd(|hz| + |kz|);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
P = rnd(rnd(rnd(vx + vy) + vz) + skin);

B = 10b-53 + 82b-106;
eb = rnd(B * P);

# The true value of the same inputs.
Cx = Fy * dz - Fz * dy; Cy = Fz * dx - Fx * dz; Cz = Fx * dy - Fy * dx;
Ex = Cx * Cx + Cy * Cy + Cz * Cz - r * r * (dx * dx + dy * dy + dz * dz);

# The permanent with no rounding of its own.
Phat = (|hx| + |kx|) * (|hx| + |kx|) + (|hy| + |ky|) * (|hy| + |ky|)
     + (|hz| + |kz|) * (|hz| + |kz|) + skin;

# What each rounding multiplied its argument by.
rfx = fx / Fx; rfy = fy / Fy; rfz = fz / Fz;
rhx = hx / (fy * dz); rkx = kx / (fz * dy);
rhy = hy / (fz * dx); rky = ky / (fx * dz);
rhz = hz / (fx * dy); rkz = kz / (fy * dx);
rqx = qx / Qx; rqy = qy / Qy; rqz = qz / Qz;
rsx = sx / (qx * qx); rsy = sy / (qy * qy); rsz = sz / (qz * qz);
rs01 = s01 / S01; rs012 = s012 / S012;
rtx = tx / (dx * dx); rty = ty / (dy * dy); rtz = tz / (dz * dz);
rt01 = t01 / T01; rD = D / T;
rrr = rr / (r * r); rsk = skin / (rr * D);

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

# The radius term is three pieces, one for each direction component's
# square, which sum to skin. Each true piece is r^2 d^2.
cGx = rsk * rr * rD * rt01 * tx;
cGy = rsk * rr * rD * rt01 * ty;
cGz = rsk * rr * rD * tz;
WGx = 1 / (rrr * rtx * rsk * rD * rt01) - 1;
WGy = 1 / (rrr * rty * rsk * rD * rt01) - 1;
WGz = 1 / (rrr * rtz * rsk * rD) - 1;
eGx = cGx * WGx; eGy = cGy * WGy; eGz = cGz * WGz;

# The same permanent, as four blocks of the twelve pieces the error is
# averaged over: each component's reach squared, and the radius term.
P3x = cHx + cKx + cXx; P3y = cHy + cKy + cXy; P3z = cHz + cKz + cXz;
SG = cGx + cGy + cGz;
P12 = P3x + P3y + P3z + SG;

# Each component's square as y carries it, and the radius term as Ex has it.
Yx = Kx * Qx * Qx; Yy = Ky * Qy * Qy; Yz = Kz * Qz * Qz;
Gt = r * r * (dx * dx + dy * dy + dz * dz);

# y over the permanent: each component's square as y carries it is K
# times (h - k)^2, which is K times the three pieces of its reach
# squared, the third with a sign; the radius term is taken off whole.
fHx = Kx * hx * hx; fKx = Kx * kx * kx; fXx = -(2 * Kx * hx * kx);
fHy = Ky * hy * hy; fKy = Ky * ky * ky; fXy = -(2 * Ky * hy * ky);
fHz = Kz * hz * hz; fKz = Kz * kz * kz; fXz = -(2 * Kz * hz * kz);
# The same pieces doubled, so that each f is at most about half its
# piece: Gappa averages a sum's relative errors only while each is
# above -1, and f against the piece itself reaches -1.
dHx = 2 * cHx; dKx = 2 * cKx; dXx = 2 * cXx;
dHy = 2 * cHy; dKy = 2 * cKy; dXy = 2 * cXy;
dHz = 2 * cHz; dKz = 2 * cKz; dXz = 2 * cXz;
P3dx = dHx + dKx + dXx; P3dy = dHy + dKy + dXy; P3dz = dHz + dKz + dXz;
SGd = 2 * SG;
P12d = P3dx + P3dy + P3dz + SGd;

{
  # Every input is a finite f64.
     |px| <= 1b1024 /\ |py| <= 1b1024 /\ |pz| <= 1b1024
  /\ |ox| <= 1b1024 /\ |oy| <= 1b1024 /\ |oz| <= 1b1024
  /\ |dx| <= 1b1024 /\ |dy| <= 1b1024 /\ |dz| <= 1b1024
  /\ |r| <= 1b1024
  # No factor is zero, and no component cancels to zero. For an f64 that is
  # the same as being at least the smallest subnormal.
  /\ |fx| >= 1b-1074 /\ |fy| >= 1b-1074 /\ |fz| >= 1b-1074
  /\ |dx| >= 1b-1074 /\ |dy| >= 1b-1074 /\ |dz| >= 1b-1074
  /\ |r| >= 1b-1074
  /\ |qx| >= 1b-1074 /\ |qy| >= 1b-1074 /\ |qz| >= 1b-1074
  # product_is_normal on the six halves (cross_product_halves_if_normal),
  # and on the three components, the three direction components, the
  # radius and the radius term (stays_accurate).
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ |sx| >= 1b-1022 /\ |sy| >= 1b-1022 /\ |sz| >= 1b-1022
  /\ |tx| >= 1b-1022 /\ |ty| >= 1b-1022 /\ |tz| >= 1b-1022
  /\ |rr| >= 1b-1022 /\ |skin| >= 1b-1022
  # The permanent's three squares are normal too, since no component is
  # zero here: |q| <= n, rounding being monotone and |h - k| <= |h| + |k|,
  # so v = rnd(n * n) >= rnd(q * q) = s, which is normal.
  /\ |vx| >= 1b-1022 /\ |vy| >= 1b-1022 /\ |vz| >= 1b-1022
  # y is not zero. Where it is, value is zero too, and the claim on y is
  # the claim on value.
  /\ |y| >= 1b-1074
  # The bound is normal, which decide checks.
  /\ |eb| >= 1b-1022
  # What lemma.g proves each of those gives.
  /\ hx -/ fy * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kx -/ fz * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hy -/ fz * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ky -/ fx * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ hz -/ fx * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ kz -/ fy * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sx -/ qx * qx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sy -/ qy * qy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ sz -/ qz * qz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tx -/ dx * dx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ ty -/ dy * dy in [-4503599627370497b-105, 4503599627370497b-105]
  /\ tz -/ dz * dz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ rr -/ r * r in [-4503599627370497b-105, 4503599627370497b-105]
  /\ skin -/ rr * D in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
  /\ eb -/ B * P in [-4503599627370497b-105, 4503599627370497b-105]
  # x / |x| is in [-1, 1] for any x that is not zero.
  /\ hx / |hx| in [-1, 1] /\ kx / |kx| in [-1, 1]
  /\ hy / |hy| in [-1, 1] /\ ky / |ky| in [-1, 1]
  /\ hz / |hz| in [-1, 1] /\ kz / |kz| in [-1, 1]
->
  (y - Ex) / P12 in ? /\ y / P12 in ? /\ (value - y) / y in ? /\ P -/ Phat in ?
  /\ (value - Ex) / eb in [-1, 1]
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
tx / (dx * dx) -> 1 + (tx - dx * dx) / (dx * dx);
ty / (dy * dy) -> 1 + (ty - dy * dy) / (dy * dy);
tz / (dz * dz) -> 1 + (tz - dz * dz) / (dz * dz);
t01 / T01 -> 1 + (t01 - T01) / T01;
D / T -> 1 + (D - T) / T;
rr / (r * r) -> 1 + (rr - r * r) / (r * r);
skin / (rr * D) -> 1 + (skin - rr * D) / (rr * D);

# The error is the sum of four blocks' errors, each the sum of its pieces'
# errors, each a fraction of its own piece of the permanent; add_rr then
# averages them. In blocks so that no hint's identity spans all twelve
# pieces: one hint over all twelve did not finish in 600 s.
(y - Ex) / P12 -> ((P12 + (y - Ex)) - P12) / P12;
P12 + (y - Ex) -> (P3x + (Yx - Cx * Cx)) + (P3y + (Yy - Cy * Cy)) + (P3z + (Yz - Cz * Cz)) + (SG + (Gt - skin));
P3x + (Yx - Cx * Cx) -> (cHx + eHx) + (cKx + eKx) + (cXx + eXx);
P3y + (Yy - Cy * Cy) -> (cHy + eHy) + (cKy + eKy) + (cXy + eXy);
P3z + (Yz - Cz * Cz) -> (cHz + eHz) + (cKz + eKz) + (cXz + eXz);
SG + (Gt - skin) -> (cGx + eGx) + (cGy + eGy) + (cGz + eGz);
(cHx + eHx) - cHx -> eHx;
(cKx + eKx) - cKx -> eKx;
(cXx + eXx) - cXx -> eXx;
(cHy + eHy) - cHy -> eHy;
(cKy + eKy) - cKy -> eKy;
(cXy + eXy) - cXy -> eXy;
(cHz + eHz) - cHz -> eHz;
(cKz + eKz) - cKz -> eKz;
(cXz + eXz) - cXz -> eXz;
(cGx + eGx) - cGx -> eGx;
(cGy + eGy) - cGy -> eGy;
(cGz + eGz) - cGz -> eGz;
eHx / cHx -> (hx / |hx|) * (hx / |hx|) * WHx;
eKx / cKx -> (kx / |kx|) * (kx / |kx|) * WKx;
eXx / cXx -> -((hx / |hx|) * (kx / |kx|) * WXx);
eHy / cHy -> (hy / |hy|) * (hy / |hy|) * WHy;
eKy / cKy -> (ky / |ky|) * (ky / |ky|) * WKy;
eXy / cXy -> -((hy / |hy|) * (ky / |ky|) * WXy);
eHz / cHz -> (hz / |hz|) * (hz / |hz|) * WHz;
eKz / cKz -> (kz / |kz|) * (kz / |kz|) * WKz;
eXz / cXz -> -((hz / |hz|) * (kz / |kz|) * WXz);

# From the twelve pieces to the permanent on_floats rounds, and on to eb.
(value - Ex) / eb -> (y - Ex) / eb + (value - y) / eb;
(y - Ex) / eb -> ((y - Ex) / P12) * (Phat / P) * (P / eb);
(value - y) / eb -> ((value - y) / y) * (y / P12) * (Phat / P) * (P / eb);
Phat / P -> 1 / (1 + (P - Phat) / Phat);
P / eb -> 1 / (B * (1 + (eb - B * P) / (B * P)));

# value's own rounding is a fraction of y, and y a fraction of the
# permanent, averaged over the same four blocks, doubled.
y / P12 -> 2 * (((P12d + y) - P12d) / P12d);
P12d + y -> (P3dx + Yx) + (P3dy + Yy) + (P3dz + Yz) + (SGd - skin);
P3dx + Yx -> (dHx + fHx) + (dKx + fKx) + (dXx + fXx);
P3dy + Yy -> (dHy + fHy) + (dKy + fKy) + (dXy + fXy);
P3dz + Yz -> (dHz + fHz) + (dKz + fKz) + (dXz + fXz);
(dHx + fHx) - dHx -> fHx;
(dKx + fKx) - dKx -> fKx;
(dXx + fXx) - dXx -> fXx;
(dHy + fHy) - dHy -> fHy;
(dKy + fKy) - dKy -> fKy;
(dXy + fXy) - dXy -> fXy;
(dHz + fHz) - dHz -> fHz;
(dKz + fKz) - dKz -> fKz;
(dXz + fXz) - dXz -> fXz;
fHx / dHx -> Kx * (hx / |hx|) * (hx / |hx|) / 2;
fKx / dKx -> Kx * (kx / |kx|) * (kx / |kx|) / 2;
fXx / dXx -> -(Kx * (hx / |hx|) * (kx / |kx|)) / 2;
fHy / dHy -> Ky * (hy / |hy|) * (hy / |hy|) / 2;
fKy / dKy -> Ky * (ky / |ky|) * (ky / |ky|) / 2;
fXy / dXy -> -(Ky * (hy / |hy|) * (ky / |ky|)) / 2;
fHz / dHz -> Kz * (hz / |hz|) * (hz / |hz|) / 2;
fKz / dKz -> Kz * (kz / |kz|) * (kz / |kz|) / 2;
fXz / dXz -> -(Kz * (hz / |hz|) * (kz / |kz|)) / 2;
((SGd - skin) - SGd) / SGd -> -(skin / SGd);
skin / SGd -> 1 / 2;
