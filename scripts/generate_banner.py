import os
import math
from PIL import Image, ImageDraw, ImageFont, ImageFilter

WIDTH = 2400
HEIGHT = 750
OUTPUT_PATH = "D:/PROJECTS/LoomDB/assets/loomdb-banner.png"

# Load crisp system fonts
font_hero = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 116)
font_tagline = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 34)
font_sub = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 25)
font_badge = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 21)
font_pill = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 18)

# Colors
C_BG = (8, 10, 14)
C_SURFACE = (18, 23, 34)
C_BORDER = (38, 48, 68)
C_AMBER = (234, 88, 12)
C_AMBER_BRIGHT = (249, 115, 22)
C_AMBER_GLOW = (251, 146, 60)
C_CYAN = (56, 189, 248)
C_GREEN = (34, 197, 94)
C_WHITE = (255, 255, 255)
C_PRIMARY = (241, 245, 249)
C_SECONDARY = (156, 169, 188)
C_MUTED = (95, 110, 135)

def draw_radial_glow(img):
    glow = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    
    # Warm amber central ambient glow
    cx, cy = WIDTH // 2, HEIGHT // 2 - 40
    for r in range(480, 0, -8):
        alpha = int(24 * (1.0 - r / 480.0))
        gdraw.ellipse([cx - r * 1.8, cy - r, cx + r * 1.8, cy + r], fill=(249, 115, 22, alpha))
        
    # Cyan subtle edge accents
    for r in range(300, 0, -10):
        alpha = int(12 * (1.0 - r / 300.0))
        gdraw.ellipse([cx - 700 - r, cy - r, cx - 700 + r, cy + r], fill=(56, 189, 248, alpha))
        gdraw.ellipse([cx + 700 - r, cy - r, cx + 700 + r, cy + r], fill=(56, 189, 248, alpha))
        
    img.paste(Image.alpha_composite(img.convert("RGBA"), glow).convert("RGB"), (0, 0))

def draw_subtle_grid(draw):
    for x in range(0, WIDTH, 48):
        draw.line([(x, 0), (x, HEIGHT)], fill=(15, 19, 27), width=1)
    for y in range(0, HEIGHT, 48):
        draw.line([(0, y), (WIDTH, y)], fill=(15, 19, 27), width=1)

def draw_geometric_emblem(draw, center_x, center_y, size=52):
    # Modern woven hexagonal lattice emblem (representing LoomDB's converged threads)
    # 6 points of hexagon
    hex_pts = []
    for i in range(6):
        angle = math.radians(60 * i - 30)
        px = center_x + size * math.cos(angle)
        py = center_y + size * math.sin(angle)
        hex_pts.append((px, py))
        
    # Draw outer hexagon ring
    draw.polygon(hex_pts, outline=C_AMBER_BRIGHT, width=3)
    
    # Converging woven diagonals
    for i in range(3):
        p1 = hex_pts[i]
        p2 = hex_pts[i + 3]
        draw.line([p1, p2], fill=(234, 88, 12, 180), width=2)
        
    # Central illuminated core container node
    draw.ellipse([center_x - 12, center_y - 12, center_x + 12, center_y + 12], fill=C_WHITE, outline=C_AMBER_BRIGHT, width=2)
    
    # Outer orbiting subtle nodes
    for pt in hex_pts:
        draw.ellipse([pt[0] - 4, pt[1] - 4, pt[0] + 4, pt[1] + 4], fill=C_CYAN)

