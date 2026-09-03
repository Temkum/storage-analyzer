#!/usr/bin/env python3
"""Sample key pixels of the generated icon to sanity-check the composition."""
import math
from PIL import Image

img = Image.open("app-icon-1024.png").convert("RGBA")
S = 1024
cx = cy = 512

def at(x, y):
    return img.getpixel((int(x), int(y)))

def ring_pt(deg, r):
    a = math.radians(deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))

# Radius band for the ring (INNER=250, OUTER=335), mid ~292
RMID = 266

checks = {
    "corner (20,20) transparent": at(20, 20),
    "far corner (5,200) transparent": at(5, 200),
    "slate bg left (150,512)": at(150, 512),
    "white ring bottom (512,804)": at(512, 804),
    "accent arc TOP (cyan-ish)": at(*ring_pt(270, RMID)),
    "accent arc LOWER-LEFT (blue-ish) 210deg": at(*ring_pt(222, RMID)),
    "accent arc LOWER-RIGHT (teal-ish) 330deg": at(*ring_pt(318, RMID)),
    "needle tip (512,290) white": at(512, 290),
    "pivot hub (512,512) white": at(512, 512),
    "node cluster cyan (512,636)": at(512, 636),
    "node cluster blue (466,716)": at(466, 716),
    "node cluster teal (558,716)": at(558, 716),
}
for k, v in checks.items():
    print(f"{k:42s} -> {v}")

# overall bounds
bbox = img.getbbox()
print("\ncontent bbox:", bbox, "size:", img.size)