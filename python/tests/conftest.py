"""Shared fixtures for pixelsrc Python tests."""

import pytest


@pytest.fixture
def minimal_sprite():
    """A minimal 1x1 red pixel sprite."""
    return '{ type: "sprite", name: "dot", size: [1, 1], palette: { "_": "#00000000", "x": "#FF0000" }, regions: { "x": { points: [[0, 0]], z: 0 } } }'


@pytest.fixture
def heart_with_palette():
    """A 4x4 heart sprite with a named palette."""
    return """\
{ type: "palette", name: "reds", colors: { "_": "#00000000", "r": "#FF0000", "p": "#FF6B6B" } }
{ type: "sprite", name: "heart", size: [4, 4], palette: "reds", regions: { "r": { points: [[1,0],[2,0],[0,1],[1,1],[2,1],[3,1],[1,2],[2,2],[2,3]], z: 0 } } }"""


@pytest.fixture
def empty_sprite():
    """An 8x8 sprite with no regions."""
    return '{ type: "sprite", name: "empty", width: 8, height: 8, regions: [] }'
