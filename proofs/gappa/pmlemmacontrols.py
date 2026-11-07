"""Write c_mixedn_half.g, c_mixeds_half.g and c_sum_half.g: each lemma of
464.35 (mixedn.g, mixeds.g, sum.g) with its opening comment replaced by one
line and each goal at [-1b-54, 1b-54], u / 2, which must fail.
Run as: python3 pmlemmacontrols.py"""
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))

for lemma in ("mixedn", "mixeds", "sum"):
    lines = open(lemma + ".g").read().split("\n")
    body = [line for line in lines if not line.startswith("#")]
    body = [line.replace("1b-53", "1b-54") for line in body]
    text = f"# {lemma}.g's control: the same at u / 2, which must fail.\n" + "\n".join(body)
    assert "1b-54" in text
    open(f"c_{lemma}_half.g", "w").write(text)
print("wrote c_mixedn_half.g, c_mixeds_half.g and c_sum_half.g")
