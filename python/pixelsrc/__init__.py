"""Pixelsrc - Semantic pixel art format and compiler.

Native Python bindings for the pixelsrc pixel art format and compiler.
Provides parsing, rendering, validation, and PNG import via PyO3 FFI.
"""

__version__ = "0.2.0"

from pixelsrc._native import *  # noqa: F401, F403
