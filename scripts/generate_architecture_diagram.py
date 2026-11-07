import os
import math
from PIL import Image, ImageDraw, ImageFont

WIDTH = 1920
HEIGHT = 740
OUTPUT_PATH = "D:/PROJECTS/LoomDB/assets/architecture-diagram.png"

# Fonts
font_header = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 28)
font_card_title = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 19)
font_card_body = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 14)
font_api = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 12)
font_label = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 11)
font_novelty = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 12)
font_novelty_text = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 13)

# Palette
BG = (8, 10, 15)
GRID = (14, 18, 25)
SURFACE = (17, 22, 32)
CARD_BG = (22, 28, 40)
BORDER = (38, 48, 68)
AMBER = (249, 115, 22)
AMBER_DIM = (140, 65, 12)
CYAN = (56, 189, 248)
GREEN = (34, 197, 94)
WHITE = (245, 248, 252)
SECONDARY = (145, 160, 180)
MUTED = (95, 110, 132)


def rounded_rect(draw, box, radius, fill, outline=None, width=1):
    x0, y0, x1, y1 = box
    r = radius
    draw.pieslice([x0, y0, x0+2*r, y0+2*r], 180, 270, fill=fill, outline=outline, width=width)
    draw.pieslice([x1-2*r, y0, x1, y0+2*r], 270, 360, fill=fill, outline=outline, width=width)
    draw.pieslice([x0, y1-2*r, x0+2*r, y1], 90, 180, fill=fill, outline=outline, width=width)
    draw.pieslice([x1-2*r, y1-2*r, x1, y1], 0, 90, fill=fill, outline=outline, width=width)
    draw.rectangle([x0+r, y0, x1-r, y1], fill=fill)
    draw.rectangle([x0, y0+r, x0+r, y1-r], fill=fill)
    draw.rectangle([x1-r, y0+r, x1, y1-r], fill=fill)
    if outline:
        draw.line([(x0+r, y0), (x1-r, y0)], fill=outline, width=width)
        draw.line([(x0+r, y1), (x1-r, y1)], fill=outline, width=width)
        draw.line([(x0, y0+r), (x0, y1-r)], fill=outline, width=width)
        draw.line([(x1, y0+r), (x1, y1-r)], fill=outline, width=width)


def draw_grid(draw):
    for x in range(0, WIDTH, 48):
        draw.line([(x, 0), (x, HEIGHT)], fill=GRID, width=1)
    for y in range(0, HEIGHT, 48):
        draw.line([(0, y), (WIDTH, y)], fill=GRID, width=1)


