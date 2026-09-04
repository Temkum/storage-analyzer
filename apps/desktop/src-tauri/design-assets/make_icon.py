#!/usr/bin/env python3
"""
Generate the "System Analyzer" app icon (master 1024x1024 PNG).

Concept:  Analysis gauge  +  network pulse
  - slate-900 (#0f172a) rounded square  -> app's brand mark / sidebar
  - white gauge ring                    -> "analyzer" dial
  - blue->cyan->teal live arc (top)     -> disk/network activity telemetry
  - white needle into the arc           -> measurement / monitoring
  - 3-node network dot cluster          -> network utility

Rendered with 4x supersampling for crisp edges, then downscaled.
All color stops come from the web app's design tokens (Treemap palette +
DashboardLayout sidebar accent).
"""

from PIL import Image, ImageDraw

SIZE = 1024
SS = 4  # supersampling factor

# Brand / design tokens (from the web app)
SLATE = (15, 23, 42)        # #0f172a  brand mark / sidebar
WHITE = (248, 250, 252)     # #f8fafc  near-white text
BLUE = (37, 99, 235)        # #2563eb  active accent
CYAN = (6, 182, 212)        # #06b6d4
TEAL = (20, 184, 166)       # #14b8a6

CENTER = (SIZE / 2, SIZE / 2)


def lerp(a, b, t):
    return tuple(round(a[c] + (b[c] - a[c]) * t) for c in range(3))


def lerp3(colors, t):
    """Interpolate across a list of color stops by normalized t in [0,1]."""
    if t <= 0:
        return colors[0]
    if t >= 1:
        return colors[-1]
    n = len(colors) - 1
    f = t * n
    i = min(int(f), n - 1)
    local = f - i
    return lerp(colors[i], colors[i + 1], local)


def main():
    img = Image.new("RGBA", (SIZE * SS, SIZE * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    s = SS  # everything drawn in 1024-space scaled up

    def rc(xy, radius, fill):
        d.rounded_rectangle(
            [v * s for v in xy], radius=radius * s, fill=fill
        )

    def wedge(box, start, end, fill):
        d.pieslice(
            [v * s for v in box], start, end, fill=fill
        )

    def circle(c, r, fill):
        x, y = c
        d.ellipse([(x - r) * s, (y - r) * s, (x + r) * s, (y + r) * s], fill=fill)

    # --- 1. Background rounded square (slate) ---
    MARGIN = 20
    rc([MARGIN, MARGIN, SIZE - MARGIN, SIZE - MARGIN], 235, SLATE)

    # --- 2. Gauge ---------------------------------------------------------
    INNER = 250
    OUTER = 335

    # White ring: draw filled disc then punch the hole with slate.
    circle(CENTER, OUTER, WHITE)
    circle(CENTER, INNER, SLATE)

    # Live accent arc across the top (PIL: 210deg -> 330deg through 12 o'clock),
    # gradient blue -> cyan -> teal, drawn as thin wedge slices.
    A0, A1 = 210, 330
    STOPS = [BLUE, CYAN, TEAL]
    SLICES = 96
    for i in range(SLICES):
        t0 = i / SLICES
        t1 = (i + 1) / SLICES
        start = A0 + (A1 - A0) * t0
        end = A0 + (A1 - A0) * t1
        color = lerp3(STOPS, (t0 + t1) / 2)
        wedge(
            [CENTER[0] - OUTER, CENTER[1] - OUTER,
             CENTER[0] + OUTER, CENTER[1] + OUTER],
            start, end, color,
        )
    # Re-punch the hole so the arc only covers the ring band.
    circle(CENTER, INNER, SLATE)

    # --- 3. Needle (white) pointing into the top of the arc ----------------
    # A tapering needle from the pivot toward 12 o'clock.
    import math

    def needle(tip_deg, length, base_w, tip_w):
        a = math.radians(tip_deg)
        dx, dy = math.sin(a), -math.cos(a)  # y up in image space
        cx, cy = CENTER
        px, py = -dy, dx  # perpendicular
        tip = (cx + dx * length, cy + dy * length)
        b1 = (cx + dx * 40 + px * base_w, cy + dy * 40 + py * base_w)
        b2 = (cx + dx * 40 - px * base_w, cy + dy * 40 - py * base_w)
        t1 = (tip[0] + px * tip_w, tip[1] + py * tip_w)
        t2 = (tip[0] - px * tip_w, tip[1] - py * tip_w)
        d.polygon([v * s for v in b1 + b2 + t1 + t2], fill=WHITE)

    needle(0, length=222, base_w=26, tip_w=10)  # point to 12 o'clock

    # Pivot hub
    circle(CENTER, 24, WHITE)

    # --- 4. Network node cluster (subtle, inside the hole) -----------------
    # three dots joined by thin lines -> the "network" half of the utility
    a = math.radians(180)  # straight DOWN from center ((sin,-cos) -> (0,1))
    cx, cy = CENTER
    dx, dy = math.sin(a), -math.cos(a)
    basex, basey = cx + dx * 170, cy + dy * 170  # lower half of the dial hole
    nodes = [
        (basex - 46, basey + 34, BLUE),
        (basex, basey - 46, CYAN),
        (basex + 46, basey + 34, TEAL),
    ]
    # connecting lines (to a small center hub of the cluster)
    hub = (basex, basey)
    for (nx, ny, _) in nodes:
        d.line(
            [hub[0] * s, hub[1] * s, nx * s, ny * s],
            fill=(148, 163, 184, 200), width=round(4 * s),
        )
    for (nx, ny, color) in nodes:
        circle((nx, ny), 20, color)
    circle(hub, 11, (148, 163, 184))

    # --- Downsample with high-quality filter ------------------------------
    result = img.resize((SIZE, SIZE), Image.LANCZOS)
    out = "/home/tem/Documents/Workspace/system-analyzer/apps/desktop/src-tauri/design-assets/app-icon-1024.png"
    result.save(out, "PNG")
    print("wrote", out, result.size)


if __name__ == "__main__":
    main()