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
    font_hero = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 80)
    font_title = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 60)
    font_subtitle = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 30)
    font_subtitle_b = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 30)
    font_body = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 24)
    font_body_b = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 24)
    font_mono = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 22)
    font_mono_b = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 23)
    font_mono_lg = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 28)
    font_small = ImageFont.truetype("C:/Windows/Fonts/consola.ttf", 18)
    font_card_title = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 38)
except Exception as e:
    print("Fallback to default font:", e)
    font_hero = font_title = font_subtitle = font_subtitle_b = font_body = font_body_b = font_mono = font_mono_b = font_mono_lg = font_small = font_card_title = ImageFont.load_default()

# Colors
C_BG = (7, 8, 11)
C_CARBON = (12, 14, 19)
C_SURFACE = (18, 22, 30)
C_ELEVATED = (25, 31, 42)
C_BORDER = (34, 42, 58)
C_AMBER = (234, 88, 12)
C_AMBER_BRIGHT = (249, 115, 22)
C_AMBER_GLOW = (251, 146, 60)
C_CYAN = (56, 189, 248)
C_GREEN = (34, 197, 94)
C_WHITE = (255, 255, 255)
C_PRIMARY = (241, 245, 249)
C_SECONDARY = (160, 174, 192)
C_MUTED = (100, 116, 139)

def draw_grid(draw):
    for x in range(0, WIDTH, 64):
        draw.line([(x, 0), (x, HEIGHT)], fill=(16, 20, 27), width=1)
    for y in range(0, HEIGHT, 64):
        draw.line([(0, y), (WIDTH, y)], fill=(16, 20, 27), width=1)

def ease_out(t):
    return 1 - (1 - t) ** 3

def render_scene_1(frame_idx):
    # Frames 0 - 89 (0.0s - 3.0s)
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = frame_idx / 90.0
    
    # Eyebrow
    draw.rectangle([120, 100, 520, 140], fill=(234, 88, 12, 35), outline=C_AMBER, width=1)
    draw.text((136, 110), "SYSTEMS ARCHITECTURE // 2026 AUDIT", font=font_small, fill=C_AMBER_BRIGHT)
    
    # Main Headline
    draw.text((120, 165), "EMBEDDED DATABASES ARE MISSING", font=font_title, fill=C_WHITE)
    draw.text((120, 245), "CORE STORAGE GUARANTEES.", font=font_title, fill=(240, 90, 45))
    
    draw.text((120, 335), "Existing embedded engines compromise on branching, determinism, and budget bounds:", font=font_subtitle, fill=C_SECONDARY)
    
    # 3 Problem Cards
    cards = [
        ("NO CHEAP BRANCHING", "Cloned database files explode disk and RAM usage.", "Full-copy overhead"),
        ("NON-DETERMINISTIC", "Edge crashes and heisenbugs cannot be reproduced bit-for-bit.", "Unreproducible state"),
        ("UNENFORCED BUDGETS", "Zero resource governance leads to silent OOM process kills.", "Fatal OOM crashes")
    ]
    
    card_w = 520
    card_h = 200
    card_y = 410
    
    for i, (title, desc, tag) in enumerate(cards):
        cx = 120 + i * (card_w + 55)
        card_t = max(0, min(1, (frame_idx - 12 - i * 8) / 14.0))
        if card_t > 0:
            scale_y = int(card_y + (1 - ease_out(card_t)) * 35)
            draw.rectangle([cx, scale_y, cx + card_w, scale_y + card_h], fill=C_SURFACE, outline=C_BORDER, width=2)
            # Left accent red bar
            draw.rectangle([cx, scale_y, cx + 8, scale_y + card_h], fill=(220, 38, 38))
            draw.text((cx + 28, scale_y + 26), title, font=font_body_b, fill=C_WHITE)
            draw.text((cx + 28, scale_y + 72), desc, font=font_body, fill=C_SECONDARY)
            # Tag
            draw.rectangle([cx + 28, scale_y + 138, cx + 250, scale_y + 172], fill=(35, 18, 22), outline=(220, 38, 38), width=1)
            draw.text((cx + 42, scale_y + 144), tag, font=font_small, fill=(248, 113, 113))
            
    # Bottom Alert Bar
    if frame_idx >= 45:
        bar_t = min(1, (frame_idx - 45) / 12.0)
        alpha = ease_out(bar_t)
        by = int(665 + (1 - alpha) * 30)
        draw.rectangle([120, by, WIDTH - 120, by + 90], fill=(28, 14, 18), outline=(220, 38, 38), width=2)
        draw.text((150, by + 18), "THE RESULT: UNPREDICTABLE EDGE FAILURES & PRODUCTION DAEMON SPRAWL", font=font_mono_lg, fill=(254, 202, 202))
        draw.text((150, by + 54), "Developers are forced to glue together 4 heavy daemons just to get multi-modal capabilities.", font=font_body, fill=C_MUTED)

    # Question Hook
    if frame_idx >= 65:
        hook_t = min(1, (frame_idx - 65) / 10.0)
        hy = int(820 + (1 - ease_out(hook_t)) * 25)
        draw.text((120, hy), "WHAT IF AN EMBEDDED ENGINE GAVE YOU HARD GUARANTEES?", font=font_title, fill=C_AMBER_BRIGHT)
        
    return img