def generate_diagram():
    img = Image.new("RGB", (WIDTH, HEIGHT), BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)

    center_x = WIDTH // 2

    # ─── TOP: Unified Storage Pager ───
    pager_w = 560
    pager_h = 72
    pager_x = center_x - pager_w // 2
    pager_y = 40

    for g in range(18, 0, -2):
        c = tuple(int(BG[j] + (AMBER[j] - BG[j]) * 0.06 * (1 - g / 18)) for j in range(3))
        draw.rectangle([pager_x-g, pager_y-g, pager_x+pager_w+g, pager_y+pager_h+g], fill=c)

    rounded_rect(draw, [pager_x, pager_y, pager_x+pager_w, pager_y+pager_h], 12, fill=(30, 22, 16), outline=AMBER, width=2)
    ix = pager_x + 28
    iy = pager_y + 18
    for off in [0, 5, 10]:
        draw.rectangle([ix+off, iy+14-off, ix+24+off, iy+34-off], fill=CARD_BG, outline=AMBER, width=1)
    draw.text((pager_x+72, pager_y+12), "LoomDB Unified Storage Pager", font=font_header, fill=WHITE)
    draw.text((pager_x+72, pager_y+46), "Single-file .loom memory-mapped container with virtual paging layer", font=font_card_body, fill=SECONDARY)

    # ─── TRUNK ───
    trunk_top = pager_y + pager_h
    trunk_fork_y = trunk_top + 55
    draw.line([(center_x, trunk_top), (center_x, trunk_fork_y)], fill=AMBER_DIM, width=3)

    # ─── CARDS ───
    cards = [
        {
            "title": "Forkable COW Snapshots",
            "num": "1", "color": AMBER,
            "api": ["fork()", "checkout()", "discard()", "merge()"],
            "lines": ["Page-level copy-on-write branching", "O(1) allocation, shared base roots", "Zero-cost instant snapshots"],
            "novelty": "Cheap branching as first-class storage"
        },
        {
            "title": "Deterministic Flight Recorder",
            "num": "2", "color": CYAN,
            "api": ["record_tx()", "replay(tx_id)", "verify_checksum()"],
            "lines": ["WAL captures timestamps & seeds", "Bit-for-bit replay of any transaction", "Checksum-verified state reproduction"],
            "novelty": "Reproducible embedded DB behavior"
        },
        {
            "title": "Hard Budget Governor",
            "num": "3", "color": GREEN,
            "api": ["_sys_budget", "max_ram_bytes", "max_wal_bytes"],
            "lines": ["User-declared resource caps enforced", "Backpressure & hard error responses", "Zero silent OOM process crashes"],
            "novelty": "Enforceable contract for edge devices"
        },
        {
            "title": "Zero-Copy Stable ABI",
            "num": "4", "color": WHITE,
            "api": ["mmap_reader()", "typed_view()", "seqlock_gen()"],
            "lines": ["Stable binary container byte layout", "Lock-free readers + single-writer", "Typed views over mmap'd pages"],
            "novelty": "Multi-process zero-copy with stable ABI"
        },
        {
            "title": "OPFS Multi-Tab Coordination",
            "num": "5", "color": AMBER,
            "api": ["Web Locks API", "BroadcastChannel", "heartbeat()"],
            "lines": ["Browser-native writer / multi-reader", "Cooperative write leases for Wasm", "Heartbeat crash recovery on tab close"],
            "novelty": "No multi-tab corruption, no server"
        }
    ]

    n = len(cards)
    card_w = 330
    card_h = 325
    gap = 24
    total_w = n * card_w + (n - 1) * gap
    start_x = (WIDTH - total_w) // 2
    card_y = trunk_fork_y + 58

    bar_l = start_x + card_w // 2
    bar_r = start_x + (n - 1) * (card_w + gap) + card_w // 2
    draw.line([(bar_l, trunk_fork_y), (bar_r, trunk_fork_y)], fill=AMBER_DIM, width=3)

    for i, cd in enumerate(cards):
        cx = start_x + i * (card_w + gap)
        ccx = cx + card_w // 2
        col = cd["color"]

        # Connector
        draw.line([(ccx, trunk_fork_y), (ccx, card_y - 6)], fill=AMBER_DIM, width=2)
        draw.polygon([(ccx-5, card_y-8), (ccx+5, card_y-8), (ccx, card_y)], fill=AMBER_DIM)
        draw.ellipse([ccx-5, trunk_fork_y-5, ccx+5, trunk_fork_y+5], fill=col)

        # Card
        rounded_rect(draw, [cx, card_y, cx+card_w, card_y+card_h], 10, fill=CARD_BG, outline=BORDER, width=1)
        draw.rectangle([cx+1, card_y+1, cx+card_w-1, card_y+6], fill=col)

        # Number badge
        bx, by = cx + 14, card_y + 18
        draw.ellipse([bx, by, bx+28, by+28], fill=col)
        nb = font_card_title.getbbox(cd["num"])
        draw.text((bx + 14 - (nb[2]-nb[0])//2, by+3), cd["num"], font=font_card_title, fill=BG)

        # Title
        draw.text((cx+52, card_y+20), cd["title"], font=font_card_title, fill=WHITE)

        # Separator
        draw.line([(cx+14, card_y+56), (cx+card_w-14, card_y+56)], fill=BORDER, width=1)

        # API pills
        ay = card_y + 68
        ax = cx + 14
        for api in cd["api"]:
            ab = font_api.getbbox(api)
            aw = (ab[2]-ab[0]) + 14
            if ax + aw > cx + card_w - 10:
                ax = cx + 14
                ay += 26
            draw.rectangle([ax, ay, ax+aw, ay+22], fill=SURFACE, outline=BORDER, width=1)
            draw.text((ax+7, ay+4), api, font=font_api, fill=col)
            ax += aw + 6

        # Description
        dy = card_y + 136
        for line in cd["lines"]:
            draw.text((cx+16, dy), line, font=font_card_body, fill=SECONDARY)
            dy += 20

        # Novelty footer
        ny = card_y + card_h - 48
        draw.line([(cx+14, ny-6), (cx+card_w-14, ny-6)], fill=BORDER, width=1)
        draw.text((cx+16, ny), "Novelty", font=font_novelty, fill=col)
        draw.text((cx+16, ny+18), cd["novelty"], font=font_novelty_text, fill=MUTED)

    # ─── FOOTER: 6th Primitive ───
    fy = card_y + card_h + 18
    fh = 42
    rounded_rect(draw, [start_x, fy, start_x+total_w, fy+fh], 8, fill=SURFACE, outline=BORDER, width=1)
    draw.ellipse([start_x+16, fy+9, start_x+38, fy+33], fill=CYAN)
    draw.text((start_x+22, fy+11), "6", font=font_card_title, fill=BG)
    draw.text((start_x+48, fy+10), "Self-Queryable System Telemetry", font=font_card_title, fill=WHITE)
    draw.text((start_x+335, fy+12), "SELECT * FROM _sys_telemetry;", font=font_api, fill=CYAN)
    draw.text((start_x+610, fy+12), "Latency percentiles, cache hits, WAL volume, page faults, and query counts as a native data modality", font=font_card_body, fill=SECONDARY)

    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)
    img.save(OUTPUT_PATH, "PNG", optimize=True)
    print(f"Diagram generated: {OUTPUT_PATH} ({WIDTH}x{HEIGHT})")

if __name__ == "__main__":
    generate_diagram()
