# mixeds.g's control: the same at u / 2, which must fail.
@rnd = float<ieee_64, ne>;

q = rnd(P);

{
  |P| <= 1b-1022
->
  (q - P) / (|P| + 1b-1022) in [-1b-54, 1b-54]
}
