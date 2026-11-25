#!/usr/bin/env python3
"""Regenerate README art and code samples with the pinned released renderer."""

import argparse
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
ART = Path("docs/assets/readme")
VERSION = "0.4.0"
SAMPLES = {"coin": 24, "hero": 12}
ASSETS = [
    ("logo", "pixelsrc_logo", "-c", 1),
    ("logo", "pixelsrc_logo", "-c", 4),
    ("coin", "coin", "-s", 1),
    ("coin", "coin", "-s", 24),
    ("hero", "hero", "-s", 1),
    ("hero", "hero", "-s", 12),
]


def image_name(name: str, scale: int) -> str:
    return f"{name}.png" if scale == 1 else f"{name}-{scale}x.png"


def readme_sections() -> dict[str, str]:
    sections = {
        "logo": "[![pixelsrc: a neon p with a brush-stroke descender]"
        f"({ART}/logo-4x.png)]({ART}/logo.pxl)"
    }
    for name, scale in SAMPLES.items():
        source = (ROOT / ART / f"{name}.pxl").read_text().rstrip()
        sections[name] = (
            f"```json5\n{source}\n```\n\n"
            f"![{name} rendered from the source above]({ART}/{image_name(name, scale)})\n\n"
            f"[Editable source]({ART}/{name}.pxl) · rendered at {scale}× nearest-neighbor scale."
        )
    return sections


def expected_readme(current: str) -> str:
    expected = current
    for name, content in readme_sections().items():
        start = f"<!-- pixelsrc-readme:{name}:start -->"
        end = f"<!-- pixelsrc-readme:{name}:end -->"
        pattern = re.compile(re.escape(start) + ".*?" + re.escape(end), re.S)
        if len(pattern.findall(expected)) != 1:
            raise ValueError(f"README must have exactly one {name} source/preview section")
        expected = pattern.sub(lambda _: f"{start}\n{content}\n{end}", expected)

    # Every README JSON5 example must have a declared, rendered source. A new
    # unpaired code block fails rather than silently escaping the asset check.
    blocks = re.findall(r"```json5\n(.*?)\n```", expected, re.S)
    sources = [(ROOT / ART / f"{name}.pxl").read_text().rstrip() for name in SAMPLES]
    if blocks != sources:
        raise ValueError("Every README JSON5 example needs a declared source/preview section")
    return expected


def render_readme_assets(renderer: str, check: bool) -> None:
    version = subprocess.check_output([renderer, "--version"], text=True).strip()
    if version != f"pxl {VERSION}":
        raise ValueError(f"README assets require released pxl {VERSION}; found {version}")
    readme = ROOT / "README.md"
    current = readme.read_text()
    expected = expected_readme(current)
    stale = ["README.md"] if expected != current else []

    with tempfile.TemporaryDirectory(prefix="pixelsrc-readme-") as directory:
        for name, renderable, selector, scale in ASSETS:
            filename = image_name(name, scale)
            output = Path(directory) / filename
            command = [
                renderer, "render", str(ART / f"{name}.pxl"), "--strict",
                selector, renderable, "--scale", str(scale), "-o", str(output),
            ]
            subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True)
            destination = ROOT / ART / filename
            if check:
                if not destination.exists() or destination.read_bytes() != output.read_bytes():
                    stale.append(str(ART / filename))
            else:
                shutil.copyfile(output, destination)

    if check and stale:
        raise ValueError("Stale README assets: " + ", ".join(stale) + ". Run just readme-assets.")
    if not check:
        readme.write_text(expected)
    print(f"README logo and {len(SAMPLES)} examples {'verified' if check else 'regenerated'} with {version}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pxl", default="pxl", help=f"path to released pxl {VERSION}")
    parser.add_argument("--check", action="store_true", help="fail on stale PNGs or README source")
    args = parser.parse_args()
    try:
        render_readme_assets(args.pxl, args.check)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"README asset check failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
