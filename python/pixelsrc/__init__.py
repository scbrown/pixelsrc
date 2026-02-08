"""Pixelsrc - Semantic pixel art format and compiler.

Native Python bindings for the pixelsrc pixel art format and compiler.
Provides parsing, rendering, validation, and PNG import via PyO3 FFI.
"""

__version__ = "0.2.0"

from pixelsrc._native import *  # noqa: F401, F403
from pixelsrc._native import RenderResult


def _render_result_to_numpy(self):
    """Convert RGBA pixel data to a NumPy array with shape (height, width, 4).

    Requires the ``numpy`` package (``pip install pixelsrc[numpy]``).

    Returns:
        numpy.ndarray: Array of dtype ``uint8`` with shape ``(height, width, 4)``.
    """
    import numpy as np

    arr = np.frombuffer(self.pixels, dtype=np.uint8)
    return arr.reshape((self.height, self.width, 4)).copy()


def _render_result_to_pil(self):
    """Convert RGBA pixel data to a PIL Image.

    Requires the ``Pillow`` package (``pip install pixelsrc[images]``).

    Returns:
        PIL.Image.Image: An RGBA image.
    """
    from PIL import Image

    return Image.frombytes("RGBA", (self.width, self.height), self.pixels)


RenderResult.to_numpy = _render_result_to_numpy
RenderResult.to_pil = _render_result_to_pil
