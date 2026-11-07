# sum.g's control: the same at u / 2, which must fail.
@rnd = float<ieee_64, ne>;

a = rnd(a_); b = rnd(b_);
S = a + b; s = rnd(S);
T = a - b; t = rnd(T);

{
     |a| <= 1b1024 /\ |b| <= 1b1024
  /\ |S| >= 1b-1074 /\ |T| >= 1b-1074
->
  (s - S) / S in [-1b-54, 1b-54] /\ (t - T) / T in [-1b-54, 1b-54]
}
