"""Tests for pixelsrc parsing and listing functions (PY-4)."""

from pixelsrc import list_sprites, list_palettes, parse


class TestListSprites:
    def test_single_sprite(self):
        pxl = '{"type": "sprite", "name": "hero", "size": [8, 8], "palette": "pal", "regions": {"x": {"points": [[0, 0]], "z": 0}}}'
        assert list_sprites(pxl) == ["hero"]

    def test_multiple_sprites(self):
        pxl = (
            '{"type": "sprite", "name": "a", "size": [1, 1], "palette": {"x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}\n'
            '{"type": "sprite", "name": "b", "size": [1, 1], "palette": {"x": "#00FF00"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}'
        )
        assert list_sprites(pxl) == ["a", "b"]

    def test_no_sprites(self):
        pxl = '{"type": "palette", "name": "pal", "colors": {}}'
        assert list_sprites(pxl) == []

    def test_mixed_objects(self, heart_with_palette):
        names = list_sprites(heart_with_palette)
        assert names == ["heart"]

    def test_empty_input(self):
        assert list_sprites("") == []


class TestListPalettes:
    def test_single_palette(self):
        pxl = '{"type": "palette", "name": "warm", "colors": {"x": "#FF0000"}}'
        assert list_palettes(pxl) == ["warm"]

    def test_multiple_palettes(self):
        pxl = (
            '{"type": "palette", "name": "warm", "colors": {"x": "#FF0000"}}\n'
            '{"type": "palette", "name": "cool", "colors": {"y": "#0000FF"}}'
        )
        assert list_palettes(pxl) == ["warm", "cool"]

    def test_no_palettes(self):
        pxl = '{"type": "sprite", "name": "dot", "size": [1, 1], "palette": {"x": "#FF0000"}, "regions": {"x": {"points": [[0, 0]], "z": 0}}}'
        assert list_palettes(pxl) == []

    def test_mixed_objects(self, heart_with_palette):
        names = list_palettes(heart_with_palette)
        assert names == ["reds"]

    def test_empty_input(self):
        assert list_palettes("") == []


class TestParse:
    def test_single_palette(self):
        pxl = '{"type": "palette", "name": "test", "colors": {"x": "#FF0000"}}'
        objs = parse(pxl)
        assert len(objs) == 1
        assert objs[0]["type"] == "palette"
        assert objs[0]["name"] == "test"
        assert objs[0]["colors"]["x"] == "#FF0000"

    def test_single_sprite(self):
        pxl = '{"type": "sprite", "name": "dot", "size": [1, 1], "palette": "colors", "regions": {"x": {"points": [[0, 0]], "z": 0}}}'
        objs = parse(pxl)
        assert len(objs) == 1
        assert objs[0]["type"] == "sprite"
        assert objs[0]["name"] == "dot"
        assert objs[0]["size"] == [1, 1]

    def test_multiple_objects(self, heart_with_palette):
        objs = parse(heart_with_palette)
        assert len(objs) == 2
        assert objs[0]["type"] == "palette"
        assert objs[0]["name"] == "reds"
        assert objs[1]["type"] == "sprite"
        assert objs[1]["name"] == "heart"

    def test_parse_warnings(self):
        pxl = '{"type": "palette", "name": "ok", "colors": {}}\n{bad json}'
        objs = parse(pxl)
        # First object parses fine, second produces a warning
        palette_objs = [o for o in objs if "type" in o]
        warning_objs = [o for o in objs if "warning" in o]
        assert len(palette_objs) == 1
        assert len(warning_objs) == 1
        assert "line" in warning_objs[0]

    def test_empty_input(self):
        assert parse("") == []

    def test_variant_object(self):
        pxl = (
            '{"type": "palette", "name": "p", "colors": {"x": "#FF0000"}}\n'
            '{"type": "sprite", "name": "base", "size": [1, 1], "palette": "p", "regions": {"x": {"points": [[0, 0]], "z": 0}}}\n'
            '{"type": "variant", "name": "alt", "base": "base", "palette": {"x": "#00FF00"}}'
        )
        objs = parse(pxl)
        assert len(objs) == 3
        assert objs[2]["type"] == "variant"
        assert objs[2]["name"] == "alt"
        assert objs[2]["base"] == "base"
