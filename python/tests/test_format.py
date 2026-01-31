"""Tests for pixelsrc formatting function (PY-9)."""

import pytest

from pixelsrc import format_pxl


class TestFormatPxl:
    def test_formats_compact_palette(self):
        pxl = '{"type":"palette","name":"pal","colors":{"x":"#FF0000","_":"#00000000"}}'
        result = format_pxl(pxl)
        assert '"type": "palette"' in result
        assert '"name": "pal"' in result
        assert '"colors"' in result

    def test_formats_compact_sprite(self):
        pxl = '{"type":"sprite","name":"dot","size":[1,1],"palette":"pal","regions":{"x":{"points":[[0,0]],"z":0}}}'
        result = format_pxl(pxl)
        assert '"type": "sprite"' in result
        assert '"name": "dot"' in result
        assert '"size": [1, 1]' in result

    def test_formats_multi_object(self):
        pxl = (
            '{"type":"palette","name":"p","colors":{"x":"#FF0000"}}'
            '{"type":"sprite","name":"s","size":[2,2],"palette":"p",'
            '"regions":{"x":{"rect":[0,0,2,2],"z":0}}}'
        )
        result = format_pxl(pxl)
        # Should have blank line between objects
        assert "\n\n" in result
        assert '"type": "palette"' in result
        assert '"type": "sprite"' in result

    def test_returns_string(self):
        pxl = '{"type":"palette","name":"p","colors":{"a":"#FFF"}}'
        result = format_pxl(pxl)
        assert isinstance(result, str)

    def test_output_is_valid_and_reparseable(self):
        """Formatted output should be parseable by parse()."""
        from pixelsrc import parse

        pxl = '{"type":"palette","name":"p","colors":{"x":"#FF0000"}}'
        formatted = format_pxl(pxl)
        result = parse(formatted)
        assert len(result) == 1
        assert result[0]["type"] == "palette"
        assert result[0]["name"] == "p"

    def test_empty_input(self):
        result = format_pxl("")
        assert result == ""

    def test_invalid_input_raises(self):
        with pytest.raises(ValueError):
            format_pxl("{invalid json content that is not valid")

    def test_formats_variant(self):
        pxl = '{"type":"variant","name":"alt","base":"hero","palette":{"x":"#00FF00"}}'
        result = format_pxl(pxl)
        assert '"type": "variant"' in result
        assert '"name": "alt"' in result
        assert '"base": "hero"' in result

    def test_formats_animation(self):
        pxl = '{"type":"animation","name":"walk","frames":["f1","f2"],"duration":100}'
        result = format_pxl(pxl)
        assert '"type": "animation"' in result
        assert '"frames": ["f1", "f2"]' in result
