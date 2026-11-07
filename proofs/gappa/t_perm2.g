# restates: crates/rcad-geom/src/cylinder.rs on_floats at e59fab75
# The permanent alone: six halves, each a normal double, and skin a normal
# double; P as on_floats rounds it against Phat with no rounding of its own.
@rnd = float<ieee_64, ne>;
hx = rnd(hx_); kx = rnd(kx_); hy = rnd(hy_); ky = rnd(ky_); hz = rnd(hz_); kz = rnd(kz_);
skin = rnd(skin_);
Nx = |hx| + |kx|; Ny = |hy| + |ky|; Nz = |hz| + |kz|;
nx = rnd(Nx); ny = rnd(Ny); nz = rnd(Nz);
vx = rnd(nx * nx); vy = rnd(ny * ny); vz = rnd(nz * nz);
V01 = vx + vy; v01 = rnd(V01);
V012 = v01 + vz; v012 = rnd(V012);
PP = v012 + skin; P = rnd(PP);
Phat = Nx * Nx + Ny * Ny + Nz * Nz + skin;
{
     |hx| <= 1b1024 /\ |kx| <= 1b1024 /\ |hy| <= 1b1024
  /\ |ky| <= 1b1024 /\ |hz| <= 1b1024 /\ |kz| <= 1b1024 /\ |skin| <= 1b1024
  /\ |hx| >= 1b-1022 /\ |kx| >= 1b-1022 /\ |hy| >= 1b-1022
  /\ |ky| >= 1b-1022 /\ |hz| >= 1b-1022 /\ |kz| >= 1b-1022
  /\ skin >= 1b-1022
  /\ vx -/ nx * nx in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vy -/ ny * ny in [-4503599627370497b-105, 4503599627370497b-105]
  /\ vz -/ nz * nz in [-4503599627370497b-105, 4503599627370497b-105]
->
  P -/ Phat in ?
}
