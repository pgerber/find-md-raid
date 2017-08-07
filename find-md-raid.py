#!/usr/bin/python3
import itertools
import io
import sys

with open(sys.argv[1], 'rb') as f, io.BufferedReader(f) as f:
    offset = 0
    for buffer in iter(lambda: f.read(512), b''):
        if buffer.startswith(b'\xfc\x4e\x2b\xa9'):
                print("hit at byte {}".format(offset))
        offset += 512
