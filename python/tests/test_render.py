"""Tests for stateless rendering functions (PY-3)."""

import pixelsrc


def test_render_to_png(minimal_sprite):
    """render_to_png returns valid PNG bytes for a simple sprite."""
    result = pixelsrc.render_to_png(minimal_sprite)
    assert isinstance(result, bytes)
    assert len(result) > 0
    # PNG magic bytes
    assert result[:4] == b"\x89PNG"


def test_render_to_png_no_sprites():
    """render_to_png returns empty bytes when no sprites are present."""
    result = pixelsrc.render_to_png(
        '{"type": "palette", "name": "empty", "colors": {}}'
    )
    assert isinstance(result, bytes)
    assert len(result) == 0


def test_render_to_rgba(minimal_sprite):
    """render_to_rgba returns a RenderResult with correct dimensions and data."""
    result = pixelsrc.render_to_rgba(minimal_sprite)
    assert isinstance(result, pixelsrc.RenderResult)
    assert result.width == 1
    assert result.height == 1
    assert isinstance(result.pixels, bytes)
    assert len(result.pixels) == 4  # 1 pixel * 4 bytes (RGBA)
    # Red pixel: #FF0000 = R=255, G=0, B=0, A=255
    assert result.pixels[0] == 255  # R
    assert result.pixels[1] == 0    # G
    assert result.pixels[2] == 0    # B
    assert result.pixels[3] == 255  # A


def test_render_to_rgba_no_sprites():
    """render_to_rgba returns empty result with warning when no sprites found."""
    result = pixelsrc.render_to_rgba(
        '{"type": "palette", "name": "empty", "colors": {}}'
    )
    assert result.width == 0
    assert result.height == 0
    assert len(result.pixels) == 0
    assert any("No sprites" in w for w in result.warnings)


def test_render_to_rgba_heart(heart_with_palette):
    """render_to_rgba renders a multi-pixel sprite with a named palette."""
    result = pixelsrc.render_to_rgba(heart_with_palette)
    assert result.width == 4
    assert result.height == 4
    assert len(result.pixels) == 4 * 4 * 4  # 4x4 pixels * 4 bytes
    assert isinstance(result.warnings, list)


def test_render_to_png_heart(heart_with_palette):
    """render_to_png produces valid PNG for a named-palette sprite."""
    result = pixelsrc.render_to_png(heart_with_palette)
    assert len(result) > 0
    assert result[:4] == b"\x89PNG"
