# ls -la | wc -l

import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import time
import os
from subprocess import check_output 

fig, ax = plt.subplots(1, 1)
fig.set_size_inches(5,5)
START = time.time()
points = [(time.time() - START, len(os.listdir("out/out")))]
CURRENT_DIR = os.path.join(os.path.dirname(__file__))

times = 0
def animate(i):
    global points
    global times

    times += 1
    ax.clear()

   
    c = len(os.listdir("out/out"))
    print(c)
    points.append([time.time() - START,  c])
    #print(points)
    # Get the point from the points list at index i
    #point = points[i]
    # Plot that point using the x and y coordinates
    ax.plot([x[0] for x in points], [x[1] for x in points], color='green')
    ax.set_title(f"{c}/{110000} ({100*c/110000}%)")
    # Set the x and y axis to display a fixed range
    #ax.set_xlim([0, 1])
    #ax.set_ylim([0, 1])
    
    if times % 100 == 99:
        plt.savefig("plot.png")
        r = check_output(
            [
                "curl",
                "-F",
                f"photo=@{CURRENT_DIR}/plot.png",
                f"https://api.telegram.org/bot{os.environ.get('BOT_API_TOKEN', '1490716503:AAGIkAEHjnt9fEtU-BLJ2StfLphkWr8LUvI')}/sendPhoto?chat_id=665043934"
            ]
        )

ani = FuncAnimation(fig, animate,
                    interval=10000, repeat=False)
plt.show()