def generate_banner():
    img = Image.new("RGB", (WIDTH, HEIGHT), C_BG)
    draw_radial_glow(img)
    draw = ImageDraw.Draw(img)
    draw_subtle_grid(draw)

    # Sleek top & bottom framing border lines
    draw.line([(0, 0), (WIDTH, 0)], fill=C_BORDER, width=2)
    draw.line([(0, HEIGHT - 1), (WIDTH, HEIGHT - 1)], fill=C_BORDER, width=2)
    # Subtle amber glow line at top
    draw.line([((WIDTH - 900) // 2, 0), ((WIDTH + 900) // 2, 0)], fill=C_AMBER_BRIGHT, width=2)

    # 1. Top Mini-Pill: Systems Architecture Category
    pill_text = "EMBEDDED MULTI-MODAL DATABASE  //  v0.1.0-STABLE"
    p_bbox = font_pill.getbbox(pill_text)
    pw = (p_bbox[2] - p_bbox[0]) + 36
    px = (WIDTH - pw) // 2
    py = 65
    draw.rectangle([px, py, px + pw, py + 36], fill=(24, 18, 16), outline=C_AMBER, width=1)
    draw.text((px + 18, py + 7), pill_text, font=font_pill, fill=C_AMBER_BRIGHT)

    # 2. Main Hero Logo & Wordmark
    # Measure wordmark width
    wm_text = "LoomDB"
    wm_bbox = font_hero.getbbox(wm_text)
    wm_w = wm_bbox[2] - wm_bbox[0]
    
    emblem_w = 120
    spacing = 30
    total_brand_w = emblem_w + spacing + wm_w
    
    brand_start_x = (WIDTH - total_brand_w) // 2
    emblem_center_x = brand_start_x + emblem_w // 2
    emblem_center_y = 195
    
    # Draw geometric icon
    draw_geometric_emblem(draw, emblem_center_x, emblem_center_y, size=46)
    
    # Draw Wordmark
    wm_x = brand_start_x + emblem_w + spacing
    wm_y = 135
    draw.text((wm_x, wm_y), wm_text, font=font_hero, fill=C_WHITE)
    
    # Small single-file badge next to wordmark
    sf_x = wm_x + wm_w + 16
    sf_y = wm_y + 30
    draw.rectangle([sf_x, sf_y, sf_x + 95, sf_y + 30], fill=C_SURFACE, outline=C_BORDER, width=1)
    draw.text((sf_x + 10, sf_y + 5), ".LOOM", font=font_badge, fill=C_CYAN)

    # 3. Primary Value Proposition Tagline (Bold & Centered)
    tagline = "Forkable · Deterministic · Budget-Enforced · Zero-Copy"
    t_bbox = font_tagline.getbbox(tagline)
    tw = t_bbox[2] - t_bbox[0]
    tx = (WIDTH - tw) // 2
    ty = 295
    draw.text((tx, ty), tagline, font=font_tagline, fill=C_WHITE)

    # 4. Secondary Architecture Subtitle (Clean & Elegant)
    sub_text = "Five native database paradigms woven into a single portable container file."
    s_bbox = font_sub.getbbox(sub_text)
    sw = s_bbox[2] - s_bbox[0]
    sx = (WIDTH - sw) // 2
    sy = 352
    draw.text((sx, sy), sub_text, font=font_sub, fill=C_SECONDARY)

    # 5. Modalities Minimal Strip
    mod_text = "ANSI SQL RELATIONAL  ·  SCHEMALESS JSON  ·  AVX-512 VECTORS  ·  GORILLA TIME-SERIES  ·  CSR GRAPH"
    m_bbox = font_badge.getbbox(mod_text)
    mw = m_bbox[2] - m_bbox[0]
    mx = (WIDTH - mw) // 2
    my = 405
    draw.text((mx, my), mod_text, font=font_badge, fill=C_MUTED)

    # 6. Central Five Metric Badge Pills (Spacious, breathing room)
    badges = [
        ("< 512 KB IDLE RAM", C_AMBER_BRIGHT),
        ("< 850 KB BINARY", C_CYAN),
        ("STABLE SINGLE-FILE ABI", C_PRIMARY),
        ("NATIVE OPFS MULTI-TAB", C_GREEN),
        ("100% PURE SAFE RUST", C_WHITE)
    ]
    
    pill_y = 485
    pill_h = 56
    
    # Calculate total width for perfect centering
    pill_widths = []
    for b_text, _ in badges:
        bb = font_badge.getbbox(b_text)
        pill_widths.append((bb[2] - bb[0]) + 44)
        
    gap = 20
    total_pills_w = sum(pill_widths) + gap * (len(badges) - 1)
    cur_x = (WIDTH - total_pills_w) // 2
    
    for i, (b_text, b_col) in enumerate(badges):
        pw = pill_widths[i]
        # Pill background
        draw.rectangle([cur_x, pill_y, cur_x + pw, pill_y + pill_h], fill=C_SURFACE, outline=C_BORDER, width=1)
        # Small colored accent dot inside pill
        draw.ellipse([cur_x + 18, pill_y + 23, cur_x + 28, pill_y + 33], fill=b_col)
        # Text
        draw.text((cur_x + 38, pill_y + 16), b_text, font=font_badge, fill=C_PRIMARY if b_col == C_WHITE else b_col)
        cur_x += pw + gap

    # 7. Bottom Minimal Technical Spec Footer
    bot_y = 615
    f_left = "github.com/armoredglock/LoomDB"
    f_mid = "•   Safe Rust   •   WebAssembly   •   Linux / macOS / Windows"
    f_right = "Dual License: MIT / Apache-2.0"
    
    draw.text((120, bot_y), f_left, font=font_badge, fill=C_AMBER_BRIGHT)
    
    m_bbox2 = font_badge.getbbox(f_mid)
    draw.text(((WIDTH - (m_bbox2[2] - m_bbox2[0])) // 2, bot_y), f_mid, font=font_badge, fill=C_MUTED)
    
    r_bbox = font_badge.getbbox(f_right)
    draw.text((WIDTH - 120 - (r_bbox[2] - r_bbox[0]), bot_y), f_right, font=font_badge, fill=C_SECONDARY)

    # Save high-res optimized PNG
    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)
    img.save(OUTPUT_PATH, "PNG", optimize=True)
    print(f"Banner generated successfully: {OUTPUT_PATH} ({WIDTH}x{HEIGHT})")

if __name__ == "__main__":
    generate_banner()
