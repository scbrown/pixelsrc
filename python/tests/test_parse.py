"""Tests for pixelsrc parsing and listing functions (PY-4)."""

from pixelsrc import list_palettes, list_sprites, parse


class TestListSprites:
    def test_single_sprite(self, minimal_sprite):
        names = list_sprites(minimal_sprite)
        assert names == ["dot"]

    def test_sprite_from_multi_object(self, heart_with_palette):
        names = list_sprites(heart_with_palette)
        assert names == ["heart"]

    def test_no_sprites(self):
        pxl = '{ type: "palette", name: "pal", colors: { "_": "#000" } }'
        assert list_sprites(pxl) == []

    def test_multiple_sprites(self):
        pxl = """\
{ type: "palette", name: "p", colors: { "_": "#00000000", "x": "#FF0000" } }
{ type: "sprite", name: "a", size: [1, 1], palette: "p", regions: { "x": { points: [[0, 0]], z: 0 } } }
{ type: "sprite", name: "b", size: [1, 1], palette: "p", regions: { "x": { points: [[0, 0]], z: 0 } } }"""
        names = list_sprites(pxl)
        assert names == ["a", "b"]

    def test_ignores_variants(self):
        pxl = """\
{ type: "palette", name: "p", colors: { "_": "#00000000", "x": "#FF0000" } }
{ type: "sprite", name: "base", size: [1, 1], palette: "p", regions: { "x": { points: [[0, 0]], z: 0 } } }
{ type: "variant", name: "alt", base: "base", palette: { "x": "#00FF00" } }"""
        names = list_sprites(pxl)
        assert names == ["base"]


class TestListPalettes:
    def test_single_palette(self):
        pxl = '{ type: "palette", name: "mono", colors: { "_": "#000" } }'
        assert list_palettes(pxl) == ["mono"]

    def test_palette_from_multi_object(self, heart_with_palette):
        names = list_palettes(heart_with_palette)
        assert names == ["reds"]

    def test_no_palettes(self, minimal_sprite):
        assert list_palettes(minimal_sprite) == []

    def test_multiple_palettes(self):
        pxl = """\
{ type: "palette", name: "warm", colors: { "r": "#FF0000" } }
{ type: "palette", name: "cool", colors: { "b": "#0000FF" } }"""
        names = list_palettes(pxl)
        assert names == ["warm", "cool"]


class TestParse:
    def test_parse_palette(self):
        pxl = '{ type: "palette", name: "mono", colors: { "_": "#000", "x": "#FFF" } }'
        result = parse(pxl)
        assert len(result) == 1
        obj = result[0]
        assert obj["type"] == "palette"
        assert obj["name"] == "mono"
        assert obj["colors"]["_"] == "#000"
        assert obj["colors"]["x"] == "#FFF"

    def test_parse_sprite(self, minimal_sprite):
        result = parse(minimal_sprite)
        assert len(result) == 1
        obj = result[0]
        assert obj["type"] == "sprite"
        assert obj["name"] == "dot"
        assert obj["size"] == [1, 1]

    def test_parse_multi_object(self, heart_with_palette):
        result = parse(heart_with_palette)
        assert len(result) == 2
        assert result[0]["type"] == "palette"
        assert result[0]["name"] == "reds"
        assert result[1]["type"] == "sprite"
        assert result[1]["name"] == "heart"

    def test_parse_variant(self):
        pxl = """\
{ type: "palette", name: "p", colors: { "_": "#00000000", "x": "#FF0000" } }
{ type: "sprite", name: "base", size: [1, 1], palette: "p", regions: { "x": { points: [[0, 0]], z: 0 } } }
{ type: "variant", name: "alt", base: "base", palette: { "x": "#00FF00" } }"""
        result = parse(pxl)
        assert len(result) == 3
        assert result[2]["type"] == "variant"
        assert result[2]["name"] == "alt"
        assert result[2]["base"] == "base"

    def test_parse_returns_warnings_for_invalid_input(self):
        pxl = "{invalid json}"
        result = parse(pxl)
        assert len(result) == 1
        assert "warning" in result[0]
        assert "line" in result[0]

    def test_parse_empty_string(self):
        result = parse("")
        assert result == []

    def test_parse_sprite_regions_structure(self):
        pxl = '{ type: "sprite", name: "test", size: [4, 4], palette: { "_": "#0000", "a": "#FF00" }, regions: { "a": { rect: [0, 0, 2, 2], z: 0 } } }'
        result = parse(pxl)
        assert len(result) == 1
        sprite = result[0]
        assert "regions" in sprite
        assert "a" in sprite["regions"]
        assert sprite["regions"]["a"]["z"] == 0
