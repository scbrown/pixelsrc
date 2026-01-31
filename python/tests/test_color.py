"""Tests for color parsing and ramp generation (PY-6)."""

import pytest

from pixelsrc import generate_ramp, parse_color


class TestParseColor:
    def test_named_red(self):
        assert parse_color("red") == "#ff0000"

    def test_named_blue(self):
        assert parse_color("blue") == "#0000ff"

    def test_named_white(self):
        assert parse_color("white") == "#ffffff"

    def test_named_black(self):
        assert parse_color("black") == "#000000"

    def test_hex_six_digit(self):
        assert parse_color("#FF0000") == "#ff0000"

    def test_hex_three_digit(self):
        assert parse_color("#F00") == "#ff0000"

    def test_hex_eight_digit_alpha(self):
        assert parse_color("#00ff0080") == "#00ff0080"

    def test_rgb_functional(self):
        assert parse_color("rgb(255, 0, 0)") == "#ff0000"

    def test_hsl_functional(self):
        assert parse_color("hsl(0, 100%, 50%)") == "#ff0000"

    def test_transparent(self):
        assert parse_color("transparent") == "#00000000"

    def test_returns_lowercase(self):
        result = parse_color("#AABBCC")
        assert result == "#aabbcc"
        assert result == result.lower()

    def test_opaque_uses_six_digits(self):
        result = parse_color("#ff0000")
        assert len(result) == 7  # # + 6 hex chars

    def test_alpha_uses_eight_digits(self):
        result = parse_color("#ff000080")
        assert len(result) == 9  # # + 8 hex chars

    def test_invalid_color_raises(self):
        with pytest.raises(ValueError):
            parse_color("notacolor")

    def test_empty_string_raises(self):
        with pytest.raises(ValueError):
            parse_color("")


class TestGenerateRamp:
    def test_basic_black_to_white(self):
        ramp = generate_ramp("#000000", "#ffffff", 5)
        assert len(ramp) == 5
        assert ramp[0] == "#000000"
        assert ramp[4] == "#ffffff"

    def test_single_step_returns_from_color(self):
        ramp = generate_ramp("#ff0000", "#0000ff", 1)
        assert ramp == ["#ff0000"]

    def test_two_steps_returns_endpoints(self):
        ramp = generate_ramp("#000000", "#ffffff", 2)
        assert ramp == ["#000000", "#ffffff"]

    def test_midpoint_interpolation(self):
        ramp = generate_ramp("#000000", "#ffffff", 3)
        assert ramp[1] == "#808080"

    def test_zero_steps_raises(self):
        with pytest.raises(ValueError):
            generate_ramp("#000000", "#ffffff", 0)

    def test_invalid_from_color_raises(self):
        with pytest.raises(ValueError):
            generate_ramp("notacolor", "#ffffff", 3)

    def test_invalid_to_color_raises(self):
        with pytest.raises(ValueError):
            generate_ramp("#000000", "notacolor", 3)

    def test_named_colors(self):
        ramp = generate_ramp("black", "white", 3)
        assert len(ramp) == 3
        assert ramp[0] == "#000000"
        assert ramp[2] == "#ffffff"

    def test_alpha_interpolation(self):
        ramp = generate_ramp("#ff000000", "#ff0000ff", 3)
        assert ramp[0] == "#ff000000"
        assert ramp[1] == "#ff000080"
        # Full alpha (255) collapses to 6-digit form
        assert ramp[2] == "#ff0000"

    def test_all_elements_are_strings(self):
        ramp = generate_ramp("red", "blue", 4)
        assert all(isinstance(c, str) for c in ramp)

    def test_all_elements_start_with_hash(self):
        ramp = generate_ramp("red", "blue", 4)
        assert all(c.startswith("#") for c in ramp)

    def test_red_to_blue_channel_transition(self):
        ramp = generate_ramp("#ff0000", "#0000ff", 3)
        assert ramp[0] == "#ff0000"
        assert ramp[2] == "#0000ff"
        # Midpoint: r=128, g=0, b=128
        assert ramp[1] == "#800080"

    def test_same_color_endpoints(self):
        ramp = generate_ramp("#abcdef", "#abcdef", 3)
        assert all(c == "#abcdef" for c in ramp)
