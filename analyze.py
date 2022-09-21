# -*- coding: utf-8 -*-
import matplotlib.pyplot as plt
import matplotlib.cm as cm
from matplotlib.colors import LogNorm
import sqlite3

# Global constants
DATABASE_NAME = 'geo_end.xyz.db'

# Prepare figures
fig = plt.figure()
ax1 = fig.add_subplot(111)

# Import database
con = sqlite3.connect(DATABASE_NAME)
cur = con.cursor()
xys = [xy for xy in cur.execute("SELECT x, y FROM traj WHERE element='C'")]
xs = [xy[0] for xy in xys]
ys = [xy[1] for xy in xys]

# Draw plot
H1 = ax1.hist2d(xs, ys, bins=100, cmap=cm.jet, norm=LogNorm())
ax1.set_aspect("equal", adjustable="box")
# ax1.set_xlim(-15.0, 15.0)
# ax1.set_ylim(-15.0, 15.0)
fig.colorbar(H1[3], ax=ax1)
plt.show()