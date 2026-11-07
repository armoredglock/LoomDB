import os
import sys
import math
from PIL import Image, ImageDraw, ImageFont

WIDTH = 1920
HEIGHT = 1080
TOTAL_FRAMES = 600
FPS = 30

OUTPUT_DIR = "D:/PROJECTS/LoomDB/brag-output/work/frames"
os.makedirs(OUTPUT_DIR, exist_ok=True)

# Load fonts
try:
    font_title = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 68)
    font_hero = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 84)
    font_sub = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 32)
    font_sub_b = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 32)
    font_body = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 26)
    font_body_b = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 26)
    font_mono = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 24)
    font_mono_b = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 25)
    font_mono_lg = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 30)
    font_small = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 20)
    font_card_title = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 46)
except Exception as e:
    print("Fallback to default font:", e)
    font_title = font_hero = font_sub = font_sub_b = font_body = font_body_b = font_mono = font_mono_b = font_mono_lg = font_small = font_card_title = ImageFont.load_default()

# Colors
C_BG = (7, 8, 10)
C_CARBON = (12, 14, 19)
C_SURFACE = (18, 21, 28)
C_ELEVATED = (24, 29, 38)
C_BORDER = (31, 37, 50)
C_AMBER = (232, 89, 44)
C_AMBER_BRIGHT = (249, 115, 22)
C_CYAN = (56, 189, 248)
C_WHITE = (255, 255, 255)
C_PRIMARY = (234, 236, 241)
C_SECONDARY = (157, 166, 183)
C_MUTED = (100, 109, 128)

def draw_grid(draw):
    for x in range(0, WIDTH, 64):
        draw.line([(x, 0), (x, HEIGHT)], fill=(16, 20, 27), width=1)
    for y in range(0, HEIGHT, 64):
        draw.line([(0, y), (WIDTH, y)], fill=(16, 20, 27), width=1)

def ease_out(t):
    return 1 - (1 - t) ** 3

def ease_in_out(t):
    return 0.5 * (1 - math.cos(math.pi * t))

def render_scene_1(frame_idx):
    # Frames 0 - 89 (0.0s - 3.0s)
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = frame_idx / 90.0
    
    # Eyebrow
    draw.rectangle([140, 140, 520, 180], fill=(232, 89, 44, 40), outline=C_AMBER, width=1)
    draw.text((156, 148), "ENGINEERING PARADOX // 2026", font=font_small, fill=C_AMBER_BRIGHT)
    
    # Main Headline
    draw.text((140, 210), "POLYGLOT DATABASE SPRAWL", font=font_title, fill=C_WHITE)
    draw.text((140, 295), "IS SLOWING YOU DOWN.", font=font_title, fill=(240, 90, 45))
    
    # 4 Heavy Database Cards
    cards = [
        ("POSTGRESQL", "Relational ANSI Tables", "42 MB Idle RAM"),
        ("MONGODB", "JSON Document Store", "350 MB Idle RAM"),
        ("PINECONE", "Cloud Vector Search", "210 MB Idle RAM"),
        ("INFLUXDB", "Time-Series Telemetry", "85 MB Idle RAM")
    ]
    
    card_w = 380
    card_h = 160
    card_y = 440
    
    for i, (name, role, ram) in enumerate(cards):
        cx = 140 + i * (card_w + 35)
        # Staggered pop-in
        card_t = max(0, min(1, (frame_idx - 15 - i * 6) / 12.0))
        if card_t > 0:
            scale_y = int(card_y + (1 - ease_out(card_t)) * 40)
            draw.rectangle([cx, scale_y, cx + card_w, scale_y + card_h], fill=C_SURFACE, outline=C_BORDER, width=2)
            draw.text((cx + 24, scale_y + 24), name, font=font_body_b, fill=C_WHITE)
            draw.text((cx + 24, scale_y + 64), role, font=font_small, fill=C_SECONDARY)
            draw.text((cx + 24, scale_y + 104), ram, font=font_mono_b, fill=(248, 113, 113))
            
    # Bottom Warning Bar (Slams in at frame 50)
    if frame_idx >= 50:
        bar_t = min(1, (frame_idx - 50) / 12.0)
        alpha = ease_out(bar_t)
        by = int(660 + (1 - alpha) * 30)
        draw.rectangle([140, by, 140 + 4 * card_w + 3 * 35, by + 90], fill=(28, 14, 18), outline=(220, 38, 38), width=2)
        draw.text((170, by + 18), "OVERHEAD: 687 MB RAM IDLE · 4 RUNNING DAEMONS · CLUSTER SPRAWL", font=font_mono_lg, fill=(254, 202, 202))
        draw.text((170, by + 54), "All for a single developer project.", font=font_body, fill=C_MUTED)

    # Question Hook (Frame 68 onwards)
    if frame_idx >= 68:
        hook_t = min(1, (frame_idx - 68) / 10.0)
        hy = int(810 + (1 - ease_out(hook_t)) * 25)
        draw.text((140, hy), "WHAT IF ALL 5 PARADIGMS WERE ONE SINGLE FILE?", font=font_title, fill=C_AMBER_BRIGHT)
        
    return img

