import sys
import re
#import argparse
import math

#argParser = argparse.ArgumentParser()
#argParser.add_argument("-n", help="number of bins", type=int, default=50)
#args = argParser.parse_args()

e = re.compile(r"\(([0-9]+), ([0-9]+)\)")

confirmed_tip = None

data = []
min_d = None
max_d = None

for line in sys.stdin:
    res = e.search(line)
    if res:
        slot = int(res.group(1))
        tip = int(res.group(2))
        if not confirmed_tip is None:
            for i in range(confirmed_tip+1, tip+1):
                latency = slot - i
                data.append(latency)
                if min_d is None or min_d > latency:
                    min_d = latency
                if max_d is None or max_d < latency:
                    max_d = latency
        confirmed_tip = tip

n_bins = max_d-min_d+1
bins = [0.0 for i in range(n_bins)]
width = 1

for d in data:
    idx = d-min_d
    bins[idx] += 1/len(data)/width

for v in range(min_d):
    print("({}, {}, {})".format(v, v+width, 0.0))

for idx in range(n_bins):
    val = 0.0
    for i in range(0, idx+1):
        val += bins[i]
    print("({}, {}, {})".format(min_d+idx*width, min_d+idx*width+width, val))
