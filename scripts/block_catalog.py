#!/usr/bin/env python3
"""
Block Catalog & Snapshot CLI for Low-Cube

Usage:
    ./scripts/block_catalog.py list
    ./scripts/block_catalog.py list --forms
    ./scripts/block_catalog.py list --materials
    ./scripts/block_catalog.py render <block_id> [output_path.png]
    ./scripts/block_catalog.py render-all [output_directory]
"""

import sys
import os
import json
import math
import subprocess
from PIL import Image

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DOCS_IMAGES_DIR = os.path.join(REPO_ROOT, "docs", "images", "blocks")

def normalize(v):
    l = math.sqrt(sum(x*x for x in v))
    if l < 1e-6:
        return (0.0, 1.0, 0.0)
    return tuple(x / l for x in v)

def cross(a, b):
    return (
        a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0]
    )

def dot(a, b):
    return sum(x*y for x, y in zip(a, b))

def load_catalog_data():
    """Extract block geometry from world crate export_blocks example."""
    res = subprocess.run(
        ["cargo", "run", "--quiet", "--example", "export_blocks"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=True
    )
    return json.loads(res.stdout)

def render_block_image(block_entry, output_path, size=512, supersample=2):
    """Render a single block with isometric projection and daylight shading."""
    render_size = size * supersample
    scale = render_size * 0.42

    target = (0.5, 0.5, 0.5)
    dist = 3.5
    elev = math.radians(28.0)
    azim = math.radians(35.0)

    eye_x = target[0] - dist * math.cos(elev) * math.sin(azim)
    eye_y = target[1] + dist * math.sin(elev)
    eye_z = target[2] - dist * math.cos(elev) * math.cos(azim)
    eye = (eye_x, eye_y, eye_z)

    fwd = normalize((target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]))
    world_up = (0.0, 1.0, 0.0)
    right = normalize(cross(fwd, world_up))
    up = normalize(cross(right, fwd))
    light_dir = normalize((-0.35, 0.82, -0.45))

    quads = block_entry['quads']
    depth_buf = [999999.0] * (render_size * render_size)
    color_buf = [(0, 0, 0, 0)] * (render_size * render_size)

    for q in quads:
        verts = q['v']
        normal = q['n']
        base_col = q['c']

        norm = normalize(normal) if any(normal) else (0, 1, 0)
        cos_theta = dot(norm, light_dir)
        diff = max(0.0, cos_theta)
        if diff < 0.1:
            diff = max(diff, max(0.0, -cos_theta) * 0.7)

        intensity = 0.38 + 0.62 * diff
        r = int(min(255, max(0, base_col[0] * intensity * 255)))
        g = int(min(255, max(0, base_col[1] * intensity * 255)))
        b = int(min(255, max(0, base_col[2] * intensity * 255)))
        face_color = (r, g, b, 255)

        proj_v = []
        for v in verts:
            to_v = (v[0] - eye[0], v[1] - eye[1], v[2] - eye[2])
            depth = dot(to_v, fwd)
            dx = v[0] - target[0]
            dy = v[1] - target[1]
            dz = v[2] - target[2]
            sx = (dx * right[0] + dy * right[1] + dz * right[2]) * scale + render_size * 0.5
            sy = render_size * 0.5 - (dx * up[0] + dy * up[1] + dz * up[2]) * scale
            proj_v.append((sx, sy, depth))

        tris = [(proj_v[0], proj_v[1], proj_v[2]), (proj_v[0], proj_v[2], proj_v[3])]

        for t in tris:
            v0, v1, v2 = t
            min_x = max(0, int(math.floor(min(v0[0], v1[0], v2[0]))))
            max_x = min(render_size - 1, int(math.ceil(max(v0[0], v1[0], v2[0]))))
            min_y = max(0, int(math.floor(min(v0[1], v1[1], v2[1]))))
            max_y = min(render_size - 1, int(math.ceil(max(v0[1], v1[1], v2[1]))))

            denom = (v1[1] - v2[1]) * (v0[0] - v2[0]) + (v2[0] - v1[0]) * (v0[1] - v2[1])
            if abs(denom) < 1e-6:
                continue

            for py in range(min_y, max_y + 1):
                row_idx = py * render_size
                for px in range(min_x, max_x + 1):
                    w0 = ((v1[1] - v2[1]) * (px - v2[0]) + (v2[0] - v1[0]) * (py - v2[1])) / denom
                    w1 = ((v2[1] - v0[1]) * (px - v2[0]) + (v0[0] - v2[0]) * (py - v2[1])) / denom
                    w2 = 1.0 - w0 - w1

                    if w0 >= -1e-4 and w1 >= -1e-4 and w2 >= -1e-4:
                        depth = w0 * v0[2] + w1 * v1[2] + w2 * v2[2]
                        pixel_idx = row_idx + px
                        if depth < depth_buf[pixel_idx]:
                            depth_buf[pixel_idx] = depth
                            color_buf[pixel_idx] = face_color

    flat_bytes = bytearray()
    for c in color_buf:
        flat_bytes.extend(c)
    img = Image.frombytes('RGBA', (render_size, render_size), bytes(flat_bytes))
    img = img.resize((size, size), Image.Resampling.LANCZOS)
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    img.save(output_path)