def render_scene_2(frame_idx):
    # Frames 90 - 209 (3.0s - 7.0s)
    rel_f = frame_idx - 90
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = rel_f / 120.0
    
    # Eyebrow
    draw.rectangle([120, 60, 520, 100], fill=(234, 88, 12, 35), outline=C_AMBER, width=1)
    draw.text((136, 70), "THE NOVEL STORAGE BUNDLE // v0.1.0", font=font_small, fill=C_AMBER_BRIGHT)
    
    draw.text((120, 120), "LoomDB", font=font_hero, fill=C_WHITE)
    
    # Positioning Sentence Callout
    draw.rectangle([120, 220, WIDTH - 120, 310], fill=C_SURFACE, outline=C_AMBER, width=2)
    draw.text((144, 235), "LoomDB is a forkable, deterministic, budget-enforced, zero-copy embedded", font=font_subtitle_b, fill=C_WHITE)
    draw.text((144, 268), "multi-modal database with a stable single-file ABI and native OPFS multi-tab coordination.", font=font_subtitle, fill=C_AMBER_BRIGHT)
    
    # 5 Novelty Primitives
    primitives = [
        ("1. Forkable COW Snapshots", "fork(), checkout(), discard() at page level. Snapshots are cheap COW pages, not full copies.", "STORAGE PRIMITIVE"),
        ("2. Deterministic Flight Recorder", "Lightweight WAL recording seeds, timestamps & external inputs. Bit-for-bit replay.", "GUARANTEED REPLAY"),
        ("3. Hard Resource-Budget Governor", "Strict caps: RAM, WAL size, page cache. Enforced via backpressure and _sys_budget table.", "ENFORCED CONTRACT"),
        ("4. Zero-Copy Mmap ABI", "Documented byte layout, lock-free readers + single-writer protocol, typed reader slices.", "STABLE ABI"),
        ("5. Native OPFS Multi-Tab Coord", "Browser-native single-writer/multi-reader coordination via Web Locks & BroadcastChannel CDC.", "CORRUPTION-FREE")
    ]
    
    card_w = WIDTH - 240
    card_h = 75
    start_y = 335
    
    for i, (name, desc, badge) in enumerate(primitives):
        cy = start_y + i * 85
        card_t = max(0, min(1, (rel_f - 10 - i * 10) / 12.0))
        if card_t > 0:
            cx = int(120 + (1 - ease_out(card_t)) * 40)
            is_active = (int(rel_f / 18.0) % 5 == i)
            
            fill_col = (26, 32, 45) if is_active else C_CARBON
            border_col = C_AMBER_BRIGHT if is_active else C_BORDER
            
            draw.rectangle([cx, cy, cx + card_w, cy + card_h], fill=fill_col, outline=border_col, width=2 if is_active else 1)
            draw.text((cx + 24, cy + 15), name, font=font_body_b, fill=C_AMBER_BRIGHT if is_active else C_WHITE)
            draw.text((cx + 420, cy + 18), desc, font=font_small, fill=C_WHITE if is_active else C_SECONDARY)
            
            # Badge
            draw.rectangle([cx + card_w - 240, cy + 20, cx + card_w - 30, cy + 54], fill=C_SURFACE, outline=border_col, width=1)
            draw.text((cx + card_w - 220, cy + 26), badge, font=font_small, fill=C_AMBER_BRIGHT if is_active else C_MUTED)

    # Footprint Spec Strip
    if rel_f >= 75:
        fy = 780
        draw.rectangle([120, fy, WIDTH - 120, fy + 75], fill=(16, 20, 28), outline=C_BORDER, width=2)
        draw.text((150, fy + 24), "IDLE RAM: < 512 KB", font=font_mono_b, fill=C_AMBER_BRIGHT)
        draw.text((450, fy + 24), "BINARY SIZE: < 850 KB", font=font_mono_b, fill=C_CYAN)
        draw.text((790, fy + 24), "STORAGE: SINGLE .LOOM FILE", font=font_mono_b, fill=C_GREEN)
        draw.text((1200, fy + 24), "MODALITIES: SQL · JSON · VECTORS · TIME-SERIES · GRAPH", font=font_mono, fill=C_SECONDARY)

    return img

