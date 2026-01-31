"""Tests for pixelsrc package metadata."""

import pixelsrc


def test_version_exists():
    assert hasattr(pixelsrc, "__version__")


def test_version_is_string():
    assert isinstance(pixelsrc.__version__, str)


def test_version_format():
    parts = pixelsrc.__version__.split(".")
    assert len(parts) >= 2
    assert all(p.isdigit() for p in parts)
