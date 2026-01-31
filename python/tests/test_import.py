"""Tests for PNG import and analysis roundtrips (PY-7)."""

import pytest

import pixelsrc


class TestImportPng:
    def test_basic_import(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        assert isinstance(result, pixelsrc.ImportResult)

    def test_name_defaults_to_filename(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        assert result.name == "dot"

    def test_custom_name(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png), name="custom")
        assert result.name == "custom"

    def test_dimensions(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        assert result.width == 1
        assert result.height == 1

    def test_heart_dimensions(self, tmp_heart_png):
        result = pixelsrc.import_png(str(tmp_heart_png))
        assert result.width == 4
        assert result.height == 4

    def test_palette_is_dict(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        assert isinstance(result.palette, dict)
        assert len(result.palette) > 0

    def test_palette_values_are_hex(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        for token, color in result.palette.items():
            assert isinstance(token, str)
            assert isinstance(color, str)
            assert color.startswith("#")

    def test_max_colors(self, tmp_heart_png):
        result = pixelsrc.import_png(str(tmp_heart_png), max_colors=4)
        assert isinstance(result, pixelsrc.ImportResult)
        assert len(result.palette) <= 4

    def test_nonexistent_file_raises(self):
        with pytest.raises(OSError):
            pixelsrc.import_png("/nonexistent/path/sprite.png")

    def test_analysis_is_none_without_analyzed(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        assert result.analysis is None


class TestImportPngAnalyzed:
    def test_basic_analyzed(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(str(tmp_heart_png))
        assert isinstance(result, pixelsrc.ImportResult)

    def test_analysis_present(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(str(tmp_heart_png))
        analysis = result.analysis
        assert analysis is not None
        assert isinstance(analysis, dict)

    def test_analysis_has_expected_keys(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(str(tmp_heart_png))
        analysis = result.analysis
        expected_keys = {
            "roles", "relationships", "symmetry",
            "naming_hints", "z_order", "dither_patterns",
            "upscale_info", "outlines",
        }
        assert expected_keys == set(analysis.keys())

    def test_analysis_roles_is_dict(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(str(tmp_heart_png))
        roles = result.analysis["roles"]
        assert isinstance(roles, dict)

    def test_custom_name_analyzed(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(str(tmp_heart_png), name="my_heart")
        assert result.name == "my_heart"

    def test_custom_max_colors(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(str(tmp_heart_png), max_colors=4)
        assert len(result.palette) <= 4

    def test_nonexistent_file_raises(self):
        with pytest.raises(OSError):
            pixelsrc.import_png_analyzed("/nonexistent/path/sprite.png")

    def test_invalid_dither_handling_raises(self, tmp_heart_png):
        with pytest.raises(ValueError):
            pixelsrc.import_png_analyzed(
                str(tmp_heart_png), dither_handling="invalid"
            )

    def test_valid_dither_handling_keep(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(
            str(tmp_heart_png), dither_handling="keep"
        )
        assert isinstance(result, pixelsrc.ImportResult)

    def test_valid_dither_handling_merge(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(
            str(tmp_heart_png), dither_handling="merge"
        )
        assert isinstance(result, pixelsrc.ImportResult)

    def test_valid_dither_handling_analyze(self, tmp_heart_png):
        result = pixelsrc.import_png_analyzed(
            str(tmp_heart_png), dither_handling="analyze"
        )
        assert isinstance(result, pixelsrc.ImportResult)


class TestImportResultOutput:
    def test_to_pxl_returns_string(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        pxl = result.to_pxl()
        assert isinstance(pxl, str)
        assert len(pxl) > 0

    def test_to_pxl_contains_palette(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        pxl = result.to_pxl()
        assert "palette" in pxl

    def test_to_pxl_contains_sprite(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        pxl = result.to_pxl()
        assert "sprite" in pxl

    def test_to_jsonl_returns_string(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        jsonl = result.to_jsonl()
        assert isinstance(jsonl, str)
        assert len(jsonl) > 0

    def test_to_jsonl_is_parseable(self, tmp_png):
        result = pixelsrc.import_png(str(tmp_png))
        jsonl = result.to_jsonl()
        parsed = pixelsrc.parse(jsonl)
        assert len(parsed) >= 1

    def test_to_jsonl_has_palette_and_sprite(self, tmp_heart_png):
        result = pixelsrc.import_png(str(tmp_heart_png))
        jsonl = result.to_jsonl()
        parsed = pixelsrc.parse(jsonl)
        types = [obj["type"] for obj in parsed]
        assert "palette" in types
        assert "sprite" in types


class TestImportRoundtrip:
    def test_render_import_dimensions_match(self, minimal_sprite, tmp_png):
        """Rendering a sprite and re-importing should preserve dimensions."""
        imported = pixelsrc.import_png(str(tmp_png))
        assert imported.width == 1
        assert imported.height == 1

    def test_render_import_heart_dimensions(self, tmp_heart_png):
        """Larger sprite roundtrip preserves dimensions."""
        imported = pixelsrc.import_png(str(tmp_heart_png))
        assert imported.width == 4
        assert imported.height == 4

    def test_jsonl_roundtrip_renders(self, tmp_heart_png, tmp_path):
        """Imported JSONL can be rendered back to PNG."""
        imported = pixelsrc.import_png(str(tmp_heart_png))
        jsonl = imported.to_jsonl()
        png = pixelsrc.render_to_png(jsonl)
        assert isinstance(png, bytes)
        assert png[:4] == b"\x89PNG"
