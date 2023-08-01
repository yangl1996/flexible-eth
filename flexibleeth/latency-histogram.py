import sys
import re
import argparse
import math

argParser = argparse.ArgumentParser()
argParser.add_argument("-n", help="number of bins", type=int, default=50)
args = argParser.parse_args()

e = re.compile(r"t=([0-9]+) tip=([0-9]+),")

confirmed_tip = None

data = []
min_d = None
max_d = None
bins = [0.0 for i in range(args.n)]

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

width = (max_d-min_d)/args.n

for d in data:
    idx = math.floor((d-min_d)/width)
    if idx == args.n:
        idx = args.n-1
    bins[idx] += 1/len(data)/width

for idx in range(args.n):
    print(min_d+idx*width, min_d+idx*width+width, bins[idx])
