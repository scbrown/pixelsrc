"""Build skull_comparison.png: the reference beside pixelsrc's render.

    python3 examples/skull/compare.py   (needs Pillow; run from the repo root)

Expects examples/skull/skull.png to be rendered first (see README.md).
"""
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).parent
ref = Image.open(HERE / "reference-skull.jpg").convert("RGB")
mine = Image.open(HERE / "skull.png").convert("RGBA")

# Crop both to the skull, then scale the render (nearest neighbour) so the
# two skulls are the same height on screen.
ref = ref.crop((50, 20, 285, 300))
mine = mine.crop(mine.getbbox())
k = 8
mine = mine.resize((mine.width * k, mine.height * k), Image.NEAREST)
ref = ref.resize((round(ref.width * mine.height / ref.height), mine.height), Image.NEAREST)

pad, label_h = 16, 24
out = Image.new("RGB", (ref.width + mine.width + pad * 3, mine.height + pad * 2 + label_h), "white")
out.paste(ref, (pad, pad + label_h))
out.paste(mine, (ref.width + pad * 2, pad + label_h), mine)
draw = ImageDraw.Draw(out)
draw.text((pad, pad), "reference-skull.jpg", fill=(80, 76, 72))
draw.text((ref.width + pad * 2, pad), "skull.pxl -> pxl render (40x40, shown 8x)", fill=(80, 76, 72))
out.save(HERE / "skull_comparison.png")
print("Saved:", HERE / "skull_comparison.png")
