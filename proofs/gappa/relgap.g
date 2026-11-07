# restates: crates/rcad-geom/src/sphere.rs on_floats at 05d44c66
# relgap.g: a gap whose square is below the normals still has a ratio hint.
@rnd = float<ieee_64, ne>;
px = rnd(px_); cx = rnd(cx_);
Gx = px - cx;
gx = rnd(Gx);
{ |px| <= 1b1024 /\ |cx| <= 1b1024 /\ |gx| >= 1b-1074 /\ |gx| <= 1b-511
  -> gx -/ Gx in ? /\ (gx - Gx) / Gx in ? }