def cmd_list(args):
    data = load_catalog_data()
    sub = args[0] if args else None

    if sub == "--forms":
        forms = {}
        for entry in data.values():
            sid = entry["shape_id"]
            if sid not in forms:
                forms[sid] = entry["title"].split(" - ")[-1]
        print(f"\n{'Shape-ID':<28} {'Form-Bezeichnung'}")
        print("-" * 55)
        for sid, name in forms.items():
            print(f"{sid:<28} {name}")
        print(f"\nGesamt: {len(forms)} Formen.")
        return

    if sub == "--materials":
        materials = {}
        for entry in data.values():
            mid = entry["mat_id"]
            if mid not in materials:
                materials[mid] = entry["title"].split(" - ")[0]
        print(f"\n{'Material-ID':<20} {'Material-Bezeichnung'}")
        print("-" * 50)
        for mid, name in materials.items():
            print(f"{mid:<20} {name}")
        print(f"\nGesamt: {len(materials)} Materialien.")
        return

    print(f"\n{'Block-ID':<36} {'Material':<25} {'Form'}")
    print("-" * 90)
    for key, entry in sorted(data.items()):
        mat = entry["title"].split(" - ")[0]
        form = entry["title"].split(" - ")[-1]
        print(f"{key:<36} {mat:<25} {form}")
    print(f"\nGesamt: {len(data)} Blöcke verfügbar.")

def cmd_render(args):
    if not args:
        print("Fehler: Bitte Block-ID angeben. (z.B. 'dirt_stair' oder './scripts/block_catalog.py list')")
        sys.exit(1)
    block_id = args[0]
    out_path = args[1] if len(args) > 1 else f"{block_id}.png"

    data = load_catalog_data()
    if block_id not in data:
        print(f"Fehler: Unbekannte Block-ID '{block_id}'. Siehe './scripts/block_catalog.py list'")
        sys.exit(1)

    print(f"Rendere '{block_id}' nach '{out_path}'...")
    render_block_image(data[block_id], out_path)
    print(f"Fertig: {out_path}")

def cmd_render_all(args):
    out_dir = args[0] if args else DOCS_IMAGES_DIR
    data = load_catalog_data()
    total = len(data)
    print(f"Rendere alle {total} Blöcke nach '{out_dir}'...")
    count = 0
    for key, entry in data.items():
        out_path = os.path.join(out_dir, f"{key}.png")
        render_block_image(entry, out_path)
        count += 1
        if count % 15 == 0 or count == total:
            print(f"  Fortschritt: {count}/{total} gerendert...")
    print(f"Alle {total} Blöcke erfolgreich gerendert!")

def main():
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help", "help"):
        print(__doc__)
        return

    cmd = sys.argv[1]
    args = sys.argv[2:]

    if cmd == "list":
        cmd_list(args)
    elif cmd == "render":
        cmd_render(args)
    elif cmd == "render-all":
        cmd_render_all(args)
    else:
        print(f"Unbekannter Befehl '{cmd}'. Siehe --help.")
        sys.exit(1)

if __name__ == "__main__":
    main()
