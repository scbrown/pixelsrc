"""Build skull_contact_sheet.png: a few frames from each skull animation.

    cargo build --release && python3 examples/skull/contact_sheet.py

pxl renders every frame as a spritesheet (`pxl render --spritesheet`). This
script only picks frames, adds labels and stacks the rows. Needs Pillow.
"""
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).parent
PXL = HERE.parent.parent / "target" / "release" / "pxl"
SCALE, CELL = 4, 44  # every frame is placed in a 44x44 cell, shown 4x

# (file, animation, frame size, frames to show, label)
ROWS = [
    ("skull_anim.pxl", "skull_idle", 44, [0, 5, 6, 11, 20, 21], "idle: 1px float, blink"),
    ("skull_anim.pxl", "skull_laugh", 44, [0, 1, 2, 3, 5, 7], "laugh: chatter + head tip"),
    ("skull_anim.pxl", "skull_eyes", 44, [0, 1, 3, 8, 12, 15], "eyes: ember glow flicker"),
    ("skull.pxl", "skull_talking", 40, [0, 1, 2, 5, 7, 16], "talking: speech timing"),
]

label_w, pad = 190, 8
cols = max(len(r[3]) for r in ROWS)
cell = CELL * SCALE
sheet = Image.new("RGB", (label_w + cols * (cell + pad) + pad, len(ROWS) * (cell + pad) + pad), "white")
draw = ImageDraw.Draw(sheet)

with tempfile.TemporaryDirectory() as tmp:
    for row, (src, anim, size, frames, label) in enumerate(ROWS):
        strip_path = Path(tmp) / f"{anim}.png"
        subprocess.run(
            [str(PXL), "render", str(HERE / src), "--spritesheet", "--animation", anim,
             "--scale", str(SCALE), "-o", str(strip_path)],
            check=True, capture_output=True,
        )
        strip = Image.open(strip_path).convert("RGBA")
        y = pad + row * (cell + pad)
        draw.text((pad, y + cell // 2 - 12), label, fill=(80, 76, 72))
        draw.text((pad, y + cell // 2 + 4), f"{src} / {anim}", fill=(150, 144, 136))
        inset = (CELL - size) * SCALE // 2
        for col, f in enumerate(frames):
            frame = strip.crop((f * size * SCALE, 0, (f + 1) * size * SCALE, size * SCALE))
            x = label_w + col * (cell + pad)
            sheet.paste(frame, (x + inset, y + inset), frame)
            draw.text((x + 4, y + 2), f"#{f}", fill=(170, 164, 156))

sheet.save(HERE / "skull_contact_sheet.png")
print("Saved:", HERE / "skull_contact_sheet.png")