def render_scene_3(frame_idx):
    # Frames 210 - 359 (7.0s - 12.0s)
    rel_f = frame_idx - 210
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    # Eyebrow & Headline
    draw.text((120, 60), "LIVE COMMAND EXECUTION // RUST ENGINE VERIFICATION", font=font_small, fill=C_AMBER_BRIGHT)
    draw.text((120, 95), "Interactive Terminal Shell", font=font_title, fill=C_WHITE)
    
    # Terminal Window Outer
    tx = 120
    ty = 175
    tw = 1680
    th = 840
    
    draw.rectangle([tx, ty, tx + tw, ty + th], fill=(10, 12, 16), outline=C_BORDER, width=2)
    # Titlebar
    draw.rectangle([tx, ty, tx + tw, ty + 46], fill=(17, 20, 27))
    draw.ellipse([tx + 20, ty + 16, tx + 34, ty + 30], fill=(255, 95, 86))
    draw.ellipse([tx + 44, ty + 16, tx + 58, ty + 30], fill=(255, 189, 46))
    draw.ellipse([tx + 68, ty + 16, tx + 82, ty + 30], fill=(39, 201, 63))
    
    draw.text((tx + tw // 2 - 180, ty + 12), "loom — ./data.loom [Micro Profile / Budget Active]", font=font_mono, fill=C_MUTED)
    
    # Terminal Content
    cx = tx + 36
    cy = ty + 68
    
    draw.text((cx, cy), "[LOOM_DB] Safe Rust Embedded Engine v0.1.0", font=font_mono_b, fill=C_CYAN)
    draw.text((cx, cy + 30), "Container: ./data.loom | Single-Writer Protocol | Flight Recorder ON", font=font_mono, fill=C_MUTED)
    
    # Step 1: Fork Branch (rel_f >= 10)
    cmd1 = "FORK BRANCH 'experiment-v1';"
    typed1 = cmd1[:int(min(len(cmd1), max(0, (rel_f - 10) * 1.8)))]
    draw.text((cx, cy + 75), "loom> ", font=font_mono_b, fill=C_AMBER_BRIGHT)
    draw.text((cx + 75, cy + 75), typed1, font=font_mono, fill=C_WHITE)
    
    if rel_f >= 30:
        draw.text((cx, cy + 105), "[OK] Branch 'experiment-v1' active. (COW copy-on-write 0-byte allocation in 312 µs)", font=font_mono_b, fill=C_GREEN)
        
    # Step 2: Query Budget (rel_f >= 45)
    if rel_f >= 45:
        cmd2 = "SELECT metric, cap_bytes, used_bytes, status FROM _sys_budget;"
        typed2 = cmd2[:int(min(len(cmd2), max(0, (rel_f - 45) * 1.8)))]
        draw.text((cx, cy + 150), "loom> ", font=font_mono_b, fill=C_AMBER_BRIGHT)
        draw.text((cx + 75, cy + 150), typed2, font=font_mono, fill=C_WHITE)
        
    if rel_f >= 75:
        table_lines = [
            "+------------------+---------------+---------------+---------------------+",
            "| metric           | cap_bytes     | used_bytes    | status              |",
            "+------------------+---------------+---------------+---------------------+",
            "| max_ram_bytes    | 524,288 (512K)| 42,880 (41KB) | 8.2% [ENFORCED]     |",
            "| max_wal_bytes    | 1,048,576(1MB)| 18,432 (18KB) | 1.8% [HEALTHY]      |",
            "| max_page_cache   | 2,097,152(2MB)| 131,072(128K) | 6.2% [BOUNDED]      |",
            "+------------------+---------------+---------------+---------------------+"
        ]
        by = cy + 185
        for li, line in enumerate(table_lines):
            col = C_AMBER_BRIGHT if "8.2%" in line else (C_WHITE if "|" in line and li in [1, 3, 4, 5] else C_SECONDARY)
            draw.text((cx, by + li * 28), line, font=font_mono, fill=col)
            
    # Step 3: Flight Recorder Replay (rel_f >= 95)
    if rel_f >= 95:
        cmd3 = "SELECT replay_verify(tx_id=402);"
        typed3 = cmd3[:int(min(len(cmd3), max(0, (rel_f - 95) * 2.0)))]
        draw.text((cx, cy + 405), "loom> ", font=font_mono_b, fill=C_AMBER_BRIGHT)
        draw.text((cx + 75, cy + 405), typed3, font=font_mono, fill=C_WHITE)
        
    if rel_f >= 115:
        draw.text((cx, cy + 438), "[FLIGHT_RECORDER] Bit-for-bit replay verified. Non-deterministic WAL replayed.", font=font_mono_b, fill=C_CYAN)
        draw.text((cx, cy + 468), "Reconstructed State Checksum: 0x9AF2C188 | Delta: 0 bytes | Time: 0.21 ms", font=font_mono_b, fill=C_GREEN)

    # Step 4: Vector Query (rel_f >= 125)
    if rel_f >= 125:
        draw.text((cx, cy + 515), "loom> SELECT id, topic, cosine_similarity(emb, [...]) FROM memories ORDER BY score DESC LIMIT 1;", font=font_mono, fill=C_WHITE)
        draw.text((cx, cy + 545), "| mem_01 | High-Performance Storage Pager | score: 0.9842 | Time: 0.38 ms (AVX-512 SIMD)", font=font_mono_b, fill=C_AMBER_BRIGHT)

    # Success Badge
    draw.rectangle([tx + tw - 540, ty + th - 80, tx + tw - 40, ty + th - 25], fill=(24, 30, 42), outline=C_AMBER, width=2)
    draw.text((tx + tw - 515, ty + th - 62), "DETERMINISTIC · HARD BUDGETS · ZERO-COPY", font=font_mono_b, fill=C_AMBER_BRIGHT)

    return img

def render_scene_4(frame_idx):
    # Frames 360 - 479 (12.0s - 16.0s)
    rel_f = frame_idx - 360
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = rel_f / 120.0
    
    draw.text((120, 60), "EDGE & BROWSER ARCHITECTURE // RIGOROUS BENCHMARK", font=font_small, fill=C_AMBER_BRIGHT)
    draw.text((120, 95), "OPFS Multi-Tab Coordination & Micro Footprint", font=font_title, fill=C_WHITE)
    
    # Left Card: OPFS Multi-Tab Coordination Diagram
    lx = 120
    lw = 820
    ly = 180
    lh = 630
    draw.rectangle([lx, ly, lx + lw, ly + lh], fill=C_CARBON, outline=C_BORDER, width=2)
    draw.text((lx + 32, ly + 28), "BROWSER OPFS MULTI-TAB COORDINATION", font=font_mono_b, fill=C_CYAN)
    draw.text((lx + 32, ly + 65), "Safe single-writer & multi-reader coordination in WebAssembly:", font=font_body, fill=C_SECONDARY)
    
    # Diagram Boxes
    # Tab 1
    draw.rectangle([lx + 40, ly + 120, lx + lw - 40, ly + 200], fill=C_SURFACE, outline=C_AMBER, width=2)
    draw.text((lx + 60, ly + 138), "Browser Tab 1 (Leader / Writer)", font=font_body_b, fill=C_WHITE)
    draw.text((lx + 60, ly + 168), "Acquires Web Locks API 'loom-writer-lock' · Exclusive writes", font=font_small, fill=C_AMBER_BRIGHT)
    
    # Arrow down
    draw.text((lx + lw // 2 - 80, ly + 215), "↓ Writes mutations", font=font_mono, fill=C_MUTED)
    
    # Storage File
    draw.rectangle([lx + 40, ly + 250, lx + lw - 40, ly + 330], fill=(22, 28, 38), outline=C_GREEN, width=2)
    draw.text((lx + 60, ly + 268), "Origin Private File System (OPFS)", font=font_body_b, fill=C_WHITE)
    draw.text((lx + 60, ly + 298), "Single data.loom container · Shared direct storage buffer", font=font_small, fill=C_GREEN)
    
    # Arrow down
    draw.text((lx + lw // 2 - 120, ly + 345), "↓ BroadcastChannel CDC Events", font=font_mono, fill=C_CYAN)
    
    # Tab 2
    draw.rectangle([lx + 40, ly + 380, lx + lw - 40, ly + 460], fill=C_SURFACE, outline=C_BORDER, width=2)
    draw.text((lx + 60, ly + 398), "Browser Tab 2..N (Lock-Free Readers)", font=font_body_b, fill=C_WHITE)
    draw.text((lx + 60, ly + 428), "Reads snapshots via Generation Seqlock · Instant UI update", font=font_small, fill=C_SECONDARY)
    
    # Summary list
    draw.text((lx + 40, ly + 490), "• Eliminates multi-tab database corruption without a server", font=font_small, fill=C_WHITE)
    draw.text((lx + 40, ly + 520), "• Heartbeat crash detection re-assigns writer lease on tab close", font=font_small, fill=C_WHITE)
    draw.text((lx + 40, ly + 550), "• Works in Safari, Chrome, Firefox, and Edge with zero plugins", font=font_small, fill=C_WHITE)
    
    # Right Card: Hardware Benchmark Knockdown
    rx = 980
    rw = 820
    ry = 180
    rh = 630
    draw.rectangle([rx, ry, rx + rw, ry + rh], fill=C_CARBON, outline=C_BORDER, width=2)
    draw.text((rx + 32, ry + 28), "IDLE RAM OVERHEAD BENCHMARK", font=font_mono_b, fill=C_AMBER_BRIGHT)
    draw.text((rx + 32, ry + 65), "Real-world memory consumption (Lower is Better):", font=font_body, fill=C_SECONDARY)
    
    benchmarks = [
        ("LoomDB v0.1.0", "< 0.48 MB", 0.04, True),
        ("SQLite 3.45", "~ 4.2 MB", 0.14, False),
        ("DuckDB 0.10", "~ 32.0 MB", 0.38, False),
        ("Pinecone / Qdrant", "~ 210.0 MB", 0.68, False),
        ("MongoDB 7.0", "~ 350.0 MB", 0.95, False)
    ]
    
    chart_y = ry + 120
    max_w = 480
    
    for i, (name, val, target_ratio, is_loom) in enumerate(benchmarks):
        row_y = chart_y + i * 85
        anim_ratio = min(1.0, max(0.0, (rel_f - 10 - i * 8) / 25.0))
        cur_w = int(max_w * target_ratio * ease_out(anim_ratio))
        
        draw.text((rx + 32, row_y + 12), name, font=font_mono_b if is_loom else font_mono, fill=C_AMBER_BRIGHT if is_loom else C_WHITE)
        draw.rectangle([rx + 250, row_y, rx + 250 + max_w, row_y + 45], fill=C_SURFACE, outline=C_BORDER, width=1)
        
        if cur_w > 0:
            fill_col = C_AMBER if is_loom else (51, 65, 85)
            draw.rectangle([rx + 250, row_y, rx + 250 + cur_w, row_y + 45], fill=fill_col)
            
        draw.text((rx + 250 + max_w + 15, row_y + 12), val, font=font_mono_b if is_loom else font_mono, fill=C_AMBER_BRIGHT if is_loom else C_SECONDARY)

    # Bottom Callout Banner
    draw.rectangle([120, 840, WIDTH - 120, 960], fill=C_SURFACE, outline=C_AMBER, width=2)
    draw.text((160, 865), "5 CONVERGED MODALITIES: SQL · JSON · AVX-512 VECTORS · TIME-SERIES · GRAPH", font=font_mono_b, fill=C_WHITE)
    draw.text((160, 905), "All woven into a single portable container file with deterministic replay and zero external daemons.", font=font_body, fill=C_SECONDARY)

    return img

def render_scene_5(frame_idx):
    # Frames 480 - 599 (16.0s - 20.0s)
    rel_f = frame_idx - 480
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw = ImageDraw.Draw(img)
    draw_grid(draw)
    
    t = rel_f / 120.0
    
    # Outer frame card
    draw.rectangle([120, 60, WIDTH - 120, 990], fill=C_CARBON, outline=C_AMBER, width=2)
    
    # Title
    draw.text((180, 115), "THE REINVENTED EMBEDDED DATABASE.", font=font_hero, fill=C_WHITE)
    draw.text((180, 220), "Production-ready standalone binaries & packages available today:", font=font_subtitle, fill=C_SECONDARY)
    
    # 3 OS Cards
    os_boxes = [
        ("WINDOWS", "loom-setup.exe", "Self-contained Setup Wizard (.EXE)\nVerified standalone binary (1.3 MB)"),
        ("MACOS", "loom-setup-macos.sh", "Apple Silicon M1/M2/M3 & Intel\nSingle standalone installer script"),
        ("LINUX & WASM", "@loomdb/wasm", "x86_64 / ARM64 native binaries\nNative OPFS browser Wasm package")
    ]
    
    card_w = 480
    card_h = 230
    card_y = 285
    
    for i, (os_name, fname, desc) in enumerate(os_boxes):
        cx = 180 + i * (card_w + 40)
        draw.rectangle([cx, card_y, cx + card_w, card_y + card_h], fill=C_SURFACE, outline=C_BORDER, width=2)
        draw.text((cx + 28, card_y + 22), os_name, font=font_mono_b, fill=C_AMBER_BRIGHT)
        draw.text((cx + 28, card_y + 60), fname, font=font_subtitle_b, fill=C_WHITE)
        draw.text((cx + 28, card_y + 115), desc, font=font_body, fill=C_SECONDARY)
        draw.text((cx + 28, card_y + 185), "Download Artifact →", font=font_mono_b, fill=C_AMBER)
        
    # Quick Terminal Install Box
    ty = 560
    draw.rectangle([180, ty, WIDTH - 180, ty + 95], fill=(15, 18, 24), outline=C_BORDER, width=2)
    draw.text((210, ty + 18), "POWERSHELL QUICK INSTALL (STANDALONE CLI + ENVIRONMENT CONFIGURATION):", font=font_small, fill=C_MUTED)
    draw.text((210, ty + 48), "irm https://raw.githubusercontent.com/armoredglock/LoomDB/main/scripts/install.ps1 | iex", font=font_mono_lg, fill=C_CYAN)
    
    # Novelty Summary Bar
    sy = 690
    draw.rectangle([180, sy, WIDTH - 180, sy + 75], fill=C_SURFACE, outline=C_BORDER, width=1)
    draw.text((210, sy + 24), "FORKABLE SNAPSHOTS · DETERMINISTIC REPLAY · BUDGET GOVERNOR · ZERO-COPY ABI · OPFS MULTI-TAB", font=font_mono_b, fill=C_WHITE)
    
    # Final Footer Line & GitHub Link
    draw.text((180, 800), "Open-source under Apache-2.0 / MIT Dual License. Star the repository on GitHub:", font=font_subtitle, fill=C_SECONDARY)
    draw.text((180, 860), "github.com/armoredglock/LoomDB", font=font_hero, fill=C_AMBER_BRIGHT)

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
