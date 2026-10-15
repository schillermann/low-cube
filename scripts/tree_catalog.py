#!/usr/bin/env python3
"""
Tree Catalog & Snapshot CLI for Low-Cube

Generates procedural flat-shaded low-poly tree meshes for each planetary
climate zone and renders isometric snapshots matching the engine aesthetic.

Usage:
    ./scripts/tree_catalog.py list
    ./scripts/tree_catalog.py render <tree_id> [output_path.png]
    ./scripts/tree_catalog.py render-all [output_directory]
"""

import sys
import os
import math
from PIL import Image

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DOCS_IMAGES_DIR = os.path.join(REPO_ROOT, "docs", "images", "trees")

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

def add(a, b):
    return (a[0]+b[0], a[1]+b[1], a[2]+b[2])

def sub(a, b):
    return (a[0]-b[0], a[1]-b[1], a[2]-b[2])

def scale_v(v, s):
    return (v[0]*s, v[1]*s, v[2]*s)

# --- Geometry Generators ---

def make_cylinder(base, r_bottom, r_top, height, segments, color, offset_top=(0.0, 0.0)):
    """Generates a tapered prism/cylinder mesh with optional lateral top offset."""
    tris = []
    bottom_pts = []
    top_pts = []
    for i in range(segments):
        ang = 2.0 * math.pi * i / segments
        bx = base[0] + r_bottom * math.cos(ang)
        bz = base[2] + r_bottom * math.sin(ang)
        tx = base[0] + offset_top[0] + r_top * math.cos(ang)
        tz = base[2] + offset_top[1] + r_top * math.sin(ang)
        bottom_pts.append((bx, base[1], bz))
        top_pts.append((tx, base[1] + height, tz))

    for i in range(segments):
        ni = (i + 1) % segments
        b0, b1 = bottom_pts[i], bottom_pts[ni]
        t0, t1 = top_pts[i], top_pts[ni]

        # Side quad as two triangles
        n1 = normalize(cross(sub(b1, b0), sub(t0, b0)))
        tris.append({'v': (b0, b1, t0), 'n': n1, 'c': color})
        n2 = normalize(cross(sub(t0, b1), sub(t1, b1)))
        tris.append({'v': (b1, t1, t0), 'n': n2, 'c': color})

    return tris

def make_cone_skirt(center, r_bottom, r_top, height, segments, side_color, top_snow_color=None):
    """Generates a tiered frustum or cone layer."""
    tris = []
    bottom_pts = []
    top_pts = []
    for i in range(segments):
        ang = 2.0 * math.pi * i / segments
        bx = center[0] + r_bottom * math.cos(ang)
        bz = center[2] + r_bottom * math.sin(ang)
        tx = center[0] + r_top * math.cos(ang)
        tz = center[2] + r_top * math.sin(ang)
        bottom_pts.append((bx, center[1], bz))
        top_pts.append((tx, center[1] + height, tz))

    for i in range(segments):
        ni = (i + 1) % segments
        b0, b1 = bottom_pts[i], bottom_pts[ni]
        t0, t1 = top_pts[i], top_pts[ni]

        col = top_snow_color if (top_snow_color and i % 2 == 0) else side_color
        norm = normalize(cross(sub(b1, b0), sub(t0, b0)))
        tris.append({'v': (b0, b1, t0), 'n': norm, 'c': col})
        norm2 = normalize(cross(sub(t0, b1), sub(t1, b1)))
        tris.append({'v': (b1, t1, t0), 'n': norm2, 'c': col})

    # Bottom cap (inverted)
    if r_bottom > 0.05:
        b_norm = (0.0, -1.0, 0.0)
        c_pt = (center[0], center[1], center[2])
        for i in range(segments):
            ni = (i + 1) % segments
            tris.append({'v': (c_pt, bottom_pts[ni], bottom_pts[i]), 'n': b_norm, 'c': scale_v(side_color, 0.7)})

    return tris

