#!/usr/bin/python3
import itertools
import io
import sys

# 'a92b4efc'
with open(sys.argv[1], 'rb') as f:
    offset = 0
    last_four_bytes = 0
    for buffer in iter(lambda: f.read(1048576), b''):
        for byte in buffer:
            last_four_bytes = last_four_bytes >> 8 | (byte << 24)
            if last_four_bytes == 0xa92b4efc:
                print("hit at byte {}".format(offset - 3))
            offset += 1