def render_scene_2(frame_idx):
    # Frames 90 - 209 (3.0s - 7.0s)
    rel_f = frame_idx - 90
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = rel_f / 120.0
    
    # Left Hero Column
    draw.rectangle([120, 120, 430, 160], fill=(232, 89, 44, 40), outline=C_AMBER, width=1)
    draw.text((136, 128), "INTRODUCING LOOMDB v0.1.0", font=font_small, fill=C_AMBER_BRIGHT)
    
    draw.text((120, 190), "LoomDB", font=font_hero, fill=C_WHITE)
    draw.text((120, 290), "Five Paradigms. One Portable Container.", font=font_sub_b, fill=C_AMBER)
    draw.text((120, 340), "Eliminate polyglot database sprawl. SQL, JSON, Vectors,\nand Time-Series woven into a single .loom file.", font=font_body, fill=C_SECONDARY)
    
    # 3 Spec Badges
    badges = [
        ("< 512 KB IDLE RAM", "Lowest memory footprint on Earth"),
        ("ZERO BACKGROUND THREADS", "Sleeps with zero CPU cycles when idle"),
        ("SINGLE PORTABLE FILE", "Everything encapsulated in ./data.loom")
    ]
    for i, (b_title, b_sub) in enumerate(badges):
        by = 460 + i * 115
        bt = max(0, min(1, (rel_f - 20 - i * 15) / 15.0))
        if bt > 0:
            bx = int(120 + (1 - ease_out(bt)) * 30)
            draw.rectangle([bx, by, bx + 640, by + 90], fill=C_SURFACE, outline=C_BORDER, width=2)
            draw.text((bx + 24, by + 16), b_title, font=font_mono_b, fill=C_WHITE)
            draw.text((bx + 24, by + 50), b_sub, font=font_body, fill=C_SECONDARY)
            
    # Right Column: Visual Paradigm Convergence Graphic
    rx = 860
    rw = 940
    ry = 120
    draw.rectangle([rx, ry, rx + rw, ry + 840], fill=C_CARBON, outline=C_BORDER, width=2)
    draw.text((rx + 36, ry + 36), "MULTI-MODAL PARADIGM CONVERGENCE", font=font_mono_b, fill=C_WHITE)
    draw.rectangle([rx + rw - 220, ry + 30, rx + rw - 36, ry + 66], fill=(24, 28, 38), outline=C_BORDER)
    draw.text((rx + rw - 200, ry + 38), "CONVERGED FABRIC", font=font_small, fill=C_PRIMARY)
    
    strands = [
        ("RELATIONAL", "ANSI SQL Tables, Primary Keys, MVCC", "B+TREE MMAP"),
        ("DOCUMENTS", "Schemaless JSON, Nested Field Paths", "BINARY JSON"),
        ("AI VECTORS", "High-Dimensional Cosine / L2 Nearest Neighbor", "AVX-512 SIMD"),
        ("TIME-SERIES", "Sensors & Telemetry High-Throughput Streams", "GORILLA DELTA"),
        ("GRAPH EDGES", "Adjacency Traversal & Relationship Mapping", "ZERO-COPY CSR")
    ]
    
    pulse_idx = int((rel_f / 15.0)) % 5
    for i, (tag, desc, badge) in enumerate(strands):
        sy = ry + 100 + i * 95
        is_active = (i == pulse_idx)
        border_col = C_AMBER if is_active else C_BORDER
        bg_col = (30, 20, 16) if is_active else C_SURFACE
        
        draw.rectangle([rx + 36, sy, rx + rw - 36, sy + 75], fill=bg_col, outline=border_col, width=2 if is_active else 1)
        draw.text((rx + 56, sy + 22), tag, font=font_mono_b, fill=C_AMBER_BRIGHT if is_active else C_AMBER)
        draw.text((rx + 240, sy + 24), desc, font=font_small, fill=C_WHITE if is_active else C_SECONDARY)
        
        # Crisp badge
        draw.rectangle([rx + rw - 230, sy + 18, rx + rw - 50, sy + 56], fill=(24, 30, 42), outline=C_AMBER if is_active else C_BORDER, width=1)
        draw.text((rx + rw - 215, sy + 25), badge, font=font_small, fill=C_AMBER_BRIGHT if is_active else C_WHITE)

    # Convergence Stream Arrow & Unified Output
    draw.text((rx + rw // 2 - 140, ry + 600), "↓ Unified Storage Virtual Pager ↓", font=font_mono, fill=C_MUTED)
    
    target_y = ry + 660
    draw.rectangle([rx + 36, target_y, rx + rw - 36, target_y + 120], fill=C_SURFACE, outline=C_AMBER, width=2)
    draw.text((rx + 60, target_y + 24), "UNIFIED STORAGE CONTAINER", font=font_small, fill=C_MUTED)
    draw.text((rx + 60, target_y + 54), "Single File Container (.loom)", font=font_card_title, fill=C_WHITE)
    
    # RAM Footprint Pill
    draw.rectangle([rx + rw - 260, target_y + 36, rx + rw - 60, target_y + 86], fill=(24, 30, 42), outline=C_AMBER, width=2)
    draw.text((rx + rw - 235, target_y + 48), "< 512 KB RAM", font=font_mono_b, fill=C_AMBER_BRIGHT)

    return img

def render_scene_3(frame_idx):
    # Frames 210 - 359 (7.0s - 12.0s)
    rel_f = frame_idx - 210
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    # Eyebrow & Headline
    draw.text((120, 80), "LIVE COMMAND EXECUTION // ZERO NETWORK LATENCY", font=font_small, fill=C_AMBER_BRIGHT)
    draw.text((120, 115), "Interactive Terminal Shell", font=font_title, fill=C_WHITE)
    
    # Terminal Window Outer
    tx = 120
    ty = 190
    tw = 1680
    th = 780
    
    draw.rectangle([tx, ty, tx + tw, ty + th], fill=(10, 12, 16), outline=C_BORDER, width=2)
    # Titlebar
    draw.rectangle([tx, ty, tx + tw, ty + 50], fill=(17, 20, 27))
    draw.ellipse([tx + 20, ty + 18, tx + 34, ty + 32], fill=(255, 95, 86))
    draw.ellipse([tx + 44, ty + 18, tx + 58, ty + 32], fill=(255, 189, 46))
    draw.ellipse([tx + 68, ty + 18, tx + 82, ty + 32], fill=(39, 201, 63))
    
    draw.text((tx + tw // 2 - 160, ty + 14), "loom — ./data.loom [Micro Profile]", font=font_mono, fill=C_MUTED)
    
    # Terminal Content
    cx = tx + 36
    cy = ty + 80
    
    draw.text((cx, cy), "[LOOM_DB] Interactive Shell v0.1.0", font=font_mono_b, fill=C_CYAN)
    draw.text((cx, cy + 35), "Mounted Container: ./data.loom (Single Portable Storage File)", font=font_mono, fill=C_MUTED)
    draw.text((cx, cy + 70), "Type .help for dot commands or execute standard SQL queries.", font=font_mono, fill=C_MUTED)
    
    # Typing Query Effect (Frames 15 to 80 of this scene)
    line1_full = "SELECT id, topic, notes, cosine_similarity(embedding, [0.12, 0.45, -0.22, 0.81]) AS score"
    line2_full = "FROM memories ORDER BY score DESC LIMIT 2;"
    
    total_len = len(line1_full) + len(line2_full)
    typed_len = int(min(total_len, max(0, (rel_f - 15) * 2.4)))
    
    typed_l1 = line1_full[:min(len(line1_full), typed_len)]
    typed_l2 = line2_full[:max(0, typed_len - len(line1_full))]
    
    draw.text((cx, cy + 120), "loom> ", font=font_mono_b, fill=C_AMBER_BRIGHT)
    draw.text((cx + 80, cy + 120), typed_l1, font=font_mono, fill=C_WHITE)
    
    if typed_l2:
        draw.text((cx + 80, cy + 155), typed_l2, font=font_mono, fill=C_WHITE)
    
    # Blinking cursor
    if rel_f < 85 and (rel_f // 6) % 2 == 0:
        if not typed_l2:
            cursor_x = cx + 80 + len(typed_l1) * 13.5
            draw.rectangle([cursor_x, cy + 120, cursor_x + 12, cy + 145], fill=C_AMBER_BRIGHT)
        else:
            cursor_x = cx + 80 + len(typed_l2) * 13.5
            draw.rectangle([cursor_x, cy + 155, cursor_x + 12, cy + 180], fill=C_AMBER_BRIGHT)
        
    # Result Execution (From rel_f >= 85)
    if rel_f >= 85:
        res_y = cy + 205
        table_lines = [
            "+--------+--------------------------------+-----------------------------+--------+",
            "| id     | topic                          | notes                       | score  |",
            "+--------+--------------------------------+-----------------------------+--------+",
            "| mem_01 | High-Performance Rust Storage  | Micro profile <512KB RAM    | 0.9842 |",
            "| mem_02 | Fused HNSW SIMD Traversal      | Single-pass filter search   | 0.9120 |",
            "+--------+--------------------------------+-----------------------------+--------+",
            "2 rows in set. Execution time: 0.38 ms (AVX-512 SIMD Fused Engine)"
        ]
        
        for li, line in enumerate(table_lines):
            col = C_AMBER_BRIGHT if "0.38 ms" in line else (C_WHITE if "|" in line and li in [1, 3, 4] else C_SECONDARY)
            draw.text((cx, res_y + li * 34), line, font=font_mono_b if li in [3, 4, 6] else font_mono, fill=col)
            
        # Success Badge Overlay
        draw.rectangle([tx + tw - 460, ty + th - 85, tx + tw - 40, ty + th - 25], fill=(24, 30, 42), outline=C_AMBER, width=2)
        draw.text((tx + tw - 435, ty + th - 68), "0.38 ms SIMD FUSED SEARCH", font=font_mono_b, fill=C_AMBER_BRIGHT)

    return img

def render_scene_4(frame_idx):
    # Frames 360 - 479 (12.0s - 16.0s)
    rel_f = frame_idx - 360
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = rel_f / 120.0
    
    draw.text((120, 80), "HARDWARE BENCHMARK VERIFICATION // REAL WORKLOAD", font=font_small, fill=C_AMBER_BRIGHT)
    draw.text((120, 115), "RAM Footprint: Lower is Better", font=font_title, fill=C_WHITE)
    draw.text((120, 200), "Idle memory consumption comparison across embedded and distributed databases:", font=font_sub, fill=C_SECONDARY)
    
    benchmarks = [
        ("LoomDB v0.1.0", "< 0.48 MB", 0.04, True),
        ("SQLite 3.45", "~ 4.2 MB", 0.14, False),
        ("DuckDB 0.10", "~ 32.0 MB", 0.38, False),
        ("Pinecone / Qdrant", "~ 210.0 MB", 0.68, False),
        ("MongoDB 7.0", "~ 350.0 MB", 0.95, False)
    ]
    
    chart_y = 280
    max_w = 1200
    
    for i, (name, val, target_ratio, is_loom) in enumerate(benchmarks):
        row_y = chart_y + i * 105
        # Bar animation
        anim_ratio = min(1.0, max(0.0, (rel_f - 10 - i * 8) / 25.0))
        cur_w = int(max_w * target_ratio * ease_out(anim_ratio))
        
        # Name
        draw.text((120, row_y + 14), name, font=font_mono_b if is_loom else font_mono, fill=C_AMBER_BRIGHT if is_loom else C_WHITE)
        
        # Track
        draw.rectangle([460, row_y, 460 + max_w, row_y + 55], fill=C_CARBON, outline=C_BORDER, width=1)
        
        # Fill
        if cur_w > 0:
            fill_col = C_AMBER if is_loom else (51, 65, 85)
            draw.rectangle([460, row_y, 460 + cur_w, row_y + 55], fill=fill_col)
            
        # Value Label
        draw.text((460 + max_w + 30, row_y + 14), val, font=font_mono_b if is_loom else font_mono, fill=C_AMBER_BRIGHT if is_loom else C_SECONDARY)
        
    # Big Callout Footer
    draw.rectangle([120, 830, WIDTH - 120, 950], fill=C_SURFACE, outline=C_AMBER, width=2)
    draw.text((160, 855), "ZERO EXTERNAL THREADS · ZERO CLOUD BILLS · RUNS ANYWHERE", font=font_mono_b, fill=C_WHITE)
    draw.text((160, 895), "Compiles to WebAssembly OPFS, Raspberry Pi ARM64, and native Windows/macOS/Linux executables.", font=font_body, fill=C_SECONDARY)

    return img

def render_scene_5(frame_idx):
    # Frames 480 - 599 (16.0s - 20.0s)
    rel_f = frame_idx - 480
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = rel_f / 120.0
    
    # Final Climax Card
    draw.rectangle([120, 80, WIDTH - 120, 980], fill=C_CARBON, outline=C_AMBER, width=2)
    
    # Title
    draw.text((180, 140), "READY TO WEAVE YOUR DATA.", font=font_hero, fill=C_WHITE)
    draw.text((180, 245), "Download standalone single-file packages for your machine:", font=font_sub, fill=C_SECONDARY)
    
    # 3 OS Cards
    os_boxes = [
        ("WINDOWS", "loom-setup.exe", "Self-contained Setup Wizard (.EXE)\nConfigures PATH & background service"),
        ("MACOS", "loom-setup-macos.sh", "Apple Silicon M1/M2/M3 & Intel\nSingle standalone installer script"),
        ("LINUX", "loom-setup-linux.sh", "x86_64 & ARM64 Standalone\nRegisters systemd service unit")
    ]
    
    card_w = 480
    card_h = 240
    card_y = 320
    
    for i, (os_name, fname, desc) in enumerate(os_boxes):
        cx = 180 + i * (card_w + 40)
        draw.rectangle([cx, card_y, cx + card_w, card_y + card_h], fill=C_SURFACE, outline=C_BORDER, width=2)
        draw.text((cx + 28, card_y + 24), os_name, font=font_mono_b, fill=C_AMBER_BRIGHT)
        draw.text((cx + 28, card_y + 65), fname, font=font_sub_b, fill=C_WHITE)
        draw.text((cx + 28, card_y + 125), desc, font=font_body, fill=C_SECONDARY)
        draw.text((cx + 28, card_y + 195), "Download Ready →", font=font_mono_b, fill=C_AMBER)
        
    # Quick Terminal Install Box
    ty = 610
    draw.rectangle([180, ty, WIDTH - 180, ty + 90], fill=(15, 18, 24), outline=C_BORDER, width=2)
    draw.text((210, ty + 16), "TERMINAL ONE-LINER (AUTOMATIC PATH & SERVICE REGISTRATION):", font=font_small, fill=C_MUTED)
    draw.text((210, ty + 46), "irm https://loomdb.dev/install.ps1 | iex", font=font_mono_lg, fill=C_CYAN)
    
    # Final Footer Line & GitHub Link
    draw.text((180, 750), "LoomDB is 100% free and open-source under dual Apache-2.0 / MIT licenses.", font=font_sub, fill=C_SECONDARY)
    draw.text((180, 810), "github.com/loomdb/loomdb", font=font_hero, fill=C_AMBER_BRIGHT)

    return img

def render_frame(frame_idx):
    if frame_idx < 90:
        return render_scene_1(frame_idx)
    elif frame_idx < 210:
        return render_scene_2(frame_idx)
    elif frame_idx < 360:
        return render_scene_3(frame_idx)
    elif frame_idx < 480:
        return render_scene_4(frame_idx)
    else:
        return render_scene_5(frame_idx)

def main():
    print(f"Rendering {TOTAL_FRAMES} frames @ 1920x1080 (30fps)...")
    for i in range(TOTAL_FRAMES):
        img = render_frame(i)
        filename = os.path.join(OUTPUT_DIR, f"frame_{i:04d}.png")
        img.save(filename, format="PNG")
        if i % 60 == 0 or i == TOTAL_FRAMES - 1:
            print(f"Rendered frame {i}/{TOTAL_FRAMES} ({(i/TOTAL_FRAMES)*100:.1f}%)")
    print("All frames rendered successfully.")

if __name__ == "__main__":
    main()