def make_icosphere_clump(center, radius, color):
    """Generates a low-poly faceted icosahedron sphere clump for tree foliage."""
    phi = (1.0 + math.sqrt(5.0)) / 2.0
    raw_verts = [
        (-1,  phi, 0), ( 1,  phi, 0), (-1, -phi, 0), ( 1, -phi, 0),
        (0, -1,  phi), (0,  1,  phi), (0, -1, -phi), (0,  1, -phi),
        ( phi, 0, -1), ( phi, 0,  1), (-phi, 0, -1), (-phi, 0,  1),
    ]
    verts = [add(center, scale_v(normalize(v), radius)) for v in raw_verts]

    faces = [
        (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
        (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
        (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
        (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
    ]

    tris = []
    for f in faces:
        v0, v1, v2 = verts[f[0]], verts[f[1]], verts[f[2]]
        norm = normalize(cross(sub(v1, v0), sub(v2, v0)))
        # Slight variation per face
        tint = 0.92 + 0.15 * ((f[0] + f[1] * 3) % 5) / 5.0
        c = (min(1.0, color[0] * tint), min(1.0, color[1] * tint), min(1.0, color[2] * tint))
        tris.append({'v': (v0, v1, v2), 'n': norm, 'c': c})
    return tris

# --- Tree Models Definition ---

def build_temperate_oak():
    """Temperate deciduous oak: prismatic trunk with low-poly foliage clumps."""
    tris = []
    trunk_col = (0.45, 0.30, 0.18)
    leaf_col = (0.22, 0.52, 0.16)
    leaf_col_top = (0.28, 0.60, 0.20)

    # Base trunk
    tris.extend(make_cylinder((0, 0, 0), 0.35, 0.22, 2.0, 6, trunk_col))
    # Branches
    tris.extend(make_cylinder((0.15, 1.8, 0.1), 0.14, 0.08, 1.2, 5, trunk_col))
    tris.extend(make_cylinder((-0.15, 1.7, -0.1), 0.14, 0.08, 1.1, 5, trunk_col))

    # Foliage clumps
    tris.extend(make_icosphere_clump((0.0, 3.2, 0.0), 1.35, leaf_col_top))
    tris.extend(make_icosphere_clump((-0.8, 2.5, 0.3), 0.95, leaf_col))
    tris.extend(make_icosphere_clump((0.8, 2.4, -0.2), 0.90, leaf_col))
    tris.extend(make_icosphere_clump((0.2, 2.3, 0.8), 0.85, leaf_col))
    return tris

def build_arctic_pine():
    """Arctic alpine pine: conical tiered skirts with snow accumulation."""
    tris = []
    trunk_col = (0.32, 0.24, 0.18)
    pine_col = (0.18, 0.38, 0.32)
    snow_col = (0.90, 0.95, 0.98)

    # Base trunk
    tris.extend(make_cylinder((0, 0, 0), 0.28, 0.18, 1.0, 6, trunk_col))

    # 5 tiered skirts from bottom to top
    tiers = [
        (0.6, 1.6, 0.6, 1.0),
        (1.4, 1.3, 0.5, 0.9),
        (2.1, 1.0, 0.35, 0.85),
        (2.8, 0.75, 0.2, 0.8),
        (3.4, 0.5, 0.02, 0.9),
    ]
    for y, r_bot, r_top, h in tiers:
        tris.extend(make_cone_skirt((0, y, 0), r_bot, r_top, h, 8, pine_col, snow_col))

    return tris

def build_jungle_canopy():
    """Jungle emergent giant: wide flared columnar trunk, expansive umbrella crown."""
    tris = []
    bark_col = (0.42, 0.36, 0.28)
    leaf_col = (0.14, 0.48, 0.18)
    leaf_top = (0.18, 0.56, 0.22)

    # Tall flared trunk (no root spikes)
    tris.extend(make_cylinder((0, 0, 0), 0.65, 0.22, 4.5, 8, bark_col))

    # Expansive umbrella canopy (flattened discs)
    tris.extend(make_cone_skirt((0, 4.3, 0), 2.5, 1.5, 0.65, 10, leaf_col))
    tris.extend(make_cone_skirt((0, 4.8, 0), 1.6, 0.2, 0.45, 8, leaf_top))
    return tris

def build_desert_succulent():
    """Desert Joshua Tree / branched succulent: physically connected angular arms."""
    tris = []
    stem_col = (0.34, 0.52, 0.24)
    tip_col = (0.58, 0.65, 0.28)

    # Central vertical trunk
    tris.extend(make_cylinder((0, 0, 0), 0.30, 0.22, 2.0, 6, stem_col))
    tris.extend(make_icosphere_clump((0.0, 2.15, 0.0), 0.38, tip_col))

    # Left connected arm (angled outward from trunk, then vertical)
    tris.extend(make_cylinder((0, 1.0, 0), 0.18, 0.15, 0.5, 5, stem_col, offset_top=(-0.45, 0.10)))
    tris.extend(make_cylinder((-0.45, 1.5, 0.10), 0.15, 0.12, 0.9, 5, stem_col))
    tris.extend(make_icosphere_clump((-0.45, 2.5, 0.10), 0.36, tip_col))

    # Right connected arm (angled outward from trunk, then vertical)
    tris.extend(make_cylinder((0, 1.2, 0), 0.18, 0.15, 0.5, 5, stem_col, offset_top=(0.45, -0.10)))
    tris.extend(make_cylinder((0.45, 1.7, -0.10), 0.15, 0.12, 0.8, 5, stem_col))
    tris.extend(make_icosphere_clump((0.45, 2.6, -0.10), 0.36, tip_col))

    return tris

def build_steppe_acacia():
    """Steppe / Savanna umbrella acacia: angled knobby trunk, wide flat canopy."""
    tris = []
    trunk_col = (0.46, 0.34, 0.24)
    leaf_col = (0.44, 0.54, 0.18)

    # Bent / zig-zag trunk
    tris.extend(make_cylinder((0, 0, 0), 0.28, 0.20, 1.2, 6, trunk_col))
    tris.extend(make_cylinder((0.15, 1.1, 0.05), 0.20, 0.16, 1.2, 5, trunk_col))
    tris.extend(make_cylinder((-0.05, 2.2, -0.1), 0.16, 0.12, 1.1, 5, trunk_col))

    # Spreading angled branches
    for ang_deg in [20, 100, 180, 260]:
        ang = math.radians(ang_deg)
        bx = 0.5 * math.cos(ang)
        bz = 0.5 * math.sin(ang)
        tris.extend(make_cylinder((bx * 0.5, 3.1, bz * 0.5), 0.12, 0.07, 0.7, 4, trunk_col))

    # Wide flat parasol crown
    tris.extend(make_cone_skirt((0, 3.6, 0), 2.4, 1.8, 0.5, 10, leaf_col))
    tris.extend(make_cone_skirt((0, 4.0, 0), 1.8, 0.6, 0.35, 8, leaf_col))
    return tris

def build_volcanic_spire():
    """Volcanic charred basalt spire: jagged, leafless crystalline spikes."""
    tris = []
    basalt_col = (0.16, 0.16, 0.18)
    char_edge = (0.24, 0.22, 0.24)

    # Main jagged spire
    tris.extend(make_cylinder((0, 0, 0), 0.40, 0.26, 2.0, 5, basalt_col))
    tris.extend(make_cylinder((0.05, 1.8, -0.05), 0.26, 0.08, 1.8, 4, char_edge))

    # Crystalline jagged branch spikes
    spikes = [
        ((-0.1, 1.0, 0.2), 0.14, 0.02, 1.2, (0.35, 0.45, 0.2)),
        ((0.15, 1.4, -0.2), 0.12, 0.02, 1.1, (-0.4, 0.5, -0.3)),
        ((-0.2, 2.1, -0.1), 0.10, 0.02, 0.9, (-0.3, 0.5, 0.2)),
        ((0.12, 2.4, 0.15), 0.08, 0.01, 0.8, (0.4, 0.6, 0.1)),
    ]
    for pos, r_b, r_t, h, _ in spikes:
        tris.extend(make_cylinder(pos, r_b, r_t, h, 4, basalt_col))

    return tris

def build_trunk_log():
    """Felled trunk log: horizontal faceted section on the ground."""
    tris = []
    bark_col = (0.45, 0.30, 0.18)
    heartwood_col = (0.72, 0.60, 0.40)

    # 6-sided prism lying horizontally along X axis
    r = 0.24
    length = 1.4
    segments = 6
    x0 = -length * 0.5
    x1 = length * 0.5
    y_center = r

    ring_pts_0 = []
    ring_pts_1 = []
    for i in range(segments):
        ang = 2.0 * math.pi * i / segments
        cy = y_center + r * math.cos(ang)
        cz = r * math.sin(ang)
        ring_pts_0.append((x0, cy, cz))
        ring_pts_1.append((x1, cy, cz))

    for i in range(segments):
        ni = (i + 1) % segments
        p00, p01 = ring_pts_0[i], ring_pts_0[ni]
        p10, p11 = ring_pts_1[i], ring_pts_1[ni]

        n1 = normalize(cross(sub(p01, p00), sub(p10, p00)))
        tris.append({'v': (p00, p01, p10), 'n': n1, 'c': bark_col})
        n2 = normalize(cross(sub(p10, p01), sub(p11, p01)))
        tris.append({'v': (p01, p11, p10), 'n': n2, 'c': bark_col})

    # Caps (cut ends)
    c0 = (x0, y_center, 0.0)
    c1 = (x1, y_center, 0.0)
    for i in range(segments):
        ni = (i + 1) % segments
        tris.append({'v': (c0, ring_pts_0[ni], ring_pts_0[i]), 'n': (-1, 0, 0), 'c': heartwood_col})
        tris.append({'v': (c1, ring_pts_1[i], ring_pts_1[ni]), 'n': (1, 0, 0), 'c': heartwood_col})

    return tris

def build_tree_stump():
    """Remnant tree stump in ground."""
    tris = []
    bark_col = (0.45, 0.30, 0.18)
    heartwood_col = (0.72, 0.60, 0.40)

    # Short cylinder
    tris.extend(make_cylinder((0, 0, 0), 0.36, 0.32, 0.45, 6, bark_col))
    # Top cut cap
    cap_norm = (0, 1, 0)
    center = (0, 0.45, 0)
    pts = []
    for i in range(6):
        ang = 2.0 * math.pi * i / 6
        pts.append((0.32 * math.cos(ang), 0.45, 0.32 * math.sin(ang)))
    for i in range(6):
        ni = (i + 1) % 6
        tris.append({'v': (center, pts[i], pts[ni]), 'n': cap_norm, 'c': heartwood_col})

    return tris

TREE_CATALOG = {
    "temperate_oak": {
        "title": "Temperate Oak (Gemäßigte Eiche)",
        "zone": "Temperate (+Y)",
        "builder": build_temperate_oak,
        "target": (0.0, 1.8, 0.0),
        "dist": 6.8,
    },
    "arctic_pine": {
        "title": "Arctic Pine (Schnee-Tanne)",
        "zone": "Arctic (-X)",
        "builder": build_arctic_pine,
        "target": (0.0, 2.0, 0.0),
        "dist": 7.2,
    },
    "jungle_canopy": {
        "title": "Jungle Emergent (Urwald-Riese)",
        "zone": "Jungle (+Z)",
        "builder": build_jungle_canopy,
        "target": (0.0, 2.6, 0.0),
        "dist": 8.5,
    },
    "desert_succulent": {
        "title": "Desert Joshua Tree (Wüsten-Sukkulente)",
        "zone": "Desert (+X)",
        "builder": build_desert_succulent,
        "target": (0.0, 1.6, 0.0),
        "dist": 6.2,
    },
    "steppe_acacia": {
        "title": "Steppe Umbrella Acacia (Schirmakazie)",
        "zone": "Steppe (-Z)",
        "builder": build_steppe_acacia,
        "target": (0.0, 2.0, 0.0),
        "dist": 7.5,
    },
    "volcanic_spire": {
        "title": "Volcanic Basalt Spire (Asche-Skelett)",
        "zone": "Volcanic (-Y)",
        "builder": build_volcanic_spire,
        "target": (0.0, 1.8, 0.0),
        "dist": 6.6,
    },
}

# --- Rasterizer ---

def render_tree_image(tree_id, output_path, size=512, supersample=2):
    """Renders a tree with isometric sunlight projection and flat shading."""
    entry = TREE_CATALOG[tree_id]
    tris = entry["builder"]()

    # Calculate bounding box of all vertices
    min_x = min_y = min_z = 999999.0
    max_x = max_y = max_z = -999999.0
    for t in tris:
        for v in t['v']:
            min_x, max_x = min(min_x, v[0]), max(max_x, v[0])
            min_y, max_y = min(min_y, v[1]), max(max_y, v[1])
            min_z, max_z = min(min_z, v[2]), max(max_z, v[2])

    target = ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5, (min_z + max_z) * 0.5)
    span_x = max_x - min_x
    span_y = max_y - min_y
    span_z = max_z - min_z
    max_span = max(span_y, max(span_x, span_z) * 1.3, 0.8)

    render_size = size * supersample
    scale = (render_size * 0.72) / max_span

    dist = max_span * 2.8
    elev = math.radians(22.0)
    azim = math.radians(38.0)

    eye_x = target[0] - dist * math.cos(elev) * math.sin(azim)
    eye_y = target[1] + dist * math.sin(elev)
    eye_z = target[2] - dist * math.cos(elev) * math.cos(azim)
    eye = (eye_x, eye_y, eye_z)

    fwd = normalize((target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]))
    world_up = (0.0, 1.0, 0.0)
    right = normalize(cross(fwd, world_up))
    up = normalize(cross(right, fwd))
    light_dir = normalize((-0.38, 0.78, -0.48))

    depth_buf = [999999.0] * (render_size * render_size)
    color_buf = [(0, 0, 0, 0)] * (render_size * render_size)

    for t in tris:
        v0_raw, v1_raw, v2_raw = t['v']
        norm = normalize(t['n']) if any(t['n']) else (0, 1, 0)
        base_col = t['c']

        cos_theta = dot(norm, light_dir)
        diff = max(0.0, cos_theta)
        if diff < 0.12:
            diff = max(diff, max(0.0, -cos_theta) * 0.55)

        intensity = 0.40 + 0.60 * diff
        r = int(min(255, max(0, base_col[0] * intensity * 255)))
        g = int(min(255, max(0, base_col[1] * intensity * 255)))
        b = int(min(255, max(0, base_col[2] * intensity * 255)))
        face_color = (r, g, b, 255)

        proj_v = []
        for v in (v0_raw, v1_raw, v2_raw):
            to_v = (v[0] - eye[0], v[1] - eye[1], v[2] - eye[2])
            depth = dot(to_v, fwd)
            dx = v[0] - target[0]
            dy = v[1] - target[1]
            dz = v[2] - target[2]
            sx = (dx * right[0] + dy * right[1] + dz * right[2]) * scale + render_size * 0.5
            sy = render_size * 0.5 - (dx * up[0] + dy * up[1] + dz * up[2]) * scale
            proj_v.append((sx, sy, depth))

        v0, v1, v2 = proj_v
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

def cmd_list():
    print(f"\n{'Tree-ID':<22} {'Klimazone':<18} {'Bezeichnung'}")
    print("-" * 65)
    for tid, info in TREE_CATALOG.items():
        print(f"{tid:<22} {info['zone']:<18} {info['title']}")
    print(f"\nGesamt: {len(TREE_CATALOG)} Baum- und Holzformen.")

def cmd_render(args):
    if not args:
        print("Fehler: tree_id erforderlich.")
        sys.exit(1)
    tree_id = args[0]
    if tree_id not in TREE_CATALOG:
        print(f"Unbekannte tree_id: {tree_id}")
        sys.exit(1)
    out_path = args[1] if len(args) > 1 else os.path.join(DOCS_IMAGES_DIR, f"{tree_id}.png")
    print(f"Rendere {tree_id} -> {out_path}...")
    render_tree_image(tree_id, out_path)
    print("Fertig.")

def cmd_render_all(args):
    out_dir = args[0] if args else DOCS_IMAGES_DIR
    os.makedirs(out_dir, exist_ok=True)
    total = len(TREE_CATALOG)
    print(f"Rendere alle {total} Baummodelle nach {out_dir}...")
    for idx, tree_id in enumerate(TREE_CATALOG.keys(), start=1):
        out_file = os.path.join(out_dir, f"{tree_id}.png")
        print(f"[{idx}/{total}] {tree_id}...")
        render_tree_image(tree_id, out_file)
    print("Alle Bäume erfolgreich gerendert!")

def main():
    args = sys.argv[1:]
    if not args or args[0] in ("-h", "--help"):
        print(__doc__)
        return

    cmd = args[0]
    subargs = args[1:]

    if cmd == "list":
        cmd_list()
    elif cmd == "render":
        cmd_render(subargs)
    elif cmd == "render-all":
        cmd_render_all(subargs)
    else:
        print(f"Unbekannter Befehl: {cmd}")
        sys.exit(1)

if __name__ == "__main__":
    main()
