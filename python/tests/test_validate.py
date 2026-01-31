"""Tests for pixelsrc validation functions (PY-5)."""

import os
import tempfile

from pixelsrc import validate, validate_file


class TestValidate:
    def test_valid_sprite_no_issues(self, minimal_sprite):
        messages = validate(minimal_sprite)
        assert messages == []

    def test_valid_multi_object(self, heart_with_palette):
        messages = validate(heart_with_palette)
        assert messages == []

    def test_empty_input(self):
        assert validate("") == []

    def test_invalid_json(self):
        messages = validate("{not valid json}")
        assert len(messages) == 1
        assert "ERROR" in messages[0]

    def test_missing_type_field(self):
        messages = validate('{"name": "test"}')
        assert len(messages) == 1
        assert "ERROR" in messages[0]
        assert "type" in messages[0]

    def test_unknown_type(self):
        messages = validate('{"type": "foobar", "name": "test"}')
        assert len(messages) == 1
        assert "WARNING" in messages[0]
        assert "foobar" in messages[0]

    def test_invalid_color(self):
        pxl = '{"type": "palette", "name": "bad", "colors": {"x": "#GGG"}}'
        messages = validate(pxl)
        assert len(messages) >= 1
        assert any("ERROR" in m and "color" in m.lower() for m in messages)

    def test_undefined_token(self):
        pxl = """\
{"type": "palette", "name": "p", "colors": {"a": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"b": {"rect": [0, 0, 4, 4]}}}"""
        messages = validate(pxl)
        assert any("WARNING" in m and "Undefined token" in m for m in messages)

    def test_missing_palette(self):
        pxl = '{"type": "sprite", "name": "s", "size": [4, 4], "palette": "nonexistent", "regions": {"a": {"rect": [0, 0, 4, 4]}}}'
        messages = validate(pxl)
        assert any("WARNING" in m and "not defined" in m for m in messages)

    def test_duplicate_palette_name(self):
        pxl = """\
{"type": "palette", "name": "dup", "colors": {"a": "#FF0000"}}
{"type": "palette", "name": "dup", "colors": {"b": "#00FF00"}}"""
        messages = validate(pxl)
        assert any("Duplicate" in m for m in messages)

    def test_returns_list_of_strings(self):
        messages = validate("{bad}")
        assert isinstance(messages, list)
        assert all(isinstance(m, str) for m in messages)

    def test_line_numbers_in_messages(self):
        pxl = """\
{"type": "palette", "name": "p", "colors": {"a": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"b": {"rect": [0, 0, 4, 4]}}}"""
        messages = validate(pxl)
        assert any("line 2:" in m for m in messages)

    def test_multiline_json5(self):
        pxl = """{
  type: "palette",
  name: "test",
  colors: {
    "x": "#FF0000"
  }
}"""
        messages = validate(pxl)
        assert messages == []


class TestValidateFile:
    def test_valid_file(self, minimal_sprite):
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".pxl", delete=False
        ) as f:
            f.write(minimal_sprite)
            f.flush()
            try:
                messages = validate_file(f.name)
                assert messages == []
            finally:
                os.unlink(f.name)

    def test_invalid_file(self):
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".pxl", delete=False
        ) as f:
            f.write("{not valid json}")
            f.flush()
            try:
                messages = validate_file(f.name)
                assert len(messages) >= 1
                assert any("ERROR" in m for m in messages)
            finally:
                os.unlink(f.name)

    def test_nonexistent_file(self):
        try:
            validate_file("/nonexistent/path/to/file.pxl")
            assert False, "Expected OSError"
        except OSError:
            pass

    def test_file_with_multiple_issues(self):
        pxl = """\
{"type": "palette", "name": "p", "colors": {"a": "#FF0000"}}
{"type": "sprite", "name": "s", "size": [4, 4], "palette": "p", "regions": {"b": {"rect": [0, 0, 4, 4]}}}
{"type": "sprite", "name": "s2", "size": [4, 4], "palette": "missing", "regions": {"x": {"rect": [0, 0, 4, 4]}}}"""
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".pxl", delete=False
        ) as f:
            f.write(pxl)
            f.flush()
            try:
                messages = validate_file(f.name)
                assert len(messages) >= 2
            finally:
                os.unlink(f.name)

    def test_returns_list_of_strings(self):
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".pxl", delete=False
        ) as f:
            f.write("{bad}")
            f.flush()
            try:
                messages = validate_file(f.name)
                assert isinstance(messages, list)
                assert all(isinstance(m, str) for m in messages)
            finally:
                os.unlink(f.name)
