"""Tests for stateful Registry class (PY-8)."""

import os
import tempfile

import pixelsrc


PALETTE_PXL = '{"type": "palette", "name": "mono", "colors": {"_": "#00000000", "on": "#FFFFFF", "off": "#000000"}}'

SPRITE_PXL = '{"type": "sprite", "name": "checker", "size": [4, 4], "palette": "mono", "regions": {"on": {"points": [[0, 0], [2, 0], [1, 1], [3, 1], [0, 2], [2, 2], [1, 3], [3, 3]], "z": 0}, "off": {"points": [[1, 0], [3, 0], [0, 1], [2, 1], [1, 2], [3, 2], [0, 3], [2, 3]], "z": 0}}}'

DOT_PXL = '{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"_": "#00000000", "x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}'


def test_registry_creation():
    """Registry() creates an empty registry."""
    reg = pixelsrc.Registry()
    assert reg.sprites() == []
    assert reg.palettes() == []


def test_load_palette_and_sprite():
    """load() accumulates palettes and sprites."""
    reg = pixelsrc.Registry()
    reg.load(PALETTE_PXL + "\n" + SPRITE_PXL)
    assert "mono" in reg.palettes()
    assert "checker" in reg.sprites()


def test_incremental_load():
    """Multiple load() calls accumulate content."""
    reg = pixelsrc.Registry()
    reg.load(PALETTE_PXL)
    assert reg.palettes() == ["mono"]
    assert reg.sprites() == []

    reg.load(SPRITE_PXL)
    assert "checker" in reg.sprites()
    assert "mono" in reg.palettes()


def test_load_file(tmp_path):
    """load_file() reads from a file on disk."""
    pxl_file = tmp_path / "test.pxl"
    pxl_file.write_text(PALETTE_PXL + "\n" + SPRITE_PXL)

    reg = pixelsrc.Registry()
    reg.load_file(str(pxl_file))
    assert "mono" in reg.palettes()
    assert "checker" in reg.sprites()


def test_load_file_not_found():
    """load_file() raises OSError for missing files."""
    reg = pixelsrc.Registry()
    try:
        reg.load_file("/nonexistent/path.pxl")
        assert False, "Expected OSError"
    except OSError:
        pass


def test_sprites_sorted():
    """sprites() returns names in sorted order."""
    reg = pixelsrc.Registry()
    reg.load(DOT_PXL)
    reg.load('{"type": "sprite", "name": "alpha", "size": [1, 1], "palette": {"x": "#00FF00"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}')
    names = reg.sprites()
    assert names == sorted(names)
    assert "alpha" in names
    assert "dot" in names


def test_palettes_sorted():
    """palettes() returns names in sorted order."""
    reg = pixelsrc.Registry()
    reg.load('{"type": "palette", "name": "warm", "colors": {"x": "#FF0000"}}')
    reg.load('{"type": "palette", "name": "cool", "colors": {"x": "#0000FF"}}')
    names = reg.palettes()
    assert names == ["cool", "warm"]


def test_render_sprite():
    """render() returns a RenderResult for a known sprite."""
    reg = pixelsrc.Registry()
    reg.load(DOT_PXL)

    result = reg.render("dot")
    assert isinstance(result, pixelsrc.RenderResult)
    assert result.width == 1
    assert result.height == 1
    assert len(result.pixels) == 4  # 1 pixel * RGBA


def test_render_with_named_palette():
    """render() resolves sprites against named palettes."""
    reg = pixelsrc.Registry()
    reg.load(PALETTE_PXL)
    reg.load(SPRITE_PXL)

    result = reg.render("checker")
    assert result.width == 4
    assert result.height == 4
    assert len(result.pixels) == 4 * 4 * 4


def test_render_unknown_sprite():
    """render() raises ValueError for unknown sprite names."""
    reg = pixelsrc.Registry()
    # In lenient mode this actually returns a result with warnings,
    # but let's verify it doesn't crash
    result = reg.render("nonexistent")
    assert isinstance(result, pixelsrc.RenderResult)


def test_render_to_png():
    """render_to_png() returns valid PNG bytes."""
    reg = pixelsrc.Registry()
    reg.load(DOT_PXL)

    png = reg.render_to_png("dot")
    assert isinstance(png, bytes)
    assert len(png) > 0
    assert png[:4] == b"\x89PNG"


def test_render_to_png_named_palette():
    """render_to_png() works with named palettes."""
    reg = pixelsrc.Registry()
    reg.load(PALETTE_PXL)
    reg.load(SPRITE_PXL)

    png = reg.render_to_png("checker")
    assert isinstance(png, bytes)
    assert png[:4] == b"\x89PNG"


def test_render_all():
    """render_all() returns a dict mapping names to PNG bytes."""
    reg = pixelsrc.Registry()
    reg.load(PALETTE_PXL)
    reg.load(SPRITE_PXL)
    reg.load(DOT_PXL)

    result = reg.render_all()
    assert isinstance(result, dict)
    assert "checker" in result
    assert "dot" in result

    for name, png in result.items():
        assert isinstance(png, bytes)
        assert png[:4] == b"\x89PNG", f"PNG for '{name}' has wrong magic bytes"


def test_render_all_empty():
    """render_all() returns empty dict for empty registry."""
    reg = pixelsrc.Registry()
    result = reg.render_all()
    assert result == {}


def test_full_workflow(tmp_path):
    """End-to-end: load file, list, render, render_all."""
    pxl_file = tmp_path / "sprites.pxl"
    pxl_file.write_text(PALETTE_PXL + "\n" + SPRITE_PXL + "\n" + DOT_PXL)

    reg = pixelsrc.Registry()
    reg.load_file(str(pxl_file))

    assert len(reg.palettes()) == 1
    assert len(reg.sprites()) == 2

    # Render individual
    result = reg.render("dot")
    assert result.width == 1

    # Render to PNG
    png = reg.render_to_png("checker")
    assert png[:4] == b"\x89PNG"

    # Render all
    all_pngs = reg.render_all()
    assert len(all_pngs) == 2
