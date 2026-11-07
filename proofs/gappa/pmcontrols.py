"""Write pm_n's further controls, which must each fail: c_pm_n_l.g, pm_n.g
with L free down to 2^-1074, below the normals, and c_pm_n_34.g, pm_n.g at
34u, below the 34.73u the two averages reach together.
Run as: python3 controls.py"""
import os

import pointsub

os.chdir(os.path.dirname(os.path.abspath(__file__)))
zero, state = next((z, s) for z, s in pointsub.cases() if pointsub.name_of(z, s) == "pm_n")
open("c_pm_n_l.g", "w").write(pointsub.make(zero, state, lmin="1b-1074", part="both"))
open("c_pm_n_34.g", "w").write(
    pointsub.make(zero, state, "(out - Ex) / L in [-34b-53, 34b-53]", part="both"))
print("wrote c_pm_n_l.g and c_pm_n_34.g")
